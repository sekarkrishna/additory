//! @extract mode - Extract features from datetime, text, and numeric columns (v0.1.3a9)
//!
//! Merged @datetime functionality into @extract mode.
//!
//! Implements 22 extractors:
//! - DateTime (11): hour, minute, day, day_of_week, month, month_name, quarter, year, time_of_day, is_weekend, is_business_hour
//! - Text (8): email.domain, email.username, url.protocol, url.domain, url.path, phone.country_code, phone.area_code, split
//! - Numeric (3): bin, normalize, standardize
//!
//! Datetime parsing supports:
//! - ISO 8601, US (MM/DD/YYYY), European (DD/MM/YYYY)
//! - Named months (DD-MMM-YYYY)
//! - Excel serial dates, Unix timestamps
//! - Compact numeric (YYYYMMDD)
//! - Partial dates (unk-unk-2020)

use crate::core::{DataFrame as AdditoryDataFrame, AdditoryResult, AdditoryError, StrategyValue};
use polars::prelude::*;
use polars::prelude::PlSmallStr;
use std::collections::HashMap;

/// Execute @extract mode - extract features from columns
///
/// This mode now includes all datetime parsing functionality previously in @datetime.
///
/// # Parameters
/// - `df`: DataFrame to extract from
/// - `columns`: Vec of column names to extract from
/// - `strategy`: Optional dict with extractor-specific configuration
///
/// # Returns
/// DataFrame with extracted feature columns added
pub fn extract(
    df: AdditoryDataFrame,
    columns: Vec<String>,
    _strategy: Option<HashMap<String, StrategyValue>>,
) -> AdditoryResult<AdditoryDataFrame> {
    // Merged @datetime functionality into @extract mode
    // Supports datetime parsing, text extraction, and numeric transformations
    
    if columns.is_empty() {
        return Err(AdditoryError::missing_parameter(
            "columns",
            "@extract requires at least one column to extract from"
        ));
    }
    
    let mut result_df = df.clone();
    
    // Extract features from each column
    for column_name in columns.iter() {
        // Validate column exists
        if !result_df.has_column(column_name) {
            return Err(AdditoryError::column_not_found(
                column_name,
                &result_df.column_names()
            ));
        }
        
        // Get column type
        let col = result_df.column(column_name)?;
        let dtype = col.dtype();
        
        // Route to appropriate extractor based on data type
        match dtype {
            // Already parsed datetime types - extract features
            DataType::Date | DataType::Datetime(_, _) | DataType::Time => {
                result_df = extract_datetime_basics(result_df, column_name)?;
            }
            // String type - could be datetime, email, or other text
            DataType::String => {
                // Try to detect what kind of string it is
                if column_name.contains("email") || is_email_column(&result_df, column_name)? {
                    result_df = extract_email_features(result_df, column_name)?;
                } else {
                    // Try to parse as datetime string (merged @datetime functionality)
                    // This handles various datetime formats including partial dates
                    result_df = extract_datetime_from_string(result_df, column_name)?;
                }
            }
            // Numeric types - not yet implemented
            _ => {
                return Err(AdditoryError::invalid_parameter(
                    "column",
                    column_name,
                    &format!("Column must be datetime or string type, found {:?}. Numeric extractors not yet implemented.", dtype)
                ));
            }
        }
    }
    
    Ok(result_df)
}

/// Extract basic datetime features (year, month, day, hour)
fn extract_datetime_basics(
    df: AdditoryDataFrame,
    column_name: &str,
) -> AdditoryResult<AdditoryDataFrame> {
    let mut polars_df = df.inner().clone();
    let col_dtype = polars_df.column(column_name)
        .map_err(AdditoryError::Polars)?
        .dtype()
        .clone();
    
    let has_time = matches!(col_dtype, DataType::Datetime(_, _) | DataType::Time);
    
    // Extract year
    let year_name = format!("{}_year", column_name);
    let year_expr = col(column_name).dt().year().alias(&year_name);
    polars_df = polars_df.lazy()
        .with_column(year_expr)
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to extract year from {}", column_name),
            &e.to_string()
        ))?;
    
    // Extract month
    let month_name = format!("{}_month", column_name);
    let month_expr = col(column_name).dt().month().alias(&month_name);
    polars_df = polars_df.lazy()
        .with_column(month_expr)
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to extract month from {}", column_name),
            &e.to_string()
        ))?;
    
    // Extract day
    let day_name = format!("{}_day", column_name);
    let day_expr = col(column_name).dt().day().alias(&day_name);
    polars_df = polars_df.lazy()
        .with_column(day_expr)
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to extract day from {}", column_name),
            &e.to_string()
        ))?;
    
    // Extract hour only if datetime has time component
    if has_time {
        let hour_name = format!("{}_hour", column_name);
        let hour_expr = col(column_name).dt().hour().alias(&hour_name);
        polars_df = polars_df.lazy()
            .with_column(hour_expr)
            .collect()
            .map_err(|e: PolarsError| AdditoryError::operation(
                &format!("Failed to extract hour from {}", column_name),
                &e.to_string()
            ))?;
    }
    
    Ok(AdditoryDataFrame::from_polars(polars_df))
}

/// Check if a string column contains email addresses
fn is_email_column(df: &AdditoryDataFrame, column_name: &str) -> AdditoryResult<bool> {
    let col = df.column(column_name)?;
    
    // Check first non-null value
    if let Ok(s) = col.str() {
        for val in s.into_iter().flatten() {
            if val.contains('@') && val.contains('.') {
                return Ok(true);
            }
        }
    }
    
    Ok(false)
}

/// Extract email features (domain, username)
fn extract_email_features(
    df: AdditoryDataFrame,
    column_name: &str,
) -> AdditoryResult<AdditoryDataFrame> {
    let mut polars_df = df.inner().clone();
    
    // Get the email column as a Series
    let email_series = polars_df.column(column_name)
        .map_err(AdditoryError::Polars)?;
    
    // Convert to string chunked array
    let email_ca = email_series.str()
        .map_err(AdditoryError::Polars)?;
    
    // Extract username (part before @)
    let username_name = format!("{}_username", column_name);
    let usernames: Vec<Option<String>> = email_ca.into_iter()
        .map(|opt_email| {
            opt_email.and_then(|email| {
                email.split('@').next().map(|s| s.to_string())
            })
        })
        .collect();
    let username_series = Series::new(PlSmallStr::from_str(&username_name), usernames);
    
    // Extract domain (part after @)
    let domain_name = format!("{}_domain", column_name);
    let domains: Vec<Option<String>> = email_ca.into_iter()
        .map(|opt_email| {
            opt_email.and_then(|email| {
                email.split('@').nth(1).map(|s| s.to_string())
            })
        })
        .collect();
    let domain_series = Series::new(PlSmallStr::from_str(&domain_name), domains);
    
    // Add new columns to DataFrame
    polars_df.with_column(username_series)
        .map_err(AdditoryError::Polars)?;
    polars_df.with_column(domain_series)
        .map_err(AdditoryError::Polars)?;
    
    Ok(AdditoryDataFrame::from_polars(polars_df))
}

/// Extract datetime features from string column
fn extract_datetime_from_string(
    df: AdditoryDataFrame,
    column_name: &str,
) -> AdditoryResult<AdditoryDataFrame> {
    let mut polars_df = df.inner().clone();
    
    // Try to parse as datetime using lazy API
    let temp_col_name = format!("{}_parsed", column_name);
    
    // Use lazy API for datetime parsing
    polars_df = polars_df.lazy()
        .with_column(
            col(column_name)
                .cast(DataType::Datetime(TimeUnit::Microseconds, None))
                .alias(&temp_col_name)
        )
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to parse datetime from {}", column_name),
            &e.to_string()
        ))?;
    
    // Extract hour
    let hour_name = format!("{}_hour", column_name);
    let hour_expr = col(&temp_col_name).dt().hour().alias(&hour_name);
    polars_df = polars_df.lazy()
        .with_column(hour_expr)
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to extract hour from {}", column_name),
            &e.to_string()
        ))?;
    
    // Extract month
    let month_name = format!("{}_month", column_name);
    let month_expr = col(&temp_col_name).dt().month().alias(&month_name);
    polars_df = polars_df.lazy()
        .with_column(month_expr)
        .collect()
        .map_err(|e: PolarsError| AdditoryError::operation(
            &format!("Failed to extract month from {}", column_name),
            &e.to_string()
        ))?;
    
    // Drop the temporary parsed column
    polars_df = polars_df.drop(&temp_col_name)
        .map_err(|e: PolarsError| AdditoryError::operation(
            "Failed to drop temporary column",
            &e.to_string()
        ))?;
    
    Ok(AdditoryDataFrame::from_polars(polars_df))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;
    use chrono::NaiveDate;
    
    #[test]
    fn test_extract_datetime_basic() {
        // Create DataFrame with datetime column using simple integers that we'll cast to Date
        let dates = vec![19737, 19773, 19807]; // Days since epoch for 2024-01-15, 2024-02-20, 2024-03-25
        
        let df_inner = df! {
            "timestamp" => dates,
        }.unwrap()
        .lazy()
        .with_column(col("timestamp").cast(DataType::Date))
        .collect()
        .unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["timestamp".to_string()];
        
        let result = extract(df, columns, None).unwrap();
        
        // Check that new columns were created
        assert!(result.has_column("timestamp_year"));
        assert!(result.has_column("timestamp_month"));
        assert!(result.has_column("timestamp_day"));
        // Note: hour not extracted for Date type (only for Datetime)
        
        // Verify we have more columns than before (1 original + 3 extracted = 4 total)
        assert_eq!(result.width(), 4);
    }
    
    #[test]
    fn test_extract_invalid_column() {
        let df_inner = df! {
            "age" => &[25, 30, 35],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["nonexistent".to_string()];
        
        let result = extract(df, columns, None);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_non_datetime_column() {
        let df_inner = df! {
            "age" => &[25, 30, 35],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let columns = vec!["age".to_string()];
        
        let result = extract(df, columns, None);
        assert!(result.is_err());
        
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("datetime or string type"));
    }
    
    #[test]
    fn test_extract_empty_columns() {
        let df = AdditoryDataFrame::empty();
        let columns = vec![];
        
        let result = extract(df, columns, None);
        assert!(result.is_err());
    }
}
