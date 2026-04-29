# Text Modes

Four transform modes for working with text and categorical data: extract patterns, split columns, one-hot encode, and label encode.

---

## @extract

Pull patterns or datetime components out of text columns.

### Pattern Extraction

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'email': ['alice@example.com', 'bob@company.org', 'carol@test.net'],
})

result = add.transform('@extract', df, columns='email', expression=r'@(\w+)')
print(result)
```

### Datetime Extraction

Extract year, month, day, or other components from datetime columns:

```python
df = pl.DataFrame({
    'created_at': ['2026-01-15', '2026-03-22', '2026-07-04'],
})

result = add.transform('@extract', df, columns='created_at', expression='year')
```

Available datetime components: `year`, `month`, `day`, `hour`, `minute`, `second`, `weekday`.

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `columns` | `str` or `list[str]` | Column(s) to extract from |
| `expression` | `str` | Regex pattern or datetime component name |

---

## @split

Split a text column into multiple columns based on a separator.

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'full_name': ['Alice Smith', 'Bob Jones', 'Carol Lee'],
})

result = add.transform('@split', df, columns='full_name', by=' ')
print(result)
```

This creates columns `full_name_0`, `full_name_1`, etc.

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `columns` | `str` or `list[str]` | *(required)* | Column(s) to split |
| `by` | `str` | *(required)* | Separator string |

---

## @onehot

One-hot encode categorical columns into binary indicator columns.

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'color': ['red', 'blue', 'green', 'red', 'blue'],
    'size': ['S', 'M', 'L', 'M', 'S'],
})

result = add.transform('@onehot', df, columns='color')
print(result)
```

This creates columns like `color_red`, `color_blue`, `color_green` with values `1` or `0`.

### Multiple Columns

```python
result = add.transform('@onehot', df, columns=['color', 'size'])
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `columns` | `str` or `list[str]` | Column(s) to one-hot encode |

---

## @label

Label encode categorical columns into numeric values.

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'status': ['active', 'inactive', 'pending', 'active', 'inactive'],
})

result = add.transform('@label', df, columns='status')
print(result)
```

Each unique value gets a numeric label (e.g., `active` → 0, `inactive` → 1, `pending` → 2). The original column is preserved and a new `{column}_label` column is added.

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `columns` | `str` or `list[str]` | Column(s) to label encode |

---

## Practical Scenarios

### Preparing text data for analysis

```python
import additory as add
import polars as pl

survey = pl.DataFrame({
    'response': ['Very Satisfied', 'Satisfied', 'Neutral', 'Dissatisfied', 'Very Satisfied'],
    'comment': ['Great service at NYC office', 'Good experience in LA', 'OK visit to CHI branch'],
})

# Label encode the ordinal response
result = add.transform('@label', survey, columns='response')

# Extract city abbreviations from comments
result = add.transform('@extract', result, columns='comment', expression=r'\b[A-Z]{2,3}\b')
```

### Splitting and encoding

```python
products = pl.DataFrame({
    'sku': ['ELEC-001-US', 'FOOD-042-UK', 'ELEC-007-DE'],
    'category': ['Electronics', 'Food', 'Electronics'],
})

# Split SKU into components
result = add.transform('@split', products, columns='sku', by='-')

# One-hot encode category
result = add.transform('@onehot', result, columns='category')
```

---

## Next Steps

- [@deduce](deduce.md) — impute missing values after encoding
- [@calc](calc.md) — calculate new columns from encoded data
- [add.transform()](../functions/transform.md) — all 12 transform modes
