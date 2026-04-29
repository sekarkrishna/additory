//! DataFrame type detection utilities
//!
//! Detects whether a DataFrame is Pandas, Polars, or cuDF.
//! Used for automatic type conversion and preservation.

use crate::core::types::DataFrameType;

/// Detect DataFrame type from Python object
///
/// This will be implemented in the Python bindings layer.
/// For now, we provide the interface.
pub fn detect_dataframe_type(_py_obj: &str) -> DataFrameType {
    // Placeholder - actual implementation in Python bindings
    DataFrameType::Polars
}

/// Check if type conversion is needed
pub fn needs_conversion(from: DataFrameType, to: DataFrameType) -> bool {
    from != to
}

/// Get type name as string
pub fn type_name(df_type: DataFrameType) -> &'static str {
    match df_type {
        DataFrameType::Pandas => "Pandas",
        DataFrameType::Polars => "Polars",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_conversion() {
        assert!(!needs_conversion(DataFrameType::Polars, DataFrameType::Polars));
        assert!(needs_conversion(DataFrameType::Pandas, DataFrameType::Polars));
        assert!(needs_conversion(DataFrameType::Polars, DataFrameType::Pandas));
    }

    #[test]
    fn test_type_name() {
        assert_eq!(type_name(DataFrameType::Pandas), "Pandas");
        assert_eq!(type_name(DataFrameType::Polars), "Polars");
    }
}
