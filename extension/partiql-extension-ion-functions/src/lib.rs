#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use crate::scan_ion::ScanIonFunction;
use partiql_catalog::catalog::Catalog;
use partiql_catalog::extension::{ExtensionError, ExtensionResultError};
use partiql_catalog::table_fn::{BaseTableFunctionInfo, TableFunction};
use partiql_extension_ion::decode::IonDecodeError;
use partiql_value::Value;
use read_ion::ReadIonFunction;
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

mod buffer;
mod read_ion;
mod scan_ion;

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

    /// Ion Stream Error
    #[error("Ion Stream Error: `{}`", .0)]
    IonStreamError(IonDecodeError),

    /// Ion Read Error
    #[error("Ion Read Error: `{}`", .0)]
    IonReadError(Box<dyn Error>),

    /// Io error.
    #[error("`read_ion` io error: `{}`", .0)]
    IoError(std::io::Error),

    /// Data error. Generally this will result in a `MISSING` in place of this data item.
    #[error("Data error: `{}`", .0)]
    DataError(ExtensionError),

    /// Any other reading error.
    #[error("Ion read error: unknown error")]
    Unknown,
}

pub type IonTableExprResult<'a> = Result<IonTableExprResultValueIter<'a>, IonExtensionError>;

pub type IonTableExprResultValueIter<'a> =
    Box<dyn 'a + Iterator<Item = Result<Value, IonExtensionError>>>;

impl From<std::io::Error> for IonExtensionError {
    fn from(e: std::io::Error) -> Self {
        IonExtensionError::IoError(e)
    }
}

impl From<IonExtensionError> for ExtensionResultError {
    fn from(value: IonExtensionError) -> Self {
        match value {
            IonExtensionError::FunctionError(_) => ExtensionResultError::ReadError(Box::new(value)),
            IonExtensionError::IoError(_) => ExtensionResultError::ReadError(Box::new(value)),
            IonExtensionError::DataError(_) => ExtensionResultError::DataError(Box::new(value)),
            IonExtensionError::Unknown => ExtensionResultError::ReadError(Box::new(value)),
            IonExtensionError::IonStreamError(_) => {
                ExtensionResultError::ReadError(Box::new(value))
            }
            IonExtensionError::IonReadError(_) => ExtensionResultError::ReadError(Box::new(value)),
        }
    }
}

#[derive(Debug)]
pub struct IonExtension {}

impl partiql_catalog::extension::Extension for IonExtension {
    fn name(&self) -> String {
        "ion".into()
    }

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), ExtensionResultError> {
        load_table_fn(catalog, Box::new(ReadIonFunction::new()))?;
        load_table_fn(catalog, Box::new(ScanIonFunction::new()))?;
        Ok(())
    }
}

fn load_table_fn(
    catalog: &mut dyn Catalog,
    fn_info: Box<dyn BaseTableFunctionInfo>,
) -> Result<(), ExtensionResultError> {
    match catalog.add_table_function(TableFunction::new(fn_info)) {
        Ok(_) => Ok(()),
        Err(e) => Err(ExtensionResultError::LoadError(e.into())),
    }
}
