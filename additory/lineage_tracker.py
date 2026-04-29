"""
Lineage Tracker module for recording transformation history

This module implements lineage tracking for additory operations, supporting
both pandas and polars DataFrames with different storage mechanisms:

- Pandas: Uses native DataFrame.attrs['lineage']
- Polars: Uses global registry with (id, version_counter) tuples as keys

The dual-storage approach addresses the following issues:
1. Polars DataFrames don't have a native .attrs mechanism
2. Python's id() function can reuse memory addresses after garbage collection
3. Version counters prevent collisions when addresses are reused
4. Weak references enable automatic cleanup when DataFrames are garbage collected

Session-Only Lineage (v0.1.4):
Lineage metadata is session-scoped and does not persist when DataFrames are
saved to disk. This applies to both pandas and Polars DataFrames. Persistent
lineage via add.save() and add.load() will be added in v0.2.0.
"""

from typing import Union, List, Dict, Any, Optional, Tuple
from datetime import datetime
import weakref
import pandas as pd
import polars as pl


# Global registry for Polars lineage (session-scoped memory)
# Uses (id, version_counter) tuples as keys to prevent id() reuse collisions
_polars_lineage_registry: Dict[Tuple[int, int], Dict] = {}
_polars_version_counter: int = 0
_polars_weak_refs: Dict[int, weakref.ref] = {}

# Reverse index for O(1) lookup: id -> (id, version_counter)
# This prevents O(n) scan of registry on every get call
_ID_TO_KEY: Dict[int, Tuple[int, int]] = {}


def _get_polars_key(df: pl.DataFrame) -> Tuple[int, int]:
    """
    Generate unique key for Polars DataFrame.
    
    Uses (id, version_counter) tuple to prevent id() reuse collisions.
    Maintains weak references for garbage collection cleanup.
    
    Args:
        df: Polars DataFrame
        
    Returns:
        Tuple of (id, version_counter) for use as registry key
    """
    global _polars_version_counter
    df_id = id(df)
    
    # Check if this id already has a version (O(1) lookup via reverse index)
    if df_id in _ID_TO_KEY:
        # Verify weak reference is still valid
        if df_id in _polars_weak_refs and _polars_weak_refs[df_id]() is not None:
            # Existing DataFrame - retrieve version
            return _ID_TO_KEY[df_id]
    
    # New DataFrame or garbage collected - assign new version
    _polars_version_counter += 1
    version = _polars_version_counter
    key = (df_id, version)
    
    # Store reverse index for O(1) lookup
    _ID_TO_KEY[df_id] = key
    
    # Create weak reference with cleanup callback
    _polars_weak_refs[df_id] = weakref.ref(df, lambda ref: _cleanup_registry(df_id))
    
    return key


def _cleanup_registry(df_id: int) -> None:
    """
    Clean up registry entries when DataFrame is garbage collected.
    
    Args:
        df_id: The id() of the garbage collected DataFrame
    """
    # Remove all registry entries for this id
    if df_id in _ID_TO_KEY:
        key = _ID_TO_KEY[df_id]
        if key in _polars_lineage_registry:
            del _polars_lineage_registry[key]
        del _ID_TO_KEY[df_id]
    
    # Remove weak reference
    if df_id in _polars_weak_refs:
        del _polars_weak_refs[df_id]


def _get_lineage(df: Union[pd.DataFrame, pl.DataFrame]) -> Optional[Dict]:
    """
    Get lineage metadata from DataFrame.
    
    Args:
        df: pandas or polars DataFrame
        
    Returns:
        Lineage metadata dictionary or None if not present
    """
    if isinstance(df, pd.DataFrame):
        # Pandas: use native attrs
        return df.attrs.get('lineage', None)
    elif isinstance(df, pl.DataFrame):
        # Polars: use global registry
        key = _get_polars_key(df)
        return _polars_lineage_registry.get(key, None)
    else:
        raise TypeError(f"Expected pandas or polars DataFrame, got {type(df).__name__}")


def _set_lineage(df: Union[pd.DataFrame, pl.DataFrame], lineage: Dict) -> Union[pd.DataFrame, pl.DataFrame]:
    """
    Set lineage metadata on DataFrame.
    
    Args:
        df: pandas or polars DataFrame
        lineage: Lineage metadata dictionary
        
    Returns:
        DataFrame with lineage metadata attached
    """
    if isinstance(df, pd.DataFrame):
        # Pandas: use native attrs
        df.attrs['lineage'] = lineage
        return df
    elif isinstance(df, pl.DataFrame):
        # Polars: use global registry
        key = _get_polars_key(df)
        _polars_lineage_registry[key] = lineage
        return df
    else:
        raise TypeError(f"Expected pandas or polars DataFrame, got {type(df).__name__}")


class Lineage_Tracker:
    """
    Records and manages transformation lineage.
    
    This class implements lineage tracking for additory operations with:
    - Dual-storage mechanism (pandas attrs vs. Polars global registry)
    - Lazy evaluation for performance
    - Sampling for large datasets
    - Row index compression
    - Memory overhead monitoring
    
    Attributes:
        sampling_threshold: Row count above which sampling is applied (default: 10000)
        max_excluded_rows: Maximum excluded rows to track (default: 1000)
        compression_enabled: Whether to compress row indices (default: True)
    """
    
    def __init__(self):
        """Initialize Lineage_Tracker with default settings."""
        self.sampling_threshold = 10000
        self.max_excluded_rows = 1000
        self.compression_enabled = True

        # Lazy evaluation cache (Requirement 27)
        # Cache stores computed lineage details to avoid recomputation
        # Key: DataFrame ID (for pandas: id(df), for polars: (id, version))
        # Value: Dict with computed details (dependency_graph, traced_cells, etc.)
        self._cache = {}
        self._cache_timestamps = {}  # Track when cache entries were created
    
    def should_sample(self, row_count: int) -> bool:
        """
        Determine if sampling should be used based on row count.
        
        Args:
            row_count: Number of rows in DataFrame
            
        Returns:
            True if sampling should be applied, False otherwise
        """
        return row_count >= self.sampling_threshold
    def _get_cache_key(self, df: Union[pd.DataFrame, pl.DataFrame]) -> str:
        """
        Generate cache key for DataFrame.

        For pandas: uses id(df)
        For Polars: uses (id, version_counter) from global registry

        Args:
            df: DataFrame to generate key for

        Returns:
            Cache key as string
        """
        if isinstance(df, pd.DataFrame):
            return f"pandas_{id(df)}"
        else:
            key = _get_polars_key(df)
            return f"polars_{key[0]}_{key[1]}"

    def _get_cached_details(self, df: Union[pd.DataFrame, pl.DataFrame], detail_type: str) -> Optional[Any]:
        """
        Retrieve cached lineage details for DataFrame.

        Args:
            df: DataFrame to get cached details for
            detail_type: Type of detail to retrieve (e.g., 'dependency_graph', 'traced_cells')

        Returns:
            Cached details if available, None otherwise
        """
        cache_key = self._get_cache_key(df)
        if cache_key in self._cache:
            return self._cache[cache_key].get(detail_type)
        return None

    def _set_cached_details(self, df: Union[pd.DataFrame, pl.DataFrame], detail_type: str, details: Any) -> None:
        """
        Store computed lineage details in cache.

        Args:
            df: DataFrame to cache details for
            detail_type: Type of detail to store
            details: Computed details to cache
        """
        cache_key = self._get_cache_key(df)
        if cache_key not in self._cache:
            self._cache[cache_key] = {}
            self._cache_timestamps[cache_key] = datetime.now()

        self._cache[cache_key][detail_type] = details

    def _invalidate_cache(self, df: Union[pd.DataFrame, pl.DataFrame]) -> None:
        """
        Invalidate cache for DataFrame when new operations are performed.

        This ensures cached details are recomputed after DataFrame changes.

        Args:
            df: DataFrame to invalidate cache for
        """
        cache_key = self._get_cache_key(df)
        if cache_key in self._cache:
            del self._cache[cache_key]
        if cache_key in self._cache_timestamps:
            del self._cache_timestamps[cache_key]
    def calculate_memory_overhead(
        self,
        df: Union[pd.DataFrame, pl.DataFrame],
        lineage: Dict
    ) -> float:
        """
        Calculate memory overhead of lineage metadata.

        Computes the size of lineage metadata relative to DataFrame size
        and returns the overhead as a percentage.

        Args:
            df: DataFrame to calculate overhead for
            lineage: Lineage metadata dictionary

        Returns:
            Memory overhead in MB
        """
        import sys

        # Calculate DataFrame size
        if isinstance(df, pd.DataFrame):
            df_size = df.memory_usage(deep=True).sum()
        else:
            # Polars estimated size
            df_size = df.estimated_size()

        # Calculate lineage metadata size (deep size estimation)
        def get_size(obj, seen=None):
            """Recursively calculate size of object and its contents."""
            if seen is None:
                seen = set()

            obj_id = id(obj)
            if obj_id in seen:
                return 0

            seen.add(obj_id)
            size = sys.getsizeof(obj)

            if isinstance(obj, dict):
                size += sum(get_size(k, seen) + get_size(v, seen) for k, v in obj.items())
            elif isinstance(obj, (list, tuple, set)):
                size += sum(get_size(item, seen) for item in obj)

            return size

        lineage_size = get_size(lineage)

        # Convert to MB
        lineage_mb = lineage_size / (1024 * 1024)

        return round(lineage_mb, 3)

    def check_memory_overhead_warning(
        self,
        df: Union[pd.DataFrame, pl.DataFrame],
        lineage: Dict
    ) -> Optional[str]:
        """
        Check if memory overhead exceeds warning threshold.

        Returns warning message if overhead exceeds 30% of DataFrame size
        AND is at least 1 MB in absolute terms (to avoid false positives
        for small DataFrames).

        Args:
            df: DataFrame to check
            lineage: Lineage metadata dictionary

        Returns:
            Warning message if overhead is high, None otherwise
        """
        import sys

        # Calculate sizes
        if isinstance(df, pd.DataFrame):
            df_size = df.memory_usage(deep=True).sum()
        else:
            df_size = df.estimated_size()

        # Get lineage size from metadata if available
        lineage_mb = lineage['metadata'].get('memory_overhead_mb', 0)
        lineage_size = lineage_mb * 1024 * 1024

        # Calculate overhead percentage
        if df_size > 0:
            overhead_pct = (lineage_size / df_size) * 100
        else:
            overhead_pct = 0

        # Check threshold (30% AND at least 1 MB)
        # This avoids false positives for small DataFrames
        if overhead_pct > 30 and lineage_mb >= 1.0:
            return (
                f"⚠️ Lineage metadata overhead is {overhead_pct:.1f}% "
                f"({lineage_mb:.2f} MB). Consider using sampling or "
                f"reducing tracked operations."
            )

        return None
    
    def sample_rows(self, indices: List[int], max_samples: int) -> List[int]:
        """
        Sample row indices to limit memory overhead.
        
        Uses uniform sampling to select a representative subset of indices.
        
        Args:
            indices: List of row indices to sample
            max_samples: Maximum number of samples to return
            
        Returns:
            Sampled list of row indices
        """
        if len(indices) <= max_samples:
            return indices
        
        # Uniform sampling
        import random
        return sorted(random.sample(indices, max_samples))
    
    def compress_row_indices(self, indices: List[int]) -> List[Tuple[int, int]]:
        """
        Compress list of row indices into ranges.
        
        Converts consecutive indices into (start, end) tuples for memory efficiency.
        Provides ~70% memory savings for consecutive indices.
        
        Args:
            indices: List of row indices
            
        Returns:
            List of (start, end) tuples representing ranges
            
        Example:
            >>> compress_row_indices([1, 2, 3, 5, 7, 8, 9, 10])
            [(1, 3), (5, 5), (7, 10)]
        """
        if not indices:
            return []
        
        if not self.compression_enabled:
            return [(idx, idx) for idx in indices]
        
        sorted_indices = sorted(set(indices))
        ranges = []
        start = sorted_indices[0]
        end = sorted_indices[0]
        
        for idx in sorted_indices[1:]:
            if idx == end + 1:
                end = idx
            else:
                ranges.append((start, end))
                start = idx
                end = idx
        
        ranges.append((start, end))
        return ranges
    
    def record_operation(
        self,
        df: Union[pd.DataFrame, pl.DataFrame],
        operation_type: str,
        params: Dict,
        rows_before: int,
        rows_after: int,
        columns_added: Optional[List[str]] = None,
        columns_modified: Optional[List[str]] = None,
        excluded_rows: Optional[List[Tuple[int, str]]] = None
    ) -> Union[pd.DataFrame, pl.DataFrame]:
        """
        Record operation metadata in DataFrame lineage.
        
        For pandas: stores in DataFrame.attrs['lineage']
        For Polars: stores in global registry with (id, version_counter) key
        
        Uses lazy evaluation - stores minimal info during operation,
        computes details only when scan() is called.
        
        Performance Target (Requirement 27.5):
        Metadata storage completes in <100ms per operation to minimize overhead.
        
        Session-Only Lineage (Requirement 22):
        Lineage metadata is session-scoped and does not persist when DataFrames
        are saved to disk using native methods. When users attempt to save a
        DataFrame with lineage using methods like write_parquet(), to_parquet(),
        or to_csv(), a warning is displayed:
        
        "⚠️ Lineage metadata is session-only and will not be saved. 
         Use add.save() in v0.2.0 for persistent lineage."
        
        This limitation applies to both pandas and Polars DataFrames in v0.1.4.
        Persistent lineage via add.save() and add.load() will be added in v0.2.0.
        
        Args:
            df: DataFrame to attach lineage to
            operation_type: Type of operation ('add.to', 'add.transform', 'add.synthetic')
            params: Operation parameters
            rows_before: Row count before operation
            rows_after: Row count after operation
            columns_added: List of columns added by operation
            columns_modified: List of columns modified by operation
            excluded_rows: List of (row_index, reason) tuples for excluded rows
            
        Returns:
            DataFrame with lineage metadata attached
        """
        import time
        start_time = time.time()
        
        # Invalidate cache since DataFrame is being modified (Requirement 27.4)
        self._invalidate_cache(df)
        
        # Get existing lineage or create new
        lineage = _get_lineage(df)
        
        if lineage is None:
            # Start fresh lineage (Requirement 36)
            lineage = self._start_fresh_lineage(df)
        
        # Create operation record (minimal info for lazy evaluation)
        timestamp = datetime.now().isoformat()
        operation = {
            'operation_type': operation_type,
            'timestamp': timestamp,
            'params': params,
            'rows_before': rows_before,
            'rows_after': rows_after,
            'columns_added': columns_added or [],
            'columns_modified': columns_modified or []
        }
        
        # Add operation to lineage
        lineage['operations'].append(operation)
        
        # Update column sources for added columns
        operation_index = len(lineage['operations']) - 1
        for col in (columns_added or []):
            lineage['column_sources'][col] = {
                'source_type': 'calculated',  # Calculated by this operation
                'source_table': None,
                'source_column': None,
                'formula': None,
                'dependencies': []  # Will be populated by specific operation if needed
            }
        
        # Track excluded rows if provided
        if excluded_rows:
            # Apply sampling if too many excluded rows
            if len(excluded_rows) > self.max_excluded_rows:
                excluded_rows = self.sample_rows(
                    [idx for idx, _ in excluded_rows],
                    self.max_excluded_rows
                )
                excluded_rows = [(idx, f"Sampled exclusion") for idx in excluded_rows]
                lineage['metadata']['sampling_applied'] = True
            
            for row_idx, reason in excluded_rows:
                lineage['excluded_rows'].append({
                    'original_index': row_idx,
                    'excluded_in_operation': operation_index,
                    'reason': reason
                })
        
        # Update metadata
        lineage['metadata']['last_updated'] = timestamp
        lineage['metadata']['total_operations'] = len(lineage['operations'])
        
        # Track timing for performance monitoring
        elapsed_ms = (time.time() - start_time) * 1000
        if 'operation_timings' not in lineage['metadata']:
            lineage['metadata']['operation_timings'] = []
        lineage['metadata']['operation_timings'].append({
            'operation': operation_index,
            'elapsed_ms': round(elapsed_ms, 2)
        })
        
        # Calculate memory overhead periodically (every 10 operations) to reduce overhead
        # This balances monitoring with performance (Requirement 26.1, 26.2, 26.3)
        if operation_index % 10 == 0 or operation_index == 0:
            memory_overhead_mb = self.calculate_memory_overhead(df, lineage)
            lineage['metadata']['memory_overhead_mb'] = memory_overhead_mb
        
        # Store lineage back to DataFrame
        result_df = _set_lineage(df, lineage)
        
        # Check for memory overhead warning only when we calculated it (Requirement 26.5)
        if 'memory_overhead_mb' in lineage['metadata'] and (operation_index % 10 == 0 or operation_index == 0):
            warning = self.check_memory_overhead_warning(result_df, lineage)
            if warning:
                import warnings
                warnings.warn(warning, UserWarning, stacklevel=2)
        
        return result_df
    
    def _start_fresh_lineage(self, df: Union[pd.DataFrame, pl.DataFrame]) -> Dict:
        """
        Initialize lineage metadata for a DataFrame without existing lineage.
        
        This allows users to start tracking lineage at any point in their pipeline,
        treating the current DataFrame as the "original" source (Requirement 36).
        
        Args:
            df: DataFrame to initialize lineage for
            
        Returns:
            New lineage metadata dictionary
        """
        timestamp = datetime.now().isoformat()
        row_count = len(df)
        
        # Get column names
        if isinstance(df, pd.DataFrame):
            columns = df.columns.tolist()
        else:
            columns = df.columns
        
        return {
            'operations': [],
            'column_sources': {
                col: {
                    'source_type': 'original',
                    'source_table': None,
                    'source_column': None,
                    'formula': None,
                    'dependencies': []
                }
                for col in columns
            },
            'row_mapping': {i: [i] for i in range(row_count)},
            'excluded_rows': [],
            'metadata': {
                'version': '0.1.4',
                'created': timestamp,
                'last_updated': timestamp,
                'total_operations': 0,
                'sampling_applied': False,
                'fresh_start': True,  # Indicates lineage started mid-pipeline
                'compression_enabled': False
            }
        }
    
    def update_column_sources_for_to(
        self,
        lineage: Dict,
        operation_index: int,
        brought_columns: List[str],
        source_table: str
    ) -> None:
        """
        Update column sources for add.to() operation.
        
        Marks brought columns as coming from the source table.
        
        Args:
            lineage: Lineage metadata dictionary
            operation_index: Index of the operation
            brought_columns: List of columns brought from source
            source_table: Identifier of the source table
        """
        for col in brought_columns:
            lineage['column_sources'][col] = {
                'source': 'brought',
                'operation': operation_index,
                'source_table': source_table,
                'source_column': col
            }
    
    def update_column_sources_for_calc(
        self,
        lineage: Dict,
        operation_index: int,
        calculated_columns: Dict[str, str],
        available_columns: List[str]
    ) -> None:
        """
        Update column sources for add.transform('@calc') operation.
        
        Marks calculated columns with their formulas and dependencies.
        Uses DependencyTracker to parse formulas and extract dependencies.
        
        Args:
            lineage: Lineage metadata dictionary
            operation_index: Index of the operation
            calculated_columns: Dictionary mapping column names to formulas
            available_columns: List of column names available at time of calculation
        """
        tracker = DependencyTracker()
        
        for col, formula in calculated_columns.items():
            # Parse dependencies from formula
            dependencies = tracker.parse_formula(formula, available_columns)
            
            lineage['column_sources'][col] = {
                'source': 'calculated',
                'operation': operation_index,
                'formula': formula,
                'dependencies': dependencies
            }
    
    def update_column_sources_for_aggregate(
        self,
        lineage: Dict,
        operation_index: int,
        aggregated_columns: Dict[str, str],
        source_columns: Dict[str, Optional[str]]
    ) -> None:
        """
        Update column sources for add.transform('@aggregate') operation.
        
        Marks aggregated columns with their aggregation function and source column.
        
        Args:
            lineage: Lineage metadata dictionary
            operation_index: Index of the operation
            aggregated_columns: Dictionary mapping column names to aggregation functions
            source_columns: Dictionary mapping column names to source columns (None for count)
        """
        for col, agg_func in aggregated_columns.items():
            lineage['column_sources'][col] = {
                'source': 'aggregated',
                'operation': operation_index,
                'aggregation': agg_func,
                'source_column': source_columns.get(col, None)
            }
    
    def update_row_mapping_for_filter(
        self,
        lineage: Dict,
        kept_indices: List[int]
    ) -> None:
        """
        Update row_mapping after filter operation.
        
        When rows are filtered out, update the mapping to reflect new indices.
        
        Args:
            lineage: Lineage metadata dictionary
            kept_indices: List of row indices that passed the filter
        """
        mapper = RowMapper()
        current_mapping = lineage.get('row_mapping', {})
        lineage['row_mapping'] = mapper.update_for_filter(current_mapping, kept_indices)
    
    def update_row_mapping_for_aggregation(
        self,
        lineage: Dict,
        groups: Dict[int, List[int]]
    ) -> None:
        """
        Update row_mapping after aggregation operation.
        
        After aggregation, each result row represents a group of original rows.
        
        Args:
            lineage: Lineage metadata dictionary
            groups: Dictionary mapping new row index to list of pre-aggregation indices
        """
        mapper = RowMapper()
        current_mapping = lineage.get('row_mapping', {})
        lineage['row_mapping'] = mapper.update_for_aggregation(current_mapping, groups)


class OperationRecorder:
    """Records specific operation types with appropriate metadata."""
    
    def record_to_operation(self, params: Dict) -> Dict:
        """
        Record add.to() operation metadata.
        
        Captures:
        - Source DataFrame identifier
        - Columns brought from source
        - Matching key used for join
        - Join type (lookup, left, inner, outer)
        - Number of matched and unmatched rows
        
        Args:
            params: Operation parameters including:
                - bring_from: Source DataFrame identifier
                - bring: List of columns brought
                - against: Matching key
                - join_type: Type of join
                - matched_rows: Number of matched rows
                - unmatched_rows: Number of unmatched rows
            
        Returns:
            Operation metadata dictionary
        """
        metadata = {
            'source_table': params.get('bring_from', 'unknown'),
            'columns_brought': params.get('bring', []),
            'matching_key': params.get('against', None),
            'join_type': params.get('join_type', 'lookup'),
            'matched_rows': params.get('matched_rows', 0),
            'unmatched_rows': params.get('unmatched_rows', 0)
        }
        return metadata
    
    def record_transform_operation(self, mode: str, params: Dict) -> Dict:
        """
        Record add.transform() operation metadata.
        
        Captures mode-specific parameters:
        - @calc: formulas and dependencies
        - @filter: filter condition and excluded row count
        - @aggregate: grouping columns and aggregation functions
        - @sort: sort columns and order
        - @deduce: imputation method and filled value count
        
        Args:
            mode: Transform mode (@calc, @filter, @aggregate, @sort, @deduce, etc.)
            params: Operation parameters specific to the mode
            
        Returns:
            Operation metadata dictionary
        """
        metadata = {'mode': mode}
        
        if mode == '@calc':
            # Record formulas and dependencies
            metadata['strategy'] = params.get('strategy', {})
            # Dependencies will be parsed later by DependencyTracker
            
        elif mode == '@filter':
            # Record filter condition and excluded rows
            metadata['where'] = params.get('where', '')
            metadata['excluded_count'] = params.get('excluded_count', 0)
            
        elif mode == '@aggregate':
            # Record grouping and aggregation
            metadata['by'] = params.get('by', [])
            metadata['strategy'] = params.get('strategy', {})
            
        elif mode == '@sort':
            # Record sort columns and order
            metadata['by'] = params.get('by', [])
            metadata['descending'] = params.get('descending', False)
            
        elif mode == '@deduce':
            # Record imputation method and filled count
            metadata['method'] = params.get('method', 'forward')
            metadata['filled_count'] = params.get('filled_count', 0)
        
        else:
            # For other modes, store all params
            metadata.update(params)
        
        return metadata
    
    def record_synthetic_operation(self, mode: str, params: Dict) -> Dict:
        """
        Record add.synthetic() operation metadata.
        
        Captures:
        - Generation mode (@new, @augment)
        - Generation strategies and row count
        - Which rows are synthetic (for @augment)
        - Random seed if provided
        
        Args:
            mode: Synthetic mode (@new, @augment)
            params: Operation parameters including:
                - strategy: Generation strategies
                - n_rows: Number of rows generated
                - synthetic_rows: List of synthetic row indices (for @augment)
                - seed: Random seed
            
        Returns:
            Operation metadata dictionary
        """
        metadata = {'mode': mode}
        
        if mode == '@new':
            # Record generation strategies and row count
            metadata['strategy'] = params.get('strategy', {})
            metadata['n_rows'] = params.get('n_rows', 0)
            metadata['seed'] = params.get('seed', None)
            
        elif mode == '@augment':
            # Record which rows are synthetic
            metadata['strategy'] = params.get('strategy', {})
            metadata['synthetic_rows'] = params.get('synthetic_rows', [])
            metadata['seed'] = params.get('seed', None)
        
        else:
            # For other modes, store all params
            metadata.update(params)
        
        return metadata


class RowMapper:
    """
    Tracks row index mappings across transformations.
    
    This class maintains the relationship between current row indices and their
    original source row indices. It handles:
    - Identity mapping for initial DataFrames
    - Mapping updates after filter operations (rows removed)
    - Mapping updates after aggregation operations (many-to-one)
    - Tracing current rows back to original source rows
    
    The row_mapping structure is a dictionary where:
    - Key: current row index (int)
    - Value: list of original row indices (List[int])
    
    For most operations, each current row maps to a single original row.
    After aggregation, each current row maps to multiple original rows (the group).
    
    Attributes:
        sampling_threshold: Row count above which sampling is applied (default: 10000)
        max_tracked_rows: Maximum rows to track per group (default: 1000)
    """
    
    def __init__(self):
        """Initialize RowMapper with default settings."""
        self.sampling_threshold = 10000
        self.max_tracked_rows = 1000
    
    def initialize_mapping(self, row_count: int) -> Dict[int, List[int]]:
        """
        Create initial identity mapping for a DataFrame.
        
        Each row maps to itself: row i → [i]
        
        Args:
            row_count: Number of rows in the DataFrame
            
        Returns:
            Dictionary mapping each row index to a list containing itself
            
        Example:
            >>> mapper = RowMapper()
            >>> mapping = mapper.initialize_mapping(3)
            >>> mapping
            {0: [0], 1: [1], 2: [2]}
        """
        return {i: [i] for i in range(row_count)}
    
    def update_for_filter(
        self,
        mapping: Dict[int, List[int]],
        kept_indices: List[int]
    ) -> Dict[int, List[int]]:
        """
        Update mapping after filter operation.
        
        When rows are filtered out, the remaining rows get new indices.
        This method creates a new mapping from new indices to original indices.
        
        Args:
            mapping: Current row mapping (before filter)
            kept_indices: List of row indices that passed the filter
            
        Returns:
            New mapping from filtered row indices to original indices
            
        Example:
            >>> mapper = RowMapper()
            >>> original_mapping = {0: [0], 1: [1], 2: [2], 3: [3], 4: [4]}
            >>> # Keep rows 1, 3, 4 (filter out 0, 2)
            >>> new_mapping = mapper.update_for_filter(original_mapping, [1, 3, 4])
            >>> new_mapping
            {0: [1], 1: [3], 2: [4]}
        """
        new_mapping = {}
        for new_idx, old_idx in enumerate(kept_indices):
            # Preserve the original mapping chain
            new_mapping[new_idx] = mapping[old_idx]
        return new_mapping
    
    def update_for_aggregation(
        self,
        mapping: Dict[int, List[int]],
        groups: Dict[int, List[int]]
    ) -> Dict[int, List[int]]:
        """
        Update mapping after aggregation operation.
        
        After aggregation, each result row represents a group of original rows.
        This method creates a mapping from aggregated row indices to all original
        rows in that group.
        
        Applies sampling if groups are too large (>sampling_threshold rows total).
        
        Args:
            mapping: Current row mapping (before aggregation)
            groups: Dictionary mapping new row index to list of pre-aggregation indices
                   Example: {0: [0, 2, 4], 1: [1, 3, 5]} means:
                   - New row 0 comes from old rows 0, 2, 4
                   - New row 1 comes from old rows 1, 3, 5
            
        Returns:
            New mapping from aggregated row indices to original indices
            
        Example:
            >>> mapper = RowMapper()
            >>> original_mapping = {0: [0], 1: [1], 2: [2], 3: [3], 4: [4], 5: [5]}
            >>> # Group rows: [0,2,4] → row 0, [1,3,5] → row 1
            >>> groups = {0: [0, 2, 4], 1: [1, 3, 5]}
            >>> new_mapping = mapper.update_for_aggregation(original_mapping, groups)
            >>> new_mapping
            {0: [0, 2, 4], 1: [1, 3, 5]}
        """
        new_mapping = {}
        total_rows = sum(len(group) for group in groups.values())
        should_sample = total_rows > self.sampling_threshold
        
        for new_idx, group_indices in groups.items():
            # Collect all original indices for this group
            original_indices = []
            for group_idx in group_indices:
                original_indices.extend(mapping[group_idx])
            
            # Apply sampling if needed
            if should_sample and len(original_indices) > self.max_tracked_rows:
                import random
                original_indices = sorted(random.sample(original_indices, self.max_tracked_rows))
            
            new_mapping[new_idx] = original_indices
        
        return new_mapping
    
    def trace_row_origin(
        self,
        current_index: int,
        mapping: Dict[int, List[int]]
    ) -> List[int]:
        """
        Trace current row back to original index(es).
        
        Returns the list of original row indices that contributed to the current row.
        For most operations, this is a single index.
        For aggregated rows, this is a list of all indices in the group.
        
        Args:
            current_index: Current row index to trace
            mapping: Row mapping dictionary
            
        Returns:
            List of original row indices
            
        Raises:
            KeyError: If current_index is not in mapping
            
        Example:
            >>> mapper = RowMapper()
            >>> mapping = {0: [1], 1: [3], 2: [4]}  # After filter
            >>> mapper.trace_row_origin(0, mapping)
            [1]
            >>> 
            >>> # After aggregation
            >>> agg_mapping = {0: [0, 2, 4], 1: [1, 3, 5]}
            >>> mapper.trace_row_origin(0, agg_mapping)
            [0, 2, 4]
        """
        if current_index not in mapping:
            raise KeyError(
                f"Row index {current_index} not found in mapping. "
                f"Valid indices: {sorted(mapping.keys())}"
            )
        
        return mapping[current_index]



class DependencyTracker:
    """
    Tracks column dependencies in calculated formulas.
    
    This class parses formulas to extract column names and build dependency graphs.
    It handles:
    - Extracting column names from expressions
    - Filtering out Python keywords and built-in functions
    - Filtering out numeric literals
    - Building complete dependency graphs
    - Detecting circular dependencies
    
    Attributes:
        python_keywords: Set of Python keywords to exclude
        builtin_functions: Set of Python built-in functions to exclude
    """
    
    def __init__(self):
        """Initialize DependencyTracker with keyword and function lists."""
        # Python keywords to exclude
        self.python_keywords = {
            'False', 'None', 'True', 'and', 'as', 'assert', 'async', 'await',
            'break', 'class', 'continue', 'def', 'del', 'elif', 'else', 'except',
            'finally', 'for', 'from', 'global', 'if', 'import', 'in', 'is',
            'lambda', 'nonlocal', 'not', 'or', 'pass', 'raise', 'return',
            'try', 'while', 'with', 'yield'
        }
        
        # Common built-in functions to exclude
        self.builtin_functions = {
            'abs', 'all', 'any', 'ascii', 'bin', 'bool', 'bytearray', 'bytes',
            'callable', 'chr', 'classmethod', 'compile', 'complex', 'delattr',
            'dict', 'dir', 'divmod', 'enumerate', 'eval', 'exec', 'filter',
            'float', 'format', 'frozenset', 'getattr', 'globals', 'hasattr',
            'hash', 'help', 'hex', 'id', 'input', 'int', 'isinstance',
            'issubclass', 'iter', 'len', 'list', 'locals', 'map', 'max',
            'memoryview', 'min', 'next', 'object', 'oct', 'open', 'ord',
            'pow', 'print', 'property', 'range', 'repr', 'reversed', 'round',
            'set', 'setattr', 'slice', 'sorted', 'staticmethod', 'str', 'sum',
            'super', 'tuple', 'type', 'vars', 'zip',
            # Math functions commonly used in formulas
            'sqrt', 'exp', 'log', 'log10', 'sin', 'cos', 'tan', 'floor', 'ceil'
        }
    
    def parse_formula(self, formula: str, available_columns: List[str]) -> List[str]:
        """
        Extract column names from a formula expression.
        
        Parses the formula to identify column references, filtering out:
        - Python keywords (if, else, and, or, etc.)
        - Built-in functions (sum, max, min, etc.)
        - Numeric literals (123, 45.67, etc.)
        
        Args:
            formula: Formula expression (e.g., "price * quantity + discount")
            available_columns: List of column names that exist in the DataFrame
            
        Returns:
            List of column names that the formula depends on
            
        Example:
            >>> tracker = DependencyTracker()
            >>> columns = ['price', 'quantity', 'discount', 'tax']
            >>> tracker.parse_formula("price * quantity + discount", columns)
            ['price', 'quantity', 'discount']
            >>> tracker.parse_formula("price * 1.1 + 100", columns)
            ['price']
        """
        import re
        
        # Extract potential identifiers (alphanumeric + underscore)
        # This regex matches Python identifiers
        identifier_pattern = r'\b[a-zA-Z_][a-zA-Z0-9_]*\b'
        potential_identifiers = re.findall(identifier_pattern, formula)
        
        dependencies = []
        
        for identifier in potential_identifiers:
            # Skip if it's a Python keyword
            if identifier in self.python_keywords:
                continue
            
            # Skip if it's a built-in function
            if identifier in self.builtin_functions:
                continue
            
            # Check if it's a column name
            if identifier in available_columns:
                if identifier not in dependencies:
                    dependencies.append(identifier)
        
        return dependencies
    
    def build_dependency_graph(
        self,
        column_sources: Dict[str, Dict]
    ) -> Dict[str, List[str]]:
        """
        Build complete dependency graph from column sources.
        
        Creates a graph showing which columns depend on which other columns.
        
        Args:
            column_sources: Dictionary mapping column names to their source metadata
            
        Returns:
            Dictionary mapping each column to its direct dependencies
            
        Example:
            >>> column_sources = {
            ...     'a': {'source': 'original'},
            ...     'b': {'source': 'original'},
            ...     'c': {'source': 'calculated', 'dependencies': ['a', 'b']},
            ...     'd': {'source': 'calculated', 'dependencies': ['c']}
            ... }
            >>> tracker = DependencyTracker()
            >>> graph = tracker.build_dependency_graph(column_sources)
            >>> graph
            {'a': [], 'b': [], 'c': ['a', 'b'], 'd': ['c']}
        """
        graph = {}
        
        for col_name, col_info in column_sources.items():
            dependencies = col_info.get('dependencies', [])
            graph[col_name] = dependencies
        
        return graph
    
    def trace_dependencies(
        self,
        column: str,
        graph: Dict[str, List[str]],
        visited: Optional[set] = None
    ) -> List[str]:
        """
        Recursively trace dependencies to find all source columns.
        
        Traces a column back through all its dependencies to find the original
        source columns (columns with no dependencies).
        
        Args:
            column: Column name to trace
            graph: Dependency graph from build_dependency_graph()
            visited: Set of already visited columns (for circular detection)
            
        Returns:
            List of source column names (columns with no dependencies)
            
        Raises:
            ValueError: If circular dependency is detected
            
        Example:
            >>> graph = {
            ...     'a': [],
            ...     'b': [],
            ...     'c': ['a', 'b'],
            ...     'd': ['c'],
            ...     'e': ['d', 'b']
            ... }
            >>> tracker = DependencyTracker()
            >>> tracker.trace_dependencies('e', graph)
            ['a', 'b']
        """
        if visited is None:
            visited = set()
        
        # Check for circular dependency
        if column in visited:
            raise ValueError(
                f"Circular dependency detected: {column} depends on itself "
                f"through the chain: {' → '.join(visited)} → {column}"
            )
        
        # Mark as visited
        visited.add(column)
        
        # Get direct dependencies
        if column not in graph:
            # Column not in graph - treat as source
            return [column]
        
        dependencies = graph[column]
        
        # If no dependencies, this is a source column
        if not dependencies:
            return [column]
        
        # Recursively trace dependencies
        sources = []
        for dep in dependencies:
            dep_sources = self.trace_dependencies(dep, graph, visited.copy())
            for source in dep_sources:
                if source not in sources:
                    sources.append(source)
        
        return sources
    
    def detect_circular_dependencies(
        self,
        graph: Dict[str, List[str]]
    ) -> List[List[str]]:
        """
        Detect circular dependencies in the dependency graph.
        
        Identifies any cycles in the dependency graph where a column depends
        on itself through a chain of other columns.
        
        Args:
            graph: Dependency graph from build_dependency_graph()
            
        Returns:
            List of circular dependency chains (empty if no cycles)
            Each chain is a list of column names forming a cycle
            
        Example:
            >>> # No circular dependencies
            >>> graph = {'a': [], 'b': ['a'], 'c': ['b']}
            >>> tracker = DependencyTracker()
            >>> tracker.detect_circular_dependencies(graph)
            []
            
            >>> # Circular dependency: c → d → c
            >>> graph = {'a': [], 'b': ['a'], 'c': ['d'], 'd': ['c']}
            >>> tracker.detect_circular_dependencies(graph)
            [['c', 'd', 'c']]
        """
        cycles = []
        
        def visit(node: str, path: List[str], visited: set):
            """DFS to detect cycles."""
            if node in path:
                # Found a cycle
                cycle_start = path.index(node)
                cycle = path[cycle_start:] + [node]
                cycles.append(cycle)
                return
            
            if node in visited:
                return
            
            visited.add(node)
            path.append(node)
            
            # Visit dependencies
            for dep in graph.get(node, []):
                visit(dep, path.copy(), visited)
        
        # Check each column
        for column in graph:
            visit(column, [], set())
        
        return cycles
