"""
Position validation for add.to() operations.

This module validates position parameters for column placement.
"""


def validate_position(position, n_columns):
    """
    Validate position index for column placement.
    
    Valid range: 0 to n-1 (positive) or -1 to -(n-1) (negative)
    
    Args:
        position: Position value - int, str, or None
        n_columns: Number of columns in the DataFrame
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # Valid positive indices
        >>> is_valid, msg = validate_position(0, 5)
        >>> assert is_valid
        >>> is_valid, msg = validate_position(4, 5)
        >>> assert is_valid
        
        >>> # Valid negative indices
        >>> is_valid, msg = validate_position(-1, 5)
        >>> assert is_valid
        >>> is_valid, msg = validate_position(-4, 5)
        >>> assert is_valid
        
        >>> # Invalid positive index
        >>> is_valid, msg = validate_position(5, 5)
        >>> assert not is_valid
        
        >>> # Invalid negative index
        >>> is_valid, msg = validate_position(-5, 5)
        >>> assert not is_valid
        
        >>> # String positions always valid (validated at runtime)
        >>> is_valid, msg = validate_position('after:id', 5)
        >>> assert is_valid
        
        >>> # None is valid (means append to end)
        >>> is_valid, msg = validate_position(None, 5)
        >>> assert is_valid
    """
    # None is valid (means append to end)
    if position is None:
        return True, ""
    
    # String positions are validated at runtime (after:col, before:col, start, end)
    if isinstance(position, str):
        return True, ""
    
    # Must be an integer (but not bool, which is a subclass of int)
    if not isinstance(position, int) or isinstance(position, bool):
        return False, f"Position must be int, str, or None, got {type(position).__name__}"
    
    # Check range for positive indices
    if position >= 0:
        if position >= n_columns:
            return False, (
                f"Invalid position index {position} for DataFrame with {n_columns} columns. "
                f"Valid range: 0 to {n_columns-1} (positive) or -1 to -{n_columns-1} (negative)."
            )
    # Check range for negative indices
    else:
        if position < -(n_columns - 1):
            return False, (
                f"Invalid position index {position} for DataFrame with {n_columns} columns. "
                f"Valid range: 0 to {n_columns-1} (positive) or -1 to -{n_columns-1} (negative)."
            )
    
    return True, ""


def validate_string_position(position, df):
    """
    Validate string position references (after:col, before:col).
    
    Args:
        position: String position value
        df: DataFrame to validate against
    
    Returns:
        tuple: (is_valid: bool, error_message: str)
    
    Examples:
        >>> # Valid string positions
        >>> is_valid, msg = validate_string_position('after:id', df)
        >>> assert is_valid
        >>> is_valid, msg = validate_string_position('before:name', df)
        >>> assert is_valid
        >>> is_valid, msg = validate_string_position('start', df)
        >>> assert is_valid
        >>> is_valid, msg = validate_string_position('end', df)
        >>> assert is_valid
        
        >>> # Invalid - column doesn't exist
        >>> is_valid, msg = validate_string_position('after:nonexistent', df)
        >>> assert not is_valid
    """
    if not isinstance(position, str):
        return True, ""
    
    # Handle special keywords
    if position in ['start', 'end']:
        return True, ""
    
    # Handle after:col and before:col
    if ':' in position:
        parts = position.split(':', 1)
        if len(parts) != 2:
            return False, f"Invalid position format: '{position}'. Expected 'after:col' or 'before:col'"
        
        directive, col_name = parts
        
        if directive not in ['after', 'before']:
            return False, (
                f"Invalid position directive: '{directive}'. "
                f"Expected 'after' or 'before' (e.g., 'after:id' or 'before:name')"
            )
        
        # Check if column exists in DataFrame
        try:
            # Try Polars first
            columns = df.columns
        except AttributeError:
            # Pandas DataFrame
            columns = df.columns.tolist()
        
        if col_name not in columns:
            return False, (
                f"Column '{col_name}' not found in DataFrame. "
                f"Available columns: {', '.join(columns)}"
            )
        
        return True, ""
    
    # Unknown string format
    return False, (
        f"Invalid position format: '{position}'. "
        f"Expected: 'after:col', 'before:col', 'start', 'end', or numeric index"
    )


def normalize_position(position, n_columns):
    """
    Normalize position to a positive index.
    
    Args:
        position: Position value - int, str, or None
        n_columns: Number of columns in the DataFrame
    
    Returns:
        int or str: Normalized position (positive index or string)
    
    Examples:
        >>> # Positive indices stay the same
        >>> normalize_position(0, 5)
        0
        >>> normalize_position(2, 5)
        2
        
        >>> # Negative indices converted to positive
        >>> normalize_position(-1, 5)
        4
        >>> normalize_position(-2, 5)
        3
        
        >>> # None becomes end
        >>> normalize_position(None, 5)
        5
        
        >>> # Strings stay as strings
        >>> normalize_position('after:id', 5)
        'after:id'
    """
    if position is None:
        return n_columns  # Append to end
    
    if isinstance(position, str):
        return position  # String positions handled separately
    
    if position < 0:
        # Convert negative to positive
        return n_columns + position + 1
    
    return position
