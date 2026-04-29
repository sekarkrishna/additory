# Type Handling

Additory works with both pandas and polars DataFrames. By default, the output type matches the input type. Use `as_type` to override this behaviour.

---

## Default Behaviour

Pass in a pandas DataFrame, get a pandas DataFrame back. Same for polars:

=== "Polars"

    ```python
    import additory as add
    import polars as pl

    df = pl.DataFrame({'x': [1, 2, 3]})
    result = add.transform('@calc', df, expression='x * 2', name='x2')
    type(result)  # polars.DataFrame
    ```

=== "Pandas"

    ```python
    import additory as add
    import pandas as pd

    df = pd.DataFrame({'x': [1, 2, 3]})
    result = add.transform('@calc', df, expression='x * 2', name='x2')
    type(result)  # pandas.DataFrame
    ```

---

## The as_type Parameter

Force the output to a specific type regardless of input:

```python
import additory as add
import pandas as pd

df = pd.DataFrame({'x': [1, 2, 3]})

# Input is pandas, but force polars output
result = add.transform('@calc', df, expression='x * 2', name='x2', as_type='polars')
type(result)  # polars.DataFrame
```

| `as_type` Value | Output Type |
|----------------|-------------|
| `None` (default) | Same as input |
| `'pandas'` | `pandas.DataFrame` |
| `'polars'` | `polars.DataFrame` |

### Supported Functions

| Function | `as_type` Support |
|----------|:-----------------:|
| `add.to()` | ✅ |
| `add.transform()` | ✅ |
| `add.synthetic()` | ✅ |
| `add.scan()` | ✅ (also supports `'dict'` and `'text'`) |

---

## Internal Processing

Internally, additory converts all DataFrames to polars for processing in Rust, then converts back to the target type. This means:

- **Polars input** → no conversion overhead (native format)
- **Pandas input** → converted to polars via Arrow IPC, processed, converted back

!!! tip "Polars is faster"
    If performance matters, use polars DataFrames directly to avoid the pandas↔polars conversion overhead. The conversion is fast (sub-millisecond for small DataFrames) but adds up in tight loops.

---

## Type Preservation

Additory preserves column data types through operations:

- Integer columns stay integer (unless the operation produces floats)
- String columns stay string
- Datetime columns stay datetime
- Null values are preserved as native null types

---

## Dependencies

| Library | Status | Notes |
|---------|--------|-------|
| polars | Required | Core dependency, always available |
| pyarrow | Required | Needed for DataFrame serialization |
| pandas | Optional | Install with `pip install additory[pandas]` or `pip install pandas` |

If pandas is not installed and you pass `as_type='pandas'`, additory raises an `ImportError` with a helpful message.

---

## Mutual Exclusion with Lineage

`as_type` cannot be used together with `lineage=True`. Lineage metadata is stored in the DataFrame's native format and would be lost during type conversion. See [Lineage Tracking](lineage.md) for details.

```python
# This raises ValueError
result = add.to(df, ref, 'name', 'id', lineage=True, as_type='polars')
```

---

## Next Steps

- [Lineage Tracking](lineage.md) — understand the lineage/as_type interaction
- [Logging & Timing](logging.md) — measure conversion overhead
- [Pipe Compatibility](pipe.md) — chaining with different DataFrame types
