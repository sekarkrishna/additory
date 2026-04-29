//! Logging utilities
//!
//! Implements the "always warn" principle - explain all magic when logging is enabled.

use log::{info, warn, debug};

/// Logger for additory operations
pub struct Logger {
    enabled: bool,
}

impl Logger {
    /// Create new logger
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Log operation start
    pub fn log_start(&self, function: &str, mode: &str) {
        if self.enabled {
            info!("[{}] Mode: {}", function, mode);
        }
    }

    /// Log DataFrame info
    pub fn log_dataframe(&self, function: &str, name: &str, rows: usize, cols: usize) {
        if self.enabled {
            info!("[{}] {}: {} rows × {} columns", function, name, rows, cols);
        }
    }

    /// Log parameter
    pub fn log_param(&self, function: &str, param: &str, value: &str) {
        if self.enabled {
            info!("[{}] {}: {}", function, param, value);
        }
    }

    /// Log operation result
    pub fn log_result(&self, function: &str, message: &str) {
        if self.enabled {
            info!("[{}] {}", function, message);
        }
    }

    /// Log warning
    pub fn log_warning(&self, function: &str, message: &str) {
        if self.enabled {
            warn!("[{}] {}", function, message);
        }
    }

    /// Log debug info
    pub fn log_debug(&self, function: &str, message: &str) {
        if self.enabled {
            debug!("[{}] {}", function, message);
        }
    }

    /// Log type conversion
    pub fn log_type_conversion(&self, function: &str, from: &str, to: &str) {
        if self.enabled {
            warn!(
                "[{}] Converting DataFrame from {} to {} for processing",
                function, from, to
            );
        }
    }

    /// Log assumption
    pub fn log_assumption(&self, function: &str, assumption: &str) {
        if self.enabled {
            warn!("[{}] Assumption: {}", function, assumption);
        }
    }

    /// Log execution time
    pub fn log_execution_time(&self, function: &str, operation: &str, duration_ms: f64) {
        if self.enabled {
            info!(
                "[{}] {} completed in {:.2}ms",
                function, operation, duration_ms
            );
        }
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(true);
        assert!(logger.enabled);

        let logger = Logger::new(false);
        assert!(!logger.enabled);
    }

    #[test]
    fn test_logger_default() {
        let logger = Logger::default();
        assert!(!logger.enabled);
    }

    #[test]
    fn test_logging_methods() {
        // Just ensure methods don't panic
        let logger = Logger::new(true);
        logger.log_start("test", "@filter");
        logger.log_dataframe("test", "df", 100, 5);
        logger.log_param("test", "fetch", "age");
        logger.log_result("test", "Success");
        logger.log_warning("test", "Warning message");
        logger.log_debug("test", "Debug info");
        logger.log_type_conversion("test", "Pandas", "Polars");
        logger.log_assumption("test", "Using default value");
        logger.log_execution_time("test", "filter", 12.5);
    }
}
