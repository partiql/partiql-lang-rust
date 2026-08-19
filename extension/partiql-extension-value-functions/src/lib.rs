#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use partiql_catalog::call_defs::ScalarFnCallDef;
use partiql_catalog::catalog::Catalog;
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::ExtensionResultError;
use partiql_catalog::scalar_fn::{
    vararg_scalar_fn_overloads, ScalarFnExpr, ScalarFnExprResult, ScalarFunction,
    SimpleScalarFunctionInfo,
};
use partiql_value::datum::{DatumCategory, DatumCategoryRef};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;

/// Rejects a non-tuple argument to a tuple function.
///
/// `TUPLEUNION` is defined by the PartiQL specification only over tuple arguments; scalar/sequence
/// coercion is a `SELECT *` concern handled by the planner, not by these functions. `NULL`/`MISSING`
/// arguments never reach here — they are short-circuited by the dispatcher's argument checker.
fn require_tuple(func: &str, arg: &Value) -> Result<(), ExtensionResultError> {
    let found = match arg.category() {
        DatumCategoryRef::Tuple(_) => return Ok(()),
        DatumCategoryRef::Null => "null",
        DatumCategoryRef::Missing => "missing",
        DatumCategoryRef::Sequence(_) => "sequence",
        DatumCategoryRef::Scalar(_) => "scalar",
        DatumCategoryRef::Graph(_) => "graph",
    };
    Err(ExtensionResultError::DataError(
        format!("`{func}` expects tuple arguments, found {found}").into(),
    ))
}

#[derive(Debug, Default)]
pub struct PartiqlValueFnExtension {}

impl partiql_catalog::extension::Extension for PartiqlValueFnExtension {
    fn name(&self) -> String {
        "value-functions".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), ExtensionResultError> {
        for scfn in [function_catalog_tupleunion, function_catalog_tupleconcat] {
            match catalog.add_scalar_function(scfn()) {
                Ok(_) => continue,
                Err(e) => return Err(ExtensionResultError::LoadError(e.into())),
            }
        }
        Ok(())
    }
}

fn function_catalog_tupleunion() -> ScalarFunction {
    let scalar_fn = Box::new(TupleUnionFnExpr::default());
    let call_def = ScalarFnCallDef {
        names: vec!["tupleunion"],
        overloads: vararg_scalar_fn_overloads(scalar_fn),
    };

    let info = SimpleScalarFunctionInfo::new(call_def);
    ScalarFunction::new(Box::new(info))
}

/// Represents a built-in tupleunion function,
/// e.g. `tupleunion({ 'bob': 1 }, { 'sally': 2 }, { 'sally': 2 })` -> `{'bob: 1, 'sally':1, 'sally':2}`.
#[derive(Debug, Clone, Default)]
struct TupleUnionFnExpr {}
impl ScalarFnExpr for TupleUnionFnExpr {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> ScalarFnExprResult<'c> {
        let mut t = Tuple::default();
        for arg in args {
            require_tuple("tupleunion", arg.as_ref())?;
            t.extend(
                arg.as_tuple_ref()
                    .pairs()
                    .map(|(k, v)| (k.as_str(), v.clone())),
            )
        }
        Ok(Cow::Owned(Value::from(t)))
    }
}

fn function_catalog_tupleconcat() -> ScalarFunction {
    let scalar_fn = Box::new(TupleConcatFnExpr::default());
    let call_def = ScalarFnCallDef {
        names: vec!["tupleconcat"],
        overloads: vararg_scalar_fn_overloads(scalar_fn),
    };

    let info = SimpleScalarFunctionInfo::new(call_def);
    ScalarFunction::new(Box::new(info))
}

/// Represents a built-in tupleconcat function,
/// e.g. `tupleconcat({ 'bob': 1 }, { 'sally': 2 }, { 'sally': 2 })` -> `{'bob: 1, 'sally':2}`.
#[derive(Debug, Clone, Default)]
struct TupleConcatFnExpr {}
impl ScalarFnExpr for TupleConcatFnExpr {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext,
    ) -> ScalarFnExprResult<'c> {
        for arg in args {
            require_tuple("tupleconcat", arg.as_ref())?;
        }
        let result = args
            .iter()
            .map(|val| val.as_tuple_ref())
            .reduce(|l, r| Cow::Owned(l.tuple_concat(&r)))
            .map(|v| v.into_owned())
            .unwrap_or_default();
        Ok(Cow::Owned(Value::from(result)))
    }
}
