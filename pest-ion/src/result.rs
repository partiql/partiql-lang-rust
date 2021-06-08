// Copyright Amazon.com, Inc. or its affiliates.

//! [`Error`](std::error::Error) and [`Result`] types for working with Pest to Ion.

use thiserror::Error;

/// Main [`Result`] type for Pest to Ion.
pub type PestToIonResult<T> = Result<T, PestToIonError>;

/// Error type for problems in this crate.
#[derive(Error, Debug)]
pub enum PestToIonError {
    /// An error working with [`pest_meta`].
    #[error("Pest Error: {0}")]
    Pest(#[from] pest::error::Error<pest_meta::parser::Rule>),

    /// An error working with [`ion_rs`].
    #[error("Ion Error: {0}")]
    Ion(#[from] ion_rs::result::IonError),

    /// General error from this library.
    #[error("Pest to Ion Error: {0}")]
    Invalid(String),
}

/// Convenience function to create a general error result.
pub fn invalid<T, S: Into<String>>(message: S) -> PestToIonResult<T> {
    Err(PestToIonError::Invalid(message.into()))
}
