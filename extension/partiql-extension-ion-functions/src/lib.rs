#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use ion_rs_old::data_source::ToIonDataSource;
use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::TableFunction;
use partiql_catalog::{
    BaseTableExpr, BaseTableExprResult, BaseTableExprResultError, BaseTableExprResultValueIter,
    BaseTableFunctionInfo, Catalog,
};
use partiql_extension_ion::decode::{IonDecoderBuilder, IonDecoderConfig};
use partiql_extension_ion::Encoding;
use partiql_logical as logical;
use partiql_value::Value;
use std::borrow::Cow;

use partiql_catalog::context::SessionContext;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use thiserror::Error;

/// Errors in ion extension.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum IonExtensionError {
    /// Function error.
    #[error("`read_ion` function error: `{}`", .0)]
    FunctionError(String),

    /// Io error.
    #[error("`read_ion` io error: `{}`", .0)]
    IoError(std::io::Error),

    /// Any other reading error.
    #[error("Ion read error: unknown error")]
    Unknown,
}

impl From<std::io::Error> for IonExtensionError {
    fn from(e: std::io::Error) -> Self {
        IonExtensionError::IoError(e)
    }
}

#[derive(Debug)]
pub struct IonExtension {}

impl partiql_catalog::Extension for IonExtension {
    fn name(&self) -> String {
        "ion".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), Box<dyn Error>> {
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
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext<'c>,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(path) => parse_ion_file(path),
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
    let path = PathBuf::from(path).canonicalize()?;
    let file = File::open(path)?;

    parse_ion_read(file)
}

fn parse_ion_read<'a>(mut reader: impl 'a + Read + Seek) -> BaseTableExprResult<'a> {
    let mut header: [u8; 4] = [0; 4];
    reader.read_exact(&mut header).expect("file header");
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
    let reader = ion_rs_old::ReaderBuilder::new().build(input).unwrap();
    let decoder =
        IonDecoderBuilder::new(IonDecoderConfig::default().with_mode(Encoding::Ion)).build(reader);
    let decoder = decoder.map_err(err_map)?.map(move |it| it.map_err(err_map));
    Ok(Box::new(decoder) as BaseTableExprResultValueIter<'_>)
}

#[cfg(test)]
mod tests {
    use super::*;

    use partiql_catalog::context::SystemContext;
    use partiql_catalog::{Catalog, Extension, PartiqlCatalog};
    use partiql_eval::env::basic::MapBindings;
    use partiql_eval::eval::BasicContext;
    use partiql_eval::plan::EvaluationMode;
    use partiql_parser::{Parsed, ParserResult};
    use partiql_value::{bag, tuple, DateTime, Value};

    #[track_caller]
    #[inline]
    pub(crate) fn parse(statement: &str) -> ParserResult<'_> {
        partiql_parser::Parser::default().parse(statement)
    }

    #[track_caller]
    #[inline]
    pub(crate) fn lower(
        catalog: &dyn Catalog,
        parsed: &Parsed<'_>,
    ) -> partiql_logical::LogicalPlan<partiql_logical::BindingsOp> {
        let planner = partiql_logical_planner::LogicalPlanner::new(catalog);
        planner.lower(parsed).expect("lower")
    }

    #[track_caller]
    #[inline]
    pub(crate) fn evaluate(
        catalog: &dyn Catalog,
        logical: partiql_logical::LogicalPlan<partiql_logical::BindingsOp>,
        bindings: MapBindings<Value>,
    ) -> Value {
        let mut planner =
            partiql_eval::plan::EvaluatorPlanner::new(EvaluationMode::Permissive, catalog);

        let mut plan = planner.compile(&logical).expect("Expect no plan error");

        let sys = SystemContext {
            now: DateTime::from_system_now_utc(),
        };
        let ctx = BasicContext::new(bindings, sys);
        if let Ok(out) = plan.execute_mut(&ctx) {
            out.result
        } else {
            Value::Missing
        }
    }

    #[track_caller]
    #[inline]
    #[allow(dead_code)]
    pub(crate) fn pass_eval(statement: &str, env: &Option<Value>, expected: &Value) {
        let mut catalog = PartiqlCatalog::default();
        let ext = IonExtension {};
        ext.load(&mut catalog)
            .expect("ion extension load to succeed");

        let parsed = parse(statement);
        let lowered = lower(&catalog, &parsed.expect("parse"));
        let bindings = env
            .as_ref()
            .map(std::convert::Into::into)
            .unwrap_or_default();
        let out = evaluate(&catalog, lowered, bindings);

        assert!(out.is_bag());
        assert_eq!(&out, expected);
    }

    fn expected() -> Value {
        bag![
            tuple![("Program", "p1"), ("Operation", "get")],
            tuple![("Program", "p1"), ("Operation", "put")],
            tuple![("Program", "p2"), ("Operation", "get")],
            tuple![("Program", "p2"), ("Operation", "put")],
            tuple![("Program", "p3"), ("Operation", "update")],
        ]
        .into()
    }

    #[track_caller]
    fn custom_ion_scan(file: &str) {
        let value = expected();
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test");
        path.push(file);
        let path = path.as_path().display();

        let query = format!("SELECT DISTINCT Program, Operation from read_ion('{path}') as fel");
        pass_eval(&query, &None, &value);
    }

    #[test]
    fn custom_ion_scan_text() {
        custom_ion_scan("test.ion");
    }

    #[test]
    fn custom_ion_scan_binary() {
        custom_ion_scan("test.10n");
    }

    #[test]
    fn custom_ion_scan_zstd() {
        custom_ion_scan("test.10n.zst");
    }
}
