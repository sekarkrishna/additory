//! Utility modules - validation, logging, type detection, distance calculation, TF-IDF vectorization

pub mod validation;
pub mod logging;
pub mod type_detection;
pub mod distance;
pub mod tfidf;

// Re-exports
pub use validation::Validator;
pub use logging::Logger;
pub use type_detection::detect_dataframe_type;
pub use distance::{DistanceCalculator, EuclideanDistance, ManhattanDistance, CosineDistance};
pub use tfidf::{TfidfVectorizer, cosine_similarity};
