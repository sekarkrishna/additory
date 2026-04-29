//! Expression types for .add file parsing and resolution
//!
//! Defines the core data structures for expressions, reconciliations,
//! and .add file format detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reserved expression names that conflict with additory API methods.
pub const RESERVED_NAMES: &[&str] = &["to", "synthetic", "scan", "transform", "harmonize"];

/// Known function names excluded from identifier extraction.
/// These appear in formulas but are not column references.
pub const KNOWN_FUNCTIONS: &[&str] = &[
    "if_else", "today", "abs", "min", "max", "sum", "mean",
    "sqrt", "log", "exp", "round", "ceil", "floor", "pow",
];

/// Regex pattern for allowed characters in expression formulas.
/// Permits: alphanumeric, underscores, operators, parentheses, numbers,
/// whitespace, commas, dots, quotes.
pub const EXPRESSION_SAFE_PATTERN: &str = r#"^[A-Za-z0-9_\s+\-*/%()\.,<>=!^'"]+$"#;

/// Definition of a single expression input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDef {
    /// The type of the input (e.g. "numeric", "text", "date").
    pub type_name: String,
    /// The unit of the input (e.g. "kg", "m", "").
    pub unit: String,
    /// Human-readable description of the input.
    pub description: String,
}

/// A single expression definition parsed from an .add file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpressionDef {
    pub name: String,
    pub formula: String,
    pub description: String,
    pub category: String,
    pub output_column: String,
    pub inputs: HashMap<String, InputDef>,
    pub source_file: Option<String>,
}

/// A reconciliation definition parsed from an .add file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationDef {
    pub name: String,
    pub description: String,
    pub aliases: HashMap<String, Vec<String>>,
    pub groups: HashMap<String, Vec<String>>,
    pub source_file: Option<String>,
}

/// The parsed result from an .add file — either expressions or a reconciliation.
#[derive(Debug, Clone)]
pub enum ParsedAddFile {
    Expressions(Vec<ExpressionDef>),
    Reconciliation(ReconciliationDef),
}
