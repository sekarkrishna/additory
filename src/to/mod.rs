//! add.to() module - Add data FROM external source (v0.1.3a5)
//!
//! Implements lookup and join operations with aggregation support.

use crate::core::{DataFrame, AdditoryResult, AdditoryError, UniversalParams, JoinType, FetchColumn, Against, Position};

// Submodules
pub mod lookup;
pub mod aggregation;
pub mod position;

/// Main entry point for add.to()
///
/// # Parameters
/// - `target`: Target DataFrame to add columns to
/// - `params`: Universal parameters containing:
///   - `fetch`: Columns to fetch from reference
///   - `against`: Join key(s)
///   - `reference`: Reference DataFrame
///   - `join_type`: Type of join (lookup, left, inner, outer)
///   - `strategy`: Aggregation strategies
///   - `position`: Where to insert new columns
///
/// # Returns
/// DataFrame with new columns added
pub fn to(
    target: DataFrame,
    params: UniversalParams,
) -> AdditoryResult<DataFrame> {
    // Validate required parameters
    let fetch = params.fetch.ok_or_else(|| AdditoryError::missing_parameter(
        "fetch",
        "Specify columns to fetch from reference DataFrame"
    ))?;
    
    let against = params.against.ok_or_else(|| AdditoryError::missing_parameter(
        "against",
        "Specify join key(s) for matching rows"
    ))?;
    
    let reference = params.reference.ok_or_else(|| AdditoryError::missing_parameter(
        "reference",
        "Specify reference DataFrame to fetch columns from"
    ))?;
    
    // Get join type (default to lookup)
    let join_type = params.join_type.unwrap_or(JoinType::Lookup);
    
    // Perform operation based on join type
    match join_type {
        JoinType::Lookup => {
            // Use lookup with aggregation
            lookup::lookup(
                target,
                reference,
                fetch,
                against,
                params.strategy,
                params.position,
            )
        }
        JoinType::Left => {
            // Left join requires 1:1 cardinality
            validate_one_to_one(&target, &reference, &against)?;
            explicit_join(
                target,
                reference,
                fetch,
                against,
                polars::prelude::JoinType::Left,
                params.position,
            )
        }
        JoinType::Inner => {
            // Inner join requires 1:1 cardinality
            validate_one_to_one(&target, &reference, &against)?;
            explicit_join(
                target,
                reference,
                fetch,
                against,
                polars::prelude::JoinType::Inner,
                params.position,
            )
        }
        JoinType::Outer => {
            // Outer join requires 1:1 cardinality
            validate_one_to_one(&target, &reference, &against)?;
            explicit_join(
                target,
                reference,
                fetch,
                against,
                polars::prelude::JoinType::Full,
                params.position,
            )
        }
    }
}

/// Validate 1:1 cardinality for explicit join types
fn validate_one_to_one(
    target: &DataFrame,
    reference: &DataFrame,
    against: &Against,
) -> AdditoryResult<()> {
    let join_keys = against.keys();
    let key = join_keys[0]; // TODO: Handle composite keys
    
    // Check target cardinality
    let target_total = target.height();
    let target_col = target.column(key)?;
    let target_series = target_col.as_materialized_series();
    let target_unique = target_series.n_unique().map_err(AdditoryError::Polars)?;
    
    // Check reference cardinality
    let ref_total = reference.height();
    let ref_col = reference.column(key)?;
    let ref_series = ref_col.as_materialized_series();
    let ref_unique = ref_series.n_unique().map_err(AdditoryError::Polars)?;
    
    // Both must be 1:1 (unique keys in both)
    let target_has_dups = target_total > target_unique;
    let ref_has_dups = ref_total > ref_unique;
    
    if target_has_dups || ref_has_dups {
        let relationship = if target_has_dups && ref_has_dups {
            "many:many"
        } else if target_has_dups {
            "many:1"
        } else {
            "1:many"
        };
        
        return Err(AdditoryError::validation(
            &format!(
                "Explicit join types require 1:1 relationship. Detected {} relationship on '{}'.\n\
                 Target: {} rows, {} unique values\n\
                 Reference: {} rows, {} unique values",
                relationship, key, target_total, target_unique, ref_total, ref_unique
            ),
            "Use join_type='lookup' with strategy for aggregation, or ensure both DataFrames have unique keys"
        ));
    }
    
    Ok(())
}

/// Perform explicit join (left, inner, outer) with 1:1 cardinality
fn explicit_join(
    target: DataFrame,
    reference: DataFrame,
    fetch: Vec<FetchColumn>,
    against: Against,
    join_type: polars::prelude::JoinType,
    position: Option<Position>,
) -> AdditoryResult<DataFrame> {
    let join_keys = against.keys();
    
    // Validate join keys exist in both DataFrames
    for key in &join_keys {
        if !target.has_column(key) {
            return Err(AdditoryError::column_not_found(key, &target.column_names()));
        }
        if !reference.has_column(key) {
            return Err(AdditoryError::column_not_found(key, &reference.column_names()));
        }
    }
    
    // Validate fetch columns exist in reference
    for fetch_col in &fetch {
        let source = fetch_col.original();
        if !reference.has_column(source) {
            return Err(AdditoryError::column_not_found(source, &reference.column_names()));
        }
    }
    
    // Perform join
    let join_keys_vec: Vec<String> = join_keys.iter().map(|s| s.to_string()).collect();
    let mut result = target.join(
        &reference,
        &join_keys_vec,
        &join_keys_vec,
        join_type,
    )?;
    
    // Apply renames if specified
    for fetch_col in &fetch {
        if fetch_col.is_rename() {
            let original = fetch_col.original();
            let target_name = fetch_col.target();
            if result.has_column(original) && original != target_name {
                result = result.rename(original, target_name)?;
            }
        }
    }
    
    // Apply position if specified
    if let Some(pos) = position {
        let new_col_names: Vec<String> = fetch.iter().map(|f| f.target().to_string()).collect();
        crate::to::position::apply_position(result, &new_col_names, &pos)
    } else {
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FetchColumn, Against};
    use polars::prelude::*;
    
    #[test]
    fn test_to_basic() {
        // Create test DataFrames
        let target_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let mut params = UniversalParams::default();
        params.fetch = Some(vec![FetchColumn::NoRename("name".to_string())]);
        params.against = Some(Against::Single("id".to_string()));
        params.reference = Some(reference);
        
        let result = to(target, params).unwrap();
        
        assert!(result.has_column("name"));
        assert_eq!(result.height(), 3);
    }
    
    #[test]
    fn test_to_missing_parameters() {
        let df = crate::DataFrame::empty();
        let params = UniversalParams::default();
        
        let result = to(df, params);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_to_left_join() {
        // 1:1 relationship - should work
        let target_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let mut params = UniversalParams::default();
        params.fetch = Some(vec![FetchColumn::NoRename("name".to_string())]);
        params.against = Some(Against::Single("id".to_string()));
        params.reference = Some(reference);
        params.join_type = Some(crate::core::JoinType::Left);
        
        let result = to(target, params).unwrap();
        
        assert!(result.has_column("name"));
        assert_eq!(result.height(), 3);
    }
    
    #[test]
    fn test_to_inner_join() {
        // 1:1 relationship - should work
        let target_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 2],
            "name" => &["Alice", "Bob"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let mut params = UniversalParams::default();
        params.fetch = Some(vec![FetchColumn::NoRename("name".to_string())]);
        params.against = Some(Against::Single("id".to_string()));
        params.reference = Some(reference);
        params.join_type = Some(crate::core::JoinType::Inner);
        
        let result = to(target, params).unwrap();
        
        assert!(result.has_column("name"));
        assert_eq!(result.height(), 2); // Only matching rows
    }
    
    #[test]
    fn test_to_join_type_validation_fails() {
        // 1:many relationship - should fail for explicit join types
        let target_df = df! {
            "id" => &[1, 2, 3],
            "value" => &[10, 20, 30],
        }.unwrap();
        
        let reference_df = df! {
            "id" => &[1, 1, 2, 2, 3],  // Duplicates
            "name" => &["Alice", "Alice2", "Bob", "Bob2", "Charlie"],
        }.unwrap();
        
        let target = crate::DataFrame::from_polars(target_df);
        let reference = crate::DataFrame::from_polars(reference_df);
        
        let mut params = UniversalParams::default();
        params.fetch = Some(vec![FetchColumn::NoRename("name".to_string())]);
        params.against = Some(Against::Single("id".to_string()));
        params.reference = Some(reference);
        params.join_type = Some(crate::core::JoinType::Left);
        
        let result = to(target, params);
        assert!(result.is_err());
        
        // Error message should mention 1:many relationship
        let err_msg = format!("{:?}", result.unwrap_err());
        assert!(err_msg.contains("1:many"));
    }
}
