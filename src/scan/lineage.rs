//! Lineage report generation for scan module
//!
//! This module provides transformation tracking and explainability functionality.

use crate::core::AdditoryResult;
use crate::utils::logging::Logger;
use super::types::{LineageParams, LineageMetadata, Operation};
use std::collections::HashMap;

/// Execute lineage report generation
pub fn execute_lineage(
    lineage: LineageMetadata,
    _params: &LineageParams,
    logger: &Logger,
) -> AdditoryResult<String> {
    logger.log_result("add.scan(@lineage)", "Generating lineage report");
    
    let mut report = String::new();
    
    // Add header
    report.push_str(&format_header(&lineage));
    
    // Add operations
    for (idx, operation) in lineage.operations.iter().enumerate() {
        report.push_str(&format_operation(operation, idx + 1));
    }
    
    // Add dependency graph
    report.push_str(&format_dependency_graph(&lineage.column_sources));
    
    // Add summary
    report.push_str(&format_summary(&lineage));
    
    Ok(report)
}

/// Format report header with DataFrame info
fn format_header(lineage: &LineageMetadata) -> String {
    let total_ops = lineage.operations.len();
    let final_rows = if let Some(last_op) = lineage.operations.last() {
        last_op.rows_after
    } else {
        0
    };
    
    let total_cols = lineage.column_sources.len();
    
    format!(
        "═══════════════════════════════════════════════════════════════\n\
                          LINEAGE REPORT\n\
         ═══════════════════════════════════════════════════════════════\n\n\
         DataFrame: {} rows × {} columns\n\
         Operations: {} transformations applied\n\n",
        final_rows, total_cols, total_ops
    )
}

/// Format individual operation display with step numbers, timestamps, row changes
fn format_operation(operation: &Operation, step_num: usize) -> String {
    let mut output = String::new();
    
    output.push_str(&format!(
        "───────────────────────────────────────────────────────────────\n\
         Step {}: {} - {}\n\
         ───────────────────────────────────────────────────────────────\n",
        step_num,
        operation.operation_type,
        operation.timestamp.split('.').next().unwrap_or(&operation.timestamp)
    ));
    
    // Row changes
    let row_change = operation.rows_after - operation.rows_before;
    let row_change_str = if row_change == 0 {
        "no change".to_string()
    } else if row_change > 0 {
        format!("{} rows added", row_change)
    } else {
        format!("{} rows removed", row_change.abs())
    };
    
    output.push_str(&format!(
        "  Rows: {} → {} ({})\n",
        operation.rows_before, operation.rows_after, row_change_str
    ));
    
    // Warning for significant row removal
    if row_change < 0 {
        let pct_removed = (row_change.abs() as f64 / operation.rows_before as f64) * 100.0;
        if pct_removed > 10.0 {
            output.push_str(&format!("  ⚠ WARNING: {:.1}% of rows excluded\n", pct_removed));
        }
    }
    
    // Columns added
    if !operation.columns_added.is_empty() {
        output.push_str(&format!(
            "  Columns Added: {}\n",
            operation.columns_added.join(", ")
        ));
    }
    
    // Columns modified
    if !operation.columns_modified.is_empty() {
        output.push_str(&format!(
            "  Columns Modified: {}\n",
            operation.columns_modified.join(", ")
        ));
    }
    
    // Parameters
    if !operation.params.is_empty() {
        output.push_str("  \n  Parameters:\n");
        for (key, value) in &operation.params {
            output.push_str(&format!("    {}: {}\n", key, value));
        }
    }
    
    output.push('\n');
    output
}

/// Format dependency graph for column source visualization
fn format_dependency_graph(column_sources: &HashMap<String, super::types::ColumnSource>) -> String {
    let mut output = String::new();
    
    output.push_str(
        "───────────────────────────────────────────────────────────────\n\
         DEPENDENCY GRAPH\n\
         ───────────────────────────────────────────────────────────────\n\n"
    );
    
    for (col_name, source) in column_sources {
        match source.source_type.as_str() {
            "original" => {
                // Skip original columns in dependency graph
                continue;
            }
            "fetched" => {
                if let (Some(table), Some(col)) = (&source.source_table, &source.source_column) {
                    let deps = if !source.dependencies.is_empty() {
                        format!(" (via {})", source.dependencies.join(", "))
                    } else {
                        String::new()
                    };
                    output.push_str(&format!("{} ← {}.{}{}\n", col_name, table, col, deps));
                }
            }
            "calculated" => {
                if let Some(formula) = &source.formula {
                    output.push_str(&format!("{} ← {}\n", col_name, formula));
                }
            }
            _ => {}
        }
    }
    
    output.push('\n');
    output
}

/// Format data flow summary with warnings
fn format_summary(lineage: &LineageMetadata) -> String {
    let mut output = String::new();
    
    output.push_str(
        "───────────────────────────────────────────────────────────────\n\
         SUMMARY\n\
         ───────────────────────────────────────────────────────────────\n\n"
    );
    
    // Calculate total row changes
    if let (Some(first_op), Some(last_op)) = (lineage.operations.first(), lineage.operations.last()) {
        let initial_rows = first_op.rows_before;
        let final_rows = last_op.rows_after;
        let total_change = final_rows - initial_rows;
        
        output.push_str(&format!(
            "Data Flow: {} rows → {} rows\n",
            initial_rows, final_rows
        ));
        
        if total_change < 0 {
            let pct_removed = (total_change.abs() as f64 / initial_rows as f64) * 100.0;
            output.push_str(&format!(
                "{} rows ({:.1}%) removed during transformations\n\n",
                total_change.abs(), pct_removed
            ));
        } else if total_change > 0 {
            output.push_str(&format!(
                "{} rows added during transformations\n\n",
                total_change
            ));
        } else {
            output.push_str("No net change in row count\n\n");
        }
    }
    
    // Helpful hints
    output.push_str("Use focus='excluded' to see why rows were removed\n");
    output.push_str("Use focus='nulls' to analyze null value sources\n\n");
    
    output.push_str("═══════════════════════════════════════════════════════════════\n");
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::{Operation, ColumnSource, MetadataInfo};
    use std::collections::HashMap;

    #[test]
    fn test_format_header() {
        let mut operations = Vec::new();
        operations.push(Operation {
            operation_type: "add.to".to_string(),
            timestamp: "2024-01-15T10:30:45.123456".to_string(),
            rows_before: 1000,
            rows_after: 1000,
            columns_added: vec!["col1".to_string()],
            columns_modified: vec![],
            params: HashMap::new(),
        });
        
        let lineage = LineageMetadata {
            operations,
            column_sources: HashMap::new(),
            metadata: MetadataInfo {
                fresh_start: false,
                sampling_applied: false,
                compression_enabled: true,
            },
        };
        
        let header = format_header(&lineage);
        assert!(header.contains("LINEAGE REPORT"));
        assert!(header.contains("1000 rows"));
        assert!(header.contains("1 transformations"));
    }

    #[test]
    fn test_format_operation() {
        let operation = Operation {
            operation_type: "add.transform".to_string(),
            timestamp: "2024-01-15T10:30:45.123456".to_string(),
            rows_before: 1000,
            rows_after: 850,
            columns_added: vec![],
            columns_modified: vec![],
            params: {
                let mut params = HashMap::new();
                params.insert("mode".to_string(), serde_json::json!("@filter"));
                params.insert("where".to_string(), serde_json::json!("price > 100"));
                params
            },
        };
        
        let formatted = format_operation(&operation, 1);
        assert!(formatted.contains("Step 1"));
        assert!(formatted.contains("add.transform"));
        assert!(formatted.contains("1000 → 850"));
        assert!(formatted.contains("150 rows removed"));
        assert!(formatted.contains("WARNING"));
    }

    #[test]
    fn test_execute_lineage_simple() {
        let mut operations = Vec::new();
        operations.push(Operation {
            operation_type: "add.to".to_string(),
            timestamp: "2024-01-15T10:30:45.123456".to_string(),
            rows_before: 1000,
            rows_after: 1000,
            columns_added: vec!["customer_name".to_string()],
            columns_modified: vec![],
            params: HashMap::new(),
        });
        
        let mut column_sources = HashMap::new();
        column_sources.insert(
            "customer_name".to_string(),
            ColumnSource {
                source_type: "fetched".to_string(),
                source_table: Some("customers".to_string()),
                source_column: Some("name".to_string()),
                formula: None,
                dependencies: vec!["customer_id".to_string()],
            },
        );
        
        let lineage = LineageMetadata {
            operations,
            column_sources,
            metadata: MetadataInfo {
                fresh_start: false,
                sampling_applied: false,
                compression_enabled: true,
            },
        };
        
        let logger = Logger::new(false);
        let params = LineageParams {
            columns: None,
            where_clause: None,
            rows: None,
            trace: None,
            focus: None,
        };
        
        let report = execute_lineage(lineage, &params, &logger).unwrap();
        
        assert!(report.contains("LINEAGE REPORT"));
        assert!(report.contains("Step 1"));
        assert!(report.contains("add.to"));
        assert!(report.contains("DEPENDENCY GRAPH"));
        assert!(report.contains("customer_name"));
        assert!(report.contains("SUMMARY"));
    }
}

