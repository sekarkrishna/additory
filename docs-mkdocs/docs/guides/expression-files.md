# Expression Files

Expression files (`.add` files) define named formulas in TOML format. They power the dynamic API (`add.<name>(df)`) and the `inbuilt:` prefix in `@calc` mode.

---

## The .add File Format

Each `.add` file is a TOML file where every top-level table defines one expression:

```toml
[bmi]
expression = "weight / (height ** 2)"
description = "Body Mass Index - weight in kg, height in meters"
category = "medical"

[roi]
expression = "((revenue - cost) / cost) * 100"
description = "Return on Investment percentage"
category = "finance"
```

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `expression` | string | The formula to evaluate. Column names become variables. |
| `description` | string | Human-readable description of the expression |
| `category` | string | Category for organization (e.g., `"finance"`, `"medical"`) |

### Optional Input Definitions

Add an `[expression_name.inputs]` table to document expected columns:

```toml
[bmi]
expression = "weight / (height ** 2)"
description = "Body Mass Index"
category = "medical"

[bmi.inputs]
weight = { type = "numeric", unit = "kg", description = "Body weight" }
height = { type = "numeric", unit = "m", description = "Height" }
```

Input definitions are used for documentation and validation. They don't change how the expression is evaluated.

---

## Loading Expression Files

### From a folder

Use `add.scan('@set', folder_path)` to load all `.add` files from a directory:

```python
import additory as add

# Load expressions from a folder
add.scan('@set', '/path/to/my/expressions/')
```

All `.add` files in the folder are parsed and registered. After loading, expressions are available via the dynamic API:

```python
result = add.bmi(df)
result = add.roi(df)
```

### Querying loaded expressions

Use `add.scan('@set', 'show')` to list all currently registered expressions:

```python
info = add.scan('@set', 'show')
print(info)
```

---

## Writing a Custom .add File

Here's a complete example for a retail analytics team:

```toml title="retail_metrics.add"
[revenue_per_unit]
expression = "total_revenue / units_sold"
description = "Average revenue per unit sold"
category = "retail"

[gross_margin]
expression = "((selling_price - cost_price) / selling_price) * 100"
description = "Gross margin percentage"
category = "retail"

[inventory_turnover]
expression = "cost_of_goods_sold / average_inventory"
description = "Inventory turnover ratio"
category = "retail"

[days_of_supply]
expression = "average_inventory / (cost_of_goods_sold / 365)"
description = "Days of supply remaining"
category = "retail"
```

Save this as `retail_metrics.add` in your expressions folder, then load it:

```python
import additory as add
import polars as pl

# Load the custom expressions
add.scan('@set', './my_expressions/')

# Use them
products = pl.DataFrame({
    'product': ['Widget', 'Gadget'],
    'total_revenue': [50000, 80000],
    'units_sold': [1000, 500],
})

result = add.revenue_per_unit(products)
print(result)
```

---

## How Expressions Are Resolved

When you call `add.bmi(df)`, additory:

1. Looks up `"bmi"` in the expression registry
2. Finds the formula: `weight / (height ** 2)`
3. Matches column names in the formula to columns in your DataFrame
4. Evaluates the expression via `@calc` mode
5. Returns the DataFrame with the new column

### Column Auto-Matching

The expression's variable names are matched against your DataFrame's column names. If your DataFrame has columns named `weight` and `height`, the `bmi` expression works automatically.

### Explicit Mapping

If your column names don't match, use keyword arguments:

```python
# DataFrame has 'mass' and 'stature' instead of 'weight' and 'height'
result = add.bmi(df, weight='mass', height='stature')
```

---

## Built-in Expression Categories

Additory ships with expressions in these categories:

| Category | File | Count | Examples |
|----------|------|-------|---------|
| Core | `core.add` | 10 | bmi, profit, total_price |
| Finance | `finance.add` | 10 | roi, compound_interest, debt_to_equity |
| Medical | `medical.add` | 15 | heart_rate_max, bsa_dubois, bmr_male |
| Physics | `physics.add` | 11 | velocity, force, kinetic_energy |
| Chemistry | `chemistry.add` | 8 | molarity, percent_yield, ideal_gas_pressure |
| Engineering | `engineering.add` | 10 | power_electrical, resistance, reynolds_number |
| Statistics | `statistics.add` | 10 | z_score, standard_error, r_squared |

See the [Expression Catalog](../reference/expressions.md) for the complete list.

---

## Reserved Names

Some names are reserved and cannot be used as expression names:

- `to`, `transform`, `synthetic`, `scan` — core function names
- `analyze`, `analyse` — scan mode aliases
- `play` — internal use

If you try to register an expression with a reserved name, additory raises a `ValueError`.

---

## Next Steps

- [Expression Catalog](../reference/expressions.md) — all built-in expressions
- [add.\<dynamic\>()](../functions/dynamic.md) — the dynamic API
- [Reconciliation](reconciliation.md) — `.add` files for diff
