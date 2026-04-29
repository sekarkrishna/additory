//! Type definitions for additory
//!
//! Core types used across all modules:
//! - Mode: Enum for all operation modes
//! - UniversalParams: Common parameters across functions
//! - FetchColumn: Column specification with optional rename

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::DataFrame;

/// Operation mode for add.to(), add.transform(), add.synthetic()
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    // add.to() modes
    Lookup,
    New,
    NewPolars,
    NewCudf,
    Merge,

    // add.transform() modes
    Filter,
    Transpose,
    Split,
    Extract,
    OneHot,
    Label,
    Calc,
    Aggregate,
    Sort,
    Knn,
    Harmonize,

    // add.synthetic() modes
    SyntheticNew,
    Augment,
    Analyze,
}

impl Mode {
    /// Parse mode string into Mode enum
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            // add.to()
            "lookup" | "LOOKUP" => Ok(Mode::Lookup),
            "@new" => Ok(Mode::New),
            "@new:polars" => Ok(Mode::NewPolars),
            "@new:cudf" => Ok(Mode::NewCudf),
            "@merge" => Ok(Mode::Merge),

            // add.transform()
            "@filter" => Ok(Mode::Filter),
            "@transpose" => Ok(Mode::Transpose),
            "@split" => Ok(Mode::Split),
            "@extract" => Ok(Mode::Extract),
            "@onehot" => Ok(Mode::OneHot),
            "@label" => Ok(Mode::Label),
            "@calc" => Ok(Mode::Calc),
            "@aggregate" => Ok(Mode::Aggregate),
            "@sort" => Ok(Mode::Sort),
            "@knn" => Ok(Mode::Knn),
            "@harmonize" => Ok(Mode::Harmonize),

            // add.synthetic()
            "@new" => Ok(Mode::SyntheticNew), // Context-dependent
            "augment" | "AUGMENT" => Ok(Mode::Augment),
            "@analyze" => Ok(Mode::Analyze),

            _ => Err(format!("Unknown mode: {}", s)),
        }
    }

    /// Get valid modes for a function
    pub fn valid_modes_for_function(function: &str) -> Vec<&'static str> {
        match function {
            "to" => vec!["LOOKUP", "@new", "@new:polars", "@new:cudf", "@merge"],
            "transform" => vec![
                "@filter", "@transpose", "@split", "@extract",
                "@onehot", "@label", "@calc", "@aggregate",
                "@sort", "@knn", "@harmonize",
            ],
            "synthetic" => vec!["@new", "augment", "@analyze"],
            _ => vec![],
        }
    }
}

/// Column specification with optional rename
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FetchColumn {
    /// Fetch column as-is (no rename)
    NoRename(String),
    /// Fetch column and rename (original, new)
    Rename(String, String),
}

impl FetchColumn {
    /// Get the original column name
    pub fn original(&self) -> &str {
        match self {
            FetchColumn::NoRename(name) => name,
            FetchColumn::Rename(original, _) => original,
        }
    }

    /// Get the target column name (after rename)
    pub fn target(&self) -> &str {
        match self {
            FetchColumn::NoRename(name) => name,
            FetchColumn::Rename(_, new) => new,
        }
    }

    /// Check if this is a rename operation
    pub fn is_rename(&self) -> bool {
        matches!(self, FetchColumn::Rename(_, _))
    }
}

/// Universal parameters used across functions
#[derive(Debug, Clone, Default)]
pub struct UniversalParams {
    /// Explicit mode override (e.g., "@sort", "@filter")
    pub explicit_mode: Option<String>,
    
    /// Columns to fetch/select
    pub fetch: Option<Vec<FetchColumn>>,
    
    /// Key/separator/grouping column
    pub by: Option<String>,
    
    /// Expression/calculation/components
    pub expression: Option<Expression>,
    
    /// Filter condition
    pub where_clause: Option<String>,
    
    /// Output name(s)/order
    pub as_param: Option<AsParam>,
    
    /// Position for new columns
    pub fetch_at: FetchAt,
    
    /// Advanced options
    pub strategy: Option<HashMap<String, StrategyValue>>,
    
    /// Enable detailed logging
    pub logging: bool,
    
    /// Reference DataFrame for add.to() operations
    pub reference: Option<DataFrame>,
    
    /// Number of rows to generate/add (for add.synthetic())
    pub n: Option<usize>,
    
    /// Column specifications for @new mode (for add.synthetic())
    pub fetch_specs: Option<HashMap<String, String>>,
}

/// Expression parameter (can be string, list, or dict)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Single(String),
    Multiple(Vec<String>),
    Dict(HashMap<String, String>),
}

/// As parameter (can be string or list)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AsParam {
    Single(String),
    Multiple(Vec<String>),
}

/// Position for new columns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FetchAt {
    Start,
    End,
    After(String),
    Before(String),
    Index(usize),
}

impl Default for FetchAt {
    fn default() -> Self {
        FetchAt::End
    }
}

impl FetchAt {
    /// Parse fetch_at string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "start" => Ok(FetchAt::Start),
            "end" => Ok(FetchAt::End),
            s if s.starts_with("after:") => {
                Ok(FetchAt::After(s.strip_prefix("after:").unwrap().to_string()))
            }
            s if s.starts_with("before:") => {
                Ok(FetchAt::Before(s.strip_prefix("before:").unwrap().to_string()))
            }
            s => s.parse::<usize>()
                .map(FetchAt::Index)
                .map_err(|_| format!("Invalid fetch_at value: {}", s)),
        }
    }
}

/// Strategy parameter value (can be various types)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StrategyValue {
    String(String),
    Number(f64),
    Bool(bool),
    List(Vec<String>),
    Dict(HashMap<String, StrategyValue>),
    Tuple(Vec<String>), // For add.to() column-specific strategy
}

/// DataFrame type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFrameType {
    Pandas,
    Polars,
    Cudf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_parsing() {
        assert_eq!(Mode::from_str("@filter").unwrap(), Mode::Filter);
        assert_eq!(Mode::from_str("@calc").unwrap(), Mode::Calc);
        assert_eq!(Mode::from_str("@new").unwrap(), Mode::New);
        assert!(Mode::from_str("@invalid").is_err());
    }

    #[test]
    fn test_fetch_column() {
        let no_rename = FetchColumn::NoRename("age".to_string());
        assert_eq!(no_rename.original(), "age");
        assert_eq!(no_rename.target(), "age");
        assert!(!no_rename.is_rename());

        let rename = FetchColumn::Rename("employee_name".to_string(), "name".to_string());
        assert_eq!(rename.original(), "employee_name");
        assert_eq!(rename.target(), "name");
        assert!(rename.is_rename());
    }

    #[test]
    fn test_fetch_at_parsing() {
        assert_eq!(FetchAt::from_str("start").unwrap(), FetchAt::Start);
        assert_eq!(FetchAt::from_str("end").unwrap(), FetchAt::End);
        assert_eq!(
            FetchAt::from_str("after:age").unwrap(),
            FetchAt::After("age".to_string())
        );
        assert_eq!(FetchAt::from_str("5").unwrap(), FetchAt::Index(5));
    }

    #[test]
    fn test_valid_modes() {
        let modes = Mode::valid_modes_for_function("transform");
        assert!(modes.contains(&"@filter"));
        assert!(modes.contains(&"@calc"));
        assert!(!modes.contains(&"@merge"));
    }
}
