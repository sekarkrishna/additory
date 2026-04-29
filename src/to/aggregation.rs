//! Aggregation modes for add.to() operations
//!
//! Implements 16 aggregation modes with mode:match syntax support.

use crate::core::{AdditoryResult, AdditoryError, AggregationMode};
use polars::prelude::*;

/// Apply aggregation mode to a grouped column
///
/// # Parameters
/// - `series`: The series to aggregate
/// - `mode`: The aggregation mode with optional match modifier
///
/// # Returns
/// Aggregated value as a scalar (owned String for string results)
pub fn aggregate_series(
    series: &Series,
    mode: &AggregationMode,
) -> AdditoryResult<String> {
    // Handle empty series
    if series.is_empty() {
        return Ok(String::new());
    }
    
    // Parse mode and match modifier
    let mode_str = mode.mode.as_str();
    let match_modifier = mode.match_modifier.as_str();
    
    match mode_str {
        // Basic modes
        "auto" => aggregate_auto(series, match_modifier),
        "strict" => aggregate_strict(series),
        "first" => aggregate_first(series, match_modifier),
        "last" => aggregate_last(series, match_modifier),
        "shortest" => aggregate_shortest(series),
        "longest" => aggregate_longest(series),
        "most_common" => aggregate_most_common(series),
        "forward_fill" => aggregate_forward_fill(series),
        "backward_fill" => aggregate_backward_fill(series),
        
        // Numeric modes
        "sum" => aggregate_sum(series),
        "count" => aggregate_count(series),
        "average" | "mean" => aggregate_average(series),  // Support both 'average' and 'mean'
        "min" => aggregate_min(series),
        "max" => aggregate_max(series),
        
        // Concat mode
        "concat" => aggregate_concat(series, mode.separator.as_deref()),
        
        _ => Err(AdditoryError::aggregation(
            &format!("Unknown aggregation mode: {}", mode_str),
            "Valid modes: auto, strict, first, last, shortest, longest, most_common, forward_fill, backward_fill, sum, count, average, mean, min, max, concat"
        ))
    }
}

/// Auto mode: Return value if all same, error if conflict
fn aggregate_auto(series: &Series, _match: &str) -> AdditoryResult<String> {
    let unique = series.n_unique().map_err(AdditoryError::Polars)?;
    
    if unique == 1 {
        // All values are the same
        let value = series.get(0).map_err(AdditoryError::Polars)?;
        Ok(format!("{}", value))
    } else {
        Err(AdditoryError::aggregation(
            &format!("Multiple different values found in column '{}' (auto mode)", series.name()),
            "Use a specific aggregation mode like 'first', 'last', 'concat', etc."
        ))
    }
}

/// Strict mode: Error if multiple values
fn aggregate_strict(series: &Series) -> AdditoryResult<String> {
    if series.len() > 1 {
        Err(AdditoryError::aggregation(
            &format!("Multiple values found in column '{}' (strict mode)", series.name()),
            "Strict mode requires exactly one value per group"
        ))
    } else {
        let value = series.get(0).map_err(AdditoryError::Polars)?;
        Ok(format!("{}", value))
    }
}

/// First mode: Return first value
fn aggregate_first(series: &Series, match_modifier: &str) -> AdditoryResult<String> {
    let value = series.get(0).map_err(AdditoryError::Polars)?;
    let value_str = anyvalue_to_string(&value);
    
    // Apply match modifier if needed
    match match_modifier {
        "trim" => Ok(value_str.trim().to_string()),
        _ => Ok(value_str)
    }
}

/// Helper to convert AnyValue to String without quotes
fn anyvalue_to_string(value: &AnyValue) -> String {
    match value {
        AnyValue::String(s) => s.to_string(),
        AnyValue::StringOwned(s) => s.to_string(),
        _ => format!("{}", value)
    }
}

/// Last mode: Return last value
fn aggregate_last(series: &Series, match_modifier: &str) -> AdditoryResult<String> {
    let idx = series.len() - 1;
    let value = series.get(idx).map_err(AdditoryError::Polars)?;
    let value_str = format!("{}", value);
    
    // Apply match modifier if needed
    match match_modifier {
        "trim" => Ok(value_str.trim().to_string()),
        _ => Ok(value_str)
    }
}

/// Shortest mode: Return shortest string value
fn aggregate_shortest(series: &Series) -> AdditoryResult<String> {
    let mut shortest = String::new();
    let mut shortest_len = usize::MAX;
    
    for i in 0..series.len() {
        let value = series.get(i).map_err(AdditoryError::Polars)?;
        let value_str = format!("{}", value);
        if value_str.len() < shortest_len {
            shortest = value_str;
            shortest_len = shortest.len();
        }
    }
    
    Ok(shortest)
}

/// Longest mode: Return longest string value
fn aggregate_longest(series: &Series) -> AdditoryResult<String> {
    let mut longest = String::new();
    
    for i in 0..series.len() {
        let value = series.get(i).map_err(AdditoryError::Polars)?;
        let value_str = format!("{}", value);
        if value_str.len() > longest.len() {
            longest = value_str;
        }
    }
    
    Ok(longest)
}

/// Most common mode: Return most frequent value
fn aggregate_most_common(series: &Series) -> AdditoryResult<String> {
    // Use value_counts to find most common
    let counts = series.value_counts(true, false, PlSmallStr::from_str("count"), false)
        .map_err(AdditoryError::Polars)?;
    
    // Get first value (most common)
    let value_col = counts.column(series.name())
        .map_err(AdditoryError::Polars)?;
    
    let value = value_col.get(0).map_err(AdditoryError::Polars)?;
    Ok(format!("{}", value))
}

/// Forward fill mode: Use previous row value
fn aggregate_forward_fill(series: &Series) -> AdditoryResult<String> {
    // For aggregation, just return first non-null value
    for i in 0..series.len() {
        let value = series.get(i).map_err(AdditoryError::Polars)?;
        if !matches!(value, AnyValue::Null) {
            return Ok(format!("{}", value));
        }
    }
    Ok(String::new())
}

/// Backward fill mode: Use next row value
fn aggregate_backward_fill(series: &Series) -> AdditoryResult<String> {
    // For aggregation, just return last non-null value
    for i in (0..series.len()).rev() {
        let value = series.get(i).map_err(AdditoryError::Polars)?;
        if !matches!(value, AnyValue::Null) {
            return Ok(format!("{}", value));
        }
    }
    Ok(String::new())
}

/// Sum mode: Sum numeric values
fn aggregate_sum(series: &Series) -> AdditoryResult<String> {
    match series.sum::<f64>() {
        Ok(sum) => Ok(format!("{}", sum)),
        Err(_) => Err(AdditoryError::aggregation(
            &format!("Cannot sum non-numeric column '{}'", series.name()),
            "Sum mode requires numeric data"
        ))
    }
}

/// Count mode: Count occurrences
fn aggregate_count(series: &Series) -> AdditoryResult<String> {
    Ok(format!("{}", series.len()))
}

/// Average mode: Average numeric values
fn aggregate_average(series: &Series) -> AdditoryResult<String> {
    let mean = series.mean()
        .ok_or_else(|| AdditoryError::aggregation(
            &format!("Cannot average non-numeric column '{}'", series.name()),
            "Average mode requires numeric data"
        ))?;
    
    Ok(format!("{}", mean))
}

/// Min mode: Minimum value
fn aggregate_min(series: &Series) -> AdditoryResult<String> {
    match series.min::<f64>() {
        Ok(Some(min)) => Ok(format!("{}", min)),
        Ok(None) => Err(AdditoryError::aggregation(
            &format!("No values in column '{}'", series.name()),
            "Min mode requires at least one value"
        )),
        Err(_) => Err(AdditoryError::aggregation(
            &format!("Cannot find min of non-numeric column '{}'", series.name()),
            "Min mode requires numeric data"
        ))
    }
}

/// Max mode: Maximum value
fn aggregate_max(series: &Series) -> AdditoryResult<String> {
    match series.max::<f64>() {
        Ok(Some(max)) => Ok(format!("{}", max)),
        Ok(None) => Err(AdditoryError::aggregation(
            &format!("No values in column '{}'", series.name()),
            "Max mode requires at least one value"
        )),
        Err(_) => Err(AdditoryError::aggregation(
            &format!("Cannot find max of non-numeric column '{}'", series.name()),
            "Max mode requires numeric data"
        ))
    }
}

/// Concat mode: Concatenate values with separator
fn aggregate_concat(series: &Series, separator: Option<&str>) -> AdditoryResult<String> {
    let sep = separator.unwrap_or("|");
    let mut result = String::new();
    
    for i in 0..series.len() {
        let value = series.get(i).map_err(AdditoryError::Polars)?;
        
        if i > 0 {
            result.push_str(sep);
        }
        
        match value {
            AnyValue::Null => {}, // Skip nulls
            _ => result.push_str(&anyvalue_to_string(&value))
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aggregate_first() {
        let series = Series::new("test".into(), &["a", "b", "c"]);
        let mode = AggregationMode {
            mode: "first".to_string(),
            match_modifier: "auto".to_string(),
            separator: None,
        };
        
        let result = aggregate_series(&series, &mode).unwrap();
        assert_eq!(result, "a");
    }
    
    #[test]
    fn test_aggregate_sum() {
        let series = Series::new("test".into(), &[1, 2, 3]);
        let mode = AggregationMode {
            mode: "sum".to_string(),
            match_modifier: "auto".to_string(),
            separator: None,
        };
        
        let result = aggregate_series(&series, &mode).unwrap();
        assert_eq!(result, "6");
    }
    
    #[test]
    fn test_aggregate_concat() {
        let series = Series::new("test".into(), &["a", "b", "c"]);
        let mode = AggregationMode {
            mode: "concat".to_string(),
            match_modifier: "auto".to_string(),
            separator: Some(",".to_string()),
        };
        
        let result = aggregate_series(&series, &mode).unwrap();
        assert_eq!(result, "a,b,c");
    }
}
