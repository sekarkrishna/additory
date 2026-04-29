//! Diff engine module.
//!
//! Provides the public `diff()` function that orchestrates the full pipeline:
//! validate → parse strategy → apply aliases → detect/validate key →
//! handle duplicates → classify → return DiffResult.

pub mod types;
pub mod strategy;
pub mod key;
pub mod duplicates;
pub mod classify;
pub mod aliases;

pub use types::*;
pub use strategy::parse_strategy;

use std::collections::HashMap;
use polars::prelude::DataFrame;
use crate::core::{AdditoryResult, AdditoryError};

/// Perform a full diff between two DataFrames.
///
/// Orchestrates the complete pipeline:
/// 1. Parse and validate strategy
/// 2. Apply aliases (if provided)
/// 3. Detect or validate key columns
/// 4. Handle duplicates
/// 5. Classify rows
/// 6. Return structured DiffResult
pub fn diff(
    old: &DataFrame,
    new: &DataFrame,
    key_col: Option<&str>,
    strategy_map: Option<HashMap<String, serde_json::Value>>,
) -> AdditoryResult<DiffResult> {
    // Validate inputs
    if old.width() == 0 && new.width() == 0 {
        return Err(AdditoryError::Validation(
            "Both old and new DataFrames are empty.".to_string(),
            "Provide at least one non-empty DataFrame.".to_string(),
        ));
    }

    // Step 1: Parse strategy
    let strat = if let Some(ref map) = strategy_map {
        parse_strategy(map)?
    } else {
        StrategyConfig::default()
    };

    // Step 2: Apply aliases
    let (mut work_old, mut work_new) = if let Some(ref alias_source) = strat.aliases {
        match alias_source {
            AliasSource::Inline(alias_map) => {
                aliases::apply_aliases(old, new, alias_map)?
            }
            AliasSource::Registry(_name) => {
                // Registry resolution would require expression registry integration.
                // For now, pass through without renaming.
                (old.clone(), new.clone())
            }
        }
    } else {
        (old.clone(), new.clone())
    };

    // Step 3: Detect or validate key
    let key_cols = if let Some(k) = key_col {
        let cols: Vec<String> = k.split(',').map(|s| s.trim().to_string()).collect();
        key::validate_key(&work_old, &work_new, &cols)?;
        cols
    } else {
        key::detect_key(&work_old, &work_new)?
    };

    // Step 4: Handle duplicates
    let (cleaned_old, cleaned_new, dup_rows) =
        duplicates::handle_duplicates(&work_old, &work_new, &key_cols)?;
    work_old = cleaned_old;
    work_new = cleaned_new;

    // Step 5: Classify rows
    let mut result = classify::classify_rows(
        &work_old,
        &work_new,
        &key_cols,
        &strat.exclude,
        &strat.carry,
        &None, // groups resolution placeholder
    )?;

    // Attach duplicate rows from step 4
    result.duplicate_rows = dup_rows;

    Ok(result)
}
