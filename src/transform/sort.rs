//! @sort mode - Sort DataFrame
//!
//! Implements sorting by one or more columns.

use crate::core::{DataFrame, AdditoryResult, AdditoryError, By, AsParam};
use polars::prelude::*;

/// Sort DataFrame by columns
///
/// # Parameters
/// - `df`: DataFrame to sort
/// - `by`: Column(s) to sort by
/// - `name`: Sort order ('asc' or 'desc')
///
/// # Returns
/// Sorted DataFrame
pub fn sort(
    df: DataFrame,
    by: By,
    name: Option<AsParam>,
) -> AdditoryResult<DataFrame> {
    // Get sort order (default to ascending)
    let descending = match name {
        Some(AsParam::SortOrder(order)) | Some(AsParam::Single(order)) => {
            match order.to_lowercase().as_str() {
                "asc" => false,
                "desc" => true,
                _ => return Err(AdditoryError::validation(
                    &format!("Invalid sort order: {}. Must be 'asc' or 'desc'", order),
                    "Use name='asc' or name='desc'"
                )),
            }
        }
        None => false, // Default to ascending
        _ => return Err(AdditoryError::validation(
            "name parameter for @sort must be 'asc' or 'desc'",
            "Use name='asc' or name='desc'"
        )),
    };
    
    // Get columns to sort by
    let columns = by.columns();
    
    // Validate columns exist
    for col_name in &columns {
        if !df.has_column(col_name) {
            return Err(AdditoryError::column_not_found(col_name, &df.column_names()));
        }
    }
    
    // Sort DataFrame
    let inner = df.inner();
    let sorted = sort_dataframe(inner, &columns, descending)?;
    
    Ok(DataFrame::from_polars(sorted))
}

/// Sort DataFrame by columns
fn sort_dataframe(
    df: &polars::prelude::DataFrame,
    columns: &[&str],
    descending: bool,
) -> AdditoryResult<polars::prelude::DataFrame> {
    // Create sort options for each column
    let sort_options = SortMultipleOptions::default()
        .with_order_descending(descending);
    
    // Convert to Vec<String> which implements IntoVec<PlSmallStr>
    let col_strings: Vec<String> = columns.iter().map(|s| s.to_string()).collect();
    
    // Sort by columns
    let sorted = df.sort(col_strings, sort_options)
        .map_err(AdditoryError::Polars)?;
    
    Ok(sorted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_sort_single_column_asc() {
        let df_inner = df! {
            "name" => &["Charlie", "Alice", "Bob"],
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("age".to_string());
        let name = Some(AsParam::SortOrder("asc".to_string()));
        
        let result = sort(df, by, name).unwrap();
        
        assert_eq!(result.height(), 3);
        // First row should be Alice (age 25)
        let age_col = result.column("age").unwrap();
        let age_series = age_col.as_materialized_series();
        assert_eq!(age_series.i32().unwrap().get(0).unwrap(), 25);
    }
    
    #[test]
    fn test_sort_single_column_desc() {
        let df_inner = df! {
            "name" => &["Charlie", "Alice", "Bob"],
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("age".to_string());
        let name = Some(AsParam::SortOrder("desc".to_string()));
        
        let result = sort(df, by, name).unwrap();
        
        assert_eq!(result.height(), 3);
        // First row should be Charlie (age 35)
        let age_col = result.column("age").unwrap();
        let age_series = age_col.as_materialized_series();
        assert_eq!(age_series.i32().unwrap().get(0).unwrap(), 35);
    }
    
    #[test]
    fn test_sort_multiple_columns() {
        let df_inner = df! {
            "category" => &["A", "A", "B", "B"],
            "value" => &[20, 10, 40, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Multiple(vec!["category".to_string(), "value".to_string()]);
        
        let result = sort(df, by, None).unwrap();
        
        assert_eq!(result.height(), 4);
        // First row should be A, 10
        let value_col = result.column("value").unwrap();
        let value_series = value_col.as_materialized_series();
        assert_eq!(value_series.i32().unwrap().get(0).unwrap(), 10);
    }
    
    #[test]
    fn test_sort_default_ascending() {
        let df_inner = df! {
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("age".to_string());
        
        let result = sort(df, by, None).unwrap();
        
        // Should default to ascending
        let age_col = result.column("age").unwrap();
        let age_series = age_col.as_materialized_series();
        assert_eq!(age_series.i32().unwrap().get(0).unwrap(), 25);
    }
    
    #[test]
    fn test_sort_invalid_column() {
        let df_inner = df! {
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("invalid_column".to_string());
        
        let result = sort(df, by, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sort_invalid_order() {
        let df_inner = df! {
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let by = By::Single("age".to_string());
        let name = Some(AsParam::SortOrder("invalid".to_string()));
        
        let result = sort(df, by, name);
        assert!(result.is_err());
    }
}
