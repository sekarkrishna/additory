# Quickstart

Get up and running with additory in 5 minutes. This walkthrough covers the five core operations: lookup, transform, synthetic data, scanning, and dynamic expressions.

!!! tip "Try the Shuffle button"
    Click **🔀 Shuffle** to see every example rewritten for a different domain — healthcare, finance, retail, and more. The code pattern stays the same; only the data changes.

## Setup

```python
import additory as add
import polars as pl
```

---

## 1. add.to() — Look up data from another table

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

Bring columns from a reference DataFrame into your target DataFrame by matching on a key column.

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

The four positional arguments are: `bring_to`, `bring_from`, `bring`, `against`. You can also use keyword arguments:

```python
result = add.to(
    bring_to=patients,
    bring_from=doctors,
    bring='name',
    against='doctor_id',
)
```

Or pipe it:

```python
result = patients.pipe(add.to, doctors, 'name', 'doctor_id')
```

[Full add.to() documentation :material-arrow-right:](../functions/to.md)

---

## 2. add.transform() — Calculate, filter, sort, and more

Transform data within a single DataFrame. The first argument is the mode.

```python
# Calculate a new column
result = add.transform('@calc', df, expression='price * quantity', name='total')

# Filter rows
result = add.transform('@filter', df, where='age >= 18')

# Sort
result = add.transform('@sort', df, by='created_at', strategy='desc')

# Aggregate
result = add.transform('@aggregate', df, by='department', strategy={'salary': 'average'})
```

There are 12 modes in total: `@calc`, `@filter`, `@sort`, `@aggregate`, `@harmonize`, `@round`, `@transpose`, `@extract`, `@onehot`, `@label`, `@deduce`, `@split`.

[Full add.transform() documentation :material-arrow-right:](../functions/transform.md)

---

## 3. add.synthetic() — Generate data

Create synthetic DataFrames from scratch or augment existing ones.

```python
# Create a brand-new DataFrame
synthetic_df = add.synthetic('@new', n=1000, strategy={
    'age': 'normal:mean=35:std=10',
    'salary': 'lognormal:mean=10.5:std=0.5',
    'department': 'categorical',
})

# Augment an existing DataFrame with 200 more rows (pipe-friendly)
augmented = df.pipe(add.synthetic, n=200)
```

[Full add.synthetic() documentation :material-arrow-right:](../functions/synthetic.md)

---

## 4. add.scan() — Analyze and compare

### Analyze data quality

```python
report = add.scan('@analyze', df)
```

### Compare two DataFrames with @diff

```python
diff_result = add.scan('@diff', old=df_january, new=df_february)
```

`@diff` auto-detects the key column, classifies every row as added, removed, modified, or unchanged, and returns a summary with inline `"old → new"` change markers.

[Full add.scan() documentation :material-arrow-right:](../functions/scan.md) ·
[Diff documentation :material-arrow-right:](../functions/diff.md)

---

## 5. add.&lt;dynamic&gt;() — Named expressions

Load expression files and call them by name directly on the `add` module.

```python
# Load expressions from a folder
add.scan('@set', './my_expressions')

# Call an expression by name — columns are auto-matched
result = add.bmi(df)

# Explicit column mapping
result = add.bmi(df, weight='body_weight', height='body_height')
```

Expression files use a TOML-based `.add` format:

```toml
[bmi]
expression = "weight / (height ** 2)"
category = "medical"
```

[Full dynamic API documentation :material-arrow-right:](../functions/dynamic.md)

---

## Next steps

- Explore [Transform Modes](../transform/index.md) for the full power of `add.transform()`
- Learn about [Pipe Compatibility](../features/pipe.md) for fluent DataFrame workflows
- Set up [Expression Files](../guides/expression-files.md) for reusable calculations
- Browse the [API Reference](../reference/api.md) for complete function signatures
