//! @label mode - Label encoding
//!
//! Implements label encoding for categorical columns.
//! Converts categorical values to numeric codes (0, 1, 2, ...).

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::utils::{Validator, Logger};
use polars::prelude::*;
use std::collections::HashMap;

/// Parameters for @label operation
pub struct LabelParams {
    pub column: String,
    pub new_column: Option<String>,
    pub logging: bool,
}

/// Execute @label mode - label encode categorical column
pub fn execute(df: DataFrame, params: LabelParams) -> AdditoryResult<DataFrame> {
    let logger = Logger::new(params.logging);
    
    logger.log_start("add.transform", "@label");
    logger.log_dataframe("add.transform", "Input", df.height(), df.width());
    
    // Validate parameters
    Validator::validate_not_empty(&df, "@label")?;
    
    // Validate column exists
    if !df.has_column(&params.column) {
        return Err(AdditoryError::column_not_found(&params.column, &df.column_names()));
    }
    
    logger.log_param("add.transform", "column", &params.column);
    
    // Determine new column name
    let new_col_name = params.new_column.unwrap_or_else(|| format!("{}_code", params.column));
    logger.log_param("add.transform", "new_column", &new_col_name);
    
    // Perform label encoding
    let result = perform_label_encoding(df, &params.column, &new_col_name)?;
    
    logger.log_result(
        "add.transform",
        &format!("Created label-encoded column '{}'", new_col_name),
    );
    
    Ok(result)
}

/// Perform label encoding
fn perform_label_encoding(
    df: DataFrame,
    column: &str,
    new_column: &str,
) -> AdditoryResult<DataFrame> {
    // Get the column
    let col_ref = df.column(column)?;
    let col_series = col_ref.as_materialized_series();
    
    // Get unique values and create mapping
    let (unique_values, mapping) = create_label_mapping(col_series)?;
    
    if unique_values.is_empty() {
        return Err(AdditoryError::operation(
            "No unique values found in column",
            "Column may be empty or contain only null values"
        ));
    }
    
    // Create the encoded column
    let encoded_series = encode_column(col_series, &mapping)?;
    
    // Add the new column to the dataframe
    let result_polars = df.inner()
        .clone()
        .with_column(encoded_series.with_name(new_column.into()))
        .map_err(|e| AdditoryError::operation(
            &format!("Failed to add encoded column: {}", e),
            "Error adding label-encoded column to dataframe"
        ))?
        .clone();
    
    Ok(DataFrame::new(result_polars, df.original_type()))
}

/// Create label mapping (value -> code)
fn create_label_mapping(series: &Series) -> AdditoryResult<(Vec<String>, HashMap<String, i32>)> {
    let mut unique_values = Vec::new();
    let mut mapping = HashMap::new();
    
    match series.dtype() {
        DataType::String => {
            let ca = series.str().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to string: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            let mut seen = std::collections::HashSet::new();
            for val in ca.into_iter().flatten() {
                if seen.insert(val.to_string()) {
                    unique_values.push(val.to_string());
                }
            }
        }
        DataType::Int32 => {
            let ca = series.i32().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to i32: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            let mut seen = std::collections::HashSet::new();
            for val in ca.into_iter().flatten() {
                if seen.insert(val) {
                    unique_values.push(val.to_string());
                }
            }
        }
        DataType::Int8 | DataType::Int16 | DataType::Int64 => {
            let ca = series.i64().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to integer: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            let mut seen = std::collections::HashSet::new();
            for val in ca.into_iter().flatten() {
                if seen.insert(val) {
                    unique_values.push(val.to_string());
                }
            }
        }
        _ => {
            return Err(AdditoryError::invalid_parameter(
                "column",
                series.name(),
                &format!("Unsupported column type for label encoding: {:?}. Use string or numeric types", series.dtype())
            ));
        }
    }
    
    // Sort for consistent ordering
    unique_values.sort();
    
    // Create mapping
    for (idx, value) in unique_values.iter().enumerate() {
        mapping.insert(value.to_string(), idx as i32);
    }
    
    Ok((unique_values, mapping))
}

/// Encode column values using mapping
fn encode_column(series: &Series, mapping: &HashMap<String, i32>) -> AdditoryResult<Series> {
    let mut encoded_values = Vec::with_capacity(series.len());
    
    match series.dtype() {
        DataType::String => {
            let ca = series.str().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to string: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    let code = mapping.get(val).copied().unwrap_or(-1);
                    encoded_values.push(Some(code));
                } else {
                    encoded_values.push(None);
                }
            }
        }
        DataType::Int32 => {
            let ca = series.i32().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to i32: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    let key = val.to_string();
                    let code = mapping.get(&key).copied().unwrap_or(-1);
                    encoded_values.push(Some(code));
                } else {
                    encoded_values.push(None);
                }
            }
        }
        DataType::Int8 | DataType::Int16 | DataType::Int64 => {
            let ca = series.i64().map_err(|e| AdditoryError::operation(&
                format!("Failed to convert to integer: {}", e),
                "Column must be string or numeric type"
            ))?;
            
            for opt_val in ca.into_iter() {
                if let Some(val) = opt_val {
                    let key = val.to_string();
                    let code = mapping.get(&key).copied().unwrap_or(-1);
                    encoded_values.push(Some(code));
                } else {
                    encoded_values.push(None);
                }
            }
        }
        _ => {
            return Err(AdditoryError::invalid_parameter(
                "column",
                series.name(),
                &format!("Unsupported column type: {:?}", series.dtype())
            ));
        }
    }
    
    Ok(Series::new("encoded".into(), encoded_values))
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
    fn test_label_basic() {
        let df = create_test_df();
        
        let params = LabelParams {
            column: "category".to_string(),
            new_column: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Should have new column category_code
        assert!(result.has_column("category_code"));
        
        // Check values (A=0, B=1, C=2 in sorted order)
        let code_col = result.column("category_code").unwrap();
        let codes: Vec<i32> = code_col.as_materialized_series().i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(codes, vec![0, 1, 0, 2, 1]);
    }

    #[test]
    fn test_label_with_custom_name() {
        let df = create_test_df();
        
        let params = LabelParams {
            column: "status".to_string(),
            new_column: Some("status_id".to_string()),
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("status_id"));
        assert!(!result.has_column("status_code"));
    }

    #[test]
    fn test_label_numeric_column() {
        let polars_df = df! {
            "id" => &[1, 2, 3, 4, 5],
            "grade" => &[1, 2, 1, 3, 2],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = LabelParams {
            column: "grade".to_string(),
            new_column: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("grade_code"));
        
        // Check values (1=0, 2=1, 3=2 in sorted order)
        let code_col = result.column("grade_code").unwrap();
        let codes: Vec<i32> = code_col.as_materialized_series().i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(codes, vec![0, 1, 0, 2, 1]);
    }

    #[test]
    fn test_label_preserves_original() {
        let df = create_test_df();
        
        let params = LabelParams {
            column: "category".to_string(),
            new_column: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Original column should still exist
        assert!(result.has_column("category"));
    }

    #[test]
    fn test_label_missing_column() {
        let df = create_test_df();
        
        let params = LabelParams {
            column: "nonexistent".to_string(),
            new_column: None,
            logging: false,
        };
        
        let result = execute(df, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_label_consistent_ordering() {
        let polars_df = df! {
            "id" => &[1, 2, 3, 4],
            "letter" => &["D", "A", "C", "B"],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = LabelParams {
            column: "letter".to_string(),
            new_column: None,
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Check values (A=0, B=1, C=2, D=3 in sorted order)
        let code_col = result.column("letter_code").unwrap();
        let codes: Vec<i32> = code_col.as_materialized_series().i32().unwrap().into_iter().map(|v| v.unwrap()).collect();
        assert_eq!(codes, vec![3, 0, 2, 1]);
    }
}
