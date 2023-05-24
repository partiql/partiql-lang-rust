use ion_rs::data_source::ToIonDataSource;
use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::{
    BaseTableExpr, BaseTableExprResult, BaseTableExprResultError, BaseTableExprResultValueIter,
    BaseTableFunctionInfo, Catalog,
};
use partiql_catalog::{CatalogError, ObjectId, TableFunction};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical as logical;
use partiql_value::Value;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs::{read, File};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use thiserror::Error;

/// Errors in ion extension.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum IonExtensionError {
    /// Stream error.
    #[error("`read_ion` function error: `{}`", .0)]
    FunctionError(String),

    /// Any other reading error.
    #[error("Ion read error: unknown error")]
    Unknown,
}

#[derive(Debug)]
pub(crate) struct IonExtension {}

impl partiql_catalog::Extension for IonExtension {
    fn name(&self) -> String {
        "ion".into()
    }

    fn load(&self, catalog: &mut Box<dyn Catalog>) -> Result<(), Box<dyn Error>> {
        match catalog.add_table_function(TableFunction::new(Box::new(ReadIonFunction::new()))) {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e) as Box<dyn Error>),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ReadIonFunction {
    call_def: CallDef,
}

impl ReadIonFunction {
    pub fn new() -> Self {
        ReadIonFunction {
            call_def: CallDef {
                names: vec!["read_ion"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        logical::ValueExpr::Call(logical::CallExpr {
                            name: logical::CallName::ByName("read_ion".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for ReadIonFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(EvalFnReadIon {})
    }
}

#[derive(Debug)]
pub(crate) struct EvalFnReadIon {}

impl BaseTableExpr for EvalFnReadIon {
    fn evaluate(&self, args: &[Cow<Value>]) -> BaseTableExprResult {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(path) => parse_ion_file(&path),
                _ => {
                    let error = IonExtensionError::FunctionError(
                        "expected string path argument".to_string(),
                    );
                    Err(Box::new(error) as BaseTableExprResultError)
                }
            }
        } else {
            let error = IonExtensionError::FunctionError("expected path argument".to_string());
            Err(Box::new(error) as BaseTableExprResultError)
        }
    }
}

fn parse_ion_file<'a>(path: &str) -> BaseTableExprResult<'a> {
    let path = PathBuf::from(path).canonicalize().unwrap();
    let mut file = File::open(path).unwrap();

    parse_ion_read(file)
}

fn parse_ion_read<'a>(mut reader: impl 'a + Read + Seek) -> BaseTableExprResult<'a> {
    let mut header: [u8; 4] = [0; 4];
    reader.read(&mut header).expect("file header");
    reader.seek(SeekFrom::Start(0)).expect("file seek");

    if header.starts_with(&[0x1f, 0x8b]) {
        let decoder = flate2::read::GzDecoder::new(reader);
        let buffered = BufReader::new(decoder);
        parse_ion_buff(buffered)
    } else if header.starts_with(&[0x28, 0xB5, 0x2F, 0xFD]) {
        let decoder = zstd::Decoder::new(reader).expect("zstd reader creation");
        let buffered = BufReader::new(decoder);
        parse_ion_buff(buffered)
    } else {
        let buffered = BufReader::new(reader);
        parse_ion_buff(buffered)
    }
}

fn parse_ion_buff<'a, I: 'a + ToIonDataSource>(input: I) -> BaseTableExprResult<'a> {
    let err_map = |e| Box::new(e) as BaseTableExprResultError;
    let mut reader = ion_rs::ReaderBuilder::new().build(input).unwrap();
    let decoder =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion)).build(reader);
    let mut decoder = decoder.map_err(err_map)?.map(move |it| it.map_err(err_map));
    Ok(Box::new(decoder) as BaseTableExprResultValueIter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    use partiql_catalog::{Catalog, Extension, PartiqlCatalog};
    use partiql_eval::env::basic::MapBindings;
    use partiql_parser::{Parsed, ParserResult};
    use partiql_value::{partiql_bag, partiql_list, partiql_tuple, DateTime, Value};
    use rust_decimal_macros::dec;
    use std::num::NonZeroU8;

    #[track_caller]
    #[inline]
    pub(crate) fn parse(statement: &str) -> ParserResult {
        partiql_parser::Parser::default().parse(statement)
    }

    #[track_caller]
    #[inline]
    pub(crate) fn lower(
        catalog: &Box<dyn Catalog>,
        parsed: &Parsed,
    ) -> partiql_logical::LogicalPlan<partiql_logical::BindingsOp> {
        let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
        planner.lower(parsed).expect("lower")
    }

    #[track_caller]
    #[inline]
    pub(crate) fn evaluate(
        catalog: &Box<dyn Catalog>,
        logical: partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
        bindings: MapBindings<Value>,
    ) -> Value {
        let planner = partiql_eval::plan::EvaluatorPlanner::new(catalog);

        let mut plan = planner.compile(&logical);

        if let Ok(out) = plan.execute_mut(bindings) {
            out.result
        } else {
            Value::Missing
        }
    }

    #[track_caller]
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn pass_eval(statement: &str, env: &Option<Value>, expected: &Value) {
        let mut catalog = Box::new(PartiqlCatalog::default()) as Box<dyn Catalog>;
        let ext = IonExtension {};
        ext.load(&mut catalog);

        let parsed = parse(statement);
        let lowered = lower(&catalog, &parsed.expect("parse"));
        let bindings = env
            .as_ref()
            .map(|e| (e).into())
            .unwrap_or_else(MapBindings::default);
        let out = evaluate(&catalog, lowered, bindings);

        assert!(out.is_bag());
        assert_eq!(&out, expected);
    }

    #[test]
    fn custom_ion_scan() {
        let value = partiql_bag![
            partiql_tuple![("Program", "p1"), ("Operation", "get")],
            partiql_tuple![("Program", "p1"), ("Operation", "put")],
            partiql_tuple![("Program", "p2"), ("Operation", "get")],
            partiql_tuple![("Program", "p2"), ("Operation", "put")],
            partiql_tuple![("Program", "p3"), ("Operation", "update")],
        ]
        .into();

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test/test.ion");
        let path = path.as_path().display();

        let query = format!("SELECT DISTINCT Program, Operation from read_ion('{path}') as fel");
        pass_eval(&query, &None, &value);
    }
}
