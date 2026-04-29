# Transform Modes

`add.transform(mode, df, ...)` supports 12 modes for in-place DataFrame transformations. Each mode is accessed by passing a string starting with `@` as the first argument.

---

| Mode | Purpose | Key Parameters | Page |
|------|---------|----------------|------|
| `@calc` | Calculate new columns | `expression`, `name`, `strategy` | [→](calc.md) |
| `@filter` | Filter rows | `where`, `columns` | [→](filter-sort.md) |
| `@sort` | Sort rows | `by`, `strategy` | [→](filter-sort.md) |
| `@aggregate` | Group and summarize | `by`, `strategy` | [→](aggregate.md) |
| `@deduce` | Impute missing values | `infer`, `method`, `against` | [→](deduce.md) |
| `@extract` | Extract patterns / datetime | `columns`, `expression` | [→](text.md) |
| `@split` | Split text columns | `columns`, `by` | [→](text.md) |
| `@onehot` | One-hot encoding | `columns` | [→](text.md) |
| `@label` | Label encoding | `columns` | [→](text.md) |
| `@harmonize` | Unit conversions | `columns`, `strategy` | [→](harmonize.md) |
| `@round` | Round numeric values | `columns`, `strategy` | — |
| `@transpose` | Transpose DataFrame | — | — |

---

## Quick Examples

```python
import additory as add

# Calculate
result = add.transform('@calc', df, expression='price * qty', name='total')

# Filter
result = add.transform('@filter', df, where='age >= 18')

# Sort
result = add.transform('@sort', df, by='score', strategy='desc')

# Aggregate
result = add.transform('@aggregate', df, by='dept', strategy={'salary': 'average'})

# Impute
result = add.transform('@deduce', df, infer='age', method='mean')

# Encode
result = add.transform('@onehot', df, columns='color')
```

→ [add.transform() reference](../functions/transform.md)
