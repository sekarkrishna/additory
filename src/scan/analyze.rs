//! Statistical analysis implementation for scan module
//!
//! This module provides statistical profiling and data quality assessment functionality.

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::utils::logging::Logger;
use super::types::AnalyzeParams;
use polars::prelude::*;

/// Execute statistical analysis
pub fn execute_analyze(
    df: DataFrame,
    _params: &AnalyzeParams,
    logger: &Logger,
) -> AdditoryResult<DataFrame> {
    logger.log_result("add.scan(@analyze)", "Analyzing data quality and distributions");

    // Get the Polars DataFrame
    let original_df = df.inner();
    
    // Analyze each column
    let mut column_names: Vec<String> = Vec::new();
    let mut data_types: Vec<String> = Vec::new();
    let mut row_counts: Vec<i64> = Vec::new();
    let mut null_counts: Vec<i64> = Vec::new();
    let mut null_percentages: Vec<f64> = Vec::new();
    let mut unique_counts: Vec<i64> = Vec::new();
    let mut means: Vec<Option<f64>> = Vec::new();
    let mut stds: Vec<Option<f64>> = Vec::new();
    let mut mins: Vec<Option<f64>> = Vec::new();
    let mut maxs: Vec<Option<f64>> = Vec::new();
    
    let total_rows = original_df.height() as i64;
    
    for col in original_df.get_columns() {
        let series = col.as_materialized_series();
        let col_name = series.name().to_string();
        column_names.push(col_name.clone());
        data_types.push(format!("{:?}", series.dtype()));
        row_counts.push(total_rows);
        
        // Count nulls
        let null_count = series.null_count() as i64;
        null_counts.push(null_count);
        null_percentages.push((null_count as f64 / total_rows as f64) * 100.0);
        
        // Count unique values
        let unique_count = series.n_unique()
            .map_err(|e: PolarsError| AdditoryError::operation(
                "Failed to count unique values",
                &e.to_string()
            ))? as i64;
        unique_counts.push(unique_count);
        
        // Calculate statistics for numeric columns
        match series.dtype() {
            DataType::Float64 | DataType::Float32 | 
            DataType::Int64 | DataType::Int32 | DataType::Int16 | DataType::Int8 |
            DataType::UInt64 | DataType::UInt32 | DataType::UInt16 | DataType::UInt8 => {
                // Cast to f64 for statistics
                let numeric_col = series.cast(&DataType::Float64)
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to cast to numeric",
                        &e.to_string()
                    ))?;
                
                let f64_col = numeric_col.f64()
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to extract numeric values",
                        &e.to_string()
                    ))?;
                
                // Calculate mean
                let mean_val = f64_col.mean();
                means.push(mean_val);
                
                // Calculate std
                let std_val = f64_col.std(1);  // ddof=1 for sample std
                stds.push(std_val);
                
                // Calculate min/max
                let min_val = f64_col.min();
                let max_val = f64_col.max();
                mins.push(min_val);
                maxs.push(max_val);
            }
            _ => {
                // Non-numeric column
                means.push(None);
                stds.push(None);
                mins.push(None);
                maxs.push(None);
            }
        }
    }
    
    // Create analysis DataFrame
    let analysis_df = polars::prelude::DataFrame::new(vec![
        polars::prelude::Column::Series(Series::new("column".into(), &column_names)),
        polars::prelude::Column::Series(Series::new("dtype".into(), &data_types)),
        polars::prelude::Column::Series(Series::new("count".into(), &row_counts)),
        polars::prelude::Column::Series(Series::new("null_count".into(), &null_counts)),
        polars::prelude::Column::Series(Series::new("null_pct".into(), &null_percentages)),
        polars::prelude::Column::Series(Series::new("unique".into(), &unique_counts)),
        polars::prelude::Column::Series(Series::new("mean".into(), &means)),
        polars::prelude::Column::Series(Series::new("std".into(), &stds)),
        polars::prelude::Column::Series(Series::new("min".into(), &mins)),
        polars::prelude::Column::Series(Series::new("max".into(), &maxs)),
    ])
    .map_err(|e: PolarsError| AdditoryError::operation(
        "Failed to create analysis DataFrame",
        &e.to_string()
    ))?;
    
    logger.log_dataframe("add.scan(@analyze)", "Analysis results", analysis_df.height(), analysis_df.width());
    
    Ok(DataFrame::from_polars(analysis_df))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use crate::core::DataFrame as AdditoryDataFrame;

    #[test]
    fn test_analyze_basic() {
        // Create test DataFrame
        let df = polars::prelude::DataFrame::new(vec![
            polars::prelude::Column::Series(Series::new("age".into(), &[25, 30, 35])),
            polars::prelude::Column::Series(Series::new("name".into(), &["Alice", "Bob", "Charlie"])),
        ]).unwrap();
        
        let params = AnalyzeParams {
            columns: None,
            where_clause: None,
            rows: None,
            focus: None,
        };
        
        let logger = Logger::new(false);
        let result = execute_analyze(AdditoryDataFrame::from_polars(df), &params, &logger).unwrap();
        
        // Verify result structure
        let result_df = result.inner();
        assert_eq!(result_df.height(), 2);  // 2 columns analyzed
        assert!(result_df.column("column").is_ok());
        assert!(result_df.column("dtype").is_ok());
        assert!(result_df.column("null_count").is_ok());
    }
}

