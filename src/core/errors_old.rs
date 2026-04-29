//! Error types for additory
//!
//! All errors in additory use the AdditoryError enum.
//! Error messages are designed to be:
//! - Clear: Explain what went wrong
//! - Actionable: Suggest how to fix it
//! - Contextual: Show relevant information

use thiserror::Error;

/// Result type for additory operations
pub type AdditoryResult<T> = Result<T, AdditoryError>;

/// Main error type for additory
#[derive(Error, Debug)]
pub enum AdditoryError {
    /// Invalid mode string
    #[error("Invalid mode '{0}'. Valid modes: {1}")]
    InvalidMode(String, String),

    /// Missing required parameter
    #[error("Missing required parameter '{0}'. {1}")]
    MissingParameter(String, String),

    /// Invalid parameter value
    #[error("Invalid parameter '{0}': {1}. {2}")]
    InvalidParameter(String, String, String),

    /// Column not found
    #[error("Column '{0}' not found in DataFrame. Available columns: {1}")]
    ColumnNotFound(String, String),

    /// Invalid column type
    #[error("Column '{0}' has type '{1}', expected '{2}'. {3}")]
    InvalidColumnType(String, String, String, String),

    /// Duplicate column name
    #[error("Duplicate column name '{0}'. {1}")]
    DuplicateColumn(String, String),

    /// Operation failed
    #[error("Operation failed: {0}. {1}")]
    OperationFailed(String, String),

    /// Type conversion error
    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    /// Expression error
    #[error("Expression error: {0}")]
    Expression(String),

    /// Expression not found in namespace
    #[error("Expression '{0}' not found in namespace '{1}'. Available expressions: {2}. {3}")]
    ExpressionNotFound(String, String, String, String),

    /// Invalid expression reference format
    #[error("Invalid expression reference '{0}'. {1}")]
    InvalidExpressionReference(String, String),

    /// Python feature unavailable
    #[error("Python feature unavailable: {0}. {1}")]
    PythonFeatureUnavailable(String, String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Polars error
    #[error("Polars error: {0}")]
    Polars(String),

    /// Python error (for PyO3 bindings)
    #[cfg(feature = "python")]
    #[error("Python error: {0}")]
    Python(String),
}

impl AdditoryError {
    /// Create an InvalidMode error with suggestions
    pub fn invalid_mode(mode: &str, valid_modes: &[&str]) -> Self {
        Self::InvalidMode(
            mode.to_string(),
            valid_modes.join(", "),
        )
    }

    /// Create a ColumnNotFound error with available columns
    pub fn column_not_found(column: &str, available: &[String]) -> Self {
        Self::ColumnNotFound(
            column.to_string(),
            available.join(", "),
        )
    }

    /// Create a MissingParameter error with help text
    pub fn missing_parameter(param: &str, help: &str) -> Self {
        Self::MissingParameter(param.to_string(), help.to_string())
    }

    /// Create an InvalidParameter error with help text
    pub fn invalid_parameter(param: &str, value: &str, help: &str) -> Self {
        Self::InvalidParameter(
            param.to_string(),
            value.to_string(),
            help.to_string(),
        )
    }

    /// Create an ExpressionNotFound error with available expressions
    pub fn expression_not_found(
        name: &str,
        namespace: &str,
        available: &[String],
        suggestion: &str,
    ) -> Self {
        let available_str = if available.is_empty() {
            "none".to_string()
        } else {
            available.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        };
        Self::ExpressionNotFound(
            name.to_string(),
            namespace.to_string(),
            available_str,
            suggestion.to_string(),
        )
    }

    /// Create an InvalidExpressionReference error with help text
    pub fn invalid_expression_reference(reference: &str, help: &str) -> Self {
        Self::InvalidExpressionReference(reference.to_string(), help.to_string())
    }

    /// Create a PythonFeatureUnavailable error with help text
    pub fn python_feature_unavailable(feature: &str, help: &str) -> Self {
        Self::PythonFeatureUnavailable(feature.to_string(), help.to_string())
    }
}

// Convert Polars errors to AdditoryError
impl From<polars::error::PolarsError> for AdditoryError {
    fn from(err: polars::error::PolarsError) -> Self {
        Self::Polars(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_mode_error() {
        let err = AdditoryError::invalid_mode("@invalid", &["@filter", "@calc"]);
        assert!(err.to_string().contains("@invalid"));
        assert!(err.to_string().contains("@filter"));
    }

    #[test]
    fn test_column_not_found_error() {
        let err = AdditoryError::column_not_found("age", &["name".to_string(), "id".to_string()]);
        assert!(err.to_string().contains("age"));
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_expression_not_found_error() {
        let err = AdditoryError::expression_not_found(
            "bmi",
            "inbuilt",
            &["weight".to_string(), "height".to_string()],
            "Check the expression name and namespace",
        );
        assert!(err.to_string().contains("bmi"));
        assert!(err.to_string().contains("inbuilt"));
        assert!(err.to_string().contains("weight"));
        assert!(err.to_string().contains("Check the expression name"));
    }

    #[test]
    fn test_invalid_expression_reference_error() {
        let err = AdditoryError::invalid_expression_reference(
            "invalid_ref",
            "Expected format: 'namespace:name'",
        );
        assert!(err.to_string().contains("invalid_ref"));
        assert!(err.to_string().contains("namespace:name"));
    }

    #[test]
    fn test_python_feature_unavailable_error() {
        let err = AdditoryError::python_feature_unavailable(
            "expression resolution",
            "Python bindings not available",
        );
        assert!(err.to_string().contains("expression resolution"));
        assert!(err.to_string().contains("Python bindings"));
    }
}
