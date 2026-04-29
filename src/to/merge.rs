//! @merge mode - Merge multiple DataFrames
//!
//! Implements merging of multiple DataFrames in three ways:
//! - Vertical: Stack rows (same columns)
//! - Horizontal: Join on key (add columns)
//! - Diagonal: Stack with different columns (fill NaN)

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::utils::{Validator, Logger};
use polars::prelude::*;

/// Merge type
#[derive(Debug, Clone, PartialEq)]
pub enum MergeType {
    Vertical,
    Horizontal,
    Diagonal,
}

impl MergeType {
    /// Parse merge type from string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "vertical" | "v" => Ok(MergeType::Vertical),
            "horizontal" | "h" => Ok(MergeType::Horizontal),
            "diagonal" | "d" => Ok(MergeType::Diagonal),
            _ => Err(format!("Unknown merge type: {}. Use 'vertical', 'horizontal', or 'diagonal'", s)),
        }
    }
}

/// Parameters for @merge operation
pub struct MergeParams {
    pub dataframes: Vec<DataFrame>,
    pub merge_type: MergeType,
    pub by: Option<String>,  // Required for horizontal merge
    pub logging: bool,
}

/// Execute @merge mode - merge multiple DataFrames
pub fn execute(params: MergeParams) -> AdditoryResult<DataFrame> {
    let logger = Logger::new(params.logging);
    
    logger.log_start("add.to", "@merge");
    logger.log_param("add.to", "merge_type", &format!("{:?}", params.merge_type));
    logger.log_param("add.to", "dataframes", &format!("{} DataFrames", params.dataframes.len()));
    
    // Validate parameters
    if params.dataframes.len() < 2 {
        return Err(AdditoryError::invalid_parameter(
            "dataframes",
            &format!("{} DataFrames", params.dataframes.len()),
            "At least 2 DataFrames are required for merging"
        ));
    }
    
    // Validate all DataFrames are not empty
    for (i, df) in params.dataframes.iter().enumerate() {
        Validator::validate_not_empty(df, &format!("DataFrame {}", i))?;
    }
    
    // Validate horizontal merge has 'by' parameter
    if params.merge_type == MergeType::Horizontal && params.by.is_none() {
        return Err(AdditoryError::missing_parameter(
            "by",
            "Horizontal merge requires a key column specified via 'by' parameter"
        ));
    }
    
    // Perform merge based on type
    let result = match params.merge_type {
        MergeType::Vertical => merge_vertical(&params.dataframes)?,
        MergeType::Horizontal => merge_horizontal(&params.dataframes, params.by.as_ref().unwrap())?,
        MergeType::Diagonal => merge_diagonal(&params.dataframes)?,
    };
    
    logger.log_result(
        "add.to",
        &format!("Merged {} DataFrames: {} rows × {} columns", 
                 params.dataframes.len(), result.height(), result.width()),
    );
    
    Ok(result)
}

/// Merge DataFrames vertically (stack rows)
fn merge_vertical(dataframes: &[DataFrame]) -> AdditoryResult<DataFrame> {
    // Validate all DataFrames have same columns
    let first_cols = dataframes[0].column_names();
    for (i, df) in dataframes.iter().enumerate().skip(1) {
        let cols = df.column_names();
        if cols != first_cols {
            return Err(AdditoryError::invalid_parameter(
                "dataframes",
                &format!("DataFrame {} has different columns", i),
                &format!("Vertical merge requires all DataFrames to have same columns. \
                         First: {:?}, DataFrame {}: {:?}", first_cols, i, cols)
            ));
        }
    }
    
    // Stack DataFrames using lazy evaluation for better performance
    let lazy_dfs: Vec<_> = dataframes.iter().map(|df| df.inner().clone().lazy()).collect();
    
    let result = polars::prelude::concat(
        lazy_dfs,
        UnionArgs {
            parallel: true,
            rechunk: false,
            to_supertypes: false,
            diagonal: false,
            from_partitioned_ds: false,
        }
    )
    .map_err(|e| AdditoryError::OperationFailed(
        format!("Failed to stack DataFrames vertically: {}", e),
        "Ensure all DataFrames have compatible column types".to_string()
    ))?
    .collect()
    .map_err(|e| AdditoryError::OperationFailed(
        format!("Failed to collect vertical merge result: {}", e),
        "Internal error during vertical merge".to_string()
    ))?;
    
    Ok(DataFrame::new(result, dataframes[0].original_type()))
}

/// Merge DataFrames horizontally (join on key)
fn merge_horizontal(dataframes: &[DataFrame], by: &str) -> AdditoryResult<DataFrame> {
    // Validate all DataFrames have the key column
    for (i, df) in dataframes.iter().enumerate() {
        if !df.has_column(by) {
            return Err(AdditoryError::column_not_found(
                &format!("{} (in DataFrame {})", by, i),
                &df.column_names()
            ));
        }
    }
    
    // Use lazy evaluation for better performance
    let mut result_lazy = dataframes[0].inner().clone().lazy();
    
    for df in dataframes.iter().skip(1) {
        result_lazy = result_lazy.join(
            df.inner().clone().lazy(),
            &[col(by)],
            &[col(by)],
            JoinArgs::new(JoinType::Left),
        );
    }
    
    let result = result_lazy.collect()
        .map_err(|e| AdditoryError::OperationFailed(
            format!("Failed to merge DataFrames horizontally: {}", e),
            "Check that join keys have compatible types".to_string()
        ))?;
    
    Ok(DataFrame::new(result, dataframes[0].original_type()))
}

/// Merge DataFrames diagonally (stack with different columns, fill NaN)
fn merge_diagonal(dataframes: &[DataFrame]) -> AdditoryResult<DataFrame> {
    // Convert to LazyFrames and use concat with diagonal option
    let lazy_dfs: Vec<_> = dataframes.iter().map(|df| df.inner().clone().lazy()).collect();
    
    // Concatenate with diagonal option
    let result = polars::prelude::concat(
        lazy_dfs,
        UnionArgs {
            parallel: true,
            rechunk: false,
            to_supertypes: false,
            diagonal: true,
            from_partitioned_ds: false,
        }
    )
    .map_err(|e| AdditoryError::OperationFailed(
        format!("Failed to merge DataFrames diagonally: {}", e),
        "Check that DataFrames have compatible types".to_string()
    ))?
    .collect()
    .map_err(|e| AdditoryError::OperationFailed(
        format!("Failed to collect diagonal merge result: {}", e),
        "Internal error during diagonal merge".to_string()
    ))?;
    
    Ok(DataFrame::new(result, dataframes[0].original_type()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_df1() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    fn create_df2() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[4, 5, 6],
            "name" => &["David", "Eve", "Frank"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    fn create_df3() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "age" => &[25, 30, 35],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    fn create_df4() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2],
            "email" => &["alice@example.com", "bob@example.com"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_merge_vertical() {
        let df1 = create_df1();
        let df2 = create_df2();
        
        let params = MergeParams {
            dataframes: vec![df1, df2],
            merge_type: MergeType::Vertical,
            by: None,
            logging: false,
        };
        
        let result = execute(params).unwrap();
        
        // Should have 6 rows (3 + 3)
        assert_eq!(result.height(), 6);
        // Should have 2 columns (id, name)
        assert_eq!(result.width(), 2);
    }

    #[test]
    fn test_merge_horizontal() {
        let df1 = create_df1();
        let df3 = create_df3();
        
        let params = MergeParams {
            dataframes: vec![df1, df3],
            merge_type: MergeType::Horizontal,
            by: Some("id".to_string()),
            logging: false,
        };
        
        let result = execute(params).unwrap();
        
        // Should have 3 rows
        assert_eq!(result.height(), 3);
        // Should have 3 columns (id, name, age)
        assert_eq!(result.width(), 3);
        assert!(result.has_column("name"));
        assert!(result.has_column("age"));
    }

    #[test]
    fn test_merge_horizontal_multiple() {
        let df1 = create_df1();
        let df3 = create_df3();
        let df4 = create_df4();
        
        let params = MergeParams {
            dataframes: vec![df1, df3, df4],
            merge_type: MergeType::Horizontal,
            by: Some("id".to_string()),
            logging: false,
        };
        
        let result = execute(params).unwrap();
        
        // Should have 3 rows
        assert_eq!(result.height(), 3);
        // Should have 4 columns (id, name, age, email)
        assert_eq!(result.width(), 4);
        assert!(result.has_column("name"));
        assert!(result.has_column("age"));
        assert!(result.has_column("email"));
    }

    #[test]
    fn test_merge_diagonal() {
        let df1 = create_df1();  // id, name
        let df3 = create_df3();  // id, age
        
        let params = MergeParams {
            dataframes: vec![df1, df3],
            merge_type: MergeType::Diagonal,
            by: None,
            logging: false,
        };
        
        let result = execute(params).unwrap();
        
        // Should have 6 rows (3 + 3)
        assert_eq!(result.height(), 6);
        // Should have 3 columns (id, name, age) with NaN where missing
        assert_eq!(result.width(), 3);
        assert!(result.has_column("id"));
        assert!(result.has_column("name"));
        assert!(result.has_column("age"));
    }

    #[test]
    fn test_merge_type_from_str() {
        assert_eq!(MergeType::from_str("vertical").unwrap(), MergeType::Vertical);
        assert_eq!(MergeType::from_str("v").unwrap(), MergeType::Vertical);
        assert_eq!(MergeType::from_str("horizontal").unwrap(), MergeType::Horizontal);
        assert_eq!(MergeType::from_str("h").unwrap(), MergeType::Horizontal);
        assert_eq!(MergeType::from_str("diagonal").unwrap(), MergeType::Diagonal);
        assert_eq!(MergeType::from_str("d").unwrap(), MergeType::Diagonal);
        assert!(MergeType::from_str("invalid").is_err());
    }

    #[test]
    fn test_merge_too_few_dataframes() {
        let df1 = create_df1();
        
        let params = MergeParams {
            dataframes: vec![df1],
            merge_type: MergeType::Vertical,
            by: None,
            logging: false,
        };
        
        let result = execute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_vertical_column_mismatch() {
        let df1 = create_df1();  // id, name
        let df3 = create_df3();  // id, age
        
        let params = MergeParams {
            dataframes: vec![df1, df3],
            merge_type: MergeType::Vertical,
            by: None,
            logging: false,
        };
        
        let result = execute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_horizontal_missing_by() {
        let df1 = create_df1();
        let df3 = create_df3();
        
        let params = MergeParams {
            dataframes: vec![df1, df3],
            merge_type: MergeType::Horizontal,
            by: None,  // Missing 'by' parameter
            logging: false,
        };
        
        let result = execute(params);
        assert!(result.is_err());
    }

    #[test]
    fn test_merge_horizontal_missing_key_column() {
        let df1 = create_df1();
        let polars_df = df! {
            "other_id" => &[1, 2, 3],
            "age" => &[25, 30, 35],
        }
        .unwrap();
        let df_no_id = crate::core::DataFrame::from_polars(polars_df);
        
        let params = MergeParams {
            dataframes: vec![df1, df_no_id],
            merge_type: MergeType::Horizontal,
            by: Some("id".to_string()),
            logging: false,
        };
        
        let result = execute(params);
        assert!(result.is_err());
    }
}
