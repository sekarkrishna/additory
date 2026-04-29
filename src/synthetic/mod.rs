//! add.synthetic() module - Create or augment with synthetic data
//!
//! Two modes:
//! - @new: Create synthetic DataFrame
//! - @augment: Add synthetic rows to existing DataFrame

use crate::core::{DataFrame, AdditoryResult, AdditoryError};
use crate::core::types::UniversalParams;
use crate::utils::logging::Logger;

pub mod new;
pub mod augment;

/// Synthetic mode enum
#[derive(Debug, Clone, PartialEq, Eq)]
enum SyntheticMode {
    New,
    Augment,
}

/// Detect synthetic mode from parameters
fn detect_mode(df: Option<&DataFrame>, mode_str: Option<&str>) -> Result<SyntheticMode, AdditoryError> {
    match (df, mode_str) {
        (None, Some("@new")) => Ok(SyntheticMode::New),
        (None, Some("@analyze")) => Err(AdditoryError::mode_parsing(
            "@analyze",
            &["Mode '@analyze' has been moved. Use add.analyze(df) or add.scan('@analyze', df) instead."]
        )),
        (None, Some("@analyse")) => Err(AdditoryError::mode_parsing(
            "@analyse",
            &["Mode '@analyse' has been moved. Use add.analyse(df) or add.scan('@analyse', df) instead."]
        )),
        (Some(_), Some("@analyze")) => Err(AdditoryError::mode_parsing(
            "@analyze",
            &["Mode '@analyze' has been moved. Use add.analyze(df) or add.scan('@analyze', df) instead."]
        )),
        (Some(_), Some("@analyse")) => Err(AdditoryError::mode_parsing(
            "@analyse",
            &["Mode '@analyse' has been moved. Use add.analyse(df) or add.scan('@analyse', df) instead."]
        )),
        (Some(_), Some("@augment")) => Ok(SyntheticMode::Augment),
        (Some(_), None) => Ok(SyntheticMode::Augment),
        (None, None) => Err(AdditoryError::mode_parsing(
            "None",
            &["@new", "@augment"]
        )),
        _ => Err(AdditoryError::mode_parsing(
            mode_str.unwrap_or("unknown"),
            &["@new", "@augment"]
        )),
    }
}

/// Main entry point for add.synthetic()
pub fn synthetic(
    mode_str: Option<&str>,
    df: Option<DataFrame>,
    params: UniversalParams,
) -> AdditoryResult<DataFrame> {
    // Create logger
    let logger = if params.logging {
        Logger::new(true)
    } else {
        Logger::new(false)
    };

    // Detect mode
    let mode = detect_mode(df.as_ref(), mode_str)?;
    
    logger.log_start("add.synthetic()", &format!("{:?}", mode));

    // Route to appropriate module
    match mode {
        SyntheticMode::New => new::execute(params, &logger),
        SyntheticMode::Augment => {
            let df = df.ok_or_else(|| AdditoryError::missing_parameter(
                "dataframe",
                "Augment mode requires a DataFrame"
            ))?;
            augment::execute(df, params, &logger)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_detection() {
        // @new mode — no DataFrame, mode string '@new'
        assert_eq!(
            detect_mode(None, Some("@new")).unwrap(),
            SyntheticMode::New
        );

        // Augment mode — DataFrame present, no mode string
        let df = DataFrame::from_polars(polars::prelude::DataFrame::empty());
        assert_eq!(
            detect_mode(Some(&df), None).unwrap(),
            SyntheticMode::Augment
        );

        // Augment mode — DataFrame present, explicit '@augment' mode string
        // (Python side now passes params['mode'] = '@augment' in augment mode)
        assert_eq!(
            detect_mode(Some(&df), Some("@augment")).unwrap(),
            SyntheticMode::Augment
        );

        // @analyze mode should error with helpful message
        let err = detect_mode(Some(&df), Some("@analyze")).unwrap_err();
        assert!(err.to_string().contains("add.analyze(df)"));

        // @analyse mode should error with helpful message
        let err = detect_mode(Some(&df), Some("@analyse")).unwrap_err();
        assert!(err.to_string().contains("add.analyse(df)"));

        // Error cases
        assert!(detect_mode(None, None).is_err());
        assert!(detect_mode(None, Some("@analyze")).is_err());
    }
}
