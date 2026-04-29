//! augment mode - Add synthetic rows to existing DataFrame

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::core::types::UniversalParams;
use crate::utils::logging::Logger;
use polars::prelude::*;
use rand::Rng;
use rand_distr::{Distribution, Normal};

pub fn execute(
    df: DataFrame,
    params: UniversalParams,
    logger: &Logger,
) -> AdditoryResult<DataFrame> {
    logger.log_result("add.synthetic()", "Executing augment mode - adding synthetic rows");

    // Extract n parameter (number of rows to add)
    let n = params.n.ok_or_else(|| AdditoryError::missing_parameter(
        "n",
        "augment mode requires 'n' parameter (number of rows to add)"
    ))?;

    logger.log_param("add.synthetic()", "n (rows to add)", &n.to_string());

    // Get the Polars DataFrame
    let original_df = df.inner();
    
    // Simple distribution-based augmentation
    // For each numeric column, fit a normal distribution and generate new values
    // For categorical columns, sample from existing values
    
    let mut new_columns: Vec<Series> = Vec::new();
    
    for col in original_df.get_columns() {
        let series = col.as_materialized_series();
        let col_name = series.name().to_string();
        logger.log_param("add.synthetic()", &format!("Processing column '{}'", col_name), &format!("{:?}", series.dtype()));
        
        match series.dtype() {
            DataType::Float64 | DataType::Float32 => {
                // Numeric column - fit normal distribution
                let values: Vec<f64> = series.f64()
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to extract numeric values",
                        &e.to_string()
                    ))?
                    .into_iter()
                    .flatten()
                    .collect();
                
                if values.is_empty() {
                    return Err(AdditoryError::invalid_parameter(
                        &col_name,
                        "empty",
                        "Cannot augment empty numeric column"
                    ));
                }
                
                // Calculate mean and std
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
                let std = variance.sqrt();
                
                // Generate new values
                let mut rng = rand::thread_rng();
                let normal = Normal::new(mean, std.max(0.01))  // Avoid zero std
                    .map_err(|e: rand_distr::NormalError| AdditoryError::operation(
                        "Failed to create normal distribution",
                        &e.to_string()
                    ))?;
                
                let new_values: Vec<f64> = (0..n)
                    .map(|_| normal.sample(&mut rng))
                    .collect();
                
                new_columns.push(Series::new(col_name.as_str().into(), &new_values));
            }
            DataType::Int64 | DataType::Int32 | DataType::Int16 | DataType::Int8 |
            DataType::UInt64 | DataType::UInt32 | DataType::UInt16 | DataType::UInt8 => {
                // Integer column - fit normal distribution and round
                let values: Vec<f64> = series.cast(&DataType::Float64)
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to cast integer column",
                        &e.to_string()
                    ))?
                    .f64()
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to extract integer values",
                        &e.to_string()
                    ))?
                    .into_iter()
                    .flatten()
                    .collect();
                
                if values.is_empty() {
                    return Err(AdditoryError::invalid_parameter(
                        &col_name,
                        "empty",
                        "Cannot augment empty integer column"
                    ));
                }
                
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let variance = values.iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / values.len() as f64;
                let std = variance.sqrt();
                
                let mut rng = rand::thread_rng();
                let normal = Normal::new(mean, std.max(0.01))
                    .map_err(|e: rand_distr::NormalError| AdditoryError::operation(
                        "Failed to create normal distribution",
                        &e.to_string()
                    ))?;
                
                let new_values: Vec<i64> = (0..n)
                    .map(|_| normal.sample(&mut rng).round() as i64)
                    .collect();
                
                new_columns.push(Series::new(col_name.as_str().into(), &new_values));
            }
            DataType::String => {
                // Categorical column - sample from existing values
                let values: Vec<Option<&str>> = series.str()
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to extract string values",
                        &e.to_string()
                    ))?
                    .into_iter()
                    .collect();
                
                let non_null_values: Vec<&str> = values.iter()
                    .filter_map(|v| *v)
                    .collect();
                
                if non_null_values.is_empty() {
                    return Err(AdditoryError::invalid_parameter(
                        &col_name,
                        "empty",
                        "Cannot augment empty string column"
                    ));
                }
                
                // Sample with replacement
                let mut rng = rand::thread_rng();
                let new_values: Vec<&str> = (0..n)
                    .map(|_| {
                        let idx = rng.gen_range(0..non_null_values.len());
                        non_null_values[idx]
                    })
                    .collect();
                
                new_columns.push(Series::new(col_name.as_str().into(), &new_values));
            }
            _ => {
                // Unsupported type - just repeat first value
                logger.log_warning("add.synthetic()", &format!("Unsupported column type {:?} for column '{}', using first value", series.dtype(), col_name));
                
                let first_value = series.get(0)
                    .map_err(|e: PolarsError| AdditoryError::operation(
                        "Failed to get first value",
                        &e.to_string()
                    ))?;
                
                let new_series = Series::new(col_name.as_str().into(), vec![first_value; n]);
                new_columns.push(new_series);
            }
        }
    }
    
    // Create new DataFrame from synthetic columns
    // Convert Series to Column for newer Polars API
    let new_cols: Vec<polars::prelude::Column> = new_columns.into_iter()
        .map(|s| s.into())
        .collect();
    
    let synthetic_df = polars::prelude::DataFrame::new(new_cols)
        .map_err(|e: PolarsError| AdditoryError::operation(
            "Failed to create synthetic DataFrame",
            &e.to_string()
        ))?;
    
    // Concatenate original and synthetic DataFrames
    let result_df = original_df.vstack(&synthetic_df)
        .map_err(|e: PolarsError| AdditoryError::operation(
            "Failed to concatenate DataFrames",
            &e.to_string()
        ))?;
    
    logger.log_dataframe("add.synthetic()", "Augmented DataFrame", result_df.height(), result_df.width());
    
    Ok(DataFrame::from_polars(result_df))
}
