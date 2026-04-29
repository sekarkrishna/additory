# @filter & @sort

Filter rows with SQL-like conditions and sort DataFrames by one or more columns.

---

## @filter

### Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'name': ['Alice', 'Bob', 'Carol', 'Dave'],
    'age': [25, 17, 30, 15],
    'department': ['Sales', 'HR', 'Sales', 'HR'],
})

result = add.transform('@filter', df, where='age >= 18')
print(result)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `where` | `str` | `None` | SQL-like filter condition |
| `columns` | `str` or `list[str]` | `None` | Select specific columns in the output |

### Filter Conditions

The `where` parameter accepts SQL-like expressions:

```python
# Comparison operators
result = add.transform('@filter', df, where='age >= 18')
result = add.transform('@filter', df, where='salary > 50000')
result = add.transform('@filter', df, where='status == "active"')

# Null checks
result = add.transform('@filter', df, where='email is not null')
result = add.transform('@filter', df, where='phone is null')

# String matching
result = add.transform('@filter', df, where='name == "Alice"')
```

### Column Selection

Use `columns` to select specific columns in the output (independent of row filtering):

```python
# Select columns only
result = add.transform('@filter', df, columns=['name', 'age'])

# Filter rows AND select columns
result = add.transform('@filter', df, where='age >= 18', columns=['name', 'department'])
```

---

## @sort

### Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'name': ['Carol', 'Alice', 'Bob'],
    'score': [85, 92, 78],
})

result = add.transform('@sort', df, by='score', strategy='desc')
print(result)
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `by` | `str` or `list[str]` | *(required)* | Column(s) to sort by |
| `strategy` | `str` | `'asc'` | Sort direction: `'asc'` (ascending) or `'desc'` (descending) |

### Ascending and Descending

```python
# Ascending (default)
result = add.transform('@sort', df, by='name')

# Descending
result = add.transform('@sort', df, by='score', strategy='desc')
```

### Multi-column Sort

Sort by multiple columns in priority order:

```python
df = pl.DataFrame({
    'department': ['Sales', 'HR', 'Sales', 'HR'],
    'name': ['Carol', 'Alice', 'Bob', 'Dave'],
    'salary': [60000, 55000, 70000, 50000],
})

result = add.transform('@sort', df, by=['department', 'salary'], strategy='asc')
```

---

## Practical Scenarios

### Filter then sort

```python
import additory as add
import polars as pl

employees = pl.DataFrame({
    'name': ['Alice', 'Bob', 'Carol', 'Dave', 'Eve'],
    'department': ['Engineering', 'Sales', 'Engineering', 'HR', 'Sales'],
    'salary': [90000, 60000, 85000, 55000, 70000],
})

# Filter to engineering, then sort by salary descending
engineers = add.transform('@filter', employees, where='department == "Engineering"')
ranked = add.transform('@sort', engineers, by='salary', strategy='desc')
print(ranked)
```

### Column selection for reporting

```python
# Get just names and departments, sorted
report = add.transform('@filter', employees, columns=['name', 'department'])
report = add.transform('@sort', report, by='name')
```

---

## Next Steps

- [@calc](calc.md) — calculate new columns
- [@aggregate](aggregate.md) — group and summarize
- [add.transform()](../functions/transform.md) — all 12 transform modes
