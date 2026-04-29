//! Three-tier configuration resolution.
//!
//! Resolution order: expression folder → ~/.additory/ → built-in defaults.
//! Tracks which source files contributed to the merged configuration.

use std::path::Path;
use crate::core::{AdditoryResult, AdditoryError};
use super::types::*;
use super::parser::parse_config_toml;

const CONFIG_FILENAME: &str = "config.toml";

/// Load and merge configuration from the three-tier resolution chain.
///
/// 1. Expression folder `config.toml` (highest priority)
/// 2. `~/.additory/config.toml`
/// 3. Built-in defaults (lowest priority)
pub fn load_config(
    expr_folder: Option<&Path>,
    home_dir: Option<&Path>,
) -> AdditoryResult<ConfigData> {
    let mut merged = ConfigData::default();

    // Tier 3: built-in defaults (already set by Default impl)

    // Tier 2: ~/.additory/config.toml
    if let Some(home) = home_dir {
        let home_config = home.join(".additory").join(CONFIG_FILENAME);
        if home_config.exists() {
            let content = std::fs::read_to_string(&home_config).map_err(|e| {
                AdditoryError::Validation(
                    format!("Failed to read config file {}: {}", home_config.display(), e),
                    "Check file permissions and path.".to_string(),
                )
            })?;
            let parsed = parse_config_toml(&content, &home_config.to_string_lossy())?;
            merge_config(&mut merged, &parsed);
        }
    }

    // Tier 1: expression folder config.toml (highest priority)
    if let Some(expr) = expr_folder {
        let expr_config = expr.join(CONFIG_FILENAME);
        if expr_config.exists() {
            let content = std::fs::read_to_string(&expr_config).map_err(|e| {
                AdditoryError::Validation(
                    format!("Failed to read config file {}: {}", expr_config.display(), e),
                    "Check file permissions and path.".to_string(),
                )
            })?;
            let parsed = parse_config_toml(&content, &expr_config.to_string_lossy())?;
            merge_config(&mut merged, &parsed);
        }
    }

    Ok(merged)
}

/// Merge a higher-priority config into the base config.
/// Non-default values from `higher` override values in `base`.
fn merge_config(base: &mut ConfigData, higher: &ConfigData) {
    // Organization: higher wins if present
    if higher.organization.is_some() {
        base.organization = higher.organization.clone();
    }

    // Defaults: override individual fields if they differ from defaults
    let default_defaults = DefaultsConfig::default();

    if higher.defaults.seed != default_defaults.seed {
        base.defaults.seed = higher.defaults.seed.clone();
    }
    if higher.defaults.logging_level != default_defaults.logging_level {
        base.defaults.logging_level = higher.defaults.logging_level.clone();
    }
    if higher.defaults.as_type != default_defaults.as_type {
        base.defaults.as_type = higher.defaults.as_type.clone();
    }

    // Expressions: override individual fields
    let default_expr = ExpressionsConfig::default();

    if higher.expressions.folder.is_some() {
        base.expressions.folder = higher.expressions.folder.clone();
    }
    if higher.expressions.allow_user_override != default_expr.allow_user_override {
        base.expressions.allow_user_override = higher.expressions.allow_user_override;
    }
    if higher.expressions.fallback_to_inbuilt != default_expr.fallback_to_inbuilt {
        base.expressions.fallback_to_inbuilt = higher.expressions.fallback_to_inbuilt;
    }

    // Track source files
    for src in &higher.source_files {
        if !base.source_files.contains(src) {
            base.source_files.push(src.clone());
        }
    }
}

/// Produce a human-readable representation of the active configuration.
pub fn show_config(config: &ConfigData) -> String {
    let mut lines = Vec::new();

    lines.push("Active Configuration".to_string());
    lines.push("====================".to_string());

    if let Some(ref org) = config.organization {
        lines.push(String::new());
        lines.push("[organization]".to_string());
        lines.push(format!("  name       = \"{}\"", org.name));
        lines.push(format!("  managed_by = \"{}\"", org.managed_by));
    }

    lines.push(String::new());
    lines.push("[defaults]".to_string());
    lines.push(format!("  seed          = {}", config.defaults.seed));
    lines.push(format!("  logging_level = \"{}\"", config.defaults.logging_level));
    lines.push(format!("  as_type       = \"{}\"", config.defaults.as_type));

    lines.push(String::new());
    lines.push("[expressions]".to_string());
    if let Some(ref folder) = config.expressions.folder {
        lines.push(format!("  folder              = \"{}\"", folder));
    } else {
        lines.push("  folder              = (not set)".to_string());
    }
    lines.push(format!(
        "  allow_user_override = {}",
        config.expressions.allow_user_override
    ));
    lines.push(format!(
        "  fallback_to_inbuilt = {}",
        config.expressions.fallback_to_inbuilt
    ));

    if !config.source_files.is_empty() {
        lines.push(String::new());
        lines.push("Source files:".to_string());
        for src in &config.source_files {
            lines.push(format!("  - {}", src));
        }
    } else {
        lines.push(String::new());
        lines.push("Source files: (built-in defaults only)".to_string());
    }

    lines.join("\n")
}
