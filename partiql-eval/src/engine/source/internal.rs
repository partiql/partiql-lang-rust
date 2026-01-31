use super::RegisterWriter;
use crate::engine::error::Result;
use crate::engine::source::api::{DataSource, DataSourceFactory, ScanCapabilities, ScanSource};
use crate::engine::source::ion_reader::IonDataSource;
use crate::engine::source::mem_reader::InMemGeneratedReader;

/// Internal enum for row reader implementations
///
/// This enum enables static dispatch for known reader types (InMem, Ion).
/// For custom data sources, use the two-phase catalog pattern with
/// CompilationCatalog + ExecutionCatalog instead.
pub(crate) enum DataSourceImpl {
    InMem(InMemGeneratedReader),
    Ion(IonDataSource),
    Catalog(Box<dyn DataSource>),
}

impl DataSourceImpl {
    pub fn open(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::InMem(r) => r.open(),
            DataSourceImpl::Ion(r) => r.open(),
            DataSourceImpl::Catalog(r) => r.open(),
        }
    }

    pub fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        match self {
            DataSourceImpl::InMem(r) => r.next_row(writer),
            DataSourceImpl::Ion(r) => r.next_row(writer),
            DataSourceImpl::Catalog(r) => r.next_row(writer),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::InMem(r) => r.close(),
            DataSourceImpl::Ion(r) => r.close(),
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
    pub(crate) fn caps(&self) -> ScanCapabilities {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.caps(),
            DataSourceFactoryInner::Ion(factory) => factory.caps(),
        }
    }

    pub(crate) fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.resolve(field_name),
            DataSourceFactoryInner::Ion(factory) => factory.resolve(field_name),
        }
    }
}
