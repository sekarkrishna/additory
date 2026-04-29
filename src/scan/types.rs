//! Shared types for the scan module

use serde::Deserialize;
use std::collections::HashMap;

/// Scan mode enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ScanMode {
    /// Statistical profiling mode (both @analyze and @analyse map here)
    Analyze,
    /// Transformation tracking mode
    Lineage,
    /// Expression loading mode (handled in Python)
    Set,
}

impl ScanMode {
    /// Parse mode string to ScanMode enum
    pub fn parse_mode(s: &str) -> Result<Self, ScanError> {
        match s {
            "@analyze" | "@analyse" => Ok(ScanMode::Analyze),
            "@lineage" => Ok(ScanMode::Lineage),
            "@set" => Ok(ScanMode::Set),
            _ => Err(ScanError::InvalidMode(s.to_string())),
        }
    }
}

/// Output format enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    DataFrame,
    Dict,
    Text,
}

/// Scan output wrapper
#[derive(Debug)]
pub enum ScanOutput {
    /// Arrow IPC bytes
    DataFrame(Vec<u8>),
    /// JSON string
    Dict(String),
    /// UTF-8 text
    Text(String),
}

/// Row specification format
#[derive(Debug, Clone, PartialEq)]
pub enum RowSpec {
    /// "first:10"
    First(usize),
    /// "last:5"
    Last(usize),
    /// "25-50"
    Range(usize, usize),
    /// "61"
    Single(usize),
    /// "last"
    LastSingle,
}

/// Scan parameters
#[derive(Debug, Clone)]
pub struct ScanParams {
    pub mode: ScanMode,
    pub columns: Option<Vec<String>>,
    pub where_clause: Option<String>,
    pub rows: Option<Vec<RowSpec>>,
    pub trace: Option<(usize, usize)>,
    pub focus: Option<String>,
    pub as_type: OutputFormat,
    pub lineage_json: Option<String>,
}

/// Analyze-specific parameters
#[derive(Debug, Clone)]
pub struct AnalyzeParams {
    pub columns: Option<Vec<String>>,
    pub where_clause: Option<String>,
    pub rows: Option<Vec<RowSpec>>,
    pub focus: Option<AnalyzeFocus>,
}

/// Analyze focus modes
#[derive(Debug, Clone, PartialEq)]
pub enum AnalyzeFocus {
    Outliers,
    Correlations,
    Distributions,
    Nulls,
    Excluded,
}

/// Lineage-specific parameters
#[derive(Debug, Clone)]
pub struct LineageParams {
    pub columns: Option<Vec<String>>,
    pub where_clause: Option<String>,
    pub rows: Option<Vec<RowSpec>>,
    pub trace: Option<(usize, usize)>,
    pub focus: Option<LineageFocus>,
}

/// Lineage focus modes
#[derive(Debug, Clone, PartialEq)]
pub enum LineageFocus {
    Nulls,
    Excluded,
    Source(String),
}

/// Lineage metadata structure (parsed from JSON)
#[derive(Debug, Clone, Deserialize)]
pub struct LineageMetadata {
    pub operations: Vec<Operation>,
    pub column_sources: HashMap<String, ColumnSource>,
    pub metadata: MetadataInfo,
}

/// Single operation record
#[derive(Debug, Clone, Deserialize)]
pub struct Operation {
    pub operation_type: String,
    pub timestamp: String,
    pub rows_before: i64,
    pub rows_after: i64,
    pub columns_added: Vec<String>,
    pub columns_modified: Vec<String>,
    pub params: HashMap<String, serde_json::Value>,
}

/// Column source information
#[derive(Debug, Clone, Deserialize)]
pub struct ColumnSource {
    pub source_type: String,  // "original", "calculated", "fetched"
    pub source_table: Option<String>,
    pub source_column: Option<String>,
    pub formula: Option<String>,
    pub dependencies: Vec<String>,
}

/// Metadata information
#[derive(Debug, Clone, Deserialize)]
pub struct MetadataInfo {
    pub fresh_start: bool,
    pub sampling_applied: bool,
    pub compression_enabled: bool,
}

/// Scan error types
#[derive(Debug)]
pub enum ScanError {
    /// Invalid mode string
    InvalidMode(String),
    
    /// Invalid parameter value
    InvalidParameter { param: String, reason: String },
    
    /// DataFrame parsing failed
    DataFrameParseError(String),
    
    /// Lineage metadata missing
    MissingLineage,
    
    /// Lineage JSON parsing failed
    LineageParseError(String),
    
    /// Column not found in DataFrame
    ColumnNotFound(String),
    
    /// Invalid row specification
    InvalidRowSpec(String),
    
    /// Invalid trace coordinates
    InvalidTrace { col_idx: usize, row_idx: usize, reason: String },
    
    /// Focus mode not supported for this scan mode
    InvalidFocus { focus: String, mode: String },
    
    /// Output format conversion failed
    FormatConversionError(String),
    
    /// Statistical computation failed
    StatisticalError(String),
    
    /// Lineage report generation failed
    LineageError(String),
}

impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::InvalidMode(mode) => write!(
                f,
                "Error: Invalid Mode\n\nMode '{}' is not recognized. Valid modes are:\n  - '@analyze' or '@analyse': Statistical profiling\n  - '@lineage': Transformation tracking\n  - '@set': Expression loading (Python-side)\n\nExample:\n  result = add.scan('@analyze', df)",
                mode
            ),
            ScanError::InvalidParameter { param, reason } => write!(
                f,
                "Error: Invalid Parameter\n\nParameter '{}': {}\n\nPlease check the parameter value and try again.",
                param, reason
            ),
            ScanError::MissingLineage => write!(
                f,
                "Error: Missing Lineage Metadata\n\nNo lineage metadata found. Lineage tracking must be enabled by adding\nlineage=True to add.to(), add.transform(), or add.synthetic() calls.\n\nExample:\n  df = add.transform('@calc', df, strategy={{'total': 'price * qty'}}, lineage=True)\n  result = add.scan('@lineage', df)"
            ),
            ScanError::ColumnNotFound(col) => write!(
                f,
                "Error: Column Not Found\n\nColumn '{}' not found in DataFrame.\n\nPlease check the column name and try again.",
                col
            ),
            ScanError::InvalidTrace { col_idx, row_idx, reason } => write!(
                f,
                "Error: Invalid Trace Coordinates\n\nColumn index {} or row index {} is invalid: {}\n\nExample:\n  result = add.scan('@lineage', df, trace=[2, 5])  # Column 2, Row 5",
                col_idx, row_idx, reason
            ),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl std::error::Error for ScanError {}
