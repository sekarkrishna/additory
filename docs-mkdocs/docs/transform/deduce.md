# @deduce

Fill missing values using statistical imputation or deduce labels from text similarity. Seven imputation methods handle numeric gaps; TF-IDF handles categorical deduction from text columns.

---

## Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'name': ['Alice', 'Bob', 'Carol', 'Dave'],
    'age': [25, None, 35, None],
    'salary': [50000, 60000, None, 55000],
})

result = add.transform('@deduce', df, infer='age', method='mean')
print(result)
```

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `infer` | `str` or `list[str]` | *(required)* | Column(s) to fill |
| `method` | `str` or `list[str]` | *(required for numeric)* | Imputation method |
| `against` | `str` or `list[str]` | `None` | Text column(s) for TF-IDF similarity |
| `strategy` | `dict` | `None` | Advanced options (e.g., KNN parameters) |
| `name` | `str` | `None` | Output column name (defaults to `{infer}_infer`) |

---

## Imputation Methods

### mean

Replace missing values with the column mean:

```python
result = add.transform('@deduce', df, infer='salary', method='mean')
```

### median

Replace missing values with the column median:

```python
result = add.transform('@deduce', df, infer='salary', method='median')
```

### mode

Replace missing values with the most frequent value:

```python
result = add.transform('@deduce', df, infer='department', method='mode')
```

### forward_fill

Propagate the last valid value forward:

```python
result = add.transform('@deduce', df, infer='temperature', method='forward_fill')
```

### backward_fill

Propagate the next valid value backward:

```python
result = add.transform('@deduce', df, infer='temperature', method='backward_fill')
```

### interpolate

Linear interpolation between known values:

```python
result = add.transform('@deduce', df, infer='pressure', method='interpolate')
```

### knn

K-nearest neighbors imputation — uses similar rows to estimate missing values:

```python
result = add.transform('@deduce', df, infer='age', method='knn')
```

Configure KNN with the `strategy` parameter:

```python
result = add.transform('@deduce', df, infer='age', method='knn',
    strategy={'k': 5, 'weights': 'distance'},
)
```

| Strategy Key | Type | Default | Description |
|-------------|------|---------|-------------|
| `k` | `int` | `3` | Number of neighbors |
| `weights` | `str` | `'uniform'` | Weighting: `'uniform'` or `'distance'` |

!!! info "Pure Rust implementation"
    KNN imputation runs entirely in Rust with support for Euclidean, Manhattan, and Cosine distance metrics. No scikit-learn dependency required.

---

## TF-IDF Label Deduction

For categorical columns, use TF-IDF text similarity to deduce missing labels from related text columns:

```python
df = pl.DataFrame({
    'description': [
        'Chest pain and shortness of breath',
        'Broken arm from fall',
        'Persistent cough and wheezing',
        'Swollen ankle after running',
    ],
    'category': ['Cardiology', None, 'Pulmonology', None],
})

result = add.transform('@deduce', df, infer='category', against='description')
```

The engine builds a TF-IDF matrix from the `against` column(s), finds the most similar rows that have known labels, and assigns those labels to the missing entries.

!!! tip "Multiple text columns"
    Pass a list to `against` to combine multiple text columns for similarity:
    ```python
    result = add.transform('@deduce', df,
        infer='category',
        against=['title', 'description', 'notes'],
    )
    ```

---

## Multiple Columns

Impute several columns at once by passing lists:

```python
result = add.transform('@deduce', df,
    infer=['age', 'salary'],
    method=['mean', 'median'],
)
```

Each column is paired with its corresponding method.

---

## Practical Scenarios

### Clinical data cleanup

```python
import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1, 2, 3, 4, 5],
    'age': [45, None, 62, None, 38],
    'blood_pressure': [120, 135, None, 128, None],
    'notes': [
        'History of hypertension',
        'Elevated BP readings',
        'Cardiac monitoring',
        'Routine checkup',
        'Young healthy patient',
    ],
    'risk_level': ['High', None, 'High', None, 'Low'],
})

# Numeric imputation
result = add.transform('@deduce', patients, infer='age', method='knn')
result = add.transform('@deduce', result, infer='blood_pressure', method='mean')

# Label deduction from notes
result = add.transform('@deduce', result, infer='risk_level', against='notes')
```

### Time-series forward fill

```python
readings = pl.DataFrame({
    'timestamp': ['09:00', '09:15', '09:30', '09:45', '10:00'],
    'temperature': [22.1, None, None, 23.0, None],
})

result = add.transform('@deduce', readings, infer='temperature', method='forward_fill')
```

---

## Next Steps

- [@calc](calc.md) — calculate columns after imputation
- [@aggregate](aggregate.md) — summarize cleaned data
- [add.transform()](../functions/transform.md) — all 12 transform modes
