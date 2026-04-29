"""
Increment strategy for synthetic data generation.

Supports:
- Simple increment: 'increment' → 1, 2, 3, ...
- Step size: 'increment[3]' → 1, 4, 7, 10, ...
- Start value: 'increment:start=100' → 100, 101, 102, ...
- Pattern: 'increment:start=1:pattern=EMP_[001]' → EMP_001, EMP_002, ...
- Combined: 'increment[5]:start=100:pattern=ID_[0001]' → ID_0100, ID_0105, ...
"""

import re
from typing import Tuple, Optional, List, Union


def parse_increment_strategy(strategy_string: str) -> Tuple[int, int, Optional[str]]:
    """
    Parse increment strategy string.
    
    Args:
        strategy_string: Strategy string (e.g., 'increment[3]:start=100')
    
    Returns:
        tuple: (step, start, pattern)
            - step: Increment step size (default: 1)
            - start: Starting value (default: 1)
            - pattern: Pattern template with [000] placeholder (default: None)
    
    Examples:
        >>> parse_increment_strategy('increment')
        (1, 1, None)
        
        >>> parse_increment_strategy('increment[3]')
        (3, 1, None)
        
        >>> parse_increment_strategy('increment[5]:start=100')
        (5, 100, None)
        
        >>> parse_increment_strategy('increment:start=1:pattern=EMP_[001]')
        (1, 1, 'EMP_[001]')
        
        >>> parse_increment_strategy('increment[2]:start=1:pattern=EMP_[001]')
        (2, 1, 'EMP_[001]')
    """
    # Default values
    step = 1
    start = 1
    pattern = None
    
    # Extract step size from increment[step]
    step_match = re.search(r'increment\[(\d+)\]', strategy_string)
    if step_match:
        step = int(step_match.group(1))
    
    # Extract start value from :start=value
    start_match = re.search(r':start=(\d+)', strategy_string)
    if start_match:
        start = int(start_match.group(1))
    
    # Extract pattern from :pattern=template
    pattern_match = re.search(r':pattern=([^\:]+)', strategy_string)
    if pattern_match:
        pattern = pattern_match.group(1)
    
    return step, start, pattern


def generate_increment(n: int, step: int = 1, start: int = 1, 
                      pattern: Optional[str] = None) -> List[Union[int, str]]:
    """
    Generate increment sequence.
    
    Args:
        n: Number of values to generate
        step: Increment step size (default: 1)
        start: Starting value (default: 1)
        pattern: Pattern template with [000] or [001] placeholder (default: None)
    
    Returns:
        list: Generated sequence (integers or strings if pattern provided)
    
    Examples:
        >>> generate_increment(5)
        [1, 2, 3, 4, 5]
        
        >>> generate_increment(5, step=3)
        [1, 4, 7, 10, 13]
        
        >>> generate_increment(5, step=1, start=100)
        [100, 101, 102, 103, 104]
        
        >>> generate_increment(3, step=1, start=1, pattern='EMP_[001]')
        ['EMP_001', 'EMP_002', 'EMP_003']
        
        >>> generate_increment(3, step=5, start=100, pattern='ID_[0001]')
        ['ID_0100', 'ID_0105', 'ID_0110']
    """
    # Generate numeric sequence
    values = [start + (i * step) for i in range(n)]
    
    # Apply pattern if provided
    if pattern:
        # Find placeholder pattern [000], [001], [0001], etc. (any digits in brackets)
        placeholder_match = re.search(r'\[\d+\]', pattern)
        if placeholder_match:
            placeholder = placeholder_match.group(0)
            # Determine padding width from placeholder
            padding_width = len(placeholder) - 2  # Subtract brackets
            
            # Use regex sub to replace placeholder with formatted numbers
            formatted_values = []
            for value in values:
                # Replace the placeholder pattern with the formatted number
                formatted = re.sub(
                    r'\[\d+\]',
                    str(value).zfill(padding_width),
                    pattern
                )
                formatted_values.append(formatted)
            
            return formatted_values
        else:
            # No valid placeholder, return pattern + number
            return [f"{pattern}{value}" for value in values]
    
    return values


def validate_increment_step(step: int, n: int) -> Tuple[bool, str]:
    """
    Validate increment step size.
    
    Rule: Step size must be ≤ n/2 to avoid gaps in sequence.
    
    Args:
        step: Increment step size
        n: Number of rows to generate
    
    Returns:
        tuple: (is_valid, error_message)
    
    Examples:
        >>> validate_increment_step(10, 100)
        (True, '')
        
        >>> validate_increment_step(60, 100)
        (False, 'Step size 60 exceeds n/2 (50.0)...')
    """
    if step > n / 2:
        return False, (
            f"Step size {step} exceeds n/2 ({n/2}). "
            f"Step size must be ≤ n/2 to avoid gaps in sequence."
        )
    
    return True, ""


def format_with_pattern(value: int, pattern: str) -> str:
    """
    Format a numeric value with a pattern template.
    
    Args:
        value: Numeric value to format
        pattern: Pattern template with [000] or [001] placeholder
    
    Returns:
        str: Formatted string
    
    Examples:
        >>> format_with_pattern(5, 'EMP_[001]')
        'EMP_005'
        
        >>> format_with_pattern(123, 'ID_[0001]')
        'ID_0123'
        
        >>> format_with_pattern(5, 'USER_[00000]')
        'USER_00005'
    """
    # Find placeholder pattern [000], [001], [0001], etc. (any digits in brackets)
    placeholder_match = re.search(r'\[\d+\]', pattern)
    if placeholder_match:
        placeholder = placeholder_match.group(0)
        # Determine padding width from placeholder
        padding_width = len(placeholder) - 2  # Subtract brackets
        
        # Use regex sub to replace placeholder with formatted number
        formatted = re.sub(
            r'\[\d+\]',
            str(value).zfill(padding_width),
            pattern
        )
        
        return formatted
    else:
        # No valid placeholder, return pattern + number
        return f"{pattern}{value}"
