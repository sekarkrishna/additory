//! @split mode - Split text column into multiple columns
//!
//! Implements splitting a text column based on a separator.

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::utils::{Validator, Logger};
use polars::prelude::*;

/// Parameters for @split operation
pub struct SplitParams {
    pub column: String,
    pub separator: String,
    pub new_columns: Vec<String>,
    pub logging: bool,
}

/// Execute @split mode - split text column into multiple columns
pub fn execute(df: DataFrame, params: SplitParams) -> AdditoryResult<DataFrame> {
    let logger = Logger::new(params.logging);
    
    logger.log_start("add.transform", "@split");
    logger.log_dataframe("add.transform", "Input", df.height(), df.width());
    
    // Validate parameters
    Validator::validate_not_empty(&df, "@split")?;
    
    if params.new_columns.is_empty() {
        return Err(AdditoryError::missing_parameter(
            "new_columns",
            "At least one new column name must be specified"
        ));
    }
    
    // Validate column exists
    if !df.has_column(&params.column) {
        return Err(AdditoryError::column_not_found(&params.column, &df.column_names()));
    }
    
    // Validate column is string type
    let col = df.column(&params.column)?;
    if !matches!(col.dtype(), DataType::String) {
        return Err(AdditoryError::invalid_parameter(
            "column",
            &params.column,
            &format!("Column must be of type String, found {:?}", col.dtype())
        ));
    }
    
    logger.log_param("add.transform", "column", &params.column);
    logger.log_param("add.transform", "separator", &params.separator);
    logger.log_param("add.transform", "new_columns", &format!("{:?}", params.new_columns));
    
    // Perform split
    let result = perform_split(df, &params.column, &params.separator, &params.new_columns)?;
    
    logger.log_result(
        "add.transform",
        &format!("Split into {} new columns", params.new_columns.len()),
    );
    
    Ok(result)
}

/// Perform the split operation
fn perform_split(
    df: DataFrame,
    column: &str,
    separator: &str,
    new_columns: &[String],
) -> AdditoryResult<DataFrame> {
    // Get the column as a Series
    let col_series = df.column(column)?;
    
    // Convert to ChunkedArray<Utf8>
    let ca = col_series.str().map_err(|e| AdditoryError::operation(&
        format!("Failed to convert column to string type: {}", e),
        "Column must be of string type"
    ))?;
    
    // Split each string and collect results
    let mut new_series_vec: Vec<Series> = Vec::new();
    
    for (i, new_col_name) in new_columns.iter().enumerate() {
        let mut values: Vec<Option<String>> = Vec::new();
        
        for opt_str in ca.into_iter() {
            if let Some(s) = opt_str {
                let parts: Vec<&str> = s.split(separator).collect();
                if i < parts.len() {
                    values.push(Some(parts[i].to_string()));
                } else {
                    values.push(None);
                }
            } else {
                values.push(None);
            }
        }
        
        let series = Series::new(new_col_name.as_str().into(), values);
        new_series_vec.push(series);
    }
    
    // Add new columns to the dataframe
    let mut result_polars = df.inner().clone();
    for series in new_series_vec {
        result_polars = result_polars.with_column(series.clone()).map_err(|e| AdditoryError::operation(&
            format!("Failed to add split column: {}", e),
            "Error adding new columns to dataframe"
        ))?.clone();
    }
    
    Ok(DataFrame::new(result_polars, df.original_type()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_df() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice Smith", "Bob Jones", "Charlie Brown"],
            "email" => &["alice@example.com", "bob@test.org", "charlie@demo.net"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_split_name() {
        let df = create_test_df();
        
        let params = SplitParams {
            column: "name".to_string(),
            separator: " ".to_string(),
            new_columns: vec!["first".to_string(), "last".to_string()],
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        // Should have original columns plus 2 new columns
        assert!(result.has_column("first"));
        assert!(result.has_column("last"));
        
        // Check values
        let first_col = result.column("first").unwrap();
        let first_vals: Vec<Option<&str>> = first_col.str().unwrap().into_iter().collect();
        assert_eq!(first_vals[0], Some("Alice"));
        assert_eq!(first_vals[1], Some("Bob"));
        assert_eq!(first_vals[2], Some("Charlie"));
    }

    #[test]
    fn test_split_email() {
        let df = create_test_df();
        
        let params = SplitParams {
            column: "email".to_string(),
            separator: "@".to_string(),
            new_columns: vec!["username".to_string(), "domain".to_string()],
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("username"));
        assert!(result.has_column("domain"));
        
        let username_col = result.column("username").unwrap();
        let usernames: Vec<Option<&str>> = username_col.str().unwrap().into_iter().collect();
        assert_eq!(usernames[0], Some("alice"));
        assert_eq!(usernames[1], Some("bob"));
    }

    #[test]
    fn test_split_single_column() {
        let df = create_test_df();
        
        let params = SplitParams {
            column: "email".to_string(),
            separator: "@".to_string(),
            new_columns: vec!["username".to_string()],
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("username"));
        assert!(!result.has_column("domain"));
    }

    #[test]
    fn test_split_three_parts() {
        let polars_df = df! {
            "path" => &["a/b/c", "x/y/z", "1/2/3"],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = SplitParams {
            column: "path".to_string(),
            separator: "/".to_string(),
            new_columns: vec!["part1".to_string(), "part2".to_string(), "part3".to_string()],
            logging: false,
        };
        
        let result = execute(df, params).unwrap();
        
        assert!(result.has_column("part1"));
        assert!(result.has_column("part2"));
        assert!(result.has_column("part3"));
        
        let part1_col = result.column("part1").unwrap();
        let part1_vals: Vec<Option<&str>> = part1_col.str().unwrap().into_iter().collect();
        assert_eq!(part1_vals[0], Some("a"));
        assert_eq!(part1_vals[1], Some("x"));
    }

    #[test]
    fn test_split_missing_column() {
        let df = create_test_df();
        
        let params = SplitParams {
            column: "nonexistent".to_string(),
            separator: " ".to_string(),
            new_columns: vec!["first".to_string(), "last".to_string()],
            logging: false,
        };
        
        let result = execute(df, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_non_string_column() {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[100, 200, 300],
        }
        .unwrap();
        let df = crate::core::DataFrame::from_polars(polars_df);
        
        let params = SplitParams {
            column: "value".to_string(),
            separator: " ".to_string(),
            new_columns: vec!["part1".to_string()],
            logging: false,
        };
        
        let result = execute(df, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_empty_new_columns() {
        let df = create_test_df();
        
        let params = SplitParams {
            column: "name".to_string(),
            separator: " ".to_string(),
            new_columns: vec![],
            logging: false,
        };
        
        let result = execute(df, params);
        assert!(result.is_err());
    }
}
