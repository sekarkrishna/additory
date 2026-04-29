//! add.scan() module - Unified data inspection and analysis
//!
//! This module provides a unified entry point for data inspection, statistical profiling,
//! and lineage tracking. It consolidates the functionality previously split between
//! add.analyze() and standalone lineage reporting.

pub mod analyze;
pub mod lineage;
pub mod types;

pub use analyze::execute_analyze;
pub use lineage::execute_lineage;
pub use types::{
    ScanMode, ScanParams, ScanOutput, OutputFormat, RowSpec,
    AnalyzeParams, AnalyzeFocus, LineageParams, LineageFocus,
    LineageMetadata, Operation, ColumnSource, MetadataInfo,
    ScanError,
};

use crate::core::{AdditoryResult, AdditoryError, DataFrame};
use crate::utils::logging::Logger;
use polars::prelude::*;
use std::io::Cursor;

/// Main entry point for scan operations
pub fn execute_scan(
    df_bytes: &[u8],
    params: ScanParams,
    logger: &Logger,
) -> AdditoryResult<ScanOutput> {
    logger.log_result("add.scan()", &format!("Executing scan with mode: {:?}", params.mode));
    
    // Deserialize Arrow IPC bytes to Polars DataFrame
    let cursor = Cursor::new(df_bytes);
    let polars_df = polars::io::ipc::IpcReader::new(cursor)
        .finish()
        .map_err(|e| AdditoryError::operation(
            "Failed to deserialize DataFrame from Arrow IPC",
            &e.to_string()
        ))?;
    
    let df = DataFrame::from_polars(polars_df);
    
    // Route based on mode
    match params.mode {
        ScanMode::Analyze => {
            // Build AnalyzeParams
            let analyze_params = AnalyzeParams {
                columns: params.columns,
                where_clause: params.where_clause,
                rows: params.rows,
                focus: None, // TODO: Parse focus parameter
            };
            
            // Execute analyze
            let result_df = execute_analyze(df, &analyze_params, logger)?;
            
            // Convert output based on as_type
            convert_dataframe_output(result_df, params.as_type, logger)
        }
        ScanMode::Lineage => {
            // Parse lineage JSON
            let lineage_json = params.lineage_json.ok_or_else(|| {
                AdditoryError::operation(
                    "Missing lineage metadata",
                    "Lineage mode requires lineage_json parameter"
                )
            })?;
            
            let lineage: LineageMetadata = serde_json::from_str(&lineage_json)
                .map_err(|e| AdditoryError::operation(
                    "Failed to parse lineage JSON",
                    &e.to_string()
                ))?;
            
            // Build LineageParams
            let lineage_params = LineageParams {
                columns: params.columns,
                where_clause: params.where_clause,
                rows: params.rows,
                trace: params.trace,
                focus: None, // TODO: Parse focus parameter
            };
            
            // Execute lineage
            let report = execute_lineage(lineage, &lineage_params, logger)?;
            
            // Convert output based on as_type
            convert_text_output(report, params.as_type)
        }
        ScanMode::Set => {
            Err(AdditoryError::operation(
                "@set mode should be handled in Python",
                "The @set scan mode is intercepted in scan.py before Rust dispatch",
            ))
        }
    }
}

/// Convert DataFrame output to requested format
fn convert_dataframe_output(
    df: DataFrame,
    format: OutputFormat,
    logger: &Logger,
) -> AdditoryResult<ScanOutput> {
    match format {
        OutputFormat::DataFrame => {
            // Serialize to Arrow IPC bytes
            let polars_df = df.inner();
            let mut buffer = Vec::new();
            let mut writer = polars::io::ipc::IpcWriter::new(&mut buffer);
            writer.finish(&mut polars_df.clone())
                .map_err(|e| AdditoryError::operation(
                    "Failed to serialize DataFrame to Arrow IPC",
                    &e.to_string()
                ))?;
            
            logger.log_result("add.scan()", "Returning DataFrame output");
            Ok(ScanOutput::DataFrame(buffer))
        }
        OutputFormat::Dict => {
            // Convert DataFrame to dict structure
            // For now, return a simple JSON representation with column names and row count
            let polars_df = df.inner();
            let columns: Vec<String> = polars_df.get_column_names()
                .iter()
                .map(|s| s.to_string())
                .collect();
            
            let dict = serde_json::json!({
                "columns": columns,
                "rows": polars_df.height(),
                "data": format!("{}", polars_df)
            });
            
            logger.log_result("add.scan()", "Returning dict output");
            Ok(ScanOutput::Dict(dict.to_string()))
        }
        OutputFormat::Text => {
            // Convert DataFrame to text representation
            let polars_df = df.inner();
            let text = format!("{}", polars_df);
            
            logger.log_result("add.scan()", "Returning text output");
            Ok(ScanOutput::Text(text))
        }
    }
}

/// Convert text output to requested format
fn convert_text_output(
    text: String,
    format: OutputFormat,
) -> AdditoryResult<ScanOutput> {
    match format {
        OutputFormat::Text => Ok(ScanOutput::Text(text)),
        OutputFormat::Dict => {
            // Wrap text in a dict
            let dict = serde_json::json!({ "report": text });
            Ok(ScanOutput::Dict(dict.to_string()))
        }
        OutputFormat::DataFrame => {
            // Cannot convert text to DataFrame
            Err(AdditoryError::operation(
                "Cannot convert lineage report to DataFrame",
                "Use as_type='text' or as_type='dict' for lineage reports"
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_mode_from_str_analyze() {
        let mode = ScanMode::parse_mode("@analyze").unwrap();
        assert_eq!(mode, ScanMode::Analyze);
    }

    #[test]
    fn test_scan_mode_from_str_analyse() {
        let mode = ScanMode::parse_mode("@analyse").unwrap();
        assert_eq!(mode, ScanMode::Analyze);
    }

    #[test]
    fn test_scan_mode_from_str_lineage() {
        let mode = ScanMode::parse_mode("@lineage").unwrap();
        assert_eq!(mode, ScanMode::Lineage);
    }

    #[test]
    fn test_scan_mode_from_str_invalid() {
        let result = ScanMode::parse_mode("@invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_mode_from_str_set() {
        let mode = ScanMode::parse_mode("@set").unwrap();
        assert_eq!(mode, ScanMode::Set);
    }
}
