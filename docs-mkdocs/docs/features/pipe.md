# Pipe Compatibility

Some additory functions work with `df.pipe()` for fluent method chaining. Others require a different calling pattern.

---

## Which Functions Are Pipe-Friendly?

| Function | Pipe-Friendly | Reason |
|----------|:------------:|--------|
| `add.to()` | ✅ | First argument is a DataFrame (`bring_to`) |
| `add.synthetic()` | ✅ | First argument accepts a DataFrame (`df_or_mode`) |
| `add.<dynamic>()` | ✅ | First argument is a DataFrame |
| `add.transform()` | ❌ | First argument is `mode` (a string) |
| `add.scan()` | ❌ | First argument is `mode` (a string) |

---

## Pipe-Friendly Functions

### add.to()

```python
import additory as add
import polars as pl

patients = pl.DataFrame({
    'patient_id': [1, 2, 3],
    'doctor_id': [101, 102, 101],
})

doctors = pl.DataFrame({
    'doctor_id': [101, 102],
    'name': ['Dr. Sharma', 'Dr. Tanaka'],
})

# Standard call
result = add.to(patients, doctors, 'name', 'doctor_id')

# Pipe call — patients flows in as the first argument
result = patients.pipe(add.to, doctors, 'name', 'doctor_id')
```

Chain multiple lookups:

```python
departments = pl.DataFrame({
    'doctor_id': [101, 102],
    'department': ['Cardiology', 'Orthopedics'],
})

result = (
    patients
    .pipe(add.to, doctors, 'name', 'doctor_id')
    .pipe(add.to, departments, 'department', 'doctor_id')
)
```

### add.synthetic()

In augment mode, the first argument is a DataFrame:

```python
df = pl.DataFrame({'x': [1, 2, 3], 'y': [4, 5, 6]})

# Standard call
result = add.synthetic(df, n=100)

# Pipe call
result = df.pipe(add.synthetic, n=100)
```

### add.\<dynamic\>()

Dynamic functions take a DataFrame as the first argument:

```python
df = pl.DataFrame({
    'weight': [70, 85, 60],
    'height': [1.70, 1.80, 1.65],
})

# Standard call
result = add.bmi(df)

# Pipe call
result = df.pipe(add.bmi)
```

---

## Why add.transform() Is Not Pipe-Friendly

`add.transform()` takes `mode` as its first argument:

```python
# mode is the first argument, not a DataFrame
result = add.transform('@calc', df, expression='x * 2', name='x2')
```

Since `df.pipe()` passes the DataFrame as the first argument, it would collide with the `mode` parameter. Use `add.transform()` as a standalone call instead.

!!! tip "Workaround with a lambda"
    If you really want chaining, wrap the call:
    ```python
    result = (
        df
        .pipe(lambda d: add.transform('@calc', d, expression='x * 2', name='x2'))
        .pipe(lambda d: add.transform('@filter', d, where='x2 > 4'))
    )
    ```
    This works but is less readable than sequential calls.

---

## Why add.scan() Is Not Pipe-Friendly

Same reason — `mode` is the first argument:

```python
result = add.scan('@analyze', df)
```

Use it as a standalone call.

---

## Practical Pattern

Combine pipe-friendly and standalone calls in a workflow:

```python
import additory as add
import polars as pl

# Pipe-friendly chain
enriched = (
    orders
    .pipe(add.to, customers, 'name', 'customer_id')
    .pipe(add.to, products, 'price', 'product_id')
)

# Standalone transform calls
result = add.transform('@calc', enriched, expression='price * quantity', name='total')
result = add.transform('@filter', result, where='total > 100')
result = add.transform('@sort', result, by='total', strategy='desc')
```

---

## Next Steps

- [add.to()](../functions/to.md) — full lookup documentation
- [add.synthetic()](../functions/synthetic.md) — synthetic data with pipe
- [add.\<dynamic\>()](../functions/dynamic.md) — dynamic expressions with pipe
