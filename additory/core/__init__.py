"""
Core utilities for additory v0.1.3a5

Modules:
- mode_parser: Parse simplified mode syntax
- param_handler: Handle flexible input formats
"""

from .mode_parser import parse_mode, validate_mode, parse_strategy_value
from .param_handler import (
    normalize_column_input,
    normalize_key_input,
    normalize_by_input,
    normalize_expression_input,
    normalize_as_input,
    validate_expression_as_match,
)

__all__ = [
    # Mode parsing
    'parse_mode',
    'validate_mode',
    'parse_strategy_value',
    # Parameter handling
    'normalize_column_input',
    'normalize_key_input',
    'normalize_by_input',
    'normalize_expression_input',
    'normalize_as_input',
    'validate_expression_as_match',
]

