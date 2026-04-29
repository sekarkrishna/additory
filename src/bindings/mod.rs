//! # Bindings Module
//!
//! Language bindings for additory
//!
//! This module provides bridges to Python-specific features via PyO3.

#[cfg(feature = "python")]
pub mod python_features;

#[cfg(feature = "python")]
pub mod expression_cache;

// Re-exports
#[cfg(feature = "python")]
pub use python_features::{resolve_expression, knn_impute};

#[cfg(feature = "python")]
pub use expression_cache::{get_cached_expression, cache_expression, clear_cache};
