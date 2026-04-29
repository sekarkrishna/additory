"""
Synthetic data generation strategies.

This module provides strategies for generating synthetic data.
"""

from .strategies.increment import parse_increment_strategy, generate_increment

__all__ = [
    'parse_increment_strategy',
    'generate_increment',
]
