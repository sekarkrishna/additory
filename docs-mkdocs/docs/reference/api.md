# API Reference

Complete function signatures for additory v0.1.3a11.

---

## add.to()

Bring columns from one DataFrame into another by matching on a key column.

```python
add.to(
    bring_to: Union[DataFrame, list[DataFrame]],
    bring_from: Union[DataFrame, list[DataFrame]],
    bring: Union[str, list[str]],
    against: Union[str, list[str]],
    position: Optional[Union[str, int]] = None,
    *,
    strategy: Optional[dict[str, Union[str, dict]]] = None,
    join_type: str = 'lookup',
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None,
) -> Union[DataFrame, list[DataFrame]]
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `bring_to` | `DataFrame` or `list[DataFrame]` | *(required)* | Target DataFrame(s) to add columns to |
| `bring_from` | `DataFrame` or `list[DataFrame]` | *(required)* | Reference DataFrame(s) to look up from |
| `bring` | `str` or `list[str]` | *(required)* | Column name(s) to bring from the reference |
| `against` | `str` or `list[str]` | *(required)* | Key column(s) to match on |
| `position` | `str` or `int` | `None` | Where to place new columns: `'start'`, `'end'`, `'after:col'`, `'before:col'`, or integer index |
| `strategy` | `dict` | `None` | Column-level aggregation strategies |
| `join_type` | `str` | `'lookup'` | Join type: `'lookup'`, `'left'`, `'inner'`, `'outer'` |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

**Returns:** `DataFrame` (single input) or `list[DataFrame]` (list input)

**Raises:** `ImportError`, `ValueError`, `TypeError`, `RuntimeError`

**Pipe-friendly:** ✅ — `df.pipe(add.to, ref, 'col', 'key')`

→ [Full documentation](../functions/to.md)

---

## add.transform()

Transform data within a single DataFrame.

```python
add.transform(
    mode: str,
    df: DataFrame,
    columns: Optional[Union[str, list[str]]] = None,
    *,
    expression: Optional[Union[str, list[str]]] = None,
    where: Optional[str] = None,
    by: Optional[Union[str, list[str]]] = None,
    name: Optional[Union[str, list[str]]] = None,
    order: Optional[str] = None,
    position: Union[str, int] = 'end',
    strategy: Optional[Union[str, dict[str, Any]]] = None,
    infer: Optional[Union[str, list[str]]] = None,
    against: Optional[Union[str, list[str]]] = None,
    method: Optional[Union[str, list[str]]] = None,
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None,
) -> DataFrame
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mode` | `str` | *(required)* | Transform mode (e.g., `'@calc'`, `'@filter'`, `'@sort'`) |
| `df` | `DataFrame` | *(required)* | Input DataFrame to transform |
| `columns` | `str` or `list[str]` | `None` | Column(s) to transform or select |
| `expression` | `str` or `list[str]` | `None` | Expression(s) to evaluate (for `@calc`) |
| `where` | `str` | `None` | Filter condition (for `@filter`) |
| `by` | `str` or `list[str]` | `None` | Grouping or sorting column(s) |
| `name` | `str` or `list[str]` | `None` | Output column name(s) |
| `order` | `str` | `None` | Sort order: `'asc'` or `'desc'` |
| `position` | `str` or `int` | `'end'` | Where to place new columns |
| `strategy` | `str` or `dict` | `None` | Mode-specific options |
| `infer` | `str` or `list[str]` | `None` | Column(s) to impute (for `@deduce`) |
| `against` | `str` or `list[str]` | `None` | Text column(s) for TF-IDF (for `@deduce`) |
| `method` | `str` or `list[str]` | `None` | Imputation method (for `@deduce`) |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

**Available modes:** `@calc`, `@filter`, `@sort`, `@aggregate`, `@harmonize`, `@round`, `@transpose`, `@extract`, `@onehot`, `@label`, `@deduce`, `@split`

**Returns:** `DataFrame`

**Raises:** `ImportError`, `ValueError`, `TypeError`, `RuntimeError`

**Pipe-friendly:** ❌ — first argument is `mode`, not a DataFrame

→ [Full documentation](../functions/transform.md)

---

## add.synthetic()

Generate synthetic data or augment existing DataFrames.

```python
add.synthetic(
    df_or_mode: Union[DataFrame, str],
    n: Optional[int] = None,
    *,
    strategy: Optional[dict[str, Union[str, dict]]] = None,
    seed: int = 42,
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None,
) -> DataFrame
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df_or_mode` | `DataFrame` or `str` | *(required)* | DataFrame for augment mode, or `'@new'` for new mode |
| `n` | `int` | `None` | Number of rows to generate |
| `strategy` | `dict` | `None` | Column generation specifications |
| `seed` | `int` | `42` | Random seed for reproducibility |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

**Mode inference:**

- `add.synthetic(df, n=100)` → augment mode (DataFrame as first arg)
- `add.synthetic('@new', n=100, strategy={...})` → new mode (string as first arg)

**Returns:** `DataFrame`

**Raises:** `ImportError`, `ValueError`, `TypeError`, `RuntimeError`

**Pipe-friendly:** ✅ — `df.pipe(add.synthetic, n=100)`

→ [Full documentation](../functions/synthetic.md)

---

## add.scan()

Inspect, analyze, and compare DataFrames.

```python
add.scan(
    mode: str,
    df: Optional[DataFrame] = None,
    *,
    columns: Optional[Union[str, list[str]]] = None,
    where: Optional[str] = None,
    rows: Optional[Union[str, list[str]]] = None,
    trace: Optional[str] = None,
    focus: Optional[str] = None,
    old: Optional[DataFrame] = None,
    new: Optional[DataFrame] = None,
    key: Optional[Union[str, list[str]]] = None,
    strategy: Optional[dict] = None,
    logging: bool = False,
    as_type: Optional[Literal['pandas', 'polars', 'dict', 'text']] = None,
) -> Union[DataFrame, dict, str]
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mode` | `str` | *(required)* | Scan mode: `'@analyze'`, `'@analyse'`, `'@lineage'`, `'@diff'`, `'@set'` |
| `df` | `DataFrame` | `None` | Input DataFrame (for `@analyze`, `@lineage`) |
| `columns` | `str` or `list[str]` | `None` | Column filter |
| `where` | `str` | `None` | Row filter condition |
| `rows` | `str` or `list[str]` | `None` | Row range specifications |
| `trace` | `str` | `None` | Trace mode |
| `focus` | `str` | `None` | Specialized analysis mode |
| `old` | `DataFrame` | `None` | Baseline DataFrame (for `@diff`) |
| `new` | `DataFrame` | `None` | Updated DataFrame (for `@diff`) |
| `key` | `str` or `list[str]` | `None` | Key column(s) (for `@diff`) |
| `strategy` | `dict` | `None` | Mode-specific options |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `as_type` | `str` | `None` | Output format: `'pandas'`, `'polars'`, `'dict'`, `'text'` |

**Available modes:**

| Mode | Purpose |
|------|---------|
| `@analyze` / `@analyse` | Statistical profiling and data quality |
| `@lineage` | View lineage tracking report |
| `@diff` | Compare two DataFrames |
| `@set` | Load or query expression files |

**Returns:** `DataFrame`, `dict`, or `str` depending on mode and `as_type`

**Pipe-friendly:** ❌ — first argument is `mode`, not a DataFrame

→ [Full documentation](../functions/scan.md) · [Diff documentation](../functions/diff.md)

---

## add.\<dynamic\>()

Call named expressions as functions. Any expression registered in the expression registry can be called as `add.<name>(df)`.

```python
add.<name>(
    df: DataFrame,
    *,
    name: Optional[str] = None,
    position: Optional[Union[str, int]] = None,
    logging: bool = False,
    lineage: bool = False,
    as_type: Optional[Literal['pandas', 'polars']] = None,
    **column_mapping,
) -> DataFrame
```

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df` | `DataFrame` | *(required)* | Input DataFrame |
| `name` | `str` | `None` | Output column name (defaults to expression name) |
| `position` | `str` or `int` | `None` | Where to place the new column |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type |
| `**column_mapping` | keyword args | — | Map expression variables to DataFrame columns |

**Examples:**

```python
# Auto-match columns
result = add.bmi(df)

# Explicit column mapping
result = add.bmi(df, weight='mass', height='stature')
```

**Pipe-friendly:** ✅ — `df.pipe(add.bmi)`

→ [Full documentation](../functions/dynamic.md)
