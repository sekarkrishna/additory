//! Row classification for diff operations.
//!
//! Performs an outer join on key columns and classifies each row as
//! new, deleted, changed, or no_change.

use std::collections::HashMap;
use polars::prelude::*;
use crate::core::{AdditoryResult, AdditoryError};
use super::types::{DiffResult, ChangedRow, CellChange};

/// Classify rows from two DataFrames into new, deleted, changed, and no_change.
///
/// Performs a full outer join on `key_cols`, then inspects each row to determine
/// its classification. Columns in `exclude` are skipped during comparison.
/// Columns in `carry` are included in output but not compared.
/// `groups` is reserved for hierarchical change detection.
pub fn classify_rows(
    old: &DataFrame,
    new: &DataFrame,
    key_cols: &[String],
    exclude: &[String],
    carry: &[String],
    _groups: &Option<HashMap<String, Vec<String>>>,
) -> AdditoryResult<DiffResult> {
    let old_cols: Vec<String> = old.get_column_names().iter().map(|s| s.to_string()).collect();
    let new_cols: Vec<String> = new.get_column_names().iter().map(|s| s.to_string()).collect();

    // Determine comparison columns (common non-key, non-excluded, non-carry)
    let compare_cols: Vec<String> = old_cols
        .iter()
        .filter(|c| {
            new_cols.contains(c)
                && !key_cols.contains(c)
                && !exclude.contains(c)
                && !carry.contains(c)
        })
        .cloned()
        .collect();

    // Build key-value maps for old and new
    let old_map = build_row_map(old, key_cols)?;
    let new_map = build_row_map(new, key_cols)?;

    // Collect all distinct keys
    let mut all_keys: Vec<String> = Vec::new();
    for k in old_map.keys() {
        all_keys.push(k.clone());
    }
    for k in new_map.keys() {
        if !all_keys.contains(k) {
            all_keys.push(k.clone());
        }
    }

    let mut new_row_indices: Vec<u32> = Vec::new();
    let mut deleted_row_indices: Vec<u32> = Vec::new();
    let mut no_change_indices_old: Vec<u32> = Vec::new();
    let mut changed_rows: Vec<ChangedRow> = Vec::new();

    for key_str in &all_keys {
        let in_old = old_map.get(key_str);
        let in_new = new_map.get(key_str);

        match (in_old, in_new) {
            (None, Some(&new_idx)) => {
                new_row_indices.push(new_idx as u32);
            }
            (Some(&old_idx), None) => {
                deleted_row_indices.push(old_idx as u32);
            }
            (Some(&old_idx), Some(&new_idx)) => {
                // Compare values in comparison columns
                let mut changes = Vec::new();
                let mut old_row_vals = HashMap::new();
                let mut new_row_vals = HashMap::new();

                // Populate key values
                let mut key_values = HashMap::new();
                for kc in key_cols {
                    let val = get_cell_str(old, kc, old_idx);
                    key_values.insert(kc.clone(), val.clone());
                    old_row_vals.insert(kc.clone(), val);
                    new_row_vals.insert(kc.clone(), get_cell_str(new, kc, new_idx));
                }

                for col in &compare_cols {
                    let old_val = get_cell_str(old, col, old_idx);
                    let new_val = get_cell_str(new, col, new_idx);
                    old_row_vals.insert(col.clone(), old_val.clone());
                    new_row_vals.insert(col.clone(), new_val.clone());

                    if old_val != new_val {
                        changes.push(CellChange {
                            column: col.clone(),
                            old_value: old_val,
                            new_value: new_val,
                            is_hierarchical: false,
                        });
                    }
                }

                if changes.is_empty() {
                    no_change_indices_old.push(old_idx as u32);
                } else {
                    changed_rows.push(ChangedRow {
                        key_values,
                        changes,
                        old_row: old_row_vals,
                        new_row: new_row_vals,
                    });
                }
            }
            (None, None) => {
                // Should not happen
            }
        }
    }

    // Build result DataFrames by selecting rows by index
    let new_rows = select_rows(new, &new_row_indices)?;
    let deleted_rows = select_rows(old, &deleted_row_indices)?;
    let no_change_rows = select_rows(old, &no_change_indices_old)?;

    Ok(DiffResult {
        key_cols: key_cols.to_vec(),
        new_rows,
        deleted_rows,
        changed_rows,
        no_change_rows,
        duplicate_rows: DataFrame::empty(),
    })
}

/// Build a map from composite key string → row index.
fn build_row_map(
    df: &DataFrame,
    key_cols: &[String],
) -> AdditoryResult<HashMap<String, usize>> {
    let mut map = HashMap::new();
    for idx in 0..df.height() {
        let key_str: String = key_cols
            .iter()
            .map(|kc| get_cell_str(df, kc, idx))
            .collect::<Vec<_>>()
            .join("|");
        // First occurrence wins (duplicates should already be handled)
        map.entry(key_str).or_insert(idx);
    }
    Ok(map)
}

/// Get a cell value as a string.
fn get_cell_str(df: &DataFrame, col: &str, idx: usize) -> String {
    df.column(col)
        .ok()
        .and_then(|s| s.get(idx).ok())
        .map(|v| format!("{}", v))
        .unwrap_or_else(|| "null".to_string())
}

/// Select rows from a DataFrame by index.
fn select_rows(df: &DataFrame, indices: &[u32]) -> AdditoryResult<DataFrame> {
    if indices.is_empty() {
        return Ok(df.head(Some(0)));
    }
    let idx = IdxCa::new("idx".into(), indices);
    df.take(&idx).map_err(|e| {
        AdditoryError::Validation(
            format!("Failed to select rows: {}", e),
            String::new(),
        )
    })
}
