use crate::batch::LogicalType;
use crate::error::EvalError;
use std::fmt;

/// Enhanced error types for multi-format BatchReader support
/// Provides detailed context for debugging projection and data source issues

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectionError {
    /// Projection specification validation failed
    InvalidSpec { reason: String, context: String },
    /// Projection source is not supported by this reader
    UnsupportedSource {
        source_type: String,
        reader_type: String,
        supported_sources: Vec<String>,
    },
    /// Field or column referenced in projection doesn't exist
    SourceNotFound {
        source: String,
        available_sources: Vec<String>,
    },
    /// Projection would result in non-scalar data (Phase 0 constraint)
    NonScalarData {
        source: String,
        actual_type: String,
        expected_scalar_types: Vec<String>,
    },
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectionError::InvalidSpec { reason, context } => {
                write!(
                    f,
                    "Invalid projection specification: {}. Context: {}",
                    reason, context
                )
            }
            ProjectionError::UnsupportedSource {
                source_type,
                reader_type,
                supported_sources,
            } => {
                write!(
                    f,
                    "Projection source '{}' is not supported by {} reader. Supported sources: [{}]",
                    source_type,
                    reader_type,
                    supported_sources.join(", ")
                )
            }
            ProjectionError::SourceNotFound {
                source,
                available_sources,
            } => {
                write!(
                    f,
                    "Projection source '{}' not found. Available sources: [{}]",
                    source,
                    available_sources.join(", ")
                )
            }
            ProjectionError::NonScalarData {
                source,
                actual_type,
                expected_scalar_types,
            } => {
                write!(
                    f,
                    "Projection source '{}' contains non-scalar data of type '{}'. Phase 0 only supports scalar types: [{}]",
                    source,
                    actual_type,
                    expected_scalar_types.join(", ")
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataSourceError {
    /// File or resource access failed
    AccessFailed { resource: String, reason: String },
    /// Data format is corrupted or invalid
    CorruptedData {
        resource: String,
        location: String,
        details: String,
    },
    /// Required data source configuration is missing
    MissingConfiguration {
        parameter: String,
        required_for: String,
    },
    /// Data source initialization failed
    InitializationFailed { source_type: String, reason: String },
}

impl fmt::Display for DataSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataSourceError::AccessFailed { resource, reason } => {
                write!(f, "Failed to access data source '{}': {}", resource, reason)
            }
            DataSourceError::CorruptedData {
                resource,
                location,
                details,
            } => {
                write!(
                    f,
                    "Corrupted data in '{}' at {}: {}",
                    resource, location, details
                )
            }
            DataSourceError::MissingConfiguration {
                parameter,
                required_for,
            } => {
                write!(
                    f,
                    "Missing required configuration parameter '{}' for {}",
                    parameter, required_for
                )
            }
            DataSourceError::InitializationFailed {
                source_type,
                reason,
            } => {
                write!(
                    f,
                    "Failed to initialize {} data source: {}",
                    source_type, reason
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeConversionError {
    /// Type mismatch between source data and declared projection type
    TypeMismatch {
        source: String,
        source_type: String,
        target_type: LogicalType,
        conversion_hint: Option<String>,
    },
    /// Value cannot be converted to target type (e.g., overflow, invalid format)
    ConversionFailed {
        source: String,
        value: String,
        target_type: LogicalType,
        reason: String,
    },
    /// Null value found where non-null was expected
    UnexpectedNull {
        source: String,
        target_type: LogicalType,
    },
}

impl fmt::Display for TypeConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeConversionError::TypeMismatch {
                source,
                source_type,
                target_type,
                conversion_hint,
            } => {
                let hint = conversion_hint
                    .as_ref()
                    .map(|h| format!(" Hint: {}", h))
                    .unwrap_or_default();
                write!(
                    f,
                    "Type mismatch for source '{}': found '{}', expected '{:?}'.{}",
                    source, source_type, target_type, hint
                )
            }
            TypeConversionError::ConversionFailed {
                source,
                value,
                target_type,
                reason,
            } => {
                write!(
                    f,
                    "Failed to convert value '{}' from source '{}' to {:?}: {}",
                    value, source, target_type, reason
                )
            }
            TypeConversionError::UnexpectedNull {
                source,
                target_type,
            } => {
                write!(
                    f,
                    "Unexpected null value from source '{}' when converting to {:?}",
                    source, target_type
                )
            }
        }
    }
}

/// Error severity classification for better error handling
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    /// Fatal error - operation cannot continue
    Fatal,
    /// Recoverable error - operation can continue with degraded functionality
    Recoverable,
    /// Warning - operation succeeded but with potential issues
    Warning,
}

/// Enhanced error wrapper with severity and context
#[derive(Debug, Clone, PartialEq)]
pub struct BatchReaderError {
    pub severity: ErrorSeverity,
    pub error_type: BatchReaderErrorType,
    pub context: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchReaderErrorType {
    Projection(ProjectionError),
    DataSource(DataSourceError),
    TypeConversion(TypeConversionError),
}

impl BatchReaderError {
    pub fn projection(error: ProjectionError) -> Self {
        Self {
            severity: ErrorSeverity::Fatal,
            error_type: BatchReaderErrorType::Projection(error),
            context: Vec::new(),
        }
    }

    pub fn data_source(error: DataSourceError) -> Self {
        Self {
            severity: ErrorSeverity::Fatal,
            error_type: BatchReaderErrorType::DataSource(error),
            context: Vec::new(),
        }
    }

    pub fn type_conversion(error: TypeConversionError) -> Self {
        Self {
            severity: ErrorSeverity::Recoverable,
            error_type: BatchReaderErrorType::TypeConversion(error),
            context: Vec::new(),
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context.push(context);
        self
    }

    pub fn with_severity(mut self, severity: ErrorSeverity) -> Self {
        self.severity = severity;
        self
    }
}

impl fmt::Display for BatchReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let severity_str = match self.severity {
            ErrorSeverity::Fatal => "FATAL",
            ErrorSeverity::Recoverable => "RECOVERABLE",
            ErrorSeverity::Warning => "WARNING",
        };

        let error_msg = match &self.error_type {
            BatchReaderErrorType::Projection(e) => e.to_string(),
            BatchReaderErrorType::DataSource(e) => e.to_string(),
            BatchReaderErrorType::TypeConversion(e) => e.to_string(),
        };

        write!(f, "[{}] {}", severity_str, error_msg)?;

        if !self.context.is_empty() {
            write!(f, "\nContext:")?;
            for (i, ctx) in self.context.iter().enumerate() {
                write!(f, "\n  {}: {}", i + 1, ctx)?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for BatchReaderError {}

/// Convert BatchReaderError to EvalError for compatibility with existing error handling
impl From<BatchReaderError> for EvalError {
    fn from(error: BatchReaderError) -> Self {
        EvalError::General(error.to_string())
    }
}

/// Convenience functions for creating common errors
impl ProjectionError {
    pub fn invalid_spec(reason: &str, context: &str) -> Self {
        Self::InvalidSpec {
            reason: reason.to_string(),
            context: context.to_string(),
        }
    }

    pub fn unsupported_source(source_type: &str, reader_type: &str, supported: &[&str]) -> Self {
        Self::UnsupportedSource {
            source_type: source_type.to_string(),
            reader_type: reader_type.to_string(),
            supported_sources: supported.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn source_not_found(source: &str, available: &[String]) -> Self {
        Self::SourceNotFound {
            source: source.to_string(),
            available_sources: available.to_vec(),
        }
    }

    pub fn non_scalar_data(source: &str, actual_type: &str) -> Self {
        Self::NonScalarData {
            source: source.to_string(),
            actual_type: actual_type.to_string(),
            expected_scalar_types: vec![
                "Int64".to_string(),
                "Float64".to_string(),
                "Boolean".to_string(),
                "String".to_string(),
            ],
        }
    }
}

impl DataSourceError {
    pub fn access_failed(resource: &str, reason: &str) -> Self {
        Self::AccessFailed {
            resource: resource.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn corrupted_data(resource: &str, location: &str, details: &str) -> Self {
        Self::CorruptedData {
            resource: resource.to_string(),
            location: location.to_string(),
            details: details.to_string(),
        }
    }

    pub fn missing_configuration(parameter: &str, required_for: &str) -> Self {
        Self::MissingConfiguration {
            parameter: parameter.to_string(),
            required_for: required_for.to_string(),
        }
    }

    pub fn initialization_failed(source_type: &str, reason: &str) -> Self {
        Self::InitializationFailed {
            source_type: source_type.to_string(),
            reason: reason.to_string(),
        }
    }
}

impl TypeConversionError {
    pub fn type_mismatch(
        source: &str,
        source_type: &str,
        target_type: LogicalType,
        hint: Option<&str>,
    ) -> Self {
        Self::TypeMismatch {
            source: source.to_string(),
            source_type: source_type.to_string(),
            target_type,
            conversion_hint: hint.map(|h| h.to_string()),
        }
    }

    pub fn conversion_failed(
        source: &str,
        value: &str,
        target_type: LogicalType,
        reason: &str,
    ) -> Self {
        Self::ConversionFailed {
            source: source.to_string(),
            value: value.to_string(),
            target_type,
            reason: reason.to_string(),
        }
    }

    pub fn unexpected_null(source: &str, target_type: LogicalType) -> Self {
        Self::UnexpectedNull {
            source: source.to_string(),
            target_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_error_display() {
        let error = ProjectionError::invalid_spec(
            "Duplicate indices",
            "Projections at indices [0, 0] conflict",
        );
        let display = error.to_string();
        assert!(display.contains("Invalid projection specification"));
        assert!(display.contains("Duplicate indices"));
        assert!(display.contains("Projections at indices [0, 0] conflict"));
    }

    #[test]
    fn test_data_source_error_display() {
        let error = DataSourceError::access_failed("test.parquet", "File not found");
        let display = error.to_string();
        assert!(display.contains("Failed to access data source 'test.parquet'"));
        assert!(display.contains("File not found"));
    }

    #[test]
    fn test_type_conversion_error_display() {
        let error = TypeConversionError::type_mismatch(
            "column_a",
            "String",
            LogicalType::Int64,
            Some("Use CAST function for explicit conversion"),
        );
        let display = error.to_string();
        assert!(display.contains("Type mismatch for source 'column_a'"));
        assert!(display.contains("found 'String', expected 'Int64'"));
        assert!(display.contains("Hint: Use CAST function"));
    }

    #[test]
    fn test_batch_reader_error_with_context() {
        let projection_error = ProjectionError::source_not_found(
            "missing_field",
            &vec!["field_a".to_string(), "field_b".to_string()],
        );
        let error = BatchReaderError::projection(projection_error)
            .with_context("While processing Ion data".to_string())
            .with_context("In batch 5 of query execution".to_string());

        let display = error.to_string();
        assert!(display.contains("[FATAL]"));
        assert!(display.contains("missing_field"));
        assert!(display.contains("Context:"));
        assert!(display.contains("While processing Ion data"));
        assert!(display.contains("In batch 5 of query execution"));
    }

    #[test]
    fn test_error_severity_classification() {
        let fatal_error =
            BatchReaderError::projection(ProjectionError::invalid_spec("test", "test"));
        assert_eq!(fatal_error.severity, ErrorSeverity::Fatal);

        let recoverable_error = BatchReaderError::type_conversion(
            TypeConversionError::unexpected_null("test", LogicalType::Int64),
        );
        assert_eq!(recoverable_error.severity, ErrorSeverity::Recoverable);

        let warning_error = recoverable_error.with_severity(ErrorSeverity::Warning);
        assert_eq!(warning_error.severity, ErrorSeverity::Warning);
    }

    #[test]
    fn test_eval_error_conversion() {
        let batch_error = BatchReaderError::data_source(DataSourceError::corrupted_data(
            "test.parquet",
            "row 100",
            "Invalid checksum",
        ));
        let eval_error: EvalError = batch_error.into();

        match eval_error {
            EvalError::General(msg) => {
                assert!(msg.contains("[FATAL]"));
                assert!(msg.contains("Corrupted data"));
            }
            _ => panic!("Expected General error variant"),
        }
    }
}
