use super::RegisterWriter;
use crate::engine::error::Result;
use crate::engine::source::api::{BufferStability, DataSource, DataSourceFactory, ScanSource};

/// Internal wrapper for data source implementations
///
/// All data sources now go through the ExecutionCatalog pattern.
pub(crate) enum DataSourceImpl {
    Catalog(Box<dyn DataSource>),
}

impl DataSourceImpl {
    pub fn open(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::Catalog(r) => r.open(),
        }
    }

    pub fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        match self {
            DataSourceImpl::Catalog(r) => r.next_row(writer),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::Catalog(r) => r.close(),
        }
    }
}

/// Internal enum for reader factory implementations
#[derive(Clone)]
pub(crate) enum DataSourceFactoryInner {
    InMem(crate::engine::source::mem_reader::InMemGeneratedDataSourceHandle),
    Ion(crate::engine::source::ion_reader::IonDataSourceFactory),
}

impl DataSourceFactoryInner {
    pub(crate) fn buffer_stability(&self) -> BufferStability {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.buffer_stability(),
            DataSourceFactoryInner::Ion(factory) => factory.buffer_stability(),
        }
    }

    pub(crate) fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.resolve(field_name),
            DataSourceFactoryInner::Ion(factory) => factory.resolve(field_name),
        }
    }
}
