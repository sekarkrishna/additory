//! @deduce mode - Missing value imputation
//!
//! This module implements 7 imputation methods for filling missing values:
//! - auto: Automatically choose method based on data type
//! - mean: Fill with column mean (numeric only)
//! - median: Fill with column median (numeric only)
//! - mode: Fill with most frequent value
//! - forward: Forward fill (propagate last valid value)
//! - backward: Backward fill (propagate next valid value)
//! - knn: K-Nearest Neighbors imputation
//!
//! # Requirements
//! - Requirement 5.2: Add @deduce as a transform mode
//! - Requirement 5.3: Support 7 imputation methods
//! - Requirement 5.4: Support k parameter in strategy dict for KNN method
//! - Requirement 8.1-8.5: Preserve original columns (addition-only philosophy)
//! - Requirement 11.2: Create new columns instead of modifying existing

use crate::core::{AdditoryError, AdditoryResult, DataFrame};
use crate::transform::knn::{KnnImputer, WeightStrategy, DistanceMetric};
use crate::utils::tfidf::{TfidfVectorizer, cosine_similarity};
use polars::prelude::*;
use std::collections::HashMap;

/// Imputation method for @deduce mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImputationMethod {
    /// Automatically choose method based on data type
    Auto,
    /// Fill with column mean (numeric only)
    Mean,
    /// Fill with column median (numeric only)
    Median,
    /// Fill with most frequent value
    Mode,
    /// Forward fill (propagate last valid value)
    Forward,
    /// Backward fill (propagate next valid value)
    Backward,
    /// K-Nearest Neighbors imputation
    Knn,
    /// TF-IDF based label deduction (text-based)
    Tfidf,
}

impl ImputationMethod {
    /// Parse imputation method from string
    pub fn parse_method(s: &str) -> Result<Self, AdditoryError> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(ImputationMethod::Auto),
            "mean" => Ok(ImputationMethod::Mean),
            "median" => Ok(ImputationMethod::Median),
            "mode" => Ok(ImputationMethod::Mode),
            "forward" | "ffill" => Ok(ImputationMethod::Forward),
            "backward" | "bfill" => Ok(ImputationMethod::Backward),
            "knn" => Ok(ImputationMethod::Knn),
            "tfidf" => Ok(ImputationMethod::Tfidf),
            _ => Err(AdditoryError::Validation(
                format!("Invalid imputation method: '{}'", s),
                "method must be one of: auto, mean, median, mode, forward, backward, knn, tfidf".to_string(),
            )),
        }
    }
}

/// Deduce (impute) missing values in specified columns
///
/// # Arguments
/// * `df` - Input DataFrame with missing values
/// * `infer_columns` - Columns to impute (contains missing values)
/// * `output_columns` - Names for new columns with imputed values
/// * `method` - Imputation method for each column
/// * `against_text` - Text columns for TF-IDF similarity (optional)
/// * `strategy` - Strategy dictionary with method parameters (optional)
///
/// # Strategy Parameters
/// - k: Number of neighbors for KNN (default: 5)
/// - weights: Weight strategy for KNN (default: "uniform")
/// - metric: Distance metric for KNN (default: "euclidean")
///
/// # Returns
/// DataFrame with new columns containing imputed values (original columns preserved)
pub fn deduce(
    df: DataFrame,
    infer_columns: Vec<String>,
    output_columns: Vec<String>,
    method: Vec<String>,
    _against_text: Option<Vec<String>>,
    strategy: Option<HashMap<String, crate::core::StrategyValue>>,
) -> AdditoryResult<DataFrame> {
    // Validate parameters
    if infer_columns.len() != output_columns.len() {
        return Err(AdditoryError::Validation(
            format!("Number of infer columns ({}) must match number of output columns ({})", 
                    infer_columns.len(), output_columns.len()),
            "Provide one output name for each infer column".to_string(),
        ));
    }
    
    if infer_columns.len() != method.len() {
        return Err(AdditoryError::Validation(
            format!("Number of infer columns ({}) must match number of methods ({})", 
                    infer_columns.len(), method.len()),
            "Provide one method for each infer column".to_string(),
        ));
    }
    
    // Start with original DataFrame
    let mut result_df = df;
    
    // Process each column
    for i in 0..infer_columns.len() {
        let infer_col = &infer_columns[i];
        let output_col = &output_columns[i];
        let method_str = &method[i];
        
        // Validate infer column exists
        if !result_df.column_names().iter().any(|c| c == infer_col) {
            return Err(AdditoryError::Validation(
                format!("Column '{}' not found in DataFrame", infer_col),
                format!("Available columns: {:?}", result_df.column_names()),
            ));
        }
        
        // Validate output column doesn't exist
        if result_df.column_names().iter().any(|c| c == output_col) {
            return Err(AdditoryError::Validation(
                format!("Column '{}' already exists in DataFrame", output_col),
                "Use a different 'name' parameter or remove the existing column".to_string(),
            ));
        }
        
        // Parse imputation method
        let imputation_method = ImputationMethod::parse_method(method_str)?;
        
        // Apply imputation based on method
        let filled_series = match imputation_method {
            ImputationMethod::Auto => {
                // Auto-detect method based on data type
                let col = result_df.column(infer_col)?;
                let dtype = col.dtype();
                let auto_method = match dtype {
                    DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
                    DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
                    DataType::Float32 | DataType::Float64 => ImputationMethod::Mean,
                    DataType::String | DataType::Categorical(_, _) => ImputationMethod::Mode,
                    _ => ImputationMethod::Forward,
                };
                
                match auto_method {
                    ImputationMethod::Mean => impute_mean_series(&result_df, infer_col)?,
                    ImputationMethod::Mode => impute_mode_series(&result_df, infer_col)?,
                    ImputationMethod::Forward => impute_forward_series(&result_df, infer_col)?,
                    _ => return Err(AdditoryError::operation("Auto-detection failed", "Unexpected method")),
                }
            }
            ImputationMethod::Mean => impute_mean_series(&result_df, infer_col)?,
            ImputationMethod::Median => impute_median_series(&result_df, infer_col)?,
            ImputationMethod::Mode => impute_mode_series(&result_df, infer_col)?,
            ImputationMethod::Forward => impute_forward_series(&result_df, infer_col)?,
            ImputationMethod::Backward => impute_backward_series(&result_df, infer_col)?,
            ImputationMethod::Knn => {
                // Extract KNN parameters from strategy
                let k = if let Some(ref strat) = strategy {
                    if let Some(crate::core::StrategyValue::Number(n)) = strat.get("k") {
                        *n as usize
                    } else {
                        5
                    }
                } else {
                    5
                };
                
                let weights_str = if let Some(ref strat) = strategy {
                    if let Some(crate::core::StrategyValue::String(s)) = strat.get("weights") {
                        s.as_str()
                    } else {
                        "uniform"
                    }
                } else {
                    "uniform"
                };
                let weights = WeightStrategy::parse_strategy(weights_str)?;
                
                let metric_str = if let Some(ref strat) = strategy {
                    if let Some(crate::core::StrategyValue::String(s)) = strat.get("metric") {
                        s.as_str()
                    } else {
                        "euclidean"
                    }
                } else {
                    "euclidean"
                };
                let metric = DistanceMetric::parse_metric(metric_str)?;
                
                // Use KNN imputer - this returns a DataFrame, we need to extract the series
                let imputer = KnnImputer::new(k, weights, metric)?;
                let imputed_df = imputer.impute(result_df.clone(), vec![infer_col.clone()])?;
                imputed_df.column(infer_col)?.as_materialized_series().clone()
            }
            ImputationMethod::Tfidf => {
                // Extract against_text parameter
                let against_cols = _against_text.as_ref()
                    .ok_or_else(|| AdditoryError::missing_parameter(
                        "against",
                        "TF-IDF method requires 'against' parameter specifying text columns for similarity calculation"
                    ))?;
                
                // Call impute_tfidf_series with correct parameters
                impute_tfidf_series(&result_df, infer_col, against_cols)?
            }
        };
        
        // Add new column with imputed values (preserves original)
        let new_series = filled_series.with_name(output_col.as_str().into());
        result_df = result_df.with_column(Column::Series(new_series))?;
    }
    
    Ok(result_df)
}

/// Validate parameters for TF-IDF label deduction
///
/// Checks:
/// 1. Target column (infer_col) exists
/// 2. All text columns (against_cols) exist
/// 3. At least 3 labeled examples exist in the target column
///
/// # Arguments
/// * `df` - Reference to the DataFrame to validate
/// * `infer_col` - Name of the column containing labels (with some null values)
/// * `against_cols` - Vector of column names containing text for similarity calculation
///
/// # Returns
/// Ok(()) if validation passes, Err(AdditoryError) otherwise
///
/// # Errors
/// - ColumnNotFound: If infer_col or any against_col doesn't exist
/// - Validation: If fewer than 3 labeled examples exist
fn validate_tfidf_params(
    df: &DataFrame,
    infer_col: &str,
    against_cols: &[String],
) -> AdditoryResult<()> {
    const MIN_LABELED: usize = 3;
    
    // Check target column exists
    if !df.has_column(infer_col) {
        return Err(AdditoryError::column_not_found(
            infer_col,
            &df.column_names(),
        ));
    }

    // Check all text columns exist
    for text_col in against_cols {
        if !df.has_column(text_col) {
            return Err(AdditoryError::column_not_found(
                text_col,
                &df.column_names(),
            ));
        }
    }

    // Count labeled examples (non-null values in target column)
    let target_col = df.column(infer_col)?;
    let null_count = target_col.null_count();
    let labeled_count = df.height() - null_count;

    // At least 3 labeled examples required
    if labeled_count < MIN_LABELED {
        return Err(AdditoryError::validation(
            &format!(
                "Insufficient labeled examples: found {}, minimum {} required",
                labeled_count, MIN_LABELED
            ),
            "Provide at least 3 rows with non-null values in the target column",
        ));
    }

    Ok(())
}

/// Separate DataFrame into labeled and unlabeled rows
///
/// Splits the DataFrame based on whether the target column (infer_col) has null values.
/// Labeled rows have non-null values in the target column, unlabeled rows have null values.
///
/// # Arguments
/// * `df` - Reference to the DataFrame to split
/// * `infer_col` - Name of the column to check for null values
///
/// # Returns
/// Tuple of (labeled_df, unlabeled_df) where:
/// - labeled_df: DataFrame with non-null values in infer_col
/// - unlabeled_df: DataFrame with null values in infer_col
///
/// # Errors
/// Returns AdditoryError if column doesn't exist or filtering fails
fn separate_labeled_unlabeled(df: &DataFrame, infer_col: &str) -> AdditoryResult<(DataFrame, DataFrame)> {
    let polars_df = df.inner();
    
    // Create mask for labeled rows (non-null in target column)
    let target_col = polars_df.column(infer_col)?;
    let is_labeled = target_col.is_not_null();
    let is_unlabeled = target_col.is_null();
    
    // Filter labeled and unlabeled rows
    let labeled_polars = polars_df.filter(&is_labeled)?;
    let unlabeled_polars = polars_df.filter(&is_unlabeled)?;
    
    Ok((
        DataFrame::from_polars(labeled_polars),
        DataFrame::from_polars(unlabeled_polars),
    ))
}

/// Combine text from multiple columns into a single string per row
fn combine_text_columns(df: &DataFrame, text_columns: &[String]) -> AdditoryResult<Vec<String>> {
    let polars_df = df.inner();
    let num_rows = polars_df.height();
    
    let mut combined_texts = vec![String::new(); num_rows];
    
    // Iterate through each text column
    for (col_idx, text_col_name) in text_columns.iter().enumerate() {
        let col = polars_df.column(text_col_name)?;
        let str_col = col.str()?;
        
        // Add text from this column to combined texts
        for (row_idx, opt_text) in str_col.into_iter().enumerate() {
            if let Some(text) = opt_text {
                if col_idx > 0 && !combined_texts[row_idx].is_empty() {
                    combined_texts[row_idx].push(' '); // Add space between columns
                }
                combined_texts[row_idx].push_str(text);
            }
        }
    }
    
    Ok(combined_texts)
}

/// Assign labels to unlabeled rows based on cosine similarity to labeled rows
///
/// For each unlabeled row, finds the labeled row with the highest cosine similarity
/// and assigns its label to the unlabeled row.
///
/// # Arguments
/// * `unlabeled_vectors` - TF-IDF vectors for unlabeled rows
/// * `labeled_vectors` - TF-IDF vectors for labeled rows
/// * `labeled_labels` - Labels corresponding to labeled rows
///
/// # Returns
/// Vector of labels assigned to unlabeled rows based on similarity
fn assign_labels_by_similarity(
    unlabeled_vectors: &[Vec<f64>],
    labeled_vectors: &[Vec<f64>],
    labeled_labels: &[String],
) -> Vec<String> {
    unlabeled_vectors
        .iter()
        .map(|unlabeled_vec| {
            // Find the labeled row with highest cosine similarity
            let (best_idx, _best_similarity) = labeled_vectors
                .iter()
                .enumerate()
                .map(|(idx, labeled_vec)| {
                    let similarity = cosine_similarity(unlabeled_vec, labeled_vec);
                    (idx, similarity)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or((0, 0.0));
            
            // Return the label from the most similar labeled row
            labeled_labels[best_idx].clone()
        })
        .collect()
}

/// Impute missing labels using TF-IDF similarity
///
/// This function implements TF-IDF-based label deduction for missing values in a categorical column.
/// It uses text similarity from specified columns to predict labels for unlabeled rows.
///
/// # Algorithm
/// 1. Validate at least 3 labeled examples exist
/// 2. Separate labeled and unlabeled rows
/// 3. Combine text from `against` columns
/// 4. Vectorize using TF-IDF
/// 5. Compute cosine similarity between unlabeled and labeled rows
/// 6. Assign labels from most similar labeled rows
/// 7. Return Series with deduced labels (preserving original order)
///
/// # Arguments
/// * `df` - Reference to the DataFrame containing the data
/// * `infer_col` - Name of the column with labels to impute (contains null values)
/// * `against_cols` - Vector of column names containing text for similarity calculation
///
/// # Returns
/// Series with imputed labels, maintaining the original DataFrame order
///
/// # Errors
/// - Validation error if fewer than 3 labeled examples exist
/// - Column not found if infer_col or any against_col doesn't exist
/// - Operation error if TF-IDF vectorization or similarity calculation fails
fn impute_tfidf_series(
    df: &DataFrame,
    infer_col: &str,
    against_cols: &[String],
) -> AdditoryResult<Series> {
    // Step 1: Validate parameters (at least 3 labeled examples)
    validate_tfidf_params(df, infer_col, against_cols)?;
    
    // Step 2: Separate labeled and unlabeled rows
    let (labeled_df, unlabeled_df) = separate_labeled_unlabeled(df, infer_col)?;
    
    // Step 3: Combine text from against columns for both labeled and unlabeled
    let labeled_texts = combine_text_columns(&labeled_df, against_cols)?;
    let unlabeled_texts = combine_text_columns(&unlabeled_df, against_cols)?;
    
    // Step 4: Vectorize using TF-IDF
    // Create vectorizer and fit on labeled texts
    let mut vectorizer = TfidfVectorizer::new();
    let labeled_vectors = vectorizer.fit_transform(&labeled_texts);
    
    // Transform unlabeled texts using the fitted vectorizer
    let unlabeled_vectors = vectorizer.transform(&unlabeled_texts);
    
    // Step 5 & 6: Extract labels from labeled rows
    let labeled_col = labeled_df.column(infer_col)?;
    let labeled_str_col = labeled_col.str().map_err(|e| {
        AdditoryError::operation(
            "Failed to extract labels from labeled rows",
            &format!("Column '{}' must be a string column: {}", infer_col, e),
        )
    })?;
    
    let labeled_labels: Vec<String> = labeled_str_col
        .into_iter()
        .map(|opt_str| opt_str.unwrap_or("").to_string())
        .collect();
    
    // Step 7: Assign labels based on cosine similarity
    let deduced_labels = assign_labels_by_similarity(
        &unlabeled_vectors,
        &labeled_vectors,
        &labeled_labels,
    );
    
    // Step 8: Create result Series maintaining original DataFrame order
    // We need to reconstruct the full series with labeled and unlabeled values
    let polars_df = df.inner();
    let target_col = polars_df.column(infer_col)?;
    let is_null = target_col.is_null();
    
    let mut result_values: Vec<Option<String>> = Vec::with_capacity(df.height());
    let mut unlabeled_idx = 0;
    let mut labeled_idx = 0;
    
    // Iterate through original DataFrame and fill in values
    for i in 0..df.height() {
        if is_null.get(i).unwrap_or(false) {
            // This row was unlabeled, use deduced label
            result_values.push(Some(deduced_labels[unlabeled_idx].clone()));
            unlabeled_idx += 1;
        } else {
            // This row was labeled, use original label
            result_values.push(Some(labeled_labels[labeled_idx].clone()));
            labeled_idx += 1;
        }
    }
    
    // Create Series from result values
    let result_series = Series::new(infer_col.into(), result_values);
    
    Ok(result_series)
}

/// Mean imputation: Fill with column mean (numeric only) - returns Series
fn impute_mean_series(df: &DataFrame, col_name: &str) -> AdditoryResult<Series> {
    let inner_df = df.inner();
    
    // Check if column exists
    if inner_df.column(col_name).is_err() {
        let col_names: Vec<String> = inner_df.get_column_names().iter().map(|s| s.to_string()).collect();
        return Err(AdditoryError::Validation(
            format!("Column '{}' not found in DataFrame", col_name),
            format!("Available columns: {:?}", col_names),
        ));
    }
    
    // Use LazyFrame for easier expression-based operations
    let lazy_df = inner_df.clone().lazy();
    
    // Use Polars expression to fill nulls with mean
    let result_df = lazy_df
        .select([col(col_name).fill_null(col(col_name).mean())])
        .collect()
        .map_err(|e| AdditoryError::operation(
            "Failed to impute with mean",
            &e.to_string()
        ))?;
    
    Ok(result_df.column(col_name)?.as_materialized_series().clone())
}

/// Median imputation: Fill with column median (numeric only) - returns Series
fn impute_median_series(df: &DataFrame, col_name: &str) -> AdditoryResult<Series> {
    let inner_df = df.inner();
    
    // Check if column exists
    if inner_df.column(col_name).is_err() {
        let col_names: Vec<String> = inner_df.get_column_names().iter().map(|s| s.to_string()).collect();
        return Err(AdditoryError::Validation(
            format!("Column '{}' not found in DataFrame", col_name),
            format!("Available columns: {:?}", col_names),
        ));
    }
    
    // Use LazyFrame for easier expression-based operations
    let lazy_df = inner_df.clone().lazy();
    
    // Use Polars expression to fill nulls with median
    let result_df = lazy_df
        .select([col(col_name).fill_null(col(col_name).median())])
        .collect()
        .map_err(|e| AdditoryError::operation(
            "Failed to impute with median",
            &e.to_string()
        ))?;
    
    Ok(result_df.column(col_name)?.as_materialized_series().clone())
}

/// Mode imputation: Fill with most frequent value - returns Series
/// Note: For simplicity, we use forward fill as a proxy for mode imputation
/// A full mode implementation would require calculating the mode value and filling with it
fn impute_mode_series(df: &DataFrame, col_name: &str) -> AdditoryResult<Series> {
    // For now, use forward fill as a simple approximation
    // TODO: Implement proper mode calculation and filling
    impute_forward_series(df, col_name)
}

/// Forward fill: Propagate last valid value forward - returns Series
fn impute_forward_series(df: &DataFrame, col_name: &str) -> AdditoryResult<Series> {
    let inner_df = df.inner();
    
    // Check if column exists
    if inner_df.column(col_name).is_err() {
        let col_names: Vec<String> = inner_df.get_column_names().iter().map(|s| s.to_string()).collect();
        return Err(AdditoryError::Validation(
            format!("Column '{}' not found in DataFrame", col_name),
            format!("Available columns: {:?}", col_names),
        ));
    }
    
    // Use LazyFrame for easier expression-based operations
    let lazy_df = inner_df.clone().lazy();
    
    // Use Polars expression for forward fill
    let result_df = lazy_df
        .select([col(col_name).forward_fill(None)])
        .collect()
        .map_err(|e| AdditoryError::operation(
            "Failed to forward fill",
            &e.to_string()
        ))?;
    
    Ok(result_df.column(col_name)?.as_materialized_series().clone())
}

/// Backward fill: Propagate next valid value backward - returns Series
fn impute_backward_series(df: &DataFrame, col_name: &str) -> AdditoryResult<Series> {
    let inner_df = df.inner();
    
    // Check if column exists
    if inner_df.column(col_name).is_err() {
        let col_names: Vec<String> = inner_df.get_column_names().iter().map(|s| s.to_string()).collect();
        return Err(AdditoryError::Validation(
            format!("Column '{}' not found in DataFrame", col_name),
            format!("Available columns: {:?}", col_names),
        ));
    }
    
    // Use LazyFrame for easier expression-based operations
    let lazy_df = inner_df.clone().lazy();
    
    // Use Polars expression for backward fill
    let result_df = lazy_df
        .select([col(col_name).backward_fill(None)])
        .collect()
        .map_err(|e| AdditoryError::operation(
            "Failed to backward fill",
            &e.to_string()
        ))?;
    
    Ok(result_df.column(col_name)?.as_materialized_series().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    #[test]
    fn test_imputation_method_from_str() {
        assert_eq!(ImputationMethod::parse_method("auto").unwrap(), ImputationMethod::Auto);
        assert_eq!(ImputationMethod::parse_method("mean").unwrap(), ImputationMethod::Mean);
        assert_eq!(ImputationMethod::parse_method("median").unwrap(), ImputationMethod::Median);
        assert_eq!(ImputationMethod::parse_method("mode").unwrap(), ImputationMethod::Mode);
        assert_eq!(ImputationMethod::parse_method("forward").unwrap(), ImputationMethod::Forward);
        assert_eq!(ImputationMethod::parse_method("ffill").unwrap(), ImputationMethod::Forward);
        assert_eq!(ImputationMethod::parse_method("backward").unwrap(), ImputationMethod::Backward);
        assert_eq!(ImputationMethod::parse_method("bfill").unwrap(), ImputationMethod::Backward);
        assert_eq!(ImputationMethod::parse_method("knn").unwrap(), ImputationMethod::Knn);
        assert!(ImputationMethod::parse_method("invalid").is_err());
    }

    #[test]
    fn test_forward_fill_series() {
        let df = df! {
            "a" => &[Some(1), None, Some(3), None, Some(5)],
            "b" => &[Some(10), Some(20), None, None, Some(50)]
        }.unwrap();
        
        let result_series = impute_forward_series(&crate::core::DataFrame::from_polars(df.clone()), "a").unwrap();
        
        // Check that nulls are filled with previous values
        assert_eq!(result_series.null_count(), 0);
    }

    #[test]
    fn test_backward_fill_series() {
        let df = df! {
            "a" => &[Some(1), None, Some(3), None, Some(5)],
            "b" => &[Some(10), Some(20), None, None, Some(50)]
        }.unwrap();
        
        let result_series = impute_backward_series(&crate::core::DataFrame::from_polars(df.clone()), "a").unwrap();
        
        // Check that nulls are filled with next values
        assert_eq!(result_series.null_count(), 0);
    }

    #[test]
    fn test_mean_imputation_series() {
        let df = df! {
            "a" => &[Some(1.0), None, Some(3.0), None, Some(5.0)]
        }.unwrap();
        
        let result_series = impute_mean_series(&crate::core::DataFrame::from_polars(df.clone()), "a").unwrap();
        
        // Check that nulls are filled
        assert_eq!(result_series.null_count(), 0);
    }

    #[test]
    fn test_median_imputation_series() {
        let df = df! {
            "a" => &[Some(1.0), None, Some(3.0), None, Some(5.0)]
        }.unwrap();
        
        let result_series = impute_median_series(&crate::core::DataFrame::from_polars(df.clone()), "a").unwrap();
        
        // Check that nulls are filled
        assert_eq!(result_series.null_count(), 0);
    }
    
    #[test]
    fn test_column_preservation() {
        let df = df! {
            "a" => &[Some(1.0), None, Some(3.0), None, Some(5.0)]
        }.unwrap();
        
        let input_df = crate::core::DataFrame::from_polars(df.clone());
        let result = deduce(
            input_df.clone(),
            vec!["a".to_string()],
            vec!["a_filled".to_string()],
            vec!["mean".to_string()],
            None,
            None
        ).unwrap();
        
        // Check that original column is preserved
        assert!(result.has_column("a"));
        assert!(result.has_column("a_filled"));
        
        // Check that original column still has nulls
        let original_col = result.column("a").unwrap();
        assert!(original_col.null_count() > 0);
        
        // Check that new column has no nulls
        let filled_col = result.column("a_filled").unwrap();
        assert_eq!(filled_col.null_count(), 0);
    }

    // ========== TF-IDF Tests ==========

    /// Helper function to create a test DataFrame for TF-IDF tests
    fn create_tfidf_test_df(
        texts: Vec<&str>,
        labels: Vec<Option<&str>>,
    ) -> crate::core::DataFrame {
        let text_series = Column::new("text".into(), texts);
        let label_series = Column::new("label".into(), labels);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text_series, label_series]).unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_validate_tfidf_params_success() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3", "text4"],
            vec![Some("A"), Some("B"), Some("A"), None],
        );
        
        // Should succeed: 3 labeled examples
        let result = validate_tfidf_params(&df, "label", &vec!["text".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tfidf_params_insufficient_labeled() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3"],
            vec![Some("A"), Some("B"), None],
        );
        
        // Should fail: only 2 labeled examples
        let result = validate_tfidf_params(&df, "label", &vec!["text".to_string()]);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("Insufficient labeled examples"));
        assert!(err_msg.contains("found 2"));
        assert!(err_msg.contains("minimum 3 required"));
    }

    #[test]
    fn test_validate_tfidf_params_target_column_not_found() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3"],
            vec![Some("A"), Some("B"), Some("C")],
        );
        
        // Should fail: target column doesn't exist
        let result = validate_tfidf_params(&df, "nonexistent", &vec!["text".to_string()]);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("nonexistent"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_validate_tfidf_params_text_column_not_found() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3"],
            vec![Some("A"), Some("B"), Some("C")],
        );
        
        // Should fail: text column doesn't exist
        let result = validate_tfidf_params(&df, "label", &vec!["nonexistent".to_string()]);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("nonexistent"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_validate_tfidf_params_exactly_min_labeled() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3", "text4"],
            vec![Some("A"), Some("B"), Some("C"), None],
        );
        
        // Should succeed: exactly 3 labeled examples (minimum)
        let result = validate_tfidf_params(&df, "label", &vec!["text".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tfidf_params_all_labeled() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3", "text4"],
            vec![Some("A"), Some("B"), Some("C"), Some("A")],
        );
        
        // Should succeed: all rows labeled (4 > 3)
        let result = validate_tfidf_params(&df, "label", &vec!["text".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tfidf_params_multiple_text_columns() {
        // Create DataFrame with multiple text columns
        let text1_series = Column::new("text1".into(), vec!["a", "b", "c", "d"]);
        let text2_series = Column::new("text2".into(), vec!["x", "y", "z", "w"]);
        let label_series = Column::new("label".into(), vec![Some("A"), Some("B"), Some("C"), None]);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text1_series, text2_series, label_series]).unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        // Should succeed: both text columns exist and 3 labeled examples
        let result = validate_tfidf_params(&df, "label", &vec!["text1".to_string(), "text2".to_string()]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_tfidf_params_one_text_column_missing() {
        let text1_series = Column::new("text1".into(), vec!["a", "b", "c", "d"]);
        let label_series = Column::new("label".into(), vec![Some("A"), Some("B"), Some("C"), None]);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text1_series, label_series]).unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        // Should fail: text2 column doesn't exist
        let result = validate_tfidf_params(&df, "label", &vec!["text1".to_string(), "text2".to_string()]);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("text2"));
        assert!(err_msg.contains("not found"));
    }

    #[test]
    fn test_separate_labeled_unlabeled_basic() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3", "text4", "text5"],
            vec![Some("A"), Some("B"), None, Some("C"), None],
        );
        
        let (labeled, unlabeled) = separate_labeled_unlabeled(&df, "label").unwrap();
        
        assert_eq!(labeled.height(), 3); // 3 labeled rows
        assert_eq!(unlabeled.height(), 2); // 2 unlabeled rows
    }

    #[test]
    fn test_separate_labeled_unlabeled_all_labeled() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3"],
            vec![Some("A"), Some("B"), Some("C")],
        );
        
        let (labeled, unlabeled) = separate_labeled_unlabeled(&df, "label").unwrap();
        
        assert_eq!(labeled.height(), 3); // All labeled
        assert_eq!(unlabeled.height(), 0); // No unlabeled
    }

    #[test]
    fn test_combine_text_columns_single_column() {
        let df = create_tfidf_test_df(
            vec!["hello world", "foo bar", "test text"],
            vec![Some("A"), Some("B"), Some("C")],
        );
        
        let combined = combine_text_columns(&df, &vec!["text".to_string()]).unwrap();
        
        assert_eq!(combined.len(), 3);
        assert_eq!(combined[0], "hello world");
        assert_eq!(combined[1], "foo bar");
        assert_eq!(combined[2], "test text");
    }

    #[test]
    fn test_combine_text_columns_multiple_columns() {
        // Create DataFrame with multiple text columns
        let text1_series = Column::new("text1".into(), vec!["hello", "foo", "test"]);
        let text2_series = Column::new("text2".into(), vec!["world", "bar", "text"]);
        let label_series = Column::new("label".into(), vec![Some("A"), Some("B"), Some("C")]);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text1_series, text2_series, label_series]).unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let combined = combine_text_columns(&df, &vec!["text1".to_string(), "text2".to_string()]).unwrap();
        
        assert_eq!(combined.len(), 3);
        assert_eq!(combined[0], "hello world");
        assert_eq!(combined[1], "foo bar");
        assert_eq!(combined[2], "test text");
    }

    #[test]
    fn test_impute_tfidf_series_basic() {
        // Create DataFrame with labeled and unlabeled rows
        let df = create_tfidf_test_df(
            vec![
                "login issue",
                "password reset",
                "app crash",
                "billing question",
                "login error", // Similar to "login issue"
            ],
            vec![
                Some("Technical"),
                Some("Account"),
                Some("Technical"),
                Some("Billing"),
                None, // Should be labeled based on similarity
            ],
        );
        
        let result = impute_tfidf_series(&df, "label", &vec!["text".to_string()]).unwrap();
        
        // Should have same number of rows
        assert_eq!(result.len(), 5);
        
        // All rows should be labeled (no nulls)
        assert_eq!(result.null_count(), 0);
    }

    #[test]
    fn test_impute_tfidf_series_all_labeled() {
        // All rows already labeled - should return all labels
        let df = create_tfidf_test_df(
            vec!["login issue", "password reset", "app crash"],
            vec![Some("Technical"), Some("Account"), Some("Technical")],
        );
        
        let result = impute_tfidf_series(&df, "label", &vec!["text".to_string()]).unwrap();
        
        // Should have same number of rows
        assert_eq!(result.len(), 3);
        
        // All rows should still be labeled
        assert_eq!(result.null_count(), 0);
    }

    #[test]
    fn test_impute_tfidf_series_multiple_text_columns() {
        // Create DataFrame with multiple text columns
        let text1_series = Column::new("title".into(), vec!["Login", "Password", "App", "Login"]);
        let text2_series = Column::new("description".into(), vec!["issue", "reset", "crash", "error"]);
        let label_series = Column::new("category".into(), vec![Some("Tech"), Some("Account"), Some("Tech"), None]);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text1_series, text2_series, label_series]).unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let result = impute_tfidf_series(&df, "category", &vec!["title".to_string(), "description".to_string()]).unwrap();
        
        // Should have same number of rows
        assert_eq!(result.len(), 4);
        
        // All rows should be labeled (no nulls)
        assert_eq!(result.null_count(), 0);
    }

    #[test]
    fn test_impute_tfidf_series_insufficient_labeled() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3"],
            vec![Some("A"), Some("B"), None],
        );
        
        // Should fail: only 2 labeled examples
        let result = impute_tfidf_series(&df, "label", &vec!["text".to_string()]);
        assert!(result.is_err());
        
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("Insufficient labeled examples"));
    }

    #[test]
    fn test_deduce_tfidf_integration() {
        // Integration test using the deduce() function with TF-IDF method
        let df = create_tfidf_test_df(
            vec![
                "login issue",
                "password reset",
                "app crash",
                "login error", // Similar to "login issue"
            ],
            vec![
                Some("Technical"),
                Some("Account"),
                Some("Technical"),
                None,
            ],
        );
        
        let result = deduce(
            df,
            vec!["label".to_string()],
            vec!["label_deduced".to_string()],
            vec!["tfidf".to_string()],
            Some(vec!["text".to_string()]),
            None,
        ).unwrap();
        
        // Check that original column is preserved
        assert!(result.has_column("label"));
        assert!(result.has_column("label_deduced"));
        
        // Check that original column still has nulls
        let original_col = result.column("label").unwrap();
        assert!(original_col.null_count() > 0);
        
        // Check that new column has no nulls
        let deduced_col = result.column("label_deduced").unwrap();
        assert_eq!(deduced_col.null_count(), 0);
    }

    #[test]
    fn test_deduce_tfidf_missing_against_parameter() {
        let df = create_tfidf_test_df(
            vec!["text1", "text2", "text3", "text4"],
            vec![Some("A"), Some("B"), Some("C"), None],
        );
        
        // Should fail: against parameter is required for TF-IDF
        let result = deduce(
            df,
            vec!["label".to_string()],
            vec!["label_deduced".to_string()],
            vec!["tfidf".to_string()],
            None, // Missing against parameter
            None,
        );
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("against"));
        assert!(err_msg.contains("required"));
    }

    #[test]
    fn test_deduce_tfidf_multiple_columns() {
        // Test TF-IDF with multiple text columns
        let text1_series = Column::new("title".into(), vec!["Login", "Password", "App", "Login"]);
        let text2_series = Column::new("description".into(), vec!["issue", "reset", "crash", "error"]);
        let label_series = Column::new("category".into(), vec![Some("Tech"), Some("Account"), Some("Tech"), None]);
        
        let polars_df = polars::prelude::DataFrame::new(vec![text1_series, text2_series, label_series]).unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let result = deduce(
            df,
            vec!["category".to_string()],
            vec!["category_deduced".to_string()],
            vec!["tfidf".to_string()],
            Some(vec!["title".to_string(), "description".to_string()]),
            None,
        ).unwrap();
        
        // Check that both columns exist
        assert!(result.has_column("category"));
        assert!(result.has_column("category_deduced"));
        
        // Check that new column has no nulls
        let deduced_col = result.column("category_deduced").unwrap();
        assert_eq!(deduced_col.null_count(), 0);
    }

    #[test]
    fn test_assign_labels_by_similarity() {
        // Test the similarity assignment logic
        let unlabeled_vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        
        let labeled_vectors = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ];
        
        let labeled_labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        
        let result = assign_labels_by_similarity(&unlabeled_vectors, &labeled_vectors, &labeled_labels);
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "A"); // First unlabeled vector matches first labeled vector
        assert_eq!(result[1], "B"); // Second unlabeled vector matches second labeled vector
    }
}
