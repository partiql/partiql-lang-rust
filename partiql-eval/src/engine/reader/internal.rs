use crate::engine::error::Result;
use crate::engine::reader::api::{ReaderCaps, RowReader, RowReaderFactory, ScanSource};
use crate::engine::reader::ion_reader::IonRowReader;
use crate::engine::reader::mem_reader::InMemGeneratedReader;
use crate::engine::row::ValueWriter;

/// Internal enum for row reader implementations
///
/// This enum enables static dispatch for known reader types (InMem, Ion)
/// while still supporting custom readers via dynamic dispatch.
/// This avoids vtable overhead for the common cases.
pub(crate) enum ReaderImpl {
    InMem(InMemGeneratedReader),
    Ion(IonRowReader),
    Custom(Box<dyn crate::engine::reader::api::RowReader>),
}

impl ReaderImpl {
    pub fn open(&mut self) -> Result<()> {
        match self {
            ReaderImpl::InMem(r) => r.open(),
            ReaderImpl::Ion(r) => r.open(),
            ReaderImpl::Custom(r) => r.open(),
        }
    }

    pub fn next_row(&mut self, writer: &mut ValueWriter<'_, '_>) -> Result<bool> {
        match self {
            ReaderImpl::InMem(r) => r.next_row(writer),
            ReaderImpl::Ion(r) => r.next_row(writer),
            ReaderImpl::Custom(r) => r.next_row(writer),
        }
    }

    pub fn close(&mut self) -> Result<()> {
        match self {
            ReaderImpl::InMem(r) => r.close(),
            ReaderImpl::Ion(r) => r.close(),
            ReaderImpl::Custom(r) => r.close(),
        }
    }
}

/// Internal enum for reader factory implementations
#[derive(Clone)]
pub(crate) enum ReaderFactoryInner {
    InMem(crate::engine::reader::mem_reader::InMemGeneratedReaderFactory),
    Ion(crate::engine::reader::ion_reader::IonRowReaderFactory),
    Custom(std::sync::Arc<dyn crate::engine::reader::api::RowReaderFactory>),
}

impl ReaderFactoryInner {
    pub(crate) fn caps(&self) -> ReaderCaps {
        match self {
            ReaderFactoryInner::InMem(factory) => factory.caps(),
            ReaderFactoryInner::Ion(factory) => factory.caps(),
            ReaderFactoryInner::Custom(factory) => factory.caps(),
        }
    }

    pub(crate) fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        match self {
            ReaderFactoryInner::InMem(factory) => factory.resolve(field_name),
            ReaderFactoryInner::Ion(factory) => factory.resolve(field_name),
            ReaderFactoryInner::Custom(factory) => factory.resolve(field_name),
        }
    }
}
