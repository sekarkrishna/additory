// Magic operation tracking and explanation
// Tracks implicit operations performed by Additory

use std::collections::HashMap;

/// Type of magic operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MagicOperationType {
    ImplicitAggregation,
    AutomaticTypeConversion,
    DefaultValueApplication,
    ColumnNameNormalization,
}

/// Magic operation with tracking information
#[derive(Debug, Clone)]
pub struct MagicOperation {
    pub operation_type: MagicOperationType,
    pub reason: String,
    pub impact: String,
    pub details: HashMap<String, String>,
}

impl MagicOperation {
    /// Create a new magic operation
    pub fn new(
        operation_type: MagicOperationType,
        reason: String,
        impact: String,
    ) -> Self {
        Self {
            operation_type,
            reason,
            impact,
            details: HashMap::new(),
        }
    }

    /// Add a detail to the operation
    pub fn with_detail(mut self, key: String, value: String) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Format the magic operation for logging
    pub fn format(&self) -> String {
        let operation_name = match self.operation_type {
            MagicOperationType::ImplicitAggregation => "Implicit Aggregation",
            MagicOperationType::AutomaticTypeConversion => "Automatic Type Conversion",
            MagicOperationType::DefaultValueApplication => "Default Value Application",
            MagicOperationType::ColumnNameNormalization => "Column Name Normalization",
        };

        let mut output = format!(
            "{}\n  Reason: {}\n  Impact: {}",
            operation_name, self.reason, self.impact
        );

        if !self.details.is_empty() {
            output.push_str("\n  Details:");
            for (key, value) in &self.details {
                output.push_str(&format!("\n    {}: {}", key, value));
            }
        }

        output
    }

    /// Create an implicit aggregation magic operation
    pub fn implicit_aggregation(
        aggregation_function: &str,
        column: &str,
        reason: &str,
    ) -> Self {
        Self::new(
            MagicOperationType::ImplicitAggregation,
            reason.to_string(),
            format!(
                "Column '{}' will be aggregated using '{}'",
                column, aggregation_function
            ),
        )
        .with_detail("column".to_string(), column.to_string())
        .with_detail("function".to_string(), aggregation_function.to_string())
    }

    /// Create an automatic type conversion magic operation
    pub fn automatic_type_conversion(
        column: &str,
        from_type: &str,
        to_type: &str,
        reason: &str,
    ) -> Self {
        Self::new(
            MagicOperationType::AutomaticTypeConversion,
            reason.to_string(),
            format!(
                "Column '{}' will be converted from {} to {}",
                column, from_type, to_type
            ),
        )
        .with_detail("column".to_string(), column.to_string())
        .with_detail("from_type".to_string(), from_type.to_string())
        .with_detail("to_type".to_string(), to_type.to_string())
    }

    /// Create a default value application magic operation
    pub fn default_value_application(parameter: &str, default_value: &str) -> Self {
        Self::new(
            MagicOperationType::DefaultValueApplication,
            format!("Parameter '{}' not provided", parameter),
            format!("Using default value: {}", default_value),
        )
        .with_detail("parameter".to_string(), parameter.to_string())
        .with_detail("default_value".to_string(), default_value.to_string())
    }

    /// Create a column name normalization magic operation
    pub fn column_name_normalization(
        original_name: &str,
        normalized_name: &str,
        reason: &str,
    ) -> Self {
        Self::new(
            MagicOperationType::ColumnNameNormalization,
            reason.to_string(),
            format!(
                "Column '{}' normalized to '{}'",
                original_name, normalized_name
            ),
        )
        .with_detail("original_name".to_string(), original_name.to_string())
        .with_detail("normalized_name".to_string(), normalized_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_operation_creation() {
        let op = MagicOperation::new(
            MagicOperationType::ImplicitAggregation,
            "Many-to-one cardinality detected".to_string(),
            "Values will be aggregated".to_string(),
        );

        assert_eq!(op.operation_type, MagicOperationType::ImplicitAggregation);
        assert_eq!(op.reason, "Many-to-one cardinality detected");
        assert_eq!(op.impact, "Values will be aggregated");
    }

    #[test]
    fn test_magic_operation_with_details() {
        let op = MagicOperation::new(
            MagicOperationType::ImplicitAggregation,
            "Test reason".to_string(),
            "Test impact".to_string(),
        )
        .with_detail("column".to_string(), "amount".to_string())
        .with_detail("function".to_string(), "sum".to_string());

        assert_eq!(op.details.len(), 2);
        assert_eq!(op.details.get("column"), Some(&"amount".to_string()));
        assert_eq!(op.details.get("function"), Some(&"sum".to_string()));
    }

    #[test]
    fn test_implicit_aggregation_helper() {
        let op = MagicOperation::implicit_aggregation(
            "sum",
            "amount",
            "Many-to-one cardinality detected",
        );

        assert_eq!(op.operation_type, MagicOperationType::ImplicitAggregation);
        assert!(op.impact.contains("amount"));
        assert!(op.impact.contains("sum"));
    }

    #[test]
    fn test_automatic_type_conversion_helper() {
        let op = MagicOperation::automatic_type_conversion(
            "age",
            "Int32",
            "Float64",
            "Required for calculation",
        );

        assert_eq!(
            op.operation_type,
            MagicOperationType::AutomaticTypeConversion
        );
        assert!(op.impact.contains("age"));
        assert!(op.impact.contains("Int32"));
        assert!(op.impact.contains("Float64"));
    }

    #[test]
    fn test_default_value_application_helper() {
        let op = MagicOperation::default_value_application("join_type", "left");

        assert_eq!(
            op.operation_type,
            MagicOperationType::DefaultValueApplication
        );
        assert!(op.impact.contains("left"));
    }

    #[test]
    fn test_column_name_normalization_helper() {
        let op = MagicOperation::column_name_normalization(
            "Customer Name",
            "customer_name",
            "Spaces replaced with underscores",
        );

        assert_eq!(
            op.operation_type,
            MagicOperationType::ColumnNameNormalization
        );
        assert!(op.impact.contains("Customer Name"));
        assert!(op.impact.contains("customer_name"));
    }

    #[test]
    fn test_format_output() {
        let op = MagicOperation::implicit_aggregation(
            "sum",
            "amount",
            "Many-to-one cardinality detected",
        );

        let formatted = op.format();
        assert!(formatted.contains("Implicit Aggregation"));
        assert!(formatted.contains("Reason:"));
        assert!(formatted.contains("Impact:"));
        assert!(formatted.contains("Details:"));
    }
}
