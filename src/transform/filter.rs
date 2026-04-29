//! @filter mode - Filter rows and select columns
//!
//! Implements row filtering and column selection.

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use polars::prelude::*;

/// Filter rows and/or select columns
///
/// # Parameters
/// - `df`: DataFrame to filter
/// - `where_`: Optional filter condition
/// - `fetch`: Optional columns to select
///
/// # Returns
/// Filtered DataFrame
pub fn filter(
    df: DataFrame,
    where_: Option<String>,
    fetch: Option<Vec<String>>,
) -> AdditoryResult<DataFrame> {
    // Start with the inner DataFrame directly, no clone needed
    let mut result = df.inner().clone();  // TODO: Can we avoid this clone?
    
    // Apply filter condition if provided
    if let Some(condition) = where_ {
        result = apply_filter(&result, &condition)?;
    }
    
    // Select columns if provided
    if let Some(columns) = fetch {
        result = select_columns(&result, &columns)?;
    }
    
    Ok(DataFrame::from_polars(result))
}

/// Apply filter condition to DataFrame
fn apply_filter(df: &polars::prelude::DataFrame, condition: &str) -> AdditoryResult<polars::prelude::DataFrame> {
    // For now, use a simple implementation that evaluates the condition as a boolean expression
    // TODO: Implement proper expression parser
    
    // Try to parse simple conditions like "age > 30"
    let filtered = if let Some((col_name, op, value)) = parse_simple_condition(condition) {
        apply_simple_filter(df, &col_name, &op, &value)?
    } else {
        return Err(AdditoryError::operation(
            &format!("Unsupported filter condition: {}", condition),
            "Currently only simple conditions like 'column > value' are supported"
        ));
    };
    
    Ok(filtered)
}

/// Parse simple condition like "age > 30"
fn parse_simple_condition(condition: &str) -> Option<(String, String, String)> {
    let condition = condition.trim();
    
    for op in &[">", "<", ">=", "<=", "==", "!="] {
        if let Some(pos) = condition.find(op) {
            let col_name = condition[..pos].trim().to_string();
            let value = condition[pos + op.len()..].trim().to_string();
            return Some((col_name, op.to_string(), value));
        }
    }
    
    None
}

/// Apply simple filter
fn apply_simple_filter(
    df: &polars::prelude::DataFrame,
    col_name: &str,
    op: &str,
    value: &str,
) -> AdditoryResult<polars::prelude::DataFrame> {
    // Validate column name exists in DataFrame before using it
    let df_col_names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
    if !df_col_names.iter().any(|c| c == col_name) {
        return Err(AdditoryError::column_not_found(col_name, &df_col_names));
    }

    // Get column
    let col = df.column(col_name)
        .map_err(AdditoryError::Polars)?;
    
    let series = col.as_materialized_series();
    
    // Parse value and create mask
    let mask = match series.dtype() {
        DataType::Int32 | DataType::Int64 => {
            let val: i64 = value.parse().map_err(|_| AdditoryError::validation(
                &format!("Cannot parse '{}' as integer", value),
                "Provide a valid integer value"
            ))?;
            
            match op {
                ">" => series.gt(val).map_err(AdditoryError::Polars)?,
                "<" => series.lt(val).map_err(AdditoryError::Polars)?,
                ">=" => series.gt_eq(val).map_err(AdditoryError::Polars)?,
                "<=" => series.lt_eq(val).map_err(AdditoryError::Polars)?,
                "==" => series.equal(val).map_err(AdditoryError::Polars)?,
                "!=" => series.not_equal(val).map_err(AdditoryError::Polars)?,
                _ => return Err(AdditoryError::validation(
                    &format!("Unsupported operator: {}", op),
                    "Use >, <, >=, <=, ==, or !="
                )),
            }
        }
        _ => {
            return Err(AdditoryError::validation(
                &format!("Filtering on {} columns not yet supported", series.dtype()),
                "Currently only integer columns are supported"
            ));
        }
    };
    
    // Filter DataFrame
    let filtered = df.filter(&mask)
        .map_err(AdditoryError::Polars)?;
    
    Ok(filtered)
}

/// Select specific columns from DataFrame
fn select_columns(df: &polars::prelude::DataFrame, columns: &[String]) -> AdditoryResult<polars::prelude::DataFrame> {
    // Validate columns exist
    let df_col_names = df.get_column_names();
    for col_name in columns {
        if !df_col_names.iter().any(|c| c.as_str() == col_name.as_str()) {
            return Err(AdditoryError::column_not_found(
                col_name,
                &df_col_names.iter().map(|s| s.to_string()).collect::<Vec<_>>()
            ));
        }
    }
    
    // Select columns
    let selected = df.select(columns)
        .map_err(AdditoryError::Polars)?;
    
    Ok(selected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_filter_rows_only() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
            "salary" => &[50000, 60000, 70000, 80000],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let where_ = Some("age > 30".to_string());
        
        let result = filter(df, where_, None).unwrap();
        
        assert_eq!(result.height(), 2); // Only 35 and 40
        assert_eq!(result.width(), 2); // Both columns
    }
    
    #[test]
    fn test_filter_columns_only() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
            "salary" => &[50000, 60000, 70000, 80000],
            "name" => &["Alice", "Bob", "Charlie", "David"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let fetch = Some(vec!["age".to_string(), "name".to_string()]);
        
        let result = filter(df, None, fetch).unwrap();
        
        assert_eq!(result.height(), 4); // All rows
        assert_eq!(result.width(), 2); // Only age and name
        assert!(result.has_column("age"));
        assert!(result.has_column("name"));
        assert!(!result.has_column("salary"));
    }
    
    #[test]
    fn test_filter_rows_and_columns() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
            "salary" => &[50000, 60000, 70000, 80000],
            "name" => &["Alice", "Bob", "Charlie", "David"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let where_ = Some("age > 30".to_string());
        let fetch = Some(vec!["age".to_string(), "name".to_string()]);
        
        let result = filter(df, where_, fetch).unwrap();
        
        assert_eq!(result.height(), 2); // Only 35 and 40
        assert_eq!(result.width(), 2); // Only age and name
    }
    
    #[test]
    fn test_filter_invalid_column() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let fetch = Some(vec!["invalid_column".to_string()]);
        
        let result = filter(df, None, fetch);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_filter_no_operation() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner.clone());
        
        let result = filter(df, None, None).unwrap();
        
        // Should return unchanged DataFrame
        assert_eq!(result.height(), 4);
        assert_eq!(result.width(), 1);
    }
}
