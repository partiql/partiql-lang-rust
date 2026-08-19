use crate::common::{eval_query_with_catalog, TestError};
use assert_matches::assert_matches;
use partiql_catalog::call_defs::{CallSpecArg, ScalarFnCallDef, ScalarFnCallSpec};
use partiql_catalog::catalog::{MutableCatalog, PartiqlCatalog};
use partiql_catalog::context::SessionContext;
use partiql_catalog::scalar_fn::{
    ScalarFnExpr, ScalarFnExprResult, ScalarFunction, SimpleScalarFunctionInfo,
};
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_value::Value;
use std::borrow::Cow;

mod common;

/// A test-only scalar UDF taking a single `String` argument, registered directly into the
/// catalog (not shipped by any extension). It exercises the dispatcher's handling of scalar
/// arguments to user-registered scalar functions: with the struct-of-dynamic guard this call
/// short-circuited to `MISSING`/error before the body ran.
#[derive(Debug, Clone, Default)]
struct StrLengthFnExpr {}
impl ScalarFnExpr for StrLengthFnExpr {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> ScalarFnExprResult<'c> {
        let result = match args.first().map(|arg| arg.as_ref()) {
            Some(Value::String(s)) => Value::from(s.chars().count() as i64),
            _ => Value::Missing,
        };
        Ok(Cow::Owned(result))
    }
}

fn str_length_fn() -> ScalarFunction {
    let call_def = ScalarFnCallDef {
        names: vec!["str_length"],
        overloads: vec![ScalarFnCallSpec {
            input: vec![CallSpecArg::Positional],
            output: Box::new(StrLengthFnExpr::default()),
        }],
    };
    ScalarFunction::new(Box::new(SimpleScalarFunctionInfo::new(call_def)))
}

#[track_caller]
#[inline]
pub fn eval(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
    let mut catalog = PartiqlCatalog::default();
    catalog
        .add_scalar_function(str_length_fn())
        .expect("register str_length");
    let catalog = catalog.to_shared_catalog();

    eval_query_with_catalog(statement, &catalog, mode)
}

/// A user-registered scalar function invoked with a scalar (non-struct) argument runs and
/// returns its result, in both permissive and strict modes.
#[test]
fn scalar_fn_string_arg_permissive() {
    let res = eval("str_length('hello')", EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    assert_matches!(res.unwrap().result, Value::Integer(5));
}

#[test]
fn scalar_fn_string_arg_strict() {
    let res = eval("str_length('hello')", EvaluationMode::Strict);
    assert_matches!(res, Ok(_));
    assert_matches!(res.unwrap().result, Value::Integer(5));
}
