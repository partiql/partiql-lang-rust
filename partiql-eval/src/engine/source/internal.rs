use super::RegisterWriter;
use crate::engine::error::Result;
use crate::engine::source::api::{DataSource, DataSourceFactory, ScanCapabilities, ScanSource};
use crate::engine::source::ion_reader::IonDataSource;
use crate::engine::source::mem_reader::InMemGeneratedReader;

/// Internal enum for row reader implementations
///
/// This enum enables static dispatch for known reader types (InMem, Ion)
/// while still supporting custom readers via dynamic dispatch.
/// This avoids vtable overhead for the common cases.
pub(crate) enum DataSourceImpl {
    InMem(InMemGeneratedReader),
    Ion(IonDataSource),
    Custom(Box<dyn crate::engine::source::api::DataSource>),
}

impl DataSourceImpl {
    pub fn open(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::InMem(r) => r.open(),
            DataSourceImpl::Ion(r) => r.open(),
            DataSourceImpl::Custom(r) => r.open(),
        }
    }

    pub fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
        match self {
            DataSourceImpl::InMem(r) => r.next_row(writer),
            DataSourceImpl::Ion(r) => r.next_row(writer),
            DataSourceImpl::Custom(r) => r.next_row(writer),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            DataSourceImpl::InMem(r) => r.close(),
            DataSourceImpl::Ion(r) => r.close(),
            DataSourceImpl::Custom(r) => r.close(),
        }
    }
}

/// Internal enum for reader factory implementations
#[derive(Clone)]
pub(crate) enum DataSourceFactoryInner {
    InMem(crate::engine::source::mem_reader::InMemGeneratedDataSourceHandle),
    Ion(crate::engine::source::ion_reader::IonDataSourceFactory),
    Custom(std::sync::Arc<dyn crate::engine::source::api::DataSourceFactory>),
}

impl DataSourceFactoryInner {
    pub(crate) fn caps(&self) -> ScanCapabilities {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.caps(),
            DataSourceFactoryInner::Ion(factory) => factory.caps(),
            DataSourceFactoryInner::Custom(factory) => factory.caps(),
        }
    }

    pub(crate) fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match self {
            DataSourceFactoryInner::InMem(factory) => factory.resolve(field_name),
            DataSourceFactoryInner::Ion(factory) => factory.resolve(field_name),
            DataSourceFactoryInner::Custom(factory) => factory.resolve(field_name),
        }
    }
}
