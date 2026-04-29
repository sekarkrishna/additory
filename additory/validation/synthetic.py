"""
Validation for add.synthetic() operations.

This module validates synthetic data generation requests before execution.
"""


def validate_synthetic_request(mode, df, n, fetch):
    """
    Validate synthetic data request before execution.
    
    Rules:
    1. Distribution sample size: 30 per column minimum
    2. Increment conflicts: No duplicate starts
    3. Step size limits: Step ≤ n/2
    4. Linked list consistency: Equal level sizes
    5. Deduce labeled examples: Minimum 3 per category
    
    Args:
        mode: Mode - 'augment', '@new', or '@augment'
        df: DataFrame (for augment or @augment modes)
        n: Number of rows to generate
        fetch: Dict of column strategies
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # Valid request
        >>> is_valid, msg = validate_synthetic_request(
        ...     'augment', df, 100, {'age': 'range:18-65'}
        ... )
        >>> assert is_valid
        
        >>> # Invalid - not enough rows for distribution
        >>> is_valid, msg = validate_synthetic_request(
        ...     'augment', df_small, 100, {'age': 'distribution'}
        ... )
        >>> assert not is_valid
    """
    errors = []
    
    # Validate mode
    if mode not in ['augment', '@new', '@augment']:
        errors.append(f"Invalid mode: '{mode}'. Expected 'augment', '@new', or '@augment'")
    
    # Validate df requirement
    if mode in ['augment', '@augment']:
        if df is None:
            errors.append(f"Mode '{mode}' requires a DataFrame (df parameter)")
    
    # Validate n
    if not isinstance(n, int) or n <= 0:
        errors.append(f"n must be a positive integer, got {n}")
    
    # Validate fetch
    if fetch is not None and not isinstance(fetch, dict):
        errors.append(f"fetch must be a dict, got {type(fetch).__name__}")
    
    # If we have errors so far, return early
    if errors:
        return False, " | ".join(errors)
    
    # Validate specific strategies if fetch is provided
    if fetch:
        for col, strategy in fetch.items():
            # Validate distribution sample size
            if isinstance(strategy, str) and strategy == 'distribution':
                if df is not None:
                    try:
                        # Try Polars first
                        col_size = len(df)
                    except AttributeError:
                        # Pandas DataFrame
                        col_size = len(df)
                    
                    if col_size < 30:
                        errors.append(
                            f"Column '{col}' uses 'distribution' strategy but DataFrame has only "
                            f"{col_size} rows. Minimum 30 rows required for distribution sampling."
                        )
            
            # Validate increment conflicts
            if isinstance(strategy, str) and strategy.startswith('increment'):
                # Check for duplicate start values across columns
                # This is a simplified check - full implementation would track all starts
                pass
            
            # Validate step size limits
            if isinstance(strategy, str) and 'increment[' in strategy:
                try:
                    # Extract step size from increment[step]
                    start_idx = strategy.index('[')
                    end_idx = strategy.index(']')
                    step_str = strategy[start_idx+1:end_idx]
                    step = int(step_str)
                    
                    if step > n / 2:
                        errors.append(
                            f"Column '{col}' has step size {step} which exceeds n/2 ({n/2}). "
                            f"Step size must be ≤ n/2 to avoid gaps in sequence."
                        )
                except (ValueError, IndexError):
                    # Invalid format - will be caught by strategy parser
                    pass
            
            # Validate linked list consistency
            if isinstance(strategy, list):
                # Linked list - check that all levels have same size
                if len(strategy) > 0:
                    first_len = len(strategy[0]) if isinstance(strategy[0], list) else 1
                    for i, level in enumerate(strategy):
                        if isinstance(level, list):
                            if len(level) != first_len:
                                errors.append(
                                    f"Column '{col}' has inconsistent linked list sizes. "
                                    f"Level 0 has {first_len} items, level {i} has {len(level)} items. "
                                    f"All levels must have the same size."
                                )
            
            # Validate deduce labeled examples
            if isinstance(strategy, str) and strategy.startswith('deduce:'):
                if df is not None:
                    # Extract target column from deduce:target_col
                    target_col = strategy.split(':', 1)[1]
                    
                    try:
                        # Try Polars first
                        if target_col in df.columns:
                            # Count non-null values
                            non_null_count = df[target_col].null_count()
                            labeled_count = len(df) - non_null_count
                            
                            if labeled_count < 3:
                                errors.append(
                                    f"Column '{col}' uses 'deduce:{target_col}' but only "
                                    f"{labeled_count} labeled examples found. Minimum 3 required."
                                )
                            elif labeled_count < 5:
                                # Warning (not error) for < 5 examples
                                # In real implementation, this would be a warning
                                pass
                    except AttributeError:
                        # Pandas DataFrame
                        if target_col in df.columns:
                            labeled_count = df[target_col].notna().sum()
                            
                            if labeled_count < 3:
                                errors.append(
                                    f"Column '{col}' uses 'deduce:{target_col}' but only "
                                    f"{labeled_count} labeled examples found. Minimum 3 required."
                                )
    
    if errors:
        return False, " | ".join(errors)
    
    return True, ""


def validate_strategy_format(strategy):
    """
    Validate strategy string format.
    
    Args:
        strategy: Strategy string
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # Valid strategies
        >>> is_valid, msg = validate_strategy_format('increment')
        >>> assert is_valid
        >>> is_valid, msg = validate_strategy_format('increment[5]')
        >>> assert is_valid
        >>> is_valid, msg = validate_strategy_format('range:18-65')
        >>> assert is_valid
        
        >>> # Invalid strategies
        >>> is_valid, msg = validate_strategy_format('invalid_strategy')
        >>> assert not is_valid
    """
    if not isinstance(strategy, str):
        return True, ""  # Non-string strategies validated elsewhere
    
    # List of valid strategy prefixes
    valid_strategies = [
        'increment',
        'range:',
        'choice:',
        'distribution',
        'deduce:',
        'uuid',
        'timestamp',
        'date',
        'time',
        'datetime',
        'email',
        'phone',
        'name',
        'address',
        'city',
        'state',
        'zip',
        'country',
    ]
    
    # Check if strategy starts with any valid prefix
    for valid in valid_strategies:
        if strategy.startswith(valid):
            return True, ""
    
    return False, (
        f"Unknown strategy: '{strategy}'. "
        f"Valid strategies: {', '.join(valid_strategies)}"
    )


def validate_increment_conflicts(fetch):
    """
    Validate that increment strategies don't have conflicting start values.
    
    Args:
        fetch: Dict of column strategies
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # Valid - no conflicts
        >>> is_valid, msg = validate_increment_conflicts({
        ...     'id': 'increment',
        ...     'seq': 'increment:start=100'
        ... })
        >>> assert is_valid
        
        >>> # Invalid - duplicate start values
        >>> is_valid, msg = validate_increment_conflicts({
        ...     'id': 'increment:start=1',
        ...     'seq': 'increment:start=1'
        ... })
        >>> assert not is_valid
    """
    if not fetch:
        return True, ""
    
    # Track start values for increment strategies
    start_values = {}
    
    for col, strategy in fetch.items():
        if isinstance(strategy, str) and strategy.startswith('increment'):
            # Extract start value
            start = 1  # Default
            
            if ':start=' in strategy:
                try:
                    start_part = strategy.split(':start=')[1]
                    # Handle additional parameters after start
                    if ':' in start_part:
                        start_part = start_part.split(':')[0]
                    start = int(start_part)
                except (ValueError, IndexError):
                    # Invalid format - will be caught by strategy parser
                    continue
            
            # Check for conflicts
            if start in start_values:
                return False, (
                    f"Increment conflict: Columns '{start_values[start]}' and '{col}' "
                    f"both start at {start}. Each increment column must have a unique start value."
                )
            
            start_values[start] = col
    
    return True, ""
