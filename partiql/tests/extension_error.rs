use std::any::Any;
use std::borrow::Cow;

use std::error::Error;

use thiserror::Error;

use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::context::{SessionContext, SystemContext};
use partiql_catalog::{
    BaseTableExpr, BaseTableExprResult, BaseTableExprResultError, BaseTableFunctionInfo, Catalog,
    Extension, PartiqlCatalog, TableFunction,
};
use partiql_eval::env::basic::MapBindings;
use partiql_eval::error::{EvalErr, EvaluationError};
use partiql_eval::eval::{BasicContext, Evaluated};
use partiql_eval::plan::EvaluationMode;
use partiql_parser::{Parsed, ParserResult};
use partiql_value::{bag, tuple, DateTime, Value};

use partiql_logical as logical;

#[derive(Debug)]
pub struct UserCtxTestExtension {}

impl partiql_catalog::Extension for UserCtxTestExtension {
    fn name(&self) -> String {
        "test_extension".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), Box<dyn Error>> {
        match catalog
            .add_table_function(TableFunction::new(Box::new(TestUserContextFunction::new())))
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e) as Box<dyn Error>),
        }
    }
}

#[derive(Debug)]
pub(crate) struct TestUserContextFunction {
    call_def: CallDef,
}

impl TestUserContextFunction {
    pub fn new() -> Self {
        TestUserContextFunction {
            call_def: CallDef {
                names: vec!["test_user_context"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        logical::ValueExpr::Call(logical::CallExpr {
                            name: logical::CallName::ByName("test_user_context".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for TestUserContextFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(EvalTestCtxTable {})
    }
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum UserCtxError {
    #[error("bad arguments")]
    BadArgs,
    #[error("runtime error")]
    Runtime,
}

#[derive(Debug)]
pub(crate) struct EvalTestCtxTable {}

impl BaseTableExpr for EvalTestCtxTable {
    fn evaluate<'c>(
        &self,
        args: &[Cow<Value>],
        _ctx: &'c dyn SessionContext<'c>,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(_name) => Ok(Box::new(TestDataGen {})),
                _ => {
                    let error = UserCtxError::BadArgs;
                    Err(Box::new(error) as BaseTableExprResultError)
                }
            }
        } else {
            let error = UserCtxError::BadArgs;
            Err(Box::new(error) as BaseTableExprResultError)
        }
    }
}

struct TestDataGen {}

impl Iterator for TestDataGen {
    type Item = Result<Value, BaseTableExprResultError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Err(Box::new(UserCtxError::Runtime)))
    }
}
#[track_caller]
#[inline]
pub(crate) fn parse(statement: &str) -> ParserResult {
    partiql_parser::Parser::default().parse(statement)
}

#[track_caller]
#[inline]
pub(crate) fn lower(
    catalog: &dyn Catalog,
    parsed: &Parsed,
) -> partiql_logical::LogicalPlan<partiql_logical::BindingsOp> {
    let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
    planner.lower(parsed).expect("lower")
}

#[track_caller]
#[inline]
pub(crate) fn evaluate(
    mode: EvaluationMode,
    catalog: &dyn Catalog,
    logical: partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    bindings: MapBindings<Value>,
    ctx_vals: &[(String, &(dyn Any))],
) -> Result<Evaluated, EvalErr> {
    let mut planner = partiql_eval::plan::EvaluatorPlanner::new(mode, catalog);

    let mut plan = planner.compile(&logical).expect("Expect no plan error");

    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let mut ctx = BasicContext::new(bindings, sys);
    for (k, v) in ctx_vals {
        ctx.user.insert(k.as_str().into(), *v);
    }

    plan.execute_mut(&ctx)
}

#[test]
fn test_context_bad_args_permissive() {
    let query = "SELECT foo, bar from test_user_context(9) as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"));
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(
        EvaluationMode::Permissive,
        &catalog,
        lowered,
        bindings,
        &ctx,
    );

    assert!(out.is_ok());
    assert_eq!(out.unwrap().result, bag!(tuple!()).into());
}
#[test]
fn test_context_bad_args_strict() {
    use assert_matches::assert_matches;
    let query = "SELECT foo, bar from test_user_context(9) as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"));
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(EvaluationMode::Strict, &catalog, lowered, bindings, &ctx);

    assert!(out.is_err());
    let err = out.unwrap_err();
    assert_eq!(err.errors.len(), 1);
    let err = &err.errors[0];
    assert_matches!(err, EvaluationError::ExtensionResultError(err) => {
        assert_eq!(err.to_string(), "bad arguments")
    });
}

#[test]
fn test_context_runtime_permissive() {
    let query = "SELECT foo, bar from test_user_context('counter') as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"));
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(
        EvaluationMode::Permissive,
        &catalog,
        lowered,
        bindings,
        &ctx,
    );

    assert!(out.is_ok());
    assert_eq!(out.unwrap().result, bag!(tuple!()).into());
}

#[test]
fn test_context_runtime_strict() {
    use assert_matches::assert_matches;
    let query = "SELECT foo, bar from test_user_context('counter') as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"));
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(EvaluationMode::Strict, &catalog, lowered, bindings, &ctx);

    assert!(out.is_err());
    let err = out.unwrap_err();
    assert_eq!(err.errors.len(), 1);
    let err = &err.errors[0];
    assert_matches!(err, EvaluationError::ExtensionResultError(err) => {
        assert_eq!(err.to_string(), "runtime error")
    });
}
