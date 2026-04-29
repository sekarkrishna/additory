# add.transform()

Transform data within a single DataFrame — calculate new columns, filter rows, sort, aggregate, encode, impute, and more.

!!! tip "Try the Shuffle button"
    Click **🔀 Shuffle** to see every example rewritten for a different domain — healthcare, finance, retail, and more. The code pattern stays the same; only the data changes.

---

## Simple Example

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

=== "Polars"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import polars as pl

{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Calculate a new column
result = add.transform('@calc', {{target.name}},
    expression='1',
    name='row_flag',
)
print(result)">import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Calculate a new column
result = add.transform('@calc', patients,
    expression='1',
    name='row_flag',
)
print(result)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Calculate a new column
result = add.transform('@calc', {{target.name}},
    expression='1',
    name='row_flag',
)
print(result)">import additory as add
import pandas as pd

patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Calculate a new column
result = add.transform('@calc', patients,
    expression='1',
    name='row_flag',
)
print(result)</code></pre>

</div>

The first argument is always the **mode** — a string starting with `@` that tells additory what kind of transformation to perform.

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mode` | `str` | *(required)* | Transform mode (e.g., `'@calc'`, `'@filter'`, `'@sort'`) |
| `df` | `DataFrame` | *(required)* | Input DataFrame to transform |
| `columns` | `str` or `list[str]` | `None` | Column(s) to transform or select |
| `expression` | `str` or `list[str]` | `None` | Expression(s) to evaluate (for `@calc`) |
| `where` | `str` | `None` | Filter condition (for `@filter`) |
| `by` | `str` or `list[str]` | `None` | Grouping or sorting column(s) |
| `name` | `str` or `list[str]` | `None` | Output column name(s) for calculated results |
| `order` | `str` | `None` | Sort order: `'asc'` or `'desc'` |
| `position` | `str` or `int` | `'end'` | Where to place new columns |
| `strategy` | `str` or `dict` | `None` | Mode-specific options |
| `infer` | `str` or `list[str]` | `None` | Column(s) to impute (for `@deduce`) |
| `against` | `str` or `list[str]` | `None` | Text column(s) for TF-IDF similarity (for `@deduce`) |
| `method` | `str` or `list[str]` | `None` | Imputation method (for `@deduce`) |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

!!! info "Signature"
    ```
    add.transform(mode, df, columns=None, *, expression=None, where=None,
                  by=None, name=None, order=None, position='end',
                  strategy=None, infer=None, against=None, method=None,
                  logging=False, lineage=False, as_type=None)
    ```

---

## All 12 Modes

| Mode | Purpose | Key Parameters | Details |
|------|---------|----------------|---------|
| [`@calc`](../transform/calc.md) | Calculate new columns | `expression`, `name`, `strategy` | Arithmetic, string ops, built-in expressions |
| [`@filter`](../transform/filter-sort.md) | Filter rows | `where`, `columns` | SQL-like conditions |
| [`@sort`](../transform/filter-sort.md) | Sort rows | `by`, `strategy` | Ascending/descending |
| [`@aggregate`](../transform/aggregate.md) | Group and summarize | `by`, `strategy` | sum, count, average, min, max, and more |
| [`@harmonize`](../transform/harmonize.md) | Unit conversions | `columns`, `strategy` | Weight, length, temperature, and more |
| `@round` | Round numeric values | `columns`, `strategy` | Banker's rounding |
| `@transpose` | Transpose DataFrame | — | Swap rows and columns |
| [`@extract`](../transform/text.md) | Extract patterns | `columns`, `expression` | Regex and datetime extraction |
| [`@split`](../transform/text.md) | Split text columns | `columns`, `by` | Split into multiple columns |
| [`@onehot`](../transform/text.md) | One-hot encoding | `columns` | Binary indicator columns |
| [`@label`](../transform/text.md) | Label encoding | `columns` | Numeric category labels |
| [`@deduce`](../transform/deduce.md) | Impute missing values | `infer`, `method`, `against` | mean, median, knn, TF-IDF |

---

## Mode Quick Reference

### @calc — Calculate new columns

```python
# Single expression
result = add.transform('@calc', df, expression='price * quantity', name='total')

# Multiple expressions
result = add.transform('@calc', df, strategy={
    'total': 'price * quantity',
    'tax': 'total * 0.1',
})

# Built-in expression
result = add.transform('@calc', df, expression='inbuilt:bmi')
```

### @filter — Filter rows

```python
result = add.transform('@filter', df, where='age >= 18')
```

### @sort — Sort rows

```python
result = add.transform('@sort', df, by='created_at', strategy='desc')
```

### @aggregate — Group and summarize

```python
result = add.transform('@aggregate', df, by='department', strategy={
    'salary': 'average',
    'name': 'count',
})
```

### @deduce — Impute missing values

```python
# Mean imputation
result = add.transform('@deduce', df, infer='salary', method='mean')

# KNN imputation
result = add.transform('@deduce', df, infer='age', method='knn')

# TF-IDF label deduction
result = add.transform('@deduce', df, infer='category', against='description')
```

---

## Practical Scenarios

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

### Filtering and sorting

<pre><code class="language-python" data-shuffle-template="import additory as add
import polars as pl

{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Filter rows
filtered = add.transform('@filter', {{target.name}},
    where='{{target.columns.key}} is not null',
)

# Sort by key column
sorted_df = add.transform('@sort', {{target.name}},
    by='{{target.columns.id}}',
    strategy='desc',
)
print(sorted_df)">import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Filter rows
filtered = add.transform('@filter', patients,
    where='doctor_id is not null',
)

# Sort by key column
sorted_df = add.transform('@sort', patients,
    by='patient_id',
    strategy='desc',
)
print(sorted_df)</code></pre>

</div>

!!! note "Not pipe-friendly"
    `add.transform()` takes `mode` as its first argument, so it cannot be used directly with `df.pipe()`. Use it as a standalone call instead. See [Pipe Compatibility](../features/pipe.md) for details.

---

## Next Steps

- Explore each mode in detail: [@calc](../transform/calc.md), [@filter & @sort](../transform/filter-sort.md), [@aggregate](../transform/aggregate.md), [@deduce](../transform/deduce.md), [Text Modes](../transform/text.md), [@harmonize](../transform/harmonize.md)
- [Lineage Tracking](../features/lineage.md) — trace transformations
- [Logging & Timing](../features/logging.md) — measure performance
