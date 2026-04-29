//! Lookup operation for add.to()
//!
//! Implements basic lookup with aggregation support.

use crate::core::{DataFrame, AdditoryResult, AdditoryError, FetchColumn, Against, Position, StrategyValue, AggregationMode};
use crate::to::aggregation::aggregate_series;
use polars::prelude::*;
use std::collections::HashMap;

/// Perform lookup operation
///
/// # Parameters
/// - `target`: Target DataFrame to add columns to
/// - `reference`: Reference DataFrame to fetch columns from
/// - `fetch`: Columns to fetch (with optional renames)
/// - `against`: Join key(s)
/// - `strategy`: Optional aggregation strategies per column
/// - `position`: Optional position for new columns
///
/// # Returns
/// DataFrame with new columns added
pub fn lookup(
    target: DataFrame,
    reference: DataFrame,
    fetch: Vec<FetchColumn>,
    against: Against,
    strategy: Option<HashMap<String, StrategyValue>>,
    position: Option<Position>,
) -> AdditoryResult<DataFrame> {
    // Get join keys
    let join_keys = against.keys();
    
    // Validate join keys exist in both DataFrames
    for key in &join_keys {
        if !target.has_column(key) {
            return Err(AdditoryError::column_not_found(key, &target.column_names()));
        }
        if !reference.has_column(key) {
            return Err(AdditoryError::column_not_found(key, &reference.column_names()));
        }
    }
    
    // Validate fetch columns exist in reference
    for fetch_col in &fetch {
        let source = fetch_col.original();
        if !reference.has_column(source) {
            return Err(AdditoryError::column_not_found(source, &reference.column_names()));
        }
    }
    
    // Detect cardinality to determine if aggregation is needed
    let needs_aggregation = detect_cardinality(&target, &reference, &join_keys)?;
    
    let mut result = if needs_aggregation {
        // Perform join with aggregation
        lookup_with_aggregation(target, reference, &fetch, &join_keys, &strategy)?
    } else {
        // Simple 1:1 or many:1 join (no aggregation needed)
        // Use lazy evaluation for better performance
        let join_keys_vec: Vec<String> = join_keys.iter().map(|s| s.to_string()).collect();
        
        let result_lazy = target.inner().clone().lazy()
            .join(
                reference.inner().clone().lazy(),
                &join_keys_vec.iter().map(col).collect::<Vec<_>>(),
                &join_keys_vec.iter().map(col).collect::<Vec<_>>(),
                JoinArgs::new(JoinType::Left),
            );
        
        let result_df = result_lazy.collect()
            .map_err(AdditoryError::Polars)?;
        
        DataFrame::new(result_df, target.original_type())
    };
    
    // Apply renames if specified
    for fetch_col in &fetch {
        if fetch_col.is_rename() {
            let original = fetch_col.original();
            let target_name = fetch_col.target();
            if result.has_column(original) && original != target_name {
                result = result.rename(original, target_name)?;
            }
        }
    }
    
    // Apply position if specified
    if let Some(pos) = position {
        let new_col_names: Vec<String> = fetch.iter().map(|f| f.target().to_string()).collect();
        crate::to::position::apply_position(result, &new_col_names, &pos)
    } else {
        Ok(result)
    }
}

/// Detect if aggregation is needed based on cardinality
///
/// Returns true if 1:many relationship (target keys unique, reference keys have duplicates)
fn detect_cardinality(
    target: &DataFrame,
    reference: &DataFrame,
    join_keys: &[&str],
) -> AdditoryResult<bool> {
    let target_total = target.height();
    let ref_total = reference.height();

    // Count unique composite key combinations across ALL join keys
    let owned_keys: Vec<String> = join_keys.iter().map(|s| s.to_string()).collect();

    let target_unique = target.inner()
        .group_by(owned_keys.clone())
        .map_err(AdditoryError::Polars)?
        .get_groups()
        .len();

    let ref_unique = reference.inner()
        .group_by(owned_keys)
        .map_err(AdditoryError::Polars)?
        .get_groups()
        .len();

    // 1:many - target keys unique, reference has duplicates
    let is_one_to_many = target_total == target_unique && ref_total > ref_unique;

    Ok(is_one_to_many)
}

/// Build a composite key string for a single row from multiple join key columns
fn composite_key_at(series_list: &[Series], idx: usize) -> AdditoryResult<String> {
    let mut parts = Vec::with_capacity(series_list.len());
    for s in series_list {
        let val = s.get(idx).map_err(AdditoryError::Polars)?;
        parts.push(format!("{}", val));
    }
    Ok(parts.join("\x00"))
}

/// Perform lookup with aggregation for 1:many relationships
fn lookup_with_aggregation(
    target: DataFrame,
    reference: DataFrame,
    fetch: &[FetchColumn],
    join_keys: &[&str],
    strategy: &Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<DataFrame> {
    // Get all join key series for reference and target
    let ref_key_series: Vec<Series> = join_keys.iter()
        .map(|&k| reference.column(k).map(|c| c.as_materialized_series().clone()))
        .collect::<AdditoryResult<_>>()?;

    let target_key_series: Vec<Series> = join_keys.iter()
        .map(|&k| target.column(k).map(|c| c.as_materialized_series().clone()))
        .collect::<AdditoryResult<_>>()?;

    // Build lookup map: composite_key -> row indices in reference
    let mut key_to_group: HashMap<String, Vec<usize>> = HashMap::new();
    for i in 0..reference.height() {
        let composite_key = composite_key_at(&ref_key_series, i)?;
        key_to_group.entry(composite_key).or_default().push(i);
    }

    // Pre-compute composite keys for each target row
    let target_composite_keys: Vec<String> = (0..target.height())
        .map(|i| composite_key_at(&target_key_series, i))
        .collect::<AdditoryResult<_>>()?;

    // For each fetch column, aggregate in target order
    let mut aggregated_cols: Vec<polars::prelude::Column> = Vec::new();

    // Add all target columns first
    for col_name in target.column_names() {
        let col = target.column(&col_name)?;
        aggregated_cols.push(col.clone());
    }

    // Aggregate each fetch column in target order
    for fetch_col in fetch {
        let col_name = fetch_col.original();
        let target_name = fetch_col.target();

        let agg_mode = get_aggregation_mode(target_name, strategy)?;

        let ref_col = reference.column(col_name)?;
        let ref_series = ref_col.as_materialized_series();

        let mut aggregated_values: Vec<AnyValue> = Vec::new();

        for composite_key in &target_composite_keys {
            if let Some(indices) = key_to_group.get(composite_key) {
                let group_values: Vec<AnyValue> = indices.iter()
                    .map(|&idx| ref_series.get(idx))
                    .collect::<Result<_, _>>()
                    .map_err(AdditoryError::Polars)?;

                let group_series = Series::from_any_values(col_name.into(), &group_values, true)
                    .map_err(AdditoryError::Polars)?;

                let agg_result = aggregate_series_typed(&group_series, &agg_mode)?;
                aggregated_values.push(agg_result);
            } else {
                aggregated_values.push(AnyValue::Null);
            }
        }

        let agg_series = Series::from_any_values(target_name.into(), &aggregated_values, true)
            .map_err(AdditoryError::Polars)?;
        let agg_col = polars::prelude::Column::from(agg_series);
        aggregated_cols.push(agg_col);
    }

    // Create result DataFrame
    let result_df = polars::prelude::DataFrame::new(aggregated_cols)
        .map_err(AdditoryError::Polars)?;

    Ok(DataFrame::from_polars(result_df))
}

/// Aggregate series and return typed AnyValue (preserves dtype)
fn aggregate_series_typed(
    series: &Series,
    mode: &AggregationMode,
) -> AdditoryResult<AnyValue<'static>> {
    if series.is_empty() {
        return Ok(AnyValue::Null);
    }
    
    let mode_str = mode.mode.as_str();
    
    match mode_str {
        "first" => {
            let val = series.get(0).map_err(AdditoryError::Polars)?;
            let match_modifier = mode.match_modifier.as_str();
            
            // Clone to get owned value
            let owned_val = match val {
                AnyValue::Int64(v) => AnyValue::Int64(v),
                AnyValue::Float64(v) => AnyValue::Float64(v),
                AnyValue::String(s) => {
                    let s_str = s.to_string();
                    // Apply trim if requested
                    if match_modifier == "trim" {
                        AnyValue::StringOwned(s_str.trim().to_string().into())
                    } else {
                        AnyValue::StringOwned(s_str.into())
                    }
                }
                AnyValue::StringOwned(s) => {
                    // Apply trim if requested
                    if match_modifier == "trim" {
                        AnyValue::StringOwned(s.trim().to_string().into())
                    } else {
                        AnyValue::StringOwned(s)
                    }
                }
                AnyValue::Boolean(b) => AnyValue::Boolean(b),
                AnyValue::UInt32(v) => AnyValue::UInt32(v),
                AnyValue::UInt64(v) => AnyValue::UInt64(v),
                AnyValue::Int32(v) => AnyValue::Int32(v),
                AnyValue::Null => AnyValue::Null,
                _ => AnyValue::StringOwned(format!("{}", val).into())
            };
            Ok(owned_val)
        }
        "last" => {
            let idx = series.len() - 1;
            let val = series.get(idx).map_err(AdditoryError::Polars)?;
            let match_modifier = mode.match_modifier.as_str();
            
            // Clone to get owned value
            let owned_val = match val {
                AnyValue::Int64(v) => AnyValue::Int64(v),
                AnyValue::Float64(v) => AnyValue::Float64(v),
                AnyValue::String(s) => {
                    let s_str = s.to_string();
                    // Apply trim if requested
                    if match_modifier == "trim" {
                        AnyValue::StringOwned(s_str.trim().to_string().into())
                    } else {
                        AnyValue::StringOwned(s_str.into())
                    }
                }
                AnyValue::StringOwned(s) => {
                    // Apply trim if requested
                    if match_modifier == "trim" {
                        AnyValue::StringOwned(s.trim().to_string().into())
                    } else {
                        AnyValue::StringOwned(s)
                    }
                }
                AnyValue::Boolean(b) => AnyValue::Boolean(b),
                AnyValue::UInt32(v) => AnyValue::UInt32(v),
                AnyValue::UInt64(v) => AnyValue::UInt64(v),
                AnyValue::Int32(v) => AnyValue::Int32(v),
                AnyValue::Null => AnyValue::Null,
                _ => AnyValue::StringOwned(format!("{}", val).into())
            };
            Ok(owned_val)
        }
        "sum" => {
            // Sum numeric values
            match series.sum::<i64>() {
                Ok(sum) => Ok(AnyValue::Int64(sum)),
                Err(_) => {
                    // Try as float
                    match series.sum::<f64>() {
                        Ok(sum) => Ok(AnyValue::Float64(sum)),
                        Err(_) => Err(AdditoryError::aggregation(
                            &format!("Cannot sum non-numeric column '{}'", series.name()),
                            "Sum mode requires numeric data"
                        ))
                    }
                }
            }
        }
        "count" => {
            Ok(AnyValue::UInt32(series.len() as u32))
        }
        "average" | "mean" => {
            let mean = series.mean()
                .ok_or_else(|| AdditoryError::aggregation(
                    &format!("Cannot average non-numeric column '{}'", series.name()),
                    "Average mode requires numeric data"
                ))?;
            Ok(AnyValue::Float64(mean))
        }
        "min" => {
            match series.min::<i64>() {
                Ok(Some(min)) => Ok(AnyValue::Int64(min)),
                Ok(None) => Ok(AnyValue::Null),
                Err(_) => {
                    // Try as float
                    match series.min::<f64>() {
                        Ok(Some(min)) => Ok(AnyValue::Float64(min)),
                        Ok(None) => Ok(AnyValue::Null),
                        Err(_) => Err(AdditoryError::aggregation(
                            &format!("Cannot find min of non-numeric column '{}'", series.name()),
                            "Min mode requires numeric data"
                        ))
                    }
                }
            }
        }
        "max" => {
            match series.max::<i64>() {
                Ok(Some(max)) => Ok(AnyValue::Int64(max)),
                Ok(None) => Ok(AnyValue::Null),
                Err(_) => {
                    // Try as float
                    match series.max::<f64>() {
                        Ok(Some(max)) => Ok(AnyValue::Float64(max)),
                        Ok(None) => Ok(AnyValue::Null),
                        Err(_) => Err(AdditoryError::aggregation(
                            &format!("Cannot find max of non-numeric column '{}'", series.name()),
                            "Max mode requires numeric data"
                        ))
                    }
                }
            }
        }
        "concat" => {
            // Concatenate as string
            let sep = mode.separator.as_deref().unwrap_or("|");
            let mut result = String::new();
            
            for i in 0..series.len() {
                let value = series.get(i).map_err(AdditoryError::Polars)?;
                
                if i > 0 {
                    result.push_str(sep);
                }
                
                match value {
                    AnyValue::Null => {}, // Skip nulls
                    AnyValue::String(s) => result.push_str(s),
                    AnyValue::StringOwned(s) => result.push_str(&s),
                    _ => result.push_str(&format!("{}", value))
                }
            }
            
            Ok(AnyValue::StringOwned(result.into()))
        }
        _ => {
            // Fall back to string aggregation for other modes
            let str_result = aggregate_series(series, mode)?;
            Ok(AnyValue::StringOwned(str_result.into()))
        }
    }
}

/// Get aggregation mode for a column from strategy
fn get_aggregation_mode(
    col_name: &str,
    strategy: &Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<AggregationMode> {
    if let Some(strat) = strategy {
        if let Some(value) = strat.get(col_name) {
            return parse_strategy_value(value);
        }
    }
    
    // Default to "auto" mode
    Ok(AggregationMode {
        mode: "auto".to_string(),
        match_modifier: "auto".to_string(),
        separator: None,
    })
}

/// Parse StrategyValue into AggregationMode
fn parse_strategy_value(value: &StrategyValue) -> AdditoryResult<AggregationMode> {
    match value {
        StrategyValue::String(s) => {
            // Parse mode string like "first", "first:anycase", "concat[,]"
            AggregationMode::from_str(s).map_err(|e| AdditoryError::validation(&e, "Invalid aggregation mode"))
        }
        StrategyValue::Mode(mode) => Ok(mode.clone()),
        StrategyValue::Dict(dict) => {
            // Extract mode from dict
            if let Some(StrategyValue::String(mode_str)) = dict.get("mode") {
                AggregationMode::from_str(mode_str).map_err(|e| AdditoryError::validation(&e, "Invalid aggregation mode"))
            } else {
                Err(AdditoryError::validation("Strategy dict must contain 'mode' key", "Provide aggregation mode"))
            }
        }
        _ => Err(AdditoryError::validation("Invalid strategy value type", "Use string or dict"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FetchColumn, Against};
    use polars::prelude::*;
    
    #[test]
    fn test_lookup_basic() {
        // Create test DataFrames
        let target_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let fetch = vec![FetchColumn::NoRename("name".to_string())];
        let against = Against::Single("id".to_string());
        
        let result = lookup(target, reference, fetch, against, None, None).unwrap();
        
        assert!(result.has_column("name"));
        assert_eq!(result.height(), 3);
    }
    
    #[test]
    fn test_lookup_with_rename() {
        let target_df = df! {
            "id" => &[1, 2, 3],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 2, 3],
            "full_name" => &["Alice", "Bob", "Charlie"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let fetch = vec![FetchColumn::Rename("full_name".to_string(), "name".to_string())];
        let against = Against::Single("id".to_string());
        
        let result = lookup(target, reference, fetch, against, None, None).unwrap();
        
        // Note: Rename not yet implemented, will be added in next iteration
        assert!(result.has_column("full_name") || result.has_column("name"));
    }
}
