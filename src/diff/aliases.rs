//! Alias application for diff operations.
//!
//! Renames columns in both DataFrames to canonical names using
//! a case-insensitive variant-to-canonical mapping.

use std::collections::HashMap;
use polars::prelude::*;
use crate::core::{AdditoryResult, AdditoryError};

/// Apply aliases to both DataFrames, renaming variant columns to canonical names.
///
/// The alias mapping is `canonical_name → [variant1, variant2, ...]`.
/// Matching is case-insensitive.
pub fn apply_aliases(
    old: &DataFrame,
    new: &DataFrame,
    aliases: &HashMap<String, Vec<String>>,
) -> AdditoryResult<(DataFrame, DataFrame)> {
    let old_renamed = rename_columns(old, aliases, "old")?;
    let new_renamed = rename_columns(new, aliases, "new")?;
    Ok((old_renamed, new_renamed))
}

/// Rename columns in a single DataFrame according to alias mappings.
fn rename_columns(
    df: &DataFrame,
    aliases: &HashMap<String, Vec<String>>,
    label: &str,
) -> AdditoryResult<DataFrame> {
    let mut result = df.clone();
    let current_cols: Vec<String> = result.get_column_names().iter().map(|s| s.to_string()).collect();

    for (canonical, variants) in aliases {
        for variant in variants {
            // Case-insensitive match against current column names
            for col in &current_cols {
                if col.eq_ignore_ascii_case(variant) && col != canonical {
                    result.rename(col, canonical.clone().into()).map_err(|e| {
                        AdditoryError::Validation(
                            format!(
                                "Failed to rename column '{}' to '{}' in {} DataFrame: {}",
                                col, canonical, label, e
                            ),
                            String::new(),
                        )
                    })?;
                    break;
                }
            }
        }
    }

    Ok(result)
}
