#![deny(rust_2018_idioms)]
#![deny(clippy::all)]

use crate::context::SessionContext;
use catalog::Catalog;
use scalar_fn::ScalarFunctionInfo;
use std::error::Error;
use std::fmt::Debug;
use table_fn::BaseTableFunctionInfo;

pub mod call_defs;

pub mod context;

pub mod catalog;
pub mod extension;
pub mod scalar_fn;
pub mod table_fn;
