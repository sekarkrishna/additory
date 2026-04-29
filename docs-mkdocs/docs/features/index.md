# Cross-Cutting Features

Features that span multiple additory functions.

---

## Pipe Compatibility

Some functions work with `df.pipe()` for fluent method chaining. `add.to()`, `add.synthetic()`, and `add.<dynamic>()` are pipe-friendly. `add.transform()` and `add.scan()` are not (their first argument is a mode string, not a DataFrame).

→ [Pipe Compatibility](pipe.md)

---

## Lineage Tracking

Track data provenance across operations with `lineage=True`. View the full operation history, column sources, and row mappings with `add.scan('@lineage', df)`. Session-only by design — lineage lives in memory and is never written to disk.

→ [Lineage Tracking](lineage.md)

---

## Logging & Timing

Enable operation logging with `logging=True` or get detailed timing breakdowns with the `ADDITORY_TIMING` environment variable. Useful for debugging and performance profiling.

→ [Logging & Timing](logging.md)

---

## Type Handling

Additory works with both pandas and polars DataFrames. Output type matches input type by default. Use `as_type` to force a specific output type. Polars is the native internal format; pandas inputs are converted via Arrow IPC.

→ [Type Handling](type-handling.md)
