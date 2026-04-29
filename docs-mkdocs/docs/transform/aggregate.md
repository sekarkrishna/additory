# @aggregate

Group rows by one or more columns and summarize values using aggregation strategies — sum, count, average, and more.

---

## Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'department': ['Sales', 'HR', 'Sales', 'HR', 'Sales'],
    'salary': [60000, 55000, 70000, 50000, 65000],
    'name': ['Alice', 'Bob', 'Carol', 'Dave', 'Eve'],
})

result = add.transform('@aggregate', df, by='department', strategy={
    'salary': 'average',
    'name': 'count',
})
print(result)
```

---

## Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `by` | `str` or `list[str]` | *(required)* | Column(s) to group by |
| `strategy` | `dict` | *(required)* | Map of `{column: aggregation}` |

---

## Aggregation Strategies

| Strategy | Description | Numeric | Text |
|----------|-------------|---------|------|
| `'sum'` | Sum of values | ✅ | — |
| `'count'` | Count of values | ✅ | ✅ |
| `'average'` | Mean of values | ✅ | — |
| `'min'` | Minimum value | ✅ | ✅ |
| `'max'` | Maximum value | ✅ | ✅ |
| `'concat'` | Concatenate text values | — | ✅ |
| `'most_common'` | Most frequent value | ✅ | ✅ |
| `'least_common'` | Least frequent value | ✅ | ✅ |
| `'median'` | Median value | ✅ | — |
| `'std'` | Standard deviation | ✅ | — |
| `'variance'` | Variance | ✅ | — |
| `'unique_count'` | Count of distinct values | ✅ | ✅ |

---

## Grouping by Multiple Columns

```python
df = pl.DataFrame({
    'region': ['East', 'East', 'West', 'West'],
    'product': ['A', 'B', 'A', 'B'],
    'revenue': [100, 200, 150, 250],
})

result = add.transform('@aggregate', df, by=['region', 'product'], strategy={
    'revenue': 'sum',
})
```

---

## Multiple Aggregations

Apply different strategies to different columns:

```python
result = add.transform('@aggregate', df, by='department', strategy={
    'salary': 'average',
    'name': 'count',
    'bonus': 'sum',
})
```

---

## Practical Scenarios

### Sales summary

```python
import additory as add
import polars as pl

orders = pl.DataFrame({
    'product': ['Widget', 'Gadget', 'Widget', 'Gizmo', 'Gadget', 'Widget'],
    'amount': [25, 50, 30, 15, 45, 20],
    'customer': ['Alice', 'Bob', 'Carol', 'Alice', 'Dave', 'Bob'],
})

summary = add.transform('@aggregate', orders, by='product', strategy={
    'amount': 'sum',
    'customer': 'unique_count',
})
print(summary)
```

### Text concatenation

```python
df = pl.DataFrame({
    'team': ['A', 'A', 'B', 'B'],
    'member': ['Alice', 'Bob', 'Carol', 'Dave'],
})

result = add.transform('@aggregate', df, by='team', strategy={
    'member': 'concat',
})
# team "A" → member "Alice, Bob"
```

---

## Next Steps

- [@calc](calc.md) — calculate new columns before aggregating
- [@filter & @sort](filter-sort.md) — filter rows before grouping
- [add.transform()](../functions/transform.md) — all 12 transform modes
