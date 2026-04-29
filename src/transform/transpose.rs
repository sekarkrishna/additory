//! @transpose mode - Transpose DataFrame
//!
//! Implements DataFrame transposition (flip rows and columns).

use crate::core::{DataFrame, AdditoryResult, AdditoryError};

/// Transpose DataFrame (flip rows and columns)
///
/// # Parameters
/// - `df`: DataFrame to transpose
///
/// # Returns
/// Transposed DataFrame
pub fn transpose(df: DataFrame) -> AdditoryResult<DataFrame> {
    let mut inner = df.inner().clone();
    
    // Transpose using Polars
    let transposed = inner.transpose(None, None)
        .map_err(AdditoryError::Polars)?;
    
    Ok(DataFrame::from_polars(transposed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DataFrame as AdditoryDataFrame;
    use polars::prelude::*;
    
    #[test]
    fn test_transpose_basic() {
        let df_inner = df! {
            "A" => &[1, 2, 3],
            "B" => &[4, 5, 6],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        
        let result = transpose(df).unwrap();
        
        // Original: 3 rows x 2 columns
        // Transposed: 2 rows x 3 columns
        assert_eq!(result.height(), 2);
        assert_eq!(result.width(), 3);
    }
    
    #[test]
    fn test_transpose_single_row() {
        let df_inner = df! {
            "A" => &[1],
            "B" => &[2],
            "C" => &[3],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        
        let result = transpose(df).unwrap();
        
        // Original: 1 row x 3 columns
        // Transposed: 3 rows x 1 column
        assert_eq!(result.height(), 3);
        assert_eq!(result.width(), 1);
    }
    
    #[test]
    fn test_transpose_single_column() {
        let df_inner = df! {
            "A" => &[1, 2, 3, 4],
        }.unwrap();
        
        let df = AdditoryDataFrame::from_polars(df_inner);
        
        let result = transpose(df).unwrap();
        
        // Original: 4 rows x 1 column
        // Transposed: 1 row x 4 columns
        assert_eq!(result.height(), 1);
        assert_eq!(result.width(), 4);
    }
}
