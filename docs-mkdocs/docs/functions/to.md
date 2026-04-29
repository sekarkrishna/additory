# add.to()

Bring columns from one DataFrame into another by matching on a key column — the core lookup operation in additory.

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

{{reference.name}} = pl.DataFrame({
    '{{reference.columns.key}}': [{{reference.rows[0].key}}, {{reference.rows[1].key}}, {{reference.rows[2].key}}],
    '{{reference.columns.lookup1}}': ['{{reference.rows[0].lookup1}}', '{{reference.rows[1].lookup1}}', '{{reference.rows[2].lookup1}}'],
    '{{reference.columns.lookup2}}': ['{{reference.rows[0].lookup2}}', '{{reference.rows[1].lookup2}}', '{{reference.rows[2].lookup2}}'],
})

result = add.to({{target.name}}, {{reference.name}}, '{{reference.columns.lookup1}}', '{{target.columns.key}}')
print(result)">import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

doctors = pl.DataFrame({
    'doctor_id': [201, 202, 203],
    'name': ['Dr. Priya Sharma', 'Dr. Kenji Tanaka', 'Dr. Amara Osei'],
    'specialty': ['Cardiology', 'Orthopedics', 'Pulmonology'],
})

result = add.to(patients, doctors, 'name', 'doctor_id')
print(result)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

{{reference.name}} = pd.DataFrame({
    '{{reference.columns.key}}': [{{reference.rows[0].key}}, {{reference.rows[1].key}}, {{reference.rows[2].key}}],
    '{{reference.columns.lookup1}}': ['{{reference.rows[0].lookup1}}', '{{reference.rows[1].lookup1}}', '{{reference.rows[2].lookup1}}'],
    '{{reference.columns.lookup2}}': ['{{reference.rows[0].lookup2}}', '{{reference.rows[1].lookup2}}', '{{reference.rows[2].lookup2}}'],
})

result = add.to({{target.name}}, {{reference.name}}, '{{reference.columns.lookup1}}', '{{target.columns.key}}')
print(result)">import additory as add
import pandas as pd

patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

doctors = pd.DataFrame({
    'doctor_id': [201, 202, 203],
    'name': ['Dr. Priya Sharma', 'Dr. Kenji Tanaka', 'Dr. Amara Osei'],
    'specialty': ['Cardiology', 'Orthopedics', 'Pulmonology'],
})

result = add.to(patients, doctors, 'name', 'doctor_id')
print(result)</code></pre>

</div>

The four positional arguments map to a natural sentence: bring **to** this target, **from** this reference, **bring** these columns, matched **against** this key.

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `bring_to` | `DataFrame` or `list[DataFrame]` | *(required)* | Target DataFrame(s) to add columns to |
| `bring_from` | `DataFrame` or `list[DataFrame]` | *(required)* | Reference DataFrame(s) to look up from |
| `bring` | `str` or `list[str]` | *(required)* | Column name(s) to bring from the reference |
| `against` | `str` or `list[str]` | *(required)* | Key column(s) to match on |
| `position` | `str` or `int` | `None` | Where to place new columns: `'start'`, `'end'`, `'after:col'`, `'before:col'`, or integer index |
| `strategy` | `dict` | `None` | Column-level aggregation strategies (e.g., `{'amount': 'sum'}`) |
| `join_type` | `str` | `'lookup'` | Join type: `'lookup'`, `'left'`, `'inner'`, `'outer'` |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

!!! info "Signature"
    ```
    add.to(bring_to, bring_from, bring, against, position=None, *,
           strategy=None, join_type='lookup', logging=False,
           lineage=False, as_type=None)
    ```

---

## Join Types

The `join_type` parameter controls how rows are matched:

| Join Type | Behaviour |
|-----------|-----------|
| `'lookup'` | Default. Brings columns from the reference where keys match. Unmatched rows in the target keep `null` values. |
| `'left'` | Standard left join — all target rows preserved. |
| `'inner'` | Only rows with matching keys in both DataFrames. |
| `'outer'` | All rows from both DataFrames, with `null` where no match exists. |

```python
# Inner join — only matched rows
result = add.to(patients, doctors, 'name', 'doctor_id', join_type='inner')
```

---

## Aggregation Strategies

When the reference has multiple rows per key, use `strategy` to control how values are combined:

```python
result = add.to(
    customers,
    orders,
    'amount',
    'customer_id',
    strategy={'amount': 'sum'},
)
```

Available aggregation strategies:

| Strategy | Description |
|----------|-------------|
| `'sum'` | Sum of values |
| `'count'` | Count of values |
| `'average'` | Mean of values |
| `'min'` | Minimum value |
| `'max'` | Maximum value |
| `'concat'` | Concatenate text values |
| `'most_common'` | Most frequent value |
| `'least_common'` | Least frequent value |
| `'median'` | Median value |
| `'std'` | Standard deviation |
| `'variance'` | Variance |
| `'unique_count'` | Count of distinct values |

---

## List-of-DataFrames Patterns

`add.to()` supports passing lists of DataFrames for batch operations:

```python
# One-to-many: single target, multiple references
result = add.to(
    customers,
    [orders_jan, orders_feb, orders_mar],
    'amount',
    'customer_id',
    strategy={'amount': 'sum'},
)

# Many-to-one: multiple targets, single reference
results = add.to(
    [customers_a, customers_b],
    products,
    'product_name',
    'product_id',
)
# results is a list of DataFrames
```

When `bring_from` is a list, the references are concatenated vertically before the lookup. When `bring_to` is a list, each target is processed independently and a list of results is returned.

---

## Column Positioning

Control where new columns appear in the result:

```python
# Place at the start
result = add.to(df, ref, 'name', 'id', position='start')

# Place after a specific column
result = add.to(df, ref, 'name', 'id', position='after:email')

# Place before a specific column
result = add.to(df, ref, 'name', 'id', position='before:status')

# Place at a specific index
result = add.to(df, ref, 'name', 'id', position=2)
```

---

## Pipe Compatibility

`add.to()` is pipe-friendly — the first argument (`bring_to`) receives the DataFrame:

```python
result = patients.pipe(add.to, doctors, 'name', 'doctor_id')
```

This works with method chaining:

```python
result = (
    patients
    .pipe(add.to, doctors, 'name', 'doctor_id')
    .pipe(add.to, departments, 'dept_name', 'dept_id')
)
```

---

## Practical Scenarios

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

### Bringing multiple columns

=== "Polars"

    <pre><code class="language-python" data-shuffle-template="result = add.to(
    {{target.name}},
    {{reference.name}},
    ['{{reference.columns.lookup1}}', '{{reference.columns.lookup2}}'],
    '{{target.columns.key}}',
)
print(result)">result = add.to(
    patients,
    doctors,
    ['name', 'specialty'],
    'doctor_id',
)
print(result)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="result = add.to(
    {{target.name}},
    {{reference.name}},
    ['{{reference.columns.lookup1}}', '{{reference.columns.lookup2}}'],
    '{{target.columns.key}}',
)
print(result)">result = add.to(
    patients,
    doctors,
    ['name', 'specialty'],
    'doctor_id',
)
print(result)</code></pre>

### Using keyword arguments

<pre><code class="language-python" data-shuffle-template="result = add.to(
    bring_to={{target.name}},
    bring_from={{reference.name}},
    bring='{{reference.columns.lookup1}}',
    against='{{target.columns.key}}',
    position='start',
)">result = add.to(
    bring_to=patients,
    bring_from=doctors,
    bring='name',
    against='doctor_id',
    position='start',
)</code></pre>

</div>

### Lineage tracking

```python
result = add.to(df, ref, 'name', 'id', lineage=True)

# View the lineage
lineage_report = add.scan('@lineage', result)
```

!!! warning "Lineage and as_type"
    `lineage=True` and `as_type` cannot be used together. Lineage metadata is stored in the DataFrame's native format and would be lost during type conversion.

---

## Next Steps

- [Pipe Compatibility](../features/pipe.md) — fluent chaining patterns
- [Lineage Tracking](../features/lineage.md) — trace where columns came from
- [Type Handling](../features/type-handling.md) — pandas/polars interop
