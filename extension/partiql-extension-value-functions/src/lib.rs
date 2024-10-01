#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use partiql_catalog::call_defs::ScalarFnCallDef;
use partiql_catalog::catalog::Catalog;
use partiql_catalog::context::SessionContext;
use partiql_catalog::scalar_fn::{
    vararg_scalar_fn_overloads, ScalarFnExpr, ScalarFnExprResult, ScalarFunction,
    SimpleScalarFunctionInfo,
};
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::error::Error;

#[derive(Debug, Default)]
pub struct PartiqlValueFnExtension {}

impl partiql_catalog::extension::Extension for PartiqlValueFnExtension {
    fn name(&self) -> String {
        "value-functions".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), Box<dyn Error>> {
        for scfn in [function_catalog_tupleunion, function_catalog_tupleconcat] {
            match catalog.add_scalar_function(scfn()) {
                Ok(_) => continue,
                Err(e) => return Err(Box::new(e) as Box<dyn Error>),
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
        ctx: &'c dyn SessionContext<'c>,
    ) -> ScalarFnExprResult<'c> {
        let mut t = Tuple::default();
        for arg in args {
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
        ctx: &'c dyn SessionContext<'c>,
    ) -> ScalarFnExprResult<'c> {
        let result = args
            .into_iter()
            .map(|val| val.as_tuple_ref())
            .reduce(|l, r| Cow::Owned(l.tuple_concat(&r)))
            .map(|v| v.into_owned())
            .unwrap_or_default();
        Ok(Cow::Owned(Value::from(result)))
    }
}
