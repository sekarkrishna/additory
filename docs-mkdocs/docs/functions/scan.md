# add.scan()

Inspect, analyze, and explain DataFrames. `add.scan()` is the read-only counterpart to the other core functions — it never modifies data, only reports on it.

---

## Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'name': ['Alice', 'Bob', 'Charlie', None],
    'age': [28, 35, None, 42],
    'salary': [55000, 72000, 61000, 48000],
})

report = add.scan('@analyze', df)
print(report)
```

The first argument is the **mode** — a string starting with `@` that selects the scan operation.

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mode` | `str` | *(required)* | Scan mode: `'@analyze'`, `'@analyse'`, `'@lineage'`, `'@diff'`, or `'@set'` |
| `df` | `DataFrame` or `str` | `None` | Input DataFrame, or a path string for `@set` mode |
| `columns` | `str` or `list[str]` | `None` | Limit analysis to specific columns |
| `where` | `str` | `None` | SQL-like filter condition applied before analysis |
| `rows` | `str` or `list[str]` | `None` | Row range specifications |
| `trace` | `list[int]` | `None` | `[col_idx, row_idx]` for cell-level tracing |
| `focus` | `str` | `None` | Specialized analysis mode |
| `as_type` | `str` | `None` | Output format: `'pandas'`, `'polars'`, `'dict'`, or `'text'` |
| `old` | `DataFrame` | `None` | Baseline DataFrame (for `@diff` mode) |
| `new` | `DataFrame` | `None` | Updated DataFrame (for `@diff` mode) |
| `key` | `str` or `list[str]` | `None` | Key column(s) for `@diff` mode |
| `strategy` | `dict` | `None` | Options for `@diff` mode |
| `logging` | `bool` | `False` | Enable detailed operation logging |

!!! info "Signature"
    ```
    add.scan(mode, df=None, *, old=None, new=None, key=None,
             columns=None, where=None, rows=None, trace=None,
             focus=None, strategy=None, logging=False, as_type=None)
    ```

---

## Modes

| Mode | Purpose | Requires `df` | Details |
|------|---------|---------------|---------|
| `@analyze` | Statistical profiling | Yes | Data types, nulls, distributions, quality metrics |
| `@analyse` | Same as `@analyze` (British spelling) | Yes | Alias for `@analyze` |
| `@lineage` | Transformation history | Yes | Requires `lineage=True` on prior operations |
| `@diff` | Compare two DataFrames | No (`old`/`new` instead) | See [add.scan('@diff')](diff.md) |
| `@set` | Load expression files | No (path string instead) | See [add.&lt;dynamic&gt;()](dynamic.md) |

---

## @analyze / @analyse — Statistical Profiling

Produces a statistical profile of the DataFrame including data types, null counts, unique values, and distribution summaries.

```python
report = add.scan('@analyze', df)
```

### Analyzing specific columns

```python
report = add.scan('@analyze', df, columns=['age', 'salary'])
```

### Filtering rows before analysis

```python
report = add.scan('@analyze', df, where='age >= 18')
```

### Output formats

```python
# Default — returns a DataFrame
report = add.scan('@analyze', df)

# Dictionary output
report = add.scan('@analyze', df, as_type='dict')

# Text output — human-readable summary
report = add.scan('@analyze', df, as_type='text')
```

=== "Polars"

    ```python
    import additory as add
    import polars as pl

    df = pl.DataFrame({
        'name': ['Alice', 'Bob', 'Charlie', None],
        'age': [28, 35, None, 42],
        'salary': [55000.0, 72000.0, 61000.0, 48000.0],
    })

    report = add.scan('@analyze', df)
    print(report)
    ```

=== "Pandas"

    ```python
    import additory as add
    import pandas as pd

    df = pd.DataFrame({
        'name': ['Alice', 'Bob', 'Charlie', None],
        'age': [28, 35, None, 42],
        'salary': [55000.0, 72000.0, 61000.0, 48000.0],
    })

    report = add.scan('@analyze', df)
    print(report)
    ```

!!! tip "British spelling"
    `@analyse` is an alias for `@analyze` — both produce identical output. Use whichever spelling you prefer.

---

## @lineage — Transformation History

View the chain of operations that produced a DataFrame. Lineage tracking must be enabled on prior operations by passing `lineage=True`.

```python
# Enable lineage on operations
result = add.to(patients, doctors, 'name', 'doctor_id', lineage=True)
result = add.transform('@calc', result, expression='1', name='flag', lineage=True)

# View the lineage
lineage = add.scan('@lineage', result)
print(lineage)
```

### What lineage tracks

| Tracked Item | Description |
|-------------|-------------|
| Operation type | Which function was called (`add.to`, `add.transform`, `add.synthetic`) |
| Parameters | The arguments passed to each operation |
| Row counts | Rows before and after each operation |
| Column changes | Columns added or modified by each operation |
| Column sources | Which operation produced each column |

### Enabling lineage

Pass `lineage=True` to any core function:

```python
result = add.to(df, ref, 'name', 'id', lineage=True)
result = add.transform('@calc', result, expression='price * qty', name='total', lineage=True)
result = add.synthetic(result, n=100, lineage=True)
```

Lineage accumulates across operations — each step appends to the history.

!!! warning "Lineage requires prior tracking"
    Calling `add.scan('@lineage', df)` on a DataFrame without lineage metadata raises a `ValueError`. Make sure at least one prior operation used `lineage=True`.

!!! warning "Lineage and as_type are mutually exclusive"
    You cannot use `lineage=True` and `as_type` together on the same operation. Lineage metadata is stored in the DataFrame's native format and would be lost during type conversion. Convert the type separately after tracking lineage.

---

## @set — Load Expression Files

Load `.add` expression files from a folder into the expression registry, or query the current state.

### Loading expressions from a folder

```python
add.scan('@set', './my_expressions')
```

This scans the folder for `.add` files and registers all expressions found. After loading, expressions are available as dynamic functions on the `add` module.

### Viewing loaded expressions

```python
path = add.scan('@set', 'show')
print(path)
```

!!! note "@set does not accept a DataFrame"
    Unlike other scan modes, `@set` takes a path string as its second argument, not a DataFrame.

See the [Expression Files Guide](../guides/expression-files.md) for the complete `.add` file format, and [add.&lt;dynamic&gt;()](dynamic.md) for using loaded expressions.

---

## @diff — Compare Two DataFrames

The `@diff` mode compares two DataFrames row by row. It has its own dedicated page due to the depth of its feature set.

```python
diff = add.scan('@diff', old=df_january, new=df_february)
```

See [add.scan('@diff')](diff.md) for full documentation covering summary mode, detail mode, auto key detection, duplicate handling, reconciliation, and strategy options.

---

## Practical Scenarios

### Data quality check before processing

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'patient_id': [1001, 1002, 1003, 1004, 1005],
    'age': [28, None, 35, 42, None],
    'weight': [70.5, 82.0, None, 65.3, 71.0],
    'diagnosis': ['Flu', 'Fracture', 'Asthma', None, 'Flu'],
})

# Quick quality overview
report = add.scan('@analyze', df, as_type='text')
print(report)

# Focus on columns with nulls
report = add.scan('@analyze', df, columns=['age', 'weight', 'diagnosis'])
```

### Auditing a transformation pipeline

```python
import additory as add

# Build a pipeline with lineage
result = add.to(orders, products, 'product_name', 'product_id', lineage=True)
result = add.transform('@calc', result, expression='price * quantity', name='total', lineage=True)
result = add.transform('@filter', result, where='total > 100', lineage=True)

# Audit the full history
lineage = add.scan('@lineage', result)
print(lineage)
```

### Monthly reconciliation workflow

```python
import additory as add
import polars as pl

# Load this month's and last month's data
jan = pl.read_csv('data/january.csv')
feb = pl.read_csv('data/february.csv')

# Quick summary of changes
summary = add.scan('@diff', old=jan, new=feb)
print(summary)

# Detailed cell-level changes
detail = add.scan('@diff', old=jan, new=feb, strategy={'output': 'detail'})
print(detail)
```

---

## Convenience Wrappers

additory provides `add.analyze()` and `add.analyse()` as shorthand for the most common scan mode:

```python
# These are equivalent:
report = add.scan('@analyze', df)
report = add.analyze(df)

# With parameters:
report = add.analyze(df, columns=['age', 'salary'], as_type='text')
```

The convenience wrappers accept the same keyword arguments as `add.scan()` (except `mode`).

---

## Next Steps

- [add.scan('@diff')](diff.md) — detailed diff documentation
- [Lineage Tracking](../features/lineage.md) — in-depth lineage guide
- [add.&lt;dynamic&gt;()](dynamic.md) — using expressions loaded via `@set`
- [Expression Files](../guides/expression-files.md) — the `.add` file format
