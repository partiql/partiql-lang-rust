use crate::batch::LogicalType;
use crate::error::EvalError;

/// Phase 0 projection specification - defines what data to read and where to place it
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionSpec {
    pub projections: Vec<Projection>,
}

impl ProjectionSpec {
    /// Create a new projection specification
    pub fn new(projections: Vec<Projection>) -> Result<Self, EvalError> {
        let spec = Self { projections };
        spec.validate()?;
        Ok(spec)
    }

    /// Validate projection specification invariants
    fn validate(&self) -> Result<(), EvalError> {
        // Check that target vector indices are unique and contiguous from 0
        let mut indices: Vec<usize> = self.projections.iter()
            .map(|p| p.target_vector_idx)
            .collect();
        
        // Check uniqueness before sorting
        let mut unique_indices = indices.clone();
        unique_indices.sort_unstable();
        unique_indices.dedup();
        if unique_indices.len() != indices.len() {
            return Err(EvalError::General(
                "Duplicate target vector indices found".to_string()
            ));
        }

        indices.sort_unstable();

        // Check contiguous from 0
        for (i, &idx) in indices.iter().enumerate() {
            if idx != i {
                return Err(EvalError::General(format!(
                    "Projection target vector indices must be contiguous starting from 0. Expected {}, found {}",
                    i, idx
                )));
            }
        }

        Ok(())
    }

    /// Get the number of output vectors this projection will produce
    pub fn output_vector_count(&self) -> usize {
        self.projections.len()
    }
}

/// A single projection - maps from a source to a target vector with a declared type
#[derive(Debug, Clone, PartialEq)]
pub struct Projection {
    /// Where to read from
    pub source: ProjectionSource,
    /// Target vector index in the output batch
    pub target_vector_idx: usize,
    /// Declared scalar type (hard constraint)
    pub logical_type: LogicalType,
}

impl Projection {
    /// Create a new projection
    pub fn new(
        source: ProjectionSource,
        target_vector_idx: usize,
        logical_type: LogicalType,
    ) -> Self {
        Self {
            source,
            target_vector_idx,
            logical_type,
        }
    }
}

/// Source of data for a projection
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionSource {
    /// Direct column access (columnar readers only)
    ColumnIndex(usize),
    /// Flat field access by name (row / struct readers)
    /// Phase 0 supports: flat names ("field") and single-level nesting ("struct.field")
    /// Phase 0 does NOT support: deep nesting ("a.b.c", "a.b.c.d", etc.)
    FieldPath(String),
}

/// Builder for creating ProjectionSpec - convenience for compiler team
#[derive(Debug, Default)]
pub struct ProjectionBuilder {
    projections: Vec<Projection>,
}

impl ProjectionBuilder {
    /// Create a new projection builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a column index projection (for Parquet, Arrow)
    pub fn add_column(mut self, column_idx: usize, logical_type: LogicalType) -> Self {
        let target_idx = self.projections.len();
        self.projections.push(Projection::new(
            ProjectionSource::ColumnIndex(column_idx),
            target_idx,
            logical_type,
        ));
        self
    }

    /// Add a field path projection (for Ion, Tuples)
    pub fn add_field<S: Into<String>>(mut self, field_path: S, logical_type: LogicalType) -> Self {
        let target_idx = self.projections.len();
        self.projections.push(Projection::new(
            ProjectionSource::FieldPath(field_path.into()),
            target_idx,
            logical_type,
        ));
        self
    }

    /// Build the final ProjectionSpec
    pub fn build(self) -> Result<ProjectionSpec, EvalError> {
        ProjectionSpec::new(self.projections)
    }
}

/// Scalar types supported in Phase 0 - reuse existing LogicalType from batch module

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_spec_validation_success() {
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("a".to_string()), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath("b".to_string()), 1, LogicalType::String),
            Projection::new(ProjectionSource::ColumnIndex(2), 2, LogicalType::Float64),
        ];
        
        let spec = ProjectionSpec::new(projections).unwrap();
        assert_eq!(spec.output_vector_count(), 3);
    }

    #[test]
    fn test_projection_spec_validation_non_contiguous() {
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("a".to_string()), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath("b".to_string()), 2, LogicalType::String), // Skip 1
        ];
        
        let result = ProjectionSpec::new(projections);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("contiguous"));
    }

    #[test]
    fn test_projection_spec_validation_duplicate_indices() {
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("a".to_string()), 0, LogicalType::Int64),
            Projection::new(ProjectionSource::FieldPath("b".to_string()), 0, LogicalType::String), // Duplicate
        ];
        
        let result = ProjectionSpec::new(projections);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_projection_spec_validation_not_starting_from_zero() {
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("a".to_string()), 1, LogicalType::Int64), // Should start from 0
            Projection::new(ProjectionSource::FieldPath("b".to_string()), 2, LogicalType::String),
        ];
        
        let result = ProjectionSpec::new(projections);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected 0, found 1"));
    }

    #[test]
    fn test_projection_builder() {
        // Test column-based projection (Parquet/Arrow style)
        let spec = ProjectionBuilder::new()
            .add_column(0, LogicalType::Int64)
            .add_column(2, LogicalType::String)
            .add_column(1, LogicalType::Float64)
            .build()
            .unwrap();
        
        assert_eq!(spec.output_vector_count(), 3);
        assert_eq!(spec.projections[0].source, ProjectionSource::ColumnIndex(0));
        assert_eq!(spec.projections[1].source, ProjectionSource::ColumnIndex(2));
        assert_eq!(spec.projections[2].source, ProjectionSource::ColumnIndex(1));
    }

    #[test]
    fn test_projection_builder_field_paths() {
        // Test field-based projection (Ion/Tuple style)
        let spec = ProjectionBuilder::new()
            .add_field("name", LogicalType::String)
            .add_field("person.age", LogicalType::Int64)
            .add_field("score", LogicalType::Float64)
            .build()
            .unwrap();
        
        assert_eq!(spec.output_vector_count(), 3);
        assert_eq!(spec.projections[0].source, ProjectionSource::FieldPath("name".to_string()));
        assert_eq!(spec.projections[1].source, ProjectionSource::FieldPath("person.age".to_string()));
        assert_eq!(spec.projections[2].source, ProjectionSource::FieldPath("score".to_string()));
    }
}