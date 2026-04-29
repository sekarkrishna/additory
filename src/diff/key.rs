//! Key auto-detection and validation for diff operations.
//!
//! Finds unique columns shared between two DataFrames, tries single-column
//! keys first, then two-column composites.

use polars::prelude::*;
use crate::core::{AdditoryResult, AdditoryError};

/// Auto-detect a primary key from two DataFrames.
///
/// Strategy:
/// 1. Find columns present in both DataFrames.
/// 2. Try each common column as a single-column key (unique in both).
/// 3. If no single column works, try two-column composites.
pub fn detect_key(
    old: &DataFrame,
    new: &DataFrame,
) -> AdditoryResult<Vec<String>> {
    let old_cols: Vec<String> = old.get_column_names().iter().map(|s| s.to_string()).collect();
    let new_cols: Vec<String> = new.get_column_names().iter().map(|s| s.to_string()).collect();

    let common: Vec<String> = old_cols
        .iter()
        .filter(|c| new_cols.contains(c))
        .cloned()
        .collect();

    if common.is_empty() {
        return Err(AdditoryError::Validation(
            "No common columns found between old and new DataFrames.".to_string(),
            "Ensure both DataFrames share at least one column to use as a key.".to_string(),
        ));
    }

    // Try single-column keys
    for col in &common {
        if is_unique_in(old, col) && is_unique_in(new, col) {
            return Ok(vec![col.clone()]);
        }
    }

    // Try two-column composite keys
    for i in 0..common.len() {
        for j in (i + 1)..common.len() {
            let pair = vec![common[i].clone(), common[j].clone()];
            if is_composite_unique(old, &pair) && is_composite_unique(new, &pair) {
                return Ok(pair);
            }
        }
    }

    Err(AdditoryError::Validation(
        "Could not auto-detect a unique key from common columns.".to_string(),
        format!(
            "Common columns: {}. None are unique in both DataFrames. Specify a key explicitly.",
            common.join(", ")
        ),
    ))
}

/// Validate that all key columns exist in both DataFrames.
pub fn validate_key(
    old: &DataFrame,
    new: &DataFrame,
    key_cols: &[String],
) -> AdditoryResult<()> {
    let old_cols: Vec<String> = old.get_column_names().iter().map(|s| s.to_string()).collect();
    let new_cols: Vec<String> = new.get_column_names().iter().map(|s| s.to_string()).collect();

    for col in key_cols {
        if !old_cols.contains(col) {
            return Err(AdditoryError::Validation(
                format!("Key column '{}' not found in old DataFrame.", col),
                format!("Available columns in old: {}", old_cols.join(", ")),
            ));
        }
        if !new_cols.contains(col) {
            return Err(AdditoryError::Validation(
                format!("Key column '{}' not found in new DataFrame.", col),
                format!("Available columns in new: {}", new_cols.join(", ")),
            ));
        }
    }

    Ok(())
}

/// Check if a single column has all unique values in a DataFrame.
fn is_unique_in(df: &DataFrame, col: &str) -> bool {
    if let Ok(column) = df.column(col) {
        let series = column.as_materialized_series();
        let n_total = series.len();
        let n_unique = series.n_unique().unwrap_or(0);
        n_total > 0 && n_unique == n_total
    } else {
        false
    }
}

/// Check if a composite key (multiple columns) is unique in a DataFrame.
fn is_composite_unique(df: &DataFrame, cols: &[String]) -> bool {
    if df.height() == 0 {
        return true;
    }
    // Group by the composite columns and check if all groups have size 1
    let col_keys: Vec<String> = cols.iter().cloned().collect();
    match df.group_by(col_keys) {
        Ok(gb) => {
            match gb.count() {
                Ok(counts) => counts.height() == df.height(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}
