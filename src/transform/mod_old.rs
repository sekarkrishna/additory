//! add.transform() module - Transform data WITHIN DataFrame
//!
//! v0.1.3a2 Supported modes:
//! - @calc - Calculate expressions (with parentheses and power operator)
//! - @filter - Filter rows and select columns
//! - @aggregate - Group and aggregate data
//! - @sort - Sort rows by columns
//! - @bankers_round - Apply banker's rounding (round half to even)
//! - @knn - KNN imputation (Python-only)

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::core::types::{UniversalParams, FetchColumn, Expression, AsParam, StrategyValue};
use crate::utils::Logger;
use std::collections::HashMap;

#[cfg(feature = "python")]
use pyo3::{Python, ToPyObject};
#[cfg(feature = "python")]
use crate::bindings::knn_impute;

pub mod filter;
pub mod calc;
pub mod sort;
pub mod transpose;
pub mod aggregate;
pub mod split;
pub mod extract;
pub mod onehot;
pub mod label;
pub mod harmonize;
pub mod bankers_round;

/// Main entry point for add.transform()
///
/// v0.1.3a2 supports @calc, @filter, @aggregate, @sort, and @knn modes.
pub fn transform(
    df: DataFrame,
    params: UniversalParams,
) -> AdditoryResult<DataFrame> {
    let logger = Logger::new(params.logging);
    
    // Determine mode from parameters
    // If explicit_mode is provided, use it; otherwise auto-detect
    let mode = if let Some(ref explicit) = params.explicit_mode {
        explicit.as_str()
    } else {
        determine_mode(&params)?
    };
    
    logger.log_start("add.transform", mode);
    
    match mode {
        "@calc" => {
            // Extract expression(s) and column name(s)
            let expressions = match params.expression {
                Some(Expression::Single(expr)) => vec![expr],
                Some(Expression::Multiple(exprs)) => exprs,
                Some(Expression::Dict(_)) => {
                    return Err(AdditoryError::OperationFailed(
                        "Dict expressions not yet supported".to_string(),
                        "Use single string or list of strings".to_string()
                    ));
                },
                None => {
                    return Err(AdditoryError::missing_parameter("expression", "Expression is required for @calc"));
                }
            };
            
            let new_columns = match params.as_param {
                Some(AsParam::Single(col)) => vec![col],
                Some(AsParam::Multiple(cols)) => cols,
                None => {
                    return Err(AdditoryError::missing_parameter("as", "Column name is required for @calc"));
                }
            };
            
            let calc_params = if expressions.len() == 1 && new_columns.len() == 1 {
                calc::CalcParams::single(
                    expressions[0].clone(),
                    new_columns[0].clone(),
                    params.logging,
                )
            } else {
                calc::CalcParams::multiple(
                    expressions,
                    new_columns,
                    params.logging,
                )
            };
            
            calc::execute(df, calc_params)
        },
        
        "@filter" => {
            let filter_params = filter::FilterParams {
                fetch: params.fetch.map(|cols| {
                    cols.iter().map(|fc| match fc {
                        FetchColumn::NoRename(name) => name.clone(),
                        FetchColumn::Rename(original, _) => original.clone(),
                    }).collect()
                }),
                where_clause: params.where_clause,
                logging: params.logging,
            };
            
            filter::execute(df, filter_params)
        },
        
        "@aggregate" => {
            // Extract by columns
            let by_str = params.by.ok_or_else(|| {
                AdditoryError::missing_parameter("by", "Grouping columns are required for @aggregate")
            })?;
            
            // Parse by into Vec<String> (could be comma-separated)
            let by: Vec<String> = by_str.split(',').map(|s| s.trim().to_string()).collect();
            
            // Extract aggregations from expression parameter (as Dict)
            let aggregations = match params.expression {
                Some(Expression::Dict(agg_dict)) => {
                    let mut result = HashMap::new();
                    for (col, funcs_str) in agg_dict {
                        // Parse comma-separated aggregation functions
                        let agg_funcs: Vec<aggregate::AggFunc> = funcs_str
                            .split(',')
                            .map(|f| f.trim())
                            .filter(|f| !f.is_empty())
                            .map(|f| aggregate::AggFunc::from_str(f)
                                .map_err(|e| AdditoryError::invalid_parameter("expression", f, &e)))
                            .collect::<Result<Vec<_>, _>>()?;
                        result.insert(col, agg_funcs);
                    }
                    result
                },
                _ => {
                    return Err(AdditoryError::invalid_parameter(
                        "expression",
                        "must be a dict",
                        "Use format: expression={'column': 'sum,mean'}"
                    ));
                }
            };
            
            let agg_params = aggregate::AggregateParams {
                by,
                aggregations,
                logging: params.logging,
            };
            
            aggregate::execute(df, agg_params)
        },
        
        "@sort" => {
            // Extract by columns
            let by_str = params.by.ok_or_else(|| {
                AdditoryError::missing_parameter("by", "Sort columns are required for @sort")
            })?;
            
            // Parse by into Vec<String> (could be comma-separated)
            let by: Vec<String> = by_str.split(',').map(|s| s.trim().to_string()).collect();
            
            // Extract sort order from as parameter
            let descending = match params.as_param {
                Some(AsParam::Single(order)) => {
                    vec![order.to_lowercase() == "desc"; by.len()]
                },
                Some(AsParam::Multiple(orders)) => {
                    if orders.len() != by.len() {
                        return Err(AdditoryError::invalid_parameter(
                            "as",
                            &format!("{} values", orders.len()),
                            &format!("Must match length of 'by' parameter ({} columns)", by.len())
                        ));
                    }
                    orders.iter().map(|o| o.to_lowercase() == "desc").collect()
                },
                None => {
                    // Default to ascending for all columns
                    vec![false; by.len()]
                }
            };
            
            let sort_params = sort::SortParams {
                by,
                descending,
                logging: params.logging,
            };
            
            sort::execute(df, sort_params)
        },
        
        "@bankers_round" => {
            // Extract column name from 'by' parameter
            let column = params.by.ok_or_else(|| {
                AdditoryError::missing_parameter("by", "Column name is required for @bankers_round")
            })?;
            
            // Extract decimal places from strategy (optional)
            let decimals = if let Some(ref strategy) = params.strategy {
                if let Some(StrategyValue::Number(n)) = strategy.get("decimals") {
                    Some(*n as i32)
                } else {
                    None
                }
            } else {
                None
            };
            
            let round_params = bankers_round::BankersRoundParams::new(
                column,
                decimals,
                params.logging,
            );
            
            bankers_round::execute(df, round_params)
        },
        
        #[cfg(feature = "python")]
        "@knn" => execute_python_knn(df, params),
        
        #[cfg(not(feature = "python"))]
        "@knn" => Err(AdditoryError::OperationFailed(
            "@knn mode requires Python feature".to_string(),
            "Rebuild with --features python or use Rust-only modes".to_string()
        )),
        
        _ => Err(AdditoryError::OperationFailed(
            format!("Unsupported mode: {}", mode),
            "Supported modes: @calc, @filter, @aggregate, @sort, @bankers_round, @knn".to_string()
        ))
    }
}

/// Determine which transform mode to use based on parameters
fn determine_mode(params: &UniversalParams) -> AdditoryResult<&'static str> {
    // Priority order:
    // 1. @calc - if expression is a Single or Multiple (not Dict)
    // 2. @aggregate - if expression is a Dict (aggregations) and by is provided
    // 3. @sort - if by is provided with as (sort order)
    // 4. @filter - if where is provided
    // 5. @knn - if fetch is provided with strategy
    
    // Check for @calc first
    if let Some(ref expr) = params.expression {
        match expr {
            Expression::Single(_) | Expression::Multiple(_) => {
                return Ok("@calc");
            },
            Expression::Dict(_) => {
                // Dict expression means @aggregate
                if params.by.is_some() {
                    return Ok("@aggregate");
                } else {
                    return Err(AdditoryError::missing_parameter(
                        "by",
                        "Grouping columns are required when using dict expressions for aggregation"
                    ));
                }
            }
        }
    }
    
    // Check for @sort or @aggregate
    if params.by.is_some() {
        // If as_param is provided, it's @sort (sort order)
        // Otherwise, it's an error (need expression for aggregate)
        if params.as_param.is_some() {
            return Ok("@sort");
        } else {
            return Err(AdditoryError::missing_parameter(
                "expression",
                "Aggregation functions are required when using 'by' for grouping. Use expression={'column': 'sum,mean'}"
            ));
        }
    }
    
    // Check for @filter
    if params.where_clause.is_some() || params.fetch.is_some() {
        // Could be @filter or @knn
        // If strategy is provided, assume @knn
        if params.strategy.is_some() {
            return Ok("@knn");
        } else {
            return Ok("@filter");
        }
    }
    
    Err(AdditoryError::OperationFailed(
        "Could not determine transform mode".to_string(),
        "Provide one of: expression= (@calc), where= (@filter), by= with as= (@sort), by= with expression= (@aggregate), or fetch= with strategy= (@knn)".to_string()
    ))
}

/// Execute Python KNN imputation
///
/// This function delegates to the Python KNN implementation via PyO3.
///
/// # Arguments
///
/// * `df` - Input DataFrame
/// * `params` - Universal parameters containing fetch (columns) and strategy
///
/// # Returns
///
/// * `AdditoryResult<DataFrame>` - DataFrame with imputed values
#[cfg(feature = "python")]
fn execute_python_knn(df: DataFrame, params: UniversalParams) -> AdditoryResult<DataFrame> {
    // Extract parameters
    let fetch_columns = params.fetch.ok_or_else(|| {
        AdditoryError::missing_parameter("fetch", "Columns to impute are required for @knn")
    })?;
    
    // Convert FetchColumn to String column names
    let columns: Vec<String> = fetch_columns.iter().map(|fc| {
        match fc {
            FetchColumn::NoRename(name) => name.clone(),
            FetchColumn::Rename(original, _) => original.clone(),
        }
    }).collect();
    
    let strategy = params.strategy; // Optional
    
    // Convert DataFrame to Arrow IPC bytes
    let df_bytes = df.to_arrow_ipc_bytes()?;
    
    // Call Python KNN via PyO3
    let result_bytes = Python::with_gil(|py| {
        // Convert strategy to Python objects if present
        let py_strategy = if let Some(strat) = strategy {
            let mut py_map = std::collections::HashMap::new();
            for (k, v) in strat {
                // Convert StrategyValue to Python object
                let py_obj = match v {
                    StrategyValue::Number(n) => n.to_object(py),
                    StrategyValue::String(s) => s.to_object(py),
                    StrategyValue::Bool(b) => b.to_object(py),
                    StrategyValue::List(l) => l.to_object(py),
                    StrategyValue::Dict(_) => {
                        return Err(pyo3::PyErr::new::<pyo3::exceptions::PyValueError, _>(
                            "Nested dict strategies not yet supported"
                        ));
                    },
                    StrategyValue::Tuple(t) => t.to_object(py),
                };
                py_map.insert(k, py_obj);
            }
            Some(py_map)
        } else {
            None
        };
        
        knn_impute(py, &df_bytes, columns, py_strategy)
            .map_err(|e| pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                format!("KNN imputation failed: {}", e)
            ))
    }).map_err(|e| AdditoryError::OperationFailed(
        format!("KNN imputation failed: {}", e),
        "Check that columns exist and are numeric".to_string()
    ))?;
    
    // Convert result back to DataFrame
    DataFrame::from_arrow_ipc_bytes(&result_bytes, df.original_type())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests are in python-specific/tests/test_integration.py
    // These are just basic unit tests for the transform router
}
