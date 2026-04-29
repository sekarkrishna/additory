//! Core module - DataFrame abstraction, types, and errors

pub mod dataframe;
pub mod types;
pub mod errors;

// Re-exports
pub use dataframe::DataFrame;
pub use errors::{AdditoryError, AdditoryResult};
pub use types::{
    // Enums
    TransformMode,
    JoinType,
    SyntheticMode,
    FetchColumn,
    Against,
    By,
    Position,
    Expression,
    AsParam,
    AggregationMode,
    StrategyValue,
    DataFrameType,
    
    // Structs
    UniversalParams,
};
