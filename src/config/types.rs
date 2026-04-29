//! Configuration types for the config.toml system
//!
//! Defines the core data structures for organizational configuration,
//! defaults, and expression folder settings.

use serde::{Deserialize, Serialize};

/// The merged configuration from all config.toml sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    /// Optional organization branding.
    pub organization: Option<OrganizationConfig>,
    /// Default values for seed, logging, and output type.
    pub defaults: DefaultsConfig,
    /// Expression folder and override settings.
    pub expressions: ExpressionsConfig,
    /// Which config.toml files contributed to this merged config.
    pub source_files: Vec<String>,
}

/// Organization branding fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// Organization name (max 128 chars).
    pub name: String,
    /// Managed-by contact (max 128 chars).
    pub managed_by: String,
}

/// Default values for additory operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Random seed: fixed integer or "auto".
    pub seed: SeedConfig,
    /// Logging verbosity level.
    pub logging_level: LoggingLevel,
    /// Default output DataFrame type.
    pub as_type: AsTypeDefault,
}

/// Seed configuration: either a fixed integer or auto (random).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedConfig {
    Fixed(u64),
    Auto,
}

/// Logging verbosity levels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoggingLevel {
    Debug,
    Info,
    Warning,
    Error,
    Off,
}

/// Default output DataFrame type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AsTypeDefault {
    Polars,
    Pandas,
    Auto,
}

/// Expression folder and override settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionsConfig {
    /// Optional path to expression folder.
    pub folder: Option<String>,
    /// Whether user can override the expression folder via `add.scan('@set', ...)`.
    pub allow_user_override: bool,
    /// Whether to fall back to inbuilt expressions when not found in user folder.
    pub fallback_to_inbuilt: bool,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            seed: SeedConfig::Fixed(42),
            logging_level: LoggingLevel::Warning,
            as_type: AsTypeDefault::Polars,
        }
    }
}

impl Default for ExpressionsConfig {
    fn default() -> Self {
        Self {
            folder: None,
            allow_user_override: true,
            fallback_to_inbuilt: true,
        }
    }
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            organization: None,
            defaults: DefaultsConfig::default(),
            expressions: ExpressionsConfig::default(),
            source_files: Vec::new(),
        }
    }
}

impl std::fmt::Display for LoggingLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggingLevel::Debug => write!(f, "debug"),
            LoggingLevel::Info => write!(f, "info"),
            LoggingLevel::Warning => write!(f, "warning"),
            LoggingLevel::Error => write!(f, "error"),
            LoggingLevel::Off => write!(f, "off"),
        }
    }
}

impl std::fmt::Display for SeedConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedConfig::Fixed(n) => write!(f, "{}", n),
            SeedConfig::Auto => write!(f, "auto"),
        }
    }
}

impl std::fmt::Display for AsTypeDefault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsTypeDefault::Polars => write!(f, "polars"),
            AsTypeDefault::Pandas => write!(f, "pandas"),
            AsTypeDefault::Auto => write!(f, "auto"),
        }
    }
}
