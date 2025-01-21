use crate::catalog::Catalog;
use std::error::Error;
use std::fmt::Debug;
use thiserror::Error;

pub trait Extension: Debug {
    fn name(&self) -> String;

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), ExtensionResultError>;
}

pub type ExtensionError = Box<dyn Error>;

/// Errors in extension.
///
/// ### Notes
/// This is marked `#[non_exhaustive]`, to reserve the right to add more variants in the future.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ExtensionResultError {
    /// Load error. Error loading the extension.
    #[error("Scan error: `{}`", .0)]
    LoadError(ExtensionError),

    /// Read error. This will generally terminate the read operation.
    #[error("Scan error: `{}`", .0)]
    ReadError(ExtensionError),

    /// Data error. Generally this will result in a `MISSING` in place of this data item.
    #[error("Data error: `{}`", .0)]
    DataError(ExtensionError),
}
