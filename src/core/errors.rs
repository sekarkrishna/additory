//! Error types for additory v0.1.3a5
//!
//! All errors in additory use the AdditoryError enum.
//! Error messages are designed to be:
//! - Clear: Explain what went wrong
//! - Actionable: Suggest how to fix it
//! - Contextual: Show relevant information

use thiserror::Error;

/// Result type for additory operations
pub type AdditoryResult<T> = Result<T, AdditoryError>;

/// Main error type for additory v0.1.3a5
#[derive(Error, Debug)]
pub enum AdditoryError {
    // ========== Validation Errors ==========
    
    /// Cardinality validation error (many:many joins not allowed)
    #[error("Cardinality error: {0}. {1}")]
    Cardinality(String, String),

    /// Position validation error (invalid position specification)
    #[error("Position error: {0}. {1}")]
    Position(String, String),

    /// General validation error
    #[error("Validation error: {0}. {1}")]
    Validation(String, String),

    // ========== Parsing Errors ==========
    
    /// Mode parsing error (invalid mode string)
    #[error("Mode parsing error: {0}. Valid modes: {1}")]
    ModeParsing(String, String),

    /// Parameter parsing error (invalid parameter format)
    #[error("Parameter parsing error for '{0}': {1}. {2}")]
    ParameterParsing(String, String, String),

    /// Strategy parsing error (invalid strategy format)
    #[error("Strategy parsing error: {0}. {1}")]
    StrategyParsing(String, String),

    // ========== Operation Errors ==========
    
    /// Join operation error
    #[error("Join error: {0}. {1}")]
    Join(String, String),

    /// Aggregation operation error
    #[error("Aggregation error: {0}. {1}")]
    Aggregation(String, String),

    /// Transform operation error
    #[error("Transform error: {0}. {1}")]
    Transform(String, String),

    /// Synthetic data generation error
    #[error("Synthetic error: {0}. {1}")]
    Synthetic(String, String),

    /// General operation error
    #[error("Operation error: {0}. {1}")]
    Operation(String, String),

    // ========== DataFrame Errors ==========
    
    /// General DataFrame error
    #[error("DataFrame error: {0}")]
    DataFrame(String),

    /// Column not found in DataFrame
    #[error("Column '{0}' not found in DataFrame. Available columns: {1}")]
    ColumnNotFound(String, String),

    /// Type mismatch error
    #[error("Type mismatch for column '{0}': expected {1}, got {2}. {3}")]
    TypeMismatch(String, String, String, String),

    /// Duplicate column name
    #[error("Duplicate column name '{0}'. {1}")]
    DuplicateColumn(String, String),

    /// Empty DataFrame error
    #[error("Empty DataFrame: {0}")]
    EmptyDataFrame(String),

    // ========== Expression Errors ==========
    
    /// Expression evaluation error
    #[error("Expression error: {0}. {1}")]
    Expression(String, String),

    /// Expression not found in namespace
    #[error("Expression '{0}' not found in namespace '{1}'. Available: {2}")]
    ExpressionNotFound(String, String, String),

    /// Invalid expression reference format
    #[error("Invalid expression reference '{0}'. Expected format: {1}")]
    InvalidExpressionReference(String, String),

    // ========== Parameter Errors ==========
    
    /// Missing required parameter
    #[error("Missing required parameter '{0}'. {1}")]
    MissingParameter(String, String),

    /// Invalid parameter value
    #[error("Invalid parameter '{0}': {1}. {2}")]
    InvalidParameter(String, String, String),

    /// Parameter conflict
    #[error("Parameter conflict: {0}. {1}")]
    ParameterConflict(String, String),

    // ========== Serialization Errors ==========
    
    /// Serialization error (DataFrame → bytes)
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error (bytes → DataFrame)
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    // ========== External Errors ==========
    
    /// Polars error
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Python feature unavailable (for PyO3 bindings)
    #[cfg(feature = "python")]
    #[error("Python feature unavailable: {0}. {1}")]
    PythonFeatureUnavailable(String, String),

    /// Python error (for PyO3 bindings)
    #[cfg(feature = "python")]
    #[error("Python error: {0}")]
    Python(String),

    // ========== Generic Errors ==========
    
    /// Other error (catch-all)
    #[error("Error: {0}")]
    Other(String),
}

impl AdditoryError {
    // ========== Validation Error Constructors ==========
    
    /// Create a Cardinality error
    pub fn cardinality(message: &str, help: &str) -> Self {
        Self::Cardinality(message.to_string(), help.to_string())
    }

    /// Create a Position error
    pub fn position(message: &str, help: &str) -> Self {
        Self::Position(message.to_string(), help.to_string())
    }

    /// Create a Validation error
    pub fn validation(message: &str, help: &str) -> Self {
        Self::Validation(message.to_string(), help.to_string())
    }

    // ========== Parsing Error Constructors ==========
    
    /// Create a ModeParsing error with valid modes
    pub fn mode_parsing(mode: &str, valid_modes: &[&str]) -> Self {
        Self::ModeParsing(
            format!("Invalid mode '{}'", mode),
            valid_modes.join(", "),
        )
    }

    /// Create a ParameterParsing error
    pub fn parameter_parsing(param: &str, value: &str, help: &str) -> Self {
        Self::ParameterParsing(
            param.to_string(),
            value.to_string(),
            help.to_string(),
        )
    }

    /// Create a StrategyParsing error
    pub fn strategy_parsing(message: &str, help: &str) -> Self {
        Self::StrategyParsing(message.to_string(), help.to_string())
    }

    // ========== Operation Error Constructors ==========
    
    /// Create a Join error
    pub fn join(message: &str, help: &str) -> Self {
        Self::Join(message.to_string(), help.to_string())
    }

    /// Create an Aggregation error
    pub fn aggregation(message: &str, help: &str) -> Self {
        Self::Aggregation(message.to_string(), help.to_string())
    }

    /// Create a Transform error
    pub fn transform(message: &str, help: &str) -> Self {
        Self::Transform(message.to_string(), help.to_string())
    }

    /// Create a Synthetic error
    pub fn synthetic(message: &str, help: &str) -> Self {
        Self::Synthetic(message.to_string(), help.to_string())
    }

    /// Create a generic Operation error
    pub fn operation(message: &str, help: &str) -> Self {
        Self::Operation(message.to_string(), help.to_string())
    }

    // ========== DataFrame Error Constructors ==========
    
    /// Create a ColumnNotFound error with available columns
    pub fn column_not_found(column: &str, available: &[String]) -> Self {
        let available_str = if available.is_empty() {
            "none".to_string()
        } else {
            available.join(", ")
        };
        Self::ColumnNotFound(column.to_string(), available_str)
    }

    /// Create a TypeMismatch error
    pub fn type_mismatch(column: &str, expected: &str, got: &str, help: &str) -> Self {
        Self::TypeMismatch(
            column.to_string(),
            expected.to_string(),
            got.to_string(),
            help.to_string(),
        )
    }

    /// Create a DuplicateColumn error
    pub fn duplicate_column(column: &str, help: &str) -> Self {
        Self::DuplicateColumn(column.to_string(), help.to_string())
    }

    /// Create an EmptyDataFrame error
    pub fn empty_dataframe(message: &str) -> Self {
        Self::EmptyDataFrame(message.to_string())
    }

    // ========== Expression Error Constructors ==========
    
    /// Create an Expression error
    pub fn expression(message: &str, help: &str) -> Self {
        Self::Expression(message.to_string(), help.to_string())
    }

    /// Create an ExpressionNotFound error
    pub fn expression_not_found(name: &str, namespace: &str, available: &[String]) -> Self {
        let available_str = if available.is_empty() {
            "none".to_string()
        } else {
            available.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        };
        Self::ExpressionNotFound(
            name.to_string(),
            namespace.to_string(),
            available_str,
        )
    }

    /// Create an InvalidExpressionReference error
    pub fn invalid_expression_reference(reference: &str, expected_format: &str) -> Self {
        Self::InvalidExpressionReference(
            reference.to_string(),
            expected_format.to_string(),
        )
    }

    // ========== Parameter Error Constructors ==========
    
    /// Create a MissingParameter error
    pub fn missing_parameter(param: &str, help: &str) -> Self {
        Self::MissingParameter(param.to_string(), help.to_string())
    }

    /// Create an InvalidParameter error
    pub fn invalid_parameter(param: &str, value: &str, help: &str) -> Self {
        Self::InvalidParameter(
            param.to_string(),
            value.to_string(),
            help.to_string(),
        )
    }

    /// Create a ParameterConflict error
    pub fn parameter_conflict(message: &str, help: &str) -> Self {
        Self::ParameterConflict(message.to_string(), help.to_string())
    }

    // ========== Serialization Error Constructors ==========
    
    /// Create a Serialization error
    pub fn serialization(message: &str) -> Self {
        Self::Serialization(message.to_string())
    }

    /// Create a Deserialization error
    pub fn deserialization(message: &str) -> Self {
        Self::Deserialization(message.to_string())
    }

    // ========== Python Error Constructors ==========
    
    #[cfg(feature = "python")]
    /// Create a PythonFeatureUnavailable error
    pub fn python_feature_unavailable(feature: &str, help: &str) -> Self {
        Self::PythonFeatureUnavailable(feature.to_string(), help.to_string())
    }

    #[cfg(feature = "python")]
    /// Create a Python error
    pub fn python(message: &str) -> Self {
        Self::Python(message.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardinality_error() {
        let err = AdditoryError::cardinality(
            "Many-to-many join detected",
            "Use aggregation or change join type",
        );
        assert!(err.to_string().contains("Many-to-many"));
        assert!(err.to_string().contains("aggregation"));
    }

    #[test]
    fn test_position_error() {
        let err = AdditoryError::position(
            "Invalid position 'after:nonexistent'",
            "Column 'nonexistent' not found",
        );
        assert!(err.to_string().contains("Invalid position"));
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_mode_parsing_error() {
        let err = AdditoryError::mode_parsing("@invalid", &["@calc", "@filter", "@sort"]);
        assert!(err.to_string().contains("@invalid"));
        assert!(err.to_string().contains("@calc"));
    }

    #[test]
    fn test_column_not_found_error() {
        let err = AdditoryError::column_not_found(
            "age",
            &["name".to_string(), "id".to_string()],
        );
        assert!(err.to_string().contains("age"));
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let err = AdditoryError::type_mismatch(
            "age",
            "Int64",
            "String",
            "Convert column to numeric type",
        );
        assert!(err.to_string().contains("age"));
        assert!(err.to_string().contains("Int64"));
        assert!(err.to_string().contains("String"));
    }

    #[test]
    fn test_join_error() {
        let err = AdditoryError::join(
            "Key column 'id' not found in reference DataFrame",
            "Check that the key column exists in both DataFrames",
        );
        assert!(err.to_string().contains("Key column"));
        assert!(err.to_string().contains("Check that"));
    }

    #[test]
    fn test_aggregation_error() {
        let err = AdditoryError::aggregation(
            "Invalid aggregation mode 'invalid'",
            "Use one of: first, last, sum, count, average, min, max, concat",
        );
        assert!(err.to_string().contains("Invalid aggregation"));
        assert!(err.to_string().contains("first, last"));
    }

    #[test]
    fn test_expression_not_found_error() {
        let err = AdditoryError::expression_not_found(
            "bmi",
            "inbuilt",
            &["weight".to_string(), "height".to_string()],
        );
        assert!(err.to_string().contains("bmi"));
        assert!(err.to_string().contains("inbuilt"));
        assert!(err.to_string().contains("weight"));
    }

    #[test]
    fn test_missing_parameter_error() {
        let err = AdditoryError::missing_parameter(
            "fetch",
            "Specify columns to fetch with fetch=['col1', 'col2']",
        );
        assert!(err.to_string().contains("fetch"));
        assert!(err.to_string().contains("Specify columns"));
    }

    #[test]
    fn test_parameter_conflict_error() {
        let err = AdditoryError::parameter_conflict(
            "Cannot use both 'expression' and 'where' parameters",
            "Choose one parameter",
        );
        assert!(err.to_string().contains("Cannot use both"));
        assert!(err.to_string().contains("Choose one"));
    }
}
