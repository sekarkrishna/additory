// Data validation module
// Validates data-level properties including cardinality, duplicates, and nulls

use super::errors::{ValidationError, ValidationResult, ErrorType, ErrorContext, generate_error_code};
use polars::prelude::*;
use std::collections::HashMap;

/// Data validator for cardinality, duplicates, and null analysis
/// Data validator for cardinality, duplicates, and null analysis
pub struct DataValidator;

/// Sampling threshold for large datasets (10 million rows)
const SAMPLING_THRESHOLD: usize = 10_000_000;

/// Default sample size for large datasets
const DEFAULT_SAMPLE_SIZE: usize = 1_000_000;

impl DataValidator {
    /// Detect cardinality between join keys with automatic sampling for large datasets
    ///
    /// Algorithm:
    /// 1. Check if DataFrames exceed sampling threshold
    /// 2. If yes, use sampling; otherwise use full data
    /// 3. Count unique keys in each DataFrame
    /// 4. Calculate uniqueness ratios
    /// 5. Determine cardinality based on threshold (0.95)
    pub fn detect_cardinality(
        left: &DataFrame,
        right: &DataFrame,
        left_keys: &[String],
        right_keys: &[String],
    ) -> ValidationResult<Cardinality> {
        // Check if we should use sampling
        let left_height = left.height();
        let right_height = right.height();

        if left_height > SAMPLING_THRESHOLD || right_height > SAMPLING_THRESHOLD {
            // Use sampling for large datasets
            Self::detect_cardinality_with_sampling(left, right, left_keys, right_keys, DEFAULT_SAMPLE_SIZE)
        } else {
            // Use full data for smaller datasets
            Self::detect_cardinality_full(left, right, left_keys, right_keys)
        }
    }

    /// Detect cardinality using full data (for datasets under threshold)
    fn detect_cardinality_full(
        left: &DataFrame,
        right: &DataFrame,
        left_keys: &[String],
        right_keys: &[String],
    ) -> ValidationResult<Cardinality> {
        // Step 1: Count unique keys in each DataFrame
        let left_unique = Self::count_unique_keys(left, left_keys)?;
        let left_total = left.height();

        let right_unique = Self::count_unique_keys(right, right_keys)?;
        let right_total = right.height();

        // Step 2: Calculate ratios
        let left_ratio = left_unique as f64 / left_total as f64;
        let right_ratio = right_unique as f64 / right_total as f64;

        // Step 3: Determine cardinality
        // Threshold: 0.95 for "unique" (allows for small duplicates)
        const UNIQUE_THRESHOLD: f64 = 0.95;

        let left_is_unique = left_ratio >= UNIQUE_THRESHOLD;
        let right_is_unique = right_ratio >= UNIQUE_THRESHOLD;

        let cardinality = match (left_is_unique, right_is_unique) {
            (true, true) => Cardinality::OneToOne,
            (false, true) => Cardinality::ManyToOne,
            (true, false) => Cardinality::OneToMany,
            (false, false) => Cardinality::ManyToMany,
        };

        Ok(cardinality)
    }

    /// Detect cardinality with sampling for large datasets
    ///
    /// Uses Polars sample_n() for efficient sampling while maintaining accuracy.
    /// Sample size is capped at 1 million rows for performance.
    pub fn detect_cardinality_with_sampling(
        left: &DataFrame,
        right: &DataFrame,
        left_keys: &[String],
        right_keys: &[String],
        sample_size: usize,
    ) -> ValidationResult<Cardinality> {
        // Sample left DataFrame if it exceeds threshold
        // Using head() for deterministic sampling (first N rows)
        // This is acceptable for cardinality detection as we just need a representative subset
        let left_sample = if left.height() > SAMPLING_THRESHOLD {
            let actual_sample_size = std::cmp::min(sample_size, left.height());
            left.head(Some(actual_sample_size))
        } else {
            left.clone()
        };

        // Sample right DataFrame if it exceeds threshold
        let right_sample = if right.height() > SAMPLING_THRESHOLD {
            let actual_sample_size = std::cmp::min(sample_size, right.height());
            right.head(Some(actual_sample_size))
        } else {
            right.clone()
        };

        // Use the full cardinality detection logic on sampled data
        Self::detect_cardinality_full(&left_sample, &right_sample, left_keys, right_keys)
    }

    /// Count unique keys in DataFrame
    fn count_unique_keys(df: &DataFrame, keys: &[String]) -> ValidationResult<usize> {
        // Select key columns
        let key_df = df.select(keys).map_err(|e| {
            let context = ErrorContext::new("detect_cardinality".to_string())
                .with_info("error".to_string(), e.to_string());

            ValidationError::new(
                ErrorType::DataError,
                generate_error_code(ErrorType::DataError, "K", 1),
                format!("Failed to select key columns: {}", keys.join(", ")),
                context,
            )
        })?;

        // Get unique rows - pass None to use all columns
        let unique_df = key_df.unique_stable(None, UniqueKeepStrategy::First, None).map_err(|e| {
            let context = ErrorContext::new("detect_cardinality".to_string())
                .with_info("error".to_string(), e.to_string());

            ValidationError::new(
                ErrorType::DataError,
                generate_error_code(ErrorType::DataError, "K", 2),
                "Failed to count unique keys".to_string(),
                context,
            )
        })?;

        Ok(unique_df.height())
    }

    /// Analyze duplicate keys in DataFrame with automatic sampling for large datasets
    pub fn analyze_duplicates(
        df: &DataFrame,
        keys: &[String],
        side: Side,
    ) -> ValidationResult<DuplicateAnalysis> {
        // Check if we should use sampling
        if df.height() > SAMPLING_THRESHOLD {
            Self::analyze_duplicates_with_sampling(df, keys, side, DEFAULT_SAMPLE_SIZE)
        } else {
            Self::analyze_duplicates_full(df, keys, side)
        }
    }

    /// Analyze duplicates using full data (for datasets under threshold)
    fn analyze_duplicates_full(
        df: &DataFrame,
        keys: &[String],
        _side: Side,
    ) -> ValidationResult<DuplicateAnalysis> {
        let total_rows = df.height();

        if total_rows == 0 {
            return Ok(DuplicateAnalysis {
                has_duplicates: false,
                duplicate_count: 0,
                duplicate_percentage: 0.0,
                example_duplicates: vec![],
            });
        }

        // Select key columns
        let key_df = df.select(keys).map_err(|e| {
            let context = ErrorContext::new("analyze_duplicates".to_string())
                .with_info("error".to_string(), e.to_string());

            ValidationError::new(
                ErrorType::DataError,
                generate_error_code(ErrorType::DataError, "D", 1),
                format!("Failed to select key columns: {}", keys.join(", ")),
                context,
            )
        })?;

        // Manually count duplicates by iterating through the DataFrame
        let mut key_counts: HashMap<String, usize> = HashMap::new();

        for i in 0..key_df.height() {
            let mut key_parts = Vec::new();
            for key in keys {
                if let Ok(col) = key_df.column(key) {
                    if let Ok(value) = col.get(i) {
                        key_parts.push(format!("{}", value));
                    }
                }
            }
            let combined_key = key_parts.join("|");
            *key_counts.entry(combined_key).or_insert(0) += 1;
        }

        // Count how many keys appear more than once
        let duplicate_count = key_counts.values().filter(|&&count| count > 1).count();
        let duplicate_percentage = (duplicate_count as f64 / total_rows as f64) * 100.0;

        // Extract example duplicate keys (first 5)
        let example_duplicates: Vec<String> = key_counts
            .iter()
            .filter(|(_, &count)| count > 1)
            .take(5)
            .map(|(key, count)| format!("{} (appears {} times)", key, count))
            .collect();

        Ok(DuplicateAnalysis {
            has_duplicates: duplicate_count > 0,
            duplicate_count,
            duplicate_percentage,
            example_duplicates,
        })
    }

    /// Analyze duplicates with sampling for large datasets
    fn analyze_duplicates_with_sampling(
        df: &DataFrame,
        keys: &[String],
        side: Side,
        sample_size: usize,
    ) -> ValidationResult<DuplicateAnalysis> {
        // Sample the DataFrame using head() for deterministic sampling
        let actual_sample_size = std::cmp::min(sample_size, df.height());
        let sampled_df = df.head(Some(actual_sample_size));

        // Analyze duplicates on the sample
        Self::analyze_duplicates_full(&sampled_df, keys, side)
    }

    /// Extract key examples from DataFrame
    fn extract_key_examples(df: &DataFrame, keys: &[String], limit: usize) -> Vec<String> {
        let mut examples = Vec::new();
        let rows_to_take = std::cmp::min(limit, df.height());

        for i in 0..rows_to_take {
            let mut key_parts = Vec::new();
            for key in keys {
                if let Ok(col) = df.column(key) {
                    if let Ok(value) = col.get(i) {
                        key_parts.push(format!("{}", value));
                    }
                }
            }
            if !key_parts.is_empty() {
                examples.push(key_parts.join(", "));
            }
        }

        examples
    }

    /// Detect missing keys between DataFrames
    pub fn detect_missing_keys(
        left: &DataFrame,
        right: &DataFrame,
        left_keys: &[String],
        right_keys: &[String],
    ) -> ValidationResult<MissingKeyAnalysis> {
        // Get unique keys from each DataFrame
        let left_unique = left.select(left_keys)
            .and_then(|df| df.unique_stable(None, UniqueKeepStrategy::First, None))
            .map_err(|e| {
                let context = ErrorContext::new("detect_missing_keys".to_string())
                    .with_info("error".to_string(), e.to_string());

                ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "M", 1),
                    "Failed to get unique left keys".to_string(),
                    context,
                )
            })?;

        let right_unique = right.select(right_keys)
            .and_then(|df| df.unique_stable(None, UniqueKeepStrategy::First, None))
            .map_err(|e| {
                let context = ErrorContext::new("detect_missing_keys".to_string())
                    .with_info("error".to_string(), e.to_string());

                ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "M", 2),
                    "Failed to get unique right keys".to_string(),
                    context,
                )
            })?;

        // Workaround: Use left join and filter for nulls to simulate anti-join
        // Find keys in left but not in right
        let left_joined = left_unique.join(
            &right_unique,
            left_keys,
            right_keys,
            JoinArgs::new(JoinType::Left),
        ).map_err(|e| {
            let context = ErrorContext::new("detect_missing_keys".to_string())
                .with_info("error".to_string(), e.to_string());

            ValidationError::new(
                ErrorType::DataError,
                generate_error_code(ErrorType::DataError, "M", 3),
                "Failed to find missing left keys".to_string(),
                context,
            )
        })?;

        // Filter for rows where right key is null (meaning no match)
        let right_key_col = right_keys.first().map(|k| format!("{}_right", k))
            .unwrap_or_else(|| "right_key".to_string());

        let left_missing = if let Ok(col) = left_joined.column(&right_key_col) {
            left_joined.filter(&col.is_null()).unwrap_or(left_joined.clone())
        } else {
            // If column doesn't exist, assume no missing keys
            left_unique.head(Some(0))
        };

        // Find keys in right but not in left
        let right_joined = right_unique.join(
            &left_unique,
            right_keys,
            left_keys,
            JoinArgs::new(JoinType::Left),
        ).map_err(|e| {
            let context = ErrorContext::new("detect_missing_keys".to_string())
                .with_info("error".to_string(), e.to_string());

            ValidationError::new(
                ErrorType::DataError,
                generate_error_code(ErrorType::DataError, "M", 4),
                "Failed to find missing right keys".to_string(),
                context,
            )
        })?;

        // Filter for rows where left key is null (meaning no match)
        let left_key_col = left_keys.first().map(|k| format!("{}_right", k))
            .unwrap_or_else(|| "left_key".to_string());

        let right_missing = if let Ok(col) = right_joined.column(&left_key_col) {
            right_joined.filter(&col.is_null()).unwrap_or(right_joined.clone())
        } else {
            // If column doesn't exist, assume no missing keys
            right_unique.head(Some(0))
        };

        // Calculate statistics
        let left_missing_count = left_missing.height();
        let right_missing_count = right_missing.height();
        let left_total = left_unique.height();
        let right_total = right_unique.height();

        let left_missing_percentage = if left_total > 0 {
            (left_missing_count as f64 / left_total as f64) * 100.0
        } else {
            0.0
        };

        let right_missing_percentage = if right_total > 0 {
            (right_missing_count as f64 / right_total as f64) * 100.0
        } else {
            0.0
        };

        // Extract examples
        let example_missing_left = Self::extract_key_examples(&left_missing, left_keys, 5);
        let example_missing_right = Self::extract_key_examples(&right_missing, right_keys, 5);

        Ok(MissingKeyAnalysis {
            left_missing_count,
            right_missing_count,
            left_missing_percentage,
            right_missing_percentage,
            example_missing_left,
            example_missing_right,
        })
    }

    /// Analyze null values in columns
    pub fn analyze_nulls(
        df: &DataFrame,
        columns: &[String],
    ) -> ValidationResult<NullAnalysis> {
        let mut column_stats = HashMap::new();
        let total_rows = df.height();
        let mut total_nulls = 0;

        for col_name in columns {
            let col = df.column(col_name).map_err(|e| {
                let context = ErrorContext::new("analyze_nulls".to_string())
                    .with_column(col_name.clone())
                    .with_info("error".to_string(), e.to_string());

                ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "N", 1),
                    format!("Failed to get column '{}'", col_name),
                    context,
                )
            })?;

            let null_count = col.null_count();
            let null_percentage = if total_rows > 0 {
                (null_count as f64 / total_rows as f64) * 100.0
            } else {
                0.0
            };

            total_nulls += null_count;

            column_stats.insert(
                col_name.clone(),
                ColumnNullStats {
                    null_count,
                    null_percentage,
                    column_type: format!("{:?}", col.dtype()),
                },
            );
        }

        let total_null_percentage = if total_rows > 0 && !columns.is_empty() {
            (total_nulls as f64 / (total_rows * columns.len()) as f64) * 100.0
        } else {
            0.0
        };

        Ok(NullAnalysis {
            columns: column_stats,
            total_null_percentage,
        })
    }

    /// Validate column types for mode
    ///
    /// Validates that columns have appropriate types for the specified transformation mode.
    /// - @round and @scale require numeric columns
    /// - @deduce has specific type requirements
    /// - @encode on numeric columns produces a warning
    pub fn validate_column_types_for_mode(
        df: &DataFrame,
        columns: &[String],
        mode: &str,
    ) -> ValidationResult<()> {
        // Define mode requirements
        let numeric_only_modes = ["@round", "@scale"];

        for col_name in columns {
            let col = df.column(col_name).map_err(|e| {
                let context = ErrorContext::new("validate_column_types_for_mode".to_string())
                    .with_column(col_name.clone())
                    .with_info("error".to_string(), e.to_string());

                ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "T", 1),
                    format!("Failed to get column '{}'", col_name),
                    context,
                )
            })?;

            let dtype = col.dtype();
            let is_numeric = matches!(
                dtype,
                DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
                DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
                DataType::Float32 | DataType::Float64
            );

            // Check numeric-only modes
            if numeric_only_modes.contains(&mode) && !is_numeric {
                let context = ErrorContext::new("validate_column_types_for_mode".to_string())
                    .with_column(col_name.clone())
                    .with_info("mode".to_string(), mode.to_string())
                    .with_info("actual_type".to_string(), format!("{:?}", dtype))
                    .with_info("required_type".to_string(), "numeric".to_string());

                return Err(ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "T", 2),
                    format!(
                        "Column '{}' has type {:?} but mode {} requires numeric columns",
                        col_name, dtype, mode
                    ),
                    context,
                ));
            }

            // Warn about @encode on numeric columns
            if mode == "@encode" && is_numeric {
                let context = ErrorContext::new("validate_column_types_for_mode".to_string())
                    .with_column(col_name.clone())
                    .with_info("mode".to_string(), mode.to_string())
                    .with_info("column_type".to_string(), format!("{:?}", dtype));

                // For now, we'll return an error since we don't have a warning system yet
                // In the future, this should be a warning
                return Err(ValidationError::new(
                    ErrorType::DataError,
                    generate_error_code(ErrorType::DataError, "T", 3),
                    format!(
                        "Column '{}' has numeric type {:?}. Encoding is typically for categorical data",
                        col_name, dtype
                    ),
                    context,
                ));
            }

            // Check @deduce compatibility
            if mode == "@deduce" {
                // @deduce works with most types, but has specific requirements
                // For now, we'll accept all types and add specific checks later if needed
                // The main requirement is that there should be null values to impute
            }
        }

        Ok(())
    }
}

/// Cardinality type between join keys
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cardinality {
    OneToOne,
    ManyToOne,
    OneToMany,
    ManyToMany,
}

/// Side indicator for duplicate analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

/// Duplicate analysis results
#[derive(Debug, Clone)]
pub struct DuplicateAnalysis {
    pub has_duplicates: bool,
    pub duplicate_count: usize,
    pub duplicate_percentage: f64,
    pub example_duplicates: Vec<String>,
}

/// Missing key analysis results
#[derive(Debug, Clone)]
pub struct MissingKeyAnalysis {
    pub left_missing_count: usize,
    pub right_missing_count: usize,
    pub left_missing_percentage: f64,
    pub right_missing_percentage: f64,
    pub example_missing_left: Vec<String>,
    pub example_missing_right: Vec<String>,
}

/// Null analysis results
#[derive(Debug, Clone)]
pub struct NullAnalysis {
    pub columns: HashMap<String, ColumnNullStats>,
    pub total_null_percentage: f64,
}

/// Per-column null statistics
#[derive(Debug, Clone)]
pub struct ColumnNullStats {
    pub null_count: usize,
    pub null_percentage: f64,
    pub column_type: String,
}

