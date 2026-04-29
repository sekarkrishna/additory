//! Duplicate handling for diff operations.
//!
//! Collapses identical duplicate-key rows into one and flags
//! non-identical duplicate-key rows.

use polars::prelude::*;
use crate::core::{AdditoryResult, AdditoryError};

/// Handle duplicate keys in both DataFrames.
///
/// Returns `(cleaned_old, cleaned_new, duplicate_rows)` where:
/// - `cleaned_old` / `cleaned_new` have no duplicate keys
/// - `duplicate_rows` contains non-identical duplicates that were removed
pub fn handle_duplicates(
    old: &DataFrame,
    new: &DataFrame,
    key_cols: &[String],
) -> AdditoryResult<(DataFrame, DataFrame, DataFrame)> {
    let cleaned_old = deduplicate_df(old, key_cols, "old")?;
    let cleaned_new = deduplicate_df(new, key_cols, "new")?;

    // Collect non-identical duplicates from both sides
    let dup_old = find_non_identical_duplicates(old, key_cols)?;
    let dup_new = find_non_identical_duplicates(new, key_cols)?;

    // Combine duplicate rows (vertical stack if schemas match, otherwise just use old)
    let duplicate_rows = if dup_old.height() > 0 && dup_new.height() > 0 {
        // Try to align schemas and stack
        dup_old.vstack(&dup_new).unwrap_or(dup_old)
    } else if dup_new.height() > 0 {
        dup_new
    } else {
        dup_old
    };

    Ok((cleaned_old, cleaned_new, duplicate_rows))
}

/// Remove duplicate-key rows from a DataFrame.
/// Identical duplicates are collapsed to one row.
/// Non-identical duplicates: keep the first occurrence.
fn deduplicate_df(
    df: &DataFrame,
    key_cols: &[String],
    label: &str,
) -> AdditoryResult<DataFrame> {
    if df.height() == 0 {
        return Ok(df.clone());
    }

    let col_strs: Vec<String> = key_cols.to_vec();

    // Use distinct to keep first occurrence of each key combination
    df.unique::<String, String>(Some(col_strs.as_slice()), UniqueKeepStrategy::First, None)
        .map_err(|e| {
            AdditoryError::Validation(
                format!("Failed to deduplicate {} DataFrame: {}", label, e),
                "Check that key columns exist and contain valid data.".to_string(),
            )
        })
}

/// Find rows with duplicate keys where the rows are NOT identical.
fn find_non_identical_duplicates(
    df: &DataFrame,
    key_cols: &[String],
) -> AdditoryResult<DataFrame> {
    if df.height() == 0 {
        return Ok(df.clone());
    }

    let col_keys: Vec<String> = key_cols.iter().map(|s| s.clone()).collect();

    // Group by key columns
    let groups = df.group_by(col_keys).map_err(|e| {
        AdditoryError::Validation(
            format!("Failed to group by key columns: {}", e),
            "Check that key columns exist.".to_string(),
        )
    })?;

    let counts = groups.count().map_err(|e| {
        AdditoryError::Validation(
            format!("Failed to count groups: {}", e),
            String::new(),
        )
    })?;

    // Find keys that appear more than once
    let mut dup_key_masks: Vec<bool> = Vec::new();
    let mut has_dups = false;

    // Build a set of duplicate key values
    let mut dup_keys = std::collections::HashSet::new();
    for row_idx in 0..counts.height() {
        // Check if count > 1 for any count column
        for col in counts.get_columns() {
            if col.name().ends_with("_count") {
                if let Ok(val) = col.get(row_idx) {
                    if let Some(n) = val.try_extract::<u32>().ok() {
                        if n > 1 {
                            // Build key string for this row
                            let key_str: String = key_cols
                                .iter()
                                .map(|kc| {
                                    counts
                                        .column(kc)
                                        .ok()
                                        .and_then(|s| s.get(row_idx).ok())
                                        .map(|v| format!("{}", v))
                                        .unwrap_or_default()
                                })
                                .collect::<Vec<_>>()
                                .join("|");
                            dup_keys.insert(key_str);
                            has_dups = true;
                        }
                    }
                }
            }
        }
    }

    if !has_dups {
        // Return empty DataFrame with same schema
        return Ok(df.head(Some(0)));
    }

    // Filter original DataFrame to only duplicate-key rows
    for row_idx in 0..df.height() {
        let key_str: String = key_cols
            .iter()
            .map(|kc| {
                df.column(kc)
                    .ok()
                    .and_then(|s| s.get(row_idx).ok())
                    .map(|v| format!("{}", v))
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>()
            .join("|");
        dup_key_masks.push(dup_keys.contains(&key_str));
    }

    let mask = BooleanChunked::from_slice("mask".into(), &dup_key_masks);
    df.filter(&mask).map_err(|e| {
        AdditoryError::Validation(
            format!("Failed to filter duplicate rows: {}", e),
            String::new(),
        )
    })
}
