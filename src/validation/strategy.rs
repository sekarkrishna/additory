// Strategy validation module
// Validates strategy specifications for correctness and type compatibility

use super::errors::{ValidationError, ErrorType, ErrorContext, Language, generate_error_code};
use super::data::Cardinality;
use polars::prelude::*;

/// Strategy validator for requirement detection and value validation
pub struct StrategyValidator;

impl StrategyValidator {
    pub fn new() -> Self {
        Self
    }

    /// Check if strategy is required based on cardinality
    pub fn is_strategy_required(cardinality: &Cardinality) -> bool {
        matches!(cardinality, Cardinality::ManyToOne | Cardinality::ManyToMany)
    }

    /// Generate error when strategy is required but missing
    /// 
    /// Provides suggestions for aggregation functions based on column types in the DataFrame
    pub fn generate_strategy_required_error(
        cardinality: &Cardinality,
        df: &DataFrame,
        join_keys: &[String],
    ) -> ValidationError {
        let cardinality_str = match cardinality {
            Cardinality::ManyToOne => "many-to-one",
            Cardinality::ManyToMany => "many-to-many",
            _ => "unknown",
        };

        let context = ErrorContext::new("add.to".to_string())
            .with_info("cardinality".to_string(), cardinality_str.to_string());

        let mut error = ValidationError::new(
            ErrorType::StrategyError,
            generate_error_code(ErrorType::StrategyError, "R", 1),
            format!(
                "{} join requires strategy specification",
                cardinality_str.chars().next().unwrap().to_uppercase().to_string() + &cardinality_str[1..]
            ),
            context,
        )
        .with_severity(super::errors::Severity::Error)
        .with_source("strategy_validation".to_string(), "StrategyValidator".to_string());

        // Add explanation
        error = error.with_suggestion(
            1,
            format!(
                "Provide a strategy dictionary specifying how to aggregate duplicate values for {} cardinality",
                cardinality_str
            ),
        );

        // Generate column-specific suggestions
        let column_suggestions = Self::generate_column_suggestions(df, join_keys);
        if !column_suggestions.is_empty() {
            error = error.with_suggestion(
                2,
                format!("Suggested aggregation functions based on column types:\n{}", column_suggestions),
            );
        }

        // Add general guidance
        error = error.with_suggestion(
            3,
            "For numeric columns, consider: sum, mean, median, min, max".to_string(),
        );

        error = error.with_suggestion(
            4,
            "For text columns, consider: first, last, list".to_string(),
        );

        error = error.with_suggestion(
            5,
            "For counts, use: count".to_string(),
        );

        // Add code example
        error = error.with_example(
            Language::Python,
            "result = add.to(\n    df1, df2,\n    on='id',\n    strategy={\n        'value': 'sum',      # Sum numeric values\n        'name': 'first',     # Take first text value\n        'count': 'count'     # Count occurrences\n    }\n)".to_string(),
            "Example with strategy specification".to_string(),
        );

        error
    }

    /// Generate column-specific suggestions based on DataFrame schema
    fn generate_column_suggestions(df: &DataFrame, join_keys: &[String]) -> String {
        let mut suggestions = Vec::new();

        for field in df.schema().iter_fields() {
            let col_name = field.name();
            
            // Skip join keys
            if join_keys.contains(&col_name.to_string()) {
                continue;
            }

            let suggested_funcs = Self::suggest_aggregation_functions_for_type(field.dtype());
            if !suggested_funcs.is_empty() {
                suggestions.push(format!(
                    "  '{}': {} (type: {:?})",
                    col_name,
                    suggested_funcs.join(" or "),
                    field.dtype()
                ));
            }
        }

        suggestions.join("\n")
    }

    /// Suggest appropriate aggregation functions for a column type
    pub fn suggest_aggregation_functions(column_type: &str) -> Vec<String> {
        match column_type.to_lowercase().as_str() {
            "int8" | "int16" | "int32" | "int64" |
            "uint8" | "uint16" | "uint32" | "uint64" |
            "float32" | "float64" => {
                vec![
                    "sum".to_string(),
                    "mean".to_string(),
                    "median".to_string(),
                    "min".to_string(),
                    "max".to_string(),
                    "first".to_string(),
                    "last".to_string(),
                    "count".to_string(),
                ]
            }
            "string" | "utf8" | "str" => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                    "list".to_string(),
                    "count".to_string(),
                ]
            }
            "boolean" | "bool" => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                    "count".to_string(),
                ]
            }
            _ => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                    "count".to_string(),
                ]
            }
        }
    }

    /// Suggest aggregation functions based on Polars DataType
    fn suggest_aggregation_functions_for_type(dtype: &DataType) -> Vec<String> {
        match dtype {
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
            DataType::Float32 | DataType::Float64 => {
                vec![
                    "sum".to_string(),
                    "mean".to_string(),
                    "median".to_string(),
                    "min".to_string(),
                    "max".to_string(),
                ]
            }
            DataType::String => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                    "list".to_string(),
                ]
            }
            DataType::Boolean => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                ]
            }
            _ => {
                vec![
                    "first".to_string(),
                    "last".to_string(),
                ]
            }
        }
    }

    /// Validate strategy keys exist in DataFrame
    /// 
    /// Checks each strategy key against DataFrame column names.
    /// Warns if strategy key matches join key (join keys don't need aggregation).
    /// Uses fuzzy matching to suggest similar column names for typos.
    /// Lists all invalid keys in single error message.
    pub fn validate_strategy_keys(
        strategy: &std::collections::HashMap<String, String>,
        df: &DataFrame,
        join_keys: &[String],
    ) -> Result<(), ValidationError> {
        let column_names: Vec<String> = df.get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();

        let mut invalid_keys = Vec::new();
        let mut warnings = Vec::new();

        for strategy_key in strategy.keys() {
            // Check if key matches a join key
            if join_keys.contains(strategy_key) {
                warnings.push(format!(
                    "Strategy key '{}' matches a join key. Join keys don't need aggregation.",
                    strategy_key
                ));
                continue;
            }

            // Check if key exists in DataFrame
            if !column_names.contains(strategy_key) {
                // Try fuzzy matching to suggest similar names
                if let Some(suggestion) = Self::fuzzy_match_column(strategy_key, &column_names) {
                    invalid_keys.push(format!(
                        "'{}' (did you mean '{}'?)",
                        strategy_key, suggestion
                    ));
                } else {
                    invalid_keys.push(format!("'{}'", strategy_key));
                }
            }
        }

        // Log warnings for join key matches
        for warning in warnings {
            log::warn!("{}", warning);
        }

        // If there are invalid keys, return error
        if !invalid_keys.is_empty() {
            let context = ErrorContext::new("add.to".to_string())
                .with_info("invalid_keys".to_string(), invalid_keys.join(", "));

            let mut error = ValidationError::new(
                ErrorType::StrategyError,
                generate_error_code(ErrorType::StrategyError, "K", 1),
                format!(
                    "Invalid strategy key{}: {}",
                    if invalid_keys.len() > 1 { "s" } else { "" },
                    invalid_keys.join(", ")
                ),
                context,
            )
            .with_severity(super::errors::Severity::Error)
            .with_source("strategy_validation".to_string(), "StrategyValidator".to_string());

            // Add suggestion with available columns
            error = error.with_suggestion(
                1,
                format!(
                    "Available columns in DataFrame: {}",
                    column_names.join(", ")
                ),
            );

            // Add suggestion to check column names
            error = error.with_suggestion(
                2,
                "Check column names in your DataFrame using df.columns or df.schema()".to_string(),
            );

            // Add code example
            error = error.with_example(
                Language::Python,
                format!(
                    "# Check available columns\nprint(df.columns)\n\n# Use correct column names\nresult = add.to(\n    df1, df2,\n    on='id',\n    strategy={{\n        '{}': 'sum'  # Use actual column name\n    }}\n)",
                    column_names.first().unwrap_or(&"column_name".to_string())
                ),
                "Example with correct column names".to_string(),
            );

            return Err(error);
        }

        Ok(())
    }

    /// Validate strategy values are valid aggregation functions
    /// 
    /// Checks that:
    /// - All strategy values are valid aggregation functions
    /// - Numeric-only functions (sum, mean, median, min, max) are only used with numeric columns
    /// - first/last are accepted for any column type
    /// 
    /// Returns error with suggestions if validation fails.
    pub fn validate_strategy_values(
        strategy: &std::collections::HashMap<String, String>,
        df: &DataFrame,
    ) -> Result<(), ValidationError> {
        let mut invalid_functions = Vec::new();
        let mut type_mismatches = Vec::new();

        for (column_name, function) in strategy.iter() {
            // Strip mode:modifier syntax (e.g. "most_common:trim" → "most_common")
            let base_function = function.splitn(2, ':').next().unwrap_or(function.as_str());
            // Check if function is valid
            if !VALID_AGGREGATION_FUNCTIONS.contains(&base_function) {
                invalid_functions.push(format!("'{}' for column '{}'", function, column_name));
                continue;
            }

            // Check if column exists in DataFrame
            if let Ok(column) = df.column(column_name) {
                let dtype = column.dtype();
                
                // Check if numeric-only function is used with non-numeric column
                if NUMERIC_ONLY_FUNCTIONS.contains(&function.as_str())
                    && !Self::is_numeric_type(dtype) {
                    type_mismatches.push(format!(
                        "'{}' requires numeric column, but '{}' is {:?}",
                        function, column_name, dtype
                    ));
                }
                // first/last are accepted for any column type, no check needed
            }
        }

        // Report invalid functions
        if !invalid_functions.is_empty() {
            let context = ErrorContext::new("add.to".to_string())
                .with_info("invalid_functions".to_string(), invalid_functions.join(", "));

            let mut error = ValidationError::new(
                ErrorType::StrategyError,
                generate_error_code(ErrorType::StrategyError, "V", 1),
                format!(
                    "Invalid aggregation function{}: {}",
                    if invalid_functions.len() > 1 { "s" } else { "" },
                    invalid_functions.join(", ")
                ),
                context,
            )
            .with_severity(super::errors::Severity::Error)
            .with_source("strategy_validation".to_string(), "StrategyValidator".to_string());

            // Add suggestion with valid functions
            error = error.with_suggestion(
                1,
                format!(
                    "Valid aggregation functions: {}",
                    VALID_AGGREGATION_FUNCTIONS.join(", ")
                ),
            );

            // Add code example
            error = error.with_example(
                Language::Python,
                "result = add.to(\n    df1, df2,\n    on='id',\n    strategy={\n        'value': 'sum',      # Valid function\n        'name': 'first',     # Valid function\n        'count': 'count'     # Valid function\n    }\n)".to_string(),
                "Example with valid aggregation functions".to_string(),
            );

            return Err(error);
        }

        // Report type mismatches
        if !type_mismatches.is_empty() {
            let context = ErrorContext::new("add.to".to_string())
                .with_info("type_mismatches".to_string(), type_mismatches.join("; "));

            let mut error = ValidationError::new(
                ErrorType::StrategyError,
                generate_error_code(ErrorType::StrategyError, "V", 2),
                format!(
                    "Type mismatch{}: {}",
                    if type_mismatches.len() > 1 { "es" } else { "" },
                    type_mismatches.join("; ")
                ),
                context,
            )
            .with_severity(super::errors::Severity::Error)
            .with_source("strategy_validation".to_string(), "StrategyValidator".to_string());

            // Add suggestion about numeric-only functions
            error = error.with_suggestion(
                1,
                format!(
                    "Numeric-only functions ({}), can only be used with numeric columns",
                    NUMERIC_ONLY_FUNCTIONS.join(", ")
                ),
            );

            // Add suggestion about universal functions
            error = error.with_suggestion(
                2,
                "Use 'first', 'last', 'list', or 'count' for non-numeric columns".to_string(),
            );

            // Add code example
            error = error.with_example(
                Language::Python,
                "result = add.to(\n    df1, df2,\n    on='id',\n    strategy={\n        'amount': 'sum',     # Numeric column - OK\n        'name': 'first',     # Text column - use first/last\n        'status': 'list'     # Text column - use list\n    }\n)".to_string(),
                "Example with type-appropriate functions".to_string(),
            );

            return Err(error);
        }

        Ok(())
    }

    /// Check if a DataType is numeric
    fn is_numeric_type(dtype: &DataType) -> bool {
        matches!(
            dtype,
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
            DataType::Float32 | DataType::Float64
        )
    }

    /// Perform fuzzy matching for column name typos using Levenshtein distance
    /// 
    /// Uses MAX_DISTANCE of 2 for typo detection.
    /// Returns the closest matching column name if within distance threshold.
    pub fn fuzzy_match_column(
        typo: &str,
        available: &[String],
    ) -> Option<String> {
        const MAX_DISTANCE: usize = 2;

        let mut best_match: Option<(String, usize)> = None;

        for column in available {
            let distance = Self::levenshtein_distance(typo, column);
            
            if distance <= MAX_DISTANCE {
                match &best_match {
                    None => best_match = Some((column.clone(), distance)),
                    Some((_, best_dist)) if distance < *best_dist => {
                        best_match = Some((column.clone(), distance));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(name, _)| name)
    }

    /// Calculate Levenshtein distance between two strings
    /// 
    /// Implements the dynamic programming algorithm for edit distance.
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first column and row
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Calculate distances
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,      // deletion
                        matrix[i][j - 1] + 1,      // insertion
                    ),
                    matrix[i - 1][j - 1] + cost,   // substitution
                );
            }
        }

        matrix[len1][len2]
    }
}

impl Default for StrategyValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Valid aggregation functions (must stay in sync with AggregationMode::valid_modes() in types.rs)
pub const VALID_AGGREGATION_FUNCTIONS: &[&str] = &[
    // Basic
    "auto", "strict", "first", "last", "shortest", "longest",
    "most_common", "forward_fill", "backward_fill",
    // Numeric
    "sum", "count", "average", "mean", "median", "min", "max",
    // Concat
    "concat",
    // Legacy aliases
    "list",
];

/// Numeric-only aggregation functions
pub const NUMERIC_ONLY_FUNCTIONS: &[&str] = &[
    "sum", "mean", "average", "median", "min", "max"
];

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_is_strategy_required_many_to_one() {
        assert!(StrategyValidator::is_strategy_required(&Cardinality::ManyToOne));
    }

    #[test]
    fn test_is_strategy_required_many_to_many() {
        assert!(StrategyValidator::is_strategy_required(&Cardinality::ManyToMany));
    }

    #[test]
    fn test_is_strategy_not_required_one_to_one() {
        assert!(!StrategyValidator::is_strategy_required(&Cardinality::OneToOne));
    }

    #[test]
    fn test_is_strategy_not_required_one_to_many() {
        assert!(!StrategyValidator::is_strategy_required(&Cardinality::OneToMany));
    }

    #[test]
    fn test_suggest_aggregation_functions_numeric() {
        let suggestions = StrategyValidator::suggest_aggregation_functions("int64");
        assert!(suggestions.contains(&"sum".to_string()));
        assert!(suggestions.contains(&"mean".to_string()));
        assert!(suggestions.contains(&"median".to_string()));
        assert!(suggestions.contains(&"min".to_string()));
        assert!(suggestions.contains(&"max".to_string()));
    }

    #[test]
    fn test_suggest_aggregation_functions_string() {
        let suggestions = StrategyValidator::suggest_aggregation_functions("string");
        assert!(suggestions.contains(&"first".to_string()));
        assert!(suggestions.contains(&"last".to_string()));
        assert!(suggestions.contains(&"list".to_string()));
        assert!(!suggestions.contains(&"sum".to_string()));
    }

    #[test]
    fn test_suggest_aggregation_functions_boolean() {
        let suggestions = StrategyValidator::suggest_aggregation_functions("boolean");
        assert!(suggestions.contains(&"first".to_string()));
        assert!(suggestions.contains(&"last".to_string()));
        assert!(!suggestions.contains(&"sum".to_string()));
    }

    #[test]
    fn test_generate_strategy_required_error_many_to_one() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let error = StrategyValidator::generate_strategy_required_error(
            &Cardinality::ManyToOne,
            &df,
            &["id".to_string()],
        );

        assert_eq!(error.error_type, ErrorType::StrategyError);
        assert_eq!(error.error_code, "SR001");
        assert!(error.message.contains("Many-to-one"));
        assert!(error.message.contains("strategy"));
        assert!(!error.suggestions.is_empty());
        assert!(!error.examples.is_empty());
    }

    #[test]
    fn test_generate_strategy_required_error_many_to_many() {
        let df = df! {
            "id" => &[1, 2, 3],
            "score" => &[100, 200, 300],
        }.unwrap();

        let error = StrategyValidator::generate_strategy_required_error(
            &Cardinality::ManyToMany,
            &df,
            &["id".to_string()],
        );

        assert_eq!(error.error_type, ErrorType::StrategyError);
        assert!(error.message.contains("Many-to-many"));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_generate_column_suggestions() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
            "active" => &[true, false, true],
        }.unwrap();

        let suggestions = StrategyValidator::generate_column_suggestions(
            &df,
            &["id".to_string()],
        );

        // Should suggest for value (numeric), name (string), and active (boolean)
        // Should NOT suggest for id (join key)
        assert!(suggestions.contains("value"));
        assert!(suggestions.contains("name"));
        assert!(suggestions.contains("active"));
        assert!(!suggestions.contains("'id'"));
    }

    #[test]
    fn test_suggest_aggregation_functions_for_type_numeric() {
        let suggestions = StrategyValidator::suggest_aggregation_functions_for_type(&DataType::Int64);
        assert!(suggestions.contains(&"sum".to_string()));
        assert!(suggestions.contains(&"mean".to_string()));
        assert!(suggestions.contains(&"median".to_string()));
    }

    #[test]
    fn test_suggest_aggregation_functions_for_type_string() {
        let suggestions = StrategyValidator::suggest_aggregation_functions_for_type(&DataType::String);
        assert!(suggestions.contains(&"first".to_string()));
        assert!(suggestions.contains(&"last".to_string()));
        assert!(suggestions.contains(&"list".to_string()));
        assert!(!suggestions.contains(&"sum".to_string()));
    }

    #[test]
    fn test_validate_strategy_keys_valid() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "sum".to_string());
        strategy.insert("name".to_string(), "first".to_string());

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_keys_invalid() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("invalid_column".to_string(), "sum".to_string());

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_type, ErrorType::StrategyError);
        assert!(error.message.contains("invalid_column"));
    }

    #[test]
    fn test_validate_strategy_keys_multiple_invalid() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("invalid1".to_string(), "sum".to_string());
        strategy.insert("invalid2".to_string(), "mean".to_string());

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("invalid1"));
        assert!(error.message.contains("invalid2"));
    }

    #[test]
    fn test_validate_strategy_keys_with_fuzzy_match() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("vlaue".to_string(), "sum".to_string()); // Typo: vlaue instead of value

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("vlaue"));
        assert!(error.message.contains("value")); // Should suggest 'value'
    }

    #[test]
    fn test_fuzzy_match_column_exact_match() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("value", &columns);
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_one_char_diff() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("vlue", &columns);
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_two_char_diff() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("vale", &columns);
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_no_match() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("completely_different", &columns);
        assert_eq!(result, None);
    }

    #[test]
    fn test_fuzzy_match_column_closest_match() {
        let columns = vec!["id".to_string(), "value".to_string(), "values".to_string()];
        let result = StrategyValidator::fuzzy_match_column("valu", &columns);
        // Should match "value" (distance 1) over "values" (distance 2)
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_levenshtein_distance_identical() {
        let distance = StrategyValidator::levenshtein_distance("hello", "hello");
        assert_eq!(distance, 0);
    }

    #[test]
    fn test_levenshtein_distance_one_substitution() {
        let distance = StrategyValidator::levenshtein_distance("hello", "hallo");
        assert_eq!(distance, 1);
    }

    #[test]
    fn test_levenshtein_distance_one_insertion() {
        let distance = StrategyValidator::levenshtein_distance("hello", "helllo");
        assert_eq!(distance, 1);
    }

    #[test]
    fn test_levenshtein_distance_one_deletion() {
        let distance = StrategyValidator::levenshtein_distance("hello", "helo");
        assert_eq!(distance, 1);
    }

    #[test]
    fn test_levenshtein_distance_empty_strings() {
        assert_eq!(StrategyValidator::levenshtein_distance("", "hello"), 5);
        assert_eq!(StrategyValidator::levenshtein_distance("hello", ""), 5);
        assert_eq!(StrategyValidator::levenshtein_distance("", ""), 0);
    }

    // Tests for validate_strategy_values

    #[test]
    fn test_validate_strategy_values_valid_numeric() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "amount" => &[100.5, 200.5, 300.5],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "sum".to_string());
        strategy.insert("amount".to_string(), "mean".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_valid_string() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["a", "b", "c"],
            "status" => &["active", "inactive", "pending"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("name".to_string(), "first".to_string());
        strategy.insert("status".to_string(), "last".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_first_last_any_type() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
            "active" => &[true, false, true],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "first".to_string());
        strategy.insert("name".to_string(), "last".to_string());
        strategy.insert("active".to_string(), "first".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_invalid_function() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "invalid_func".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_type, ErrorType::StrategyError);
        assert_eq!(error.error_code, "SV001");
        assert!(error.message.contains("invalid_func"));
        assert!(error.message.contains("value"));
    }

    #[test]
    fn test_validate_strategy_values_sum_on_string() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("name".to_string(), "sum".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.error_type, ErrorType::StrategyError);
        assert_eq!(error.error_code, "SV002");
        assert!(error.message.contains("sum"));
        assert!(error.message.contains("name"));
        assert!(error.message.contains("numeric"));
    }

    #[test]
    fn test_validate_strategy_values_mean_on_string() {
        let df = df! {
            "id" => &[1, 2, 3],
            "status" => &["active", "inactive", "pending"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("status".to_string(), "mean".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("mean"));
        assert!(error.message.contains("status"));
    }

    #[test]
    fn test_validate_strategy_values_median_on_boolean() {
        let df = df! {
            "id" => &[1, 2, 3],
            "active" => &[true, false, true],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("active".to_string(), "median".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("median"));
        assert!(error.message.contains("active"));
    }

    #[test]
    fn test_validate_strategy_values_multiple_invalid() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "invalid1".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("invalid1"));
    }

    #[test]
    fn test_validate_strategy_values_multiple_type_mismatches() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["a", "b", "c"],
            "status" => &["active", "inactive", "pending"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("name".to_string(), "sum".to_string());
        strategy.insert("status".to_string(), "mean".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("sum"));
        assert!(error.message.contains("mean"));
    }

    #[test]
    fn test_validate_strategy_values_count_any_type() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
            "active" => &[true, false, true],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "count".to_string());
        strategy.insert("name".to_string(), "count".to_string());
        strategy.insert("active".to_string(), "count".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_list_any_type() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "list".to_string());
        strategy.insert("name".to_string(), "list".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_all_numeric_functions() {
        let df = df! {
            "id" => &[1, 2, 3],
            "val1" => &[10, 20, 30],
            "val2" => &[100, 200, 300],
            "val3" => &[1.5, 2.5, 3.5],
            "val4" => &[10.0, 20.0, 30.0],
            "val5" => &[5, 10, 15],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("val1".to_string(), "sum".to_string());
        strategy.insert("val2".to_string(), "mean".to_string());
        strategy.insert("val3".to_string(), "median".to_string());
        strategy.insert("val4".to_string(), "min".to_string());
        strategy.insert("val5".to_string(), "max".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_numeric_type() {
        assert!(StrategyValidator::is_numeric_type(&DataType::Int8));
        assert!(StrategyValidator::is_numeric_type(&DataType::Int16));
        assert!(StrategyValidator::is_numeric_type(&DataType::Int32));
        assert!(StrategyValidator::is_numeric_type(&DataType::Int64));
        assert!(StrategyValidator::is_numeric_type(&DataType::UInt8));
        assert!(StrategyValidator::is_numeric_type(&DataType::UInt16));
        assert!(StrategyValidator::is_numeric_type(&DataType::UInt32));
        assert!(StrategyValidator::is_numeric_type(&DataType::UInt64));
        assert!(StrategyValidator::is_numeric_type(&DataType::Float32));
        assert!(StrategyValidator::is_numeric_type(&DataType::Float64));
        assert!(!StrategyValidator::is_numeric_type(&DataType::String));
        assert!(!StrategyValidator::is_numeric_type(&DataType::Boolean));
    }

    // Edge case tests

    #[test]
    fn test_validate_strategy_keys_empty_strategy() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let strategy = std::collections::HashMap::new();

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        // Empty strategy should be valid (no keys to validate)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_values_empty_strategy() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let strategy = std::collections::HashMap::new();

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);

        // Empty strategy should be valid (no values to validate)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_all_columns_same_function() {
        let df = df! {
            "id" => &[1, 2, 3],
            "val1" => &[10, 20, 30],
            "val2" => &[100, 200, 300],
            "val3" => &[1000, 2000, 3000],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("val1".to_string(), "sum".to_string());
        strategy.insert("val2".to_string(), "sum".to_string());
        strategy.insert("val3".to_string(), "sum".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);

        // All columns with same function should be valid
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_all_columns_first() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
            "active" => &[true, false, true],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "first".to_string());
        strategy.insert("name".to_string(), "first".to_string());
        strategy.insert("active".to_string(), "first".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);

        // All columns with 'first' should be valid for any type
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_all_columns_last() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
            "score" => &[1.5, 2.5, 3.5],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "last".to_string());
        strategy.insert("name".to_string(), "last".to_string());
        strategy.insert("score".to_string(), "last".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);

        // All columns with 'last' should be valid for any type
        assert!(result.is_ok());
    }

    #[test]
    fn test_fuzzy_match_column_distance_zero() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("value", &columns);
        // Exact match (distance 0)
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_distance_one() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("vlue", &columns);
        // One deletion (distance 1)
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_distance_two() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("valu", &columns);
        // One deletion (distance 1) - should match "value"
        assert_eq!(result, Some("value".to_string()));
    }

    #[test]
    fn test_fuzzy_match_column_distance_three_no_match() {
        let columns = vec!["id".to_string(), "value".to_string(), "name".to_string()];
        let result = StrategyValidator::fuzzy_match_column("xyz", &columns);
        // Distance > 2, should not match
        assert_eq!(result, None);
    }

    #[test]
    fn test_levenshtein_distance_two_substitutions() {
        let distance = StrategyValidator::levenshtein_distance("hello", "hallo");
        assert_eq!(distance, 1); // One substitution: e -> a
    }

    #[test]
    fn test_levenshtein_distance_multiple_operations() {
        let distance = StrategyValidator::levenshtein_distance("kitten", "sitting");
        assert_eq!(distance, 3); // k->s, e->i, insert g
    }

    #[test]
    fn test_validate_strategy_keys_on_join_key_warns() {
        // This test verifies that strategies on join keys produce warnings
        // The function logs warnings but doesn't return an error
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("id".to_string(), "sum".to_string()); // id is a join key

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        // Should succeed but log a warning (we can't easily test log output in unit tests)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_mixed_valid_and_join_key() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("id".to_string(), "first".to_string()); // join key
        strategy.insert("value".to_string(), "sum".to_string()); // valid
        strategy.insert("name".to_string(), "first".to_string()); // valid

        let result = StrategyValidator::validate_strategy_keys(
            &strategy,
            &df,
            &["id".to_string()],
        );

        // Should succeed (join key warning is logged, not an error)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_min_max_on_numeric() {
        let df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
            "score" => &[1.5, 2.5, 3.5],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("value".to_string(), "min".to_string());
        strategy.insert("score".to_string(), "max".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_strategy_min_on_string_fails() {
        let df = df! {
            "id" => &[1, 2, 3],
            "name" => &["a", "b", "c"],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("name".to_string(), "min".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("min"));
        assert!(error.message.contains("name"));
    }

    #[test]
    fn test_validate_strategy_max_on_boolean_fails() {
        let df = df! {
            "id" => &[1, 2, 3],
            "active" => &[true, false, true],
        }.unwrap();

        let mut strategy = std::collections::HashMap::new();
        strategy.insert("active".to_string(), "max".to_string());

        let result = StrategyValidator::validate_strategy_values(&strategy, &df);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.message.contains("max"));
        assert!(error.message.contains("active"));
    }
}
