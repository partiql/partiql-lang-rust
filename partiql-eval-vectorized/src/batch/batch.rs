use crate::batch::{LogicalType, PhysicalVectorEnum, SourceTypeDef, Vector};
use crate::error::EvalError;

/// Selection vector for filtered batches
/// Maps logical row indices to physical row indices
#[derive(Debug, Clone)]
pub struct SelectionVector {
    pub indices: Vec<usize>,
}

/// Batch of columnar data
#[derive(Debug, Clone)]
pub struct VectorizedBatch {
    columns: Vec<Vector>,
    row_count: usize,
    schema: SourceTypeDef,
    /// Number of source columns (non-scratch)
    source_column_count: usize,
    /// Optional selection vector for filtering
    selection: Option<SelectionVector>,
}

impl VectorizedBatch {
    /// Create new batch with pre-allocated columns
    pub fn new(schema: SourceTypeDef, capacity: usize) -> Self {
        let columns: Vec<Vector> = schema
            .fields()
            .iter()
            .map(|f| Vector::new(f.type_info, capacity))
            .collect();

        let source_column_count = columns.len();

        Self {
            columns,
            row_count: 0,
            schema,
            source_column_count,
            selection: None,
        }
    }

    /// Get number of rows in batch
    pub fn row_count(&self) -> usize {
        self.row_count
    }

    /// Set row count
    pub fn set_row_count(&mut self, count: usize) {
        self.row_count = count;
    }

    /// Get column by index
    pub fn column(&self, idx: usize) -> Result<&Vector, EvalError> {
        self.columns
            .get(idx)
            .ok_or(EvalError::InvalidColumnIndex(idx))
    }

    /// Get mutable column by index
    pub fn column_mut(&mut self, idx: usize) -> Result<&mut Vector, EvalError> {
        self.columns
            .get_mut(idx)
            .ok_or(EvalError::InvalidColumnIndex(idx))
    }

    /// Get schema
    pub fn schema(&self) -> &SourceTypeDef {
        &self.schema
    }

    /// Get selection vector
    pub fn selection(&self) -> Option<&SelectionVector> {
        self.selection.as_ref()
    }

    /// Set selection vector
    pub fn set_selection(&mut self, selection: Option<SelectionVector>) {
        self.selection = selection;
    }

    /// Clear all columns, retaining capacity
    pub fn clear(&mut self) {
        for col in &mut self.columns {
            match &mut col.physical {
                PhysicalVectorEnum::Int64(v) => v.clear(),
                PhysicalVectorEnum::Float64(v) => v.clear(),
                PhysicalVectorEnum::Boolean(v) => v.clear(),
                PhysicalVectorEnum::String(v) => v.clear(),
            }
        }
        self.row_count = 0;
        self.selection = None;
    }

    /// Allocate a scratch column for intermediate expression results
    /// Returns the column index where the scratch column was added
    pub fn add_scratch_column(&mut self, type_info: LogicalType) -> usize {
        let capacity = self.columns.first().map(|c| c.len()).unwrap_or(1024);
        let scratch_col = Vector::new(type_info, capacity);
        self.columns.push(scratch_col);
        self.columns.len() - 1
    }

    /// Get the number of source columns (non-scratch)
    pub fn source_column_count(&self) -> usize {
        self.source_column_count
    }

    /// Get total number of columns (source + scratch)
    pub fn total_column_count(&self) -> usize {
        self.columns.len()
    }

    /// Clear only scratch columns, keeping source columns
    pub fn clear_scratch_columns(&mut self) {
        // Only clear columns beyond source_column_count
        for col in self.columns.iter_mut().skip(self.source_column_count) {
            match &mut col.physical {
                PhysicalVectorEnum::Int64(v) => v.clear(),
                PhysicalVectorEnum::Float64(v) => v.clear(),
                PhysicalVectorEnum::Boolean(v) => v.clear(),
                PhysicalVectorEnum::String(v) => v.clear(),
            }
        }
    }
}
