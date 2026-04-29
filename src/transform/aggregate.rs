//! @aggregate mode - Group by and aggregate
//!
//! Implements group-by operations with aggregation functions.

use crate::core::{DataFrame, AdditoryResult, AdditoryError, By, StrategyValue};
use polars::prelude::*;
use std::collections::HashMap;

/// Aggregate DataFrame by grouping
///
/// # Parameters
/// - `df`: DataFrame to aggregate
/// - `by`: Column(s) to group by
/// - `strategy`: Aggregation functions per column
///
/// # Supported Aggregation Functions
/// - sum, mean, median, min, max, count, std, var
///
/// # Returns
/// Aggregated DataFrame
pub fn aggregate(
    df: DataFrame,
    by: By,
    strategy: HashMap<String, StrategyValue>,
) -> AdditoryResult<DataFrame> {
    // Get group by columns
    let group_cols = by.columns();
    
    // Validate group columns exist
    for col_name in &group_cols {
        if !df.has_column(col_name) {
            return Err(AdditoryError::column_not_found(col_name, &df.column_names()));
        }
    }
    
    // Parse aggregation strategy
    let agg_specs = parse_aggregation_strategy(&strategy)?;
    
    // Validate aggregation columns exist
    for (col_name, _) in &agg_specs {
        if !df.has_column(col_name) {
            return Err(AdditoryError::column_not_found(col_name, &df.column_names()));
        }
    }
    
    // Perform aggregation
    let inner = df.inner();
    let result = perform_aggregation(inner, &group_cols, &agg_specs)?;
    
    Ok(DataFrame::from_polars(result))
}

/// Parse aggregation strategy from HashMap
fn parse_aggregation_strategy(
    strategy: &HashMap<String, StrategyValue>,
) -> AdditoryResult<Vec<(String, String)>> {
    let mut agg_specs = Vec::new();
    
    for (col_name, value) in strategy {
        let agg_func = match value {
            StrategyValue::String(s) => s.clone(),
            _ => return Err(AdditoryError::validation(
                "Aggregation function must be a string",
                "Use 'sum', 'mean', 'median', 'min', 'max', 'count', 'std', 'var'"
            )),
        };
        
        // Validate aggregation function
        if !is_valid_agg_function(&agg_func) {
            return Err(AdditoryError::validation(
                &format!("Invalid aggregation function: {}", agg_func),
                "Use 'sum', 'mean', 'median', 'min', 'max', 'count', 'std', 'var'"
            ));
        }
        
        agg_specs.push((col_name.clone(), agg_func));
    }
    
    Ok(agg_specs)
}

/// Check if aggregation function is valid
fn is_valid_agg_function(func: &str) -> bool {
    matches!(func, "sum" | "mean" | "median" | "min" | "max" | "count" | "std" | "var")
}

/// Perform aggregation on DataFrame
fn perform_aggregation(
    df: &polars::prelude::DataFrame,
    group_cols: &[&str],
    agg_specs: &[(String, String)],
) -> AdditoryResult<polars::prelude::DataFrame> {
    // Build aggregation expressions
    let mut agg_exprs = Vec::new();
    
    for (col_name, agg_func) in agg_specs {
        let expr = match agg_func.as_str() {
            "sum" => col(col_name).sum(),
            "mean" => col(col_name).mean(),
            "median" => col(col_name).median(),
            "min" => col(col_name).min(),
            "max" => col(col_name).max(),
            "count" => col(col_name).count(),
            "std" => col(col_name).std(1),
            "var" => col(col_name).var(1),
            _ => return Err(AdditoryError::validation(
                &format!("Unsupported aggregation function: {}", agg_func),
                "Use 'sum', 'mean', 'median', 'min', 'max', 'count', 'std', 'var'"
            )),
        };
        
        agg_exprs.push(expr.alias(col_name));
    }
    
    // Perform group by and aggregation
    let result = df.clone().lazy()
        .group_by(group_cols)
        .agg(&agg_exprs)
        .collect()
        .map_err(AdditoryError::Polars)?;
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_aggregate_sum() {
        let df_inner = df! {
            "category" => &["A", "A", "B", "B"],
            "value" => &[10, 20, 30, 40],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("category".to_string());
        let mut strategy = HashMap::new();
        strategy.insert("value".to_string(), StrategyValue::String("sum".to_string()));
        
        let result = aggregate(df, by, strategy).unwrap();
        
        assert_eq!(result.height(), 2); // Two categories
        assert!(result.has_column("category"));
        assert!(result.has_column("value"));
    }
    
    #[test]
    fn test_aggregate_multiple_functions() {
        let df_inner = df! {
            "category" => &["A", "A", "B", "B"],
            "value" => &[10, 20, 30, 40],
            "quantity" => &[1, 2, 3, 4],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("category".to_string());
        let mut strategy = HashMap::new();
        strategy.insert("value".to_string(), StrategyValue::String("sum".to_string()));
        strategy.insert("quantity".to_string(), StrategyValue::String("mean".to_string()));
        
        let result = aggregate(df, by, strategy).unwrap();
        
        assert_eq!(result.height(), 2);
        assert!(result.has_column("value"));
        assert!(result.has_column("quantity"));
    }
    
    #[test]
    fn test_aggregate_multiple_group_cols() {
        let df_inner = df! {
            "category" => &["A", "A", "B", "B"],
            "region" => &["East", "West", "East", "West"],
            "value" => &[10, 20, 30, 40],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Multiple(vec!["category".to_string(), "region".to_string()]);
        let mut strategy = HashMap::new();
        strategy.insert("value".to_string(), StrategyValue::String("sum".to_string()));
        
        let result = aggregate(df, by, strategy).unwrap();
        
        assert_eq!(result.height(), 4); // Four unique combinations
    }
    
    #[test]
    fn test_aggregate_invalid_function() {
        let df_inner = df! {
            "category" => &["A", "A"],
            "value" => &[10, 20],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("category".to_string());
        let mut strategy = HashMap::new();
        strategy.insert("value".to_string(), StrategyValue::String("invalid".to_string()));
        
        let result = aggregate(df, by, strategy);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_aggregate_invalid_column() {
        let df_inner = df! {
            "category" => &["A", "A"],
            "value" => &[10, 20],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("invalid_column".to_string());
        let mut strategy = HashMap::new();
        strategy.insert("value".to_string(), StrategyValue::String("sum".to_string()));
        
        let result = aggregate(df, by, strategy);
        assert!(result.is_err());
    }
}
