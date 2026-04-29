//! Type definitions for additory v0.1.3a5
//!
//! Core types used across all modules:
//! - TransformMode, JoinType, SyntheticMode: Operation modes
//! - UniversalParams: Common parameters across functions
//! - FetchColumn: Column specification with optional rename (tuple format)
//! - Against, By: Single or tuple of keys/columns
//! - Position: Column insertion position
//! - AggregationMode: Mode with match modifier (mode:match syntax)
//! - StrategyValue: Recursive strategy values

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::DataFrame;

/// Transform mode for add.transform()
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformMode {
    Calc,           // @calc - Calculate expressions
    Filter,         // @filter - Filter rows and select columns
    Sort,           // @sort - Sort DataFrame
    Aggregate,      // @aggregate - Group by and aggregate
    Round,          // @round - Rounding (standard, up, down, banker)
    BankersRound,   // @bankers_round - Banker's rounding (legacy, maps to Round)
    Transpose,      // @transpose - Flip rows and columns
    OneHotEncode,   // @onehotencode - One-hot encoding
    Extract,        // @extract - Feature extraction
    Datetime,       // @datetime - Date parsing
    Harmonize,      // @harmonize - Unit conversion
    Knn,            // @knn - KNN imputation (legacy, use @deduce instead)
    Deduce,         // @deduce - Missing value imputation (7 methods)
    Label,          // @label - Label encoding
    Split,          // @split - Split text column
}

impl TransformMode {
    /// Parse mode string into TransformMode enum
    /// Handles sub-modes like @round:2, @round:banker, @roundup, @rounddown
    pub fn from_str(s: &str) -> Result<Self, String> {
        // Extract base mode (before colon)
        let base_mode = if s.contains(':') {
            s.split(':').next().unwrap()
        } else {
            s
        };
        
        match base_mode {
            "@calc" => Ok(TransformMode::Calc),
            "@filter" => Ok(TransformMode::Filter),
            "@sort" => Ok(TransformMode::Sort),
            "@aggregate" => Ok(TransformMode::Aggregate),
            "@round" | "@roundup" | "@rounddown" => Ok(TransformMode::Round),
            "@bankers_round" => Ok(TransformMode::BankersRound),
            "@transpose" => Ok(TransformMode::Transpose),
            "@onehot" | "@onehotencode" => Ok(TransformMode::OneHotEncode),
            "@extract" => Ok(TransformMode::Extract),
            // @datetime has been merged into @extract - redirect to Extract mode
            "@datetime" => Ok(TransformMode::Extract),
            "@harmonize" => Ok(TransformMode::Harmonize),
            "@knn" => Ok(TransformMode::Knn),
            "@deduce" => Ok(TransformMode::Deduce),
            "@label" => Ok(TransformMode::Label),
            "@split" => Ok(TransformMode::Split),
            _ => Err(format!("Unknown transform mode: {}", s)),
        }
    }

    /// Get all valid transform modes
    pub fn valid_modes() -> Vec<&'static str> {
        vec![
            "@calc", "@filter", "@sort", "@aggregate",
            "@round", "@roundup", "@rounddown", "@bankers_round",
            "@transpose", "@onehot", "@onehotencode",
            "@extract", "@harmonize", "@knn", "@deduce",
            "@label", "@split",
            // Note: @datetime has been merged into @extract
        ]
    }
}

/// Join type for add.to()
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum JoinType {
    #[default]
    Lookup,  // Default: 1:many or many:1 with aggregation
    Left,    // Left join (requires 1:1)
    Inner,   // Inner join (requires 1:1)
    Outer,   // Outer join (requires 1:1)
}


impl JoinType {
    /// Parse join type string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "lookup" => Ok(JoinType::Lookup),
            "left" => Ok(JoinType::Left),
            "inner" => Ok(JoinType::Inner),
            "outer" => Ok(JoinType::Outer),
            _ => Err(format!("Unknown join type: {}", s)),
        }
    }
}

/// Synthetic mode for add.synthetic()
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyntheticMode {
    New,      // @new - Create from scratch
    Augment,  // augment - Add to existing
    Analyze,  // @analyze - Analyze distribution
}

impl SyntheticMode {
    /// Parse synthetic mode string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "@new" => Ok(SyntheticMode::New),
            "@augment" => Ok(SyntheticMode::Augment),
            "@analyze" => Ok(SyntheticMode::Analyze),
            _ => Err(format!("Unknown synthetic mode: {}", s)),
        }
    }

    /// Get all valid synthetic modes
    pub fn valid_modes() -> Vec<&'static str> {
        vec!["@new", "@augment", "@analyze"]
    }
}

/// Column specification with optional rename (tuple format)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FetchColumn {
    /// Fetch column as-is (no rename)
    NoRename(String),
    /// Fetch column and rename using tuple format (source, target)
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

/// Join keys for add.to() (single or tuple)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Against {
    /// Single key
    Single(String),
    /// Multiple keys (tuple format)
    Multiple(Vec<String>),
}

impl Against {
    /// Create from single value
    pub fn from_value(value: String) -> Self {
        Against::Single(value)
    }
    
    /// Create from tuple of values
    pub fn from_tuple(values: Vec<String>) -> Self {
        Against::Multiple(values)
    }
    
    /// Get all keys as slice
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Against::Single(key) => vec![key.as_str()],
            Against::Multiple(keys) => keys.iter().map(|s| s.as_str()).collect(),
        }
    }
    
    /// Get number of keys
    pub fn len(&self) -> usize {
        match self {
            Against::Single(_) => 1,
            Against::Multiple(keys) => keys.len(),
        }
    }
    
    /// Check if empty (shouldn't happen, but for completeness)
    pub fn is_empty(&self) -> bool {
        match self {
            Against::Single(key) => key.is_empty(),
            Against::Multiple(keys) => keys.is_empty(),
        }
    }
}

/// Group by or sort by keys (single or tuple)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum By {
    /// Single column
    Single(String),
    /// Multiple columns (tuple format)
    Multiple(Vec<String>),
}

impl By {
    /// Create from single value
    pub fn from_value(value: String) -> Self {
        By::Single(value)
    }
    
    /// Create from tuple of values
    pub fn from_tuple(values: Vec<String>) -> Self {
        By::Multiple(values)
    }
    
    /// Get all columns as slice
    pub fn columns(&self) -> Vec<&str> {
        match self {
            By::Single(col) => vec![col.as_str()],
            By::Multiple(cols) => cols.iter().map(|s| s.as_str()).collect(),
        }
    }
    
    /// Get number of columns
    pub fn len(&self) -> usize {
        match self {
            By::Single(_) => 1,
            By::Multiple(cols) => cols.len(),
        }
    }
    
    /// Check if empty
    pub fn is_empty(&self) -> bool {
        match self {
            By::Single(col) => col.is_empty(),
            By::Multiple(cols) => cols.is_empty(),
        }
    }
}

/// Position for column insertion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum Position {
    /// Insert at start
    Start,
    /// Insert at end
    #[default]
    End,
    /// Insert after specified column
    After(String),
    /// Insert before specified column
    Before(String),
    /// Insert at specific index (supports negative indexing)
    Index(i32),
}


impl Position {
    /// Parse position string
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "start" => Ok(Position::Start),
            "end" => Ok(Position::End),
            s if s.starts_with("after:") => {
                let col = s.strip_prefix("after:").unwrap();
                Ok(Position::After(col.to_string()))
            }
            s if s.starts_with("before:") => {
                let col = s.strip_prefix("before:").unwrap();
                Ok(Position::Before(col.to_string()))
            }
            _ => Err(format!("Invalid position string: {}", s)),
        }
    }
    
    /// Create from integer index
    pub fn from_int(idx: i32) -> Self {
        Position::Index(idx)
    }
}

/// Aggregation mode with optional match modifier (mode:match syntax)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AggregationMode {
    /// Mode name (first, sum, concat, etc.)
    pub mode: String,
    /// Match modifier (anycase, trim, fuzzy, etc.) - defaults to "auto"
    pub match_modifier: String,
    /// Separator for concat mode
    pub separator: Option<String>,
}

impl AggregationMode {
    /// Parse mode string: 'first', 'first:anycase', 'concat[,]'
    pub fn from_str(s: &str) -> Result<Self, String> {
        // Check for concat with separator
        if s.starts_with("concat[") && s.ends_with(']') {
            let sep = &s[7..s.len()-1];
            let sep = sep.replace("\\n", "\n")
                        .replace("\\t", "\t")
                        .replace("\\r", "\r");
            return Ok(AggregationMode {
                mode: "concat".to_string(),
                match_modifier: "auto".to_string(),
                separator: Some(sep),
            });
        }
        
        // Check for concat without separator (default to pipe)
        if s == "concat" {
            return Ok(AggregationMode {
                mode: "concat".to_string(),
                match_modifier: "auto".to_string(),
                separator: Some("|".to_string()),
            });
        }
        
        // Check for mode:match syntax
        if s.contains(':') {
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            if parts.len() == 2 {
                return Ok(AggregationMode {
                    mode: parts[0].to_string(),
                    match_modifier: parts[1].to_string(),
                    separator: None,
                });
            }
        }
        
        // Simple mode only
        Ok(AggregationMode {
            mode: s.to_string(),
            match_modifier: "auto".to_string(),
            separator: None,
        })
    }
    
    /// Get valid aggregation modes
    pub fn valid_modes() -> Vec<&'static str> {
        vec![
            "auto", "strict", "first", "last", "shortest", "longest",
            "most_common", "forward_fill", "backward_fill",
            "sum", "count", "average", "min", "max",
            "concat",
        ]
    }
    
    /// Get valid match modifiers
    pub fn valid_match_modifiers() -> Vec<&'static str> {
        vec!["auto", "anycase", "fuzzy", "enforce_case", "trim"]
    }
}

/// Strategy value (can be string, number, bool, list, or nested dict)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StrategyValue {
    String(String),
    Number(f64),
    Bool(bool),
    NestedList(Vec<Vec<String>>),  // For linked lists - must come before List
    List(Vec<String>),
    Dict(HashMap<String, StrategyValue>),
    Mode(AggregationMode),  // Parsed mode:match syntax
}

/// Expression for @calc mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expression {
    /// Single expression
    Single(String),
    /// Multiple expressions
    Multiple(Vec<String>),
    /// Dict of expressions (column_name: expression)
    Dict(HashMap<String, String>),
}

/// Output column name(s) or sort order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsParam {
    /// Single output name
    Single(String),
    /// Multiple output names
    Multiple(Vec<String>),
    /// Sort order ('asc' or 'desc')
    SortOrder(String),
}

/// Universal parameters for all three functions
#[derive(Debug, Clone, Default)]
pub struct UniversalParams {
    // add.to() parameters
    /// Columns to fetch (with optional rename)
    pub fetch: Option<Vec<FetchColumn>>,
    /// Join keys (single or tuple)
    pub against: Option<Against>,
    /// Reference DataFrame (not serializable)
    pub reference: Option<DataFrame>,
    /// Join type (lookup, left, inner, outer)
    pub join_type: Option<JoinType>,
    /// Strategy for aggregation/rename
    pub strategy: Option<HashMap<String, StrategyValue>>,
    /// Position for new columns
    pub position: Option<Position>,
    
    // add.transform() parameters
    /// Original mode string (for parsing sub-modes like @round:2)
    pub mode_string: Option<String>,
    /// Expression(s) for @calc
    pub expression: Option<Expression>,
    /// Output name(s) or sort order (renamed from as_ in v0.1.3a10)
    pub name: Option<AsParam>,
    /// Group by or sort by columns
    pub by: Option<By>,
    /// Filter condition
    pub where_: Option<String>,
    /// Columns to operate on
    pub columns: Option<Vec<String>>,
    
    // @deduce mode parameters (v0.1.3a10+)
    /// Column(s) to fill with imputed/deduced values
    pub infer: Option<Vec<String>>,
    /// Text column(s) for TF-IDF similarity (renamed from 'against' to avoid conflict with add.to())
    pub against_text: Option<Vec<String>>,
    /// Imputation method(s) - 'mean', 'median', 'mode', 'forward', 'backward', 'interpolate', 'knn', 'tfidf'
    pub method: Option<Vec<String>>,
    
    // add.synthetic() parameters
    /// Number of rows to generate/add
    pub n: Option<usize>,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Column specifications for @new mode
    pub fetch_specs: Option<HashMap<String, String>>,
    
    // Common parameters
    /// Enable detailed logging
    pub logging: bool,
    /// Output DataFrame type ('polars' or 'pandas')
    pub as_type: Option<String>,
}

/// DataFrame type detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataFrameType {
    Pandas,
    Polars,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_mode_parsing() {
        assert_eq!(TransformMode::from_str("@calc").unwrap(), TransformMode::Calc);
        assert_eq!(TransformMode::from_str("@filter").unwrap(), TransformMode::Filter);
        assert_eq!(TransformMode::from_str("@sort").unwrap(), TransformMode::Sort);
        assert!(TransformMode::from_str("@invalid").is_err());
    }

    #[test]
    fn test_join_type_parsing() {
        assert_eq!(JoinType::from_str("lookup").unwrap(), JoinType::Lookup);
        assert_eq!(JoinType::from_str("left").unwrap(), JoinType::Left);
        assert_eq!(JoinType::from_str("INNER").unwrap(), JoinType::Inner);
        assert!(JoinType::from_str("invalid").is_err());
    }

    #[test]
    fn test_synthetic_mode_parsing() {
        assert_eq!(SyntheticMode::from_str("@new").unwrap(), SyntheticMode::New);
        assert_eq!(SyntheticMode::from_str("@augment").unwrap(), SyntheticMode::Augment);
        assert!(SyntheticMode::from_str("invalid").is_err());
    }

    #[test]
    fn test_fetch_column() {
        let no_rename = FetchColumn::NoRename("age".to_string());
        assert_eq!(no_rename.original(), "age");
        assert_eq!(no_rename.target(), "age");
        assert!(!no_rename.is_rename());

        let rename = FetchColumn::Rename("full_name".to_string(), "name".to_string());
        assert_eq!(rename.original(), "full_name");
        assert_eq!(rename.target(), "name");
        assert!(rename.is_rename());
    }

    #[test]
    fn test_against() {
        let single = Against::from_value("customer_id".to_string());
        assert_eq!(single.keys(), vec!["customer_id"]);
        assert_eq!(single.len(), 1);

        let multiple = Against::from_tuple(vec!["customer_id".to_string(), "date".to_string()]);
        assert_eq!(multiple.keys(), vec!["customer_id", "date"]);
        assert_eq!(multiple.len(), 2);
    }

    #[test]
    fn test_by() {
        let single = By::from_value("category".to_string());
        assert_eq!(single.columns(), vec!["category"]);
        assert_eq!(single.len(), 1);

        let multiple = By::from_tuple(vec!["category".to_string(), "region".to_string()]);
        assert_eq!(multiple.columns(), vec!["category", "region"]);
        assert_eq!(multiple.len(), 2);
    }

    #[test]
    fn test_position_parsing() {
        assert_eq!(Position::from_str("start").unwrap(), Position::Start);
        assert_eq!(Position::from_str("end").unwrap(), Position::End);
        assert_eq!(
            Position::from_str("after:age").unwrap(),
            Position::After("age".to_string())
        );
        assert_eq!(
            Position::from_str("before:name").unwrap(),
            Position::Before("name".to_string())
        );
        assert_eq!(Position::from_int(5), Position::Index(5));
        assert_eq!(Position::from_int(-1), Position::Index(-1));
    }

    #[test]
    fn test_aggregation_mode_parsing() {
        // Simple mode
        let mode = AggregationMode::from_str("first").unwrap();
        assert_eq!(mode.mode, "first");
        assert_eq!(mode.match_modifier, "auto");
        assert_eq!(mode.separator, None);

        // Mode with match
        let mode = AggregationMode::from_str("first:anycase").unwrap();
        assert_eq!(mode.mode, "first");
        assert_eq!(mode.match_modifier, "anycase");
        assert_eq!(mode.separator, None);

        // Concat with default separator
        let mode = AggregationMode::from_str("concat").unwrap();
        assert_eq!(mode.mode, "concat");
        assert_eq!(mode.separator, Some("|".to_string()));

        // Concat with custom separator
        let mode = AggregationMode::from_str("concat[,]").unwrap();
        assert_eq!(mode.mode, "concat");
        assert_eq!(mode.separator, Some(",".to_string()));

        // Concat with newline
        let mode = AggregationMode::from_str("concat[\\n]").unwrap();
        assert_eq!(mode.mode, "concat");
        assert_eq!(mode.separator, Some("\n".to_string()));
    }

    #[test]
    fn test_valid_modes() {
        let modes = TransformMode::valid_modes();
        assert!(modes.contains(&"@calc"));
        assert!(modes.contains(&"@filter"));
        assert_eq!(modes.len(), 17);

        let modes = SyntheticMode::valid_modes();
        assert!(modes.contains(&"@new"));
        assert!(modes.contains(&"@augment"));
        assert_eq!(modes.len(), 3);
    }
}
