# Additory

**Elegant data operations for DataFrames.** Lookup, transform, generate, compare — all through a consistent, expressive API.

<span class="md-version">v0.1.3a11</span>

---

## Install

```bash
pip install additory
```

Requires Python 3.9+. Works with both **polars** (required) and **pandas** (optional).

---

## What can additory do?

### :material-table-arrow-left: add.to() — Bring data from one DataFrame to another

Look up columns from a reference table and attach them to your target DataFrame. Supports aggregation, multi-key joins, and list-of-DataFrames patterns.

```python
import additory as add

result = add.to(patients, bring_from=doctors, bring='name', against='doctor_id')
```

[Learn more :material-arrow-right:](functions/to.md)

---

### :material-swap-horizontal: add.transform() — Reshape data in place

Twelve modes for calculating, filtering, sorting, aggregating, deducing, encoding, and more — all through a single function.

```python
result = add.transform('@calc', df, expression='price * quantity', name='total')
```

[Learn more :material-arrow-right:](functions/transform.md)

---

### :material-flask-outline: add.synthetic() — Generate realistic data

Create new DataFrames from scratch or augment existing ones with synthetic rows. Pipe-friendly.

```python
result = add.synthetic(df, n=500)
result = df.pipe(add.synthetic, n=500)
```

[Learn more :material-arrow-right:](functions/synthetic.md)

---

### :material-magnify: add.scan() — Inspect and compare

Analyze data quality, track lineage, and compare DataFrames with `@diff`.

```python
add.scan('@analyze', df)
add.scan('@diff', old=df_jan, new=df_feb)
```

[Learn more :material-arrow-right:](functions/scan.md) ·
[Diff :material-arrow-right:](functions/diff.md)

---

### :material-function-variant: add.&lt;dynamic&gt;() — Call expressions by name

Load named expressions from `.add` files and call them directly on the module.

```python
add.scan('@set', './my_expressions')
result = add.bmi(df)
```

[Learn more :material-arrow-right:](functions/dynamic.md)

---

## Explore the docs

| Section | What you'll find |
|---------|-----------------|
| [Getting Started](getting-started/install.md) | Installation, quickstart walkthrough |
| [Core Functions](functions/index.md) | `add.to()`, `add.transform()`, `add.synthetic()`, `add.scan()`, `add.<dynamic>()` |
| [Transform Modes](transform/index.md) | `@calc`, `@filter`, `@sort`, `@aggregate`, `@deduce`, and more |
| [Cross-Cutting Features](features/index.md) | Pipe compatibility, lineage tracking, logging, type handling |
| [Guides](guides/index.md) | Expression files, reconciliation, troubleshooting |
| [Reference](reference/api.md) | Full API signatures, expression catalog, changelog |
