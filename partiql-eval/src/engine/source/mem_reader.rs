use crate::engine::error::{EngineError, Result};
use crate::engine::source::api::{
    BufferStability, DataSource, DataSourceFactory, ScanCapabilities, ScanLayout, ScanSource,
};

/// In-memory row reader that generates rows on-the-fly
///
/// Similar to InMemoryGeneratedReader in vectorized evaluation, but operates on single rows.
/// Generates fake columnar data with Int64 columns. All columns start at 0 and increment
/// by 1 for each row.
///
/// Note: Does not support BaseRow projection - only field-level projections
pub struct InMemGeneratedReader {
    current_row: i64,
    total_rows: usize,
    layout: ScanLayout,
    num_columns: usize,
}

impl InMemGeneratedReader {
    pub fn new(total_rows: usize, num_columns: usize, layout: ScanLayout) -> Self {
        InMemGeneratedReader {
            current_row: 0,
            total_rows,
            layout,
            num_columns,
        }
    }
}

impl DataSource for InMemGeneratedReader {
    fn open(&mut self) -> Result<()> {
        self.current_row = 0;
        Ok(())
    }

    fn next_row(&mut self, writer: &mut super::RegisterWriter<'_, '_>) -> Result<bool> {
        // Check if we've generated all rows
        if self.current_row >= self.total_rows as i64 {
            return Ok(false);
        }

        // Generate row values on-the-fly
        // All columns start at 0 and increment by 1 for each row
        let row_value = self.current_row;

        // Populate slot registers based on projection layout
        // InMem reader ONLY supports ColumnIndex projections
        for proj in &self.layout.projections {
            let target = proj.target_slot;

            match &proj.source {
                ScanSource::ColumnIndex(index) => {
                    // All columns get the same value: current_row (starting at 0)
                    if *index < self.num_columns {
                        writer.put_i64(target, row_value)?;
                    } else {
                        return Err(EngineError::ReaderError(format!(
                            "Column index {} is out of bounds (max: {})",
                            index,
                            self.num_columns - 1
                        )));
                    }
                }
                ScanSource::BaseRow | ScanSource::FieldPath(_) => {
                    return Err(EngineError::UnsupportedExpr(
                        "InMem reader only supports ColumnIndex projections".to_string(),
                    ));
                }
            };
        }

        // Increment current row for next call
        self.current_row += 1;
        Ok(true)
    }

    fn close(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct InMemGeneratedDataSourceHandle {
    pub(crate) total_rows: usize,
    pub(crate) column_names: Vec<String>,
}

impl InMemGeneratedDataSourceHandle {
    pub fn new(total_rows: usize, column_names: Vec<String>) -> Self {
        InMemGeneratedDataSourceHandle {
            total_rows,
            column_names,
        }
    }
}

impl DataSourceFactory for InMemGeneratedDataSourceHandle {
    fn create(&self, layout: ScanLayout) -> Result<Box<dyn DataSource>> {
        // Validate that all projections are ColumnIndex (not FieldPath)
        for proj in &layout.projections {
            match &proj.source {
                ScanSource::ColumnIndex(_) => {
                    // Valid for InMem reader
                }
                ScanSource::BaseRow | ScanSource::FieldPath(_) => {
                    return Err(EngineError::ProjectionNotSupported(
                        "InMem reader only supports ColumnIndex projections",
                    ));
                }
            }
        }

        Ok(Box::new(InMemGeneratedReader::new(
            self.total_rows,
            self.column_names.len(),
            layout,
        )))
    }

    fn caps(&self) -> ScanCapabilities {
        ScanCapabilities {
            stability: BufferStability::UntilNext,
            can_project: true,
            can_return_opaque: false,
        }
    }

    fn resolve(&self, field_name: &str) -> Option<ScanSource> {
        // InMem reader only supports column indexes, not field paths
        // Map field names to their column indexes based on position in column_names
        self.column_names
            .iter()
            .position(|name| name == field_name)
            .map(ScanSource::ColumnIndex)
    }
}
