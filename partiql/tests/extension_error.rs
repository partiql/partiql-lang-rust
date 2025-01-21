use std::any::Any;
use std::borrow::Cow;

use thiserror::Error;

use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::catalog::{Catalog, PartiqlCatalog};
use partiql_catalog::context::{SessionContext, SystemContext};
use partiql_catalog::extension::{Extension, ExtensionResultError};
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_eval::env::basic::MapBindings;
use partiql_eval::error::{EvalErr, EvaluationError};
use partiql_eval::eval::{BasicContext, Evaluated};
use partiql_eval::plan::EvaluationMode;
use partiql_value::{bag, tuple, DateTime, Value};

use crate::common::{lower, parse, TestError};
use partiql_logical as logical;

mod common;
#[derive(Debug)]
pub struct UserCtxTestExtension {}

impl partiql_catalog::extension::Extension for UserCtxTestExtension {
    fn name(&self) -> String {
        "test_extension".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), ExtensionResultError> {
        match catalog
            .add_table_function(TableFunction::new(Box::new(TestUserContextFunction::new())))
        {
            Ok(_) => Ok(()),
            Err(e) => Err(ExtensionResultError::LoadError(e.into())),
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
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext<'c>,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(_name) => Ok(Box::new(TestDataGen {})),
                _ => {
                    let error = UserCtxError::BadArgs;
                    Err(ExtensionResultError::LoadError(error.into()))
                }
            }
        } else {
            let error = UserCtxError::BadArgs;
            Err(ExtensionResultError::LoadError(error.into()))
        }
    }
}

#[derive(Debug)]
struct TestDataGen {}

impl Iterator for TestDataGen {
    type Item = Result<Value, ExtensionResultError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Err(ExtensionResultError::ReadError(Box::new(
            UserCtxError::Runtime,
        ))))
    }
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
fn test_context_bad_args_permissive() -> Result<(), TestError<'static>> {
    let query = "SELECT foo, bar from test_user_context(9) as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"))?;
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

    Ok(())
}
#[test]
fn test_context_bad_args_strict() -> Result<(), TestError<'static>> {
    use assert_matches::assert_matches;
    let query = "SELECT foo, bar from test_user_context(9) as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"))?;
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(EvaluationMode::Strict, &catalog, lowered, bindings, &ctx);

    assert!(out.is_err());
    let err = out.unwrap_err();
    assert_eq!(err.errors.len(), 1);
    let err = &err.errors[0];
    assert_matches!(err, EvaluationError::ExtensionResultError(err) => {
        assert_eq!(err.to_string(), "Scan error: `bad arguments`")
    });

    Ok(())
}

#[test]
fn test_context_runtime_permissive() -> Result<(), TestError<'static>> {
    let query = "SELECT foo, bar from test_user_context('counter') as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"))?;
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
    Ok(())
}

#[test]
fn test_context_runtime_strict() -> Result<(), TestError<'static>> {
    use assert_matches::assert_matches;
    let query = "SELECT foo, bar from test_user_context('counter') as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"))?;
    let bindings = Default::default();

    let ctx: [(String, &dyn Any); 0] = [];
    let out = evaluate(EvaluationMode::Strict, &catalog, lowered, bindings, &ctx);

    assert!(out.is_err());
    let err = out.unwrap_err();
    assert_eq!(err.errors.len(), 1);
    let err = &err.errors[0];
    assert_matches!(err, EvaluationError::ExtensionResultError(err) => {
        assert_eq!(err.to_string(), "Scan error: `runtime error`")
    });

    Ok(())
}
