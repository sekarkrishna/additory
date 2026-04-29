"""
Cardinality validation for add.to() operations.

This module validates that join operations have appropriate cardinality
relationships between the two DataFrames.
"""


def validate_cardinality(fetch_to, fetch_from, against, join_type='lookup'):
    """
    Validate cardinality before executing add.to() operation.
    
    Rules:
    1. Default (lookup): Only 1:many or many:1 (NO many:many)
    2. Explicit join_type: Only 1:1 (strictest)
    
    Args:
        fetch_to: Target DataFrame
        fetch_from: Source DataFrame
        against: Join key(s) - str or tuple
        join_type: Join type - 'lookup', 'left', 'inner', 'outer'
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # 1:1 relationship (valid for all)
        >>> is_valid, msg = validate_cardinality(df1, df2, 'id', 'lookup')
        >>> assert is_valid
        
        >>> # 1:many relationship (valid for lookup, invalid for explicit)
        >>> is_valid, msg = validate_cardinality(df1, df2, 'id', 'lookup')
        >>> assert is_valid
        >>> is_valid, msg = validate_cardinality(df1, df2, 'id', 'left')
        >>> assert not is_valid
        
        >>> # many:many relationship (invalid for all)
        >>> is_valid, msg = validate_cardinality(df1, df2, 'id', 'lookup')
        >>> assert not is_valid
    """
    # Handle both Pandas and Polars DataFrames
    try:
        # Try Polars first
        to_total = len(fetch_to)
        from_total = len(fetch_from)
        
        # Get unique counts for the join key(s)
        if isinstance(against, tuple):
            # Multiple keys - need to check combination
            to_unique = fetch_to.select(list(against)).n_unique()
            from_unique = fetch_from.select(list(against)).n_unique()
        else:
            # Single key
            to_unique = fetch_to[against].n_unique()
            from_unique = fetch_from[against].n_unique()
    except AttributeError:
        # Pandas DataFrame
        if isinstance(against, tuple):
            # Multiple keys
            to_unique = fetch_to[list(against)].drop_duplicates().shape[0]
            from_unique = fetch_from[list(against)].drop_duplicates().shape[0]
        else:
            # Single key
            to_unique = fetch_to[against].nunique()
            from_unique = fetch_from[against].nunique()
    
    # Determine if there are duplicates
    to_has_dups = to_total > to_unique
    from_has_dups = from_total > from_unique
    
    # Check for many:many (NEVER ALLOWED)
    if to_has_dups and from_has_dups:
        key_str = str(against) if isinstance(against, str) else ', '.join(against)
        return False, (
            f"Many:many relationship detected on '{key_str}'. "
            f"add.to() requires 1:many or many:1 relationships. "
            f"Target has {to_total} rows with {to_unique} unique keys, "
            f"source has {from_total} rows with {from_unique} unique keys."
        )
    
    # If explicit join_type, require 1:1
    if join_type in ['inner', 'outer', 'left']:
        if to_has_dups or from_has_dups:
            if to_has_dups:
                cardinality = 'many:1'
                side = 'target'
                total = to_total
                unique = to_unique
            else:
                cardinality = '1:many'
                side = 'source'
                total = from_total
                unique = from_unique
            
            key_str = str(against) if isinstance(against, str) else ', '.join(against)
            return False, (
                f"join_type='{join_type}' requires 1:1 relationship. "
                f"Detected {cardinality} relationship on '{key_str}'. "
                f"The {side} DataFrame has {total} rows with {unique} unique keys. "
                f"Use join_type='lookup' (default) for non-1:1 relationships."
            )
    
    return True, ""


def get_cardinality_type(fetch_to, fetch_from, against):
    """
    Determine the cardinality type of the relationship.
    
    Args:
        fetch_to: Target DataFrame
        fetch_from: Source DataFrame
        against: Join key(s) - str or tuple
    
    Returns:
        str: Cardinality type - '1:1', '1:many', 'many:1', or 'many:many'
    
    Examples:
        >>> cardinality = get_cardinality_type(df1, df2, 'id')
        >>> print(cardinality)  # '1:1', '1:many', 'many:1', or 'many:many'
    """
    # Handle both Pandas and Polars DataFrames
    try:
        # Try Polars first
        to_total = len(fetch_to)
        from_total = len(fetch_from)
        
        # Get unique counts for the join key(s)
        if isinstance(against, tuple):
            # Multiple keys - need to check combination
            to_unique = fetch_to.select(list(against)).n_unique()
            from_unique = fetch_from.select(list(against)).n_unique()
        else:
            # Single key
            to_unique = fetch_to[against].n_unique()
            from_unique = fetch_from[against].n_unique()
    except AttributeError:
        # Pandas DataFrame
        if isinstance(against, tuple):
            # Multiple keys
            to_unique = fetch_to[list(against)].drop_duplicates().shape[0]
            from_unique = fetch_from[list(against)].drop_duplicates().shape[0]
        else:
            # Single key
            to_unique = fetch_to[against].nunique()
            from_unique = fetch_from[against].nunique()
    
    # Determine cardinality
    to_has_dups = to_total > to_unique
    from_has_dups = from_total > from_unique
    
    if not to_has_dups and not from_has_dups:
        return '1:1'
    elif not to_has_dups and from_has_dups:
        return '1:many'
    elif to_has_dups and not from_has_dups:
        return 'many:1'
    else:
        return 'many:many'
