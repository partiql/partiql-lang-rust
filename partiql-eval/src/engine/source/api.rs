use crate::engine::arena::SlotId;
use crate::engine::error::Result;

/// Indicates how long data in a buffer remains valid after a read operation.
///
/// Used by the compiler to make optimization decisions about data handling.
/// For example, if data is only valid until the next read (`UntilNext`), the
/// compiler may need to copy values before advancing.
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
/// Used by `DataSourceMetadata::resolve()` to inform the compiler about
/// expected physical types for each field.
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
///
/// Returned as part of `ScanSource` from `DataSourceMetadata::resolve()` to
/// tell the compiler how to extract a particular field from the data source.
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
/// Returned by `DataSourceMetadata::resolve()` to provide compile-time
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

/// Describes how to project fields from a data source during a scan.
///
/// Built by the compiler during query compilation based on the fields
/// required by the query and the capabilities of the data source.
#[derive(Clone, Debug, Default)]
pub struct ScanLayout {
    /// The list of projections to apply during scanning
    pub projections: Vec<ScanProjection>,
}

impl ScanLayout {
    /// Create a layout that returns the entire row as a single value.
    pub fn base_row() -> Self {
        ScanLayout {
            projections: vec![ScanProjection {
                source: ScanSource::whole_value(),
                target_slot: 0,
            }],
        }
    }

    /// Check if this layout only requests the base row without projections.
    pub fn is_base_row_only(&self) -> bool {
        self.projections.len() == 1
            && matches!(
                self.projections[0].source.source_type,
                ScanSourceType::WholeValue
            )
            && self.projections[0].target_slot == 0
    }
}

/// A single projection mapping a source field to a target slot.
///
/// Used within `ScanLayout` to describe how to extract a field from
/// the source data and where to store it in the execution registers.
#[derive(Clone, Debug)]
pub struct ScanProjection {
    /// Source field access (includes physical type)
    pub source: ScanSource,
    /// Target register slot to write the value
    pub target_slot: SlotId,
}

/// Runtime row iterator for reading data during query execution.
///
/// Implemented by customers to provide row-by-row data iteration. The VM
/// calls `open()` once, then `next_row()` repeatedly until it returns `false`,
/// then `close()` when done.
///
/// # Example
/// ```ignore
/// struct MyDataSource {
///     rows: Vec<MyRow>,
///     current: usize,
/// }
///
/// impl DataSource for MyDataSource {
///     fn open(&mut self) -> Result<()> {
///         self.current = 0;
///         Ok(())
///     }
///
///     fn next_row(&mut self, writer: &mut RegisterWriter<'_, '_>) -> Result<bool> {
///         if self.current >= self.rows.len() {
///             return Ok(false);
///         }
///         let row = &self.rows[self.current];
///         writer.put_i64(0, row.id)?;
///         writer.put_str(1, &row.name)?;
///         self.current += 1;
///         Ok(true)
///     }
///
///     fn close(&mut self) -> Result<()> {
///         Ok(())
///     }
/// }
/// ```
pub trait DataSource {
    /// Initialize the data source for reading.
    ///
    /// Called once before the first `next_row()` call.
    fn open(&mut self) -> Result<()>;

    /// Read the next row and write values to the register writer.
    ///
    /// Returns `Ok(true)` if a row was read, `Ok(false)` if no more rows.
    /// The writer is used to populate the target slots specified in the
    /// `ScanLayout` that was passed when creating this data source.
    fn next_row(&mut self, writer: &mut super::RegisterWriter<'_, '_>) -> Result<bool>;

    /// Close the data source and release any resources.
    ///
    /// Called once after all rows have been read or on error.
    fn close(&mut self) -> Result<()>;
}

/// Compile-time metadata for a data source.
///
/// Implemented by customers to provide compile-time information about a data
/// source without coupling to execution-time data access. This enables the
/// compiler to:
/// - Optimize buffer handling based on stability guarantees
/// - Resolve field names to physical access patterns
/// - Validate queries against the schema
///
/// # Example
/// ```ignore
/// struct MyTableMetadata {
///     columns: Vec<String>,
/// }
///
/// impl DataSourceMetadata for MyTableMetadata {
///     fn buffer_stability(&self) -> BufferStability {
///         BufferStability::UntilNext
///     }
///
///     fn resolve(&self, field_name: &str) -> Option<ScanSource> {
///         self.columns
///             .iter()
///             .position(|name| name.eq_ignore_ascii_case(field_name))
///             .map(|index| ScanSource::column(index, PhysicalType::Dynamic))
///     }
/// }
/// ```
pub trait DataSourceMetadata: Send + Sync {
    /// Get the buffer stability for this data source.
    ///
    /// Indicates how long data in buffers remains valid after read operations.
    /// This allows the compiler to make informed decisions about data handling.
    fn buffer_stability(&self) -> BufferStability;

    /// Resolve a field name to ScanSource at compile time.
    ///
    /// Returns `Some(ScanSource)` if the field can be provided, `None` otherwise.
    /// Used by the compiler to determine how to access each required field.
    fn resolve(&self, field_name: &str) -> Option<ScanSource>;
}
