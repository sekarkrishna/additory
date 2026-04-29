// Logging module for Additory v0.1.3a9
// Provides three-level logging with magic operation tracking

pub mod magic;

pub use magic::{MagicOperation, MagicOperationType};

/// Logging level enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Off,        // False - no output except errors
    Default,    // 'default' - magic operations only
    Full,       // True - all operations
}

impl LogLevel {
    /// Convert from Python/R/Julia boolean or string to LogLevel
    pub fn from_value(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "false" | "off" => LogLevel::Off,
            "true" | "full" => LogLevel::Full,
            "default" => LogLevel::Default,
            _ => LogLevel::Default, // Default fallback
        }
    }
}

/// Logger struct with three-level logging
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self { level }
    }

    /// Log validation step (Full only)
    pub fn log_validation(&self, layer: &str, message: &str) {
        if self.level == LogLevel::Full {
            println!("[VALIDATION] {}: {}", layer, message);
        }
    }

    /// Log data statistics (Full only)
    pub fn log_statistics(&self, stats: &str) {
        if self.level == LogLevel::Full {
            println!("[STATISTICS] {}", stats);
        }
    }

    /// Log magic operation (Default and Full)
    pub fn log_magic_operation(&self, operation: &MagicOperation) {
        if self.level == LogLevel::Default || self.level == LogLevel::Full {
            println!("[MAGIC] {}", operation.format());
        }
    }

    /// Log warning (Default and Full)
    pub fn log_warning(&self, warning: &str) {
        if self.level == LogLevel::Default || self.level == LogLevel::Full {
            println!("[WARNING] {}", warning);
        }
    }

    /// Log error (always shown)
    pub fn log_error(&self, error: &str) {
        eprintln!("[ERROR] {}", error);
    }

    /// Get current logging level
    pub fn level(&self) -> LogLevel {
        self.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_value() {
        assert_eq!(LogLevel::from_value("false"), LogLevel::Off);
        assert_eq!(LogLevel::from_value("False"), LogLevel::Off);
        assert_eq!(LogLevel::from_value("true"), LogLevel::Full);
        assert_eq!(LogLevel::from_value("True"), LogLevel::Full);
        assert_eq!(LogLevel::from_value("default"), LogLevel::Default);
        assert_eq!(LogLevel::from_value("Default"), LogLevel::Default);
        assert_eq!(LogLevel::from_value("invalid"), LogLevel::Default);
    }

    #[test]
    fn test_logger_off() {
        let logger = Logger::new(LogLevel::Off);
        
        // These should not output anything (we can't test output easily, but we can call them)
        logger.log_validation("test", "message");
        logger.log_statistics("stats");
        logger.log_warning("warning");
        
        // Error should always output
        logger.log_error("error");
    }

    #[test]
    fn test_logger_default() {
        let logger = Logger::new(LogLevel::Default);
        
        // Validation and statistics should not output
        logger.log_validation("test", "message");
        logger.log_statistics("stats");
        
        // Warning should output
        logger.log_warning("warning");
        
        // Error should always output
        logger.log_error("error");
    }

    #[test]
    fn test_logger_full() {
        let logger = Logger::new(LogLevel::Full);
        
        // All should output
        logger.log_validation("test", "message");
        logger.log_statistics("stats");
        logger.log_warning("warning");
        logger.log_error("error");
    }
}
