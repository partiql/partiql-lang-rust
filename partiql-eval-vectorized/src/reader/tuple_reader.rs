use crate::batch::VectorizedBatch;
use crate::error::EvalError;
use crate::reader::{BatchReader, ProjectionSpec, ProjectionSource};
use partiql_value::{Value, BindingsName};
use std::borrow::Cow;

/// Convert PartiQL Value/Tuple stream to columnar batches
pub struct TupleIteratorReader {
    iter: Box<dyn Iterator<Item = Value>>,
    projection: Option<ProjectionSpec>,
    batch_size: usize,
    finished: bool,
}

impl TupleIteratorReader {
    /// Create new reader from an iterator of PartiQL Values (typically tuples)
    pub fn new(
        iter: Box<dyn Iterator<Item = Value>>,
        batch_size: usize,
    ) -> Self {
        Self {
            iter,
            projection: None,
            batch_size,
            finished: false,
        }
    }
}

impl BatchReader for TupleIteratorReader {
    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate that all projections use FieldPath (not ColumnIndex)
        for proj in &spec.projections {
            match &proj.source {
                ProjectionSource::FieldPath(_) => {
                    // Valid for tuple reader
                }
                ProjectionSource::ColumnIndex(idx) => {
                    return Err(EvalError::General(format!(
                        "TupleIteratorReader does not support ColumnIndex projections. \
                        Found ColumnIndex({}) at target vector index {}. \
                        Use FieldPath for tuple field access.",
                        idx, proj.target_vector_idx
                    )));
                }
            }
        }
        
        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        // Check if projection has been set
        let projection = self.projection.as_ref()
            .ok_or_else(|| EvalError::General("set_projection must be called before next_batch".to_string()))?;

        // Check if we're already finished
        if self.finished {
            return Ok(None);
        }
        
        // Collect tuples for this batch
        let mut batch_tuples = Vec::with_capacity(self.batch_size);
        for _ in 0..self.batch_size {
            match self.iter.next() {
                Some(value) => batch_tuples.push(value),
                None => {
                    self.finished = true;
                    break;
                }
            }
        }
        
        // If no tuples collected, we're done
        if batch_tuples.is_empty() {
            return Ok(None);
        }
        
        // Convert tuples to columnar batch
        let batch = convert_tuples_to_batch(&batch_tuples, projection)?;
        Ok(Some(batch))
    }
}

/// Convert a collection of PartiQL Values (tuples) to a columnar VectorizedBatch
fn convert_tuples_to_batch(
    tuples: &[Value],
    projection: &ProjectionSpec,
) -> Result<VectorizedBatch, EvalError> {
    use crate::batch::{PhysicalVectorEnum, LogicalType, SourceTypeDef, Field};
    
    let batch_size = tuples.len();
    
    // Create schema from projection
    let fields: Vec<Field> = projection.projections.iter()
        .map(|p| Field {
            name: match &p.source {
                ProjectionSource::FieldPath(path) => path.clone(),
                ProjectionSource::ColumnIndex(idx) => format!("col_{}", idx),
            },
            type_info: p.logical_type,
        })
        .collect();
    
    let schema = SourceTypeDef::new(fields);
    let mut batch = VectorizedBatch::new(schema, batch_size);
    
    // Extract values for each projection
    for proj in &projection.projections {
        let field_path = match &proj.source {
            ProjectionSource::FieldPath(path) => path,
            ProjectionSource::ColumnIndex(_) => {
                return Err(EvalError::General("ColumnIndex not supported for tuple reader".to_string()));
            }
        };
        
        let vector = batch.column_mut(proj.target_vector_idx)?;
        
        // Extract field values from all tuples
        match proj.logical_type {
            LogicalType::Int64 => {
                if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                    let slice = v.as_mut_slice();
                    for (i, tuple_value) in tuples.iter().enumerate() {
                        slice[i] = extract_int64_field(tuple_value, field_path)?;
                    }
                }
            }
            LogicalType::Float64 => {
                if let PhysicalVectorEnum::Float64(v) = &mut vector.physical {
                    let slice = v.as_mut_slice();
                    for (i, tuple_value) in tuples.iter().enumerate() {
                        slice[i] = extract_float64_field(tuple_value, field_path)?;
                    }
                }
            }
            LogicalType::Boolean => {
                if let PhysicalVectorEnum::Boolean(v) = &mut vector.physical {
                    let slice = v.as_mut_slice();
                    for (i, tuple_value) in tuples.iter().enumerate() {
                        slice[i] = extract_boolean_field(tuple_value, field_path)?;
                    }
                }
            }
            LogicalType::String => {
                if let PhysicalVectorEnum::String(v) = &mut vector.physical {
                    let slice = v.as_mut_slice();
                    for (i, tuple_value) in tuples.iter().enumerate() {
                        slice[i] = extract_string_field(tuple_value, field_path)?;
                    }
                }
            }
        }
    }
    
    batch.set_row_count(batch_size);
    Ok(batch)
}

/// Extract an Int64 field from a PartiQL Value (tuple)
fn extract_int64_field(value: &Value, field_path: &str) -> Result<i64, EvalError> {
    let field_value = extract_field_value(value, field_path)?;
    
    match field_value {
        Value::Integer(i) => Ok(*i),
        Value::Real(f) => Ok(f.into_inner() as i64), // Allow conversion from float
        Value::Null | Value::Missing => Ok(0), // Default value for missing fields
        _ => Err(EvalError::General(format!(
            "Cannot convert field '{}' to Int64. Found: {:?}",
            field_path, field_value
        ))),
    }
}

/// Extract a Float64 field from a PartiQL Value (tuple)
fn extract_float64_field(value: &Value, field_path: &str) -> Result<f64, EvalError> {
    let field_value = extract_field_value(value, field_path)?;
    
    match field_value {
        Value::Real(f) => Ok(f.into_inner()),
        Value::Integer(i) => Ok(*i as f64), // Allow conversion from int
        Value::Null | Value::Missing => Ok(0.0), // Default value for missing fields
        _ => Err(EvalError::General(format!(
            "Cannot convert field '{}' to Float64. Found: {:?}",
            field_path, field_value
        ))),
    }
}

/// Extract a Boolean field from a PartiQL Value (tuple)
fn extract_boolean_field(value: &Value, field_path: &str) -> Result<bool, EvalError> {
    let field_value = extract_field_value(value, field_path)?;
    
    match field_value {
        Value::Boolean(b) => Ok(*b),
        Value::Null | Value::Missing => Ok(false), // Default value for missing fields
        _ => Err(EvalError::General(format!(
            "Cannot convert field '{}' to Boolean. Found: {:?}",
            field_path, field_value
        ))),
    }
}

/// Extract a String field from a PartiQL Value (tuple)
fn extract_string_field(value: &Value, field_path: &str) -> Result<String, EvalError> {
    let field_value = extract_field_value(value, field_path)?;
    
    match field_value {
        Value::String(s) => Ok((**s).clone()),
        Value::Integer(i) => Ok(i.to_string()), // Allow conversion from int
        Value::Real(f) => Ok(f.to_string()), // Allow conversion from float
        Value::Boolean(b) => Ok(b.to_string()), // Allow conversion from boolean
        Value::Null => Ok("null".to_string()),
        Value::Missing => Ok("".to_string()), // Empty string for missing fields
        _ => Err(EvalError::General(format!(
            "Cannot convert field '{}' to String. Found: {:?}",
            field_path, field_value
        ))),
    }
}

/// Extract a field value from a PartiQL Value using a field path
/// Supports simple field names and single-level nesting (e.g., "struct.field")
fn extract_field_value<'a>(value: &'a Value, field_path: &str) -> Result<&'a Value, EvalError> {
    // Handle single-level nesting (e.g., "struct.field")
    if field_path.contains('.') {
        let parts: Vec<&str> = field_path.split('.').collect();
        if parts.len() != 2 {
            return Err(EvalError::General(format!(
                "TupleIteratorReader only supports single-level nesting. \
                Found path: '{}'. Use format 'struct.field'.",
                field_path
            )));
        }
        
        let struct_name = parts[0];
        let field_name = parts[1];
        
        // First extract the struct
        let struct_value = extract_simple_field(value, struct_name)?;
        
        // Then extract the field from the struct
        extract_simple_field(struct_value, field_name)
    } else {
        // Simple field access
        extract_simple_field(value, field_path)
    }
}

/// Extract a simple field from a PartiQL Value (no nesting)
fn extract_simple_field<'a>(value: &'a Value, field_name: &str) -> Result<&'a Value, EvalError> {
    match value {
        Value::Tuple(tuple) => {
            // Use the get method to find the field
            let binding_name = BindingsName::CaseInsensitive(Cow::Borrowed(field_name));
            match tuple.get(&binding_name) {
                Some(field_value) => Ok(field_value),
                None => Ok(&Value::Missing), // Field not found - return Missing
            }
        }
        _ => Err(EvalError::General(format!(
            "Cannot extract field '{}' from non-tuple value: {:?}",
            field_name, value
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::{Projection, ProjectionSource, ProjectionSpec};
    use crate::batch::LogicalType;
    use partiql_value::Value;

    #[test]
    fn test_tuple_reader_basic() {
        // Create test tuples
        let tuples = vec![
            Value::Tuple(Box::new(partiql_value::Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
                ("age", Value::Integer(30)),
                ("score", Value::Real(95.5.into())),
                ("active", Value::Boolean(true)),
            ]))),
            Value::Tuple(Box::new(partiql_value::Tuple::from([
                ("name", Value::String(Box::new("Bob".to_string()))),
                ("age", Value::Integer(25)),
                ("score", Value::Real(87.2.into())),
                ("active", Value::Boolean(false)),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        // Set projection
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch
        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 2);

        // Verify no more batches
        assert!(reader.next_batch().unwrap().is_none());
    }

    #[test]
    fn test_tuple_reader_missing_fields() {
        // Create test tuples with missing fields
        let tuples = vec![
            Value::Tuple(Box::new(partiql_value::Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
                ("age", Value::Integer(30)),
            ]))),
            Value::Tuple(Box::new(partiql_value::Tuple::from([
                ("name", Value::String(Box::new("Bob".to_string()))),
                // Missing age field
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        // Set projection
        let projections = vec![
            Projection::new(ProjectionSource::FieldPath("name".to_string()), 0, LogicalType::String),
            Projection::new(ProjectionSource::FieldPath("age".to_string()), 1, LogicalType::Int64),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        reader.set_projection(projection_spec).unwrap();

        // Read batch - should handle missing fields gracefully
        let batch = reader.next_batch().unwrap().unwrap();
        assert_eq!(batch.row_count(), 2);
    }

    #[test]
    fn test_tuple_reader_column_index_rejection() {
        let tuples = vec![
            Value::Tuple(Box::new(partiql_value::Tuple::from([
                ("name", Value::String(Box::new("Alice".to_string()))),
            ]))),
        ];

        let mut reader = TupleIteratorReader::new(Box::new(tuples.into_iter()), 10);

        // Try to set projection with ColumnIndex - should fail
        let projections = vec![
            Projection::new(ProjectionSource::ColumnIndex(0), 0, LogicalType::String),
        ];
        let projection_spec = ProjectionSpec::new(projections).unwrap();
        
        let result = reader.set_projection(projection_spec);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("ColumnIndex"));
    }
}
