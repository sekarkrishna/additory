//! # additory
//!
//! Elegant data operations for DataFrames
//!
//! Three functions only:
//! - `add.to()` - Add data FROM external source
//! - `add.transform()` - Transform data WITHIN DataFrame  
//! - `add.synthetic()` - Create or augment with synthetic data

// Core modules
pub mod core;
pub mod utils;

// Function modules
pub mod to;
pub mod transform;
pub mod synthetic;
pub mod scan;

// Expression registry
pub mod expressions;

// Configuration system
pub mod config;

// Diff engine
pub mod diff;

// Validation and logging modules
pub mod validation;
pub mod logging;

// Re-export main types
pub use core::{DataFrame, AdditoryResult, AdditoryError};

// Version constant
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// PyO3 module for Python bindings
#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn _additory(_py: Python, m: &PyModule) -> PyResult<()> {
    use crate::core::types::{UniversalParams, Expression, AsParam, FetchColumn};
    use std::collections::HashMap;
    use std::io::Cursor;
    use polars::io::ipc::{IpcReader, IpcWriter};
    use polars::prelude::{SerReader, SerWriter};
    
    m.add("__version__", VERSION)?;
    
    // Add transform function wrapper
    #[pyfn(m)]
    fn transform<'py>(
        py: Python<'py>,
        df_bytes: &[u8],
        params_dict: HashMap<String, PyObject>,
    ) -> PyResult<&'py pyo3::types::PyBytes> {
        // Parse DataFrame from Arrow IPC bytes
        let cursor = Cursor::new(df_bytes);
        let polars_df = IpcReader::new(cursor).finish()
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Failed to parse DataFrame: {}", e)
            ))?;
        let df = DataFrame::from_polars(polars_df);
        
        // Parse parameters from Python dict
        let mut params = UniversalParams::default();
        
        // Parse mode (required)
        let mode = if let Some(mode_obj) = params_dict.get("mode") {
            if !mode_obj.is_none(py) {
                mode_obj.extract::<String>(py)?
            } else {
                return Err(pyo3::exceptions::PyValueError::new_err("mode is required"));
            }
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err("mode is required"));
        };
        
        // Store the original mode string for parsing sub-modes
        params.mode_string = Some(mode.clone());
        
        let transform_mode = crate::core::TransformMode::from_str(&mode)
            .map_err(pyo3::exceptions::PyValueError::new_err)?;
        
        // Parse expression (using 'on' parameter name for compatibility with v0.1.3a4)
        if let Some(expr_obj) = params_dict.get("on") {
            if !expr_obj.is_none(py) {
                if let Ok(expr_str) = expr_obj.extract::<String>(py) {
                    params.expression = Some(Expression::Single(expr_str));
                } else if let Ok(expr_list) = expr_obj.extract::<Vec<String>>(py) {
                    params.expression = Some(Expression::Multiple(expr_list));
                }
            }
        }
        
        // Parse as parameter (maps to 'name' field in UniversalParams)
        if let Some(as_obj) = params_dict.get("as") {
            if !as_obj.is_none(py) {
                if let Ok(as_str) = as_obj.extract::<String>(py) {
                    params.name = Some(AsParam::Single(as_str));
                } else if let Ok(as_list) = as_obj.extract::<Vec<String>>(py) {
                    params.name = Some(AsParam::Multiple(as_list));
                }
            }
        }
        
        // Parse fetch parameter
        if let Some(fetch_obj) = params_dict.get("fetch") {
            if !fetch_obj.is_none(py) {
                if let Ok(fetch_list) = fetch_obj.extract::<Vec<String>>(py) {
                    params.fetch = Some(fetch_list.into_iter().map(FetchColumn::NoRename).collect());
                }
            }
        }
        
        // Parse by parameter
        if let Some(by_obj) = params_dict.get("by") {
            if !by_obj.is_none(py) {
                if let Ok(by_str) = by_obj.extract::<String>(py) {
                    params.by = Some(crate::core::By::Single(by_str));
                } else if let Ok(by_list) = by_obj.extract::<Vec<String>>(py) {
                    params.by = Some(crate::core::By::Multiple(by_list));
                }
            }
        }
        
        // Parse where parameter
        if let Some(where_obj) = params_dict.get("where") {
            if !where_obj.is_none(py) {
                if let Ok(where_str) = where_obj.extract::<String>(py) {
                    params.where_ = Some(where_str);
                }
            }
        }
        
        // Parse columns parameter (CRITICAL FIX)
        if let Some(columns_obj) = params_dict.get("columns") {
            if !columns_obj.is_none(py) {
                if let Ok(columns_list) = columns_obj.extract::<Vec<String>>(py) {
                    params.columns = Some(columns_list);
                } else if let Ok(columns_dict) = columns_obj.extract::<HashMap<String, PyObject>>(py) {
                    // For @extract and @harmonize, columns can be a dict
                    // For now, just extract the keys as column names
                    params.columns = Some(columns_dict.keys().cloned().collect());
                }
            }
        }
        
        // Parse strategy parameter
        if let Some(strategy_obj) = params_dict.get("strategy") {
            if !strategy_obj.is_none(py) {
                if let Ok(strategy_dict) = strategy_obj.extract::<HashMap<String, PyObject>>(py) {
                    let mut strategy_map = HashMap::new();
                    for (k, v) in strategy_dict {
                        if let Ok(strategy_value) = parse_strategy_value(py, &v) {
                            strategy_map.insert(k, strategy_value);
                        }
                    }
                    params.strategy = Some(strategy_map);
                }
            }
        }
        
        // Parse @deduce mode parameters (v0.1.3a10+)
        // Parse infer parameter (column(s) to fill)
        if let Some(infer_obj) = params_dict.get("infer") {
            if !infer_obj.is_none(py) {
                if let Ok(infer_list) = infer_obj.extract::<Vec<String>>(py) {
                    params.infer = Some(infer_list);
                }
            }
        }
        
        // Parse against parameter (text columns for TF-IDF)
        // Note: This is different from add.to()'s 'against' parameter (join keys)
        // For @deduce mode, 'against' specifies text columns for similarity
        if let Some(against_obj) = params_dict.get("against") {
            if !against_obj.is_none(py) {
                if let Ok(against_list) = against_obj.extract::<Vec<String>>(py) {
                    params.against_text = Some(against_list);
                }
            }
        }
        
        // Parse method parameter (imputation method(s))
        if let Some(method_obj) = params_dict.get("method") {
            if !method_obj.is_none(py) {
                if let Ok(method_list) = method_obj.extract::<Vec<String>>(py) {
                    params.method = Some(method_list);
                }
            }
        }
        
        // Parse logging parameter
        let logging_level = if let Some(logging_obj) = params_dict.get("logging") {
            if let Ok(logging_bool) = logging_obj.extract::<bool>(py) {
                params.logging = logging_bool;
                if logging_bool {
                    crate::logging::LogLevel::Full
                } else {
                    crate::logging::LogLevel::Off
                }
            } else {
                crate::logging::LogLevel::Default
            }
        } else {
            crate::logging::LogLevel::Default
        };
        
        // Validation: Call ValidationOrchestrator before expensive operations
        let orchestrator = crate::validation::ValidationOrchestrator::with_default_config();
        let columns_vec = params.columns.clone().unwrap_or_default();
        let params_map = std::collections::HashMap::new(); // Empty for now, can be extended
        
        orchestrator.validate_add_transform(
            df.inner(),
            &mode,
            &columns_vec,
            &params_map,
            logging_level,
        ).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
            format!("Validation failed: {}", e)
        ))?;
        
        // Call transform function
        let result_df = crate::transform::transform(df, transform_mode, params)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Transform failed: {}", e)
            ))?;
        
        // Convert result to Arrow IPC bytes
        let polars_df = result_df.into_inner();
        let mut buf = Vec::new();
        IpcWriter::new(&mut buf)
            .finish(&mut polars_df.clone())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Failed to serialize result: {}", e)
            ))?;
        
        Ok(pyo3::types::PyBytes::new(py, &buf))
    }
    
    // Add to function wrapper
    #[pyfn(m)]
    #[pyo3(signature = (target_df_bytes, params_dict))]
    fn to<'py>(
        py: Python<'py>,
        target_df_bytes: Option<&[u8]>,
        params_dict: HashMap<String, PyObject>,
    ) -> PyResult<&'py pyo3::types::PyBytes> {
        // Parse target DataFrame
        let target = if let Some(bytes) = target_df_bytes {
            let cursor = Cursor::new(bytes);
            let polars_df = IpcReader::new(cursor).finish()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                    format!("Failed to parse target DataFrame: {}", e)
                ))?;
            Some(DataFrame::from_polars(polars_df))
        } else {
            None
        };
        
        // Parse parameters
        let mut params = UniversalParams::default();
        
        // Parse fetch_from (reference DataFrame)
        if let Some(fetch_from_obj) = params_dict.get("fetch_from") {
            if !fetch_from_obj.is_none(py) {
                if let Ok(ref_bytes) = fetch_from_obj.extract::<&[u8]>(py) {
                    let cursor = Cursor::new(ref_bytes);
                    let polars_df = IpcReader::new(cursor).finish()
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                            format!("Failed to parse fetch_from DataFrame: {}", e)
                        ))?;
                    params.reference = Some(DataFrame::from_polars(polars_df));
                }
            }
        }
        
        // Parse fetch parameter
        if let Some(fetch_obj) = params_dict.get("fetch") {
            if !fetch_obj.is_none(py) {
                let fetch_cols = parse_fetch_parameter(py, fetch_obj)?;
                params.fetch = Some(fetch_cols);
            }
        }
        
        // Parse against parameter
        if let Some(against_obj) = params_dict.get("against") {
            if !against_obj.is_none(py) {
                if let Ok(against_str) = against_obj.extract::<String>(py) {
                    params.against = Some(crate::core::Against::Single(against_str));
                } else if let Ok(against_list) = against_obj.extract::<Vec<String>>(py) {
                    params.against = Some(crate::core::Against::Multiple(against_list));
                }
            }
        }
        
        // Parse by parameter
        if let Some(by_obj) = params_dict.get("by") {
            if !by_obj.is_none(py) {
                if let Ok(by_str) = by_obj.extract::<String>(py) {
                    params.by = Some(crate::core::By::Single(by_str));
                } else if let Ok(by_list) = by_obj.extract::<Vec<String>>(py) {
                    params.by = Some(crate::core::By::Multiple(by_list));
                }
            }
        }
        
        // Parse position parameter
        if let Some(position_obj) = params_dict.get("position") {
            if !position_obj.is_none(py) {
                if let Ok(pos_str) = position_obj.extract::<String>(py) {
                    params.position = Some(crate::core::Position::from_str(&pos_str)
                        .map_err(pyo3::exceptions::PyValueError::new_err)?);
                } else if let Ok(pos_int) = position_obj.extract::<i32>(py) {
                    params.position = Some(crate::core::Position::from_int(pos_int));
                }
            }
        }
        
        // Parse strategy parameter
        if let Some(strategy_obj) = params_dict.get("strategy") {
            if !strategy_obj.is_none(py) {
                if let Ok(strategy_dict) = strategy_obj.extract::<HashMap<String, PyObject>>(py) {
                    let mut strategy_map = HashMap::new();
                    for (k, v) in strategy_dict {
                        if let Ok(strategy_value) = parse_strategy_value(py, &v) {
                            strategy_map.insert(k, strategy_value);
                        }
                    }
                    params.strategy = Some(strategy_map);
                }
            }
        }
        
        // Parse join_type parameter
        if let Some(join_type_obj) = params_dict.get("join_type") {
            if !join_type_obj.is_none(py) {
                if let Ok(join_str) = join_type_obj.extract::<String>(py) {
                    params.join_type = Some(crate::core::JoinType::from_str(&join_str)
                        .map_err(pyo3::exceptions::PyValueError::new_err)?);
                }
            }
        }
        
        // Parse logging parameter
        let logging_level = if let Some(logging_obj) = params_dict.get("logging") {
            if let Ok(logging_bool) = logging_obj.extract::<bool>(py) {
                params.logging = logging_bool;
                if logging_bool {
                    crate::logging::LogLevel::Full
                } else {
                    crate::logging::LogLevel::Off
                }
            } else {
                crate::logging::LogLevel::Default
            }
        } else {
            crate::logging::LogLevel::Default
        };
        
        // Validation: Call ValidationOrchestrator before expensive operations
        let target_df = target.ok_or_else(|| pyo3::exceptions::PyValueError::new_err(
            "target DataFrame is required for add.to()"
        ))?;
        
        if let Some(ref reference_df) = params.reference {
            // Extract join keys from parameters
            let left_keys = if let Some(ref by) = params.by {
                match by {
                    crate::core::By::Single(s) => vec![s.clone()],
                    crate::core::By::Multiple(v) => v.clone(),
                }
            } else {
                vec![]
            };
            
            let right_keys = if let Some(ref against) = params.against {
                match against {
                    crate::core::Against::Single(s) => vec![s.clone()],
                    crate::core::Against::Multiple(v) => v.clone(),
                }
            } else {
                left_keys.clone() // Use same keys if against not specified
            };
            
            // Extract fetch columns
            let fetch_cols = params.fetch.as_ref().map(|f| {
                f.iter().map(|fc| match fc {
                    crate::core::types::FetchColumn::NoRename(s) => s.clone(),
                    crate::core::types::FetchColumn::Rename(s, _) => s.clone(),
                }).collect::<Vec<_>>()
            });
            
            // Extract join type
            let join_type_str = if let Some(ref jt) = params.join_type {
                match jt {
                    crate::core::JoinType::Lookup => "left", // Map Lookup to left for validation
                    crate::core::JoinType::Left => "left",
                    crate::core::JoinType::Inner => "inner",
                    crate::core::JoinType::Outer => "outer",
                }
            } else {
                "left" // Default
            };
            
            // Extract strategy
            let strategy_map = params.strategy.as_ref().map(|s| {
                s.iter().map(|(k, v)| {
                    let v_str = match v {
                        crate::core::types::StrategyValue::String(s) => s.clone(),
                        _ => "first".to_string(), // Default for non-string values
                    };
                    (k.clone(), v_str)
                }).collect::<std::collections::HashMap<_, _>>()
            });
            
            let orchestrator = crate::validation::ValidationOrchestrator::with_default_config();
            orchestrator.validate_add_to(
                target_df.inner(),
                reference_df.inner(),
                &right_keys,  // Fixed: use right_keys (from 'against') not left_keys (from 'by')
                &fetch_cols,
                join_type_str,
                &strategy_map,
                logging_level,
            ).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Validation failed: {}", e)
            ))?;
        }
        
        // Call to function
        let result_df = crate::to::to(target_df, params)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("add.to() failed: {}", e)
            ))?;
        
        // Convert result to Arrow IPC bytes
        let polars_df = result_df.into_inner();
        let mut buf = Vec::new();
        IpcWriter::new(&mut buf)
            .finish(&mut polars_df.clone())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Failed to serialize result: {}", e)
            ))?;
        
        Ok(pyo3::types::PyBytes::new(py, &buf))
    }
    
    // Add synthetic function wrapper
    #[pyfn(m)]
    #[pyo3(signature = (df_bytes, params_dict))]
    fn synthetic<'py>(
        py: Python<'py>,
        df_bytes: Option<&[u8]>,
        params_dict: HashMap<String, PyObject>,
    ) -> PyResult<&'py pyo3::types::PyBytes> {
        // Parse optional DataFrame (None for @new mode)
        let df = if let Some(bytes) = df_bytes {
            let cursor = Cursor::new(bytes);
            let polars_df = IpcReader::new(cursor).finish()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                    format!("Failed to parse DataFrame: {}", e)
                ))?;
            Some(DataFrame::from_polars(polars_df))
        } else {
            None
        };
        
        // Parse parameters
        let mut params = UniversalParams::default();
        
        // Parse mode string
        let mode_str = if let Some(mode_obj) = params_dict.get("mode") {
            if !mode_obj.is_none(py) {
                Some(mode_obj.extract::<String>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse n (number of rows)
        if let Some(n_obj) = params_dict.get("n") {
            if !n_obj.is_none(py) {
                if let Ok(n_val) = n_obj.extract::<usize>(py) {
                    params.n = Some(n_val);
                }
            }
        }
        
        // Parse seed parameter (CRITICAL FIX for reproducibility)
        if let Some(seed_obj) = params_dict.get("seed") {
            if !seed_obj.is_none(py) {
                if let Ok(seed_val) = seed_obj.extract::<u64>(py) {
                    params.seed = Some(seed_val);
                }
            }
        }
        
        // Parse strategy parameter
        if let Some(strategy_obj) = params_dict.get("strategy") {
            if !strategy_obj.is_none(py) {
                if let Ok(strategy_dict) = strategy_obj.extract::<HashMap<String, PyObject>>(py) {
                    let mut strategy_map = HashMap::new();
                    for (k, v) in strategy_dict {
                        if let Ok(strategy_value) = parse_strategy_value(py, &v) {
                            strategy_map.insert(k, strategy_value);
                        }
                    }
                    params.strategy = Some(strategy_map);
                }
            }
        }
        
        // Parse logging parameter
        let logging_level = if let Some(logging_obj) = params_dict.get("logging") {
            if let Ok(logging_bool) = logging_obj.extract::<bool>(py) {
                params.logging = logging_bool;
                if logging_bool {
                    crate::logging::LogLevel::Full
                } else {
                    crate::logging::LogLevel::Off
                }
            } else {
                crate::logging::LogLevel::Default
            }
        } else {
            crate::logging::LogLevel::Default
        };
        
        // Validation: Call ValidationOrchestrator before expensive operations
        if let Some(ref dataframe) = df {
            let strategy_str = mode_str.as_deref().unwrap_or("");
            let params_map = std::collections::HashMap::new(); // Empty for now, can be extended
            
            let orchestrator = crate::validation::ValidationOrchestrator::with_default_config();
            orchestrator.validate_add_synthetic(
                dataframe.inner(),
                strategy_str,
                &params_map,
                logging_level,
            ).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Validation failed: {}", e)
            ))?;
        }
        
        // Call synthetic function
        let result_df = crate::synthetic::synthetic(
            mode_str.as_deref(),
            df,
            params
        ).map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
            format!("add.synthetic() failed: {}", e)
        ))?;
        
        // Convert result to Arrow IPC bytes
        let polars_df = result_df.into_inner();
        let mut buf = Vec::new();
        IpcWriter::new(&mut buf)
            .finish(&mut polars_df.clone())
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Failed to serialize result: {}", e)
            ))?;
        
        Ok(pyo3::types::PyBytes::new(py, &buf))
    }
    
    // Add scan function wrapper
    #[pyfn(m)]
    fn scan<'py>(
        py: Python<'py>,
        df_bytes: &[u8],
        params_dict: HashMap<String, PyObject>,
    ) -> PyResult<PyObject> {
        use crate::scan::{ScanParams, ScanMode, OutputFormat, ScanOutput};
        
        // Parse mode (required)
        let mode_str = if let Some(mode_obj) = params_dict.get("mode") {
            if !mode_obj.is_none(py) {
                mode_obj.extract::<String>(py)?
            } else {
                return Err(pyo3::exceptions::PyValueError::new_err("mode is required"));
            }
        } else {
            return Err(pyo3::exceptions::PyValueError::new_err("mode is required"));
        };
        
        let mode = ScanMode::parse_mode(&mode_str)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        
        // Parse columns parameter (optional)
        let columns = if let Some(columns_obj) = params_dict.get("columns") {
            if !columns_obj.is_none(py) {
                Some(columns_obj.extract::<Vec<String>>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse where parameter (optional)
        let where_clause = if let Some(where_obj) = params_dict.get("where") {
            if !where_obj.is_none(py) {
                Some(where_obj.extract::<String>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse rows parameter (optional)
        let _rows = if let Some(rows_obj) = params_dict.get("rows") {
            if !rows_obj.is_none(py) {
                Some(rows_obj.extract::<Vec<String>>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse trace parameter (optional)
        let trace = if let Some(trace_obj) = params_dict.get("trace") {
            if !trace_obj.is_none(py) {
                let trace_list = trace_obj.extract::<Vec<usize>>(py)?;
                if trace_list.len() == 2 {
                    Some((trace_list[0], trace_list[1]))
                } else {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "trace must be a list of 2 integers [col_idx, row_idx]"
                    ));
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse focus parameter (optional)
        let focus = if let Some(focus_obj) = params_dict.get("focus") {
            if !focus_obj.is_none(py) {
                Some(focus_obj.extract::<String>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Parse as_type parameter (optional)
        let as_type_str = if let Some(as_type_obj) = params_dict.get("as_type") {
            if !as_type_obj.is_none(py) {
                Some(as_type_obj.extract::<String>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Determine output format with defaults
        let as_type = match as_type_str.as_deref() {
            Some("dataframe") => OutputFormat::DataFrame,
            Some("dict") => OutputFormat::Dict,
            Some("text") => OutputFormat::Text,
            None => {
                // Default: DataFrame for analyze, Text for lineage
                match mode {
                    ScanMode::Analyze => OutputFormat::DataFrame,
                    ScanMode::Lineage => OutputFormat::Text,
                    ScanMode::Set => OutputFormat::Text, // unreachable: handled in Python
                }
            }
            Some(other) => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    format!("Invalid as_type: '{}'. Valid values: 'dataframe', 'dict', 'text'", other)
                ));
            }
        };
        
        // Parse lineage_json parameter (optional, required for lineage mode)
        let lineage_json = if let Some(lineage_obj) = params_dict.get("lineage_json") {
            if !lineage_obj.is_none(py) {
                Some(lineage_obj.extract::<String>(py)?)
            } else {
                None
            }
        } else {
            None
        };
        
        // Build ScanParams
        let params = ScanParams {
            mode,
            columns,
            where_clause,
            rows: None, // TODO: Parse row specs
            trace,
            focus,
            as_type,
            lineage_json,
        };
        
        // Execute scan
        let logger = crate::utils::logging::Logger::new(false);
        let result = crate::scan::execute_scan(df_bytes, params, &logger)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(
                format!("Scan failed: {}", e)
            ))?;
        
        // Convert result to Python object
        match result {
            ScanOutput::DataFrame(bytes) => {
                Ok(pyo3::types::PyBytes::new(py, &bytes).into())
            }
            ScanOutput::Dict(json_str) => {
                // Parse JSON string to Python dict
                let json_module = py.import("json")?;
                let loads_fn = json_module.getattr("loads")?;
                let dict = loads_fn.call1((json_str,))?;
                Ok(dict.into())
            }
            ScanOutput::Text(text) => {
                Ok(text.into_py(py))
            }
        }
    }
    
    Ok(())
}

/// Helper function to parse fetch parameter from Python
#[cfg(feature = "python")]
fn parse_fetch_parameter(
    py: Python,
    fetch_obj: &pyo3::PyObject,
) -> PyResult<Vec<crate::core::types::FetchColumn>> {
    use crate::core::types::FetchColumn;
    
    // Try to extract as list
    if let Ok(fetch_list) = fetch_obj.extract::<Vec<pyo3::PyObject>>(py) {
        let mut result = Vec::new();
        
        for item in fetch_list {
            // Check if it's a string (no rename)
            if let Ok(col_name) = item.extract::<String>(py) {
                result.push(FetchColumn::NoRename(col_name));
            }
            // Check if it's a tuple (rename)
            else if let Ok(tuple) = item.extract::<(String, String)>(py) {
                result.push(FetchColumn::Rename(tuple.0, tuple.1));
            }
            else {
                return Err(pyo3::exceptions::PyTypeError::new_err(
                    "fetch items must be strings or (old_name, new_name) tuples"
                ));
            }
        }
        
        Ok(result)
    } else {
        Err(pyo3::exceptions::PyTypeError::new_err(
            "fetch must be a list of strings or tuples"
        ))
    }
}

/// Helper function to recursively parse StrategyValue from Python objects
#[cfg(feature = "python")]
fn parse_strategy_value(
    py: Python,
    obj: &pyo3::PyObject,
) -> PyResult<crate::core::types::StrategyValue> {
    use crate::core::types::StrategyValue;
    use std::collections::HashMap;
    
    // Try string first
    if let Ok(s) = obj.extract::<String>(py) {
        return Ok(StrategyValue::String(s));
    }
    
    // Try bool before number (bool is a subtype of int in Python)
    if let Ok(b) = obj.extract::<bool>(py) {
        return Ok(StrategyValue::Bool(b));
    }
    
    // Try number (handles both int and float)
    if let Ok(n) = obj.extract::<f64>(py) {
        return Ok(StrategyValue::Number(n));
    }
    
    // Try nested list (Vec<Vec<String>>) for linked lists - MUST come before flat list
    if let Ok(nested) = obj.extract::<Vec<Vec<String>>>(py) {
        return Ok(StrategyValue::NestedList(nested));
    }
    
    // Try list of strings
    if let Ok(l) = obj.extract::<Vec<String>>(py) {
        return Ok(StrategyValue::List(l));
    }
    
    // Try nested dict (recursive)
    if let Ok(dict) = obj.extract::<HashMap<String, pyo3::PyObject>>(py) {
        let mut nested_map = HashMap::new();
        for (k, v) in dict {
            let nested_value = parse_strategy_value(py, &v)?;
            nested_map.insert(k, nested_value);
        }
        return Ok(StrategyValue::Dict(nested_map));
    }
    
    // If nothing matches, skip this value
    Err(pyo3::exceptions::PyTypeError::new_err(
        format!("Unsupported strategy value type: {:?}", obj)
    ))
}


#[cfg(test)]
mod changelog_tests {
    use proptest::prelude::*;

    /// Read the CHANGELOG file content at test time.
    fn read_changelog() -> String {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let path = std::path::Path::new(manifest_dir).join("CHANGELOG.md");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read CHANGELOG.md at {:?}: {}", path, e))
    }

    // Feature: additory-release-a11, Property 2: CHANGELOG entry completeness
    //
    // Verifies that `## [0.1.3a11]` section exists in the CHANGELOG and
    // appears before the `## [0.1.3]` section.
    //
    // **Validates: Requirements 5.1**
    proptest! {
        #[test]
        fn changelog_a11_before_stable(_ in 0..100u32) {
            let content = read_changelog();

            // The a11 section must exist
            let a11_pos = content.find("## [0.1.3a11]")
                .expect("CHANGELOG must contain a ## [0.1.3a11] section");

            // The stable 0.1.3 section must exist
            let stable_pos = content.find("## [0.1.3]")
                .expect("CHANGELOG must contain a ## [0.1.3] section");

            // a11 must appear before the stable release
            prop_assert!(
                a11_pos < stable_pos,
                "## [0.1.3a11] (pos {}) must appear before ## [0.1.3] (pos {})",
                a11_pos,
                stable_pos
            );
        }
    }

    #[test]
    fn changelog_a11_has_subsections() {
        let content = read_changelog();

        // Find the a11 section
        let a11_start = content.find("## [0.1.3a11]")
            .expect("CHANGELOG must contain a ## [0.1.3a11] section");

        // Find the next ## section after a11 (the stable 0.1.3 entry)
        let a11_section_end = content[a11_start + 14..]
            .find("\n## [")
            .map(|pos| a11_start + 14 + pos)
            .unwrap_or(content.len());

        let a11_section = &content[a11_start..a11_section_end];

        // Must contain at least Added, Changed, and Fixed subsections
        assert!(
            a11_section.contains("### Added"),
            "## [0.1.3a11] section must contain ### Added"
        );
        assert!(
            a11_section.contains("### Changed"),
            "## [0.1.3a11] section must contain ### Changed"
        );
        assert!(
            a11_section.contains("### Fixed"),
            "## [0.1.3a11] section must contain ### Fixed"
        );
    }
}

/// Feature: additory-release-a11, Property 1: Version consistency across release artifacts
///
/// For any release of additory, the version string in `pyproject.toml` and the version string
/// in `Cargo.toml` SHALL encode the same logical version (same major, minor, patch, and
/// pre-release number), differing only in format convention (Python: `0.1.3a11`, Rust: `0.1.3-alpha.11`).
///
/// **Validates: Requirements 4.1, 4.2**
#[cfg(test)]
mod version_tests {
    /// Parse a Python PEP 440 pre-release version like "0.1.3a11" into (major, minor, patch, pre_number).
    fn parse_python_version(v: &str) -> (u32, u32, u32, u32) {
        // Format: MAJOR.MINOR.PATCHaNUM
        let alpha_pos = v.find('a').expect("Python version must contain 'a' for alpha");
        let base = &v[..alpha_pos];
        let pre_num: u32 = v[alpha_pos + 1..].parse().expect("Pre-release number must be numeric");
        let parts: Vec<u32> = base.split('.').map(|p| p.parse().expect("Version part must be numeric")).collect();
        assert_eq!(parts.len(), 3, "Version must have exactly 3 parts");
        (parts[0], parts[1], parts[2], pre_num)
    }

    /// Parse a Rust SemVer pre-release version like "0.1.3-alpha.11" into (major, minor, patch, pre_number).
    fn parse_rust_version(v: &str) -> (u32, u32, u32, u32) {
        // Format: MAJOR.MINOR.PATCH-alpha.NUM
        let dash_pos = v.find('-').expect("Rust version must contain '-' for pre-release");
        let base = &v[..dash_pos];
        let pre_part = &v[dash_pos + 1..];
        assert!(pre_part.starts_with("alpha."), "Pre-release must start with 'alpha.'");
        let pre_num: u32 = pre_part["alpha.".len()..].parse().expect("Pre-release number must be numeric");
        let parts: Vec<u32> = base.split('.').map(|p| p.parse().expect("Version part must be numeric")).collect();
        assert_eq!(parts.len(), 3, "Version must have exactly 3 parts");
        (parts[0], parts[1], parts[2], pre_num)
    }

    #[test]
    fn version_consistency_pyproject_and_cargo() {
        // Read pyproject.toml
        let pyproject_content = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("pyproject.toml")
        ).expect("Failed to read pyproject.toml");

        // Read Cargo.toml
        let cargo_content = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
        ).expect("Failed to read Cargo.toml");

        // Extract Python version from pyproject.toml
        let py_version = pyproject_content
            .lines()
            .find(|line| line.starts_with("version = "))
            .expect("pyproject.toml must have a version field");
        let py_version = py_version
            .trim_start_matches("version = ")
            .trim_matches('"');

        // Extract Rust version from Cargo.toml (first occurrence under [package])
        let cargo_version = cargo_content
            .lines()
            .find(|line| line.starts_with("version = ") && line.contains("alpha"))
            .expect("Cargo.toml must have a version field with alpha pre-release");
        let cargo_version = cargo_version
            .trim_start_matches("version = ")
            .trim_matches('"');

        // Parse both versions
        let py_parsed = parse_python_version(py_version);
        let rust_parsed = parse_rust_version(cargo_version);

        // They must encode the same logical version
        assert_eq!(
            py_parsed, rust_parsed,
            "Python version '{}' ({:?}) and Rust version '{}' ({:?}) must encode the same logical version",
            py_version, py_parsed, cargo_version, rust_parsed
        );

        // Also verify against the compiled-in VERSION constant
        assert_eq!(
            cargo_version,
            super::VERSION,
            "Cargo.toml version must match the compiled VERSION constant"
        );
    }
}
