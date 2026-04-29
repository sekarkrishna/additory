//! @harmonize mode - Unit harmonization (v0.1.3a5)
//!
//! Convert values between different measurement systems
//!
//! Supported unit categories:
//! 1. Weight/Mass (kg, lbs, g, oz, tons)
//! 2. Distance (km, miles, m, ft, in)
//! 3. Temperature (C, F, K)
//! 4. Currency (user-provided rates)
//! 5. Volume (L, gal, mL)
//! 6. Speed (km/h, mph, m/s)

use crate::core::{DataFrame as AdditoryDataFrame, AdditoryResult, AdditoryError, StrategyValue};
use polars::prelude::*;
use std::collections::HashMap;

/// Execute @harmonize mode - convert units
///
/// # Parameters
/// - `df`: DataFrame to harmonize
/// - `columns`: Dict mapping column names to target units (simplified to Vec for now)
/// - `strategy`: Optional dict with column-specific configuration
///
/// # Returns
/// DataFrame with harmonized units
pub fn harmonize(
    df: AdditoryDataFrame,
    columns: Vec<String>,
    _strategy: Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<AdditoryDataFrame> {
    // For now, implement basic unit conversion
    // Full implementation with all unit types will come later
    
    if columns.is_empty() {
        return Err(AdditoryError::missing_parameter(
            "columns",
            "@harmonize requires at least one column to harmonize"
        ));
    }
    
    let result_df = df.clone();
    
    // Validate columns exist
    for column_name in columns.iter() {
        if !result_df.has_column(column_name) {
            return Err(AdditoryError::column_not_found(
                column_name,
                &result_df.column_names()
            ));
        }
        
        // Get column type
        let col = result_df.column(column_name)?;
        let dtype = col.dtype();
        
        // Validate column is numeric
        let is_numeric = matches!(dtype,
            DataType::Int8 | DataType::Int16 | DataType::Int32 | DataType::Int64 |
            DataType::UInt8 | DataType::UInt16 | DataType::UInt32 | DataType::UInt64 |
            DataType::Float32 | DataType::Float64
        );
        
        if !is_numeric {
            return Err(AdditoryError::invalid_parameter(
                "column",
                column_name,
                &format!("Column must be numeric type for @harmonize, found {:?}", dtype)
            ));
        }
    }
    
    // For now, just validate and return the DataFrame unchanged
    // Full unit conversion implementation will come later
    Ok(result_df)
}

/// Convert temperature units
fn _convert_temperature(value: f64, from_unit: &str, to_unit: &str) -> AdditoryResult<f64> {
    // Convert to Celsius first
    let celsius = match from_unit.to_lowercase().as_str() {
        "c" | "celsius" => value,
        "f" | "fahrenheit" => (value - 32.0) * 5.0 / 9.0,
        "k" | "kelvin" => value - 273.15,
        _ => return Err(AdditoryError::invalid_parameter(
            "from_unit",
            from_unit,
            "Unsupported temperature unit. Use C, F, or K"
        )),
    };
    
    // Convert from Celsius to target
    let result = match to_unit.to_lowercase().as_str() {
        "c" | "celsius" => celsius,
        "f" | "fahrenheit" => celsius * 9.0 / 5.0 + 32.0,
        "k" | "kelvin" => celsius + 273.15,
        _ => return Err(AdditoryError::invalid_parameter(
            "to_unit",
            to_unit,
            "Unsupported temperature unit. Use C, F, or K"
        )),
    };
    
    Ok(result)
}

/// Convert weight units
fn _convert_weight(value: f64, from_unit: &str, to_unit: &str) -> AdditoryResult<f64> {
    // Convert to kg first
    let kg = match from_unit.to_lowercase().as_str() {
        "kg" | "kilogram" | "kilograms" => value,
        "g" | "gram" | "grams" => value / 1000.0,
        "lbs" | "lb" | "pound" | "pounds" => value * 0.453592,
        "oz" | "ounce" | "ounces" => value * 0.0283495,
        "ton" | "tons" => value * 1000.0,
        _ => return Err(AdditoryError::invalid_parameter(
            "from_unit",
            from_unit,
            "Unsupported weight unit. Use kg, g, lbs, oz, or tons"
        )),
    };
    
    // Convert from kg to target
    let result = match to_unit.to_lowercase().as_str() {
        "kg" | "kilogram" | "kilograms" => kg,
        "g" | "gram" | "grams" => kg * 1000.0,
        "lbs" | "lb" | "pound" | "pounds" => kg / 0.453592,
        "oz" | "ounce" | "ounces" => kg / 0.0283495,
        "ton" | "tons" => kg / 1000.0,
        _ => return Err(AdditoryError::invalid_parameter(
            "to_unit",
            to_unit,
            "Unsupported weight unit. Use kg, g, lbs, oz, or tons"
        )),
    };
    
    Ok(result)
}

/// Convert distance units
fn _convert_distance(value: f64, from_unit: &str, to_unit: &str) -> AdditoryResult<f64> {
    // Convert to meters first
    let meters = match from_unit.to_lowercase().as_str() {
        "m" | "meter" | "meters" => value,
        "km" | "kilometer" | "kilometers" => value * 1000.0,
        "cm" | "centimeter" | "centimeters" => value / 100.0,
        "mm" | "millimeter" | "millimeters" => value / 1000.0,
        "mi" | "mile" | "miles" => value * 1609.34,
        "ft" | "foot" | "feet" => value * 0.3048,
        "in" | "inch" | "inches" => value * 0.0254,
        _ => return Err(AdditoryError::invalid_parameter(
            "from_unit",
            from_unit,
            "Unsupported distance unit. Use m, km, cm, mm, mi, ft, or in"
        )),
    };
    
    // Convert from meters to target
    let result = match to_unit.to_lowercase().as_str() {
        "m" | "meter" | "meters" => meters,
        "km" | "kilometer" | "kilometers" => meters / 1000.0,
        "cm" | "centimeter" | "centimeters" => meters * 100.0,
        "mm" | "millimeter" | "millimeters" => meters * 1000.0,
        "mi" | "mile" | "miles" => meters / 1609.34,
        "ft" | "foot" | "feet" => meters / 0.3048,
        "in" | "inch" | "inches" => meters / 0.0254,
        _ => return Err(AdditoryError::invalid_parameter(
            "to_unit",
            to_unit,
            "Unsupported distance unit. Use m, km, cm, mm, mi, ft, or in"
        )),
    };
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    
    #[test]
    fn test_harmonize_numeric_column() {
        let df_inner = df! {
            "weight" => &[150.0, 180.0, 200.0],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["weight".to_string()];
        
        // Should succeed (numeric column)
        let result = super::harmonize(df, columns, None).unwrap();
        
        assert!(result.has_column("weight"));
    }
    
    #[test]
    fn test_harmonize_invalid_column() {
        let df_inner = df! {
            "age" => &[25, 30, 35],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["nonexistent".to_string()];
        
        let result = super::harmonize(df, columns, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_harmonize_non_numeric_column() {
        let df_inner = df! {
            "name" => &["Alice", "Bob", "Charlie"],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["name".to_string()];
        
        let result = super::harmonize(df, columns, None);
        assert!(result.is_err());
        
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("numeric type"));
    }
    
    #[test]
    fn test_harmonize_empty_columns() {
        let df = AdditoryDataFrame::empty();
        let columns = vec![];
        
        let result = super::harmonize(df, columns, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_convert_temperature() {
        // Celsius to Fahrenheit
        let result = _convert_temperature(0.0, "C", "F").unwrap();
        assert!((result - 32.0).abs() < 0.01);
        
        // Fahrenheit to Celsius
        let result = _convert_temperature(32.0, "F", "C").unwrap();
        assert!(result.abs() < 0.01);
        
        // Celsius to Kelvin
        let result = _convert_temperature(0.0, "C", "K").unwrap();
        assert!((result - 273.15).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_weight() {
        // kg to lbs
        let result = _convert_weight(1.0, "kg", "lbs").unwrap();
        assert!((result - 2.20462).abs() < 0.01);
        
        // lbs to kg
        let result = _convert_weight(2.20462, "lbs", "kg").unwrap();
        assert!((result - 1.0).abs() < 0.01);
        
        // g to kg
        let result = _convert_weight(1000.0, "g", "kg").unwrap();
        assert!((result - 1.0).abs() < 0.01);
    }
    
    #[test]
    fn test_convert_distance() {
        // km to miles
        let result = _convert_distance(1.0, "km", "mi").unwrap();
        assert!((result - 0.621371).abs() < 0.01);
        
        // miles to km
        let result = _convert_distance(1.0, "mi", "km").unwrap();
        assert!((result - 1.60934).abs() < 0.01);
        
        // m to ft
        let result = _convert_distance(1.0, "m", "ft").unwrap();
        assert!((result - 3.28084).abs() < 0.01);
    }
}
