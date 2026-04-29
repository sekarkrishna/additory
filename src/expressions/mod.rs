//! Expression registry module — parse, resolve, format, and scan `.add` files.
//!
//! This module is the Rust-native implementation of the expression registry,
//! porting the Python `loader.py` logic. It supports two `.add` file formats:
//!
//! - **Unified**: top-level `[name]` tables with `expression`, `description`, `category`
//! - **Reconciliation**: `[reconciliation]`, `[aliases]`, `[groups]` sections

pub mod types;
pub mod identifiers;
pub mod parser;
pub mod formatter;
pub mod scanner;

// Re-export public API
pub use types::{
    ExpressionDef, InputDef, ParsedAddFile, ReconciliationDef,
    EXPRESSION_SAFE_PATTERN, KNOWN_FUNCTIONS, RESERVED_NAMES,
};
pub use identifiers::extract_identifiers;
pub use parser::parse_add_file_content;
pub use formatter::{format_expression, format_reconciliation};
pub use scanner::{
    list_expression_names, resolve_expression_by_name, resolve_reconciliation_by_name,
    scan_folder_for_expressions,
};
