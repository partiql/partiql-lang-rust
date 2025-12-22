use crate::batch::LogicalType;
use crate::error::PlanError;

/// Field in a schema
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub type_info: LogicalType,
}

/// Source schema definition
#[derive(Debug, Clone)]
pub struct SourceTypeDef {
    fields: Vec<Field>,
}

impl SourceTypeDef {
    /// Create new schema definition
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    /// Get type for a column by name
    pub fn get_type(&self, name: &str) -> Result<LogicalType, PlanError> {
        self.fields
            .iter()
            .find(|f| f.name == name)
            .map(|f| f.type_info)
            .ok_or_else(|| PlanError::ColumnNotFound(name.to_string()))
    }

    /// Get column index by name
    pub fn get_column_index(&self, name: &str) -> Result<usize, PlanError> {
        self.fields
            .iter()
            .position(|f| f.name == name)
            .ok_or_else(|| PlanError::ColumnNotFound(name.to_string()))
    }

    /// Get field by index
    pub fn get_field(&self, idx: usize) -> Option<&Field> {
        self.fields.get(idx)
    }

    /// Number of fields
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Get all fields
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
}
