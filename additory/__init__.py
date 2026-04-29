"""
Additory - Data augmentation and transformation library

Public API:
- to() - Data operations (lookup, merge, sort, summarize)
- transform() - Transform data within DataFrame
- synthetic() - Synthetic data generation
- scan() - Inspect, analyze, and explain DataFrames

Example:
    >>> import additory as add
    >>> import polars as pl
    >>> 
    >>> df = pl.DataFrame({'weight': [70, 80], 'height': [1.75, 1.80]})
    >>> 
    >>> # Use builtin expression
    >>> result = add.transform('@calc', df, expression='inbuilt:bmi', as='bmi')
    >>> 
    >>> # Use @knn imputation
    >>> result = add.transform('@knn', df, fetch=['age'], strategy={'k': 5})
    >>> 
    >>> # Scan DataFrame
    >>> result = add.scan('@analyze', df)
"""

import builtins

import re
import sys
import os
from pathlib import Path

# Type imports for type hints
from typing import Union, List, Optional, Dict, Any, Literal

# NEW: Lineage imports
from .lineage_tracker import (
    Lineage_Tracker,
    _get_lineage,
    _set_lineage
)

# Logging setup
import logging
logger = logging.getLogger(__name__)


def _validate_lineage_as_type_exclusion(lineage: bool, as_type: Optional[str], function_name: str):
    """
    Validate that lineage and as_type are not used together.

    Args:
        lineage: Whether lineage tracking is enabled
        as_type: Output type conversion ('pandas', 'polars', or None)
        function_name: Name of calling function (for error message)

    Raises:
        ValueError: If both lineage=True and as_type is not None
    """
    if lineage and as_type is not None:
        logger.warning(
            f"{function_name}(): Validation failed - "
            f"lineage=True and as_type='{as_type}' cannot be used together"
        )
        raise ValueError(
            f"Cannot use 'as_type' with 'lineage=True' in {function_name}(). "
            f"Lineage metadata is stored in the DataFrame's native format "
            f"and would be lost during type conversion.\n"
            f"\n"
            f"To track lineage, omit 'as_type' parameter:\n"
            f"  result = {function_name}(..., lineage=True)  # Returns same type as input\n"
            f"\n"
            f"To convert type without lineage:\n"
            f"  result = {function_name}(..., as_type='{as_type}')  # No lineage tracking\n"
            f"\n"
            f"To convert after tracking lineage:\n"
            f"  result = {function_name}(..., lineage=True)  # Track lineage\n"
            f"  result_converted = pl.from_pandas(result)  # Convert separately (lineage lost)"
        )


def _get_added_columns(df_before, df_after):
    """
    Get list of columns added in operation.

    Args:
        df_before: DataFrame before operation (or None)
        df_after: DataFrame after operation

    Returns:
        List of column names added
    """
    if df_before is None:
        # All columns are new
        return list(df_after.columns)

    before_cols = builtins.set(df_before.columns)
    after_cols = builtins.set(df_after.columns)
    return list(after_cols - before_cols)


def _get_modified_columns(df_before, df_after):
    """
    Get list of columns modified in operation.
    
    Note: This function cannot reliably detect column modifications without
    comparing actual values, which would be expensive. Column modifications
    should be tracked at the operation level (e.g., in transform() based on
    the mode and strategy parameters).

    Args:
        df_before: DataFrame before operation (or None)
        df_after: DataFrame after operation

    Returns:
        Empty list (modifications should be tracked by the calling function)
    """
    # Column modifications cannot be reliably detected without value comparison.
    # The calling function should track modifications based on operation context.
    return []


def _get_excluded_rows(df_before, df_after):
    """
    Get list of (row_index, reason) for excluded rows.
    
    Note: This function provides a simplified implementation that assumes
    rows were removed from the end. For accurate tracking, the calling
    function should provide the actual excluded row indices based on the
    filter condition.

    Args:
        df_before: DataFrame before operation
        df_after: DataFrame after operation

    Returns:
        List of (row_index, reason) tuples for excluded rows, or None if no rows excluded
    """
    if df_before is None or len(df_before) == len(df_after):
        return None

    # Simplified: Assume rows were removed from the end
    # For accurate tracking, the calling function should provide actual indices
    excluded_count = len(df_before) - len(df_after)
    excluded_indices = list(range(len(df_after), len(df_before)))

    return [(idx, "Filtered out") for idx in excluded_indices[:1000]]  # Limit to 1000



# Add parent directory to path to import from python-specific
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Try to import Rust bindings
try:
    from additory import _additory
    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False
    _additory = None

# Import Python features
from .games import games as games_module
from .scan import scan as _rust_scan
from .expressions.loader import get_registry, Expression, RESERVED_NAMES

# Version
__version__ = "0.1.3a11"

# Convenience wrappers for scan modes
def analyze(
    df: Union['pd.DataFrame', 'pl.DataFrame'],
    *,
    columns: Optional[Union[str, List[str]]] = None,
    where: Optional[str] = None,
    rows: Optional[Union[str, List[str]]] = None,
    focus: Optional[str] = None,
    as_type: Optional[Literal['pandas', 'polars', 'dict', 'text']] = None
) -> Union['pd.DataFrame', 'pl.DataFrame', Dict, str]:
    """
    Analyze DataFrame - Statistical profiling (American spelling).
    
    This is a convenience wrapper for add.scan('@analyze', df, ...).
    Provides statistical profiling including data types, null counts,
    distributions, and data quality metrics.
    
    Args:
        df: Input DataFrame (pandas or polars)
        columns: Column filter (string or list of strings)
        where: SQL-like filter condition (string)
        rows: Row range specifications (string or list of strings)
        focus: Specialized analysis mode (string)
        as_type: Output format ('pandas', 'polars', 'dict', or 'text')
        
    Returns:
        Analysis results in specified format (DataFrame, dict, or text)
        
    Examples:
        >>> # Basic analysis
        >>> result = add.analyze(df)
        
        >>> # Analyze specific columns
        >>> result = add.analyze(df, columns=['age', 'salary'])
        
        >>> # Filter rows before analysis
        >>> result = add.analyze(df, where='age > 18')
        
        >>> # Get text output
        >>> report = add.analyze(df, as_type='text')
    """
    return scan('@analyze', df, columns=columns, where=where, rows=rows, 
                focus=focus, as_type=as_type)


def analyse(
    df: Union['pd.DataFrame', 'pl.DataFrame'],
    *,
    columns: Optional[Union[str, List[str]]] = None,
    where: Optional[str] = None,
    rows: Optional[Union[str, List[str]]] = None,
    focus: Optional[str] = None,
    as_type: Optional[Literal['pandas', 'polars', 'dict', 'text']] = None
) -> Union['pd.DataFrame', 'pl.DataFrame', Dict, str]:
    """
    Analyse DataFrame - Statistical profiling (British spelling).
    
    This is a convenience wrapper for add.scan('@analyse', df, ...).
    Provides statistical profiling including data types, null counts,
    distributions, and data quality metrics.
    
    Args:
        df: Input DataFrame (pandas or polars)
        columns: Column filter (string or list of strings)
        where: SQL-like filter condition (string)
        rows: Row range specifications (string or list of strings)
        focus: Specialized analysis mode (string)
        as_type: Output format ('pandas', 'polars', 'dict', or 'text')
        
    Returns:
        Analysis results in specified format (DataFrame, dict, or text)
        
    Examples:
        >>> # Basic analysis
        >>> result = add.analyse(df)
        
        >>> # Analyse specific columns
        >>> result = add.analyse(df, columns=['age', 'salary'])
        
        >>> # Filter rows before analysis
        >>> result = add.analyse(df, where='age > 18')
        
        >>> # Get text output
        >>> report = add.analyse(df, as_type='text')
    """
    return scan('@analyse', df, columns=columns, where=where, rows=rows, 
                focus=focus, as_type=as_type)

def to(
    bring_to: Union['pd.DataFrame', 'pl.DataFrame', List[Union['pd.DataFrame', 'pl.DataFrame']]],
    bring_from: Union['pd.DataFrame', 'pl.DataFrame', List[Union['pd.DataFrame', 'pl.DataFrame']]],
    bring: Union[str, List[str]],
    against: Union[str, List[str]],
    position: Optional[Union[str, int]] = None,
    *,
    strategy: Optional[Dict[str, Union[str, Dict[str, Any]]]] = None,
    join_type: str = 'lookup',
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None
) -> Union['pd.DataFrame', 'pl.DataFrame', List[Union['pd.DataFrame', 'pl.DataFrame']]]:
    """
    Add data from external source(s) to target DataFrame(s).
    
    Modes:
    - LOOKUP (default): Add columns from reference by joining on key
    - @new: Create new DataFrame from reference
    - @merge: Merge multiple DataFrames
    
    Args:
        bring_to: Target DataFrame(s) to bring columns to - single DataFrame or list of DataFrames
        bring_from: Reference DataFrame(s) to bring columns from - single DataFrame or list of DataFrames
        bring: Column(s) to bring from reference (string or list of strings)
        against: Key column(s) to match against (string or list of strings)
        position: Where to place new columns ('start', 'end', 'after:col', 'before:col', or int)
        strategy: Column-level aggregation strategies (dict)
        join_type: Type of join ('lookup', 'left', 'inner', 'outer')
        logging: Enable detailed logging
        lineage: Enable lineage tracking (default: False)
        as_type: Force output type ('pandas' or 'polars')
        
    Returns:
        DataFrame or List[DataFrame]: 
            - If bring_to is single: returns single DataFrame
            - If bring_to is list: returns list of DataFrames
        
    Patterns:
        - Single → Single: Original behavior (backward compatible)
        - Single → List: One-to-many (single target, multiple references)
        - List → Single: Many-to-one (multiple targets, single reference)
        - List → List: Many-to-many (multiple targets, multiple references)
        
    Raises:
        ImportError: If Rust bindings are not available
        ValueError: If required parameters are missing or validation fails
        TypeError: If parameter types are invalid
        RuntimeError: If operation fails
        
    Migration from v0.1.3a8 and earlier:
        - target_df → bring_to (first positional parameter)
        - fetch_from → bring_from (second positional parameter)
        - fetch → bring (third positional parameter)
        - by → against (fourth positional parameter, no longer an alias)
        - Use lists instead of tuples for multiple values
        
    Examples:
        >>> # Single DataFrame
        >>> result = add.to(df, ref_df, 'age', 'id')
        
        >>> # Multiple columns
        >>> result = add.to(df, ref_df, ['age', 'salary'], 'id')
        
        >>> # Multiple keys (using list)
        >>> result = add.to(df, ref_df, 'amount', ['customer_id', 'date'])
        
        >>> # One-to-many: Single target, multiple references
        >>> result = add.to(customers, [orders_jan, orders_feb], 
        ...                 ['amount'], 'customer_id', 
        ...                 strategy={'amount': 'sum'})
        
        >>> # Many-to-one: Multiple targets, single reference
        >>> results = add.to([customers_internal, customers_dept], 
        ...                  orders, ['amount'], 'customer_id')
        
        >>> # With position
        >>> result = add.to(df, ref_df, ['age', 'salary'], 'id', position='after:name')
        
        >>> # With lineage tracking
        >>> result = add.to(df, ref_df, 'age', 'id', lineage=True)
        >>> lineage_report = add.scan('@lineage', result)
    """
    import polars as pl
    import os
    import time
    
    # Log call entry point when logging is enabled
    if logging:
        logger.info(
            "add.to() called — bring=%s, against=%s, join_type=%s",
            bring, against, join_type,
        )
    
    # Check if timing is enabled
    TIMING_ENABLED = os.getenv('ADDITORY_TIMING', 'false').lower() == 'true'
    
    # Start timing
    t_start = time.perf_counter() if TIMING_ENABLED else None
    timings = {} if TIMING_ENABLED else None
    
    try:
        import pandas as pd
        HAS_PANDAS = True
    except ImportError:
        HAS_PANDAS = False
        pd = None
    
    if not RUST_AVAILABLE:
        raise ImportError(
            "add.to() requires Rust bindings. "
            "Install with: pip install additory[rust]"
        )
    
    # Convert tuples to lists for all list parameters
    if isinstance(bring, tuple):
        bring = list(bring)
    if isinstance(against, tuple):
        against = list(against)
    
    # Convert strings to lists (Rust expects lists)
    if isinstance(bring, str):
        bring = [bring]
    if isinstance(against, str):
        against = [against]
    
    # Validate mutual exclusion (lineage + as_type)
    _validate_lineage_as_type_exclusion(lineage, as_type, 'add.to')
    
    # Helper function to validate DataFrame input
    def _validate_dataframe_input(df_input, param_name):
        """Validate DataFrame input (single or list)."""
        if df_input is None:
            raise ValueError(f"{param_name} parameter is required for add.to()")
        
        if isinstance(df_input, list):
            if len(df_input) == 0:
                raise ValueError(f"{param_name} cannot be an empty list")
            
            for i, df in enumerate(df_input):
                is_pandas = isinstance(df, pd.DataFrame) if HAS_PANDAS and pd is not None else False
                is_polars = isinstance(df, pl.DataFrame)
                
                if not is_pandas and not is_polars:
                    raise TypeError(
                        f"{param_name}[{i}] must be pandas or polars DataFrame, "
                        f"got {type(df).__name__}"
                    )
        else:
            is_pandas = isinstance(df_input, pd.DataFrame) if HAS_PANDAS and pd is not None else False
            is_polars = isinstance(df_input, pl.DataFrame)
            
            if not is_pandas and not is_polars:
                raise TypeError(
                    f"{param_name} must be pandas or polars DataFrame, "
                    f"got {type(df_input).__name__}"
                )
    
    # Validate required parameters
    _validate_dataframe_input(bring_to, 'bring_to')
    _validate_dataframe_input(bring_from, 'bring_from')
    
    # Helper function to detect DataFrame types
    def _detect_types(df_input):
        """Detect types of DataFrame input."""
        if isinstance(df_input, list):
            types = []
            for df in df_input:
                if HAS_PANDAS and pd is not None and isinstance(df, pd.DataFrame):
                    types.append('pandas')
                elif isinstance(df, pl.DataFrame):
                    types.append('polars')
            return types
        else:
            if HAS_PANDAS and pd is not None and isinstance(df_input, pd.DataFrame):
                return ['pandas']
            elif isinstance(df_input, pl.DataFrame):
                return ['polars']
            return []
    
    # Helper function to concatenate reference DataFrames
    def _concatenate_references(ref_dfs, ref_types):
        """Concatenate reference DataFrames vertically."""
        if len(ref_dfs) == 1:
            return ref_dfs[0]
        
        # Check type consistency for references
        unique_types = {}
        for t in ref_types:
            unique_types[t] = True
        
        if len(unique_types) > 1:
            type_info = {}
            for i, t in enumerate(ref_types):
                if t not in type_info:
                    type_info[t] = []
                type_info[t].append(i)
            
            error_parts = []
            for t, indices in type_info.items():
                error_parts.append(f"{t.capitalize()} (DataFrame {', '.join(map(str, indices))})")
            
            raise ValueError(
                f"All reference DataFrames must be the same type (all Pandas or all Polars). "
                f"Found: {', '.join(error_parts)}"
            )
        
        # Concatenate based on type
        if ref_types[0] == 'polars':
            return pl.concat(ref_dfs, how='vertical')
        else:  # pandas
            return pd.concat(ref_dfs, axis=0, ignore_index=True)
    
    # Normalize inputs to lists
    target_is_list = isinstance(bring_to, list)
    reference_is_list = isinstance(bring_from, list)
    
    targets = bring_to if target_is_list else [bring_to]
    references = bring_from if reference_is_list else [bring_from]
    
    # Detect types
    target_types = _detect_types(bring_to)
    reference_types = _detect_types(bring_from)
    
    # Concatenate references if multiple
    concatenated_reference = _concatenate_references(references, reference_types)
    
    # Determine reference type for conversion
    ref_is_pandas = reference_types[0] == 'pandas' if reference_types else False
    ref_is_polars = reference_types[0] == 'polars' if reference_types else False
    
    # Convert concatenated reference to Polars for processing
    t_convert_start = time.perf_counter() if TIMING_ENABLED else None
    fetch_from_pl = pl.from_pandas(concatenated_reference) if ref_is_pandas else concatenated_reference
    
    # Convert reference to Arrow IPC bytes (once for all targets)
    import io
    ref_buffer = io.BytesIO()
    fetch_from_pl.write_ipc(ref_buffer)
    ref_bytes = ref_buffer.getvalue()
    
    if TIMING_ENABLED:
        timings['arrow_encode_ref'] = (time.perf_counter() - t_convert_start) * 1000
    
    # Process each target DataFrame
    results = []
    
    for i, target in enumerate(targets):
        # Detect target type
        if target is not None:
            target_is_pandas = isinstance(target, pd.DataFrame) if HAS_PANDAS and pd is not None else False
            target_is_polars = isinstance(target, pl.DataFrame)
            
            # Validate target type
            if not target_is_pandas and not target_is_polars:
                raise TypeError(
                    f"target_df[{i}] must be pandas or polars DataFrame, "
                    f"got {type(target).__name__}"
                )
            
            # Convert to polars if needed
            target_pl = pl.from_pandas(target) if target_is_pandas else target
        else:
            target_pl = None
            target_is_pandas = False
            target_is_polars = False
        
        # Convert target to Arrow IPC bytes
        if target_pl is not None:
            t_target_encode = time.perf_counter() if TIMING_ENABLED else None
            target_buffer = io.BytesIO()
            target_pl.write_ipc(target_buffer)
            target_bytes = target_buffer.getvalue()
            if TIMING_ENABLED:
                timings['arrow_encode_target'] = (time.perf_counter() - t_target_encode) * 1000
        else:
            target_bytes = None
        
        # Prepare parameters
        params = {
            'fetch_from': ref_bytes,
            'fetch': bring,  # Python uses 'bring', Rust expects 'fetch'
            'against': against,  # Python uses 'against', Rust expects 'against'
            'strategy': strategy,
            'rename': None,  # Not exposed in new signature
            'position': position,
            'join_type': join_type,
            'as_type': as_type,
            'logging': logging,
        }
        
        # Call Rust to function
        try:
            t_rust_start = time.perf_counter() if TIMING_ENABLED else None
            result_bytes = _additory.to(target_bytes, params)
            if TIMING_ENABLED:
                timings['rust_operation'] = (time.perf_counter() - t_rust_start) * 1000
        except Exception as e:
            if target_is_list:
                raise RuntimeError(f"add.to() failed for target DataFrame {i}: {e}") from e
            else:
                raise RuntimeError(f"add.to() failed: {e}") from e
        
        # Convert back to DataFrame
        t_decode_start = time.perf_counter() if TIMING_ENABLED else None
        result_buffer = io.BytesIO(result_bytes)
        result_df = pl.read_ipc(result_buffer)
        if TIMING_ENABLED:
            timings['arrow_decode'] = (time.perf_counter() - t_decode_start) * 1000
        
        # Handle as_type parameter for output format (BEFORE lineage tracking)
        if as_type == 'pandas':
            # Force pandas output
            result_df = result_df.to_pandas()
        elif as_type == 'polars':
            # Force polars output
            pass  # Already polars
        else:
            # Default: match input type
            if target_is_pandas:
                result_df = result_df.to_pandas()
        
        # NEW: Lineage tracking (AFTER type conversion)
        if lineage:
            # Initialize tracker
            tracker = Lineage_Tracker()
            
            # Copy existing lineage from input (if single DataFrame)
            if not target_is_list and target is not None:
                existing_lineage = _get_lineage(target)
                if existing_lineage:
                    result_df = _set_lineage(result_df, existing_lineage)
            
            # Record this operation
            result_df = tracker.record_operation(
                df=result_df,
                operation_type='add.to',
                params={
                    'bring': bring,
                    'against': against,
                    'strategy': strategy,
                    'join_type': join_type,
                    'position': position
                },
                rows_before=len(target) if target is not None else 0,
                rows_after=len(result_df),
                columns_added=bring if isinstance(bring, list) else [bring],
                columns_modified=None
            )
            
            # Update column sources
            lineage_data = _get_lineage(result_df)
            if lineage_data:
                tracker.update_column_sources_for_to(
                    lineage=lineage_data,
                    operation_index=len(lineage_data['operations']) - 1,
                    brought_columns=bring if isinstance(bring, list) else [bring],
                    source_table='bring_from'
                )
        
        results.append(result_df)
    
    # Print timing breakdown if enabled
    if TIMING_ENABLED:
        timings['total'] = (time.perf_counter() - t_start) * 1000
        print(f"\n{'='*70}")
        print(f"ADDITORY TIMING BREAKDOWN (add.to)")
        print(f"{'='*70}")
        print(f"  Arrow encode (ref):    {timings.get('arrow_encode_ref', 0):>8.2f} ms")
        print(f"  Arrow encode (target): {timings.get('arrow_encode_target', 0):>8.2f} ms")
        print(f"  Rust operation:        {timings.get('rust_operation', 0):>8.2f} ms  ← Main operation")
        print(f"  Arrow decode:          {timings.get('arrow_decode', 0):>8.2f} ms")
        print(f"  ─────────────────────────────────")
        print(f"  Total:                 {timings['total']:>8.2f} ms")
        
        # Calculate percentages
        total = timings['total']
        rust_pct = (timings.get('rust_operation', 0) / total * 100) if total > 0 else 0
        arrow_pct = ((timings.get('arrow_encode_ref', 0) + timings.get('arrow_encode_target', 0) + timings.get('arrow_decode', 0)) / total * 100) if total > 0 else 0
        
        print(f"\n  Rust operation: {rust_pct:.1f}% of total time")
        print(f"  Arrow IPC:      {arrow_pct:.1f}% of total time")
        print(f"{'='*70}\n")
    
    # Return single DataFrame or list based on input
    if target_is_list:
        return results
    else:
        return results[0]


def _auto_detect_method(df_pl, infer_col: str, against: Optional[List[str]]) -> str:
    """
    Auto-detect imputation method for @deduce mode.
    
    Args:
        df_pl: Polars DataFrame
        infer_col: Column to infer values for
        against: Text columns for TF-IDF (if provided)
    
    Returns:
        str: Detected method ('tfidf' or error)
    
    Raises:
        ValueError: If method cannot be auto-detected
    """
    import polars as pl
    
    # If against is provided, use TF-IDF
    if against is not None and len(against) > 0:
        return 'tfidf'
    
    # Check if column is numeric
    col_dtype = df_pl[infer_col].dtype
    if col_dtype in [pl.Int8, pl.Int16, pl.Int32, pl.Int64, pl.UInt8, pl.UInt16, pl.UInt32, pl.UInt64, pl.Float32, pl.Float64]:
        raise ValueError(
            f"Cannot auto-detect method for numeric column '{infer_col}'.\n"
            f"Please specify method explicitly:\n"
            f"  - 'mean': Fill with column mean\n"
            f"  - 'median': Fill with column median\n"
            f"  - 'mode': Fill with most common value\n"
            f"  - 'forward': Forward fill (use previous value)\n"
            f"  - 'backward': Backward fill (use next value)\n"
            f"  - 'interpolate': Linear interpolation\n"
            f"  - 'knn': K-nearest neighbors imputation\n"
            f"\n"
            f"Example:\n"
            f"  add.transform('@deduce', df, infer='{infer_col}', method='mean')"
        )
    
    # Text/categorical column without against parameter
    raise ValueError(
        f"Text/categorical column '{infer_col}' requires 'against' parameter for TF-IDF deduction.\n"
        f"Specify text column(s) to use for similarity calculation.\n"
        f"\n"
        f"Example:\n"
        f"  add.transform('@deduce', df, infer='{infer_col}', against='description')"
    )


def _auto_generate_name(df_pl, infer_col: str) -> str:
    """
    Auto-generate output column name for @deduce mode.
    
    Args:
        df_pl: Polars DataFrame
        infer_col: Column being inferred
    
    Returns:
        str: Generated column name
    
    Raises:
        ValueError: If too many conflicts (99+)
    """
    base_name = f"{infer_col}_infer"
    
    # Check if base name is available
    if base_name not in df_pl.columns:
        return base_name
    
    # Try numbered suffixes
    for i in range(1, 100):
        candidate = f"{base_name}_{i:02d}"
        if candidate not in df_pl.columns:
            return candidate
    
    # Too many conflicts
    raise ValueError(
        f"Cannot auto-generate name for column '{infer_col}'.\n"
        f"Base name '{base_name}' and 99 numbered variants already exist.\n"
        f"Please specify 'name' parameter explicitly."
    )


def transform(
    mode: str,
    df: Union['pd.DataFrame', 'pl.DataFrame'],
    columns: Optional[Union[str, List[str]]] = None,
    *,
    expression: Optional[Union[str, List[str]]] = None,
    where: Optional[str] = None,
    by: Optional[Union[str, List[str]]] = None,
    name: Optional[Union[str, List[str]]] = None,
    order: Optional[str] = None,
    position: Union[str, int] = 'end',
    strategy: Optional[Union[str, Dict[str, Any]]] = None,
    infer: Optional[Union[str, List[str]]] = None,
    against: Optional[Union[str, List[str]]] = None,
    method: Optional[Union[str, List[str]]] = None,
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None
) -> Union['pd.DataFrame', 'pl.DataFrame']:
    """
    Transform data within DataFrame.
    
    Modes:
    - @filter, @sort, @transpose, @aggregate, @split
    - @calc, @extract, @onehot, @label, @harmonize
    - @deduce (imputation and deduction)
    - @knn (Python-only)
    
    Args:
        mode: Transform mode (e.g., '@calc', '@filter', '@deduce', '@knn')
        df: Input DataFrame (polars or pandas)
        columns: Column(s) to transform/select (string or list of strings)
        expression: Expression(s) to apply (for @calc mode, string or list of strings)
        where: Filter condition (string)
        by: Grouping/sorting/separator column(s) (string or list of strings)
        name: New name(s) for calculated columns (string or list of strings)
        order: Sort order for @sort mode ('asc' or 'desc')
        position: Position for new columns ('start', 'end', 'after:col', 'before:col', or int)
        strategy: Advanced options (dict)
        infer: Column(s) to fill with imputed/deduced values (for @deduce mode, string or list of strings)
        against: Text column(s) for TF-IDF similarity (for @deduce mode, string or list of strings)
        method: Imputation method(s) - 'mean', 'median', 'mode', 'forward', 'backward', 'interpolate', 'knn', 'tfidf' (string or list of strings)
        logging: Enable detailed logging
        lineage: Enable lineage tracking (default: False)
        as_type: Force output type ('pandas' or 'polars')
        
    Returns:
        DataFrame: Transformed DataFrame (same type as input)
        
    Raises:
        ImportError: If Rust bindings are not available
        ValueError: If parameters are invalid
        TypeError: If parameter types are invalid
        RuntimeError: If transformation fails
        
    Migration from v0.1.3a8 and earlier:
        - fetch → columns (use columns parameter)
        - fetch_at → position (use position parameter)
        - Use lists instead of tuples for multiple values
        
    Examples:
        >>> # Filter rows
        >>> result = add.transform('@filter', df, where='age > 18')
        
        >>> # Calculate with single expression
        >>> result = add.transform('@calc', df, expression='inbuilt:bmi', name='bmi')
        
        >>> # Calculate with multiple expressions
        >>> result = add.transform('@calc', df,
        ...     expression=['inbuilt:bmi', 'price * quantity'],
        ...     name=['bmi', 'total'])
        
        >>> # Deduce with auto-detection (text-based)
        >>> result = add.transform('@deduce', df, infer='status', against='comments')
        
        >>> # Deduce with explicit method (numeric)
        >>> result = add.transform('@deduce', df, infer='price', method='mean', name='price_filled')
        
        >>> # Deduce with KNN
        >>> result = add.transform('@deduce', df, infer='age', method='knn', strategy={'k': 10})
        
        >>> # KNN imputation
        >>> result = add.transform('@knn', df, columns=['age'], strategy={'k': 5})
        
        >>> # Sort by column
        >>> result = add.transform('@sort', df, by='date', order='desc')
        
        >>> # With position
        >>> result = add.transform('@calc', df, expression='weight * 2.2', 
        ...                        name='weight_lbs', position='after:weight')
    """
    import polars as pl
    try:
        import pandas as pd
        HAS_PANDAS = True
    except ImportError:
        HAS_PANDAS = False
        pd = None
    
    # Check if deprecated as_ parameter was used
    import inspect
    frame = inspect.currentframe()
    if frame and frame.f_back:
        caller_locals = frame.f_back.f_locals
        if 'as_' in caller_locals:
            raise TypeError(
                "transform() no longer accepts 'as_' parameter.\n"
                "The 'as_' parameter has been renamed to 'name' in v0.1.4.\n"
                "\n"
                "Migration:\n"
                "  OLD: add.transform('@calc', df, expression='a + b', as_='total')\n"
                "  NEW: add.transform('@calc', df, expression='a + b', name='total')\n"
                "\n"
                "For @sort mode, use 'order' parameter instead:\n"
                "  OLD: add.transform('@sort', df, by='date', as_='desc')\n"
                "  NEW: add.transform('@sort', df, by='date', order='desc')"
            )
    
    # For @sort mode, use order parameter instead of name
    if mode == '@sort' and order is not None:
        name = order
    
    # Convert tuples to lists for all list parameters
    if isinstance(columns, tuple):
        columns = list(columns)
    if isinstance(by, tuple):
        by = list(by)
    if isinstance(expression, tuple):
        expression = list(expression)
    if isinstance(name, tuple):
        name = list(name)
    if isinstance(infer, tuple):
        infer = list(infer)
    if isinstance(against, tuple):
        against = list(against)
    if isinstance(method, tuple):
        method = list(method)
    
    # Track original types before conversion (needed for @deduce validation)
    method_was_list = isinstance(method, list)
    name_was_list = isinstance(name, list)
    
    # Convert strings to lists (Rust expects lists)
    # EXCEPT for @sort mode where name is the sort order
    if isinstance(columns, str):
        columns = [columns]
    if isinstance(by, str):
        by = [by]
    if isinstance(expression, str):
        expression = [expression]
    if isinstance(name, str) and mode != '@sort':
        name = [name]
    if isinstance(infer, str):
        infer = [infer]
    if isinstance(against, str):
        against = [against]
    if isinstance(method, str):
        method = [method]
    
    # Detect backend and convert to polars early (needed for @deduce validation)
    is_pandas = isinstance(df, pd.DataFrame) if HAS_PANDAS and pd is not None else False
    is_polars = isinstance(df, pl.DataFrame)
    
    if not is_pandas and not is_polars:
        raise TypeError(
            f"DataFrame must be pandas or polars, got {type(df).__name__}"
        )
    
    # Convert to polars if needed
    if is_pandas:
        df_pl = pl.from_pandas(df)
    else:
        df_pl = df
    
    # Validate @deduce mode parameters
    if mode == '@deduce':
        # Validate infer parameter is provided
        if infer is None:
            raise ValueError(
                "@deduce mode requires 'infer' parameter.\n"
                "Specify the column(s) to fill with imputed/deduced values.\n"
                "\n"
                "Examples:\n"
                "  add.transform('@deduce', df, infer='price', method='mean')\n"
                "  add.transform('@deduce', df, infer='status', against='comments')"
            )
        
        # Set columns to infer for Rust backend compatibility
        # The Rust dispatcher will use infer parameter, but some validation code expects columns
        if columns is None:
            columns = infer
        
        # Validate infer columns exist in DataFrame
        df_columns = df_pl.columns
        infer_list = infer if isinstance(infer, list) else [infer]
        for col in infer_list:
            if col not in df_columns:
                raise ValueError(
                    f"Column '{col}' specified in 'infer' parameter does not exist in DataFrame.\n"
                    f"Available columns: {', '.join(df_columns)}"
                )
        
        # Auto-detect method if not provided
        if method is None:
            # For single column, auto-detect
            if len(infer_list) == 1:
                method = _auto_detect_method(df_pl, infer_list[0], against)
                method = [method]  # Convert to list for consistency
            else:
                # Multiple columns require explicit method
                raise ValueError(
                    f"Multiple columns in 'infer' parameter require explicit 'method' parameter.\n"
                    f"Specify method as a string (applied to all) or list (one per column).\n"
                    f"\n"
                    f"Examples:\n"
                    f"  add.transform('@deduce', df, infer={infer_list}, method='mean')\n"
                    f"  add.transform('@deduce', df, infer={infer_list}, method=['mean', 'median'])"
                )
        else:
            # Validate method parameter
            method_list = method if isinstance(method, list) else [method]
            
            # Check if method count matches infer count (only when method was originally a list)
            if method_was_list and len(method_list) != len(infer_list):
                raise ValueError(
                    f"Number of methods ({len(method_list)}) must match number of infer columns ({len(infer_list)}).\n"
                    f"Either provide a single method (applied to all) or one method per column."
                )
            
            # If single method provided for multiple columns, replicate it
            if not method_was_list and len(infer_list) > 1:
                method = [method_list[0]] * len(infer_list)
            
            # Validate TF-IDF requires against parameter
            if 'tfidf' in method_list and against is None:
                raise ValueError(
                    "TF-IDF method requires 'against' parameter.\n"
                    "Specify the text column(s) to use for similarity calculation.\n"
                    "\n"
                    "Example:\n"
                    "  add.transform('@deduce', df, infer='status', against='comments', method='tfidf')"
                )
        
        # Validate against columns exist if provided
        if against is not None:
            against_list = against if isinstance(against, list) else [against]
            for col in against_list:
                if col not in df_columns:
                    raise ValueError(
                        f"Column '{col}' specified in 'against' parameter does not exist in DataFrame.\n"
                        f"Available columns: {', '.join(df_columns)}"
                    )
        
        # Auto-generate names if not provided
        if name is None:
            name = []
            for col in infer_list:
                generated_name = _auto_generate_name(df_pl, col)
                name.append(generated_name)
        else:
            # Validate name doesn't conflict with existing columns
            name_list = name if isinstance(name, list) else [name]
            
            # Check if name count matches infer count (only when name was originally a list)
            if name_was_list and len(name_list) != len(infer_list):
                raise ValueError(
                    f"Number of names ({len(name_list)}) must match number of infer columns ({len(infer_list)})."
                )
            
            for col in name_list:
                if col in df_columns:
                    raise ValueError(
                        f"Column '{col}' specified in 'name' parameter already exists in DataFrame.\n"
                        f"Choose a different name or use auto-naming by omitting the 'name' parameter."
                    )
    
    # Validate mutual exclusion (lineage + as_type)
    _validate_lineage_as_type_exclusion(lineage, as_type, 'add.transform')
    
    # Call Rust implementation if available
    if RUST_AVAILABLE:
        # Convert to Arrow IPC bytes
        import io
        buffer = io.BytesIO()
        df_pl.write_ipc(buffer)
        df_bytes = buffer.getvalue()
        
        # Prepare parameters
        params = {
            'mode': mode,
            'fetch': columns,  # Updated: use columns parameter
            'by': by,
            'on': expression,  # Updated: use expression parameter directly
            'where': where,
            'as': name,  # Map name parameter to 'as' field for Rust backend
            'fetch_at': position,  # Map position to fetch_at for Rust
            'strategy': strategy,
            'columns': columns,  # Also pass as columns for compatibility
            'infer': infer,  # @deduce: columns to fill
            'against': against,  # @deduce: text columns for TF-IDF
            'method': method,  # @deduce: imputation method(s)
            'logging': logging,
        }
        
        # Call Rust transform
        try:
            result_bytes = _additory.transform(df_bytes, params)
        except Exception as e:
            raise RuntimeError(f"Transform failed: {e}") from e
        
        # Convert back to DataFrame
        result_buffer = io.BytesIO(result_bytes)
        result_df = pl.read_ipc(result_buffer)
    else:
        # Fallback to pure Python (limited modes)
        if mode == '@knn':
            # Use Python KNN implementation
            from transform.knn import perform_knn_imputation
            
            if not columns:
                raise ValueError("columns parameter is required for @knn mode")
            
            result_df = perform_knn_imputation(df_pl, columns, strategy or {})
        else:
            raise NotImplementedError(
                f"Mode {mode} requires Rust bindings. "
                f"Install with: pip install additory[rust]"
            )
    
    # Handle as_type parameter for output format (BEFORE lineage tracking)
    if as_type == 'pandas':
        # Force pandas output
        result_df = result_df.to_pandas()
    elif as_type == 'polars':
        # Force polars output
        pass  # Already polars
    else:
        # Default: match input type
        if is_pandas:
            result_df = result_df.to_pandas()
    
    # NEW: Lineage tracking (AFTER type conversion)
    if lineage:
        # Initialize tracker
        tracker = Lineage_Tracker()
        
        # Copy existing lineage from input
        existing_lineage = _get_lineage(df)
        if existing_lineage:
            result_df = _set_lineage(result_df, existing_lineage)
        
        # Record operation
        result_df = tracker.record_operation(
            df=result_df,
            operation_type='add.transform',
            params={
                'mode': mode,
                'columns': columns,
                'expression': expression,
                'where': where,
                'by': by,
                'strategy': strategy
            },
            rows_before=len(df),
            rows_after=len(result_df),
            columns_added=_get_added_columns(df, result_df),
            columns_modified=_get_modified_columns(df, result_df),
            excluded_rows=_get_excluded_rows(df, result_df) if mode == '@filter' else None
        )
        
        # Mode-specific column source updates
        lineage_data = _get_lineage(result_df)
        if lineage_data:
            operation_index = len(lineage_data['operations']) - 1
            
            if mode == '@calc' and strategy:
                tracker.update_column_sources_for_calc(
                    lineage=lineage_data,
                    operation_index=operation_index,
                    calculated_columns=strategy,
                    available_columns=list(df.columns)
                )
            elif mode == '@filter':
                # Update row mapping
                kept_indices = list(range(len(result_df)))  # Simplified
                tracker.update_row_mapping_for_filter(
                    lineage=lineage_data,
                    kept_indices=kept_indices
                )
            elif mode == '@aggregate' and strategy:
                tracker.update_column_sources_for_aggregate(
                    lineage=lineage_data,
                    operation_index=operation_index,
                    aggregated_columns=strategy,
                    source_columns={col: col for col in strategy.keys()}  # Simplified
                )
    
    return result_df


def synthetic(
    df_or_mode: Union['pd.DataFrame', 'pl.DataFrame', str],
    n: Optional[int] = None,
    *,
    strategy: Optional[Dict[str, Any]] = None,
    seed: Optional[int] = 42,
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None,
    **kwargs
) -> Union['pd.DataFrame', 'pl.DataFrame']:
    """
    Create or augment with synthetic data.
    
    The first argument determines the mode:
    - Pass a DataFrame to augment it with synthetic rows (Augment mode)
    - Pass '@new' to create a new synthetic DataFrame from scratch (New mode)
    
    Args:
        df_or_mode: DataFrame to augment, or the string '@new' to create from scratch
        n: Number of rows to generate
        strategy: Generation strategies for columns (dict)
        seed: Random seed for reproducibility (default: 42 for deterministic behavior)
        logging: Enable detailed logging
        lineage: Enable lineage tracking (default: False)
        as_type: Force output type ('pandas' or 'polars')
    
    Returns:
        DataFrame: Generated or augmented DataFrame
        
    Raises:
        ImportError: If Rust bindings are not available
        ValueError: If first argument is an invalid string (not '@new')
        TypeError: If old mode= keyword is used (migration message provided)
        RuntimeError: If operation fails
        
    Examples:
        >>> # Create new synthetic DataFrame
        >>> result = add.synthetic('@new', n=1000, 
        ...                        strategy={'age': 'normal:mean=50:std=10',
        ...                                  'salary': 'lognormal:mean=10.5:std=0.5',
        ...                                  'dept': 'categorical'})
        
        >>> # Augment existing DataFrame (pipeable)
        >>> result = add.synthetic(df, n=100, strategy={'id': 'increment'})
        >>> result = df.pipe(add.synthetic, n=100, strategy={'id': 'increment'})
        
        >>> # Non-deterministic generation
        >>> result = add.synthetic('@new', n=100, 
        ...                        strategy={'id': 'increment'}, seed=None)
        
    Note:
        For data analysis, use add.analyze() or add.analyse() instead.
    """
    import polars as pl
    try:
        import pandas as pd
        HAS_PANDAS = True
    except ImportError:
        HAS_PANDAS = False
        pd = None
    import io

    # --- Catch old mode= keyword usage ---
    if 'mode' in kwargs:
        old_mode = kwargs['mode']
        raise TypeError(
            f"add.synthetic() no longer accepts 'mode' as a keyword argument.\n"
            f"The signature has changed — pass a DataFrame or '@new' as the first argument.\n"
            f"\n"
            f"Old usage:  add.synthetic(mode='{old_mode}', df=df, n=100)\n"
            f"New usage:  add.synthetic(df, n=100)          # augment mode\n"
            f"            add.synthetic('@new', n=100, strategy={{...}})  # new mode"
        )

    # Reject any other unexpected kwargs
    if kwargs:
        unexpected = ', '.join(sorted(kwargs.keys()))
        raise TypeError(
            f"add.synthetic() got unexpected keyword argument(s): {unexpected}"
        )

    # --- Mode inference from df_or_mode ---
    is_pandas_df = HAS_PANDAS and pd is not None and isinstance(df_or_mode, pd.DataFrame)
    is_polars_df = isinstance(df_or_mode, pl.DataFrame)

    if is_pandas_df or is_polars_df:
        # Augment mode — DataFrame passed
        mode = '@augment'
        df = df_or_mode
    elif isinstance(df_or_mode, str):
        if df_or_mode == '@new':
            mode = '@new'
            df = None
        else:
            raise ValueError(
                f"Invalid first argument: '{df_or_mode}'.\n"
                f"Expected a pandas/polars DataFrame or the string '@new'.\n"
                f"\n"
                f"Examples:\n"
                f"  add.synthetic(df, n=100)                        # augment existing DataFrame\n"
                f"  add.synthetic('@new', n=100, strategy={{...}})   # create new DataFrame"
            )
    else:
        raise ValueError(
            f"Invalid first argument: received {type(df_or_mode).__name__}.\n"
            f"Expected a pandas/polars DataFrame or the string '@new'.\n"
            f"\n"
            f"Examples:\n"
            f"  add.synthetic(df, n=100)                        # augment existing DataFrame\n"
            f"  add.synthetic('@new', n=100, strategy={{...}})   # create new DataFrame"
        )

    # Log non-pipeable warning for @new mode (before Rust check so it's always visible)
    if logging and mode == '@new':
        logger.info(
            "add.synthetic() called in @new mode — New mode is not pipeable. "
            "Only augment mode (pass a DataFrame) supports .pipe()."
        )

    if not RUST_AVAILABLE:
        raise ImportError(
            "add.synthetic() requires Rust bindings. "
            "Install with: pip install additory[rust]"
        )

    # Validate mutual exclusion (lineage + as_type)
    _validate_lineage_as_type_exclusion(lineage, as_type, 'add.synthetic')

    # Detect DataFrame type if provided
    target_is_pandas = False
    target_is_polars = False

    if df is not None:
        target_is_pandas = isinstance(df, pd.DataFrame) if HAS_PANDAS and pd is not None else False
        target_is_polars = isinstance(df, pl.DataFrame)

        if not target_is_pandas and not target_is_polars:
            raise TypeError(
                f"DataFrame must be pandas or polars, got {type(df).__name__}"
            )

        # Convert to Polars if needed
        if target_is_pandas:
            target_pl = pl.from_pandas(df)
        else:
            target_pl = df
    else:
        target_pl = None

    # Serialize DataFrame to Arrow IPC bytes if provided
    if target_pl is not None:
        target_buffer = io.BytesIO()
        target_pl.write_ipc(target_buffer)
        target_bytes = target_buffer.getvalue()
    else:
        target_bytes = None

    # Validate parameters based on mode
    if mode == '@new':
        if n is None:
            raise ValueError("Parameter 'n' is required for @new mode")
        if strategy is None:
            raise ValueError("Parameter 'strategy' is required for @new mode")

        # Validate n is positive
        if not isinstance(n, int) or n <= 0:
            raise ValueError(f"Parameter 'n' must be a positive integer, got: {n}")
    elif mode == '@augment':
        if n is None:
            raise ValueError("Parameter 'n' is required for @augment mode")

        # Validate n is positive
        if not isinstance(n, int) or n <= 0:
            raise ValueError(f"Parameter 'n' must be a positive integer, got: {n}")

    # Prepare parameters
    params = {
        'mode': mode,
        'n': n,
        'fetch': None,  # No longer used, strategy only
        'strategy': strategy,
        'seed': seed,
        'logging': logging,
    }

    # Call Rust synthetic function
    try:
        result_bytes = _additory.synthetic(target_bytes, params)
    except Exception as e:
        raise RuntimeError(f"add.synthetic() failed: {e}") from e

    # Deserialize result
    result_buffer = io.BytesIO(result_bytes)
    result_df = pl.read_ipc(result_buffer)

    # Handle as_type parameter for output format (BEFORE lineage tracking)
    if as_type == 'pandas':
        # Force pandas output
        result_df = result_df.to_pandas()
    elif as_type == 'polars':
        # Force polars output
        pass  # Already polars
    else:
        # Default: match input type
        if target_is_pandas:
            result_df = result_df.to_pandas()

    # Lineage tracking (AFTER type conversion)
    if lineage:
        # Initialize tracker
        tracker = Lineage_Tracker()

        # Copy existing lineage from input (for augment mode)
        if mode == '@augment' and df is not None:
            existing_lineage = _get_lineage(df)
            if existing_lineage:
                result_df = _set_lineage(result_df, existing_lineage)

        # Record operation
        result_df = tracker.record_operation(
            df=result_df,
            operation_type='add.synthetic',
            params={
                'mode': mode,
                'n': n,
                'strategy': strategy,
                'seed': seed
            },
            rows_before=len(df) if df is not None else 0,
            rows_after=len(result_df),
            columns_added=list(strategy.keys()) if mode == '@new' and strategy else None,
            columns_modified=None
        )

    return result_df


def scan(
    mode: str,
    df=None,
    *,
    old=None,
    new=None,
    key=None,
    columns=None,
    where=None,
    rows=None,
    trace=None,
    focus=None,
    strategy=None,
    logging=False,
    as_type=None,
):
    """
    Inspect, analyze, and explain DataFrames — or configure expression folders.

    When *mode* is ``'@set'``, the second positional argument is interpreted as
    a path string (to set the user expression folder) or the literal ``'show'``
    (to return the current user folder path).

    When *mode* is ``'@diff'``, compares two DataFrames (``old`` and ``new``)
    and returns a diff result classifying every row.

    All other modes are forwarded to the Rust-backed scan implementation.
    """
    if mode == '@diff':
        from .diff_engine import diff as _diff
        return _diff(
            old=old,
            new=new,
            key=key,
            strategy=strategy,
            logging=logging,
            as_type=as_type,
        )

    if mode == '@set':
        registry = get_registry()
        if df == 'show':
            if registry.user_folder is not None:
                return str(registry.user_folder.path)
            return None
        # Treat df as a folder path string
        if not isinstance(df, str):
            raise ValueError(
                "add.scan('@set', ...) expects a path string or 'show'. "
                f"Got {type(df).__name__}."
            )
        path = Path(df)
        if not path.exists():
            raise ValueError(f"Folder does not exist: {df}")
        if not path.is_dir():
            raise ValueError(f"Path is not a directory: {df}")
        registry.set_user_folder(df)
        return None

    # Forward everything else to the Rust-backed scan
    return _rust_scan(
        mode,
        df,
        columns=columns,
        where=where,
        rows=rows,
        trace=trace,
        focus=focus,
        as_type=as_type,
    )


# ---------------------------------------------------------------------------
# Dynamic expression API  (add.<name>(df))
# ---------------------------------------------------------------------------

def _make_dynamic_function(name: str, expr: Expression):
    """Create a callable that evaluates an expression against a DataFrame."""

    def dynamic_fn(
        df,
        *,
        position=None,
        logging: bool = False,
        lineage: bool = False,
        as_type=None,
        **column_mappings,
    ):
        import polars as pl
        try:
            import pandas as pd
            HAS_PANDAS = True
        except ImportError:
            HAS_PANDAS = False
            pd = None

        # --- Validate DataFrame ---
        is_pandas = HAS_PANDAS and pd is not None and isinstance(df, pd.DataFrame)
        is_polars = isinstance(df, pl.DataFrame)
        if not is_pandas and not is_polars:
            raise TypeError(
                f"First argument must be a pandas or polars DataFrame, "
                f"got {type(df).__name__}"
            )

        # --- Determine required input names ---
        required_inputs = list(expr.inputs.keys())

        # --- Reject unknown mapping keywords ---
        unknown = set(column_mappings.keys()) - set(required_inputs)
        if unknown:
            raise ValueError(
                f"Unknown column mapping keyword(s) for expression '{name}': "
                f"{', '.join(sorted(unknown))}. "
                f"Valid input names: {', '.join(sorted(required_inputs))}"
            )

        # --- Build final column mapping (input_name -> df_column) ---
        df_columns = list(df.columns)
        mapping: Dict[str, str] = {}

        for inp in required_inputs:
            if inp in column_mappings:
                # Explicit mapping
                mapping[inp] = column_mappings[inp]
            elif inp in df_columns:
                # Auto-match
                mapping[inp] = inp
            else:
                # Cannot resolve — build helpful error
                suggested_args = ", ".join(
                    f"{r}='<your_column>'" for r in required_inputs
                )
                raise ValueError(
                    f"Column auto-matching failed for expression '{name}'.\n"
                    f"Required input columns: {required_inputs}\n"
                    f"DataFrame columns: {df_columns}\n"
                    f"Missing: '{inp}'\n\n"
                    f"Suggested call:\n"
                    f"  add.{name}(df, {suggested_args})"
                )

        # --- Rewrite formula with mapped column names ---
        formula = expr.expression
        # Sort by length descending to avoid partial replacements
        replacements = sorted(
            ((inp, col) for inp, col in mapping.items() if inp != col),
            key=lambda t: len(t[0]),
            reverse=True,
        )
        for inp_name, col_name in replacements:
            # Use word-boundary replacement
            formula = re.sub(r'\b' + re.escape(inp_name) + r'\b', col_name, formula)

        output_column = expr.output_column

        # --- Log if requested ---
        if logging:
            logger.info(
                "add.%s() — formula: %s, output: %s, mapping: %s",
                name, formula, output_column, mapping,
            )
            # Warn on overwrite
            if output_column in df_columns:
                logger.warning(
                    "add.%s() — overwriting existing column '%s'",
                    name, output_column,
                )

        # --- Delegate to transform('@calc', ...) ---
        return transform(
            '@calc',
            df,
            expression=formula,
            name=output_column,
            position=position if position is not None else 'end',
            logging=logging,
            lineage=lineage,
            as_type=as_type,
        )

    dynamic_fn.__name__ = name
    dynamic_fn.__doc__ = f"{expr.description}\n\nExpression: {expr.expression}"
    return dynamic_fn


def __getattr__(name: str):
    """Resolve dynamic expression names as callable functions."""
    if name.startswith('_'):
        raise AttributeError(f"module 'additory' has no attribute '{name}'")

    if name in RESERVED_NAMES:
        raise AttributeError(f"module 'additory' has no attribute '{name}'")

    # Try to resolve from expression registry
    registry = get_registry()
    expr = registry.resolve_by_name(name)

    if expr is None:
        available = registry.list_all_names()
        hint = ', '.join(sorted(available)[:10])
        suffix = '...' if len(available) > 10 else ''
        raise AttributeError(
            f"module 'additory' has no attribute '{name}'.\n"
            f"Available expressions: {hint}{suffix}"
        )

    return _make_dynamic_function(name, expr)


# Expose key functions
__all__ = [
    'to',
    'transform',
    'synthetic',
    'scan',
    'analyze',
    'analyse',
    'set',
    '__version__',
    'RUST_AVAILABLE',
]


# Print warning if Rust bindings not available
if not RUST_AVAILABLE:
    import warnings
    warnings.warn(
        "Rust bindings not available. Only Python-only modes (@knn) will work. "
        "Install Rust bindings with: pip install additory[rust]",
        ImportWarning
    )
