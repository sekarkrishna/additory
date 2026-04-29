//! Diff types for the diff engine.
//!
//! Defines DiffResult, ChangedRow, CellChange, StrategyConfig,
//! OutputMode, and AliasSource.

use std::collections::HashMap;
use polars::prelude::DataFrame;

/// The result of a diff operation between two DataFrames.
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Columns used as the primary key.
    pub key_cols: Vec<String>,
    /// Rows present in `new` but not in `old`.
    pub new_rows: DataFrame,
    /// Rows present in `old` but not in `new`.
    pub deleted_rows: DataFrame,
    /// Rows present in both but with value changes.
    pub changed_rows: Vec<ChangedRow>,
    /// Rows present in both with no changes.
    pub no_change_rows: DataFrame,
    /// Rows flagged as non-identical duplicates.
    pub duplicate_rows: DataFrame,
}

/// A single row that changed between old and new DataFrames.
#[derive(Debug, Clone)]
pub struct ChangedRow {
    /// Key column values identifying this row.
    pub key_values: HashMap<String, String>,
    /// Cell-level changes in this row.
    pub changes: Vec<CellChange>,
    /// All column values from the old row.
    pub old_row: HashMap<String, String>,
    /// All column values from the new row.
    pub new_row: HashMap<String, String>,
}

/// A single cell-level change within a changed row.
#[derive(Debug, Clone)]
pub struct CellChange {
    /// Column name where the change occurred.
    pub column: String,
    /// Value in the old DataFrame.
    pub old_value: String,
    /// Value in the new DataFrame.
    pub new_value: String,
    /// Whether this change is hierarchical (group-based).
    pub is_hierarchical: bool,
}

/// Parsed and validated strategy configuration for a diff operation.
#[derive(Debug, Clone)]
pub struct StrategyConfig {
    /// Output mode: summary or detail.
    pub output: OutputMode,
    /// Columns to exclude from comparison.
    pub exclude: Vec<String>,
    /// Columns to carry through without comparing.
    pub carry: Vec<String>,
    /// Columns to include as context in output.
    pub context: Vec<String>,
    /// Alias source for column renaming.
    pub aliases: Option<AliasSource>,
    /// Registry name for reconciliation groups.
    pub groups: Option<String>,
}

/// Output mode for diff results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputMode {
    Summary,
    Detail,
}

/// Source of alias mappings for column renaming.
#[derive(Debug, Clone)]
pub enum AliasSource {
    /// Inline alias mapping: canonical → variants.
    Inline(HashMap<String, Vec<String>>),
    /// Registry name to resolve aliases from a reconciliation .add file.
    Registry(String),
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            output: OutputMode::Summary,
            exclude: Vec::new(),
            carry: Vec::new(),
            context: Vec::new(),
            aliases: None,
            groups: None,
        }
    }
}
