//! @onehotencode mode - One-hot encoding
//!
//! Implements one-hot encoding for categorical columns.

use crate::core::{DataFrame, AdditoryResult, AdditoryError, StrategyValue};
use polars::prelude::*;
use std::collections::HashMap;

/// One-hot encode categorical columns
///
/// # Parameters
/// - `df`: DataFrame to encode
/// - `columns`: Columns to encode
/// - `strategy`: Optional configuration (prefix, drop_first)
///
/// # Returns
/// DataFrame with one-hot encoded columns
pub fn onehotencode(
    df: DataFrame,
    columns: Vec<String>,
    strategy: Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<DataFrame> {
    // Validate columns exist
    for col_name in &columns {
        if !df.has_column(col_name) {
            return Err(AdditoryError::column_not_found(col_name, &df.column_names()));
        }
    }
    
    // Get strategy options
    let use_prefix = get_bool_option(&strategy, "prefix", true)?;
    let drop_first = get_bool_option(&strategy, "drop_first", false)?;
    
    // Encode columns
    let mut result = df.inner().clone();
    
    for col_name in columns {
        result = encode_column(&result, &col_name, use_prefix, drop_first)?;
    }
    
    Ok(DataFrame::from_polars(result))
}

/// Get boolean option from strategy
fn get_bool_option(
    strategy: &Option<HashMap<String, StrategyValue>>,
    key: &str,
    default: bool,
) -> AdditoryResult<bool> {
    if let Some(strat) = strategy {
        if let Some(value) = strat.get(key) {
            match value {
                StrategyValue::Bool(b) => Ok(*b),
                _ => Err(AdditoryError::validation(
                    &format!("Option '{}' must be a boolean", key),
                    "Use true or false"
                )),
            }
        } else {
            Ok(default)
        }
    } else {
        Ok(default)
    }
}

/// Encode a single column
fn encode_column(
    df: &polars::prelude::DataFrame,
    col_name: &str,
    use_prefix: bool,
    drop_first: bool,
) -> AdditoryResult<polars::prelude::DataFrame> {
    // Get column
    let col = df.column(col_name)
        .map_err(AdditoryError::Polars)?;
    
    let series = col.as_materialized_series();
    
    // Get unique values
    let unique = series.unique()
        .map_err(AdditoryError::Polars)?;
    
    let mut unique_values: Vec<String> = unique.str()
        .map_err(AdditoryError::Polars)?
        .into_iter()
        .filter_map(|opt| opt.map(|s| s.to_string()))
        .collect();
    
    // Sort to ensure consistent ordering (drop_first will drop alphabetically first)
    unique_values.sort();
    
    // Determine which categories to encode
    let categories_to_encode: Vec<String> = if drop_first && unique_values.len() > 1 {
        unique_values.into_iter().skip(1).collect()
    } else {
        unique_values
    };
    
    // Create binary columns for each category
    let mut result = df.clone();
    
    for category in categories_to_encode {
        let new_col_name = if use_prefix {
            format!("{}_{}", col_name, category)
        } else {
            category.clone()
        };
        
        // Create binary column
        let binary_col = create_binary_column(series, &category)?;
        
        // Add to DataFrame - with_column modifies in place
        let new_col_series = binary_col.as_materialized_series().clone().with_name(new_col_name.as_str().into());
        result.with_column(new_col_series)
            .map_err(AdditoryError::Polars)?;
    }
    
    Ok(result)
}

/// Create binary column for a category
fn create_binary_column(series: &Series, category: &str) -> AdditoryResult<Column> {
    let str_series = series.str()
        .map_err(AdditoryError::Polars)?;
    
    let binary: Vec<i32> = str_series.into_iter()
        .map(|opt| {
            match opt {
                Some(val) => if val == category { 1 } else { 0 },
                None => 0,
            }
        })
        .collect();
    
    Ok(Column::new(series.name().clone(), binary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_onehotencode_basic() {
        let df_inner = df! {
            "id" => &[1, 2, 3],
            "category" => &["A", "B", "A"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["category".to_string()];
        
        let result = onehotencode(df, columns, None).unwrap();
        
        // Should have original columns plus category_A and category_B
        assert!(result.has_column("category_A"));
        assert!(result.has_column("category_B"));
    }
    
    #[test]
    fn test_onehotencode_no_prefix() {
        let df_inner = df! {
            "id" => &[1, 2, 3],
            "category" => &["A", "B", "A"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["category".to_string()];
        let mut strategy = HashMap::new();
        strategy.insert("prefix".to_string(), StrategyValue::Bool(false));
        
        let result = onehotencode(df, columns, Some(strategy)).unwrap();
        
        // Should have columns A and B (no prefix)
        assert!(result.has_column("A"));
        assert!(result.has_column("B"));
    }
    
    #[test]
    fn test_onehotencode_drop_first() {
        let df_inner = df! {
            "id" => &[1, 2, 3],
            "category" => &["A", "B", "C"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["category".to_string()];
        let mut strategy = HashMap::new();
        strategy.insert("drop_first".to_string(), StrategyValue::Bool(true));
        
        let result = onehotencode(df, columns, Some(strategy)).unwrap();
        
        // Should have category_B and category_C (A dropped)
        assert!(!result.has_column("category_A"));
        assert!(result.has_column("category_B"));
        assert!(result.has_column("category_C"));
    }
    
    #[test]
    fn test_onehotencode_invalid_column() {
        let df_inner = df! {
            "id" => &[1, 2, 3],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["invalid_column".to_string()];
        
        let result = onehotencode(df, columns, None);
        assert!(result.is_err());
    }
}
