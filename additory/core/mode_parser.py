"""
Mode Parser for v0.1.3a5

Parses simplified mode syntax:
- Simple mode: 'first', 'sum', 'concat'
- Mode with match: 'first:anycase', 'first:trim'
- Concat with separator: 'concat[,]', 'concat[\n]', 'concat[;]'
"""


def parse_mode(value_string):
    """
    Parse mode string into mode, match, and separator components.
    
    Format:
        - Simple: 'first' → ('first', 'auto', None)
        - Mode:match: 'first:anycase' → ('first', 'anycase', None)
        - Concat default: 'concat' → ('concat', 'auto', '|')
        - Concat custom: 'concat[,]' → ('concat', 'auto', ',')
    
    Args:
        value_string (str): Mode string to parse
        
    Returns:
        tuple: (mode, match, separator)
            - mode (str): The mode name
            - match (str): The match modifier ('auto' if not specified)
            - separator (str or None): Separator for concat mode, None otherwise
            
    Examples:
        >>> parse_mode('first')
        ('first', 'auto', None)
        
        >>> parse_mode('first:anycase')
        ('first', 'anycase', None)
        
        >>> parse_mode('concat')
        ('concat', 'auto', '|')
        
        >>> parse_mode('concat[,]')
        ('concat', 'auto', ',')
        
        >>> parse_mode('concat[\\n]')
        ('concat', 'auto', '\\n')
    """
    if not isinstance(value_string, str):
        raise TypeError(f"Mode must be string, got {type(value_string).__name__}")
    
    if not value_string:
        raise ValueError("Mode string cannot be empty")
    
    # Check for concat with separator
    if value_string.startswith('concat[') and value_string.endswith(']'):
        separator = value_string[7:-1]  # Extract between [ and ]
        
        # Handle escape sequences
        separator = separator.replace('\\n', '\n')
        separator = separator.replace('\\t', '\t')
        separator = separator.replace('\\r', '\r')
        
        return 'concat', 'auto', separator
    
    # Check for concat without separator (default to pipe)
    if value_string == 'concat':
        return 'concat', 'auto', '|'
    
    # Check for mode:match syntax
    if ':' in value_string:
        parts = value_string.split(':', 1)  # Split on first colon only
        mode = parts[0]
        match = parts[1]
        
        if not mode:
            raise ValueError("Mode cannot be empty before ':'")
        if not match:
            raise ValueError("Match modifier cannot be empty after ':'")
        
        return mode, match, None
    
    # Simple mode only
    return value_string, 'auto', None


def validate_mode(mode, match=None):
    """
    Validate that mode and match are valid options.
    
    Args:
        mode (str): Mode name
        match (str, optional): Match modifier
        
    Returns:
        bool: True if valid
        
    Raises:
        ValueError: If mode or match is invalid
    """
    # Valid modes (16 total)
    VALID_MODES = {
        # Basic (9)
        'auto', 'strict', 'first', 'last', 'shortest', 'longest',
        'most_common', 'forward_fill', 'backward_fill',
        # Numeric (5)
        'sum', 'count', 'average', 'min', 'max',
        # Concat (1)
        'concat',
    }
    
    # Valid match modifiers (5)
    VALID_MATCH = {
        'auto', 'anycase', 'fuzzy', 'enforce_case', 'trim'
    }
    
    if mode not in VALID_MODES:
        raise ValueError(
            f"Invalid mode '{mode}'. "
            f"Valid modes: {', '.join(sorted(VALID_MODES))}"
        )
    
    if match and match not in VALID_MATCH:
        raise ValueError(
            f"Invalid match modifier '{match}'. "
            f"Valid modifiers: {', '.join(sorted(VALID_MATCH))}"
        )
    
    return True


def parse_strategy_value(value):
    """
    Parse a strategy value which can be:
    - Simple string: 'first' → ('first', 'auto', None)
    - Dict with mode: {'mode': 'first:anycase', 'position': 'after:id'}
    
    Args:
        value: Strategy value (str or dict)
        
    Returns:
        dict: Parsed strategy with 'mode', 'match', 'separator', and other keys
    """
    if isinstance(value, str):
        # Simple string mode
        mode, match, separator = parse_mode(value)
        return {
            'mode': mode,
            'match': match,
            'separator': separator,
        }
    
    elif isinstance(value, dict):
        # Dict with mode and other options
        if 'mode' not in value:
            raise ValueError("Strategy dict must contain 'mode' key")
        
        mode_str = value['mode']
        mode, match, separator = parse_mode(mode_str)
        
        # Merge with other dict keys
        result = {
            'mode': mode,
            'match': match,
            'separator': separator,
        }
        
        # Add other keys (like position, rename, etc.)
        for key, val in value.items():
            if key != 'mode':
                result[key] = val
        
        return result
    
    else:
        raise TypeError(
            f"Strategy value must be string or dict, got {type(value).__name__}"
        )


# Export public API
__all__ = [
    'parse_mode',
    'validate_mode',
    'parse_strategy_value',
]

