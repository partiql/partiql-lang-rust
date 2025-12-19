use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::operators::VectorizedOperator;
use crate::reader::BatchReader;

/// Scan operator - reads data from source
pub struct VectorizedScan {
    reader: Box<dyn BatchReader>,
}

impl VectorizedScan {
    /// Create new scan operator
    pub fn new(reader: Box<dyn BatchReader>) -> Self {
        Self { reader }
    }
}

impl VectorizedOperator for VectorizedScan {
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        self.reader.next_batch()
    }
}
