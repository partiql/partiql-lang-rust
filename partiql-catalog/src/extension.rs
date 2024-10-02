use crate::catalog::Catalog;
use std::error::Error;
use std::fmt::Debug;

pub trait Extension: Debug {
    fn name(&self) -> String;

    fn load(&self, catalog: &mut dyn Catalog) -> Result<(), Box<dyn Error>>;
}

pub type ExtensionResultError = Box<dyn Error>;
