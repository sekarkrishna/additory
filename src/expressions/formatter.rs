//! Pretty-printing expressions and reconciliations to .add file TOML strings.

use super::types::{ExpressionDef, ReconciliationDef};

/// Format an expression definition into a valid unified-format TOML string.
///
/// The output uses `[name]` top-level table with `expression`, `description`,
/// `category` fields and an optional `[name.inputs]` sub-table.
pub fn format_expression(expr: &ExpressionDef) -> String {
    let mut lines: Vec<String> = Vec::new();

    // [name] table
    lines.push(format!("[{}]", expr.name));
    lines.push(format!("expression = \"{}\"", expr.formula));
    lines.push(format!("description = \"{}\"", expr.description));
    lines.push(format!("category = \"{}\"", expr.category));
    if expr.output_column != expr.name {
        lines.push(format!("output_column = \"{}\"", expr.output_column));
    }
    lines.push(String::new());

    // [name.inputs] sub-table
    if !expr.inputs.is_empty() {
        lines.push(format!("[{}.inputs]", expr.name));
        let mut input_keys: Vec<&String> = expr.inputs.keys().collect();
        input_keys.sort();
        for input_name in input_keys {
            let input_def = &expr.inputs[input_name];
            let mut parts = vec![format!("type = \"{}\"", input_def.type_name)];
            if !input_def.unit.is_empty() {
                parts.push(format!("unit = \"{}\"", input_def.unit));
            }
            if !input_def.description.is_empty() {
                parts.push(format!("description = \"{}\"", input_def.description));
            }
            lines.push(format!("{} = {{ {} }}", input_name, parts.join(", ")));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

/// Format a reconciliation definition into a valid Reconciliation_Format TOML string.
///
/// The output uses `[reconciliation]`, optional `[aliases]`, optional `[groups]`
/// sections and can be parsed back by `parse_reconciliation_add_file`.
pub fn format_reconciliation(recon: &ReconciliationDef) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("[reconciliation]".to_string());
    lines.push(format!("name = \"{}\"", recon.name));
    lines.push(format!("description = \"{}\"", recon.description));
    lines.push(String::new());

    if !recon.aliases.is_empty() {
        lines.push("[aliases]".to_string());
        let mut alias_keys: Vec<&String> = recon.aliases.keys().collect();
        alias_keys.sort();
        for canonical in alias_keys {
            let variants = &recon.aliases[canonical];
            let variant_strs: Vec<String> = variants.iter().map(|v| format!("\"{}\"", v)).collect();
            lines.push(format!("{} = [{}]", canonical, variant_strs.join(", ")));
        }
        lines.push(String::new());
    }

    if !recon.groups.is_empty() {
        lines.push("[groups]".to_string());
        let mut group_keys: Vec<&String> = recon.groups.keys().collect();
        group_keys.sort();
        for parent in group_keys {
            let children = &recon.groups[parent];
            let child_strs: Vec<String> = children.iter().map(|c| format!("\"{}\"", c)).collect();
            lines.push(format!("{} = [{}]", parent, child_strs.join(", ")));
        }
        lines.push(String::new());
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use super::super::types::InputDef;

    #[test]
    fn test_format_expression() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "weight".to_string(),
            InputDef {
                type_name: "numeric".to_string(),
                unit: "kg".to_string(),
                description: "Weight in kg".to_string(),
            },
        );
        inputs.insert(
            "height".to_string(),
            InputDef {
                type_name: "numeric".to_string(),
                unit: "m".to_string(),
                description: "Height in m".to_string(),
            },
        );

        let expr = ExpressionDef {
            name: "bmi".to_string(),
            formula: "weight / (height ** 2)".to_string(),
            description: "Body Mass Index".to_string(),
            category: "medical".to_string(),
            output_column: "bmi".to_string(),
            inputs,
            source_file: None,
        };

        let output = format_expression(&expr);
        assert!(output.contains("[bmi]"));
        assert!(output.contains("expression = \"weight / (height ** 2)\""));
        assert!(output.contains("category = \"medical\""));
        assert!(output.contains("[bmi.inputs]"));
        // output_column == name, so it should NOT appear
        assert!(!output.contains("output_column"));
    }

    #[test]
    fn test_format_expression_custom_output_column() {
        let expr = ExpressionDef {
            name: "profit".to_string(),
            formula: "revenue - cost".to_string(),
            description: "Profit".to_string(),
            category: "finance".to_string(),
            output_column: "net_profit".to_string(),
            inputs: HashMap::new(),
            source_file: None,
        };

        let output = format_expression(&expr);
        assert!(output.contains("output_column = \"net_profit\""));
    }

    #[test]
    fn test_format_reconciliation() {
        let mut aliases = HashMap::new();
        aliases.insert(
            "gender".to_string(),
            vec!["sex".to_string(), "SEX".to_string()],
        );

        let mut groups = HashMap::new();
        groups.insert(
            "demographics".to_string(),
            vec!["age".to_string(), "gender".to_string()],
        );

        let recon = ReconciliationDef {
            name: "test_recon".to_string(),
            description: "Test reconciliation".to_string(),
            aliases,
            groups,
            source_file: None,
        };

        let output = format_reconciliation(&recon);
        assert!(output.contains("[reconciliation]"));
        assert!(output.contains("name = \"test_recon\""));
        assert!(output.contains("[aliases]"));
        assert!(output.contains("[groups]"));
    }

    #[test]
    fn test_format_reconciliation_empty_sections() {
        let recon = ReconciliationDef {
            name: "empty".to_string(),
            description: "No aliases or groups".to_string(),
            aliases: HashMap::new(),
            groups: HashMap::new(),
            source_file: None,
        };

        let output = format_reconciliation(&recon);
        assert!(output.contains("[reconciliation]"));
        assert!(!output.contains("[aliases]"));
        assert!(!output.contains("[groups]"));
    }
}
