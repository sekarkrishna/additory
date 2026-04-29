# @calc

Calculate new columns from expressions — arithmetic, string operations, or built-in formulas from the expression library.

---

## Simple Example

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'price': [100, 200, 300],
    'quantity': [2, 3, 1],
})

result = add.transform('@calc', df, expression='price * quantity', name='total')
print(result)
```

---

## Parameters

The `@calc` mode uses these parameters from `add.transform()`:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `expression` | `str` or `list[str]` | `None` | Expression(s) to evaluate |
| `name` | `str` or `list[str]` | `None` | Output column name(s) for the result |
| `strategy` | `dict` | `None` | Map of `{name: expression}` for multiple columns |
| `position` | `str` or `int` | `'end'` | Where to place new columns |

---

## Single Expression

Use `expression` and `name` to calculate one column at a time:

```python
result = add.transform('@calc', df, expression='price * quantity', name='total')
```

Supported operators: `+`, `-`, `*`, `/`, `**` (power), `%` (modulo).

```python
# Arithmetic
result = add.transform('@calc', df, expression='salary * 1.1', name='new_salary')

# String concatenation (when columns are text)
result = add.transform('@calc', df, expression='first_name + " " + last_name', name='full_name')
```

---

## Multiple Expressions

### Using the `strategy` parameter

The `strategy` dict maps output column names to expressions. Columns are calculated in order, so later expressions can reference earlier ones:

```python
result = add.transform('@calc', df, strategy={
    'total': 'price * quantity',
    'tax': 'total * 0.1',
    'grand_total': 'total + tax',
})
```

### Using lists

Pass parallel lists of expressions and names:

```python
result = add.transform('@calc', df,
    expression=['price * quantity', 'price * 0.1'],
    name=['total', 'discount'],
)
```

!!! tip "Strategy is cleaner for chained calculations"
    When later columns depend on earlier ones (like `tax` depending on `total`), use the `strategy` dict. The list form evaluates each expression independently against the original columns.

---

## Positioning

Control where new columns appear in the result:

```python
# At the start
result = add.transform('@calc', df, expression='price * quantity',
    name='total', position='start')

# After a specific column
result = add.transform('@calc', df, expression='price * quantity',
    name='total', position='after:price')

# Before a specific column
result = add.transform('@calc', df, expression='price * quantity',
    name='total', position='before:quantity')

# At a specific index
result = add.transform('@calc', df, expression='price * quantity',
    name='total', position=2)
```

---

## Built-in Expressions

Use the `inbuilt:` prefix to reference expressions from the built-in library:

```python
# BMI from weight and height columns
result = add.transform('@calc', df, expression='inbuilt:bmi')

# ROI from revenue and cost columns
result = add.transform('@calc', df, expression='inbuilt:roi')
```

Built-in expressions auto-match column names from your DataFrame to the expression's required inputs. For example, `inbuilt:bmi` expects columns named `weight` and `height`.

!!! info "Expression catalog"
    See the [Expression Catalog](../reference/expressions.md) for the full list of built-in expressions across core, finance, medical, physics, chemistry, engineering, and statistics categories.

---

## Practical Scenarios

### Financial calculations

```python
import additory as add
import polars as pl

orders = pl.DataFrame({
    'product': ['Widget', 'Gadget', 'Gizmo'],
    'price': [25.00, 49.99, 12.50],
    'quantity': [100, 50, 200],
    'discount_rate': [0.1, 0.05, 0.15],
})

result = add.transform('@calc', orders, strategy={
    'subtotal': 'price * quantity',
    'discount': 'subtotal * discount_rate',
    'total': 'subtotal - discount',
})
```

### Using built-in medical expressions

```python
patients = pl.DataFrame({
    'name': ['Alice', 'Bob', 'Carol'],
    'weight': [70, 85, 60],
    'height': [1.70, 1.80, 1.65],
})

result = add.transform('@calc', patients, expression='inbuilt:bmi', name='bmi')
```

---

## Next Steps

- [Expression Catalog](../reference/expressions.md) — all built-in expressions
- [Expression Files](../guides/expression-files.md) — write your own `.add` files
- [add.transform()](../functions/transform.md) — all 12 transform modes
