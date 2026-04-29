//! add.to() module - Add data FROM external source
//!
//! Three modes:
//! - LOOKUP (default): Add columns from reference to target
//! - @new: Create new DataFrame from reference
//! - @merge: Merge multiple DataFrames

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::core::types::UniversalParams;

pub mod lookup;
pub mod new;
pub mod merge;
pub mod strategy;

/// Main entry point for add.to()
pub fn to(
    target: Option<DataFrame>,
    params: UniversalParams,
) -> AdditoryResult<DataFrame> {
    // Detect mode from params
    let mode = detect_mode(&params)?;
    
    match mode {
        "LOOKUP" => {
            // LOOKUP requires target DataFrame
            let target_df = target.ok_or_else(|| 
                AdditoryError::missing_parameter("target", "LOOKUP mode requires a target DataFrame")
            )?;
            
            // Extract reference DataFrame from params
            let reference = params.reference.ok_or_else(||
                AdditoryError::missing_parameter("fetch_from", "LOOKUP mode requires a reference DataFrame")
            )?;
            
            // Extract by parameter
            let by = params.by.ok_or_else(||
                AdditoryError::missing_parameter("by", "LOOKUP mode requires a key column for joining")
            )?;
            
            // Build LookupParams
            let lookup_params = lookup::LookupParams {
                reference,
                fetch: params.fetch,
                by,
                logging: params.logging,
            };
            
            // Execute LOOKUP
            lookup::execute(target_df, lookup_params)
        },
        _ => Err(AdditoryError::OperationFailed(
            format!("Unsupported mode: {}", mode),
            "Supported modes: LOOKUP, @new, @merge".to_string()
        ))
    }
}

/// Detect which mode to use based on parameters
fn detect_mode(params: &UniversalParams) -> AdditoryResult<&'static str> {
    // If explicit mode provided, use it
    if let Some(mode_str) = &params.explicit_mode {
        // Validate mode is valid for add.to()
        match mode_str.as_str() {
            "LOOKUP" => return Ok("LOOKUP"),
            "@new" => return Ok("@new"),
            "@merge" => return Ok("@merge"),
            _ => return Err(AdditoryError::OperationFailed(
                format!("Invalid mode for add.to(): {}", mode_str),
                "Valid modes: LOOKUP, @new, @merge".to_string()
            ))
        }
    }
    
    // Auto-detect based on parameters
    // LOOKUP: has reference and by
    if params.reference.is_some() && params.by.is_some() {
        return Ok("LOOKUP");
    }
    
    Err(AdditoryError::OperationFailed(
        "Cannot determine mode from parameters".to_string(),
        "Provide explicit mode or required parameters (fetch_from and by for LOOKUP)".to_string()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_df() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    fn create_reference_df() -> crate::core::DataFrame {
        let polars_df = df! {
            "id" => &[1, 2, 3],
            "age" => &[25, 30, 35],
        }
        .unwrap();
        crate::core::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_detect_mode_lookup() {
        let reference = create_reference_df();
        
        let mut params = UniversalParams::default();
        params.reference = Some(reference);
        params.by = Some("id".to_string());
        
        let mode = detect_mode(&params).unwrap();
        assert_eq!(mode, "LOOKUP");
    }

    #[test]
    fn test_detect_mode_explicit() {
        let mut params = UniversalParams::default();
        params.explicit_mode = Some("LOOKUP".to_string());
        
        let mode = detect_mode(&params).unwrap();
        assert_eq!(mode, "LOOKUP");
    }

    #[test]
    fn test_detect_mode_missing_params() {
        let params = UniversalParams::default();
        
        let result = detect_mode(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_lookup_basic() {
        let target = create_test_df();
        let reference = create_reference_df();
        
        let mut params = UniversalParams::default();
        params.reference = Some(reference);
        params.by = Some("id".to_string());
        params.logging = false;
        
        let result = to(Some(target), params).unwrap();
        
        assert_eq!(result.height(), 3);
        assert!(result.has_column("age"));
        assert!(result.has_column("name"));
    }

    #[test]
    fn test_to_missing_target() {
        let reference = create_reference_df();
        
        let mut params = UniversalParams::default();
        params.reference = Some(reference);
        params.by = Some("id".to_string());
        
        let result = to(None, params);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_missing_reference() {
        let target = create_test_df();
        
        let mut params = UniversalParams::default();
        params.by = Some("id".to_string());
        
        let result = to(Some(target), params);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_missing_by() {
        let target = create_test_df();
        let reference = create_reference_df();
        
        let mut params = UniversalParams::default();
        params.reference = Some(reference);
        
        let result = to(Some(target), params);
        assert!(result.is_err());
    }
}
