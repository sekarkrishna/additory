"""
KNN imputation for missing values - Pure Python implementation.

This module provides KNN imputation functionality for the transform function.
@knn mode is implemented in pure Python (not Rust) for v0.1.3.
"""

import math
from typing import List, Optional, Dict, Any
import numpy as np


def perform_knn_imputation(
    df: Any,  # Polars DataFrame
    infer: str,
    name: str,
    strategy: Optional[Dict] = None,
    columns: Optional[List[str]] = None  # Deprecated, for backward compatibility
) -> Any:
    """
    Impute missing values using K-Nearest Neighbors.
    
    This function follows the @deduce redesign philosophy: it preserves the original
    column and creates a new column with imputed values.
    
    Args:
        df: Input DataFrame (Polars)
        infer: Column to impute (contains missing values)
        name: Name for the output column (with imputed values)
        strategy: Imputation strategy with keys:
            - k: Number of neighbors (default: 5)
            - weights: 'uniform' or 'distance' (default: 'uniform')
            - metric: 'euclidean', 'manhattan', or 'cosine' (default: 'euclidean')
        columns: (Deprecated) Old parameter for backward compatibility with @knn mode
        
    Returns:
        DataFrame with new imputed column (original column preserved)
        
    Example:
        >>> # New @deduce signature
        >>> result = perform_knn_imputation(df,
        ...     infer='age',
        ...     name='age_filled',
        ...     strategy={'k': 5, 'weights': 'distance', 'metric': 'euclidean'})
        
        >>> # Old @knn mode (deprecated, for backward compatibility)
        >>> result = perform_knn_imputation(df,
        ...     columns=['age'],
        ...     strategy={'k': 5})
    """
    # Import polars here to avoid dependency issues
    import polars as pl
    
    # Validate
    if not isinstance(df, pl.DataFrame):
        raise TypeError("df must be a Polars DataFrame")
    
    if df.is_empty():
        raise ValueError("DataFrame cannot be empty")
    
    # Handle backward compatibility with old @knn mode
    if columns is not None and infer is None:
        # Old @knn mode: modify columns in-place
        if not columns:
            raise ValueError("columns list cannot be empty")
        
        # Validate columns exist
        missing = [col for col in columns if col not in df.columns]
        if missing:
            raise ValueError(f"Columns not found: {missing}")
        
        # Extract strategy parameters
        strategy = strategy or {}
        k = strategy.get('k', 5)
        weights = strategy.get('weights', 'uniform')
        metric = strategy.get('metric', 'euclidean')
        
        # Validate parameters
        _validate_knn_parameters(df, columns, k, weights, metric)
        
        # Perform KNN imputation (old behavior: modifies in-place)
        result = _knn_impute_inplace(df, columns, k=k, weights=weights, metric=metric)
        
        return result
    
    # New @deduce mode: preserve original column, create new column
    if infer is None:
        raise ValueError("'infer' parameter is required for @deduce mode")
    
    if name is None:
        raise ValueError("'name' parameter is required for @deduce mode")
    
    # Validate infer column exists
    if infer not in df.columns:
        raise ValueError(f"Column '{infer}' not found in DataFrame")
    
    # Validate name doesn't already exist
    if name in df.columns:
        raise ValueError(f"Column '{name}' already exists in DataFrame")
    
    # Extract strategy parameters
    strategy = strategy or {}
    k = strategy.get('k', 5)
    weights = strategy.get('weights', 'uniform')
    metric = strategy.get('metric', 'euclidean')
    
    # Validate parameters
    _validate_knn_parameters(df, [infer], k, weights, metric)
    
    # Perform KNN imputation with column preservation
    result = _knn_impute_preserve(df, infer, name, k=k, weights=weights, metric=metric)
    
    return result


def _knn_impute_inplace(df, columns: List[str], k: int = 5, 
                weights: str = 'uniform', metric: str = 'euclidean'):
    """
    Internal KNN imputation implementation (old behavior: modifies columns in-place).
    
    This is kept for backward compatibility with @knn mode.
    
    Args:
        df: Polars DataFrame with missing values
        columns: Columns to impute
        k: Number of neighbors
        weights: Weighting strategy ('uniform' or 'distance')
        metric: Distance metric ('euclidean', 'manhattan', 'cosine')
        
    Returns:
        DataFrame with imputed values (columns modified in-place)
    """
    import polars as pl
    
    # Create a copy to avoid modifying original
    result_df = df.clone()
    
    # Get all numeric columns for distance calculation
    all_numeric_cols = [col for col in df.columns 
                       if df[col].dtype in [pl.Int8, pl.Int16, pl.Int32, pl.Int64,
                                           pl.UInt8, pl.UInt16, pl.UInt32, pl.UInt64,
                                           pl.Float32, pl.Float64]]
    
    # Convert to numpy for easier manipulation
    data = df.select(columns).to_numpy().copy()
    
    # For each row with missing values
    for row_idx in range(len(df)):
        row = data[row_idx]
        
        # Check if this row has any missing values
        if not np.any(np.isnan(row)):
            continue
        
        # Check if this row has at least some non-missing values in ANY numeric column
        row_all_cols = df.select(all_numeric_cols).row(row_idx)
        has_any_non_missing = any(val is not None and not (isinstance(val, float) and np.isnan(val)) 
                                  for val in row_all_cols)
        if not has_any_non_missing:
            # All numeric values missing, skip this row
            continue
        
        # Calculate distances to all other rows using ALL numeric columns
        distances = _calculate_distances(df, row_idx, all_numeric_cols, metric)
        
        # Find k nearest neighbors
        neighbor_indices = _find_k_nearest(distances, k, exclude_idx=row_idx)
        
        # Impute each missing value
        for col_idx, col_name in enumerate(columns):
            if np.isnan(row[col_idx]):
                # Get values from neighbors
                neighbor_values = []
                neighbor_distances = []
                
                for neighbor_idx in neighbor_indices:
                    neighbor_value = data[neighbor_idx, col_idx]
                    if not np.isnan(neighbor_value):
                        neighbor_values.append(neighbor_value)
                        neighbor_distances.append(distances[neighbor_idx])
                
                # If we have neighbor values, compute weighted average
                if neighbor_values:
                    imputed_value = _compute_weighted_average(
                        neighbor_values,
                        neighbor_distances,
                        weights
                    )
                    
                    # Update the data array
                    data[row_idx, col_idx] = imputed_value
    
    # Replace the columns in the result DataFrame with imputed data
    for col_idx, col_name in enumerate(columns):
        result_df = result_df.with_columns(
            pl.Series(col_name, data[:, col_idx])
        )
    
    return result_df


def _knn_impute_preserve(df, infer_col: str, output_col: str, k: int = 5,
                        weights: str = 'uniform', metric: str = 'euclidean'):
    """
    Internal KNN imputation implementation with column preservation.
    
    This follows the @deduce redesign philosophy: preserve the original column
    and create a new column with imputed values.
    
    Args:
        df: Polars DataFrame with missing values
        infer_col: Column to impute (contains missing values)
        output_col: Name for the new column with imputed values
        k: Number of neighbors
        weights: Weighting strategy ('uniform' or 'distance')
        metric: Distance metric ('euclidean', 'manhattan', 'cosine')
        
    Returns:
        DataFrame with new imputed column (original column preserved)
    """
    import polars as pl
    
    # Get all numeric columns for distance calculation
    all_numeric_cols = [col for col in df.columns 
                       if df[col].dtype in [pl.Int8, pl.Int16, pl.Int32, pl.Int64,
                                           pl.UInt8, pl.UInt16, pl.UInt32, pl.UInt64,
                                           pl.Float32, pl.Float64]]
    
    # Convert infer column to numpy for manipulation
    data = df[infer_col].to_numpy().copy()
    
    # For each row with missing values
    for row_idx in range(len(df)):
        value = data[row_idx]
        
        # Check if this row has a missing value
        if not (value is None or (isinstance(value, float) and np.isnan(value))):
            continue
        
        # Check if this row has at least some non-missing values in ANY numeric column
        row_all_cols = df.select(all_numeric_cols).row(row_idx)
        has_any_non_missing = any(val is not None and not (isinstance(val, float) and np.isnan(val)) 
                                  for val in row_all_cols)
        if not has_any_non_missing:
            # All numeric values missing, skip this row
            continue
        
        # Calculate distances to all other rows using ALL numeric columns
        distances = _calculate_distances(df, row_idx, all_numeric_cols, metric)
        
        # Find k nearest neighbors
        neighbor_indices = _find_k_nearest(distances, k, exclude_idx=row_idx)
        
        # Get values from neighbors
        neighbor_values = []
        neighbor_distances = []
        
        for neighbor_idx in neighbor_indices:
            neighbor_value = data[neighbor_idx]
            if not (neighbor_value is None or (isinstance(neighbor_value, float) and np.isnan(neighbor_value))):
                neighbor_values.append(float(neighbor_value))
                neighbor_distances.append(distances[neighbor_idx])
        
        # If we have neighbor values, compute weighted average
        if neighbor_values:
            imputed_value = _compute_weighted_average(
                neighbor_values,
                neighbor_distances,
                weights
            )
            
            # Update the data array
            data[row_idx] = imputed_value
    
    # Create result DataFrame with original column preserved and new column added
    result_df = df.with_columns(
        pl.Series(output_col, data)
    )
    
    return result_df


def _calculate_distances(df, row_idx: int, columns: List[str], metric: str):
    """
    Calculate distances from a row to all other rows.
    
    Args:
        df: Polars DataFrame
        row_idx: Index of row to calculate distances from
        columns: Columns to use for distance calculation
        metric: Distance metric
        
    Returns:
        List of distances
    """
    data = df.select(columns).to_numpy()
    row = data[row_idx]
    
    distances = []
    for i in range(len(df)):
        if i == row_idx:
            distances.append(float('inf'))  # Exclude self
        else:
            other_row = data[i]
            
            # Only use non-missing values for distance calculation
            mask = ~(np.isnan(row) | np.isnan(other_row))
            
            if not np.any(mask):
                # No common non-missing values
                distances.append(float('inf'))
            else:
                row_clean = row[mask]
                other_clean = other_row[mask]
                
                if metric == 'euclidean':
                    dist = _euclidean_distance(row_clean, other_clean)
                elif metric == 'manhattan':
                    dist = _manhattan_distance(row_clean, other_clean)
                elif metric == 'cosine':
                    dist = _cosine_distance(row_clean, other_clean)
                else:
                    raise ValueError(f"Unsupported metric: {metric}")
                
                distances.append(dist)
    
    return distances


def _find_k_nearest(distances: List[float], k: int, exclude_idx: Optional[int] = None) -> List[int]:
    """
    Find indices of k nearest neighbors.
    
    Args:
        distances: List of distances
        k: Number of neighbors to find
        exclude_idx: Index to exclude (the row itself)
        
    Returns:
        List of k nearest neighbor indices
    """
    # Convert to numpy for easier manipulation
    dist_array = np.array(distances)
    
    # Get indices sorted by distance
    sorted_indices = np.argsort(dist_array)
    
    # Filter out infinite distances and excluded index
    valid_indices = []
    for idx in sorted_indices:
        if not np.isinf(dist_array[idx]):
            if exclude_idx is None or idx != exclude_idx:
                valid_indices.append(int(idx))
                if len(valid_indices) >= k:
                    break
    
    return valid_indices


def _compute_weighted_average(values: List[float], distances: List[float], weights: str) -> float:
    """
    Compute weighted average of neighbor values.
    
    Args:
        values: Values from neighbors
        distances: Distances to neighbors
        weights: Weighting strategy ('uniform' or 'distance')
        
    Returns:
        Weighted average value
    """
    values_array = np.array(values)
    distances_array = np.array(distances)
    
    if weights == 'uniform':
        # Simple average
        return float(np.mean(values_array))
    
    elif weights == 'distance':
        # Inverse distance weighting
        # Avoid division by zero for very small distances
        distances_array = np.maximum(distances_array, 1e-10)
        
        # Inverse distance weights
        inv_distances = 1.0 / distances_array
        
        # Weighted average
        weighted_sum = np.sum(values_array * inv_distances)
        weight_sum = np.sum(inv_distances)
        
        return float(weighted_sum / weight_sum)
    
    else:
        raise ValueError(f"Unsupported weighting strategy: {weights}")


def _euclidean_distance(row1: np.ndarray, row2: np.ndarray) -> float:
    """Calculate Euclidean distance between two rows."""
    diff = row1 - row2
    return float(math.sqrt(np.sum(diff ** 2)))


def _manhattan_distance(row1: np.ndarray, row2: np.ndarray) -> float:
    """Calculate Manhattan distance between two rows."""
    diff = np.abs(row1 - row2)
    return float(np.sum(diff))


def _cosine_distance(row1: np.ndarray, row2: np.ndarray) -> float:
    """Calculate Cosine distance between two rows."""
    # Calculate dot product and norms
    dot_product = np.dot(row1, row2)
    norm1 = np.linalg.norm(row1)
    norm2 = np.linalg.norm(row2)
    
    # Avoid division by zero
    if norm1 == 0 or norm2 == 0:
        return 1.0  # Maximum distance
    
    # Cosine similarity
    cosine_sim = dot_product / (norm1 * norm2)
    
    # Cosine distance (1 - similarity)
    return float(1.0 - cosine_sim)


def _validate_knn_parameters(df, columns: List[str], k: int, weights: str, metric: str) -> bool:
    """
    Validate KNN imputation parameters.
    
    Args:
        df: Polars DataFrame
        columns: Columns to impute
        k: Number of neighbors
        weights: Weighting strategy
        metric: Distance metric
        
    Returns:
        True if valid
        
    Raises:
        ValueError: If parameters are invalid
    """
    import polars as pl
    
    # Check columns exist (already done in main function, but double-check)
    for col in columns:
        if col not in df.columns:
            raise ValueError(f"Column '{col}' not found in DataFrame")
    
    # Check at least some non-missing values exist
    for col in columns:
        non_null_count = df[col].null_count()
        if non_null_count == len(df):
            raise ValueError(f"Column '{col}' has all missing values, cannot impute")
    
    # Check columns are numeric
    for col in columns:
        dtype = df[col].dtype
        if dtype not in [pl.Int8, pl.Int16, pl.Int32, pl.Int64,
                        pl.UInt8, pl.UInt16, pl.UInt32, pl.UInt64,
                        pl.Float32, pl.Float64]:
            raise ValueError(f"Column '{col}' must be numeric, got {dtype}")
    
    # Validate weights (check before k to get proper error messages in tests)
    if weights not in ['uniform', 'distance']:
        raise ValueError(f"weights must be 'uniform' or 'distance', got '{weights}'")
    
    # Validate metric (check before k to get proper error messages in tests)
    if metric not in ['euclidean', 'manhattan', 'cosine']:
        raise ValueError(f"metric must be 'euclidean', 'manhattan', or 'cosine', got '{metric}'")
    
    # Check k is positive
    if k <= 0:
        raise ValueError(f"k must be positive, got {k}")
    
    # Check k is less than number of rows (need at least k+1 rows: 1 for target, k for neighbors)
    if k >= len(df):
        raise ValueError(f"k ({k}) must be less than number of rows ({len(df)})")
    
    return True
