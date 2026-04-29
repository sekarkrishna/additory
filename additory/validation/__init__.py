"""
Validation module for additory operations.

This module provides validation functions for:
- Cardinality validation (add.to)
- Position validation (add.to)
- Synthetic data validation (add.synthetic)
- List-only validation (all functions)
"""

from .cardinality import (
    validate_cardinality,
    get_cardinality_type
)

from .position import (
    validate_position,
    validate_string_position,
    normalize_position
)

from .synthetic import (
    validate_synthetic_request,
    validate_strategy_format,
    validate_increment_conflicts
)


def validate_list_not_tuple(value, param_name):
    """
    Validate that a parameter is a list, not a tuple.
    
    Philosophy Principle #3: Uniform patterns - use lists everywhere.
    
    Parameters:
        value: The value to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is a tuple
    
    Examples:
        >>> validate_list_not_tuple(['a', 'b'], 'against')  # OK
        >>> validate_list_not_tuple(('a', 'b'), 'against')  # Raises TypeError
    """
    if isinstance(value, tuple):
        list_version = list(value)
        raise TypeError(
            f"Parameter '{param_name}' must be a list, not tuple. "
            f"Use {list_version} instead of {value}"
        )


def validate_multiple_values(value, param_name):
    """
    Validate that multiple values are provided as a list.
    
    Accepts:
    - str (single value)
    - list (multiple values)
    
    Rejects:
    - tuple (with helpful error message)
    
    Parameters:
        value: The value to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is a tuple or invalid type
    
    Examples:
        >>> validate_multiple_values('col', 'against')  # OK - single value
        >>> validate_multiple_values(['col1', 'col2'], 'against')  # OK - list
        >>> validate_multiple_values(('col1', 'col2'), 'against')  # Raises TypeError
    """
    if isinstance(value, tuple):
        list_version = list(value)
        raise TypeError(
            f"Parameter '{param_name}' must be a list, not tuple. "
            f"Use {list_version} instead of {value}"
        )
    
    if not isinstance(value, (str, list)):
        raise TypeError(
            f"Parameter '{param_name}' must be a string or list, "
            f"got {type(value).__name__}"
        )





def validate_to_params(bring_to, bring_from, bring, against, **kwargs):
    """
    Validate parameters for add.to() function.
    
    Checks:
    - List vs tuple for multiple values
    
    Parameters:
        bring_to: DataFrame to bring columns to
        bring_from: DataFrame to bring columns from
        bring: Column(s) to bring
        against: Key column(s) to match against
        **kwargs: Additional parameters to check for old names
    
    Raises:
        TypeError: If validation fails
    """
    # Validate list vs tuple
    if bring is not None:
        validate_multiple_values(bring, 'bring')
    if against is not None:
        validate_multiple_values(against, 'against')


def validate_transform_params(mode, df, columns=None, by=None, **kwargs):
    """
    Validate parameters for add.transform() function.
    
    Checks:
    - List vs tuple for multiple values
    
    Parameters:
        mode: Transform mode
        df: DataFrame to transform
        columns: Column(s) to operate on
        by: Group/sort columns
        **kwargs: Additional parameters to check for old names
    
    Raises:
        TypeError: If validation fails
    """
    # Validate list vs tuple
    if columns is not None:
        validate_multiple_values(columns, 'columns')
    if by is not None:
        validate_multiple_values(by, 'by')


def validate_synthetic_params(mode, df=None, n=None, **kwargs):
    """
    Validate parameters for add.synthetic() function.
    
    Parameters:
        mode: Synthetic mode
        df: DataFrame (optional)
        n: Number of rows (optional)
        **kwargs: Additional parameters to check for old names
    
    Raises:
        TypeError: If validation fails
    """
    pass


def is_dataframe(obj):
    """
    Check if object is a pandas or polars DataFrame.
    
    Parameters:
        obj: Object to check
    
    Returns:
        bool: True if obj is a DataFrame, False otherwise
    """
    if obj is None:
        return False
    
    type_name = type(obj).__name__
    module_name = type(obj).__module__
    
    # Check for pandas DataFrame
    if 'pandas' in module_name and type_name == 'DataFrame':
        return True
    
    # Check for polars DataFrame
    if 'polars' in module_name and type_name == 'DataFrame':
        return True
    
    return False


def validate_dataframe_type(df, param_name):
    """
    Validate that parameter is a pandas or polars DataFrame.
    
    Philosophy Principle #1: Backend agnostic - support both pandas and polars.
    
    Parameters:
        df: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If df is not a DataFrame
    
    Examples:
        >>> import pandas as pd
        >>> import polars as pl
        >>> validate_dataframe_type(pd.DataFrame(), 'bring_to')  # OK
        >>> validate_dataframe_type(pl.DataFrame(), 'bring_to')  # OK
        >>> validate_dataframe_type([1, 2, 3], 'bring_to')  # Raises TypeError
    """
    if not is_dataframe(df):
        raise TypeError(
            f"Parameter '{param_name}' must be a pandas.DataFrame or polars.DataFrame, "
            f"got {type(df).__name__}"
        )


def validate_string_type(value, param_name):
    """
    Validate that parameter is a string.
    
    Parameters:
        value: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is not a string
    """
    if not isinstance(value, str):
        raise TypeError(
            f"Parameter '{param_name}' must be a string, "
            f"got {type(value).__name__}"
        )


def validate_string_or_list_type(value, param_name):
    """
    Validate that parameter is a string or list of strings.
    
    Parameters:
        value: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is not a string or list
    """
    if isinstance(value, tuple):
        # Provide helpful error for tuple
        list_version = list(value)
        raise TypeError(
            f"Parameter '{param_name}' must be a list, not tuple. "
            f"Use {list_version} instead of {value}"
        )
    
    if not isinstance(value, (str, list)):
        raise TypeError(
            f"Parameter '{param_name}' must be a string or list, "
            f"got {type(value).__name__}"
        )
    
    # If it's a list, validate all elements are strings
    if isinstance(value, list):
        for i, item in enumerate(value):
            if not isinstance(item, str):
                raise TypeError(
                    f"Parameter '{param_name}' list must contain only strings, "
                    f"but element at index {i} is {type(item).__name__}"
                )


def validate_int_type(value, param_name):
    """
    Validate that parameter is an integer.
    
    Parameters:
        value: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is not an integer
    """
    if not isinstance(value, int) or isinstance(value, bool):
        raise TypeError(
            f"Parameter '{param_name}' must be an integer, "
            f"got {type(value).__name__}"
        )


def validate_dict_type(value, param_name):
    """
    Validate that parameter is a dictionary.
    
    Parameters:
        value: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is not a dictionary
    """
    if not isinstance(value, dict):
        raise TypeError(
            f"Parameter '{param_name}' must be a dictionary, "
            f"got {type(value).__name__}"
        )


def validate_bool_or_string_type(value, param_name):
    """
    Validate that parameter is a boolean or string.
    
    Used for logging parameter which accepts False, True, or 'default'.
    
    Parameters:
        value: Object to validate
        param_name: Name of the parameter (for error message)
    
    Raises:
        TypeError: If value is not a boolean or string
    """
    if not isinstance(value, (bool, str)):
        raise TypeError(
            f"Parameter '{param_name}' must be a boolean or string, "
            f"got {type(value).__name__}"
        )


def validate_to_types(bring_to, bring_from, bring, against, position=None, 
                      strategy=None, join_type='lookup', logging='default', as_type=None):
    """
    Validate parameter types for add.to() function.
    
    Parameters:
        bring_to: DataFrame to bring columns to
        bring_from: DataFrame to bring columns from
        bring: Column(s) to bring
        against: Key column(s) to match against
        position: Where to place columns (optional)
        strategy: Advanced column control (optional)
        join_type: Type of join
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        TypeError: If any parameter has invalid type
    """
    # Validate DataFrames
    validate_dataframe_type(bring_to, 'bring_to')
    validate_dataframe_type(bring_from, 'bring_from')
    
    # Validate column specifications
    validate_string_or_list_type(bring, 'bring')
    validate_string_or_list_type(against, 'against')
    
    # Validate optional parameters
    if position is not None:
        if not isinstance(position, (str, int)):
            raise TypeError(
                f"Parameter 'position' must be a string or integer, "
                f"got {type(position).__name__}"
            )
    
    if strategy is not None:
        validate_dict_type(strategy, 'strategy')
    
    validate_string_type(join_type, 'join_type')
    validate_bool_or_string_type(logging, 'logging')
    
    if as_type is not None:
        validate_string_type(as_type, 'as_type')


def validate_transform_types(mode, df, columns=None, by=None, position=None,
                             strategy=None, logging='default', as_type=None):
    """
    Validate parameter types for add.transform() function.
    
    Parameters:
        mode: Transform mode
        df: DataFrame to transform
        columns: Column(s) to operate on (optional)
        by: Group/sort columns (optional)
        position: Where to place columns (optional)
        strategy: Mode-specific configuration (optional)
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        TypeError: If any parameter has invalid type
    """
    # Validate required parameters
    validate_string_type(mode, 'mode')
    validate_dataframe_type(df, 'df')
    
    # Validate optional parameters
    if columns is not None:
        validate_string_or_list_type(columns, 'columns')
    
    if by is not None:
        validate_string_or_list_type(by, 'by')
    
    if position is not None:
        if not isinstance(position, (str, int)):
            raise TypeError(
                f"Parameter 'position' must be a string or integer, "
                f"got {type(position).__name__}"
            )
    
    if strategy is not None:
        validate_dict_type(strategy, 'strategy')
    
    validate_bool_or_string_type(logging, 'logging')
    
    if as_type is not None:
        validate_string_type(as_type, 'as_type')


def validate_synthetic_types(mode, df=None, n=None, strategy=None, 
                             seed=42, logging='default', as_type=None):
    """
    Validate parameter types for add.synthetic() function.
    
    Parameters:
        mode: Synthetic mode
        df: DataFrame (optional)
        n: Number of rows (optional)
        strategy: Generation configuration (optional)
        seed: Random seed
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        TypeError: If any parameter has invalid type
    """
    # Validate required parameters
    validate_string_type(mode, 'mode')
    
    # Validate optional parameters
    if df is not None:
        validate_dataframe_type(df, 'df')
    
    if n is not None:
        validate_int_type(n, 'n')
    
    if strategy is not None:
        validate_dict_type(strategy, 'strategy')
    
    validate_int_type(seed, 'seed')
    validate_bool_or_string_type(logging, 'logging')
    
    if as_type is not None:
        validate_string_type(as_type, 'as_type')


def validate_join_type_value(join_type):
    """
    Validate that join_type has a valid value.
    
    Valid values: 'lookup', 'left', 'inner', 'outer'
    
    Parameters:
        join_type: Join type value to validate
    
    Raises:
        ValueError: If join_type is not a valid value
    
    Examples:
        >>> validate_join_type_value('lookup')  # OK
        >>> validate_join_type_value('left')  # OK
        >>> validate_join_type_value('invalid')  # Raises ValueError
    """
    valid_values = ['lookup', 'left', 'inner', 'outer']
    
    if join_type not in valid_values:
        raise ValueError(
            f"Parameter 'join_type' must be one of {valid_values}, "
            f"got '{join_type}'"
        )


def validate_logging_value(logging):
    """
    Validate that logging has a valid value.
    
    Valid values: False, True, 'default'
    
    Parameters:
        logging: Logging value to validate
    
    Raises:
        ValueError: If logging is not a valid value
    
    Examples:
        >>> validate_logging_value(False)  # OK
        >>> validate_logging_value(True)  # OK
        >>> validate_logging_value('default')  # OK
        >>> validate_logging_value('verbose')  # Raises ValueError
    """
    valid_values = [False, True, 'default']
    
    if logging not in valid_values:
        raise ValueError(
            f"Parameter 'logging' must be one of {valid_values}, "
            f"got {repr(logging)}"
        )


def validate_as_type_value(as_type):
    """
    Validate that as_type has a valid value.
    
    Valid values: None, 'pandas', 'polars'
    
    Parameters:
        as_type: Output type value to validate
    
    Raises:
        ValueError: If as_type is not a valid value
    
    Examples:
        >>> validate_as_type_value(None)  # OK
        >>> validate_as_type_value('pandas')  # OK
        >>> validate_as_type_value('polars')  # OK
        >>> validate_as_type_value('numpy')  # Raises ValueError
    """
    valid_values = [None, 'pandas', 'polars']
    
    if as_type not in valid_values:
        raise ValueError(
            f"Parameter 'as_type' must be one of {valid_values}, "
            f"got '{as_type}'"
        )


def validate_mode_value(mode, function_name):
    """
    Validate that mode has a valid value for the given function.
    
    For add.transform():
    - Valid modes: @calc, @harmonize, @sort, @round, @extract, @deduce, 
                   @analyze, @analyse, @knn, @cluster
    - Easter eggs (hidden): @tictactoe, @ttt, @sudoku
    
    For add.synthetic():
    - Valid modes: @new, @augment
    
    Parameters:
        mode: Mode value to validate
        function_name: Name of function ('transform' or 'synthetic')
    
    Raises:
        ValueError: If mode is not a valid value
    
    Examples:
        >>> validate_mode_value('@calc', 'transform')  # OK
        >>> validate_mode_value('@new', 'synthetic')  # OK
        >>> validate_mode_value('@invalid', 'transform')  # Raises ValueError
    """
    if function_name == 'transform':
        # Valid documented modes
        valid_modes = [
            '@calc', '@harmonize', '@sort', '@round', '@extract', '@deduce',
            '@analyze', '@analyse', '@knn', '@cluster'
        ]
        # Easter eggs (not documented but valid)
        easter_eggs = ['@tictactoe', '@ttt', '@sudoku']
        all_valid = valid_modes + easter_eggs
        
        if mode not in all_valid:
            # Don't mention easter eggs in error message
            raise ValueError(
                f"Parameter 'mode' for add.transform() must be one of {valid_modes}, "
                f"got '{mode}'"
            )
    
    elif function_name == 'synthetic':
        valid_modes = ['@new', '@augment']
        
        if mode not in valid_modes:
            raise ValueError(
                f"Parameter 'mode' for add.synthetic() must be one of {valid_modes}, "
                f"got '{mode}'"
            )
    
    else:
        raise ValueError(f"Unknown function name: {function_name}")


def validate_to_values(join_type='lookup', logging='default', as_type=None):
    """
    Validate parameter values for add.to() function.
    
    Parameters:
        join_type: Type of join
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        ValueError: If any parameter has invalid value
    """
    validate_join_type_value(join_type)
    validate_logging_value(logging)
    
    if as_type is not None:
        validate_as_type_value(as_type)


def validate_transform_values(mode, logging='default', as_type=None):
    """
    Validate parameter values for add.transform() function.
    
    Parameters:
        mode: Transform mode
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        ValueError: If any parameter has invalid value
    """
    validate_mode_value(mode, 'transform')
    validate_logging_value(logging)
    
    if as_type is not None:
        validate_as_type_value(as_type)


def validate_synthetic_values(mode, logging='default', as_type=None):
    """
    Validate parameter values for add.synthetic() function.
    
    Parameters:
        mode: Synthetic mode
        logging: Logging level
        as_type: Output type (optional)
    
    Raises:
        ValueError: If any parameter has invalid value
    """
    validate_mode_value(mode, 'synthetic')
    validate_logging_value(logging)
    
    if as_type is not None:
        validate_as_type_value(as_type)


def validate_required_params_to(bring_to, bring_from, bring, against):
    """
    Validate that all required parameters for add.to() are provided.
    
    Required parameters:
    - bring_to: DataFrame to bring columns to
    - bring_from: DataFrame to bring columns from
    - bring: Column(s) to bring
    - against: Key column(s) to match against
    
    Parameters:
        bring_to: Target DataFrame
        bring_from: Reference DataFrame
        bring: Column(s) to bring
        against: Key column(s)
    
    Raises:
        TypeError: If any required parameter is missing (None)
    
    Examples:
        >>> validate_required_params_to(df1, df2, 'col', 'key')  # OK
        >>> validate_required_params_to(None, df2, 'col', 'key')  # Raises TypeError
    """
    if bring_to is None:
        raise TypeError(
            "Missing required parameter 'bring_to'. "
            "add.to() requires a target DataFrame to bring columns to."
        )
    
    if bring_from is None:
        raise TypeError(
            "Missing required parameter 'bring_from'. "
            "add.to() requires a reference DataFrame to bring columns from."
        )
    
    if bring is None:
        raise TypeError(
            "Missing required parameter 'bring'. "
            "add.to() requires column name(s) to bring from the reference DataFrame."
        )
    
    if against is None:
        raise TypeError(
            "Missing required parameter 'against'. "
            "add.to() requires key column(s) to match rows between DataFrames."
        )


def validate_required_params_transform(mode, df):
    """
    Validate that all required parameters for add.transform() are provided.
    
    Required parameters:
    - mode: Transform mode (e.g., '@calc', '@round')
    - df: DataFrame to transform
    
    Parameters:
        mode: Transform mode
        df: DataFrame to transform
    
    Raises:
        TypeError: If any required parameter is missing (None)
    
    Examples:
        >>> validate_required_params_transform('@calc', df)  # OK
        >>> validate_required_params_transform(None, df)  # Raises TypeError
    """
    if mode is None:
        raise TypeError(
            "Missing required parameter 'mode'. "
            "add.transform() requires a mode string (e.g., '@calc', '@round', '@deduce')."
        )
    
    if df is None:
        raise TypeError(
            "Missing required parameter 'df'. "
            "add.transform() requires a DataFrame to transform."
        )


def validate_required_params_synthetic(mode):
    """
    Validate that all required parameters for add.synthetic() are provided.
    
    Required parameters:
    - mode: Synthetic mode (e.g., '@new', '@augment')
    
    Note: Either df or n must be provided, but this is validated separately
    in the function logic.
    
    Parameters:
        mode: Synthetic mode
    
    Raises:
        TypeError: If mode is missing (None)
    
    Examples:
        >>> validate_required_params_synthetic('@new')  # OK
        >>> validate_required_params_synthetic(None)  # Raises TypeError
    """
    if mode is None:
        raise TypeError(
            "Missing required parameter 'mode'. "
            "add.synthetic() requires a mode string (e.g., '@new', '@augment')."
        )


__all__ = [
    # Cardinality validation
    'validate_cardinality',
    'get_cardinality_type',
    
    # Position validation
    'validate_position',
    'validate_string_position',
    'normalize_position',
    
    # Synthetic validation
    'validate_synthetic_request',
    'validate_strategy_format',
    'validate_increment_conflicts',
    
    # List-only validation
    'validate_list_not_tuple',
    'validate_multiple_values',
    
    # Parameter validation
    'validate_to_params',
    'validate_transform_params',
    'validate_synthetic_params',
    
    # Type validation
    'is_dataframe',
    'validate_dataframe_type',
    'validate_string_type',
    'validate_string_or_list_type',
    'validate_int_type',
    'validate_dict_type',
    'validate_bool_or_string_type',
    'validate_to_types',
    'validate_transform_types',
    'validate_synthetic_types',
    
    # Value validation
    'validate_join_type_value',
    'validate_logging_value',
    'validate_as_type_value',
    'validate_mode_value',
    'validate_to_values',
    'validate_transform_values',
    'validate_synthetic_values',
    
    # Required parameter validation
    'validate_required_params_to',
    'validate_required_params_transform',
    'validate_required_params_synthetic',
]
