//! Strategy parsing and validation for diff operations.
//!
//! Rejects unknown keys, validates output mode, rejects inline dict groups,
//! and detects exclude/carry conflicts.

use std::collections::HashMap;
use crate::core::{AdditoryResult, AdditoryError};
use super::types::*;

/// Known strategy keys.
const VALID_KEYS: &[&str] = &["output", "exclude", "carry", "context", "aliases", "groups"];

/// Parse and validate a strategy dictionary into a `StrategyConfig`.
pub fn parse_strategy(
    strategy: &HashMap<String, serde_json::Value>,
) -> AdditoryResult<StrategyConfig> {
    let mut config = StrategyConfig::default();

    // Reject unknown keys
    for key in strategy.keys() {
        if !VALID_KEYS.contains(&key.as_str()) {
            return Err(AdditoryError::StrategyParsing(
                format!("Unknown strategy key '{}'.", key),
                format!("Valid keys: {}.", VALID_KEYS.join(", ")),
            ));
        }
    }

    // Parse output mode
    if let Some(val) = strategy.get("output") {
        if let Some(s) = val.as_str() {
            config.output = match s.to_lowercase().as_str() {
                "summary" => OutputMode::Summary,
                "detail" => OutputMode::Detail,
                _ => {
                    return Err(AdditoryError::StrategyParsing(
                        format!("Invalid output mode '{}'.", s),
                        "Valid options: summary, detail.".to_string(),
                    ));
                }
            };
        } else {
            return Err(AdditoryError::StrategyParsing(
                "Output mode must be a string.".to_string(),
                "Valid options: summary, detail.".to_string(),
            ));
        }
    }

    // Parse exclude
    if let Some(val) = strategy.get("exclude") {
        config.exclude = parse_string_list(val, "exclude")?;
    }

    // Parse carry
    if let Some(val) = strategy.get("carry") {
        config.carry = parse_string_list(val, "carry")?;
    }

    // Parse context
    if let Some(val) = strategy.get("context") {
        config.context = parse_string_list(val, "context")?;
    }

    // Parse aliases
    if let Some(val) = strategy.get("aliases") {
        if let Some(s) = val.as_str() {
            config.aliases = Some(AliasSource::Registry(s.to_string()));
        } else if let Some(obj) = val.as_object() {
            let mut map = HashMap::new();
            for (canonical, variants_val) in obj {
                let variants = parse_string_list(variants_val, &format!("aliases.{}", canonical))?;
                map.insert(canonical.clone(), variants);
            }
            config.aliases = Some(AliasSource::Inline(map));
        } else {
            return Err(AdditoryError::StrategyParsing(
                "Aliases must be a string (registry name) or a dict (inline mapping).".to_string(),
                "Example: aliases='my_reconciliation' or aliases={'canonical': ['variant1', 'variant2']}".to_string(),
            ));
        }
    }

    // Parse groups — reject inline dicts, only accept registry name strings
    if let Some(val) = strategy.get("groups") {
        if let Some(s) = val.as_str() {
            config.groups = Some(s.to_string());
        } else if val.is_object() {
            return Err(AdditoryError::StrategyParsing(
                "Inline dict groups are not supported.".to_string(),
                "Groups must reference a reconciliation name. Example: groups='my_reconciliation'".to_string(),
            ));
        } else {
            return Err(AdditoryError::StrategyParsing(
                "Groups must be a string (reconciliation name).".to_string(),
                "Example: groups='my_reconciliation'".to_string(),
            ));
        }
    }

    // Detect exclude/carry conflicts
    let conflicts: Vec<String> = config
        .exclude
        .iter()
        .filter(|col| config.carry.contains(col))
        .cloned()
        .collect();

    if !conflicts.is_empty() {
        return Err(AdditoryError::StrategyParsing(
            format!(
                "Columns appear in both 'exclude' and 'carry': {}.",
                conflicts.join(", ")
            ),
            "A column cannot be both excluded and carried. Remove it from one list.".to_string(),
        ));
    }

    Ok(config)
}

/// Parse a JSON value as a list of strings.
fn parse_string_list(val: &serde_json::Value, field: &str) -> AdditoryResult<Vec<String>> {
    if let Some(arr) = val.as_array() {
        arr.iter()
            .map(|v| {
                v.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        AdditoryError::StrategyParsing(
                            format!("All items in '{}' must be strings.", field),
                            format!("Got non-string value: {}", v),
                        )
                    })
            })
            .collect()
    } else if let Some(s) = val.as_str() {
        // Single string → one-element list
        Ok(vec![s.to_string()])
    } else {
        Err(AdditoryError::StrategyParsing(
            format!("'{}' must be a string or list of strings.", field),
            format!("Got: {}", val),
        ))
    }
}
