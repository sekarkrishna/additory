//! add.transform() module - Transform data WITHIN DataFrame (v0.1.3a9)
//!
//! Implements 10 transform modes (plus 2 hidden easter eggs):
//! - @calc - Calculate expressions
//! - @filter - Filter rows and select columns
//! - @sort - Sort rows
//! - @aggregate - Group and aggregate
//! - @round - Rounding (standard, up, down, banker) - creates new columns
//! - @transpose - Transpose DataFrame
//! - @onehotencode - One-hot encoding
//! - @extract - Extract features (merged @datetime functionality)
//! - @harmonize - Convert measurement units
//! - @deduce - Missing value imputation (7 methods: auto, mean, median, mode, forward, backward, knn)
//!
//! Note: @datetime mode has been merged into @extract mode for unified pattern extraction

use crate::core::{DataFrame, AdditoryResult, AdditoryError, UniversalParams, TransformMode};

// Submodules
pub mod calc;
pub mod filter;
pub mod sort;
pub mod aggregate;
pub mod bankers_round;
pub mod transpose;
pub mod onehotencode;
pub mod extract;
// Note: datetime module kept for reference but @datetime mode removed
// All datetime functionality is now in extract module
pub mod datetime;
pub mod harmonize;
pub mod knn;
pub mod deduce;
pub mod label;
pub mod split;

/// Parse @round mode string to extract sub-mode and decimal precision
/// Examples: @round, @round:2, @round:banker, @roundup, @rounddown
fn parse_round_mode(mode_str: &str) -> (bankers_round::RoundMode, u32) {
    let parts: Vec<&str> = mode_str.split(':').collect();
    let base_mode = parts[0];
    
    // Determine rounding mode
    let round_mode = match base_mode {
        "@roundup" => bankers_round::RoundMode::Up,
        "@rounddown" => bankers_round::RoundMode::Down,
        "@round" => {
            // Check if there's a sub-mode specified
            if parts.len() > 1 {
                match parts[1] {
                    "banker" => bankers_round::RoundMode::Banker,
                    _ => bankers_round::RoundMode::Standard, // Assume it's a decimal precision
                }
            } else {
                bankers_round::RoundMode::Standard
            }
        }
        _ => bankers_round::RoundMode::Standard,
    };
    
    // Determine decimal precision
    let decimals = if parts.len() > 1 {
        // Try to parse as number
        if let Ok(d) = parts[1].parse::<u32>() {
            d
        } else if parts[1] == "banker" {
            // @round:banker - default to 2 decimals
            2
        } else {
            2 // Default
        }
    } else {
        2 // Default to 2 decimal places
    };
    
    (round_mode, decimals)
}

/// Main entry point for add.transform()
///
/// # Parameters
/// - `df`: DataFrame to transform
/// - `mode`: Transform mode to apply
/// - `params`: Universal parameters containing mode-specific options
///
/// # Returns
/// Transformed DataFrame
pub fn transform(
    df: DataFrame,
    mode: TransformMode,
    params: UniversalParams,
) -> AdditoryResult<DataFrame> {
    // Dispatch to mode-specific implementation
    match mode {
        TransformMode::Calc => {
            let expression = params.expression.ok_or_else(|| AdditoryError::missing_parameter(
                "expression",
                "@calc mode requires an expression to evaluate"
            ))?;

            // Migration check: reject inbuilt: and user: prefixes
            if let crate::core::Expression::Single(ref expr_str) = expression {
                if let Some(name) = expr_str.strip_prefix("inbuilt:") {
                    return Err(AdditoryError::validation(
                        &format!(
                            "The 'inbuilt:{}' expression path has been removed.",
                            name
                        ),
                        &format!(
                            "Use the dynamic API instead: add.{}(df)",
                            name
                        ),
                    ));
                }
                if let Some(name) = expr_str.strip_prefix("user:") {
                    return Err(AdditoryError::validation(
                        &format!(
                            "The 'user:{}' expression path has been removed.",
                            name
                        ),
                        &format!(
                            "Use the dynamic API instead: add.{}(df)",
                            name
                        ),
                    ));
                }
            }

            calc::calc(df, expression, params.name)
        }
        TransformMode::Filter => {
            // where_ or fetch required
            if params.where_.is_none() && params.fetch.is_none() {
                return Err(AdditoryError::missing_parameter(
                    "where or fetch",
                    "@filter mode requires either where clause or fetch columns"
                ));
            }
            // Convert fetch from FetchColumn to String
            let fetch_strings = params.fetch.map(|fetch_cols| {
                fetch_cols.into_iter().map(|fc| fc.original().to_string()).collect()
            });
            filter::filter(df, params.where_, fetch_strings)
        }
        TransformMode::Sort => {
            let by = params.by.ok_or_else(|| AdditoryError::missing_parameter(
                "by",
                "@sort mode requires columns to sort by"
            ))?;
            sort::sort(df, by, params.name)
        }
        TransformMode::Aggregate => {
            let by = params.by.ok_or_else(|| AdditoryError::missing_parameter(
                "by",
                "@aggregate mode requires columns to group by"
            ))?;
            let strategy = params.strategy.ok_or_else(|| AdditoryError::missing_parameter(
                "strategy",
                "@aggregate mode requires aggregation strategy"
            ))?;
            aggregate::aggregate(df, by, strategy)
        }
        TransformMode::BankersRound => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@bankers_round mode requires columns to round"
            ))?;
            // Convert columns to By
            let by = if columns.len() == 1 {
                crate::core::By::Single(columns[0].clone())
            } else {
                crate::core::By::Multiple(columns)
            };
            bankers_round::bankers_round(df, by, params.strategy)
        }
        TransformMode::Round => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@round mode requires columns to round"
            ))?;
            
            // Parse mode string to get sub-mode and decimal precision
            let mode_str = params.mode_string.as_deref().unwrap_or("@round");
            let (round_mode, decimals) = parse_round_mode(mode_str);
            
            // Convert columns to By
            let by = if columns.len() == 1 {
                crate::core::By::Single(columns[0].clone())
            } else {
                crate::core::By::Multiple(columns)
            };
            
            bankers_round::round_columns(df, by, round_mode, decimals, params.strategy)
        }
        TransformMode::Transpose => {
            transpose::transpose(df)
        }
        TransformMode::OneHotEncode => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@onehotencode mode requires columns to encode"
            ))?;
            onehotencode::onehotencode(df, columns, params.strategy)
        }
        TransformMode::Extract => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@extract mode requires columns to extract from"
            ))?;
            extract::extract(df, columns, params.strategy)
        }
        // @datetime has been merged into @extract - redirect to extract functionality
        TransformMode::Datetime => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@datetime mode requires columns to extract from"
            ))?;
            extract::extract(df, columns, params.strategy)
        }
        TransformMode::Harmonize => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@harmonize mode requires columns to harmonize"
            ))?;
            harmonize::harmonize(df, columns, params.strategy)
        }
        TransformMode::Knn => {
            // Get columns to impute - check both 'columns' and 'fetch' parameters
            let columns = if let Some(cols) = params.columns {
                cols
            } else if let Some(fetch_cols) = params.fetch {
                // Convert FetchColumn to String
                fetch_cols.into_iter().map(|fc| fc.original().to_string()).collect()
            } else {
                return Err(AdditoryError::missing_parameter(
                    "columns or fetch",
                    "@knn mode requires columns to impute"
                ));
            };
            
            // Parse strategy parameters (k, weights, metric)
            let strategy = params.strategy.unwrap_or_default();
            
            // Extract k parameter (default: 5)
            let k = if let Some(crate::core::StrategyValue::Number(n)) = strategy.get("k") {
                *n as usize
            } else {
                5
            };
            
            // Extract weights parameter (default: "uniform")
            let weights_str = if let Some(crate::core::StrategyValue::String(s)) = strategy.get("weights") {
                s.as_str()
            } else {
                "uniform"
            };
            let weights = knn::WeightStrategy::parse_strategy(weights_str)?;
            
            // Extract metric parameter (default: "euclidean")
            let metric_str = if let Some(crate::core::StrategyValue::String(s)) = strategy.get("metric") {
                s.as_str()
            } else {
                "euclidean"
            };
            let metric = knn::DistanceMetric::parse_metric(metric_str)?;
            
            // Create KNN imputer and perform imputation
            let imputer = knn::KnnImputer::new(k, weights, metric)?;
            imputer.impute(df, columns)
        }
        TransformMode::Deduce => {
            // Get infer columns (columns to fill)
            let infer_cols = if let Some(cols) = params.infer {
                cols
            } else if let Some(cols) = params.columns {
                // Fallback to columns parameter for backward compatibility
                cols
            } else if let Some(fetch_cols) = params.fetch {
                // Convert FetchColumn to String
                fetch_cols.into_iter().map(|fc| fc.original().to_string()).collect()
            } else {
                return Err(AdditoryError::missing_parameter(
                    "infer",
                    "@deduce mode requires 'infer' parameter to specify columns to fill"
                ));
            };
            
            // Get output column names
            let output_cols = if let Some(name_param) = params.name {
                match name_param {
                    crate::core::AsParam::Single(s) => vec![s],
                    crate::core::AsParam::Multiple(v) => v,
                    crate::core::AsParam::SortOrder(_) => {
                        return Err(AdditoryError::Validation(
                            "@deduce mode requires column names, not sort order".to_string(),
                            "Use 'name' parameter with column names".to_string(),
                        ));
                    }
                }
            } else {
                return Err(AdditoryError::missing_parameter(
                    "name",
                    "@deduce mode requires 'name' parameter for output column names"
                ));
            };
            
            // Get imputation methods
            let methods = if let Some(m) = params.method {
                m
            } else {
                return Err(AdditoryError::missing_parameter(
                    "method",
                    "@deduce mode requires 'method' parameter to specify imputation method"
                ));
            };
            
            // Call deduce function with new signature
            deduce::deduce(df, infer_cols, output_cols, methods, params.against_text, params.strategy)
        }
        TransformMode::Label => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@label mode requires column to encode"
            ))?;
            
            if columns.is_empty() {
                return Err(AdditoryError::missing_parameter(
                    "columns",
                    "@label mode requires at least one column"
                ));
            }
            
            label::execute(df, label::LabelParams {
                column: columns[0].clone(),
                new_column: None, // Will use default naming
                logging: params.logging,
            })
        }
        TransformMode::Split => {
            let columns = params.columns.ok_or_else(|| AdditoryError::missing_parameter(
                "columns",
                "@split mode requires column to split"
            ))?;
            
            if columns.is_empty() {
                return Err(AdditoryError::missing_parameter(
                    "columns",
                    "@split mode requires at least one column"
                ));
            }
            
            let separator = if let Some(by) = params.by {
                match by {
                    crate::core::By::Single(s) => s,
                    crate::core::By::Multiple(v) => v.first().unwrap_or(&" ".to_string()).clone(),
                }
            } else {
                " ".to_string() // Default to space
            };
            
            // Generate default column names if not provided
            let new_columns = vec![
                format!("{}_1", columns[0]),
                format!("{}_2", columns[0]),
            ];
            
            split::execute(df, split::SplitParams {
                column: columns[0].clone(),
                separator,
                new_columns,
                logging: params.logging,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use crate::core::{Expression, AsParam, By};
    use polars::prelude::*;
    
    #[test]
    fn test_transform_calc() {
        let df_inner = df! {
            "a" => &[1, 2, 3],
            "b" => &[4, 5, 6],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.expression = Some(Expression::Single("a + b".to_string()));
        params.name = Some(AsParam::Single("sum".to_string()));
        
        let result = transform(df, TransformMode::Calc, params).unwrap();
        
        assert!(result.has_column("sum"));
    }
    
    #[test]
    fn test_transform_filter() {
        let df_inner = df! {
            "age" => &[25, 30, 35, 40],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.where_ = Some("age > 30".to_string());
        
        let result = transform(df, TransformMode::Filter, params).unwrap();
        
        assert_eq!(result.height(), 2);
    }
    
    #[test]
    fn test_transform_sort() {
        let df_inner = df! {
            "age" => &[35, 25, 30],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.by = Some(By::Single("age".to_string()));
        
        let result = transform(df, TransformMode::Sort, params).unwrap();
        
        let age_col = result.column("age").unwrap();
        let age_series = age_col.as_materialized_series();
        assert_eq!(age_series.i32().unwrap().get(0).unwrap(), 25);
    }
    
    #[test]
    fn test_transform_transpose() {
        let df_inner = df! {
            "A" => &[1, 2, 3],
            "B" => &[4, 5, 6],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let params = UniversalParams::default();
        
        let result = transform(df, TransformMode::Transpose, params).unwrap();
        
        assert_eq!(result.height(), 2);
        assert_eq!(result.width(), 3);
    }
    
    #[test]
    fn test_transform_extract() {
        // Create DataFrame with date column
        let dates = vec![19737, 19773, 19807]; // Days since epoch
        let df_inner = df! {
            "timestamp" => dates,
        }.unwrap()
        .lazy()
        .with_column(col("timestamp").cast(DataType::Date))
        .collect()
        .unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.columns = Some(vec!["timestamp".to_string()]);
        
        let result = transform(df, TransformMode::Extract, params).unwrap();
        
        // Should have extracted year, month, day
        assert!(result.has_column("timestamp_year"));
        assert!(result.has_column("timestamp_month"));
        assert!(result.has_column("timestamp_day"));
    }
    
    #[test]
    fn test_transform_datetime_merged_into_extract() {
        // Test that @datetime mode now maps to Extract functionality
        // Create DataFrame with date column (already parsed)
        let dates = vec![19737, 19773, 19807]; // Days since epoch
        let df_inner = df! {
            "date_col" => dates,
        }.unwrap()
        .lazy()
        .with_column(col("date_col").cast(DataType::Date))
        .collect()
        .unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.columns = Some(vec!["date_col".to_string()]);
        
        // @datetime now maps to Extract mode, so it should extract datetime features
        let result = transform(df, TransformMode::Datetime, params).unwrap();
        
        // Should have extracted year, month, day (not just preserve the column)
        assert!(result.has_column("date_col"));
        assert!(result.has_column("date_col_year"));
        assert!(result.has_column("date_col_month"));
        assert!(result.has_column("date_col_day"));
    }
    
    #[test]
    fn test_transform_harmonize() {
        // Create DataFrame with numeric column
        let df_inner = df! {
            "weight" => &[150.0, 180.0, 200.0],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.columns = Some(vec!["weight".to_string()]);
        
        // Should succeed (numeric column)
        let result = transform(df, TransformMode::Harmonize, params).unwrap();
        
        assert!(result.has_column("weight"));
    }
    
    #[test]
    fn test_transform_missing_parameters() {
        let df = AdditoryDataFrame::empty();
        let params = UniversalParams::default();
        
        // @calc requires expression
        let result = transform(df.clone(), TransformMode::Calc, params.clone());
        assert!(result.is_err());
        
        // @sort requires by
        let result = transform(df.clone(), TransformMode::Sort, params.clone());
        assert!(result.is_err());
    }
    
    #[test]
    fn test_transform_knn() {
        // Create DataFrame with missing values
        let df_inner = df! {
            "a" => &[Some(1.0), None, Some(3.0), Some(4.0)],
            "b" => &[Some(5.0), Some(6.0), None, Some(8.0)],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        let mut params = UniversalParams::default();
        params.columns = Some(vec!["a".to_string(), "b".to_string()]);
        
        // Use k=3 (valid for 4 rows)
        let mut strategy = std::collections::HashMap::new();
        strategy.insert("k".to_string(), crate::core::StrategyValue::Number(3.0));
        params.strategy = Some(strategy);
        
        let result = transform(df, TransformMode::Knn, params);
        assert!(result.is_ok(), "KNN transform should succeed");
        
        let result_df = result.unwrap();
        
        // Check that missing values were imputed
        let a_col = result_df.column("a").unwrap().as_materialized_series();
        let b_col = result_df.column("b").unwrap().as_materialized_series();
        
        assert_eq!(a_col.null_count(), 0, "Column 'a' should have no nulls after imputation");
        assert_eq!(b_col.null_count(), 0, "Column 'b' should have no nulls after imputation");
    }
}
