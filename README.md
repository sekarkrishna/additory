# additory

**Elegant data operations for DataFrames**

A Rust-powered Python library for intuitive data transformations, lookups, synthetic data generation, and DataFrame comparison — with Polars and Pandas.

[![PyPI version](https://badge.fury.io/py/additory.svg)](https://badge.fury.io/py/additory)
[![Python Support](https://img.shields.io/pypi/pyversions/additory.svg)](https://pypi.org/project/additory/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Installation

```bash
pip install additory
```

**Requirements:** Python 3.9+ · Polars (required) · pyarrow (required) · Pandas (optional)

## Quick Start

```python
import additory as add
import polars as pl
```

### Bring data from another DataFrame

```python
orders = pl.DataFrame({'order_id': [1, 2], 'customer_id': [101, 102], 'amount': [250, 180]})
customers = pl.DataFrame({'customer_id': [101, 102], 'name': ['Alice', 'Bob']})

result = add.to(orders, customers, 'name', 'customer_id')
```

### Transform data

```python
df = pl.DataFrame({'price': [100, 200, 300], 'quantity': [2, 3, 1]})

result = add.transform('@calc', df, strategy={'total': 'price * quantity', 'tax': 'total * 0.1'})
result = add.transform('@filter', df, where='price > 150')
result = add.transform('@sort', df, by='price', strategy='desc')
result = add.transform('@aggregate', df, by='category', strategy={'price': 'sum'})
```

### Generate synthetic data

```python
result = add.synthetic('@new', n=1000, strategy={
    'age': 'normal(40, 10)',
    'salary': 'normal(75000, 15000)',
})

# Augment existing data
result = add.synthetic(df, n=100)
```

### Compare DataFrames

```python
diff = add.scan('@diff', old=df_jan, new=df_feb)
```

### Use dynamic expressions

```python
patients = pl.DataFrame({'weight': [70, 85], 'height': [1.70, 1.80]})
result = add.bmi(patients)
```

### Pipe compatibility

```python
result = (
    orders
    .pipe(add.to, customers, 'name', 'customer_id')
    .pipe(add.to, products, 'price', 'product_id')
)
```

## Core Functions

| Function | Purpose | Pipe-Friendly |
|----------|---------|:------------:|
| `add.to(bring_to, bring_from, bring, against, ...)` | Bring columns from another DataFrame | ✅ |
| `add.transform(mode, df, ...)` | Transform data (12 modes) | ❌ |
| `add.synthetic(df_or_mode, n, ...)` | Generate or augment data | ✅ |
| `add.scan(mode, df, ...)` | Analyze, diff, manage expressions | ❌ |
| `add.<dynamic>(df, ...)` | Named expression functions | ✅ |

## Transform Modes

`@calc` · `@filter` · `@sort` · `@aggregate` · `@harmonize` · `@round` · `@transpose` · `@extract` · `@onehot` · `@label` · `@deduce` · `@split`

## Features

- 🔗 **Intuitive Lookups** — bring columns from external sources with natural syntax
- ⚡ **12 Transform Modes** — calculate, filter, sort, aggregate, encode, impute
- 🔍 **DataFrame Diff** — compare snapshots with `add.scan('@diff')`
- 🧮 **Dynamic Expressions** — 74 built-in formulas across 7 categories, extensible via `.add` files
- 🎲 **Synthetic Data** — generate realistic test data or augment existing datasets
- 📊 **Lineage Tracking** — trace data provenance with `lineage=True`
- 🔄 **Pipe Compatibility** — fluent chaining with `df.pipe()`
- 🚀 **Rust Performance** — ~95% Rust core via PyO3
- 🐼 **Polars & Pandas** — works with both DataFrame libraries

## Documentation

📚 **[sekarkrishna.github.io/additory](https://sekarkrishna.github.io/additory)**

## Version

Current version: **0.1.3a11**

## License

MIT License — see [LICENSE](LICENSE) for details.

**Author:** Krishnamoorthy Sankaran
