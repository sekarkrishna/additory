//! DataFrame abstraction layer
//!
//! Provides a unified interface for Pandas, Polars, and cuDF DataFrames.
//! Internally uses Polars for all operations, with automatic conversion.

use polars::prelude::*;
use crate::core::errors::{AdditoryError, AdditoryResult};
use crate::core::types::DataFrameType;

// Type alias for clarity
type Column = polars::prelude::Column;

/// Unified DataFrame wrapper
///
/// Wraps Polars DataFrame internally, with metadata about original type
/// for proper conversion back to user's format.
#[derive(Debug, Clone)]
pub struct DataFrame {
    /// Internal Polars DataFrame
    inner: PolarsDataFrame,
    
    /// Original DataFrame type (for conversion back)
    original_type: DataFrameType,
}

impl DataFrame {
    /// Create DataFrame from Polars DataFrame
    pub fn from_polars(df: PolarsDataFrame) -> Self {
        Self {
            inner: df,
            original_type: DataFrameType::Polars,
        }
    }

    /// Create DataFrame with explicit type
    pub fn new(df: PolarsDataFrame, df_type: DataFrameType) -> Self {
        Self {
            inner: df,
            original_type: df_type,
        }
    }

    /// Get reference to inner Polars DataFrame
    pub fn inner(&self) -> &PolarsDataFrame {
        &self.inner
    }

    /// Get mutable reference to inner Polars DataFrame
    pub fn inner_mut(&mut self) -> &mut PolarsDataFrame {
        &mut self.inner
    }

    /// Consume and return inner Polars DataFrame
    pub fn into_inner(self) -> PolarsDataFrame {
        self.inner
    }

    /// Get original DataFrame type
    pub fn original_type(&self) -> DataFrameType {
        self.original_type
    }

    /// Get number of rows
    pub fn height(&self) -> usize {
        self.inner.height()
    }

    /// Get number of columns
    pub fn width(&self) -> usize {
        self.inner.width()
    }

    /// Get column names
    pub fn column_names(&self) -> Vec<String> {
        self.inner
            .get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Check if column exists
    pub fn has_column(&self, name: &str) -> bool {
        self.inner.column(name).is_ok()
    }

    /// Get column by name
    pub fn column(&self, name: &str) -> AdditoryResult<&Column> {
        self.inner
            .column(name)
            .map_err(|_| AdditoryError::column_not_found(name, &self.column_names()))
    }

    /// Select columns
    pub fn select(&self, columns: &[String]) -> AdditoryResult<Self> {
        // Validate all columns exist
        for col in columns {
            if !self.has_column(col) {
                return Err(AdditoryError::column_not_found(col, &self.column_names()));
            }
        }

        let df = self.inner
            .select(columns)
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Add column
    pub fn with_column(&self, column: Column) -> AdditoryResult<Self> {
        // Clone the DataFrame first, then add column
        let mut df = self.inner.clone();
        df.with_column(column)
            .map_err(AdditoryError::Polars)?;
        
        Ok(Self::new(df, self.original_type))
    }

    /// Rename column
    pub fn rename(&self, old_name: &str, new_name: &str) -> AdditoryResult<Self> {
        if !self.has_column(old_name) {
            return Err(AdditoryError::column_not_found(old_name, &self.column_names()));
        }

        // Clone the DataFrame first, then rename
        let mut df = self.inner.clone();
        df.rename(old_name, new_name.into())
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Filter rows
    pub fn filter(&self, mask: &BooleanChunked) -> AdditoryResult<Self> {
        let df = self.inner
            .filter(mask)
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Join with another DataFrame
    pub fn join(
        &self,
        other: &DataFrame,
        left_on: &[String],
        right_on: &[String],
        how: JoinType,
    ) -> AdditoryResult<Self> {
        use polars::prelude::JoinArgs;
        
        let args = JoinArgs::new(how);
        let df = self.inner
            .join(&other.inner, left_on, right_on, args)
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Vertical concatenation (stack rows)
    pub fn vstack(&self, other: &DataFrame) -> AdditoryResult<Self> {
        let df = self.inner
            .vstack(&other.inner)
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Horizontal concatenation (add columns)
    pub fn hstack(&self, columns: &[Column]) -> AdditoryResult<Self> {
        // Polars hstack returns a new DataFrame, no need to clone first
        let df = self.inner
            .hstack(columns)
            .map_err(AdditoryError::Polars)?;

        Ok(Self::new(df, self.original_type))
    }

    /// Create empty DataFrame
    pub fn empty() -> Self {
        Self::from_polars(PolarsDataFrame::empty())
    }

    /// Check if DataFrame is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
    
    /// Convert DataFrame to Arrow IPC bytes
    ///
    /// Used for efficient transfer between Rust and Python via PyO3.
    ///
    /// # Returns
    ///
    /// * `AdditoryResult<Vec<u8>>` - DataFrame serialized as Arrow IPC bytes
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let df_bytes = df.to_arrow_ipc_bytes()?;
    /// // Send to Python via PyO3
    /// ```
    #[cfg(feature = "python")]
    pub fn to_arrow_ipc_bytes(&self) -> AdditoryResult<Vec<u8>> {
        use std::io::Cursor;
        use polars::io::ipc::IpcWriter;
        
        let mut buffer = Cursor::new(Vec::new());
        // Clone the DataFrame to get a mutable reference
        let mut df = self.inner.clone();
        IpcWriter::new(&mut buffer)
            .finish(&mut df)
            .map_err(|e| AdditoryError::operation(
                "Failed to serialize DataFrame to Arrow IPC",
                &e.to_string()
            ))?;
        
        Ok(buffer.into_inner())
    }
    
    /// Create DataFrame from Arrow IPC bytes
    ///
    /// Used for efficient transfer between Rust and Python via PyO3.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Arrow IPC serialized DataFrame
    /// * `df_type` - Original DataFrame type for proper conversion
    ///
    /// # Returns
    ///
    /// * `AdditoryResult<DataFrame>` - Deserialized DataFrame
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let df = DataFrame::from_arrow_ipc_bytes(&bytes, DataFrameType::Polars)?;
    /// ```
    #[cfg(feature = "python")]
    pub fn from_arrow_ipc_bytes(bytes: &[u8], df_type: DataFrameType) -> AdditoryResult<Self> {
        use std::io::Cursor;
        use polars::io::ipc::IpcReader;
        
        let cursor = Cursor::new(bytes);
        let df = IpcReader::new(cursor)
            .finish()
            .map_err(|e| AdditoryError::operation(
                "Failed to deserialize DataFrame from Arrow IPC",
                &e.to_string()
            ))?;
        
        Ok(Self::new(df, df_type))
    }
}

// Type alias for Polars DataFrame to avoid confusion
type PolarsDataFrame = polars::frame::DataFrame;

#[cfg(test)]
mod tests {
    use super::*;
    use polars::prelude::*;

    fn create_test_df() -> super::DataFrame {
        let polars_df = df! {
            "name" => &["Alice", "Bob", "Charlie"],
            "age" => &[25, 30, 35],
            "salary" => &[50000, 60000, 70000],
        }
        .unwrap();

        super::DataFrame::from_polars(polars_df)
    }

    #[test]
    fn test_dataframe_creation() {
        let df = create_test_df();
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 3);
        assert_eq!(df.original_type(), DataFrameType::Polars);
    }

    #[test]
    fn test_column_operations() {
        let df = create_test_df();
        
        assert!(df.has_column("name"));
        assert!(df.has_column("age"));
        assert!(!df.has_column("nonexistent"));

        let names = df.column_names();
        assert_eq!(names, vec!["name", "age", "salary"]);
    }

    #[test]
    fn test_select() {
        let df = create_test_df();
        let selected = df.select(&["name".to_string(), "age".to_string()]).unwrap();
        
        assert_eq!(selected.width(), 2);
        assert!(selected.has_column("name"));
        assert!(selected.has_column("age"));
        assert!(!selected.has_column("salary"));
    }

    #[test]
    fn test_rename() {
        let df = create_test_df();
        let renamed = df.rename("name", "employee_name").unwrap();
        
        assert!(renamed.has_column("employee_name"));
        assert!(!renamed.has_column("name"));
    }

    #[test]
    fn test_column_not_found_error() {
        let df = create_test_df();
        let result = df.select(&["nonexistent".to_string()]);
        
        assert!(result.is_err());
        match result {
            Err(AdditoryError::ColumnNotFound(col, _)) => {
                assert_eq!(col, "nonexistent");
            }
            _ => panic!("Expected ColumnNotFound error"),
        }
    }
}
