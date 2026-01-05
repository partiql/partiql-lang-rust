use crate::batch::{
    Field, LogicalType, PhysicalVectorEnum, SourceTypeDef, Vector, VectorizedBatch,
};
use crate::error::EvalError;
use crate::reader::error::{
    BatchReaderError, DataSourceError, ProjectionError, TypeConversionError,
};
use crate::reader::{BatchReader, ProjectionSource, ProjectionSpec};
use ion_rs::{Element, Value};

/// Phase 0 Ion reader implementation
///
/// Supports reading Ion data with FieldPath projections for flat field access.
/// Phase 0 constraints:
/// - Only scalar values (Int64, Float64, Boolean, String)
/// - FieldPath supports single-level nesting (e.g., "field" or "struct.field")
/// - No deep nesting ("a.b.c") - this will be supported in later phases
/// - Missing fields result in null values, not errors
///
/// # Memory Limitation
///
/// **WARNING**: This reader currently loads the entire Ion file and all elements into memory
/// using `Element::read_all()`. For large files (GB+), this defeats the purpose of
/// vectorized/streaming processing. The entire dataset must fit in memory before processing.
pub struct IonReader {
    /// Ion data elements to read from
    elements: Vec<Element>,
    /// Current position in the elements
    current_position: usize,
    /// Configured projection specification
    projection: Option<ProjectionSpec>,
    /// Batch size for output
    batch_size: usize,
    /// Cached schema built from projection (reused across batches)
    cached_schema: Option<SourceTypeDef>,
    /// Reusable batch structure (pre-allocated in set_projection)
    reusable_batch: Option<VectorizedBatch>,
}

impl IonReader {
    /// Create a new IonReader from Ion text data
    pub fn from_ion_text(ion_text: &str, batch_size: usize) -> Result<Self, EvalError> {
        let elements: Vec<Element> = Element::read_all(ion_text.as_bytes())
            .map_err(|e| {
                BatchReaderError::data_source(DataSourceError::initialization_failed(
                    "Ion",
                    &format!("Failed to parse Ion text: {}", e),
                ))
            })?
            .into();

        Ok(Self {
            elements,
            current_position: 0,
            projection: None,
            batch_size,
            cached_schema: None,
            reusable_batch: None,
        })
    }

    /// Create a new IonReader from Ion elements
    pub fn from_elements(elements: Vec<Element>, batch_size: usize) -> Self {
        Self {
            elements,
            current_position: 0,
            projection: None,
            batch_size,
            cached_schema: None,
            reusable_batch: None,
        }
    }

    /// Extract a scalar value from an Ion element based on field path
    fn extract_field_value(
        &self,
        element: &Element,
        field_path: &str,
    ) -> Result<Option<Value>, EvalError> {
        // Phase 0 supports single-level nesting: "field" or "struct.field"
        let path_parts: Vec<&str> = field_path.split('.').collect();

        if path_parts.len() > 2 {
            return Err(
                BatchReaderError::projection(ProjectionError::unsupported_source(
                    &format!("FieldPath({})", field_path),
                    "IonReader",
                    &["Single-level field paths like 'field' or 'struct.field'"],
                ))
                .into(),
            );
        }

        let current_element = element;
        let mut target_element = current_element;

        // Navigate to the target element
        for (i, part) in path_parts.iter().enumerate() {
            match target_element.value() {
                Value::Struct(struct_val) => {
                    match struct_val.get(part) {
                        Some(field_element) => {
                            target_element = field_element;
                        }
                        None => {
                            // Missing field - return None for null value
                            return Ok(None);
                        }
                    }
                }
                _ => {
                    if i == 0 && path_parts.len() == 1 {
                        // Direct field access on non-struct - this is an error
                        return Err(BatchReaderError::projection(
                            ProjectionError::source_not_found(
                                field_path,
                                &vec!["<non-struct element has no fields>".to_string()],
                            ),
                        )
                        .into());
                    } else {
                        // Nested access on non-struct - missing field
                        return Ok(None);
                    }
                }
            }
        }

        // Extract the scalar value
        Ok(Some(target_element.value().clone()))
    }

    /// Write Ion values directly to Int64 slice (eliminates Vector allocation)
    fn write_values_to_int64_slice(
        slice: &mut [i64],
        values: &[Option<Value>],
        field_path: &str,
        batch_size: usize,
    ) -> Result<(), EvalError> {
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            slice.len() >= batch_size,
            "Int64 vector buffer too small: expected {}, got {}",
            batch_size,
            slice.len()
        );
        
        for i in 0..batch_size {
            slice[i] = match &values[i] {
                Some(value) => {
                    match value {
                        Value::Int(int_val) => {
                            int_val.as_i64().ok_or_else(|| {
                                BatchReaderError::type_conversion(
                                    TypeConversionError::conversion_failed(
                                        field_path,
                                        "Ion Int",
                                        LogicalType::Int64,
                                        "Integer value too large for i64",
                                    ),
                                )
                            })?
                        }
                        Value::Null(_) => 0, // Default value, actual null is tracked separately
                        _ => {
                            return Err(BatchReaderError::type_conversion(
                                TypeConversionError::type_mismatch(
                                    field_path,
                                    &format!("{:?}", value.ion_type()),
                                    LogicalType::Int64,
                                    Some("Use explicit conversion or check data types"),
                                ),
                            )
                            .into());
                        }
                    }
                }
                None => 0, // Missing field - null value
            };
        }
        Ok(())
    }

    /// Write Ion values directly to Float64 slice (eliminates Vector allocation)
    fn write_values_to_float64_slice(
        slice: &mut [f64],
        values: &[Option<Value>],
        field_path: &str,
        batch_size: usize,
    ) -> Result<(), EvalError> {
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            slice.len() >= batch_size,
            "Float64 vector buffer too small: expected {}, got {}",
            batch_size,
            slice.len()
        );
        
        for i in 0..batch_size {
            slice[i] = match &values[i] {
                Some(value) => {
                    match value {
                        Value::Float(float_val) => *float_val,
                        Value::Int(int_val) => {
                            // Allow int to float conversion
                            int_val.as_i64().unwrap_or(0) as f64
                        }
                        Value::Decimal(decimal_val) => {
                            // Allow decimal to float conversion via string
                            let decimal_str = decimal_val.to_string();
                            decimal_str.parse::<f64>().map_err(|_| {
                                BatchReaderError::type_conversion(
                                    TypeConversionError::conversion_failed(
                                        field_path,
                                        "Ion Decimal",
                                        LogicalType::Float64,
                                        "Failed to convert decimal to f64",
                                    ),
                                )
                            })?
                        }
                        Value::Null(_) => 0.0, // Default value, actual null is tracked separately
                        _ => {
                            return Err(BatchReaderError::type_conversion(
                                TypeConversionError::type_mismatch(
                                    field_path,
                                    &format!("{:?}", value.ion_type()),
                                    LogicalType::Float64,
                                    Some("Use explicit conversion or check data types"),
                                ),
                            )
                            .into());
                        }
                    }
                }
                None => 0.0, // Missing field - null value
            };
        }
        Ok(())
    }

    /// Write Ion values directly to Boolean slice (eliminates Vector allocation)
    fn write_values_to_boolean_slice(
        slice: &mut [bool],
        values: &[Option<Value>],
        field_path: &str,
        batch_size: usize,
    ) -> Result<(), EvalError> {
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            slice.len() >= batch_size,
            "Boolean vector buffer too small: expected {}, got {}",
            batch_size,
            slice.len()
        );
        
        for i in 0..batch_size {
            slice[i] = match &values[i] {
                Some(value) => {
                    match value {
                        Value::Bool(bool_val) => *bool_val,
                        Value::Null(_) => false, // Default value, actual null is tracked separately
                        _ => {
                            return Err(BatchReaderError::type_conversion(
                                TypeConversionError::type_mismatch(
                                    field_path,
                                    &format!("{:?}", value.ion_type()),
                                    LogicalType::Boolean,
                                    Some("Use explicit conversion or check data types"),
                                ),
                            )
                            .into());
                        }
                    }
                }
                None => false, // Missing field - null value
            };
        }
        Ok(())
    }

    /// Write Ion values directly to String slice (eliminates Vector allocation)
    fn write_values_to_string_slice(
        slice: &mut [String],
        values: &[Option<Value>],
        field_path: &str,
        batch_size: usize,
    ) -> Result<(), EvalError> {
        // Verify capacity in debug builds
        #[cfg(debug_assertions)]
        debug_assert!(
            slice.len() >= batch_size,
            "String vector buffer too small: expected {}, got {}",
            batch_size,
            slice.len()
        );
        
        for i in 0..batch_size {
            slice[i] = match &values[i] {
                Some(value) => {
                    match value {
                        Value::String(string_val) => string_val.text().to_string(),
                        Value::Symbol(symbol_val) => {
                            // Allow symbol to string conversion
                            symbol_val.text().unwrap_or("").to_string()
                        }
                        Value::Null(_) => String::new(), // Default value, actual null is tracked separately
                        _ => {
                            return Err(BatchReaderError::type_conversion(
                                TypeConversionError::type_mismatch(
                                    field_path,
                                    &format!("{:?}", value.ion_type()),
                                    LogicalType::String,
                                    Some("Use explicit conversion or check data types"),
                                ),
                            )
                            .into());
                        }
                    }
                }
                None => String::new(), // Missing field - null value
            };
        }
        Ok(())
    }

    /// Convert Ion Value to PartiQL Vector based on LogicalType
    /// 
    /// NOTE: This method is kept for backward compatibility but is no longer used
    /// in the optimized path. The new direct-write methods (write_ion_to_*_slice)
    /// eliminate intermediate Vec allocations.
    fn convert_ion_values_to_vector(
        &self,
        values: Vec<Option<Value>>,
        logical_type: LogicalType,
        source_name: &str,
        batch_size: usize,
    ) -> Result<Vector, EvalError> {
        match logical_type {
            LogicalType::Int64 => {
                // Create vector and populate it
                let mut vector = Vector::new(LogicalType::Int64, batch_size);
                if let PhysicalVectorEnum::Int64(ref mut physical_vec) = vector.physical {
                    let slice = physical_vec.as_mut_slice();
                    for (i, value_opt) in values.iter().enumerate() {
                        match value_opt {
                            Some(value) => {
                                match value {
                                    Value::Int(int_val) => {
                                        slice[i] = int_val.as_i64().ok_or_else(|| {
                                            BatchReaderError::type_conversion(
                                                TypeConversionError::conversion_failed(
                                                    source_name,
                                                    "Ion Int",
                                                    LogicalType::Int64,
                                                    "Integer value too large for i64",
                                                ),
                                            )
                                        })?;
                                    }
                                    Value::Null(_) => {
                                        // Null values are handled by the physical vector's null bitmap
                                        slice[i] = 0; // Default value, actual null is tracked separately
                                    }
                                    _ => {
                                        return Err(BatchReaderError::type_conversion(
                                            TypeConversionError::type_mismatch(
                                                source_name,
                                                &format!("{:?}", value.ion_type()),
                                                LogicalType::Int64,
                                                Some("Use explicit conversion or check data types"),
                                            ),
                                        )
                                        .into());
                                    }
                                }
                            }
                            None => {
                                // Missing field - null value
                                slice[i] = 0; // Default value, actual null is tracked separately
                            }
                        }
                    }
                }
                Ok(vector)
            }
            LogicalType::Float64 => {
                let mut vector = Vector::new(LogicalType::Float64, batch_size);
                if let PhysicalVectorEnum::Float64(ref mut physical_vec) = vector.physical {
                    let slice = physical_vec.as_mut_slice();
                    for (i, value_opt) in values.iter().enumerate() {
                        match value_opt {
                            Some(value) => {
                                match value {
                                    Value::Float(float_val) => {
                                        slice[i] = *float_val;
                                    }
                                    Value::Int(int_val) => {
                                        // Allow int to float conversion
                                        slice[i] = int_val.as_i64().unwrap_or(0) as f64;
                                    }
                                    Value::Decimal(decimal_val) => {
                                        // Allow decimal to float conversion via string
                                        let decimal_str = decimal_val.to_string();
                                        slice[i] = decimal_str.parse::<f64>().map_err(|_| {
                                            BatchReaderError::type_conversion(
                                                TypeConversionError::conversion_failed(
                                                    source_name,
                                                    "Ion Decimal",
                                                    LogicalType::Float64,
                                                    "Failed to convert decimal to f64",
                                                ),
                                            )
                                        })?;
                                    }
                                    Value::Null(_) => {
                                        slice[i] = 0.0; // Default value, actual null is tracked separately
                                    }
                                    _ => {
                                        return Err(BatchReaderError::type_conversion(
                                            TypeConversionError::type_mismatch(
                                                source_name,
                                                &format!("{:?}", value.ion_type()),
                                                LogicalType::Float64,
                                                Some("Use explicit conversion or check data types"),
                                            ),
                                        )
                                        .into());
                                    }
                                }
                            }
                            None => {
                                slice[i] = 0.0; // Default value, actual null is tracked separately
                            }
                        }
                    }
                }
                Ok(vector)
            }
            LogicalType::Boolean => {
                let mut vector = Vector::new(LogicalType::Boolean, batch_size);
                if let PhysicalVectorEnum::Boolean(ref mut physical_vec) = vector.physical {
                    let slice = physical_vec.as_mut_slice();
                    for (i, value_opt) in values.iter().enumerate() {
                        match value_opt {
                            Some(value) => {
                                match value {
                                    Value::Bool(bool_val) => {
                                        slice[i] = *bool_val;
                                    }
                                    Value::Null(_) => {
                                        slice[i] = false; // Default value, actual null is tracked separately
                                    }
                                    _ => {
                                        return Err(BatchReaderError::type_conversion(
                                            TypeConversionError::type_mismatch(
                                                source_name,
                                                &format!("{:?}", value.ion_type()),
                                                LogicalType::Boolean,
                                                Some("Use explicit conversion or check data types"),
                                            ),
                                        )
                                        .into());
                                    }
                                }
                            }
                            None => {
                                slice[i] = false; // Default value, actual null is tracked separately
                            }
                        }
                    }
                }
                Ok(vector)
            }
            LogicalType::String => {
                let mut vector = Vector::new(LogicalType::String, batch_size);
                if let PhysicalVectorEnum::String(ref mut physical_vec) = vector.physical {
                    let slice = physical_vec.as_mut_slice();
                    for (i, value_opt) in values.iter().enumerate() {
                        match value_opt {
                            Some(value) => {
                                match value {
                                    Value::String(string_val) => {
                                        slice[i] = string_val.text().to_string();
                                    }
                                    Value::Symbol(symbol_val) => {
                                        // Allow symbol to string conversion
                                        slice[i] = symbol_val.text().unwrap_or("").to_string();
                                    }
                                    Value::Null(_) => {
                                        slice[i] = String::new(); // Default value, actual null is tracked separately
                                    }
                                    _ => {
                                        return Err(BatchReaderError::type_conversion(
                                            TypeConversionError::type_mismatch(
                                                source_name,
                                                &format!("{:?}", value.ion_type()),
                                                LogicalType::String,
                                                Some("Use explicit conversion or check data types"),
                                            ),
                                        )
                                        .into());
                                    }
                                }
                            }
                            None => {
                                slice[i] = String::new(); // Default value, actual null is tracked separately
                            }
                        }
                    }
                }
                Ok(vector)
            }
        }
    }
}

impl BatchReader for IonReader {
    fn open(&mut self) -> Result<(), EvalError> {
        // No-op for IonReader
        Ok(())
    }

    fn resolve(&self, field_name: &str) -> Option<ProjectionSource> {
        // For Ion reader, return FieldPath projections
        // Since Ion data is schema-less, we can't validate field existence ahead of time
        // Just return the field name as a FieldPath
        Some(ProjectionSource::FieldPath(field_name.to_string()))
    }

    fn set_projection(&mut self, spec: ProjectionSpec) -> Result<(), EvalError> {
        // Validate that all projection sources are FieldPath (Ion doesn't support ColumnIndex)
        for projection in &spec.projections {
            match &projection.source {
                ProjectionSource::FieldPath(_) => {
                    // Valid for Ion reader
                }
                ProjectionSource::ColumnIndex(idx) => {
                    return Err(
                        BatchReaderError::projection(ProjectionError::unsupported_source(
                            &format!("ColumnIndex({})", idx),
                            "IonReader",
                            &["FieldPath"],
                        ))
                        .into(),
                    );
                }
            }
        }

        // Build and cache schema from projection
        let fields: Vec<Field> = spec
            .projections
            .iter()
            .map(|p| Field {
                name: format!("col_{}", p.target_vector_idx),
                type_info: p.logical_type,
            })
            .collect();
        let schema = SourceTypeDef::new(fields);
        
        // Cache schema for reuse across batches
        self.cached_schema = Some(schema.clone());

        // Pre-allocate reusable batch structure
        self.reusable_batch = Some(VectorizedBatch::new(schema, self.batch_size));

        self.projection = Some(spec);
        Ok(())
    }

    fn next_batch(&mut self) -> Result<Option<VectorizedBatch>, EvalError> {
        let projection = self.projection.as_ref().ok_or_else(|| {
            EvalError::General("set_projection must be called before next_batch".to_string())
        })?;

        // Check if we've reached the end of data
        if self.current_position >= self.elements.len() {
            return Ok(None);
        }

        // Determine batch size (remaining elements or configured batch size)
        let remaining_elements = self.elements.len() - self.current_position;
        let actual_batch_size = std::cmp::min(self.batch_size, remaining_elements);

        // Extract elements slice first to avoid borrow checker issues
        let elements_slice = &self.elements[self.current_position..self.current_position + actual_batch_size];
        
        // Extract values for all projections first (before getting mutable batch reference)
        let mut projection_data: Vec<(usize, LogicalType, String, Vec<Option<Value>>)> = Vec::with_capacity(projection.projections.len());
        for proj in &projection.projections {
            if let ProjectionSource::FieldPath(field_path) = &proj.source {
                let mut values = Vec::with_capacity(actual_batch_size);
                for element in elements_slice.iter() {
                    let value = self.extract_field_value(element, field_path)?;
                    values.push(value);
                }
                projection_data.push((proj.target_vector_idx, proj.logical_type.clone(), field_path.clone(), values));
            }
        }

        // Now get batch and write all data directly to slices
        let batch = self.reusable_batch.as_mut().ok_or_else(|| {
            EvalError::General("Reusable batch should have been initialized in set_projection".to_string())
        })?;

        // Reset batch metadata (don't clear vectors - they maintain capacity and we'll overwrite data)
        batch.set_row_count(0);
        batch.set_selection(None);

        // Write data directly to batch vectors (eliminating Vector allocation)
        for (target_idx, logical_type, field_path, values) in projection_data {
            let vector = batch.column_mut(target_idx)?;
            
            match logical_type {
                LogicalType::Int64 => {
                    if let PhysicalVectorEnum::Int64(v) = &mut vector.physical {
                        let slice = v.as_mut_slice();
                        Self::write_values_to_int64_slice(slice, &values, &field_path, actual_batch_size)?;
                    }
                }
                LogicalType::Float64 => {
                    if let PhysicalVectorEnum::Float64(v) = &mut vector.physical {
                        let slice = v.as_mut_slice();
                        Self::write_values_to_float64_slice(slice, &values, &field_path, actual_batch_size)?;
                    }
                }
                LogicalType::Boolean => {
                    if let PhysicalVectorEnum::Boolean(v) = &mut vector.physical {
                        let slice = v.as_mut_slice();
                        Self::write_values_to_boolean_slice(slice, &values, &field_path, actual_batch_size)?;
                    }
                }
                LogicalType::String => {
                    if let PhysicalVectorEnum::String(v) = &mut vector.physical {
                        let slice = v.as_mut_slice();
                        Self::write_values_to_string_slice(slice, &values, &field_path, actual_batch_size)?;
                    }
                }
            }
        }

        // Set the actual row count
        batch.set_row_count(actual_batch_size);

        // Update position for next batch
        self.current_position += actual_batch_size;

        // Clone the batch to return (the reusable batch stays in the reader for next iteration)
        Ok(Some(batch.clone()))
    }

    fn close(&mut self) -> Result<(), EvalError> {
        // No-op for IonReader
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::{Projection, ProjectionSource};

    #[test]
    fn test_ion_reader_basic_functionality() {
        let ion_data = r#"
            {name: "Alice", age: 30, score: 95.5, active: true}
            {name: "Bob", age: 25, score: 87.2, active: false}
        "#;

        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("name".to_string()),
                0,
                LogicalType::String,
            ),
            Projection::new(
                ProjectionSource::FieldPath("age".to_string()),
                1,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("score".to_string()),
                2,
                LogicalType::Float64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("active".to_string()),
                3,
                LogicalType::Boolean,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        assert!(reader.set_projection(projection_spec).is_ok());

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 4);
    }

    #[test]
    fn test_ion_reader_missing_fields() {
        let ion_data = r#"
            {name: "Alice", age: 30}
            {name: "Bob", score: 87.2}
        "#;

        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("name".to_string()),
                0,
                LogicalType::String,
            ),
            Projection::new(
                ProjectionSource::FieldPath("age".to_string()),
                1,
                LogicalType::Int64,
            ),
            Projection::new(
                ProjectionSource::FieldPath("score".to_string()),
                2,
                LogicalType::Float64,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        assert!(reader.set_projection(projection_spec).is_ok());

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 2);
        assert_eq!(batch.total_column_count(), 3);

        // Verify that missing fields result in null values
        // Alice has age but no score, Bob has score but no age
    }

    #[test]
    fn test_ion_reader_rejects_column_index() {
        let ion_data = r#"{name: "Alice", age: 30}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![Projection::new(
            ProjectionSource::ColumnIndex(0),
            0,
            LogicalType::String,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        let result = reader.set_projection(projection_spec);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("ColumnIndex"));
        assert!(error_msg.contains("not supported"));
    }

    #[test]
    fn test_ion_reader_rejects_deep_nesting() {
        let ion_data = r#"{person: {details: {name: "Alice"}}}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![Projection::new(
            ProjectionSource::FieldPath("person.details.name".to_string()),
            0,
            LogicalType::String,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        assert!(reader.set_projection(projection_spec).is_ok());

        // Should fail when trying to read the batch due to deep nesting
        let result = reader.next_batch();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("person.details.name"));
    }

    #[test]
    fn test_ion_reader_single_level_nesting() {
        let ion_data = r#"{person: {name: "Alice", age: 30}}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![
            Projection::new(
                ProjectionSource::FieldPath("person.name".to_string()),
                0,
                LogicalType::String,
            ),
            Projection::new(
                ProjectionSource::FieldPath("person.age".to_string()),
                1,
                LogicalType::Int64,
            ),
        ];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        assert!(reader.set_projection(projection_spec).is_ok());

        let batch = reader.next_batch().unwrap();
        assert!(batch.is_some());

        let batch = batch.unwrap();
        assert_eq!(batch.row_count(), 1);
        assert_eq!(batch.total_column_count(), 2);
    }

    #[test]
    fn test_ion_reader_type_mismatch_error() {
        let ion_data = r#"{name: "Alice", age: "thirty"}"#;
        let mut reader = IonReader::from_ion_text(ion_data, 10).unwrap();

        let projections = vec![Projection::new(
            ProjectionSource::FieldPath("age".to_string()),
            0,
            LogicalType::Int64,
        )];

        let projection_spec = ProjectionSpec::new(projections).unwrap();
        assert!(reader.set_projection(projection_spec).is_ok());

        let result = reader.next_batch();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Type mismatch"));
        assert!(error_msg.contains("String"));
        assert!(error_msg.contains("Int64"));
    }
}
