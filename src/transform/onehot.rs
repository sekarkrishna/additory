//! @onehot mode - One-hot encoding
//!
//! Implements one-hot encoding for categorical columns.
//! Converts a categorical column into multiple binary columns.

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::utils::{Validator, Logger};
use polars::prelude::*;
use std::collections::HashSet;

/// Parameters for @onehot operation
pub struct OneHotParams {
    pub column: String,
    pub prefix: Option<String>,
    pub logging: bool,
}

/// Execute @onehot mode - one-hot encode categorical column
pub fn execute(df: DataFrame, params: OneHotParams) -> AdditoryResult<DataFrame> {
    let logger = Logger::new(params.logging);
    
    logger.log_start("add.transform", "@onehot");
    logger.log_dataframe("add.transform", "Input", df.height(), df.width());
    
    // Validate parameters
    Validator::validate_not_empty(&df, "@onehot")?;
    
    // Validate column exists
    if !df.has_column(&params.column) {
        return Err(AdditoryError::column_not_found(&params.column, &df.column_names()));
    }
    
    logger.log_param("add.transform", "column", &params.column);
    
    // Determine prefix
    let prefix = params.prefix.unwrap_or_else(|| format!("{}_", params.column));
    logger.log_param("add.transform", "prefix", &prefix);
    
    // Get original width before transformation
    let original_width = df.width();
    
    // Perform one-hot encoding
    let result = perform_onehot(df, &params.column, &prefix)?;
    
    let num_categories = result.width() - original_width;
    logger.log_result(
        "add.transform",
        &format!("Created {} binary columns", num_categories),
    );
    
    Ok(result)
}

/// Perform one-hot encoding
fn perform_onehot(
    df: DataFrame,
    column: &str,
    prefix: &str,
) -> AdditoryResult<DataFrame> {
    // Get unique values from the column
    let col_ref = df.column(column)?;
    let col_series = col_ref.as_materialized_series().clone();
    
    // Get unique values (handle both string and numeric types)
    let unique_values = get_unique_values(&col_series)?;
    
    if unique_values.is_empty() {
        return Err(AdditoryError::OperationFailed(
            "No unique values found in column".to_string(),
            "Column may be empty or contain only null values".to_string()
        ));
    }
    
    // Create binary columns for each unique value
    let mut exprs = Vec::new();
    
    for value in &unique_values {
        let col_name = format!("{}{}", prefix, value);
        
        // Create expression: column == value ? 1 : 0
        // Parse value back to appropriate type for comparison
        let expr = if let Ok(int_val) = value.parse::<i64>() {
            when(col(column).eq(lit(int_val)))
                .then(lit(1))
                .otherwise(lit(0))
                .alias(&col_name)
        } else if let Ok(float_val) = value.parse::<f64>() {
            when(col(column).eq(lit(float_val)))
                .then(lit(1))
                .otherwise(lit(0))
                .alias(&col_name)
        } else {
            when(col(column).eq(lit(value.clone())))
                .then(lit(1))
                .otherwise(lit(0))
                .alias(&col_name)
        };
        
        exprs.push(expr);
    }
    
    // Apply all one-hot encodings at once
    let result_polars = df.inner()
        .clone()
        .lazy()
        .with_columns(exprs)
        .collect()
        .map_err(|e| AdditoryError::OperationFailed(
            format!("Failed to create one-hot encoding: {}", e),
            "Check that the column contains valid categorical data".to_string()
        ))?;
    
    Ok(DataFrame::new(result_polars, df.original_type()))
}

/// Get unique values from a series (handles string and numeric types)
fn get_unique_values(series: &Series) -> AdditoryResult<Vec<String>> {
    let mut unique_set = HashSet::new();
    
    match series.dtype() {
        DataType::String => {
            let ca = series.str().map_err(|e| AdditoryError::OperationFailed(
                format!("Failed to convert to string: {}", e),
                "Column must be string or numeric type".to_string()
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    unique_set.insert(val.to_string());
                }
            }
        }
        DataType::Int32 => {
            let ca = series.i32().map_err(|e| AdditoryError::OperationFailed(
                format!("Failed to convert to i32: {}", e),
                "Column must be string or numeric type".to_string()
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    unique_set.insert(val.to_string());
                }
            }
        }
        DataType::Int8 | DataType::Int16 | DataType::Int64 => {
            let ca = series.i64().map_err(|e| AdditoryError::OperationFailed(
                format!("Failed to convert to integer: {}", e),
                "Column must be string or numeric type".to_string()
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    unique_set.insert(val.to_string());
                }
            }
        }
        DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 => {
            let ca = series.u64().map_err(|e| AdditoryError::OperationFailed(
                format!("Failed to convert to unsigned integer: {}", e),
                "Column must be string or numeric type".to_string()
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    unique_set.insert(val.to_string());
                }
            }
        }
        DataType::Float32 | DataType::Float64 => {
            let ca = series.f64().map_err(|e| AdditoryError::OperationFailed(
                format!("Failed to convert to float: {}", e),
                "Column must be string or numeric type".to_string()
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    unique_set.insert(val.to_string());
                }
            }
        }
        _ => {
            return Err(AdditoryError::invalid_parameter(
                "column",
                &series.name(),
                &format!("Unsupported column type for one-hot encoding: {:?}. Use string or numeric types", series.dtype())
            ));
        }
    }
    
    let mut unique_vec: Vec<String> = unique_set.into_iter().collect();
    unique_vec.sort(); // Sort for consistent ordering
    
    Ok(unique_vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_df() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3, 4, 5],
            "category" => &["A", "B", "A", "C", "B"],
            "status" => &["active", "pending", "active", "closed", "pending"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_onehot_basic() {
        let df = create_test_df();
        
        let params = OneHotParams {
            column: "category".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Should have 3 new columns (A, B, C)
        assert!(result.has_column("category_A"));
        assert!(result.has_column("category_B"));
        assert!(result.has_column("category_C"));
        
        // Check values for category_A
        let col_a = result.column("category_A").unwrap();
        let values_a: Vec<i32> = col_a.i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(values_a, vec![1, 0, 1, 0, 0]);
        
        // Check values for category_B
        let col_b = result.column("category_B").unwrap();
        let values_b: Vec<i32> = col_b.i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(values_b, vec![0, 1, 0, 0, 1]);
    }

    #[test]
    fn test_onehot_with_custom_prefix() {
        let df = create_test_df();
        
        let params = OneHotParams {
            column: "status".to_string(),
            prefix: Some("is_".to_string()),
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("is_active"));
        assert!(result.has_column("is_pending"));
        assert!(result.has_column("is_closed"));
        assert!(!result.has_column("status_active"));
    }

    #[test]
    fn test_onehot_numeric_column() {
        let polars_df = df! {
            "id" => &[1, 2, 3, 4, 5],
            "grade" => &[1, 2, 1, 3, 2],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = OneHotParams {
            column: "grade".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("grade_1"));
        assert!(result.has_column("grade_2"));
        assert!(result.has_column("grade_3"));
    }

    #[test]
    fn test_onehot_single_value() {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "constant" => &["A", "A", "A"],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = OneHotParams {
            column: "constant".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Should have 1 new column
        assert!(result.has_column("constant_A"));
        
        let col_a = result.column("constant_A").unwrap();
        let values: Vec<i32> = col_a.i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(values, vec![1, 1, 1]);
    }

    #[test]
    fn test_onehot_missing_column() {
        let df = create_test_df();
        
        let params = OneHotParams {
            column: "nonexistent".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_onehot_preserves_original_column() {
        let df = create_test_df();
        
        let params = OneHotParams {
            column: "category".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Original column should still exist
        assert!(result.has_column("category"));
    }

    #[test]
    fn test_onehot_multiple_categories() {
        let polars_df = df! {
            "id" => &[1, 2, 3, 4, 5, 6],
            "color" => &["red", "blue", "green", "red", "yellow", "blue"],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = OneHotParams {
            column: "color".to_string(),
            prefix: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Should have 4 new columns
        assert!(result.has_column("color_red"));
        assert!(result.has_column("color_blue"));
        assert!(result.has_column("color_green"));
        assert!(result.has_column("color_yellow"));
    }
}
