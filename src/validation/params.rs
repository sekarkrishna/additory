// Parameter validation module
// Validates function parameters for type, value, and presence

use super::errors::{ValidationError, ValidationResult, ErrorType, ErrorContext, Language, generate_error_code};
use std::collections::HashMap;

/// Parameter validator for type, value, and presence validation
pub struct ParameterValidator;

/// Parameter type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamType {
    DataFrame,
    String,
    List,
    Integer,
    Float,
    Boolean,
    Dictionary,
}

impl ParameterValidator {
    /// Validate parameter type matches expected type
    pub fn validate_type(
        param_name: &str,
        value_type: ParamType,
        expected_type: ParamType,
        function: &str,
    ) -> ValidationResult<()> {
        if value_type != expected_type {
            let context = ErrorContext::new(function.to_string())
                .with_parameter(param_name.to_string())
                .with_info("expected_type".to_string(), format!("{:?}", expected_type))
                .with_info("received_type".to_string(), format!("{:?}", value_type));

            let error = ValidationError::new(
                ErrorType::ParameterError,
                generate_error_code(ErrorType::ParameterError, "T", 1),
                format!(
                    "Parameter '{}' expected type {:?} but received {:?}",
                    param_name, expected_type, value_type
                ),
                context,
            )
            .with_suggestion(
                1,
                format!("Provide a {:?} value for parameter '{}'", expected_type, param_name),
            )
            .with_source("parameter".to_string(), "ParameterValidator".to_string());

            return Err(error);
        }
        Ok(())
    }

    /// Validate parameter value is in allowed set
    pub fn validate_value(
        param_name: &str,
        value: &str,
        allowed_values: &[&str],
        function: &str,
    ) -> ValidationResult<()> {
        if !allowed_values.contains(&value) {
            let context = ErrorContext::new(function.to_string())
                .with_parameter(param_name.to_string())
                .with_info("received_value".to_string(), value.to_string())
                .with_info("valid_values".to_string(), allowed_values.join(", "));

            let error = ValidationError::new(
                ErrorType::ParameterError,
                generate_error_code(ErrorType::ParameterError, "V", 1),
                format!(
                    "Parameter '{}' received invalid value '{}'",
                    param_name, value
                ),
                context,
            )
            .with_suggestion(
                1,
                format!(
                    "Use one of the valid values: {}",
                    allowed_values.join(", ")
                ),
            )
            .with_example(
                Language::Python,
                format!(
                    "{}(..., {}='{}')  # Valid option",
                    function, param_name, allowed_values[0]
                ),
                "Correct usage with valid value".to_string(),
            )
            .with_source("parameter".to_string(), "ParameterValidator".to_string());

            return Err(error);
        }
        Ok(())
    }

    /// Validate required parameters are present
    pub fn validate_required(
        params: &HashMap<String, bool>,  // param_name -> is_present
        required: &[&str],
        function: &str,
    ) -> ValidationResult<()> {
        let missing: Vec<&str> = required
            .iter()
            .filter(|&&param| !params.get(param).copied().unwrap_or(false))
            .copied()
            .collect();

        if !missing.is_empty() {
            let context = ErrorContext::new(function.to_string())
                .with_info("missing_parameters".to_string(), missing.join(", "));

            let error = ValidationError::new(
                ErrorType::ParameterError,
                generate_error_code(ErrorType::ParameterError, "R", 1),
                format!(
                    "Missing required parameter{}: {}",
                    if missing.len() > 1 { "s" } else { "" },
                    missing.join(", ")
                ),
                context,
            )
            .with_suggestion(
                1,
                format!("Provide the required parameter{}: {}", 
                    if missing.len() > 1 { "s" } else { "" },
                    missing.join(", ")
                ),
            )
            .with_source("parameter".to_string(), "ParameterValidator".to_string());

            return Err(error);
        }
        Ok(())
    }

    /// Validate conditionally required parameters
    pub fn validate_conditional(
        params: &HashMap<String, bool>,
        condition: &str,
        condition_met: bool,
        required_if_true: &[&str],
        function: &str,
    ) -> ValidationResult<()> {
        if condition_met {
            let missing: Vec<&str> = required_if_true
                .iter()
                .filter(|&&param| !params.get(param).copied().unwrap_or(false))
                .copied()
                .collect();

            if !missing.is_empty() {
                let context = ErrorContext::new(function.to_string())
                    .with_info("condition".to_string(), condition.to_string())
                    .with_info("missing_parameters".to_string(), missing.join(", "));

                let error = ValidationError::new(
                    ErrorType::ParameterError,
                    generate_error_code(ErrorType::ParameterError, "R", 2),
                    format!(
                        "Conditionally required parameter{} missing: {}",
                        if missing.len() > 1 { "s" } else { "" },
                        missing.join(", ")
                    ),
                    context,
                )
                .with_suggestion(
                    1,
                    format!(
                        "When {}, you must provide: {}",
                        condition,
                        missing.join(", ")
                    ),
                )
                .with_source("parameter".to_string(), "ParameterValidator".to_string());

                return Err(error);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_validation() {
        // Valid type
        assert!(ParameterValidator::validate_type(
            "df",
            ParamType::DataFrame,
            ParamType::DataFrame,
            "add.to"
        )
        .is_ok());

        // Invalid type
        let result = ParameterValidator::validate_type(
            "df",
            ParamType::String,
            ParamType::DataFrame,
            "add.to"
        );
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_type, ErrorType::ParameterError);
        assert!(error.message.contains("DataFrame"));
    }

    #[test]
    fn test_value_validation() {
        // Valid value
        assert!(ParameterValidator::validate_value(
            "join_type",
            "inner",
            &["left", "right", "inner", "outer"],
            "add.to"
        )
        .is_ok());

        // Invalid value
        let result = ParameterValidator::validate_value(
            "join_type",
            "invalid",
            &["left", "right", "inner", "outer"],
            "add.to"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_required_validation() {
        let mut params = HashMap::new();
        params.insert("df".to_string(), true);
        params.insert("on".to_string(), true);

        // All required present
        assert!(ParameterValidator::validate_required(
            &params,
            &["df", "on"],
            "add.to"
        )
        .is_ok());

        // Missing required
        let result = ParameterValidator::validate_required(
            &params,
            &["df", "on", "missing"],
            "add.to"
        );
        assert!(result.is_err());
    }
}
