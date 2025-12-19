use std::fmt;

/// Errors that can occur during vectorized evaluation
#[derive(Debug, Clone)]
pub enum EvalError {
    /// Type mismatch between expected and actual types
    TypeMismatch,
    /// Column not found in schema
    ColumnNotFound(String),
    /// Invalid column index
    InvalidColumnIndex(usize),
    /// General evaluation error
    General(String),
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::TypeMismatch => write!(f, "Type mismatch"),
            EvalError::ColumnNotFound(name) => write!(f, "Column not found: {}", name),
            EvalError::InvalidColumnIndex(idx) => write!(f, "Invalid column index: {}", idx),
            EvalError::General(msg) => write!(f, "Evaluation error: {}", msg),
        }
    }
}

impl std::error::Error for EvalError {}

/// Errors that can occur during planning
#[derive(Debug, Clone)]
pub enum PlanError {
    /// Column not found in schema
    ColumnNotFound(String),
    /// No function found matching the operation and types
    NoFunctionMatch {
        op: String,
        lhs_type: String,
        rhs_type: String,
    },
    /// Unsupported expression type
    UnsupportedExpr,
    /// General planning error
    General(String),
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanError::ColumnNotFound(name) => write!(f, "Column not found: {}", name),
            PlanError::NoFunctionMatch { op, lhs_type, rhs_type } => {
                write!(f, "No function for {} ({}, {})", op, lhs_type, rhs_type)
            }
            PlanError::UnsupportedExpr => write!(f, "Unsupported expression"),
            PlanError::General(msg) => write!(f, "Planning error: {}", msg),
        }
    }
}

impl std::error::Error for PlanError {}
