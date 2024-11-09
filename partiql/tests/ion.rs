use crate::common::{eval_query_with_catalog, TestError};
use assert_matches::assert_matches;
use partiql_catalog::catalog::PartiqlCatalog;
use partiql_catalog::extension::Extension;
use partiql_common::pretty::ToPretty;
use partiql_eval::eval::Evaluated;
use partiql_eval::plan::EvaluationMode;
use partiql_extension_ion::embedded::EmbeddedIonType;
use partiql_extension_value_functions::PartiqlValueFnExtension;
use partiql_value::embedded_document::DynEmbeddedDocumentTypeFactory;
use partiql_value::{EmbeddedDoc, Value};

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
    dbg!(&res);
    assert_matches!(res, Ok(_));
    let result = res.unwrap().result;
    println!("{}", result.to_pretty_string(80).unwrap());

    insta::assert_debug_snapshot!(result);
}

#[test]
fn ion_iter() {
    let contents = "[1,2,3,4]";
    let ion_typ = EmbeddedIonType::default().to_dyn_type_tag();
    let doc = ion_typ.construct(contents.as_bytes());
    let value = Value::EmbeddedDoc(Box::new(EmbeddedDoc::new(doc)));

    let items: Vec<_> = value.into_iter().collect();
    dbg!(&items);
    assert_eq!(items.len(), 4);
}
