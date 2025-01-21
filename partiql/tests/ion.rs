use crate::common::{eval_query_with_catalog, TestError};
use assert_matches::assert_matches;
use partiql_catalog::catalog::PartiqlCatalog;
use partiql_catalog::extension::Extension;
use partiql_common::pretty::ToPretty;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_ion::boxed_ion::BoxedIonType;
use partiql_extension_value_functions::PartiqlValueFnExtension;
use partiql_value::boxed_variant::DynBoxedVariantTypeFactory;
use partiql_value::{Value, Variant};

mod common;

#[track_caller]
#[inline]
pub fn eval(statement: &str, mode: EvaluationMode) -> Result<Evaluated, TestError<'_>> {
    let mut catalog = PartiqlCatalog::default();
    let ext = PartiqlValueFnExtension::default();
    ext.load(&mut catalog)?;

    eval_query_with_catalog(statement, &catalog, mode)
}

#[test]
fn ion_simple() {
    let query = "select x from `(1 hi::2)` as x";
    // << {x: `1`}, {x: `hi::2`}>>

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;

    insta::assert_snapshot!(result.to_pretty_string(25).expect("pretty"));
}

#[test]
fn ion_in_struct() {
    let query =
        "select x.data from <<{'data': `annot1::(1 hi::2)`}, {'data': `annot2::{data: 3}`}>> as x";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;

    insta::assert_snapshot!(result.to_pretty_string(25).expect("pretty"));
}

#[test]
fn ion_structs() {
    let query =
        "select x.data from <<`{data: annot1::(1 hi::2)}`, `{data: annot2::{data: 3}}`>> as x";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;

    insta::assert_snapshot!(result.to_pretty_string(25).expect("pretty"));
}

#[test]
fn ion_paths() {
    let query = "select x[1].foo from `([{foo:1}, {foo: 2}] ({foo: hi::1} {foo: world::2}))` as x";
    // << {x: `{foo: 2}`}, {x: `{foo: world::2}`}>>

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;

    insta::assert_snapshot!(result.to_pretty_string(25).expect("pretty"));
}

#[test]
fn ion_iter() {
    let contents = "[1,2,3,4]";
    let ion_typ = BoxedIonType::default().to_dyn_type_tag();
    let value = Value::Variant(Box::new(Variant::new(contents, ion_typ).expect("doc ctor")));

    let items: Vec<_> = value.into_iter().collect();
    dbg!(&items);
    assert_eq!(items.len(), 4);
}

#[test]
fn ion_cmp() {
    let query = "`foo` < `bar` OR `1.2` > `1`";

    let res = eval(query, EvaluationMode::Permissive);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;

    insta::assert_snapshot!(result.to_pretty_string(25).expect("pretty"));
}
