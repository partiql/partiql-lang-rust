use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

use partiql_catalog::{Catalog, PartiqlCatalog};
use partiql_eval::env::basic::MapBindings;
use partiql_eval::eval::EvalPlan;
use partiql_eval::plan::{EvaluatorPlanner, EvaluationMode};
use partiql_parser::Parser;
use partiql_value::Value;

use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig, IonDecodeResult};
use partiql_extension_ion::encode::{IonEncodeError, IonEncoderBuilder, IonEncoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical::{BindingsOp, LogicalPlan};
use ion_rs::element::writer::TextKind;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn eval(query: &str, env: &str) -> PyResult<(String, String)> {
    let bindings = to_bindings(env);
    let catalog = PartiqlCatalog::default();
    let mut compiler = init_compiler(query, &catalog)?;

    if let Ok(out) = compiler.execute_mut(bindings) {
        let res = &out.result;
        Ok((format!("{:?}", &res), serde_json::to_string_pretty(&res).expect("Error in unwrapping json serde")))
        // let ion_out = encode_ion_text(res, Encoding::Ion);

        // match ion_out {
        //     Ok(res) => {
        //         // Ok(res)
        //         Ok(serde_json::to_string_pretty(&res).expect("Error in unwrapping json serde"))
        //     },
        //     Err(err) => Err(PyErr::new::<PyTypeError, _>(format!("{:?}", err)))
        // }
    } else {
        Err(PyErr::new::<PyTypeError, _>("Compiler Error"))
    }
}

fn init_compiler(query: &str, catalog: &dyn Catalog) -> Result<EvalPlan, PyErr> {
    let lowered = lower(query, catalog)?;
    let mut compiler = EvaluatorPlanner::new(EvaluationMode::Permissive, catalog);
    compiler.compile(&lowered).map_err(|err| PyTypeError::new_err(format!("{:?}", err)))
}

fn lower(query: &str, catalog: &dyn Catalog) -> Result<LogicalPlan<BindingsOp>, PyErr> {
    let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
    let parsed = Parser::default().parse(query).map_err(|err| PyTypeError::new_err(format!("{:?}", err)))?;
    planner.lower(&parsed).map_err(|err| PyTypeError::new_err(format!("{:?}", err)))
}


fn to_bindings(env: &str) -> MapBindings<Value> {
    let env_as_ion = decode_ion_text(env, Encoding::Ion).unwrap_or(Value::Missing);
    MapBindings::from(env_as_ion)
}

fn decode_ion_text(contents: &str, encoding: Encoding) -> IonDecodeResult {
    let reader = ion_rs::ReaderBuilder::new().build(contents)?;
    let mut iter = IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(encoding))
    .build(reader)?;

    let val = iter.next();

    val.unwrap()
}

fn encode_ion_text(value: &Value, encoding: Encoding) -> Result<String, IonEncodeError> {
    let mut buff = vec![];
    let mut writer = ion_rs::TextWriterBuilder::new(TextKind::Compact)
    .build(&mut buff)
    .expect("writer");
    let mut encoder = IonEncoderBuilder::new(IonEncoderConfig::default().with_mode(encoding))
    .build(&mut writer)?;

    encoder.write_value(value)?;

    drop(encoder);
    drop(writer);

    Ok(String::from_utf8(buff).expect("string"))
}

/// A Python module implemented in Rust.
#[pymodule]
fn partiql_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    Ok(())
}