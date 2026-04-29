//! Position application for add.to() operations
//!
//! Handles column positioning (start, end, after, before, index).

use crate::core::{DataFrame, AdditoryResult, AdditoryError, Position};

/// Apply position to insert new columns at specified location
///
/// # Parameters
/// - `df`: DataFrame to modify
/// - `new_columns`: Names of new columns to insert
/// - `position`: Where to insert the columns
///
/// # Returns
/// DataFrame with columns reordered
pub fn apply_position(
    df: DataFrame,
    new_columns: &[String],
    position: &Position,
) -> AdditoryResult<DataFrame> {
    let all_columns = df.column_names();
    let n_cols = all_columns.len();
    
    // Calculate target index
    let target_idx = match position {
        Position::Start => 0,
        Position::End => n_cols - new_columns.len(),
        Position::After(col) => {
            let idx = all_columns.iter().position(|c| c == col)
                .ok_or_else(|| AdditoryError::column_not_found(col, &all_columns))?;
            idx + 1
        }
        Position::Before(col) => {
            all_columns.iter().position(|c| c == col)
                .ok_or_else(|| AdditoryError::column_not_found(col, &all_columns))?
        }
        Position::Index(idx) => {
            // Validate index
            let abs_idx = if *idx < 0 {
                let positive = (n_cols as i32) + idx;
                if positive < 0 {
                    return Err(AdditoryError::position(
                        &format!("Invalid position index {} for DataFrame with {} columns", idx, n_cols),
                        &format!("Valid range: 0 to {} (positive) or -1 to -{} (negative)", n_cols - 1, n_cols - 1)
                    ));
                }
                positive as usize
            } else {
                if *idx as usize >= n_cols {
                    return Err(AdditoryError::position(
                        &format!("Invalid position index {} for DataFrame with {} columns", idx, n_cols),
                        &format!("Valid range: 0 to {} (positive) or -1 to -{} (negative)", n_cols - 1, n_cols - 1)
                    ));
                }
                *idx as usize
            };
            abs_idx
        }
    };
    
    // Build new column order
    let mut new_order = Vec::new();
    let new_col_set: std::collections::HashSet<_> = new_columns.iter().collect();
    
    // Add columns before target index
    for (i, col) in all_columns.iter().enumerate() {
        if i == target_idx {
            // Insert new columns here
            for new_col in new_columns {
                new_order.push(new_col.clone());
            }
        }
        
        // Add existing column if not in new columns
        if !new_col_set.contains(col) {
            new_order.push(col.to_string());
        }
    }
    
    // If target_idx is at end, add new columns now
    if target_idx >= all_columns.len() - new_columns.len() {
        for new_col in new_columns {
            if !new_order.contains(new_col) {
                new_order.push(new_col.clone());
            }
        }
    }
    
    // Select columns in new order
    df.select(&new_order)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    
    fn create_test_df() -> crate::DataFrame {
        let polars_df = df! {
            "a" => &[1, 2, 3],
            "b" => &[4, 5, 6],
            "c" => &[7, 8, 9],
            "new" => &[10, 11, 12],  // Add the "new" column
        }.unwrap();
        crate::DataFrame::from_polars(polars_df)
    }
    
    #[test]
    fn test_position_start() {
        let df = create_test_df();
        let new_cols = vec!["new".to_string()];
        let position = Position::Start;
        
        let result = apply_position(df, &new_cols, &position).unwrap();
        let cols = result.column_names();
        
        assert_eq!(cols[0], "new");
    }
    
    #[test]
    fn test_position_end() {
        let df = create_test_df();
        let new_cols = vec!["new".to_string()];
        let position = Position::End;
        
        let result = apply_position(df, &new_cols, &position).unwrap();
        let cols = result.column_names();
        
        assert_eq!(cols[cols.len() - 1], "new");
    }
    
    #[test]
    fn test_position_after() {
        let df = create_test_df();
        let new_cols = vec!["new".to_string()];
        let position = Position::After("a".to_string());
        
        let result = apply_position(df, &new_cols, &position).unwrap();
        let cols = result.column_names();
        
        // Should be: a, new, b, c
        assert_eq!(cols[1], "new");
    }
    
    #[test]
    fn test_position_index() {
        let df = create_test_df();
        let new_cols = vec!["new".to_string()];
        let position = Position::Index(1);
        
        let result = apply_position(df, &new_cols, &position).unwrap();
        let cols = result.column_names();
        
        assert_eq!(cols[1], "new");
    }
    
    #[test]
    fn test_position_invalid_index() {
        let df = create_test_df();
        let new_cols = vec!["new".to_string()];
        let position = Position::Index(10);
        
        let result = apply_position(df, &new_cols, &position);
        assert!(result.is_err());
    }
}
