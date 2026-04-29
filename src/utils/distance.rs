//! Distance calculation utilities for KNN imputation
//!
//! This module provides distance metrics for computing similarity between data points.
//! Supports Euclidean, Manhattan, and Cosine distance calculations.

/// Trait for distance calculation between two vectors
pub trait DistanceCalculator {
    /// Calculate distance between two vectors
    ///
    /// # Arguments
    /// * `row1` - First vector
    /// * `row2` - Second vector
    ///
    /// # Returns
    /// Distance value (non-negative)
    fn calculate(&self, row1: &[f64], row2: &[f64]) -> f64;
}

/// Euclidean distance calculator
///
/// Computes distance as: sqrt(sum((x_i - y_i)^2))
pub struct EuclideanDistance;

impl DistanceCalculator for EuclideanDistance {
    fn calculate(&self, row1: &[f64], row2: &[f64]) -> f64 {
        row1.iter()
            .zip(row2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }
}

/// Manhattan distance calculator
///
/// Computes distance as: sum(|x_i - y_i|)
pub struct ManhattanDistance;

impl DistanceCalculator for ManhattanDistance {
    fn calculate(&self, row1: &[f64], row2: &[f64]) -> f64 {
        row1.iter()
            .zip(row2.iter())
            .map(|(a, b)| (a - b).abs())
            .sum()
    }
}

/// Cosine distance calculator
///
/// Computes distance as: 1 - (dot(x,y) / (norm(x) * norm(y)))
pub struct CosineDistance;

impl DistanceCalculator for CosineDistance {
    fn calculate(&self, row1: &[f64], row2: &[f64]) -> f64 {
        let dot_product: f64 = row1.iter().zip(row2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f64 = row1.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();
        let norm2: f64 = row2.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 1.0; // Maximum distance
        }

        1.0 - (dot_product / (norm1 * norm2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Property-Based Tests
    // ============================================================================
    
    use proptest::prelude::*;

    // Helper to generate non-empty vectors of reasonable size
    fn vec_strategy() -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(
            // Generate finite f64 values in a reasonable range to avoid overflow
            // Range: -1000.0 to 1000.0 (sufficient for testing distance properties)
            -1000.0..1000.0,
            1..20  // Vector size between 1 and 20
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Distance symmetry
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_euclidean_distance_symmetry(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            // Ensure vectors have same length
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = EuclideanDistance;
            let dist_ab = calc.calculate(v1, v2);
            let dist_ba = calc.calculate(v2, v1);
            
            // Distance should be symmetric: d(a,b) == d(b,a)
            prop_assert!(
                (dist_ab - dist_ba).abs() < 1e-10,
                "Euclidean distance not symmetric: d(a,b)={}, d(b,a)={}",
                dist_ab, dist_ba
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Distance symmetry
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_manhattan_distance_symmetry(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = ManhattanDistance;
            let dist_ab = calc.calculate(v1, v2);
            let dist_ba = calc.calculate(v2, v1);
            
            prop_assert!(
                (dist_ab - dist_ba).abs() < 1e-10,
                "Manhattan distance not symmetric: d(a,b)={}, d(b,a)={}",
                dist_ab, dist_ba
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Distance symmetry
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_cosine_distance_symmetry(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = CosineDistance;
            let dist_ab = calc.calculate(v1, v2);
            let dist_ba = calc.calculate(v2, v1);
            
            prop_assert!(
                (dist_ab - dist_ba).abs() < 1e-10,
                "Cosine distance not symmetric: d(a,b)={}, d(b,a)={}",
                dist_ab, dist_ba
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Non-negativity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_euclidean_distance_non_negative(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = EuclideanDistance;
            let distance = calc.calculate(v1, v2);
            
            // All distances must be non-negative
            prop_assert!(
                distance >= 0.0,
                "Euclidean distance is negative: {}",
                distance
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Non-negativity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_manhattan_distance_non_negative(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = ManhattanDistance;
            let distance = calc.calculate(v1, v2);
            
            prop_assert!(
                distance >= 0.0,
                "Manhattan distance is negative: {}",
                distance
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Non-negativity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_cosine_distance_non_negative(
            vec1 in vec_strategy(),
            vec2 in vec_strategy()
        ) {
            let min_len = vec1.len().min(vec2.len());
            let v1 = &vec1[..min_len];
            let v2 = &vec2[..min_len];
            
            let calc = CosineDistance;
            let distance = calc.calculate(v1, v2);
            
            prop_assert!(
                distance >= 0.0,
                "Cosine distance is negative: {}",
                distance
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Identity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_euclidean_distance_identity(vec in vec_strategy()) {
            let calc = EuclideanDistance;
            let distance = calc.calculate(&vec, &vec);
            
            // Distance from a vector to itself must be zero
            prop_assert!(
                distance.abs() < 1e-10,
                "Euclidean distance(a,a) is not zero: {}",
                distance
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Identity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_manhattan_distance_identity(vec in vec_strategy()) {
            let calc = ManhattanDistance;
            let distance = calc.calculate(&vec, &vec);
            
            prop_assert!(
                distance.abs() < 1e-10,
                "Manhattan distance(a,a) is not zero: {}",
                distance
            );
        }

        // Feature: rust-knn-deduce-synthetic-wrapper, Property: Identity
        // **Validates: Requirements 1.8, 1.9, 1.10**
        #[test]
        fn prop_cosine_distance_identity(vec in vec_strategy()) {
            let calc = CosineDistance;
            let distance = calc.calculate(&vec, &vec);
            
            prop_assert!(
                distance.abs() < 1e-10,
                "Cosine distance(a,a) is not zero: {}",
                distance
            );
        }
    }

    // ============================================================================
    // Unit Tests (from task 1.1)
    // ============================================================================

    #[test]
    fn test_euclidean_distance_known_vectors() {
        let calc = EuclideanDistance;
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![4.0, 6.0];
        
        // Distance = sqrt((4-1)^2 + (6-2)^2) = sqrt(9 + 16) = sqrt(25) = 5.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 5.0).abs() < 1e-10, "Expected 5.0, got {}", distance);
    }

    #[test]
    fn test_euclidean_distance_identical_vectors() {
        let calc = EuclideanDistance;
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 1e-10, "Expected 0.0, got {}", distance);
    }

    #[test]
    fn test_euclidean_distance_negative_values() {
        let calc = EuclideanDistance;
        let vec1 = vec![-1.0, -2.0];
        let vec2 = vec![1.0, 2.0];
        
        // Distance = sqrt((1-(-1))^2 + (2-(-2))^2) = sqrt(4 + 16) = sqrt(20) ≈ 4.472
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 4.472135954999579).abs() < 1e-10, "Expected ~4.472, got {}", distance);
    }

    #[test]
    fn test_manhattan_distance_known_vectors() {
        let calc = ManhattanDistance;
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![4.0, 6.0];
        
        // Distance = |4-1| + |6-2| = 3 + 4 = 7.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 7.0).abs() < 1e-10, "Expected 7.0, got {}", distance);
    }

    #[test]
    fn test_manhattan_distance_identical_vectors() {
        let calc = ManhattanDistance;
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 1e-10, "Expected 0.0, got {}", distance);
    }

    #[test]
    fn test_manhattan_distance_negative_values() {
        let calc = ManhattanDistance;
        let vec1 = vec![-1.0, -2.0];
        let vec2 = vec![1.0, 2.0];
        
        // Distance = |1-(-1)| + |2-(-2)| = 2 + 4 = 6.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 6.0).abs() < 1e-10, "Expected 6.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_orthogonal_vectors() {
        let calc = CosineDistance;
        let vec1 = vec![1.0, 0.0];
        let vec2 = vec![0.0, 1.0];
        
        // Cosine similarity = 0 (orthogonal), so distance = 1.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 1.0).abs() < 1e-10, "Expected 1.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_identical_vectors() {
        let calc = CosineDistance;
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 3.0];
        
        // Cosine similarity = 1.0 (identical), so distance = 0.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 1e-10, "Expected 0.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_opposite_vectors() {
        let calc = CosineDistance;
        let vec1 = vec![1.0, 2.0];
        let vec2 = vec![-1.0, -2.0];
        
        // Cosine similarity = -1.0 (opposite), so distance = 2.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 2.0).abs() < 1e-10, "Expected 2.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_zero_vector() {
        let calc = CosineDistance;
        let vec1 = vec![0.0, 0.0];
        let vec2 = vec![1.0, 2.0];
        
        // Zero vector should return maximum distance (1.0)
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 1.0).abs() < 1e-10, "Expected 1.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_scaled_vectors() {
        let calc = CosineDistance;
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![2.0, 4.0, 6.0];
        
        // Scaled vectors have same direction, cosine similarity = 1.0, distance = 0.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 1e-10, "Expected 0.0, got {}", distance);
    }

    #[test]
    fn test_euclidean_distance_single_dimension() {
        let calc = EuclideanDistance;
        let vec1 = vec![5.0];
        let vec2 = vec![2.0];
        
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 3.0).abs() < 1e-10, "Expected 3.0, got {}", distance);
    }

    #[test]
    fn test_manhattan_distance_single_dimension() {
        let calc = ManhattanDistance;
        let vec1 = vec![5.0];
        let vec2 = vec![2.0];
        
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 3.0).abs() < 1e-10, "Expected 3.0, got {}", distance);
    }

    #[test]
    fn test_cosine_distance_single_dimension() {
        let calc = CosineDistance;
        let vec1 = vec![5.0];
        let vec2 = vec![2.0];
        
        // Same direction, distance should be 0.0
        let distance = calc.calculate(&vec1, &vec2);
        assert!((distance - 0.0).abs() < 1e-10, "Expected 0.0, got {}", distance);
    }
}
