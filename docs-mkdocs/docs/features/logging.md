# Logging & Timing

Monitor what additory is doing under the hood with operation logging and performance timing.

---

## Operation Logging

Pass `logging=True` to any core function to see detailed operation logs:

```python
import additory as add
import polars as pl

df = pl.DataFrame({
    'price': [100, 200, 300],
    'quantity': [2, 3, 1],
})

result = add.transform('@calc', df, expression='price * quantity',
    name='total', logging=True)
```

This prints log messages showing:

- Function entry point and parameters
- Validation steps
- Rust operation calls
- Data conversion steps

Logging uses Python's standard `logging` module at the `INFO` level.

---

## Performance Timing

Set the `ADDITORY_TIMING` environment variable to get a detailed timing breakdown:

=== "Shell"

    ```bash
    export ADDITORY_TIMING=true
    python my_script.py
    ```

=== "Python"

    ```python
    import os
    os.environ['ADDITORY_TIMING'] = 'true'

    import additory as add
    # ... your code here
    ```

When enabled, each operation prints a timing breakdown:

```
======================================================================
ADDITORY TIMING BREAKDOWN (add.to)
======================================================================
  Arrow encode (ref):        0.12 ms
  Arrow encode (target):     0.08 ms
  Rust operation:            1.45 ms  ← Main operation
  Arrow decode:              0.09 ms
  ─────────────────────────────────
  Total:                     1.74 ms

  Rust operation: 83.3% of total time
  Arrow IPC:      16.7% of total time
======================================================================
```

This helps identify whether time is spent in:

- **Arrow IPC encoding/decoding** — converting between Python DataFrames and Rust
- **Rust operation** — the actual computation
- **Python overhead** — validation, lineage tracking, type conversion

---

## Supported Functions

| Function | `logging` | `ADDITORY_TIMING` |
|----------|:---------:|:-----------------:|
| `add.to()` | ✅ | ✅ |
| `add.transform()` | ✅ | ✅ |
| `add.synthetic()` | ✅ | ✅ |
| `add.scan()` | — | — |

---

## Practical Usage

### Debugging unexpected results

```python
# See exactly what parameters are being passed to Rust
result = add.to(orders, customers, 'name', 'customer_id', logging=True)
```

### Benchmarking

```python
import os
os.environ['ADDITORY_TIMING'] = 'true'

# Compare performance of different approaches
result1 = add.transform('@calc', df, expression='x * 2', name='x2')
result2 = add.transform('@calc', df, strategy={'x2': 'x * 2', 'x3': 'x * 3'})
```

!!! tip "Disable timing in production"
    Set `ADDITORY_TIMING=false` (or unset it) to suppress timing output. The overhead of timing measurement itself is negligible.

---

## Next Steps

- [Lineage Tracking](lineage.md) — trace data provenance
- [Type Handling](type-handling.md) — pandas/polars interop
