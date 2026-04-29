"""
Parameter Handler for v0.1.3a5

Handles flexible input formats for positional parameters:
- Single column: 'col' or ['col'] (both work)
- Multiple columns: ['col1', 'col2'] (list required)
- Single key: 'key' or ['key'] (both work)
- Multiple keys: ('key1', 'key2') (tuple required)
"""


def normalize_column_input(value):
    """
    Normalize column input to standard format.
    
    Rules:
    - Single column: 'col' or ['col'] → 'col' (string)
    - Multiple columns: ['col1', 'col2'] → ['col1', 'col2'] (list)
    
    Args:
        value: Column specification (str or list)
        
    Returns:
        str or list: Normalized column specification
        
    Raises:
        TypeError: If value is not str or list
        ValueError: If list is empty
        
    Examples:
        >>> normalize_column_input('name')
        'name'
        
        >>> normalize_column_input(['name'])
        'name'
        
        >>> normalize_column_input(['name', 'city'])
        ['name', 'city']
    """
    if value is None:
        return None
    
    if isinstance(value, str):
        return value
    
    if isinstance(value, list):
        if len(value) == 0:
            raise ValueError("Column list cannot be empty")
        
        if len(value) == 1:
            # Single column in list → convert to string
            return value[0]
        
        # Multiple columns → keep as list
        return value
    
    raise TypeError(
        f"Column input must be string or list, got {type(value).__name__}"
    )


def normalize_key_input(value):
    """
    Normalize key input to standard format.
    
    Rules:
    - Single key: 'key' or ['key'] → 'key' (string)
    - Multiple keys: ('key1', 'key2') → ('key1', 'key2') (tuple)
    
    Args:
        value: Key specification (str, list, or tuple)
        
    Returns:
        str or tuple: Normalized key specification
        
    Raises:
        TypeError: If value is not str, list, or tuple
        ValueError: If list is used for multiple keys (should be tuple)
        
    Examples:
        >>> normalize_key_input('customer_id')
        'customer_id'
        
        >>> normalize_key_input(['customer_id'])
        'customer_id'
        
        >>> normalize_key_input(('customer_id', 'date'))
        ('customer_id', 'date')
    """
    if value is None:
        return None
    
    if isinstance(value, str):
        return value
    
    if isinstance(value, list):
        if len(value) == 0:
            raise ValueError("Key list cannot be empty")
        
        if len(value) == 1:
            # Single key in list → convert to string
            return value[0]
        
        # Multiple keys in list → ERROR (should be tuple)
        raise ValueError(
            "Multiple keys must be tuple, not list. "
            f"Use ('key1', 'key2') instead of ['key1', 'key2']"
        )
    
    if isinstance(value, tuple):
        if len(value) == 0:
            raise ValueError("Key tuple cannot be empty")
        
        # Multiple keys → keep as tuple
        return value
    
    raise TypeError(
        f"Key input must be string, list, or tuple, got {type(value).__name__}"
    )


def normalize_by_input(value):
    """
    Normalize 'by' parameter input (for @sort, @aggregate, @bankers_round).
    
    Rules:
    - Single column: 'col' → 'col' (string)
    - Multiple columns: ('col1', 'col2') → ('col1', 'col2') (tuple)
    - Lists NOT allowed: ['col'] → ERROR
    
    This enforces tuple format for consistency with add.to() against parameter.
    
    Args:
        value: By specification (str or tuple)
        
    Returns:
        str or tuple: Normalized by specification
        
    Raises:
        ValueError: If list is provided
        
    Examples:
        >>> normalize_by_input('age')
        'age'
        
        >>> normalize_by_input(('salary', 'age'))
        ('salary', 'age')
        
        >>> normalize_by_input(['age'])  # ERROR
        ValueError: by parameter does not accept lists
    """
    if value is None:
        return None
    
    if isinstance(value, str):
        return value
    
    if isinstance(value, list):
        # Lists not allowed for 'by' parameter
        raise ValueError(
            "by parameter does not accept lists. "
            f"Use string for single column: 'col' or tuple for multiple: ('col1', 'col2')"
        )
    
    if isinstance(value, tuple):
        if len(value) == 0:
            raise ValueError("by tuple cannot be empty")
        return value
    
    raise TypeError(
        f"by input must be string or tuple, got {type(value).__name__}"
    )


def normalize_expression_input(value):
    """
    Normalize expression input for @calc mode.
    
    Rules:
    - Single expression: 'expr' → 'expr' (string)
    - Multiple expressions: ['expr1', 'expr2'] → ['expr1', 'expr2'] (list)
    
    Args:
        value: Expression specification (str or list)
        
    Returns:
        str or list: Normalized expression specification
        
    Examples:
        >>> normalize_expression_input('price * quantity')
        'price * quantity'
        
        >>> normalize_expression_input(['a + b', 'a * b'])
        ['a + b', 'a * b']
    """
    if value is None:
        return None
    
    if isinstance(value, str):
        return value
    
    if isinstance(value, list):
        if len(value) == 0:
            raise ValueError("Expression list cannot be empty")
        
        # Keep as list (even for single expression)
        # This allows consistent handling of multiple expressions
        return value
    
    raise TypeError(
        f"Expression input must be string or list, got {type(value).__name__}"
    )


def normalize_as_input(value):
    """
    Normalize 'as_' parameter input (output column names).
    
    Rules:
    - Single name: 'name' → 'name' (string)
    - Multiple names: ['name1', 'name2'] → ['name1', 'name2'] (list)
    - Sort order: 'asc' or 'desc' → 'asc' or 'desc' (string)
    
    Args:
        value: As specification (str or list)
        
    Returns:
        str or list: Normalized as specification
        
    Examples:
        >>> normalize_as_input('total')
        'total'
        
        >>> normalize_as_input(['sum', 'product'])
        ['sum', 'product']
        
        >>> normalize_as_input('desc')
        'desc'
    """
    if value is None:
        return None
    
    if isinstance(value, str):
        return value
    
    if isinstance(value, list):
        if len(value) == 0:
            raise ValueError("As list cannot be empty")
        
        # Keep as list (even for single name)
        return value
    
    raise TypeError(
        f"As input must be string or list, got {type(value).__name__}"
    )


def validate_expression_as_match(expression, as_value):
    """
    Validate that expression and as_ parameters match in length.
    
    For @calc mode with multiple expressions, the number of output names
    must match the number of expressions.
    
    Args:
        expression: Expression(s) (str or list)
        as_value: Output name(s) (str or list)
        
    Returns:
        bool: True if valid
        
    Raises:
        ValueError: If lengths don't match
    """
    if expression is None or as_value is None:
        return True
    
    # Normalize to lists for comparison
    expr_list = [expression] if isinstance(expression, str) else expression
    as_list = [as_value] if isinstance(as_value, str) else as_value
    
    if len(expr_list) != len(as_list):
        raise ValueError(
            f"Number of expressions ({len(expr_list)}) must match "
            f"number of output names ({len(as_list)})"
        )
    
    return True


# Export public API
__all__ = [
    'normalize_column_input',
    'normalize_key_input',
    'normalize_by_input',
    'normalize_expression_input',
    'normalize_as_input',
    'validate_expression_as_match',
]

