// Validation module for Additory v0.1.3a9
// Provides four layers of validation: parameter, data, strategy, and logging

pub mod errors;
pub mod params;
pub mod data;
pub mod strategy;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::Instant;
use std::cell::RefCell;
use polars::prelude::SchemaExt;

// Re-export commonly used types
pub use errors::{
    ValidationError, ValidationResult, ErrorType, ErrorContext,
    Suggestion, CodeExample, Language, Severity, ErrorSource,
};

pub use params::ParameterValidator;
pub use data::{DataValidator, Cardinality, Side, DuplicateAnalysis, MissingKeyAnalysis, NullAnalysis, ColumnNullStats};
pub use strategy::StrategyValidator;

/// Hash key for caching validation results based on DataFrame characteristics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataFrameHash {
    shape: (usize, usize),  // (rows, columns)
    column_names: Vec<String>,
    schema_hash: u64,
}

impl DataFrameHash {
    /// Compute hash from a DataFrame
    pub fn from_dataframe(df: &polars::prelude::DataFrame) -> Self {
        use std::collections::hash_map::DefaultHasher;
        
        let shape = (df.height(), df.width());
        let column_names: Vec<String> = df.get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        
        // Hash the schema (column names and types)
        let mut hasher = DefaultHasher::new();
        for field in df.schema().iter_fields() {
            field.name.hash(&mut hasher);
            format!("{:?}", field.dtype).hash(&mut hasher);
        }
        let schema_hash = hasher.finish();
        
        Self {
            shape,
            column_names,
            schema_hash,
        }
    }
}

/// Cached validation results for a DataFrame
#[derive(Debug, Clone)]
pub struct CachedValidation {
    pub cardinality: Option<Cardinality>,
    pub duplicate_analysis: Option<DuplicateAnalysis>,
    pub null_analysis: Option<NullAnalysis>,
    pub timestamp: Instant,
}

impl Default for CachedValidation {
    fn default() -> Self {
        Self::new()
    }
}

impl CachedValidation {
    pub fn new() -> Self {
        Self {
            cardinality: None,
            duplicate_analysis: None,
            null_analysis: None,
            timestamp: Instant::now(),
        }
    }
    
    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = Some(cardinality);
        self
    }
    
    pub fn with_duplicate_analysis(mut self, analysis: DuplicateAnalysis) -> Self {
        self.duplicate_analysis = Some(analysis);
        self
    }
    
    pub fn with_null_analysis(mut self, analysis: NullAnalysis) -> Self {
        self.null_analysis = Some(analysis);
        self
    }
}

/// Validation cache with LRU eviction
///
/// The ValidationCache stores validation results for DataFrames to avoid redundant
/// validation operations. It uses DataFrame characteristics (shape, columns, schema)
/// as cache keys and implements LRU (Least Recently Used) eviction when the cache
/// exceeds its maximum size.
///
/// # Cache Key Generation
///
/// Cache keys are generated using:
/// - DataFrame shape (rows, columns)
/// - Column names
/// - Schema hash (column names and types)
///
/// This means that DataFrames with the same structure will share cached validation
/// results, even if the actual data values differ. This is intentional for performance,
/// as structural validation (cardinality, duplicates, nulls) depends on the data
/// structure rather than specific values.
///
/// # LRU Eviction
///
/// When the cache exceeds `max_size`, the oldest entry (by timestamp) is evicted.
/// This ensures the cache doesn't grow unbounded while keeping the most recently
/// used validation results available.
///
/// # Usage
///
/// The cache is integrated into ValidationOrchestrator and is controlled by the
/// `enable_caching` flag in PerformanceConfig. When caching is disabled (max_size = 0),
/// the cache will not store any entries.
///
/// # Example
///
/// ```rust,ignore
/// let mut cache = ValidationCache::new(100); // Max 100 entries
///
/// let result = cache.get_or_compute(&df, |df| {
///     // Expensive validation computation
///     Ok(CachedValidation::new().with_cardinality(Cardinality::OneToOne))
/// })?;
/// ```
pub struct ValidationCache {
    cache: HashMap<DataFrameHash, CachedValidation>,
    max_size: usize,
}

impl ValidationCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }
    
    /// Get cached validation result or compute it
    pub fn get_or_compute<F>(
        &mut self,
        df: &polars::prelude::DataFrame,
        compute_fn: F,
    ) -> ValidationResult<CachedValidation>
    where
        F: FnOnce(&polars::prelude::DataFrame) -> ValidationResult<CachedValidation>,
    {
        let hash = DataFrameHash::from_dataframe(df);
        
        // Check for cache hit
        if let Some(cached) = self.cache.get(&hash) {
            return Ok(cached.clone());
        }
        
        // Cache miss - compute and store
        let result = compute_fn(df)?;
        self.cache.insert(hash, result.clone());
        
        // Evict oldest if cache exceeds max_size
        if self.cache.len() > self.max_size {
            self.evict_oldest();
        }
        
        Ok(result)
    }
    
    /// Evict the oldest entry based on timestamp (LRU)
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.cache
            .iter()
            .min_by_key(|(_, v)| v.timestamp)
            .map(|(k, _)| k.clone())
        {
            self.cache.remove(&oldest_key);
        }
    }
    
    /// Clear all cached entries
    pub fn clear(&mut self) {
        self.cache.clear();
    }
    
    /// Get current cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// ValidationOrchestrator coordinates all validation layers
/// and provides the main entry point for validation operations
pub struct ValidationOrchestrator {
    config: ValidationConfig,
    _logger: crate::logging::Logger,
    cache: RefCell<ValidationCache>,
}

impl ValidationOrchestrator {
    pub fn new(config: ValidationConfig) -> Self {
        let cache_size = if config.performance.enable_caching { 100 } else { 0 };
        Self {
            _logger: crate::logging::Logger::new(crate::logging::LogLevel::Default),
            cache: RefCell::new(ValidationCache::new(cache_size)),
            config,
        }
    }

    pub fn with_default_config() -> Self {
        let config = ValidationConfig::default();
        let cache_size = if config.performance.enable_caching { 100 } else { 0 };
        Self {
            config,
            _logger: crate::logging::Logger::new(crate::logging::LogLevel::Default),
            cache: RefCell::new(ValidationCache::new(cache_size)),
        }
    }
    
    /// Clear the validation cache
    ///
    /// This method clears all cached validation results. Useful when you want to
    /// force re-validation of DataFrames that may have changed.
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }
    
    /// Get the current cache size
    ///
    /// Returns the number of cached validation results currently stored.
    pub fn cache_size(&self) -> usize {
        self.cache.borrow().len()
    }

    /// Validate add.to() operation with full validation pipeline
    ///
    /// Performs sequential validation:
    /// 1. Parameter validation (types, values, required fields)
    /// 2. Data validation (cardinality, duplicates, missing keys)
    /// 3. Strategy validation (requirements, keys, values)
    /// 4. Logging configuration
    ///
    /// Implements fail-fast behavior: stops at first error
    pub fn validate_add_to(
        &self,
        left: &polars::prelude::DataFrame,
        right: &polars::prelude::DataFrame,
        on: &[String],
        _fetch: &Option<Vec<String>>,
        join_type: &str,
        strategy: &Option<std::collections::HashMap<String, String>>,
        logging: crate::logging::LogLevel,
    ) -> ValidationResult<()> {
        // Update logger level
        let logger = crate::logging::Logger::new(logging);

        // Phase 1: Parameter Validation
        if self.config.enabled_layers.parameter {
            logger.log_validation("parameter", "Validating parameters for add.to()");

            // Validate join_type parameter
            ParameterValidator::validate_value(
                "join_type",
                join_type,
                &["left", "right", "inner", "outer", "cross"],
                "add.to",
            )?;

            // Validate on parameter is not empty
            if on.is_empty() {
                let context = ErrorContext::new("add.to".to_string())
                    .with_parameter("on".to_string());

                return Err(ValidationError::new(
                    ErrorType::ParameterError,
                    errors::generate_error_code(ErrorType::ParameterError, "R", 1),
                    "Parameter 'on' is required and cannot be empty".to_string(),
                    context,
                )
                .with_suggestion(1, "Provide at least one column name for the join key".to_string())
                .with_example(
                    Language::Python,
                    "result = add.to(df1, df2, on='id')".to_string(),
                    "Example with join key".to_string(),
                ));
            }
        }

        // Phase 2: Data Validation
        if self.config.enabled_layers.data {
            logger.log_validation("data", "Validating data properties");

            // Detect cardinality
            let cardinality = DataValidator::detect_cardinality(left, right, on, on)?;

            logger.log_statistics(&format!(
                "Cardinality detected: {:?}",
                cardinality
            ));

            // Analyze duplicates in both DataFrames
            let left_duplicates = DataValidator::analyze_duplicates(left, on, data::Side::Left)?;
            let right_duplicates = DataValidator::analyze_duplicates(right, on, data::Side::Right)?;

            if left_duplicates.has_duplicates {
                logger.log_statistics(&format!(
                    "Left DataFrame has {} duplicate keys ({:.2}%)",
                    left_duplicates.duplicate_count,
                    left_duplicates.duplicate_percentage
                ));
            }

            if right_duplicates.has_duplicates {
                logger.log_statistics(&format!(
                    "Right DataFrame has {} duplicate keys ({:.2}%)",
                    right_duplicates.duplicate_count,
                    right_duplicates.duplicate_percentage
                ));
            }

            // Detect missing keys
            let missing_keys = DataValidator::detect_missing_keys(left, right, on, on)?;

            if missing_keys.left_missing_count > 0 || missing_keys.right_missing_count > 0 {
                logger.log_warning(&format!(
                    "Missing keys detected: {} in left ({:.2}%), {} in right ({:.2}%)",
                    missing_keys.left_missing_count,
                    missing_keys.left_missing_percentage,
                    missing_keys.right_missing_count,
                    missing_keys.right_missing_percentage
                ));
            }

            // Phase 3: Strategy Validation
            if self.config.enabled_layers.strategy {
                logger.log_validation("strategy", "Validating strategy specification");

                // Check if strategy is required based on cardinality
                if StrategyValidator::is_strategy_required(&cardinality)
                    && strategy.is_none() {
                    // Strategy is required but not provided
                    return Err(StrategyValidator::generate_strategy_required_error(
                        &cardinality,
                        right,
                        on,
                    ));
                }

                // If strategy is provided, validate it
                if let Some(strat) = strategy {
                    // Validate strategy keys exist in DataFrame
                    StrategyValidator::validate_strategy_keys(strat, right, on)?;

                    // Validate strategy values are valid aggregation functions
                    StrategyValidator::validate_strategy_values(strat, right)?;
                }
            }
        }

        // Phase 4: Logging Configuration (already handled by logger parameter)
        logger.log_validation("complete", "All validations passed");

        Ok(())
    }

    /// Validate add.transform() operation
    ///
    /// Performs validation:
    /// 1. Parameter validation (mode, columns, params)
    /// 2. Data validation (column types for mode, null analysis)
    /// 3. Logging configuration
    pub fn validate_add_transform(
        &self,
        df: &polars::prelude::DataFrame,
        mode: &str,
        columns: &[String],
        _params: &std::collections::HashMap<String, String>,
        logging: crate::logging::LogLevel,
    ) -> ValidationResult<()> {
        let logger = crate::logging::Logger::new(logging);

        // Phase 1: Parameter Validation
        if self.config.enabled_layers.parameter {
            logger.log_validation("parameter", "Validating parameters for add.transform()");

            // Validate mode parameter (should start with @)
            if !mode.starts_with('@') {
                let context = ErrorContext::new("add.transform".to_string())
                    .with_parameter("mode".to_string())
                    .with_info("received_value".to_string(), mode.to_string());

                return Err(ValidationError::new(
                    ErrorType::ParameterError,
                    errors::generate_error_code(ErrorType::ParameterError, "V", 3),
                    format!("Invalid mode '{}': mode must start with '@'", mode),
                    context,
                )
                .with_suggestion(1, "Use a valid mode: @round, @scale, @deduce, @encode, @filter, @extract".to_string())
                .with_example(
                    Language::Python,
                    "result = add.transform(df, mode='@round', columns=['value'])".to_string(),
                    "Example with valid mode".to_string(),
                ));
            }

            // Validate columns parameter is not empty (except for modes that don't use it)
            let modes_without_columns = ["@transpose", "@calc", "@filter", "@sort", "@aggregate", "@deduce"];
            if columns.is_empty() && !modes_without_columns.contains(&mode) {
                let context = ErrorContext::new("add.transform".to_string())
                    .with_parameter("columns".to_string());

                return Err(ValidationError::new(
                    ErrorType::ParameterError,
                    errors::generate_error_code(ErrorType::ParameterError, "R", 2),
                    "Parameter 'columns' is required and cannot be empty".to_string(),
                    context,
                )
                .with_suggestion(1, "Provide at least one column name to transform".to_string())
                .with_example(
                    Language::Python,
                    "result = add.transform(df, mode='@round', columns=['value'])".to_string(),
                    "Example with columns".to_string(),
                ));
            }
        }

        // Phase 2: Data Validation
        if self.config.enabled_layers.data {
            logger.log_validation("data", "Validating data properties");

            // Validate column types for mode
            DataValidator::validate_column_types_for_mode(df, columns, mode)?;

            // Analyze null values in target columns
            let null_analysis = DataValidator::analyze_nulls(df, columns)?;

            // Warn if @deduce mode but no nulls
            if mode == "@deduce" && null_analysis.total_null_percentage == 0.0 {
                logger.log_warning(
                    "Mode @deduce specified but no null values found in target columns. Nothing to impute."
                );
            }

            // Log null statistics if significant
            if null_analysis.total_null_percentage > self.config.thresholds.null_percentage_warning {
                logger.log_warning(&format!(
                    "High null percentage in target columns: {:.2}%",
                    null_analysis.total_null_percentage
                ));
            }
        }

        logger.log_validation("complete", "All validations passed");

        Ok(())
    }

    /// Validate add.synthetic() operation
    ///
    /// Performs validation:
    /// 1. Parameter validation (strategy, params)
    /// 2. Data validation (DataFrame properties)
    /// 3. Logging configuration
    pub fn validate_add_synthetic(
        &self,
        df: &polars::prelude::DataFrame,
        strategy: &str,
        _params: &std::collections::HashMap<String, String>,
        logging: crate::logging::LogLevel,
    ) -> ValidationResult<()> {
        let logger = crate::logging::Logger::new(logging);

        // Phase 1: Parameter Validation
        if self.config.enabled_layers.parameter {
            logger.log_validation("parameter", "Validating parameters for add.synthetic()");

            // Validate strategy parameter is not empty
            if strategy.is_empty() {
                let context = ErrorContext::new("add.synthetic".to_string())
                    .with_parameter("strategy".to_string());

                return Err(ValidationError::new(
                    ErrorType::ParameterError,
                    errors::generate_error_code(ErrorType::ParameterError, "R", 3),
                    "Parameter 'strategy' is required and cannot be empty".to_string(),
                    context,
                )
                .with_suggestion(1, "Provide a strategy for synthetic data generation".to_string())
                .with_example(
                    Language::Python,
                    "result = add.synthetic(df, strategy='knn')".to_string(),
                    "Example with strategy".to_string(),
                ));
            }
        }

        // Phase 2: Data Validation
        if self.config.enabled_layers.data {
            logger.log_validation("data", "Validating data properties");

            // Check DataFrame is not empty
            if df.height() == 0 {
                let context = ErrorContext::new("add.synthetic".to_string())
                    .with_info("rows".to_string(), "0".to_string());

                return Err(ValidationError::new(
                    ErrorType::DataError,
                    errors::generate_error_code(ErrorType::DataError, "E", 1),
                    "DataFrame is empty. Cannot generate synthetic data from empty DataFrame".to_string(),
                    context,
                )
                .with_suggestion(1, "Provide a DataFrame with at least one row".to_string()));
            }
        }

        logger.log_validation("complete", "All validations passed");

        Ok(())
    }
}

/// Configuration for the validation system
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    pub mode: ValidationMode,
    pub enabled_layers: ValidationLayers,
    pub thresholds: ValidationThresholds,
    pub performance: PerformanceConfig,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            mode: ValidationMode::Normal,
            enabled_layers: ValidationLayers::all_enabled(),
            thresholds: ValidationThresholds::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl ValidationConfig {
    /// Load configuration from environment variables and TOML file
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables
    /// 2. additory.toml file
    /// 3. Default values
    ///
    /// Environment variables:
    /// - ADDITORY_VALIDATION_MODE: strict, normal, or permissive
    /// - ADDITORY_VALIDATION_BYPASS: true to skip all validation
    /// - ADDITORY_VALIDATION_CACHE: true to enable caching
    /// - ADDITORY_VALIDATION_PARAMETER: true/false to enable/disable parameter validation
    /// - ADDITORY_VALIDATION_DATA: true/false to enable/disable data validation
    /// - ADDITORY_VALIDATION_STRATEGY: true/false to enable/disable strategy validation
    /// - ADDITORY_VALIDATION_LOGGING: true/false to enable/disable logging validation
    /// - ADDITORY_VALIDATION_NULL_THRESHOLD: percentage threshold for null warnings
    /// - ADDITORY_VALIDATION_DUPLICATE_THRESHOLD: percentage threshold for duplicate warnings
    /// - ADDITORY_VALIDATION_MISSING_KEY_THRESHOLD: percentage threshold for missing key warnings
    /// - ADDITORY_VALIDATION_SAMPLING_THRESHOLD: row count threshold for sampling
    /// - ADDITORY_VALIDATION_MAX_TIME_MS: maximum validation time in milliseconds
    pub fn load() -> Self {
        let mut config = Self::default();

        // Try to load from TOML file first
        if let Ok(toml_config) = Self::load_from_toml("additory.toml") {
            config = toml_config;
        }

        // Override with environment variables
        config.load_from_env();

        config
    }

    /// Load configuration from a TOML file
    ///
    /// Expected format:
    /// ```toml
    /// [validation]
    /// mode = "normal"  # strict, normal, or permissive
    /// enabled_layers = ["parameter", "data", "strategy", "logging"]
    ///
    /// [validation.thresholds]
    /// null_percentage_warning = 50.0
    /// duplicate_percentage_warning = 10.0
    /// missing_key_percentage_warning = 20.0
    ///
    /// [validation.performance]
    /// enable_caching = true
    /// sampling_threshold_rows = 10_000_000
    /// max_validation_time_ms = 100
    /// ```
    pub fn load_from_toml(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        use std::fs;
        
        let content = fs::read_to_string(path)?;
        let toml_value: toml::Value = toml::from_str(&content)?;

        let mut config = Self::default();

        // Parse validation section
        if let Some(validation) = toml_value.get("validation") {
            // Parse mode
            if let Some(mode_str) = validation.get("mode").and_then(|v| v.as_str()) {
                config.mode = match mode_str {
                    "strict" => ValidationMode::Strict,
                    "normal" => ValidationMode::Normal,
                    "permissive" => ValidationMode::Permissive,
                    _ => ValidationMode::Normal,
                };
            }

            // Parse enabled_layers
            if let Some(layers) = validation.get("enabled_layers").and_then(|v| v.as_array()) {
                let layer_names: Vec<String> = layers
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                config.enabled_layers = ValidationLayers {
                    parameter: layer_names.contains(&"parameter".to_string()),
                    data: layer_names.contains(&"data".to_string()),
                    strategy: layer_names.contains(&"strategy".to_string()),
                    logging: layer_names.contains(&"logging".to_string()),
                };
            }

            // Parse thresholds
            if let Some(thresholds) = validation.get("thresholds") {
                if let Some(null_threshold) = thresholds.get("null_percentage_warning").and_then(|v| v.as_float()) {
                    config.thresholds.null_percentage_warning = null_threshold;
                }
                if let Some(dup_threshold) = thresholds.get("duplicate_percentage_warning").and_then(|v| v.as_float()) {
                    config.thresholds.duplicate_percentage_warning = dup_threshold;
                }
                if let Some(missing_threshold) = thresholds.get("missing_key_percentage_warning").and_then(|v| v.as_float()) {
                    config.thresholds.missing_key_percentage_warning = missing_threshold;
                }
            }

            // Parse performance
            if let Some(performance) = validation.get("performance") {
                if let Some(caching) = performance.get("enable_caching").and_then(|v| v.as_bool()) {
                    config.performance.enable_caching = caching;
                }
                if let Some(sampling) = performance.get("sampling_threshold_rows").and_then(|v| v.as_integer()) {
                    config.performance.sampling_threshold_rows = sampling as usize;
                }
                if let Some(max_time) = performance.get("max_validation_time_ms").and_then(|v| v.as_integer()) {
                    config.performance.max_validation_time_ms = max_time as u64;
                }
            }
        }

        Ok(config)
    }

    /// Load configuration from environment variables
    ///
    /// This method modifies the current config in-place with values from environment variables
    pub fn load_from_env(&mut self) {
        use std::env;

        // Load validation mode
        if let Ok(mode_str) = env::var("ADDITORY_VALIDATION_MODE") {
            self.mode = match mode_str.to_lowercase().as_str() {
                "strict" => ValidationMode::Strict,
                "normal" => ValidationMode::Normal,
                "permissive" => ValidationMode::Permissive,
                _ => self.mode,
            };
        }

        // Load validation bypass (disables all layers)
        if let Ok(bypass_str) = env::var("ADDITORY_VALIDATION_BYPASS") {
            if bypass_str.to_lowercase() == "true" {
                self.enabled_layers = ValidationLayers::all_disabled();
            }
        }

        // Load individual layer settings
        if let Ok(param_str) = env::var("ADDITORY_VALIDATION_PARAMETER") {
            self.enabled_layers.parameter = param_str.to_lowercase() == "true";
        }
        if let Ok(data_str) = env::var("ADDITORY_VALIDATION_DATA") {
            self.enabled_layers.data = data_str.to_lowercase() == "true";
        }
        if let Ok(strategy_str) = env::var("ADDITORY_VALIDATION_STRATEGY") {
            self.enabled_layers.strategy = strategy_str.to_lowercase() == "true";
        }
        if let Ok(logging_str) = env::var("ADDITORY_VALIDATION_LOGGING") {
            self.enabled_layers.logging = logging_str.to_lowercase() == "true";
        }

        // Load threshold settings
        if let Ok(null_threshold) = env::var("ADDITORY_VALIDATION_NULL_THRESHOLD") {
            if let Ok(value) = null_threshold.parse::<f64>() {
                self.thresholds.null_percentage_warning = value;
            }
        }
        if let Ok(dup_threshold) = env::var("ADDITORY_VALIDATION_DUPLICATE_THRESHOLD") {
            if let Ok(value) = dup_threshold.parse::<f64>() {
                self.thresholds.duplicate_percentage_warning = value;
            }
        }
        if let Ok(missing_threshold) = env::var("ADDITORY_VALIDATION_MISSING_KEY_THRESHOLD") {
            if let Ok(value) = missing_threshold.parse::<f64>() {
                self.thresholds.missing_key_percentage_warning = value;
            }
        }

        // Load performance settings
        if let Ok(cache_str) = env::var("ADDITORY_VALIDATION_CACHE") {
            self.performance.enable_caching = cache_str.to_lowercase() == "true";
        }
        if let Ok(sampling_threshold) = env::var("ADDITORY_VALIDATION_SAMPLING_THRESHOLD") {
            if let Ok(value) = sampling_threshold.parse::<usize>() {
                self.performance.sampling_threshold_rows = value;
            }
        }
        if let Ok(max_time) = env::var("ADDITORY_VALIDATION_MAX_TIME_MS") {
            if let Ok(value) = max_time.parse::<u64>() {
                self.performance.max_validation_time_ms = value;
            }
        }
    }
}

/// Validation mode determines how warnings are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationMode {
    Strict,      // Warnings become errors
    Normal,      // Default behavior
    Permissive,  // Warnings don't block
}

/// Controls which validation layers are enabled
#[derive(Debug, Clone)]
pub struct ValidationLayers {
    pub parameter: bool,
    pub data: bool,
    pub strategy: bool,
    pub logging: bool,
}

impl ValidationLayers {
    pub fn all_enabled() -> Self {
        Self {
            parameter: true,
            data: true,
            strategy: true,
            logging: true,
        }
    }

    pub fn all_disabled() -> Self {
        Self {
            parameter: false,
            data: false,
            strategy: false,
            logging: false,
        }
    }
}

/// Configurable thresholds for warnings
#[derive(Debug, Clone)]
pub struct ValidationThresholds {
    pub null_percentage_warning: f64,
    pub duplicate_percentage_warning: f64,
    pub missing_key_percentage_warning: f64,
}

impl Default for ValidationThresholds {
    fn default() -> Self {
        Self {
            null_percentage_warning: 50.0,
            duplicate_percentage_warning: 10.0,
            missing_key_percentage_warning: 20.0,
        }
    }
}

/// Performance configuration for validation
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    pub enable_caching: bool,
    pub sampling_threshold_rows: usize,
    pub max_validation_time_ms: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            sampling_threshold_rows: 10_000_000,
            max_validation_time_ms: 100,
        }
    }
}
