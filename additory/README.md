# Additory - Unified Python API

Unified Python API for additory v0.1.3 with integrated Rust core and Python features.

## Installation

```bash
# With Rust bindings (recommended)
pip install additory[rust]

# Python-only (limited functionality)
pip install additory
```

## Quick Start

```python
import additory as add
import polars as pl

# Create DataFrame
df = pl.DataFrame({
    'weight': [70, 80, 90],
    'height': [1.75, 1.80, 1.65]
})

# Use builtin expression
result = add.transform('@calc', df, expression='inbuilt:bmi', as='bmi')

# KNN imputation
df_missing = pl.DataFrame({
    'age': [25, None, 35],
    'salary': [50000, 60000, None]
})
result = add.transform('@knn', df_missing, fetch=['age', 'salary'], strategy={'k': 2})
```

## API Reference

### transform(mode, df, **kwargs)

Transform data within DataFrame.

**Modes:**
- `@filter` - Filter rows and select columns
- `@sort` - Sort rows by column(s)
- `@transpose` - Transpose DataFrame
- `@aggregate` - Group and aggregate data
- `@split` - Split text column into multiple columns
- `@calc` - Calculate new columns from expressions
- `@extract` - Extract datetime/text components
- `@onehot` - One-hot encoding
- `@label` - Label encoding
- `@harmonize` - Unit conversions
- `@knn` - K-Nearest Neighbors imputation (Python-only)

**Parameters:**
- `mode` (str): Transform mode
- `df`: Input DataFrame (polars or pandas)
- `fetch`: Column(s) to transform/select
- `by`: Separator/grouping/sort column
- `on` or `expression`: Expression/operation/components
- `where`: Filter condition
- `as`: New name(s)/order
- `fetch_at`: Position for new columns
- `strategy`: Advanced options (dict)
- `logging`: Enable detailed logging

**Returns:**
- DataFrame (same type as input)

**Examples:**

```python
# Filter
result = add.transform('@filter', df, where='age > 18')

# Calculate with builtin expression
result = add.transform('@calc', df, expression='inbuilt:bmi', as='bmi')

# KNN imputation
result = add.transform('@knn', df, fetch=['age'], strategy={'k': 5})

# Sort
result = add.transform('@sort', df, by='date', as='desc')

# Aggregate
result = add.transform('@aggregate', df, by='category', on={'sales': 'sum'})
```

### games(name=None)

Launch game (easter egg).

**Parameters:**
- `name` (str, optional): Game name ('tictactoe' or 'sudoku')

**Examples:**

```python
# Show menu
add.games()

# Launch specific game
add.games('tictactoe')
add.games('sudoku')
```

## Architecture

```
User Code
    ↓
additory/__init__.py (Python wrapper)
    ↓
additory_rust (PyO3 module)
    ↓
Rust Transform Router
    ↓
┌─────────────┬──────────────────┐
│ Rust Modes  │  Python via PyO3 │
│ - @filter   │  - @knn          │
│ - @sort     │  - Expression    │
│ - @calc     │    resolver      │
│   (calls →) │                  │
└─────────────┴──────────────────┘
```

## Features

### Expression Resolution

Rust @calc mode can resolve namespace references:

```python
# Builtin expressions
result = add.transform('@calc', df, expression='inbuilt:bmi', as='bmi')

# Inline expressions (no resolution needed)
result = add.transform('@calc', df, expression='price * quantity', as='total')
```

### Expression Caching

Resolved expressions are cached for performance:
- First resolution: < 10ms (includes Python call)
- Cached resolution: < 0.1ms

### DataFrame Type Preservation

Input and output types are preserved:

```python
import pandas as pd

# Input: pandas DataFrame
df_pandas = pd.DataFrame({'a': [1, 2, 3]})
result = add.transform('@knn', df_pandas, fetch=['a'], strategy={'k': 2})
# Output: pandas DataFrame

# Input: polars DataFrame
df_polars = pl.DataFrame({'a': [1, 2, 3]})
result = add.transform('@knn', df_polars, fetch=['a'], strategy={'k': 2})
# Output: polars DataFrame
```

## Python-Only Modes

When Rust bindings are not available, only Python-only modes work:

- `@knn` - K-Nearest Neighbors imputation

All other modes require Rust bindings.

## Error Handling

Clear error messages with context:

```python
# Expression not found
>>> add.transform('@calc', df, expression='inbuilt:nonexistent', as='result')
RuntimeError: Failed to resolve expression 'inbuilt:nonexistent': Expression not found
Check that the expression exists in the specified namespace

# Missing parameter
>>> add.transform('@knn', df)
ValueError: fetch parameter is required for @knn mode

# Invalid DataFrame type
>>> add.transform('@knn', [1, 2, 3])
TypeError: DataFrame must be pandas or polars, got list
```

## Performance

- Expression resolution: < 1ms (cached), < 10ms (uncached)
- @knn integration overhead: < 5%
- DataFrame conversion overhead: < 5%

## Examples

See `examples/unified_api_demo.py` for comprehensive examples.

## License

MIT

## Version

0.1.3
