"""
Expression system for additory v0.1.3.

Provides expression loading, registry, and resolution for @calc mode.
"""

from .loader import (
    Expression,
    ExpressionRegistry,
    InputDef,
    UserFolder,
    get_registry,
    set_user_folder,
    resolve_expression,
    list_expressions,
    load_add_file,
    format_expression_toml,
    _extract_identifiers,
)

__all__ = [
    'Expression',
    'ExpressionRegistry',
    'InputDef',
    'UserFolder',
    'get_registry',
    'set_user_folder',
    'resolve_expression',
    'list_expressions',
    'load_add_file',
    'format_expression_toml',
    '_extract_identifiers',
]
