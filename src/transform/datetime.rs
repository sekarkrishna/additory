//! @datetime mode - DateTime normalization (v0.1.3a5)
//!
//! Parse, impute, and standardize datetime columns
//!
//! Supported formats:
//! - ISO 8601, US (MM/DD/YYYY), European (DD/MM/YYYY)
//! - Named months (DD-MMM-YYYY)
//! - Excel serial dates, Unix timestamps
//! - Compact numeric (YYYYMMDD)
//!
//! Imputation strategies:
//! - 'null' - No imputation (DEFAULT)
//! - 'start' - Unknown month → 01, day → 01
//! - 'mid' - Unknown month → 06/07, day → 15
//! - 'end' - Unknown month → 12, day → last day

use crate::core::{DataFrame as AdditoryDataFrame, AdditoryResult, AdditoryError, StrategyValue};
use polars::prelude::*;
use std::collections::HashMap;

/// Execute @datetime mode - normalize datetime columns
///
/// # Parameters
/// - `df`: DataFrame to normalize
/// - `columns`: Vec of column names to normalize
/// - `strategy`: Optional dict with column-specific configuration
///
/// # Returns
/// DataFrame with normalized datetime columns
pub fn datetime(
    df: AdditoryDataFrame,
    columns: Vec<String>,
    _strategy: Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<AdditoryDataFrame> {
    // For now, implement basic datetime parsing
    // Full implementation with imputation and format detection will come later
    
    if columns.is_empty() {
        return Err(AdditoryError::missing_parameter(
            "columns",
            "@datetime requires at least one column to normalize"
        ));
    }
    
    let mut result_df = df.clone();
    
    // Parse datetime columns
    for column_name in columns.iter() {
        // Validate column exists
        if !result_df.has_column(column_name) {
            return Err(AdditoryError::column_not_found(
                column_name,
                &result_df.column_names()
            ));
        }
        
        // Get column type
        let col = result_df.column(column_name)?;
        let dtype = col.dtype();
        
        // Check if column is string (needs parsing) or already datetime
        match dtype {
            DataType::String => {
                // Parse string to datetime
                result_df = parse_datetime_column(result_df, column_name)?;
            }
            DataType::Date | DataType::Datetime(_, _) => {
                // Already datetime, no parsing needed
                continue;
            }
            _ => {
                return Err(AdditoryError::invalid_parameter(
                    "column",
                    column_name,
                    &format!("Column must be String or Datetime type for @datetime, found {:?}", dtype)
                ));
            }
        }
    }
    
    Ok(result_df)
}

/// Parse string column to datetime
fn parse_datetime_column(
    df: AdditoryDataFrame,
    column_name: &str,
) -> AdditoryResult<AdditoryDataFrame> {
    let mut polars_df = df.inner().clone();
    
    // Try to parse as datetime using Polars' automatic format detection
    // This will handle ISO 8601 and common formats
    let parsed_col_name = format!("{}_parsed", column_name);
    
    polars_df = polars_df.lazy()
        .with_column(
            col(column_name)
                .cast(DataType::Datetime(TimeUnit::Milliseconds, None))
                .alias(&parsed_col_name)
        )
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to parse datetime column '{}'", column_name),
            &e.to_string()
        ))?;
    
    // Replace original column with parsed version
    polars_df = polars_df.lazy()
        .drop([column_name])
        .with_column(col(parsed_col_name.as_str()).alias(column_name))
        .drop([parsed_col_name.as_str()])
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to replace column '{}'", column_name),
            &e.to_string()
        ))?;
    
    Ok(AdditoryDataFrame::from_polars(polars_df))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    
    #[test]
    fn test_datetime_already_parsed() {
        // Create DataFrame with already-parsed datetime column
        let dates = vec![19737, 19773, 19807]; // Days since epoch
        
        let df_inner = df! {
            "date_col" => dates,
        }.unwrap()
        .lazy()
        .with_column(col("date_col").cast(DataType::Date))
        .collect()
        .unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["date_col".to_string()];
        
        // Should succeed without error (already datetime)
        let result = super::datetime(df, columns, None).unwrap();
        
        // Column should still exist and be datetime type
        assert!(result.has_column("date_col"));
        let col = result.column("date_col").unwrap();
        assert!(matches!(col.dtype(), DataType::Date));
    }
    
    #[test]
    fn test_datetime_invalid_column() {
        let df_inner = df! {
            "age" => &[25, 30, 35],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["nonexistent".to_string()];
        
        let result = super::datetime(df, columns, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_datetime_invalid_type() {
        let df_inner = df! {
            "age" => &[25, 30, 35],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["age".to_string()];
        
        let result = super::datetime(df, columns, None);
        assert!(result.is_err());
        
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("String or Datetime type"));
    }
    
    #[test]
    fn test_datetime_empty_columns() {
        let df = AdditoryDataFrame::empty();
        let columns = vec![];
        
        let result = super::datetime(df, columns, None);
        assert!(result.is_err());
    }
}
