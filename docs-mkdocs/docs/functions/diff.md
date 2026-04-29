# add.scan('@diff')

Compare two DataFrames row by row — detect additions, deletions, modifications, and duplicates. The diff engine auto-detects key columns, classifies every row, and produces either a compact summary or a granular detail report.

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

old_{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

new_{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', 'Updated', '{{target.rows[2].value1}}'],
})

diff = add.scan('@diff', old=old_{{target.name}}, new=new_{{target.name}})
print(diff)">import additory as add
import polars as pl

old_patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

new_patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Updated', 'Asthma'],
})

diff = add.scan('@diff', old=old_patients, new=new_patients)
print(diff)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

old_{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

new_{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', 'Updated', '{{target.rows[2].value1}}'],
})

diff = add.scan('@diff', old=old_{{target.name}}, new=new_{{target.name}})
print(diff)">import additory as add
import pandas as pd

old_patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

new_patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Updated', 'Asthma'],
})

diff = add.scan('@diff', old=old_patients, new=new_patients)
print(diff)</code></pre>

</div>

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `mode` | `str` | *(required)* | Must be `'@diff'` |
| `old` | `DataFrame` | *(required)* | The baseline (previous) DataFrame |
| `new` | `DataFrame` | *(required)* | The updated (current) DataFrame |
| `key` | `str` or `list[str]` | `None` | Key column(s) for row matching. Auto-detected if omitted. |
| `columns` | `str` or `list[str]` | `None` | Limit comparison to specific columns |
| `strategy` | `dict` | `None` | Diff options: `output`, `exclude`, `carry`, `aliases`, `groups` |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

!!! info "Signature"
    ```
    add.scan('@diff', old=df_old, new=df_new, *, key=None,
             columns=None, strategy=None, logging=False, as_type=None)
    ```

---

## Output Modes

### Summary Mode (default)

Summary mode produces one row per original row with inline change markers and a `_diff_status` column:

```python
diff = add.scan('@diff', old=df_old, new=df_new)
```

Each row gets a `_diff_status` value:

| Status | Meaning |
|--------|---------|
| `added` | Row exists only in `new` |
| `removed` | Row exists only in `old` |
| `modified` | Row exists in both but values changed |
| `no_change` | Row is identical in both |

For modified rows, changed cells show inline markers: `"old_value → new_value"`.

### Detail Mode

Detail mode produces one row per changed cell, giving granular visibility into every modification:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'output': 'detail'},
)
```

Detail output columns:

| Column | Description |
|--------|-------------|
| `_key` | The key value identifying the row |
| `_column` | The column that changed |
| `_old_value` | Value in the old DataFrame |
| `_new_value` | Value in the new DataFrame |

---

## Auto Key Detection

When `key` is not specified, the diff engine automatically detects the best key column(s) by looking for:

1. Columns with all unique values (candidate primary keys)
2. Columns with names suggesting identity (`*_id`, `*_key`, `*_code`)
3. The column with the highest cardinality ratio

```python
# Auto-detect key
diff = add.scan('@diff', old=df_old, new=df_new)

# Explicit key
diff = add.scan('@diff', old=df_old, new=df_new, key='employee_id')

# Composite key
diff = add.scan('@diff', old=df_old, new=df_new, key=['region', 'product_id'])
```

!!! tip "Explicit keys are faster"
    Auto-detection works well for most datasets, but specifying `key` explicitly skips the detection step and avoids ambiguity.

---

## Duplicate Handling

When the key column contains duplicate values, the diff engine flags them rather than producing incorrect comparisons:

```python
# If old_df has duplicate keys, they appear in the output
# with _diff_status = 'duplicate'
diff = add.scan('@diff', old=df_old, new=df_new)
```

Duplicates are reported separately so you can clean the data before re-running the diff.

---

## Strategy Options

The `strategy` dictionary controls diff behaviour:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `output` | `str` | `'summary'` | Output mode: `'summary'` or `'detail'` |
| `exclude` | `list[str]` | `[]` | Columns to ignore during comparison |
| `carry` | `list[str]` | `[]` | Extra columns to include in output (not compared) |
| `aliases` | `dict` or `str` | `None` | Column value aliases for case-insensitive matching |
| `groups` | `str` | `None` | Reconciliation group name for hierarchical comparison |

### Excluding columns

Skip volatile columns (timestamps, auto-generated IDs) that would create noise:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'exclude': ['updated_at', 'row_hash']},
)
```

### Carrying extra columns

Include columns in the output for context without comparing them:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'carry': ['created_by', 'notes']},
)
```

---

## Reconciliation

For advanced comparisons, use reconciliation `.add` files to define aliases and groups.

### Aliases — Case-insensitive variant matching

Aliases map variant spellings to a canonical form before comparison:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={
        'aliases': {
            'status': ['Active', 'ACTIVE', 'active'],
            'region': ['NA', 'North America'],
        },
    },
)
```

Or load aliases from a registered reconciliation file:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'aliases': 'my_reconciliation'},
)
```

### Groups — Hierarchical change detection

Groups define column hierarchies for structured change detection. Load them from a reconciliation `.add` file:

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'groups': 'my_reconciliation'},
)
```

See the [Reconciliation Guide](../guides/reconciliation.md) for the full `.add` file format.

---

## Practical Scenarios

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

### Monthly data comparison

=== "Polars"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import polars as pl

# January snapshot
jan_{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
})

# February snapshot — one value changed, one row added
feb_{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}, 9999],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', 'Changed', '{{target.rows[2].value1}}', 'New Entry'],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}, {{target.rows[0].key}}],
})

# Summary diff
summary = add.scan('@diff', old=jan_{{target.name}}, new=feb_{{target.name}})
print(summary)

# Detail diff
detail = add.scan('@diff', old=jan_{{target.name}}, new=feb_{{target.name}},
    strategy={'output': 'detail'},
)
print(detail)">import additory as add
import polars as pl

# January snapshot
jan_patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
    'doctor_id': [201, 202, 203],
})

# February snapshot — one value changed, one row added
feb_patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003, 9999],
    'diagnosis': ['Hypertension', 'Changed', 'Asthma', 'New Entry'],
    'doctor_id': [201, 202, 203, 201],
})

# Summary diff
summary = add.scan('@diff', old=jan_patients, new=feb_patients)
print(summary)

# Detail diff
detail = add.scan('@diff', old=jan_patients, new=feb_patients,
    strategy={'output': 'detail'},
)
print(detail)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

# January snapshot
jan_{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
})

# February snapshot — one value changed, one row added
feb_{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}, 9999],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', 'Changed', '{{target.rows[2].value1}}', 'New Entry'],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}, {{target.rows[0].key}}],
})

# Summary diff
summary = add.scan('@diff', old=jan_{{target.name}}, new=feb_{{target.name}})
print(summary)

# Detail diff
detail = add.scan('@diff', old=jan_{{target.name}}, new=feb_{{target.name}},
    strategy={'output': 'detail'},
)
print(detail)">import additory as add
import pandas as pd

# January snapshot
jan_patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
    'doctor_id': [201, 202, 203],
})

# February snapshot — one value changed, one row added
feb_patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003, 9999],
    'diagnosis': ['Hypertension', 'Changed', 'Asthma', 'New Entry'],
    'doctor_id': [201, 202, 203, 201],
})

# Summary diff
summary = add.scan('@diff', old=jan_patients, new=feb_patients)
print(summary)

# Detail diff
detail = add.scan('@diff', old=jan_patients, new=feb_patients,
    strategy={'output': 'detail'},
)
print(detail)</code></pre>

</div>

### Excluding noisy columns

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={
        'exclude': ['last_modified', 'etag'],
        'carry': ['created_by'],
    },
)
```

!!! warning "Key column cannot be excluded"
    Columns used as the key cannot appear in the `exclude` list. The diff engine will raise an error if you try.

---

## Next Steps

- [Reconciliation Guide](../guides/reconciliation.md) — aliases and groups in `.add` files
- [add.scan()](scan.md) — other scan modes (`@analyze`, `@lineage`)
- [Expression Files](../guides/expression-files.md) — the `.add` file format
