// Structured error types for validation system
// Provides rich error context with suggestions and examples

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Result type for validation operations
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Main validation error type with rich context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: ErrorType,
    pub error_code: String,
    pub message: String,
    pub context: ErrorContext,
    pub suggestions: Vec<Suggestion>,
    pub examples: Vec<CodeExample>,
    pub severity: Severity,
    pub source: ErrorSource,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(
        error_type: ErrorType,
        error_code: String,
        message: String,
        context: ErrorContext,
    ) -> Self {
        Self {
            error_type,
            error_code,
            message,
            context,
            suggestions: Vec::new(),
            examples: Vec::new(),
            severity: Severity::Error,
            source: ErrorSource {
                validation_layer: String::new(),
                validator: String::new(),
                chained_from: None,
            },
        }
    }

    /// Add a suggestion to the error
    pub fn with_suggestion(mut self, priority: u8, description: String) -> Self {
        self.suggestions.push(Suggestion {
            priority,
            description,
            rationale: None,
        });
        self
    }

    /// Add a suggestion with rationale
    pub fn with_suggestion_and_rationale(
        mut self,
        priority: u8,
        description: String,
        rationale: String,
    ) -> Self {
        self.suggestions.push(Suggestion {
            priority,
            description,
            rationale: Some(rationale),
        });
        self
    }

    /// Add a code example
    pub fn with_example(mut self, language: Language, code: String, description: String) -> Self {
        self.examples.push(CodeExample {
            language,
            code,
            description,
        });
        self
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the source information
    pub fn with_source(mut self, validation_layer: String, validator: String) -> Self {
        self.source.validation_layer = validation_layer;
        self.source.validator = validator;
        self
    }

    /// Chain another error
    pub fn chain(mut self, error: ValidationError) -> Self {
        self.source.chained_from = Some(Box::new(error));
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}]: {}",
            self.error_type, self.error_code, self.message
        )
    }
}

impl std::error::Error for ValidationError {}

/// Type of validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    ParameterError,
    DataError,
    StrategyError,
    LoggingError,
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::ParameterError => write!(f, "ParameterError"),
            ErrorType::DataError => write!(f, "DataError"),
            ErrorType::StrategyError => write!(f, "StrategyError"),
            ErrorType::LoggingError => write!(f, "LoggingError"),
        }
    }
}

/// Context information for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub function_name: String,
    pub parameter_name: Option<String>,
    pub column_name: Option<String>,
    pub additional_info: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(function_name: String) -> Self {
        Self {
            function_name,
            parameter_name: None,
            column_name: None,
            additional_info: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, parameter_name: String) -> Self {
        self.parameter_name = Some(parameter_name);
        self
    }

    pub fn with_column(mut self, column_name: String) -> Self {
        self.column_name = Some(column_name);
        self
    }

    pub fn with_info(mut self, key: String, value: String) -> Self {
        self.additional_info.insert(key, value);
        self
    }
}

/// Suggestion for fixing an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub priority: u8,  // 1 = highest priority
    pub description: String,
    pub rationale: Option<String>,
}

/// Code example showing correct usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    pub language: Language,
    pub code: String,
    pub description: String,
}

/// Programming language for code examples
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Python,
    R,
    Julia,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Python => write!(f, "Python"),
            Language::R => write!(f, "R"),
            Language::Julia => write!(f, "Julia"),
        }
    }
}

/// Severity of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,      // Blocks execution
    Warning,    // Allows execution but warns
    Info,       // Informational only
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "Error"),
            Severity::Warning => write!(f, "Warning"),
            Severity::Info => write!(f, "Info"),
        }
    }
}

/// Source information for an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorSource {
    pub validation_layer: String,
    pub validator: String,
    pub chained_from: Option<Box<ValidationError>>,
}

/// Generate error code following {LAYER}{CATEGORY}{NUMBER} format
pub fn generate_error_code(error_type: ErrorType, category: &str, number: u32) -> String {
    let layer = match error_type {
        ErrorType::ParameterError => "P",
        ErrorType::DataError => "D",
        ErrorType::StrategyError => "S",
        ErrorType::LoggingError => "L",
    };
    format!("{}{}{:03}", layer, category, number)
}

/// Details for error message generation
#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub received_value: Option<String>,
    pub expected_value: Option<String>,
    pub valid_options: Vec<String>,
    pub column_type: Option<String>,
    pub required_type: Option<String>,
    pub statistics: HashMap<String, String>,
}

impl ErrorDetails {
    pub fn new() -> Self {
        Self {
            received_value: None,
            expected_value: None,
            valid_options: Vec::new(),
            column_type: None,
            required_type: None,
            statistics: HashMap::new(),
        }
    }

    pub fn with_received(mut self, value: String) -> Self {
        self.received_value = Some(value);
        self
    }

    pub fn with_expected(mut self, value: String) -> Self {
        self.expected_value = Some(value);
        self
    }

    pub fn with_valid_options(mut self, options: Vec<String>) -> Self {
        self.valid_options = options;
        self
    }

    pub fn with_column_type(mut self, col_type: String) -> Self {
        self.column_type = Some(col_type);
        self
    }

    pub fn with_required_type(mut self, req_type: String) -> Self {
        self.required_type = Some(req_type);
        self
    }

    pub fn with_statistic(mut self, key: String, value: String) -> Self {
        self.statistics.insert(key, value);
        self
    }
}

impl Default for ErrorDetails {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a complete error message following what→why→how→example format
pub fn generate_error_message(
    error_type: ErrorType,
    context: ErrorContext,
    details: ErrorDetails,
) -> ValidationError {
    // Step 1: What went wrong
    let message = format_error_message(&error_type, &context, &details);

    // Step 2: Why it's a problem (stored in context for now)
    let _rationale = explain_problem(&error_type, &details);

    // Step 3: How to fix it (suggestions)
    let suggestions = generate_suggestions(&error_type, &context, &details);

    // Step 4: Example of correct usage
    let examples = generate_code_examples(&error_type, &context, &details);

    // Generate error code
    let error_code = generate_error_code(error_type, "G", 1);

    ValidationError {
        error_type,
        error_code,
        message,
        context,
        suggestions,
        examples,
        severity: Severity::Error,
        source: ErrorSource {
            validation_layer: "validation".to_string(),
            validator: format!("{:?}Validator", error_type),
            chained_from: None,
        },
    }
}

/// Format a clear error statement (the "what")
pub fn format_error_message(
    error_type: &ErrorType,
    context: &ErrorContext,
    details: &ErrorDetails,
) -> String {
    match error_type {
        ErrorType::ParameterError => {
            if let Some(param_name) = &context.parameter_name {
                if let Some(received) = &details.received_value {
                    if let Some(expected) = &details.expected_value {
                        format!(
                            "Parameter '{}' in function '{}' expected {} but received {}",
                            param_name, context.function_name, expected, received
                        )
                    } else if !details.valid_options.is_empty() {
                        format!(
                            "Parameter '{}' in function '{}' received invalid value '{}'",
                            param_name, context.function_name, received
                        )
                    } else {
                        format!(
                            "Parameter '{}' in function '{}' has invalid value",
                            param_name, context.function_name
                        )
                    }
                } else {
                    format!(
                        "Parameter '{}' in function '{}' is invalid",
                        param_name, context.function_name
                    )
                }
            } else {
                format!("Invalid parameter in function '{}'", context.function_name)
            }
        }
        ErrorType::DataError => {
            if let Some(column_name) = &context.column_name {
                if let Some(col_type) = &details.column_type {
                    if let Some(req_type) = &details.required_type {
                        format!(
                            "Column '{}' has type {} but requires {}",
                            column_name, col_type, req_type
                        )
                    } else {
                        format!("Column '{}' has invalid type {}", column_name, col_type)
                    }
                } else {
                    format!("Data validation failed for column '{}'", column_name)
                }
            } else {
                "Data validation failed".to_string()
            }
        }
        ErrorType::StrategyError => {
            if let Some(column_name) = &context.column_name {
                if let Some(received) = &details.received_value {
                    format!(
                        "Strategy for column '{}' has invalid value '{}'",
                        column_name, received
                    )
                } else {
                    format!("Invalid strategy for column '{}'", column_name)
                }
            } else {
                "Strategy validation failed".to_string()
            }
        }
        ErrorType::LoggingError => "Logging configuration error".to_string(),
    }
}

/// Explain why the error is a problem (the "why")
pub fn explain_problem(error_type: &ErrorType, details: &ErrorDetails) -> String {
    match error_type {
        ErrorType::ParameterError => {
            if details.expected_value.is_some() {
                "The function requires specific parameter types to operate correctly".to_string()
            } else if !details.valid_options.is_empty() {
                "The parameter must be one of the allowed values for the operation to succeed"
                    .to_string()
            } else {
                "The parameter value does not meet the function's requirements".to_string()
            }
        }
        ErrorType::DataError => {
            if details.required_type.is_some() {
                "The operation requires specific data types to perform calculations correctly"
                    .to_string()
            } else {
                "The data does not meet the requirements for this operation".to_string()
            }
        }
        ErrorType::StrategyError => {
            "The aggregation strategy must specify valid functions for the column types".to_string()
        }
        ErrorType::LoggingError => "The logging configuration must be valid".to_string(),
    }
}

/// Generate actionable suggestions (the "how")
pub fn generate_suggestions(
    error_type: &ErrorType,
    context: &ErrorContext,
    details: &ErrorDetails,
) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();

    match error_type {
        ErrorType::ParameterError => {
            // Suggest valid options if available
            if !details.valid_options.is_empty() {
                let options_str = details.valid_options.join(", ");
                suggestions.push(Suggestion {
                    priority: 1,
                    description: format!("Use one of the valid options: {}", options_str),
                    rationale: Some("These are the only accepted values for this parameter".to_string()),
                });
            }

            // Suggest type correction if expected type is known
            if let Some(expected) = &details.expected_value {
                suggestions.push(Suggestion {
                    priority: 1,
                    description: format!("Provide a {} value for this parameter", expected),
                    rationale: Some(format!("The function expects {} type", expected)),
                });
            }

            // Check for common typos
            if let (Some(received), false) = (&details.received_value, details.valid_options.is_empty()) {
                if let Some(param_name) = &context.parameter_name {
                    suggestions.push(Suggestion {
                        priority: 2,
                        description: format!("Check the spelling of '{}' for parameter '{}'", received, param_name),
                        rationale: Some("Common typos can cause parameter validation errors".to_string()),
                    });
                }
            }
        }
        ErrorType::DataError => {
            // Suggest type conversion if required type is known
            if let (Some(col_type), Some(req_type)) = (&details.column_type, &details.required_type) {
                if let Some(column_name) = &context.column_name {
                    suggestions.push(Suggestion {
                        priority: 1,
                        description: format!(
                            "Convert column '{}' from {} to {} before this operation",
                            column_name, col_type, req_type
                        ),
                        rationale: Some("Type conversion ensures compatibility with the operation".to_string()),
                    });
                }
            }

            // Suggest filtering or cleaning data
            suggestions.push(Suggestion {
                priority: 2,
                description: "Check your data for inconsistencies or unexpected values".to_string(),
                rationale: Some("Data quality issues can cause validation failures".to_string()),
            });
        }
        ErrorType::StrategyError => {
            // Suggest valid aggregation functions
            if !details.valid_options.is_empty() {
                let options_str = details.valid_options.join(", ");
                suggestions.push(Suggestion {
                    priority: 1,
                    description: format!("Use a valid aggregation function: {}", options_str),
                    rationale: Some("These functions are compatible with the column type".to_string()),
                });
            }

            // Suggest checking column types
            if let Some(column_name) = &context.column_name {
                suggestions.push(Suggestion {
                    priority: 2,
                    description: format!("Check the data type of column '{}'", column_name),
                    rationale: Some("Some aggregation functions only work with specific data types".to_string()),
                });
            }
        }
        ErrorType::LoggingError => {
            suggestions.push(Suggestion {
                priority: 1,
                description: "Use False, 'default', or True for the logging parameter".to_string(),
                rationale: Some("These are the three supported logging levels".to_string()),
            });
        }
    }

    prioritize_suggestions(suggestions)
}

/// Generate code examples in multiple languages (the "example")
pub fn generate_code_examples(
    error_type: &ErrorType,
    context: &ErrorContext,
    details: &ErrorDetails,
) -> Vec<CodeExample> {
    let mut examples = Vec::new();

    match error_type {
        ErrorType::ParameterError => {
            if let Some(param_name) = &context.parameter_name {
                // Python example
                let python_code = if !details.valid_options.is_empty() {
                    let valid_value = &details.valid_options[0];
                    format!(
                        "result = {}(df1, df2, {}='{}')",
                        context.function_name, param_name, valid_value
                    )
                } else if let Some(expected) = &details.expected_value {
                    format!(
                        "# Provide a {} value\nresult = {}(df1, df2, {}=<{}_value>)",
                        expected, context.function_name, param_name, expected
                    )
                } else {
                    format!(
                        "result = {}(df1, df2, {}=<correct_value>)",
                        context.function_name, param_name
                    )
                };

                examples.push(CodeExample {
                    language: Language::Python,
                    code: python_code.clone(),
                    description: "Correct parameter usage in Python".to_string(),
                });

                // R example
                let r_code = python_code.replace("result = ", "result <- ");
                examples.push(CodeExample {
                    language: Language::R,
                    code: r_code,
                    description: "Correct parameter usage in R".to_string(),
                });

                // Julia example
                let julia_code = python_code.clone();
                examples.push(CodeExample {
                    language: Language::Julia,
                    code: julia_code,
                    description: "Correct parameter usage in Julia".to_string(),
                });
            }
        }
        ErrorType::DataError => {
            if let Some(column_name) = &context.column_name {
                // Python example
                let python_code = if let Some(req_type) = &details.required_type {
                    format!(
                        "# Convert column to required type\ndf['{}'] = df['{}'].astype('{}')\nresult = {}(df)",
                        column_name, column_name, req_type.to_lowercase(), context.function_name
                    )
                } else {
                    format!(
                        "# Ensure column '{}' has correct data\nresult = {}(df)",
                        column_name, context.function_name
                    )
                };

                examples.push(CodeExample {
                    language: Language::Python,
                    code: python_code,
                    description: "Data preparation in Python".to_string(),
                });

                // R example
                let r_code = if let Some(req_type) = &details.required_type {
                    format!(
                        "# Convert column to required type\ndf${} <- as.{}(df${})\nresult <- {}(df)",
                        column_name,
                        req_type.to_lowercase(),
                        column_name,
                        context.function_name
                    )
                } else {
                    format!(
                        "# Ensure column '{}' has correct data\nresult <- {}(df)",
                        column_name, context.function_name
                    )
                };

                examples.push(CodeExample {
                    language: Language::R,
                    code: r_code,
                    description: "Data preparation in R".to_string(),
                });

                // Julia example
                let julia_code = if details.required_type.is_some() {
                    format!(
                        "# Convert column to required type\ndf.{} = convert.(Float64, df.{})\nresult = {}(df)",
                        column_name, column_name, context.function_name
                    )
                } else {
                    format!(
                        "# Ensure column '{}' has correct data\nresult = {}(df)",
                        column_name, context.function_name
                    )
                };

                examples.push(CodeExample {
                    language: Language::Julia,
                    code: julia_code,
                    description: "Data preparation in Julia".to_string(),
                });
            }
        }
        ErrorType::StrategyError => {
            if let Some(column_name) = &context.column_name {
                let valid_func = if !details.valid_options.is_empty() {
                    &details.valid_options[0]
                } else {
                    "first"
                };

                // Python example
                let python_code = format!(
                    "result = {}(df1, df2, on='id', strategy={{'{}': '{}'}})",
                    context.function_name, column_name, valid_func
                );

                examples.push(CodeExample {
                    language: Language::Python,
                    code: python_code.clone(),
                    description: "Correct strategy specification in Python".to_string(),
                });

                // R example
                let r_code = format!(
                    "result <- {}(df1, df2, on='id', strategy=list('{}' = '{}'))",
                    context.function_name, column_name, valid_func
                );

                examples.push(CodeExample {
                    language: Language::R,
                    code: r_code,
                    description: "Correct strategy specification in R".to_string(),
                });

                // Julia example
                let julia_code = format!(
                    "result = {}(df1, df2, on=\"id\", strategy=Dict(\"{}\" => \"{}\"))",
                    context.function_name, column_name, valid_func
                );

                examples.push(CodeExample {
                    language: Language::Julia,
                    code: julia_code,
                    description: "Correct strategy specification in Julia".to_string(),
                });
            }
        }
        ErrorType::LoggingError => {
            // Python example
            examples.push(CodeExample {
                language: Language::Python,
                code: format!("result = {}(df1, df2, logging='default')", context.function_name),
                description: "Correct logging parameter in Python".to_string(),
            });

            // R example
            examples.push(CodeExample {
                language: Language::R,
                code: format!("result <- {}(df1, df2, logging='default')", context.function_name),
                description: "Correct logging parameter in R".to_string(),
            });

            // Julia example
            examples.push(CodeExample {
                language: Language::Julia,
                code: format!("result = {}(df1, df2, logging=\"default\")", context.function_name),
                description: "Correct logging parameter in Julia".to_string(),
            });
        }
    }

    examples
}

/// Prioritize suggestions based on likelihood and simplicity
pub fn prioritize_suggestions(mut suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
    // Sort by priority (lower number = higher priority)
    suggestions.sort_by_key(|s| s.priority);
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_generation() {
        assert_eq!(
            generate_error_code(ErrorType::ParameterError, "T", 1),
            "PT001"
        );
        assert_eq!(
            generate_error_code(ErrorType::DataError, "C", 5),
            "DC005"
        );
        assert_eq!(
            generate_error_code(ErrorType::StrategyError, "K", 12),
            "SK012"
        );
    }

    #[test]
    fn test_error_creation() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("join_type".to_string());

        let error = ValidationError::new(
            ErrorType::ParameterError,
            "PV001".to_string(),
            "Invalid join_type".to_string(),
            context,
        )
        .with_suggestion(1, "Use 'inner' instead".to_string())
        .with_example(
            Language::Python,
            "add.to(df1, df2, on='id', join_type='inner')".to_string(),
            "Correct usage".to_string(),
        );

        assert_eq!(error.error_type, ErrorType::ParameterError);
        assert_eq!(error.suggestions.len(), 1);
        assert_eq!(error.examples.len(), 1);
    }

    #[test]
    fn test_error_serialization() {
        let context = ErrorContext::new("add.to".to_string());
        let error = ValidationError::new(
            ErrorType::ParameterError,
            "PT001".to_string(),
            "Test error".to_string(),
            context,
        );

        // Test JSON serialization
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: ValidationError = serde_json::from_str(&json).unwrap();

        assert_eq!(error.error_code, deserialized.error_code);
        assert_eq!(error.message, deserialized.message);
    }

    #[test]
    fn test_format_error_message_parameter_type() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("df".to_string());
        let details = ErrorDetails::new()
            .with_received("String".to_string())
            .with_expected("DataFrame".to_string());

        let message = format_error_message(&ErrorType::ParameterError, &context, &details);
        
        assert!(message.contains("add.to"));
        assert!(message.contains("df"));
        assert!(message.contains("DataFrame"));
        assert!(message.contains("String"));
    }

    #[test]
    fn test_format_error_message_parameter_value() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("join_type".to_string());
        let details = ErrorDetails::new()
            .with_received("innner".to_string())
            .with_valid_options(vec!["left".to_string(), "right".to_string(), "inner".to_string()]);

        let message = format_error_message(&ErrorType::ParameterError, &context, &details);
        
        assert!(message.contains("join_type"));
        assert!(message.contains("innner"));
        assert!(message.contains("invalid"));
    }

    #[test]
    fn test_format_error_message_data_type() {
        let context = ErrorContext::new("add.transform".to_string())
            .with_column("age".to_string());
        let details = ErrorDetails::new()
            .with_column_type("String".to_string())
            .with_required_type("Numeric".to_string());

        let message = format_error_message(&ErrorType::DataError, &context, &details);
        
        assert!(message.contains("age"));
        assert!(message.contains("String"));
        assert!(message.contains("Numeric"));
    }

    #[test]
    fn test_format_error_message_strategy() {
        let context = ErrorContext::new("add.to".to_string())
            .with_column("value".to_string());
        let details = ErrorDetails::new()
            .with_received("average".to_string());

        let message = format_error_message(&ErrorType::StrategyError, &context, &details);
        
        assert!(message.contains("value"));
        assert!(message.contains("average"));
    }

    #[test]
    fn test_explain_problem() {
        let details = ErrorDetails::new()
            .with_expected("DataFrame".to_string());

        let explanation = explain_problem(&ErrorType::ParameterError, &details);
        
        assert!(!explanation.is_empty());
        assert!(explanation.contains("type") || explanation.contains("parameter"));
    }

    #[test]
    fn test_generate_suggestions_with_valid_options() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("join_type".to_string());
        let details = ErrorDetails::new()
            .with_received("innner".to_string())
            .with_valid_options(vec![
                "left".to_string(),
                "right".to_string(),
                "inner".to_string(),
            ]);

        let suggestions = generate_suggestions(&ErrorType::ParameterError, &context, &details);
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].description.contains("left"));
        assert!(suggestions[0].description.contains("inner"));
    }

    #[test]
    fn test_generate_suggestions_with_type_mismatch() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("df".to_string());
        let details = ErrorDetails::new()
            .with_expected("DataFrame".to_string());

        let suggestions = generate_suggestions(&ErrorType::ParameterError, &context, &details);
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].description.contains("DataFrame"));
    }

    #[test]
    fn test_generate_suggestions_data_error() {
        let context = ErrorContext::new("add.transform".to_string())
            .with_column("age".to_string());
        let details = ErrorDetails::new()
            .with_column_type("String".to_string())
            .with_required_type("Numeric".to_string());

        let suggestions = generate_suggestions(&ErrorType::DataError, &context, &details);
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].description.contains("age"));
        assert!(suggestions[0].description.contains("Convert") || suggestions[0].description.contains("convert"));
    }

    #[test]
    fn test_generate_suggestions_strategy_error() {
        let context = ErrorContext::new("add.to".to_string())
            .with_column("value".to_string());
        let details = ErrorDetails::new()
            .with_valid_options(vec!["sum".to_string(), "mean".to_string(), "first".to_string()]);

        let suggestions = generate_suggestions(&ErrorType::StrategyError, &context, &details);
        
        assert!(!suggestions.is_empty());
        assert!(suggestions[0].description.contains("sum") || suggestions[0].description.contains("aggregation"));
    }

    #[test]
    fn test_prioritize_suggestions() {
        let suggestions = vec![
            Suggestion {
                priority: 3,
                description: "Third priority".to_string(),
                rationale: None,
            },
            Suggestion {
                priority: 1,
                description: "First priority".to_string(),
                rationale: None,
            },
            Suggestion {
                priority: 2,
                description: "Second priority".to_string(),
                rationale: None,
            },
        ];

        let prioritized = prioritize_suggestions(suggestions);
        
        assert_eq!(prioritized[0].priority, 1);
        assert_eq!(prioritized[1].priority, 2);
        assert_eq!(prioritized[2].priority, 3);
    }

    #[test]
    fn test_generate_code_examples_parameter_error() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("join_type".to_string());
        let details = ErrorDetails::new()
            .with_valid_options(vec!["inner".to_string()]);

        let examples = generate_code_examples(&ErrorType::ParameterError, &context, &details);
        
        assert_eq!(examples.len(), 3); // Python, R, Julia
        assert!(examples.iter().any(|e| e.language == Language::Python));
        assert!(examples.iter().any(|e| e.language == Language::R));
        assert!(examples.iter().any(|e| e.language == Language::Julia));
        
        // Check Python example
        let python_example = examples.iter().find(|e| e.language == Language::Python).unwrap();
        assert!(python_example.code.contains("join_type"));
        assert!(python_example.code.contains("inner"));
    }

    #[test]
    fn test_generate_code_examples_data_error() {
        let context = ErrorContext::new("add.transform".to_string())
            .with_column("age".to_string());
        let details = ErrorDetails::new()
            .with_required_type("Numeric".to_string());

        let examples = generate_code_examples(&ErrorType::DataError, &context, &details);
        
        assert_eq!(examples.len(), 3);
        
        // Check that examples contain column name
        for example in &examples {
            assert!(example.code.contains("age"));
        }
    }

    #[test]
    fn test_generate_code_examples_strategy_error() {
        let context = ErrorContext::new("add.to".to_string())
            .with_column("value".to_string());
        let details = ErrorDetails::new()
            .with_valid_options(vec!["sum".to_string()]);

        let examples = generate_code_examples(&ErrorType::StrategyError, &context, &details);
        
        assert_eq!(examples.len(), 3);
        
        // Check Python example
        let python_example = examples.iter().find(|e| e.language == Language::Python).unwrap();
        assert!(python_example.code.contains("strategy"));
        assert!(python_example.code.contains("value"));
        assert!(python_example.code.contains("sum"));
    }

    #[test]
    fn test_generate_error_message_complete() {
        let context = ErrorContext::new("add.to".to_string())
            .with_parameter("join_type".to_string());
        let details = ErrorDetails::new()
            .with_received("innner".to_string())
            .with_valid_options(vec![
                "left".to_string(),
                "right".to_string(),
                "inner".to_string(),
            ]);

        let error = generate_error_message(ErrorType::ParameterError, context, details);
        
        // Check all components are present
        assert!(!error.message.is_empty());
        assert!(!error.suggestions.is_empty());
        assert!(!error.examples.is_empty());
        assert_eq!(error.error_type, ErrorType::ParameterError);
        assert_eq!(error.severity, Severity::Error);
        
        // Check suggestions are prioritized
        for i in 1..error.suggestions.len() {
            assert!(error.suggestions[i-1].priority <= error.suggestions[i].priority);
        }
        
        // Check examples for all languages
        assert!(error.examples.iter().any(|e| e.language == Language::Python));
        assert!(error.examples.iter().any(|e| e.language == Language::R));
        assert!(error.examples.iter().any(|e| e.language == Language::Julia));
    }

    #[test]
    fn test_error_details_builder() {
        let details = ErrorDetails::new()
            .with_received("test".to_string())
            .with_expected("expected".to_string())
            .with_valid_options(vec!["opt1".to_string(), "opt2".to_string()])
            .with_column_type("String".to_string())
            .with_required_type("Numeric".to_string())
            .with_statistic("count".to_string(), "100".to_string());

        assert_eq!(details.received_value, Some("test".to_string()));
        assert_eq!(details.expected_value, Some("expected".to_string()));
        assert_eq!(details.valid_options.len(), 2);
        assert_eq!(details.column_type, Some("String".to_string()));
        assert_eq!(details.required_type, Some("Numeric".to_string()));
        assert_eq!(details.statistics.get("count"), Some(&"100".to_string()));
    }
}
