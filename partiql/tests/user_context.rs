use std::any::Any;
use std::borrow::Cow;
use std::cell::RefCell;

use thiserror::Error;

use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::catalog::{Catalog, PartiqlCatalog};
use partiql_catalog::context::{SessionContext, SystemContext};
use partiql_catalog::extension::{Extension, ExtensionResultError};
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::BasicContext;
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
    #[error("unknown error")]
    Unknown,
}

#[derive(Debug)]
pub(crate) struct EvalTestCtxTable {}

impl BaseTableExpr for EvalTestCtxTable {
    fn evaluate<'a, 'c, 'o>(
        &self,
        args: &[Cow<'_, Value>],
        ctx: &'c dyn SessionContext,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(name) => generated_data(name.to_string(), ctx),
                _ => {
                    let error = UserCtxError::Unknown;
                    Err(ExtensionResultError::DataError(error.into()))
                }
            }
        } else {
            let error = UserCtxError::Unknown;
            Err(ExtensionResultError::DataError(error.into()))
        }
    }
}

struct TestDataGen<'a> {
    ctx: &'a dyn SessionContext,
    name: String,
}

impl Iterator for TestDataGen<'_> {
    type Item = Result<Value, ExtensionResultError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(cv) = self.ctx.user_context(&self.name) {
            if let Some(counter) = cv.downcast_ref::<Counter>() {
                let mut n = counter.data.borrow_mut();

                if *n > 0 {
                    *n -= 1;

                    let idx: u8 = (5 - *n) as u8;
                    let id = format!("id_{idx}");
                    let m = idx % 2;

                    return Some(Ok(tuple![("foo", m), ("bar", id)].into()));
                }
            }
        }
        None
    }
}

fn generated_data(name: String, ctx: &dyn SessionContext) -> BaseTableExprResult<'_> {
    Ok(Box::new(TestDataGen { ctx, name }))
}

#[derive(Debug)]
pub struct Counter {
    data: RefCell<u32>,
}

#[track_caller]
#[inline]
pub(crate) fn evaluate(
    catalog: &dyn Catalog,
    logical: partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
    bindings: MapBindings<Value>,
    ctx_vals: &[(String, &(dyn Any))],
) -> Value {
    let mut planner =
        partiql_eval::plan::EvaluatorPlanner::new(EvaluationMode::Permissive, catalog);

    let plan = planner.compile(&logical).expect("Expect no plan error");

    let sys = SystemContext {
        now: DateTime::from_system_now_utc(),
    };
    let mut ctx = BasicContext::new(bindings, sys);
    for (k, v) in ctx_vals {
        ctx.user.insert(k.as_str().into(), *v);
    }
    if let Ok(out) = plan.execute(&ctx) {
        out.result
    } else {
        Value::Missing
    }
}

#[test]
fn test_context() -> Result<(), TestError<'static>> {
    let expected: Value = bag![
        tuple![("foo", 1), ("bar", "id_1")],
        tuple![("foo", 0), ("bar", "id_2")],
        tuple![("foo", 1), ("bar", "id_3")],
        tuple![("foo", 0), ("bar", "id_4")],
        tuple![("foo", 1), ("bar", "id_5")],
    ]
    .into();

    let query = "SELECT foo, bar from test_user_context('counter') as data";

    let mut catalog = PartiqlCatalog::default();
    let ext = UserCtxTestExtension {};
    ext.load(&mut catalog).expect("extension load to succeed");

    let parsed = parse(query);
    let lowered = lower(&catalog, &parsed.expect("parse"))?;
    let bindings = Default::default();

    let counter = Counter {
        data: RefCell::new(5),
    };
    let ctx: [(String, &dyn Any); 1] = [("counter".to_string(), &counter)];
    let out = evaluate(&catalog, lowered, bindings, &ctx);

    assert!(out.is_bag());
    assert_eq!(&out, &expected);
    assert_eq!(*counter.data.borrow(), 0);

    Ok(())
}
