//! .add file parsing — Unified TOML and Reconciliation formats.
//!
//! Ports the Python `loader.py` parsing logic to Rust.
//! All expression files use the unified TOML format (top-level tables per expression).

use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

use crate::core::{AdditoryError, AdditoryResult};
use super::identifiers::extract_identifiers;
use super::types::*;

/// Sections to skip when iterating top-level TOML tables.
const SKIP_SECTIONS: &[&str] = &["reconciliation", "aliases", "groups"];

lazy_static! {
    /// Compiled regex for validating expression formula characters.
    static ref SAFE_PATTERN: Regex = Regex::new(EXPRESSION_SAFE_PATTERN).unwrap();

    /// Detect `[reconciliation]` section header.
    static ref RECONCILIATION_RE: Regex = Regex::new(r"(?m)^\[reconciliation\]\s*$").unwrap();
}

/// Validate that a formula string contains only allowed characters.
fn validate_expression_content(
    expression: &str,
    name: &str,
    file_path: &str,
) -> AdditoryResult<()> {
    if !SAFE_PATTERN.is_match(expression) {
        return Err(AdditoryError::Validation(
            format!(
                "Expression '{}' in {} contains invalid characters",
                name, file_path
            ),
            format!(
                "Expressions may only contain column names, operators (+,-,*,/,%,**), \
                 numbers, and parentheses. Got: {:?}",
                expression
            ),
        ));
    }
    Ok(())
}

// ─── Format detection ────────────────────────────────────────────────

/// Check whether content contains a `[reconciliation]` section.
pub fn is_reconciliation_format(content: &str) -> bool {
    RECONCILIATION_RE.is_match(content)
}

// ─── Unified format parser ───────────────────────────────────────────

/// Parse a unified-format .add file using TOML.
///
/// Each top-level table (excluding `reconciliation`, `aliases`, `groups`)
/// is treated as an expression definition with required `expression`,
/// `description`, and `category` fields.
pub fn parse_unified_add_file(
    content: &str,
    file_path: &str,
) -> AdditoryResult<Vec<ExpressionDef>> {
    let data: toml::Value = toml::from_str(content).map_err(|e| {
        AdditoryError::Validation(
            format!("Failed to parse .add file {}: {}", file_path, e),
            "Check that the file is valid TOML.".to_string(),
        )
    })?;

    let table = data.as_table().ok_or_else(|| {
        AdditoryError::Validation(
            format!("Invalid TOML structure in {}", file_path),
            "Expected a TOML table at the top level.".to_string(),
        )
    })?;

    let mut expressions = Vec::new();

    for (name, value) in table {
        if SKIP_SECTIONS.contains(&name.as_str()) {
            continue;
        }

        let expr_table = match value.as_table() {
            Some(t) => t,
            None => continue, // skip non-table values
        };

        // Reject removed fields
        for removed in &["sha", "requires"] {
            if expr_table.contains_key(*removed) {
                return Err(AdditoryError::Validation(
                    format!(
                        "Field '{}' in [{}] of {} is no longer supported",
                        removed, name, file_path
                    ),
                    format!("Remove '{}' from the expression definition.", removed),
                ));
            }
        }

        // Required fields
        let formula = expr_table
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AdditoryError::Validation(
                    format!(
                        "Missing required field 'expression' in [{}] of {}",
                        name, file_path
                    ),
                    "Every expression table must have an 'expression' field.".to_string(),
                )
            })?
            .to_string();

        let description = expr_table
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AdditoryError::Validation(
                    format!(
                        "Missing required field 'description' in [{}] of {}",
                        name, file_path
                    ),
                    "Every expression table must have a 'description' field.".to_string(),
                )
            })?
            .to_string();

        let category = expr_table
            .get("category")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                AdditoryError::Validation(
                    format!(
                        "Missing required field 'category' in [{}] of {}",
                        name, file_path
                    ),
                    "Every expression table must have a 'category' field.".to_string(),
                )
            })?
            .to_string();

        validate_expression_content(&formula, name, file_path)?;

        let output_column = expr_table
            .get("output_column")
            .and_then(|v| v.as_str())
            .unwrap_or(name)
            .to_string();

        // Parse optional [name.inputs] sub-table
        let inputs = if let Some(inputs_val) = expr_table.get("inputs") {
            parse_inputs_table(inputs_val, name, file_path)?
        } else {
            // Infer inputs from formula identifiers
            let identifiers = extract_identifiers(&formula);
            identifiers
                .into_iter()
                .map(|ident| {
                    (
                        ident,
                        InputDef {
                            type_name: "numeric".to_string(),
                            unit: String::new(),
                            description: String::new(),
                        },
                    )
                })
                .collect()
        };

        expressions.push(ExpressionDef {
            name: name.clone(),
            formula,
            description,
            category,
            output_column,
            inputs,
            source_file: Some(file_path.to_string()),
        });
    }

    Ok(expressions)
}

/// Parse an inputs sub-table from TOML.
fn parse_inputs_table(
    inputs_val: &toml::Value,
    expr_name: &str,
    file_path: &str,
) -> AdditoryResult<HashMap<String, InputDef>> {
    let inputs_table = inputs_val.as_table().ok_or_else(|| {
        AdditoryError::Validation(
            format!(
                "[{}.inputs] in {} is not a table",
                expr_name, file_path
            ),
            "Expected [name.inputs] to be a TOML table.".to_string(),
        )
    })?;

    let mut inputs = HashMap::new();
    for (input_name, input_val) in inputs_table {
        if let Some(tbl) = input_val.as_table() {
            inputs.insert(
                input_name.clone(),
                InputDef {
                    type_name: tbl
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("numeric")
                        .to_string(),
                    unit: tbl
                        .get("unit")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: tbl
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                },
            );
        } else {
            // Bare value — treat as description-only numeric input
            inputs.insert(
                input_name.clone(),
                InputDef {
                    type_name: "numeric".to_string(),
                    unit: String::new(),
                    description: input_val.as_str().unwrap_or("").to_string(),
                },
            );
        }
    }

    Ok(inputs)
}

// ─── Reconciliation format parser ────────────────────────────────────

/// Parse a reconciliation-format .add file using TOML.
///
/// Expected sections: `[reconciliation]`, optional `[aliases]`, optional `[groups]`.
pub fn parse_reconciliation_add_file(
    content: &str,
    file_path: &str,
) -> AdditoryResult<ReconciliationDef> {
    let data: toml::Value = toml::from_str(content).map_err(|e| {
        AdditoryError::Validation(
            format!(
                "Failed to parse reconciliation .add file {}: {}",
                file_path, e
            ),
            "Check that the file is valid TOML.".to_string(),
        )
    })?;

    let table = data.as_table().ok_or_else(|| {
        AdditoryError::Validation(
            format!("Invalid TOML structure in {}", file_path),
            "Expected a TOML table at the top level.".to_string(),
        )
    })?;

    let recon = table.get("reconciliation").and_then(|v| v.as_table()).ok_or_else(|| {
        AdditoryError::Validation(
            format!("Missing [reconciliation] section in {}", file_path),
            "A reconciliation .add file must have a [reconciliation] section.".to_string(),
        )
    })?;

    let name = recon
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AdditoryError::Validation(
                format!(
                    "Missing required field 'name' in [reconciliation] section of {}",
                    file_path
                ),
                "Every reconciliation .add file must have a name.\n\
                 Example:\n  [reconciliation]\n  name = \"my_aliases\""
                    .to_string(),
            )
        })?
        .to_string();

    let description = recon
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Parse aliases: key = canonical, value = list of variants
    let aliases = parse_string_list_map(table.get("aliases"));

    // Parse groups: key = parent, value = list of children
    let groups = parse_string_list_map(table.get("groups"));

    Ok(ReconciliationDef {
        name,
        description,
        aliases,
        groups,
        source_file: Some(file_path.to_string()),
    })
}

/// Helper: parse a TOML table where each value is a string or array of strings.
fn parse_string_list_map(value: Option<&toml::Value>) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();
    if let Some(toml::Value::Table(tbl)) = value {
        for (key, val) in tbl {
            match val {
                toml::Value::Array(arr) => {
                    map.insert(
                        key.clone(),
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect(),
                    );
                }
                toml::Value::String(s) => {
                    map.insert(key.clone(), vec![s.clone()]);
                }
                _ => {
                    map.insert(key.clone(), vec![val.to_string()]);
                }
            }
        }
    }
    map
}

// ─── Auto-detect dispatcher ─────────────────────────────────────────

/// Parse .add file content, auto-detecting the format.
///
/// Returns `ParsedAddFile::Reconciliation` for reconciliation-only files,
/// `ParsedAddFile::Expressions` for unified expression files.
pub fn parse_add_file_content(
    content: &str,
    file_path: &str,
) -> AdditoryResult<ParsedAddFile> {
    // Reconciliation-only file
    if is_reconciliation_format(content) {
        let recon = parse_reconciliation_add_file(content, file_path)?;
        return Ok(ParsedAddFile::Reconciliation(recon));
    }

    // Unified format (all non-reconciliation files)
    let exprs = parse_unified_add_file(content, file_path)?;
    Ok(ParsedAddFile::Expressions(exprs))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_reconciliation() {
        let content = "[reconciliation]\nname = \"test\"\n";
        assert!(is_reconciliation_format(content));
    }

    #[test]
    fn test_detect_non_reconciliation() {
        let content = "[bmi]\nexpression = \"weight / height\"\n";
        assert!(!is_reconciliation_format(content));
    }

    // ─── parse_unified_add_file tests ────────────────────────────────

    #[test]
    fn test_parse_unified_explicit_inputs() {
        let content = r#"[bmi]
expression = "weight / (height ** 2)"
description = "Body Mass Index"
category = "medical"

[bmi.inputs]
weight = { type = "numeric", unit = "kg", description = "Body weight" }
height = { type = "numeric", unit = "m", description = "Height" }
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].name, "bmi");
        assert_eq!(exprs[0].formula, "weight / (height ** 2)");
        assert_eq!(exprs[0].description, "Body Mass Index");
        assert_eq!(exprs[0].category, "medical");
        assert_eq!(exprs[0].output_column, "bmi");
        assert!(exprs[0].inputs.contains_key("weight"));
        assert!(exprs[0].inputs.contains_key("height"));
        assert_eq!(exprs[0].inputs["weight"].unit, "kg");
        assert_eq!(exprs[0].inputs["height"].unit, "m");
        assert_eq!(exprs[0].inputs["weight"].type_name, "numeric");
    }

    #[test]
    fn test_parse_unified_inferred_inputs() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Calculate profit"
category = "finance"
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].name, "profit");
        // Inputs inferred from formula
        assert!(exprs[0].inputs.contains_key("revenue"));
        assert!(exprs[0].inputs.contains_key("cost"));
        assert_eq!(exprs[0].inputs.len(), 2);
        // Inferred inputs default to numeric with empty unit
        assert_eq!(exprs[0].inputs["revenue"].type_name, "numeric");
        assert_eq!(exprs[0].inputs["revenue"].unit, "");
    }

    #[test]
    fn test_parse_unified_custom_output_column() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Calculate profit"
category = "finance"
output_column = "net_profit"
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs[0].output_column, "net_profit");
    }

    #[test]
    fn test_parse_unified_default_output_column() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Calculate profit"
category = "finance"
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs[0].output_column, "profit");
    }

    #[test]
    fn test_parse_unified_multiple_expressions() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Profit"
category = "finance"

[margin]
expression = "profit / revenue"
description = "Margin"
category = "finance"
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs.len(), 2);
        let names: Vec<&str> = exprs.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"profit"));
        assert!(names.contains(&"margin"));
    }

    #[test]
    fn test_parse_unified_rejects_sha_field() {
        let content = r#"[bmi]
expression = "weight / (height ** 2)"
description = "BMI"
category = "medical"
sha = "abc123"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("sha"));
        assert!(err_msg.contains("no longer supported"));
    }

    #[test]
    fn test_parse_unified_rejects_requires_field() {
        let content = r#"[bmi]
expression = "weight / (height ** 2)"
description = "BMI"
category = "medical"
requires = "numpy"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("requires"));
        assert!(err_msg.contains("no longer supported"));
    }

    #[test]
    fn test_parse_unified_missing_expression_field() {
        let content = r#"[bmi]
description = "BMI"
category = "medical"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("expression"));
    }

    #[test]
    fn test_parse_unified_missing_description_field() {
        let content = r#"[bmi]
expression = "weight / (height ** 2)"
category = "medical"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("description"));
    }

    #[test]
    fn test_parse_unified_missing_category_field() {
        let content = r#"[bmi]
expression = "weight / (height ** 2)"
description = "BMI"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("category"));
    }

    #[test]
    fn test_parse_unified_invalid_formula() {
        let content = r#"[bad]
expression = "weight / height; DROP TABLE"
description = "bad expression"
category = "test"
"#;
        let result = parse_unified_add_file(content, "test.add");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unified_skips_reconciliation_sections() {
        let content = r#"[reconciliation]
name = "test"
description = "test"

[aliases]
gender = ["sex", "SEX"]

[profit]
expression = "revenue - cost"
description = "Profit"
category = "finance"
"#;
        let exprs = parse_unified_add_file(content, "test.add").unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].name, "profit");
    }

    #[test]
    fn test_parse_unified_invalid_toml() {
        let content = "this is not valid toml [[[";
        let result = parse_unified_add_file(content, "my_file.add");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("my_file.add"));
    }

    #[test]
    fn test_parse_unified_source_file_set() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Profit"
category = "finance"
"#;
        let exprs = parse_unified_add_file(content, "path/to/test.add").unwrap();
        assert_eq!(exprs[0].source_file, Some("path/to/test.add".to_string()));
    }

    // ─── Reconciliation tests ────────────────────────────────────────

    #[test]
    fn test_parse_reconciliation() {
        let content = r#"[reconciliation]
name = "test_recon"
description = "Test reconciliation"

[aliases]
gender = ["sex", "SEX", "Gender"]

[groups]
demographics = ["age", "gender", "race"]
"#;
        let recon = parse_reconciliation_add_file(content, "test.add").unwrap();
        assert_eq!(recon.name, "test_recon");
        assert_eq!(recon.aliases["gender"], vec!["sex", "SEX", "Gender"]);
        assert_eq!(recon.groups["demographics"], vec!["age", "gender", "race"]);
    }

    // ─── Auto-detect dispatcher tests ────────────────────────────────

    #[test]
    fn test_auto_detect_unified() {
        let content = r#"[profit]
expression = "revenue - cost"
description = "Profit"
category = "finance"
"#;
        let parsed = parse_add_file_content(content, "test.add").unwrap();
        match parsed {
            ParsedAddFile::Expressions(exprs) => {
                assert_eq!(exprs[0].name, "profit");
            }
            _ => panic!("Expected Expressions"),
        }
    }

    #[test]
    fn test_auto_detect_reconciliation() {
        let content = r#"[reconciliation]
name = "test"
description = "test"
"#;
        let parsed = parse_add_file_content(content, "test.add").unwrap();
        match parsed {
            ParsedAddFile::Reconciliation(r) => {
                assert_eq!(r.name, "test");
            }
            _ => panic!("Expected Reconciliation"),
        }
    }
}
