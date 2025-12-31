use crate::batch::{SourceTypeDef, VectorizedBatch};
use crate::error::EvalError;
use crate::operators::VectorizedOperator;
use crate::reader::{BatchReader, ProjectionSource};

/// Scan operator - reads data from source
pub struct VectorizedScan {
    reader: Box<dyn BatchReader>,
    projections: Vec<ProjectionSource>,
    output_schema: SourceTypeDef,
}

impl VectorizedScan {
    /// Create new scan operator with projections
    pub fn new(
        reader: Box<dyn BatchReader>,
        projections: Vec<ProjectionSource>,
        output_schema: SourceTypeDef,
    ) -> Self {
        Self {
            reader,
            projections,
            output_schema,
        }
    }
}

impl VectorizedOperator for VectorizedScan {
    fn open(&mut self) -> Result<(), EvalError> {
        // Create Projection objects from ProjectionSource and schema
        let projections: Vec<crate::reader::Projection> = self
            .projections
            .iter()
            .enumerate()
            .map(|(idx, source)| {
                let logical_type = self.output_schema.fields()[idx].type_info;
                crate::reader::Projection::new(source.clone(), idx, logical_type)
            })
            .collect();
        
        // Create ProjectionSpec and set it on the reader
        let projection_spec = crate::reader::ProjectionSpec::new(projections)?;
        self.reader.set_projection(projection_spec)?;
        
        // Open the reader
        self.reader.open()
    }
    
    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        self.reader.next_batch()
    }
    
    fn output_schema(&self) -> &SourceTypeDef {
        &self.output_schema
    }
    
    fn close(&mut self) -> Result<(), EvalError> {
        self.reader.close()
    }
}
