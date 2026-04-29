# add.synthetic()

Generate synthetic DataFrames from scratch or augment existing ones with realistic additional rows.

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

# Augment with 100 more rows (pipe-friendly)
augmented = {{target.name}}.pipe(add.synthetic, n=100)
print(f'Original: {len({{target.name}})} rows')
print(f'Augmented: {len(augmented)} rows')">import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Augment with 100 more rows (pipe-friendly)
augmented = patients.pipe(add.synthetic, n=100)
print(f'Original: {len(patients)} rows')
print(f'Augmented: {len(augmented)} rows')</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Augment with 100 more rows (pipe-friendly)
augmented = {{target.name}}.pipe(add.synthetic, n=100)
print(f'Original: {len({{target.name}})} rows')
print(f'Augmented: {len(augmented)} rows')">import additory as add
import pandas as pd

patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Augment with 100 more rows (pipe-friendly)
augmented = patients.pipe(add.synthetic, n=100)
print(f'Original: {len(patients)} rows')
print(f'Augmented: {len(augmented)} rows')</code></pre>

</div>

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df_or_mode` | `DataFrame` or `str` | *(required)* | A DataFrame for augment mode, or `'@new'` for generation mode |
| `n` | `int` | `None` | Number of rows to generate |
| `strategy` | `dict` | `None` | Column generation strategies (required for `@new` mode) |
| `seed` | `int` | `42` | Random seed for reproducibility |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |

!!! info "Signature"
    ```
    add.synthetic(df_or_mode, n=None, *, strategy=None, seed=42,
                  logging=False, lineage=False, as_type=None)
    ```

---

## Two Modes

### Augment Mode — Expand an existing DataFrame

When the first argument is a DataFrame, `add.synthetic()` infers column distributions from the existing data and generates additional rows that match the statistical profile.

```python
# Augment with 200 rows
augmented = add.synthetic(df, n=200)

# Pipe-friendly version
augmented = df.pipe(add.synthetic, n=200)
```

The augmented DataFrame contains the original rows plus `n` new synthetic rows. Column types, distributions, and correlations are preserved.

### @new Mode — Create a DataFrame from scratch

When the first argument is `'@new'`, you define the column schemas via `strategy`:

```python
synthetic_df = add.synthetic('@new', n=1000, strategy={
    'age': 'normal:mean=35:std=10',
    'salary': 'lognormal:mean=10.5:std=0.5',
    'department': 'categorical',
    'employee_id': 'increment:start=1:step=1',
})
```

---

## Column Strategies for @new Mode

| Strategy | Description | Example |
|----------|-------------|---------|
| `'normal:mean=M:std=S'` | Normal distribution | `'normal:mean=35:std=10'` |
| `'lognormal:mean=M:std=S'` | Log-normal distribution | `'lognormal:mean=10.5:std=0.5'` |
| `'uniform:min=A:max=B'` | Uniform distribution | `'uniform:min=0:max=100'` |
| `'categorical'` | Random categorical values | `'categorical'` |
| `'increment:start=S:step=T'` | Sequential values | `'increment:start=1:step=1'` |
| `'increment:start=S:step=T:pattern=P'` | Formatted sequential values | `'increment:start=1:step=1:pattern=EMP-{:04d}'` |

---

## Reproducibility

The `seed` parameter ensures identical results across runs:

```python
df1 = add.synthetic('@new', n=100, strategy={'x': 'normal:mean=0:std=1'}, seed=42)
df2 = add.synthetic('@new', n=100, strategy={'x': 'normal:mean=0:std=1'}, seed=42)
# df1 and df2 are identical
```

!!! tip "Default seed"
    The default seed is `42`. Pass `seed=None` for non-deterministic generation.

---

## Pipe Compatibility

`add.synthetic()` is pipe-friendly in augment mode — the first argument receives the DataFrame:

```python
result = (
    df
    .pipe(add.synthetic, n=200)
    .pipe(add.to, reference, 'name', 'id')
)
```

!!! note "@new mode is not pipe-friendly"
    When using `'@new'` as the first argument, there is no input DataFrame to pipe from.

---

## Practical Scenarios

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

### Augmenting a small dataset

=== "Polars"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import polars as pl

# Small dataset
{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Generate 500 synthetic rows matching the distribution
augmented = {{target.name}}.pipe(add.synthetic, n=500, seed=123)
print(f'Rows: {len({{target.name}})} → {len(augmented)}')">import additory as add
import polars as pl

# Small dataset
patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Generate 500 synthetic rows matching the distribution
augmented = patients.pipe(add.synthetic, n=500, seed=123)
print(f'Rows: {len(patients)} → {len(augmented)}')</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

# Small dataset
{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Generate 500 synthetic rows matching the distribution
augmented = {{target.name}}.pipe(add.synthetic, n=500, seed=123)
print(f'Rows: {len({{target.name}})} → {len(augmented)}')">import additory as add
import pandas as pd

# Small dataset
patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Generate 500 synthetic rows matching the distribution
augmented = patients.pipe(add.synthetic, n=500, seed=123)
print(f'Rows: {len(patients)} → {len(augmented)}')</code></pre>

</div>

### Creating test data from scratch

```python
test_data = add.synthetic('@new', n=10000, strategy={
    'user_id': 'increment:start=1:step=1:pattern=USR-{:06d}',
    'age': 'normal:mean=32:std=8',
    'balance': 'lognormal:mean=8:std=1.5',
    'tier': 'categorical',
}, seed=42)
```

---

## Next Steps

- [Pipe Compatibility](../features/pipe.md) — chaining synthetic with other operations
- [Lineage Tracking](../features/lineage.md) — track synthetic data provenance
- [Type Handling](../features/type-handling.md) — pandas/polars output control
