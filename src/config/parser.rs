//! TOML parsing with field validation for config.toml files.
//!
//! Validates organization name/managed_by ≤ 128 chars, logging_level,
//! seed format, and as_type against valid options.

use crate::core::AdditoryResult;
use crate::core::AdditoryError;
use super::types::*;

/// Intermediate TOML structure for deserialization before validation.
#[derive(Debug, serde::Deserialize)]
struct RawConfig {
    organization: Option<RawOrganization>,
    defaults: Option<RawDefaults>,
    expressions: Option<RawExpressions>,
}

#[derive(Debug, serde::Deserialize)]
struct RawOrganization {
    name: Option<String>,
    managed_by: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RawDefaults {
    seed: Option<toml::Value>,
    logging_level: Option<String>,
    as_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct RawExpressions {
    folder: Option<String>,
    allow_user_override: Option<bool>,
    fallback_to_inbuilt: Option<bool>,
}

/// Parse and validate a config.toml string into a `ConfigData`.
///
/// Returns errors with field name and valid options when validation fails.
pub fn parse_config_toml(content: &str, file_path: &str) -> AdditoryResult<ConfigData> {
    let raw: RawConfig = toml::from_str(content).map_err(|e| {
        AdditoryError::Validation(
            format!("Invalid TOML in {}: {}", file_path, e),
            "Ensure the file contains valid TOML syntax.".to_string(),
        )
    })?;

    let organization = if let Some(org) = raw.organization {
        let name = org.name.unwrap_or_default();
        let managed_by = org.managed_by.unwrap_or_default();

        if name.len() > 128 {
            return Err(AdditoryError::Validation(
                format!(
                    "Config error in {}:\n  Field 'organization.name' exceeds 128 characters (got {}).",
                    file_path,
                    name.len()
                ),
                "Shorten the organization name to 128 characters or fewer.".to_string(),
            ));
        }
        if managed_by.len() > 128 {
            return Err(AdditoryError::Validation(
                format!(
                    "Config error in {}:\n  Field 'organization.managed_by' exceeds 128 characters (got {}).",
                    file_path,
                    managed_by.len()
                ),
                "Shorten the managed_by value to 128 characters or fewer.".to_string(),
            ));
        }

        Some(OrganizationConfig { name, managed_by })
    } else {
        None
    };

    let defaults = if let Some(defs) = raw.defaults {
        let seed = if let Some(seed_val) = defs.seed {
            parse_seed(&seed_val, file_path)?
        } else {
            SeedConfig::Fixed(42)
        };

        let logging_level = if let Some(ref level_str) = defs.logging_level {
            parse_logging_level(level_str, file_path)?
        } else {
            LoggingLevel::Warning
        };

        let as_type = if let Some(ref as_str) = defs.as_type {
            parse_as_type(as_str, file_path)?
        } else {
            AsTypeDefault::Polars
        };

        DefaultsConfig {
            seed,
            logging_level,
            as_type,
        }
    } else {
        DefaultsConfig::default()
    };

    let expressions = if let Some(expr) = raw.expressions {
        ExpressionsConfig {
            folder: expr.folder,
            allow_user_override: expr.allow_user_override.unwrap_or(true),
            fallback_to_inbuilt: expr.fallback_to_inbuilt.unwrap_or(true),
        }
    } else {
        ExpressionsConfig::default()
    };

    Ok(ConfigData {
        organization,
        defaults,
        expressions,
        source_files: vec![file_path.to_string()],
    })
}

fn parse_seed(val: &toml::Value, file_path: &str) -> AdditoryResult<SeedConfig> {
    match val {
        toml::Value::Integer(n) => {
            if *n < 0 {
                return Err(AdditoryError::Validation(
                    format!(
                        "Config error in {}:\n  Field 'defaults.seed' has invalid value '{}'.",
                        file_path, n
                    ),
                    "Seed must be a non-negative integer or the string \"auto\".\n  Example:\n    [defaults]\n    seed = 42".to_string(),
                ));
            }
            Ok(SeedConfig::Fixed(*n as u64))
        }
        toml::Value::String(s) if s.eq_ignore_ascii_case("auto") => Ok(SeedConfig::Auto),
        other => Err(AdditoryError::Validation(
            format!(
                "Config error in {}:\n  Field 'defaults.seed' has invalid value '{}'.",
                file_path, other
            ),
            "Valid formats: an integer (e.g. 42) or the string \"auto\".\n  Example:\n    [defaults]\n    seed = 42".to_string(),
        )),
    }
}

fn parse_logging_level(s: &str, file_path: &str) -> AdditoryResult<LoggingLevel> {
    match s.to_lowercase().as_str() {
        "debug" => Ok(LoggingLevel::Debug),
        "info" => Ok(LoggingLevel::Info),
        "warning" => Ok(LoggingLevel::Warning),
        "error" => Ok(LoggingLevel::Error),
        "off" => Ok(LoggingLevel::Off),
        _ => Err(AdditoryError::Validation(
            format!(
                "Config error in {}:\n  Field 'logging_level' has invalid value '{}'.",
                file_path, s
            ),
            "Valid options: debug, info, warning, error, off.\n  Example:\n    [defaults]\n    logging_level = \"info\"".to_string(),
        )),
    }
}

fn parse_as_type(s: &str, file_path: &str) -> AdditoryResult<AsTypeDefault> {
    match s.to_lowercase().as_str() {
        "polars" => Ok(AsTypeDefault::Polars),
        "pandas" => Ok(AsTypeDefault::Pandas),
        "auto" => Ok(AsTypeDefault::Auto),
        _ => Err(AdditoryError::Validation(
            format!(
                "Config error in {}:\n  Field 'as_type' has invalid value '{}'.",
                file_path, s
            ),
            "Valid options: polars, pandas, auto.\n  Example:\n    [defaults]\n    as_type = \"polars\"".to_string(),
        )),
    }
}
