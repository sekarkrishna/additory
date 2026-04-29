//! Configuration module for the config.toml system
//!
//! Provides organizational configuration with three-tier resolution:
//! expression folder → ~/.additory/ → built-in defaults.

pub mod types;
pub mod parser;
pub mod resolver;

pub use types::*;
pub use parser::parse_config_toml;
pub use resolver::{load_config, show_config};

/// Prefix an error message with organization context when available.
///
/// When `organization.name` or `organization.managed_by` is set,
/// error messages are prefixed with `[OrgName]` for traceability.
pub fn prefix_with_org(config: &ConfigData, message: &str) -> String {
    if let Some(ref org) = config.organization {
        if !org.name.is_empty() {
            return format!("[{}] {}", org.name, message);
        }
        if !org.managed_by.is_empty() {
            return format!("[managed by: {}] {}", org.managed_by, message);
        }
    }
    message.to_string()
}
