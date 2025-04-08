#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use partiql_catalog::call_defs::{CallDef, CallSpec, CallSpecArg};
use partiql_catalog::catalog::Catalog;
use partiql_catalog::context::SessionContext;
use partiql_catalog::extension::{ExtensionError, ExtensionResultError};
use partiql_catalog::table_fn::{
    BaseTableExpr, BaseTableExprResult, BaseTableFunctionInfo, TableFunction,
};
use partiql_logical as logical;
use partiql_value::{Tuple, Value};
use std::borrow::Cow;
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

/// Errors in csv extension.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CsvExtensionError {
    /// Function error.
    #[error("`scan_csv` function error: `{}`", .0)]
    FunctionError(String),

    /// Io error.
    #[error("`scan_csv` io error: `{}`", .0)]
    IoError(std::io::Error),

    /// Io error.
    #[error("`scan_csv` io error: `{}`", .0)]
    CsvReadError(Box<dyn Error>),

    /// Data error. Generally this will result in a `MISSING` in place of this data item.
    #[error("Data error: `{}`", .0)]
    DataError(ExtensionError),

    /// Any other reading error.
    #[error("csv read error: unknown error")]
    Unknown,
}

pub type CsvTableExprResult<'a> = Result<CsvTableExprResultValueIter<'a>, CsvExtensionError>;

pub type CsvTableExprResultValueIter<'a> =
    Box<dyn 'a + Iterator<Item = Result<Value, CsvExtensionError>>>;

impl From<std::io::Error> for CsvExtensionError {
    fn from(e: std::io::Error) -> Self {
        CsvExtensionError::IoError(e)
    }
}

impl From<csv::Error> for CsvExtensionError {
    fn from(e: csv::Error) -> Self {
        CsvExtensionError::CsvReadError(Box::new(e))
    }
}

impl From<CsvExtensionError> for ExtensionResultError {
    fn from(value: CsvExtensionError) -> Self {
        match value {
            CsvExtensionError::IoError(_) => ExtensionResultError::ReadError(Box::new(value)),
            CsvExtensionError::Unknown => ExtensionResultError::ReadError(Box::new(value)),
            CsvExtensionError::FunctionError(_) => ExtensionResultError::LoadError(Box::new(value)),
            CsvExtensionError::CsvReadError(_) => ExtensionResultError::ReadError(Box::new(value)),
            CsvExtensionError::DataError(_) => ExtensionResultError::DataError(Box::new(value)),
        }
    }
}

#[derive(Debug)]
pub struct CsvExtension {}

impl partiql_catalog::extension::Extension for CsvExtension {
    fn name(&self) -> String {
        "csv".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), ExtensionResultError> {
        match catalog.add_table_function(TableFunction::new(Box::new(ScanCsvFunction::new()))) {
            Ok(_) => Ok(()),
            Err(e) => Err(ExtensionResultError::LoadError(e.into())),
        }
    }
}

#[derive(Debug)]
pub(crate) struct ScanCsvFunction {
    call_def: CallDef,
}

/// `scan_csv` scans csv data lazily, wrapping it into PartiQL Boxed Variants
impl ScanCsvFunction {
    pub fn new() -> Self {
        ScanCsvFunction {
            call_def: CallDef {
                names: vec!["scan_csv"],
                overloads: vec![CallSpec {
                    input: vec![CallSpecArg::Positional],
                    output: Box::new(|args| {
                        logical::ValueExpr::Call(logical::CallExpr {
                            name: logical::CallName::ByName("scan_csv".to_string()),
                            arguments: args,
                        })
                    }),
                }],
            },
        }
    }
}

impl BaseTableFunctionInfo for ScanCsvFunction {
    fn call_def(&self) -> &CallDef {
        &self.call_def
    }

    fn plan_eval(&self) -> Box<dyn BaseTableExpr> {
        Box::new(EvalFnScanCsv {})
    }
}

#[derive(Debug)]
pub(crate) struct EvalFnScanCsv {}

impl BaseTableExpr for EvalFnScanCsv {
    fn evaluate<'c>(
        &self,
        args: &[Cow<'_, Value>],
        _ctx: &'c dyn SessionContext<'c>,
    ) -> BaseTableExprResult<'c> {
        if let Some(arg1) = args.first() {
            match arg1.as_ref() {
                Value::String(path) => Ok(Box::new(
                    parse_csv_file(path)?.map(|it| it.map_err(Into::into)),
                )),
                _ => {
                    let error = CsvExtensionError::FunctionError(
                        "expected string path argument".to_string(),
                    );
                    Err(ExtensionResultError::ReadError(error.into()))
                }
            }
        } else {
            let error = CsvExtensionError::FunctionError("expected path argument".to_string());
            Err(ExtensionResultError::ReadError(error.into()))
        }
    }
}

fn parse_csv_file<'a>(path: &str) -> CsvTableExprResult<'a> {
    let path = PathBuf::from(path).canonicalize()?;
    let file = File::open(path)?;

    let mut rdr = csv::Reader::from_reader(file);

    let keys = rdr.headers()?.clone();
    let data = rdr.into_records();

    let rows = data.map(move |row| {
        match row {
            Ok(row) => {
                //
                let vals = row.iter();
                Ok(keys.iter().zip(vals).collect::<Tuple>().into())
            }
            Err(err) => Err(CsvExtensionError::DataError(err.into())),
        }
    });
    Ok(Box::new(rows))
}
