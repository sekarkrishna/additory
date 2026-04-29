# Core Functions

Additory provides five core functions plus a dynamic expression API. Each function handles a distinct category of DataFrame operations.

---

## add.to()

Bring columns from one DataFrame into another by matching on a key column. Supports lookups, joins, aggregation strategies, and list-of-DataFrames patterns.

```python
result = add.to(orders, customers, 'name', 'customer_id')
```

→ [Full documentation](to.md)

---

## add.transform()

Transform data within a single DataFrame using one of 12 modes — calculate, filter, sort, aggregate, encode, impute, and more.

```python
result = add.transform('@calc', df, expression='price * quantity', name='total')
```

→ [Full documentation](transform.md) · [Transform Modes](../transform/calc.md)

---

## add.synthetic()

Generate synthetic data from scratch or augment existing DataFrames with realistic rows.

```python
result = add.synthetic('@new', n=1000, strategy={'age': 'normal(40, 10)'})
```

→ [Full documentation](synthetic.md)

---

## add.scan()

Inspect, analyze, and compare DataFrames. Covers statistical profiling, lineage reports, diff comparison, and expression management.

```python
result = add.scan('@analyze', df)
```

→ [Full documentation](scan.md) · [Diff documentation](diff.md)

---

## add.\<dynamic\>()

Call named expressions as functions. Any expression registered in the expression registry becomes a callable on the `add` module.

```python
result = add.bmi(df)
```

→ [Full documentation](dynamic.md)
