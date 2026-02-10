use crate::engine::error::Result;
use crate::engine::row::SlotId;

/// Indicates how long data in a buffer remains valid after a read operation.
#[derive(Clone, Copy, Debug)]
pub enum BufferStability {
    /// Buffer is valid only until the next read operation
    UntilNext,
    /// Buffer remains valid until the data source is closed
    UntilClose,
}

/// Physical type information for compile-time optimization.
///
/// Maps to `ValueRef` variants and enables type-specific code generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhysicalType {
    /// Missing value
    Missing,
    /// Null value
    Null,
    /// Boolean value
    Bool,
    /// 64-bit signed integer
    I64,
    /// 64-bit floating point
    F64,
    /// String reference
    Str,
    /// Byte slice reference
    Bytes,
    /// Dynamic type - must be examined at runtime (maps to ValueRef::Owned)
    Dynamic,
}

/// Describes how to access a field from the data source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScanSourceType {
    /// Access by column index (zero-based)
    ColumnIndex(usize),
    /// Access by field path (string key)
    FieldPath(String),
    /// Return the entire row/value as-is
    WholeValue,
}

/// Resolved field access information combining source type and physical type.
///
/// Returned by `DataSourceConfig::resolve()` to provide compile-time
/// information about how to access a field and what type to expect.
#[derive(Clone, Debug)]
pub struct ScanSource {
    /// How to access the field (by index, path, or whole value)
    pub source_type: ScanSourceType,
    /// Expected physical type for optimization
    pub physical_type: PhysicalType,
}

impl ScanSource {
    /// Create a new ScanSource with the given source type and physical type.
    pub fn new(source_type: ScanSourceType, physical_type: PhysicalType) -> Self {
        ScanSource {
            source_type,
            physical_type,
        }
    }

    /// Create a column index source with the given type.
    pub fn column(index: usize, physical_type: PhysicalType) -> Self {
        ScanSource {
            source_type: ScanSourceType::ColumnIndex(index),
            physical_type,
        }
    }

    /// Create a field path source with the given type.
    pub fn field(path: impl Into<String>, physical_type: PhysicalType) -> Self {
        ScanSource {
            source_type: ScanSourceType::FieldPath(path.into()),
            physical_type,
        }
    }

    /// Create a whole value source with dynamic type.
    pub fn whole_value() -> Self {
        ScanSource {
            source_type: ScanSourceType::WholeValue,
            physical_type: PhysicalType::Dynamic,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ScanLayout {
    pub projections: Vec<ScanProjection>,
}

impl ScanLayout {
    pub fn base_row() -> Self {
        ScanLayout {
            projections: vec![ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: 0,
            }],
        }
    }

    pub fn is_base_row_only(&self) -> bool {
        self.projections.len() == 1
            && matches!(
                self.projections[0].source.source_type,
                ScanSourceType::WholeValue
            )
            && self.projections[0].target_slot == 0
    }
}

#[derive(Clone, Debug)]
pub struct ScanProjection {
    /// Source field access (includes physical type)
    pub source: ScanSource,
    /// Target register slot to write the value
    pub target_slot: SlotId,
}

pub trait DataSource {
    fn open(&mut self) -> Result<()>;
    fn next_row(&mut self, writer: &mut super::RegisterWriter<'_, '_>) -> Result<bool>;
    fn close(&mut self) -> Result<()>;
}

/// Compile-time metadata for a data source.
///
/// Provides buffer stability and field resolution without coupling to execution-time data access.
/// Used by DataSourceHandle to enable compile-time optimizations and validations.
pub trait DataSourceConfig: Send + Sync {
    /// Get the buffer stability for this data source.
    ///
    /// Indicates how long data in buffers remains valid after read operations.
    fn buffer_stability(&self) -> BufferStability;

    /// Resolve a field name to ScanSource at compile time.
    ///
    /// Returns `Some(ScanSource)` if the field can be provided, `None` otherwise.
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
}

pub trait DataSourceFactory: Send + Sync {
    fn create(&self, layout: ScanLayout) -> Result<Box<dyn DataSource>>;
    fn buffer_stability(&self) -> BufferStability;
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
}
