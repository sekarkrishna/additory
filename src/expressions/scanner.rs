//! Folder scanning and expression/reconciliation resolution.
//!
//! Scans directories for `.add` files, validates reserved names,
//! and resolves expressions/reconciliations by name with user-override priority.

use std::collections::HashMap;
use std::path::Path;

use crate::core::{AdditoryError, AdditoryResult};
use super::parser::{is_reconciliation_format, parse_add_file_content};
use super::types::*;

/// Scan a folder for `.add` files and return all expression definitions.
///
/// Validates that no expression uses a reserved name. Returns an error
/// if a reserved name or duplicate name is found.
pub fn scan_folder_for_expressions(
    folder: &Path,
) -> AdditoryResult<Vec<ExpressionDef>> {
    let mut expressions: Vec<ExpressionDef> = Vec::new();
    let mut seen_names: HashMap<String, String> = HashMap::new(); // name → source file

    if !folder.exists() || !folder.is_dir() {
        return Ok(expressions);
    }

    let mut add_files: Vec<_> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "add")
                .unwrap_or(false)
        })
        .map(|entry| entry.path())
        .collect();

    // Sort for deterministic ordering
    add_files.sort();

    for add_file in &add_files {
        let content = std::fs::read_to_string(add_file)?;
        let file_path_str = add_file.to_string_lossy().to_string();

        // Skip reconciliation-only files (they have no expressions)
        if is_reconciliation_format(&content) {
            continue;
        }

        let parsed = parse_add_file_content(&content, &file_path_str)?;

        if let ParsedAddFile::Expressions(exprs) = parsed {
            for expr in exprs {
                // Check reserved names
                if RESERVED_NAMES.contains(&expr.name.as_str()) {
                    return Err(AdditoryError::Validation(
                        format!(
                            "Expression name '{}' in {} is reserved",
                            expr.name, file_path_str
                        ),
                        format!(
                            "Reserved names: {}",
                            RESERVED_NAMES.join(", ")
                        ),
                    ));
                }

                // Check duplicates within this folder
                if let Some(prev_file) = seen_names.get(&expr.name) {
                    return Err(AdditoryError::Validation(
                        format!(
                            "Duplicate expression name '{}' found (defined in multiple .add files in {})",
                            expr.name,
                            folder.display()
                        ),
                        format!("Previously defined in {}", prev_file),
                    ));
                }

                seen_names.insert(expr.name.clone(), file_path_str.clone());
                expressions.push(expr);
            }
        }
    }

    Ok(expressions)
}

/// Resolve an expression by name, scanning user folder first then inbuilt folder.
///
/// Performs a fresh folder scan each time (no caching). User folder takes
/// priority over inbuilt (user overrides inbuilt).
pub fn resolve_expression_by_name(
    name: &str,
    user_folder: Option<&Path>,
    inbuilt_folder: &Path,
) -> AdditoryResult<Option<ExpressionDef>> {
    // Scan user folder first
    if let Some(uf) = user_folder {
        let user_exprs = scan_folder_for_expressions(uf)?;
        if let Some(expr) = user_exprs.into_iter().find(|e| e.name == name) {
            return Ok(Some(expr));
        }
    }

    // Scan inbuilt folder
    let inbuilt_exprs = scan_folder_for_expressions(inbuilt_folder)?;
    if let Some(expr) = inbuilt_exprs.into_iter().find(|e| e.name == name) {
        return Ok(Some(expr));
    }

    Ok(None)
}

/// Resolve a reconciliation by name, scanning user folder first then inbuilt folder.
///
/// Scans `.add` files for `[reconciliation]` sections whose `name` field matches.
pub fn resolve_reconciliation_by_name(
    name: &str,
    user_folder: Option<&Path>,
    inbuilt_folder: &Path,
) -> AdditoryResult<Option<ReconciliationDef>> {
    let folders: Vec<&Path> = user_folder
        .into_iter()
        .chain(std::iter::once(inbuilt_folder))
        .collect();

    for folder in folders {
        if !folder.exists() || !folder.is_dir() {
            continue;
        }

        let mut add_files: Vec<_> = std::fs::read_dir(folder)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|ext| ext == "add")
                    .unwrap_or(false)
            })
            .map(|entry| entry.path())
            .collect();

        add_files.sort();

        for add_file in &add_files {
            let content = std::fs::read_to_string(add_file)?;
            if !is_reconciliation_format(&content) {
                continue;
            }

            let file_path_str = add_file.to_string_lossy().to_string();
            let parsed = parse_add_file_content(&content, &file_path_str)?;

            if let ParsedAddFile::Reconciliation(recon) = parsed {
                if recon.name == name {
                    return Ok(Some(recon));
                }
            }
        }
    }

    Ok(None)
}

/// List all available expression names from user folder and inbuilt folder.
///
/// User folder names come first; inbuilt names that don't conflict are appended.
pub fn list_expression_names(
    user_folder: Option<&Path>,
    inbuilt_folder: &Path,
) -> AdditoryResult<Vec<String>> {
    let mut names: Vec<String> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // User folder first
    if let Some(uf) = user_folder {
        let user_exprs = scan_folder_for_expressions(uf)?;
        for expr in user_exprs {
            if seen.insert(expr.name.clone()) {
                names.push(expr.name);
            }
        }
    }

    // Inbuilt folder
    let inbuilt_exprs = scan_folder_for_expressions(inbuilt_folder)?;
    for expr in inbuilt_exprs {
        if seen.insert(expr.name.clone()) {
            names.push(expr.name);
        }
    }

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_temp_add_file(dir: &Path, filename: &str, content: &str) {
        fs::write(dir.join(filename), content).unwrap();
    }

    #[test]
    fn test_scan_empty_folder() {
        let dir = tempfile::tempdir().unwrap();
        let exprs = scan_folder_for_expressions(dir.path()).unwrap();
        assert!(exprs.is_empty());
    }

    #[test]
    fn test_scan_nonexistent_folder() {
        let exprs = scan_folder_for_expressions(Path::new("/nonexistent/path")).unwrap();
        assert!(exprs.is_empty());
    }

    #[test]
    fn test_scan_unified_add_file() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_add_file(
            dir.path(),
            "test.add",
            r#"[profit]
expression = "revenue - cost"
description = "Profit"
category = "core"
"#,
        );

        let exprs = scan_folder_for_expressions(dir.path()).unwrap();
        assert_eq!(exprs.len(), 1);
        assert_eq!(exprs[0].name, "profit");
        assert_eq!(exprs[0].category, "core");
    }

    #[test]
    fn test_scan_reserved_name_rejected() {
        let dir = tempfile::tempdir().unwrap();
        create_temp_add_file(
            dir.path(),
            "bad.add",
            r#"[transform]
expression = "a + b"
description = "bad"
category = "test"
"#,
        );

        let result = scan_folder_for_expressions(dir.path());
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("reserved"));
    }

    #[test]
    fn test_resolve_user_overrides_inbuilt() {
        let user_dir = tempfile::tempdir().unwrap();
        let inbuilt_dir = tempfile::tempdir().unwrap();

        create_temp_add_file(
            user_dir.path(),
            "custom.add",
            r#"[profit]
expression = "revenue - cost - tax"
description = "Profit after tax (user)"
category = "custom"
"#,
        );

        create_temp_add_file(
            inbuilt_dir.path(),
            "core.add",
            r#"[profit]
expression = "revenue - cost"
description = "Profit (inbuilt)"
category = "core"
"#,
        );

        let result = resolve_expression_by_name(
            "profit",
            Some(user_dir.path()),
            inbuilt_dir.path(),
        )
        .unwrap();

        assert!(result.is_some());
        let expr = result.unwrap();
        assert!(expr.description.contains("user"));
    }

    #[test]
    fn test_resolve_falls_back_to_inbuilt() {
        let inbuilt_dir = tempfile::tempdir().unwrap();

        create_temp_add_file(
            inbuilt_dir.path(),
            "core.add",
            r#"[profit]
expression = "revenue - cost"
description = "Profit (inbuilt)"
category = "core"
"#,
        );

        let result = resolve_expression_by_name("profit", None, inbuilt_dir.path()).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_resolve_not_found() {
        let inbuilt_dir = tempfile::tempdir().unwrap();
        let result =
            resolve_expression_by_name("nonexistent", None, inbuilt_dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_expression_names() {
        let user_dir = tempfile::tempdir().unwrap();
        let inbuilt_dir = tempfile::tempdir().unwrap();

        create_temp_add_file(
            user_dir.path(),
            "custom.add",
            r#"[custom_calc]
expression = "a + b"
description = "Custom"
category = "custom"
"#,
        );

        create_temp_add_file(
            inbuilt_dir.path(),
            "core.add",
            r#"[profit]
expression = "revenue - cost"
description = "Profit"
category = "core"
"#,
        );

        let names =
            list_expression_names(Some(user_dir.path()), inbuilt_dir.path()).unwrap();
        assert!(names.contains(&"custom_calc".to_string()));
        assert!(names.contains(&"profit".to_string()));
    }
}
