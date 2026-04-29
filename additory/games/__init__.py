"""
Games module - Easter egg games for additory v0.1.3.

Hidden feature. Not documented in main API.
"""

from .games import (
    play,
    tictactoe,
    sudoku,
    list_games
)

__all__ = [
    'play',
    'tictactoe',
    'sudoku',
    'list_games'
]
