# add.&lt;dynamic&gt;()

Call named expressions directly on the `add` module — `add.bmi(df)`, `add.revenue(df)`, `add.margin(df)`. Expressions are loaded from `.add` files and resolved at runtime via Python's `__getattr__` mechanism.

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

# Load expressions from a folder
add.scan('@set', './my_expressions')

{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Call an expression by name — columns are auto-matched
result = add.my_expression({{target.name}})
print(result)">import additory as add
import polars as pl

# Load expressions from a folder
add.scan('@set', './my_expressions')

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Call an expression by name — columns are auto-matched
result = add.my_expression(patients)
print(result)</code></pre>

=== "Pandas"

    <pre><code class="language-python" data-shuffle-template="import additory as add
import pandas as pd

# Load expressions from a folder
add.scan('@set', './my_expressions')

{{target.name}} = pd.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Call an expression by name — columns are auto-matched
result = add.my_expression({{target.name}})
print(result)">import additory as add
import pandas as pd

# Load expressions from a folder
add.scan('@set', './my_expressions')

patients = pd.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Call an expression by name — columns are auto-matched
result = add.my_expression(patients)
print(result)</code></pre>

</div>

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `df` | `DataFrame` | *(required)* | Input DataFrame |
| `position` | `str` or `int` | `None` | Where to place the new column |
| `logging` | `bool` | `False` | Enable detailed operation logging |
| `lineage` | `bool` | `False` | Enable lineage tracking |
| `as_type` | `str` | `None` | Force output type: `'pandas'` or `'polars'` |
| `**column_mappings` | `kwargs` | — | Explicit column name mappings |

!!! info "Signature"
    ```
    add.<name>(df, *, position=None, logging=False, lineage=False,
               as_type=None, **column_mappings)
    ```

---

## How It Works — `__getattr__` Resolution

When you call `add.bmi(df)`, Python's attribute lookup triggers `__getattr__('bmi')` on the additory module. The resolution chain:

1. Check if `bmi` is a reserved name (built-in function) — if so, return the built-in
2. Look up `bmi` in the expression registry
3. If found, create a dynamic function that applies the expression to the DataFrame
4. If not found, raise `AttributeError`

```python
# These are equivalent:
result = add.bmi(df)
result = add.transform('@calc', df, expression='weight / (height ** 2)', name='bmi')
```

The dynamic API is syntactic sugar over `add.transform('@calc', ...)`.

---

## .add File Format

Expressions are defined in `.add` files using TOML syntax. Each expression is a top-level table:

```toml
[bmi]
expression = "weight / (height ** 2)"
category = "medical"

[revenue]
expression = "price * quantity"
category = "finance"

[margin]
expression = "(revenue - cost) / revenue * 100"
category = "finance"
```

| Field | Required | Description |
|-------|----------|-------------|
| `expression` | Yes | The calculation formula |
| `category` | No | Grouping category for organization |

Save this as `my_expressions.add` in a folder, then load it:

```python
add.scan('@set', './expressions_folder')
```

See the [Expression Files Guide](../guides/expression-files.md) for the complete format reference.

---

## Column Auto-Matching

When you call `add.bmi(df)`, additory inspects the expression `weight / (height ** 2)` and looks for columns named `weight` and `height` in the DataFrame. If they exist, they are used automatically.

```python
df = pl.DataFrame({
    'weight': [70, 80, 65],
    'height': [1.75, 1.80, 1.60],
})

# Columns match the expression variables — auto-matched
result = add.bmi(df)
```

---

## Explicit Column Mapping

When DataFrame columns have different names, use keyword arguments to map them:

```python
df = pl.DataFrame({
    'body_weight': [70, 80, 65],
    'body_height': [1.75, 1.80, 1.60],
})

# Map expression variables to actual column names
result = add.bmi(df, weight='body_weight', height='body_height')
```

---

## Loading Expressions at Runtime

### Loading from a folder

```python
# Load all .add files from a folder
add.scan('@set', './my_expressions')
```

This scans the folder for `.add` files and registers all expressions found.

### Viewing loaded expressions

```python
# Show all registered expressions
add.scan('@set', 'show')
```

---

## Reserved Names

The following names are reserved for built-in functions and cannot be used as expression names:

| Reserved Name | Built-in Function |
|---------------|-------------------|
| `to` | `add.to()` |
| `synthetic` | `add.synthetic()` |
| `scan` | `add.scan()` |
| `transform` | `add.transform()` |
| `harmonize` | `add.transform('@harmonize', ...)` |

If an expression file defines a name that conflicts with a reserved name, the built-in function takes precedence.

!!! warning "Name collisions"
    If you name an expression `to` or `scan`, it will be silently ignored. Choose descriptive names like `calculate_bmi` or `compute_revenue` to avoid conflicts.

---

## Pipe Compatibility

Dynamic expressions are pipe-friendly — the first argument (`df`) receives the DataFrame:

```python
result = df.pipe(add.bmi)

# With explicit mapping
result = df.pipe(add.bmi, weight='body_weight', height='body_height')

# Chaining
result = (
    df
    .pipe(add.bmi)
    .pipe(add.revenue)
    .pipe(add.to, reference, 'name', 'id')
)
```

---

## Practical Scenarios

<div class="shuffle-container" markdown>

<button class="shuffle-btn md-button">🔀 Shuffle</button>
<span class="shuffle-domain"></span>

### Using built-in expressions

<pre><code class="language-python" data-shuffle-template="import additory as add
import polars as pl

{{target.name}} = pl.DataFrame({
    '{{target.columns.id}}': [{{target.rows[0].id}}, {{target.rows[1].id}}, {{target.rows[2].id}}],
    '{{target.columns.key}}': [{{target.rows[0].key}}, {{target.rows[1].key}}, {{target.rows[2].key}}],
    '{{target.columns.value1}}': ['{{target.rows[0].value1}}', '{{target.rows[1].value1}}', '{{target.rows[2].value1}}'],
})

# Load custom expressions
add.scan('@set', './my_expressions')

# Apply and chain
result = add.my_expression({{target.name}})
print(result)">import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1001, 1002, 1003],
    'doctor_id': [201, 202, 203],
    'diagnosis': ['Hypertension', 'Fracture', 'Asthma'],
})

# Load custom expressions
add.scan('@set', './my_expressions')

# Apply and chain
result = add.my_expression(patients)
print(result)</code></pre>

</div>

### Building a domain-specific library

Create a folder of `.add` files for your domain:

```
expressions/
  medical.add      # bmi, bsa, egfr, ...
  finance.add      # revenue, margin, roi, ...
  logistics.add    # delivery_time, cost_per_kg, ...
```

```python
import additory as add

# Load all domain expressions
add.scan('@set', './expressions')

# Now use them naturally
result = add.bmi(patient_df)
result = add.revenue(sales_df)
result = add.cost_per_kg(shipment_df)
```

### Lineage tracking with dynamic expressions

```python
result = add.bmi(df, lineage=True)
lineage = add.scan('@lineage', result)
```

---

## Next Steps

- [Expression Files Guide](../guides/expression-files.md) — complete `.add` file format
- [Pipe Compatibility](../features/pipe.md) — chaining dynamic expressions
- [API Reference](../reference/api.md) — full signature details
