"""
Scan module for add.scan() function - Rust-backed implementation

Provides data inspection, analysis, and lineage tracking capabilities.
The ``@set`` mode is handled entirely in Python for expression loading.
"""

from pathlib import Path
from typing import Union, List, Dict, Any, Optional
import pandas as pd
import polars as pl
import io
import json

try:
    from . import _additory
except ImportError:
    import _additory


def _handle_set_mode(
    *,
    expressions: Optional[str] = None,
    folder: Optional[str] = None,
    show: Optional[bool] = None,
) -> Optional[Dict[str, Any]]:
    """Handle scan('@set') — load expression files or folders into the registry.

    Parameters:
        expressions: Path to a single ``.add`` file to load.
        folder: Path to a directory of ``.add`` files.
        show: If ``True``, return a summary of loaded expressions.

    Returns:
        A summary dict when ``show=True``, otherwise ``None``.

    Raises:
        ValueError: When paths are invalid or no parameters are provided.
    """
    from .expressions.loader import load_add_file, get_registry

    if expressions is None and folder is None and not show:
        raise ValueError(
            "scan('@set') requires at least one parameter: "
            "expressions='path.add', folder='dir/', or show=True"
        )

    registry = get_registry()

    if expressions is not None:
        expr_path = Path(expressions)
        if not expr_path.exists():
            raise ValueError(f"Expression file does not exist: {expressions}")
        loaded = load_add_file(expr_path)
        for name, expr in loaded.items():
            registry.inbuilt[name] = expr

    if folder is not None:
        folder_path = Path(folder)
        if not folder_path.exists():
            raise ValueError(f"Folder does not exist: {folder}")
        if not folder_path.is_dir():
            raise ValueError(f"Path is not a directory: {folder}")
        registry.set_user_folder(folder)

    if show:
        result_exprs = []
        # Inbuilt expressions
        for name, expr in registry.inbuilt.items():
            result_exprs.append({
                "name": name,
                "category": expr.category,
                "source": expr.source_file or "",
            })
        # User folder expressions
        if registry.user_folder is not None:
            for name, expr in registry.user_folder.expressions.items():
                result_exprs.append({
                    "name": name,
                    "category": expr.category,
                    "source": expr.source_file or "",
                })
        return {"expressions": result_exprs}

    return None


def scan(
    mode: str,
    df: Optional[Union[pd.DataFrame, pl.DataFrame]] = None,
    *,
    columns: Optional[Union[str, List[str]]] = None,
    where: Optional[str] = None,
    rows: Optional[Union[str, List[str]]] = None,
    trace: Optional[List[int]] = None,
    focus: Optional[str] = None,
    as_type: Optional[str] = None,
    expressions: Optional[str] = None,
    folder: Optional[str] = None,
    show: Optional[bool] = None,
) -> Union[pd.DataFrame, pl.DataFrame, Dict, str, None]:
    """
    Inspect, analyze, and explain DataFrames.
    
    Modes:
        - '@analyze' or '@analyse': Statistical profiling
        - '@lineage': Transformation tracking
        - '@set': Load expression files/folders into the registry
    
    Parameters:
        mode: Scan mode string
        df: Input DataFrame (pandas or polars). Not used for '@set' mode.
        columns: Column filter (str or list)
        where: SQL-like filter condition
        rows: Row range specifications
        trace: [col_idx, row_idx] for cell tracing
        focus: Specialized analysis mode
        as_type: Output format ('dataframe', 'dict', 'text')
        expressions: Path to a .add file (for '@set' mode)
        folder: Path to a directory of .add files (for '@set' mode)
        show: Return summary of loaded expressions (for '@set' mode)
    
    Returns:
        Analysis results in specified format, or summary dict for '@set' mode
    """
    # Intercept @set mode before Rust dispatch
    if mode == '@set':
        if df is not None:
            raise ValueError("scan('@set') does not accept a DataFrame argument")
        return _handle_set_mode(expressions=expressions, folder=folder, show=show)

    # Non-@set modes require a DataFrame
    if df is None:
        raise TypeError("scan() requires a DataFrame for non-@set modes")

    # Detect DataFrame type
    is_pandas = isinstance(df, pd.DataFrame)
    is_polars = isinstance(df, pl.DataFrame)
    
    if not (is_pandas or is_polars):
        raise TypeError(
            f"df must be pandas or polars DataFrame, got {type(df).__name__}"
        )
    
    # Convert to polars for serialization
    if is_pandas:
        df_pl = pl.from_pandas(df)
    else:
        df_pl = df
    
    # Serialize DataFrame to Arrow IPC bytes
    buffer = io.BytesIO()
    df_pl.write_ipc(buffer)
    df_bytes = buffer.getvalue()
    
    # Retrieve lineage metadata if in lineage mode
    lineage_json = None
    if mode == '@lineage' or mode == '@analyse':
        lineage = _get_lineage(df)
        if lineage is not None:
            lineage_json = json.dumps(lineage)
        elif mode == '@lineage':
            # Lineage mode requires lineage metadata
            raise ValueError(
                "No lineage metadata found. Lineage tracking must be enabled by adding "
                "lineage=True to add.to(), add.transform(), or add.synthetic() calls.\n\n"
                "Example:\n"
                "  df = add.transform('@calc', df, strategy={'total': 'price * qty'}, lineage=True)\n"
                "  result = add.scan('@lineage', df)"
            )
    
    # Normalize columns parameter
    if columns is not None and isinstance(columns, str):
        columns = [columns]
    
    # Normalize rows parameter
    if rows is not None and isinstance(rows, str):
        rows = [rows]
    
    # Prepare parameters dict for Rust
    params = {
        'mode': mode,
        'columns': columns,
        'where': where,
        'rows': rows,
        'trace': trace,
        'focus': focus,
        'as_type': as_type,
        'lineage_json': lineage_json,
    }
    
    # Call Rust backend
    result = _additory.scan(df_bytes, params)
    
    # Convert result based on type
    if isinstance(result, bytes):
        # Arrow IPC bytes → DataFrame
        result_df = pl.read_ipc(io.BytesIO(result))
        
        # Convert back to original DataFrame type if as_type not specified
        if as_type is None or as_type == 'dataframe':
            if is_pandas:
                return result_df.to_pandas()
            else:
                return result_df
        else:
            return result_df
    elif isinstance(result, dict):
        # Dict output
        return result
    elif isinstance(result, str):
        # Text output
        return result
    else:
        # Unexpected type
        return result


def _get_lineage(df: Union[pd.DataFrame, pl.DataFrame]) -> Optional[Dict]:
    """
    Retrieve lineage metadata from DataFrame.
    
    For pandas: extracts from DataFrame.attrs['lineage']
    For Polars: extracts from global registry
    """
    if isinstance(df, pd.DataFrame):
        # Pandas stores lineage in attrs
        return df.attrs.get('lineage')
    elif isinstance(df, pl.DataFrame):
        # Polars uses global registry with (id, version) key
        try:
            from .lineage_tracker import _polars_lineage_registry, _get_polars_key
            key = _get_polars_key(df)
            return _polars_lineage_registry.get(key)
        except ImportError:
            return None
    else:
        return None
