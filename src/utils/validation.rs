//! Input validation utilities
//!
//! Validates parameters before operations to provide clear error messages.

use crate::core::errors::{AdditoryError, AdditoryResult};
use crate::core::types::FetchColumn;
use crate::core::DataFrame;

/// Validator for input parameters
pub struct Validator;

impl Validator {
    // TODO: Re-implement in Phase 2
    // /// Validate mode string
    // pub fn validate_mode(mode_str: &str, function: &str) -> AdditoryResult<TransformMode> {
    //     TransformMode::from_str(mode_str).map_err(|_| {
    //         AdditoryError::mode_parsing(mode_str, &["@calc", "@filter", "@sort", ...])
    //     })
    // }

    /// Validate DataFrame is not empty
    pub fn validate_not_empty(df: &DataFrame, context: &str) -> AdditoryResult<()> {
        if df.is_empty() {
            return Err(AdditoryError::empty_dataframe(
                context
            ));
        }
        Ok(())
    }

    /// Validate columns exist in DataFrame
    pub fn validate_columns_exist(
        df: &DataFrame,
        columns: &[String],
    ) -> AdditoryResult<()> {
        for col in columns {
            if !df.has_column(col) {
                return Err(AdditoryError::column_not_found(col, &df.column_names()));
            }
        }
        Ok(())
    }

    /// Validate fetch parameter
    pub fn validate_fetch(
        fetch: &Option<Vec<FetchColumn>>,
        df: &DataFrame,
    ) -> AdditoryResult<()> {
        if let Some(columns) = fetch {
            // Check all original columns exist
            for col in columns {
                let original = col.original();
                if !df.has_column(original) {
                    return Err(AdditoryError::column_not_found(
                        original,
                        &df.column_names(),
                    ));
                }
            }

            // Check for duplicate target names
            let mut target_names: Vec<&str> = columns.iter().map(|c| c.target()).collect();
            target_names.sort();
            for i in 0..target_names.len() - 1 {
                if target_names[i] == target_names[i + 1] {
                    return Err(AdditoryError::DuplicateColumn(
                        target_names[i].to_string(),
                        format!(
                            "Column '{}' would be created multiple times. Each output column must have a unique name.",
                            target_names[i]
                        ),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Validate required parameter is present
    pub fn validate_required<T>(
        param: &Option<T>,
        param_name: &str,
        context: &str,
    ) -> AdditoryResult<()> {
        if param.is_none() {
            return Err(AdditoryError::missing_parameter(
                param_name,
                &format!("{} requires '{}' parameter", context, param_name),
            ));
        }
        Ok(())
    }

    /// Validate parameter is not empty
    pub fn validate_not_empty_string(
        value: &str,
        param_name: &str,
    ) -> AdditoryResult<()> {
        if value.trim().is_empty() {
            return Err(AdditoryError::InvalidParameter(
                param_name.to_string(),
                "empty string".to_string(),
                format!("Parameter '{}' cannot be empty", param_name),
            ));
        }
        Ok(())
    }

    /// Validate list is not empty
    pub fn validate_not_empty_list<T>(
        list: &[T],
        param_name: &str,
    ) -> AdditoryResult<()> {
        if list.is_empty() {
            return Err(AdditoryError::InvalidParameter(
                param_name.to_string(),
                "empty list".to_string(),
                format!("Parameter '{}' cannot be empty", param_name),
            ));
        }
        Ok(())
    }

    /// Validate two lists have same length
    pub fn validate_same_length<T, U>(
        list1: &[T],
        list2: &[U],
        name1: &str,
        name2: &str,
    ) -> AdditoryResult<()> {
        if list1.len() != list2.len() {
            return Err(AdditoryError::InvalidParameter(
                format!("{} and {}", name1, name2),
                format!("lengths {} and {}", list1.len(), list2.len()),
                format!(
                    "Parameters '{}' and '{}' must have the same length",
                    name1, name2
                ),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_df() -> crate::core::DataFrame {
        let polars_df = df! {
            "name" => &["Alice", "Bob"],
            "age" => &[25, 30],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    // TODO: Re-enable in Phase 2
    // #[test]
    // fn test_validate_mode() {
    //     assert!(Validator::validate_mode("@filter", "transform").is_ok());
    //     assert!(Validator::validate_mode("@invalid", "transform").is_err());
    // }

    #[test]
    fn test_validate_not_empty() {
        let df = create_test_df();
        assert!(Validator::validate_not_empty(&df, "test").is_ok());

        let empty_df = crate::core::DataFrame::empty();
        assert!(Validator::validate_not_empty(&empty_df, "test").is_err());
    }

    #[test]
    fn test_validate_columns_exist() {
        let df = create_test_df();
        
        assert!(Validator::validate_columns_exist(
            &df,
            &["name".to_string(), "age".to_string()]
        ).is_ok());

        assert!(Validator::validate_columns_exist(
            &df,
            &["nonexistent".to_string()]
        ).is_err());
    }

    #[test]
    fn test_validate_duplicate_targets() {
        let df = create_test_df();
        
        let columns = vec![
            FetchColumn::Rename("name".to_string(), "col".to_string()),
            FetchColumn::Rename("age".to_string(), "col".to_string()),
        ];

        assert!(Validator::validate_fetch(&Some(columns), &df).is_err());
    }

    #[test]
    fn test_validate_same_length() {
        let list1 = vec![1, 2, 3];
        let list2 = vec!["a", "b", "c"];
        assert!(Validator::validate_same_length(&list1, &list2, "list1", "list2").is_ok());

        let list3 = vec!["a", "b"];
        assert!(Validator::validate_same_length(&list1, &list3, "list1", "list3").is_err());
    }
}
