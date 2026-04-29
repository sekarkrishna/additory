//! # Python Features Bridge
//!
//! PyO3 bridge to call Python-specific features from Rust
//!
//! This module provides two main functions:
//! 1. `resolve_expression()` - Resolve expression references (e.g., 'inbuilt:bmi')
//! 2. `knn_impute()` - Perform KNN imputation via Python implementation
//!
//! ## Architecture
//!
//! ```text
//! Rust @calc → resolve_expression() → Python expressions.loader → Resolved expression
//! Rust router → knn_impute() → Python transform.knn → Imputed DataFrame
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use pyo3::Python;
//! use crate::bindings::python_features;
//!
//! Python::with_gil(|py| {
//!     // Resolve expression reference
//!     let expr = python_features::resolve_expression(py, "inbuilt:bmi")?;
//!     // Returns: "weight / (height ** 2)"
//!     
//!     // Perform KNN imputation
//!     let result = python_features::knn_impute(
//!         py,
//!         &df_bytes,
//!         vec!["age".to_string(), "salary".to_string()],
//!         Some(strategy_map)
//!     )?;
//! });
//! ```

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
use std::collections::HashMap;

/// Resolve expression reference to actual expression string
///
/// This function calls the Python expression resolver to convert namespace
/// references like 'inbuilt:bmi' into actual expressions like 'weight / (height ** 2)'.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `reference` - Expression reference in format 'namespace:name'
///
/// # Returns
///
/// * `PyResult<String>` - Resolved expression string or error
///
/// # Errors
///
/// Returns error if:
/// - Python module cannot be imported
/// - Expression reference is not found
/// - Expression object doesn't have 'expression' attribute
///
/// # Example
///
/// ```rust,ignore
/// Python::with_gil(|py| {
///     let expr = resolve_expression(py, "inbuilt:bmi")?;
///     assert_eq!(expr, "weight / (height ** 2)");
/// });
/// ```
pub fn resolve_expression(py: Python, reference: &str) -> PyResult<String> {
    // Import Python expressions.loader module
    let expressions_module = py.import("expressions.loader")?;
    
    // Call resolve_expression function
    let resolve_fn = expressions_module.getattr("resolve_expression")?;
    let result = resolve_fn.call1((reference,))?;
    
    // Extract expression string from Expression object
    let expression: String = result.getattr("expression")?.extract()?;
    
    Ok(expression)
}

/// Perform KNN imputation on DataFrame
///
/// This function delegates to the Python KNN implementation, converting
/// DataFrames to/from Arrow IPC format for efficient transfer.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `df_bytes` - DataFrame serialized as Arrow IPC bytes
/// * `columns` - Column names to impute
/// * `strategy` - Optional strategy parameters (k, weights, metric)
///
/// # Returns
///
/// * `PyResult<Vec<u8>>` - Imputed DataFrame as Arrow IPC bytes
///
/// # Errors
///
/// Returns error if:
/// - Python module cannot be imported
/// - DataFrame conversion fails
/// - KNN imputation fails
/// - Result conversion fails
///
/// # Example
///
/// ```rust,ignore
/// Python::with_gil(|py| {
///     let mut strategy = HashMap::new();
///     strategy.insert("k".to_string(), py.eval("5", None, None)?.to_object(py));
///     
///     let result = knn_impute(
///         py,
///         &df_bytes,
///         vec!["age".to_string()],
///         Some(strategy)
///     )?;
/// });
/// ```
pub fn knn_impute(
    py: Python,
    df_bytes: &[u8],
    columns: Vec<String>,
    strategy: Option<HashMap<String, PyObject>>,
) -> PyResult<Vec<u8>> {
    // Import Python modules
    let knn_module = py.import("transform.knn")?;
    let polars_module = py.import("polars")?;
    let io_module = py.import("io")?;
    
    // Convert bytes to Python DataFrame
    let bytes_io = io_module.getattr("BytesIO")?;
    let buffer = bytes_io.call1((df_bytes,))?;
    let read_ipc = polars_module.getattr("read_ipc")?;
    let df = read_ipc.call1((buffer,))?;
    
    // Call perform_knn_imputation
    let knn_fn = knn_module.getattr("perform_knn_imputation")?;
    let result_df = if let Some(strat) = strategy {
        // Convert HashMap to PyDict
        let py_dict = PyDict::new(py);
        for (k, v) in strat {
            py_dict.set_item(k, v)?;
        }
        knn_fn.call((df, columns, py_dict), None)?
    } else {
        knn_fn.call((df, columns), None)?
    };
    
    // Convert result back to bytes
    let out_buffer = bytes_io.call0()?;
    let write_ipc = result_df.getattr("write_ipc")?;
    write_ipc.call1((out_buffer,))?;
    
    // Get bytes from buffer
    let result_bytes: Vec<u8> = out_buffer.getattr("getvalue")?.call0()?.extract()?;
    
    Ok(result_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore] // Requires Python environment
    fn test_resolve_expression_builtin() {
        Python::with_gil(|py| {
            let result = resolve_expression(py, "inbuilt:bmi");
            assert!(result.is_ok());
            let expr = result.unwrap();
            assert_eq!(expr, "weight / (height ** 2)");
        });
    }
    
    #[test]
    #[ignore] // Requires Python environment
    fn test_resolve_expression_not_found() {
        Python::with_gil(|py| {
            let result = resolve_expression(py, "inbuilt:nonexistent");
            assert!(result.is_err());
        });
    }
    
    #[test]
    #[ignore] // Requires Python environment
    fn test_knn_impute_basic() {
        Python::with_gil(|py| {
            // This would require setting up a test DataFrame
            // Skipping for now - will be tested in integration tests
        });
    }
}
