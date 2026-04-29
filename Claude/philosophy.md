# Additory — Design Philosophy

---

## Section 1 — Core Philosophy

Additory is a pure function over DataFrames.

You bring a DataFrame in. You get a DataFrame (or an analysis of one) out. That is the entire contract. Nothing is read from disk. Nothing is fetched from a network. Nothing is written anywhere. The world outside the DataFrame does not exist inside an Additory operation.

This constraint is not a limitation — it is the design. It keeps the library predictable, composable, and safe to use anywhere without side effects.

---

## Section 2 — Non-Negotiables

These are absolute. They do not bend for convenience, performance, or feature requests.

### No File I/O — Ever
Additory will never read from or write to files. No CSV loading, no Parquet writing, no expression files loaded at runtime, no config files. If data lives on disk, the user loads it with pandas or Polars before calling Additory. The library receives a DataFrame. The library returns a DataFrame.

> This includes expression definitions. The `.add` file concept has been dropped. Expressions are written inline. There is no file-based expression registry.

> **Override — Dynamic `.add` File API (2025):** The above stance is deliberately overridden for local `.add` file scanning. The dynamic `.add` file API is the soul of the library — institutions publish `.add` files (TOML expression definitions), and users consume them as `add.<name>(df)` calls. File I/O is limited to scanning local `.add` files at resolution time. No network access, no caching, no remote access. This is a deliberate, documented design decision. See the Pipe-Friendly & Dynamic .add File API spec for full details.

> **Override — config.toml System (2025):** The above stance is also deliberately overridden for `config.toml` files. Organizations can provide a `config.toml` to set defaults (seed, logging level, output type) and expression folder paths. Resolution follows a three-tier chain: expression folder → `~/.additory/` → built-in defaults. File I/O is limited to reading local TOML files at initialization and on explicit reload. This is a deliberate, documented design decision for organizational control.

### No Network Access — Ever
Additory will never make a network call. No API lookups, no remote data fetching, no telemetry, no update checks. Operations are fully offline and deterministic given the same inputs.

### DataFrame In — DataFrame or Analysis Out
Every function returns one of two things: a transformed DataFrame, or a structured analysis of a DataFrame. There is no third output type. Side effects are not outputs.

### Input DataFrames Are Never Modified
Additory does not mutate its inputs. Ever. Every operation returns a new object. The original DataFrame passed in is untouched after the call returns, regardless of what the operation does internally.

---

## Section 3 — Guiding Principles

These are strong design commitments. Deviations require deliberate justification and explicit documentation.

### Fail Fast with Actionable Errors

### Rust Core Is the Single Source of Truth
All parsing, computation, and validation logic lives in the Rust core (`additory/src/`). Language wrappers (Python via PyO3, CLI, future R bindings) are thin presentation layers that delegate to Rust. No computation logic is duplicated across wrappers. When a bug is fixed or a feature is added, it happens once in Rust and is immediately available everywhere.

### Fail Fast with Actionable Errors
When something is wrong, raise an exception immediately — not a warning, not a silent partial result. Error messages must include three things: what went wrong, why it is wrong, and a concrete example of how to fix it. A user reading the error message should not need to consult documentation.

### Preserve the User's DataFrame Type
If the user passes a Pandas DataFrame, they receive a Pandas DataFrame back. If they pass Polars, they receive Polars back. Type conversion is an explicit opt-in (`as_type`), never a surprise. Additory does not impose a preferred DataFrame library on the user.

### Enforce Cardinality — Prevent Silent Data Corruption
Many-to-many joins are rejected outright. They are not silently executed and left for the user to discover later through inflated row counts or duplicated data. Additory validates the relationship between DataFrames before performing any join and raises a clear error if the cardinality would produce unexpected results.

### Deterministic by Default
Where randomness is involved (synthetic data generation), the default behavior is deterministic. `seed=42` unless the user explicitly opts into randomness via `seed=None`. Reproducibility should require no effort; randomness should require explicit intent.

### Operations Are Atomic
An operation either completes fully or fails cleanly. There are no partial results, no half-transformed DataFrames, no columns added before an error occurs. If any part of an operation fails, the original DataFrame is returned unchanged.

### Lineage Is Opt-In with Zero Overhead When Off
Tracking transformation history is a feature, not a default behavior. When `lineage=False` (the default), there is no overhead — no metadata attached, no registry updated, no computation performed. Users who do not need lineage pay nothing for it.

---

## Section 4 — Guidelines

These are strong preferences that shape API and implementation decisions. They can be revisited, but departures should be intentional.

### Expressions Are Inline
The expression system accepts inline formulas directly in the `expression` parameter. Built-in named expressions (e.g. `inbuilt:bmi`) exist as a convenience layer over inlined formulas — they resolve to inline expressions at runtime.

> **Update (2025):** In addition to inline expressions, the dynamic `.add` file API now provides first-class `add.<name>(df)` calls backed by local `.add` files. See the "No File I/O" override note in Section 2 for the rationale.

> **Open issue**: Parenthesis support in `@calc` expressions is not yet enabled. This needs to be added before inline expressions can handle operator precedence correctly.

### Use a Domain Vocabulary, Not a Technical One
The API uses language that describes the operation in natural terms, not the underlying implementation. `bring_to`, `bring_from`, `bring`, and `against` describe a data movement in plain English. `@calc`, `@filter`, `@sort` describe what you want done, not how it is done internally. Technical terms like `target_df`, `left_join`, `merge_key` are avoided.

### The `@` Prefix Distinguishes Modes from Data
All operation modes use the `@` prefix (`@calc`, `@filter`, `@aggregate`). This is a visual and syntactic contract: anything prefixed with `@` is an instruction to Additory, not a column name or data value. This prevents an entire class of ambiguity errors.

### The User Controls Output Shape
Column placement is always explicit and user-controllable via the `position` parameter. New columns do not silently appear at the end unless the user accepts that default. The library provides `after:col`, `before:col`, `start`, and `end` as a full placement vocabulary. Additory does not decide where your columns go.

### Pandas and Polars Are Equally Supported
Neither DataFrame library is treated as primary. The internal implementation uses Polars (via Rust), but this is invisible to the user. Both input types receive identical API behavior, identical error messages, and identical output semantics. A user should be able to swap `pd.DataFrame` for `pl.DataFrame` in their code without changing a single Additory call.

---

## Section 5 — What Additory Is Not

These clarify scope and prevent feature creep.

- **Not an ETL pipeline** — it does not orchestrate multi-step workflows or manage data flow between systems
- **Not a database client** — it does not query, write to, or manage databases
- **Not a file format handler** — it does not read or write CSV, Parquet, Excel, JSON, or any other format
- **Not a visualization library** — it does not render charts, plots, or dashboards
- **Not a machine learning framework** — it does not train models, manage experiments, or serve predictions. Some operations use statistical algorithms internally (e.g. KNN for imputation, TF-IDF for text similarity in deduction) but these are implementation details of built-in transforms, not an exposed ML layer
- **Not a configuration system** — it does not read `.yaml`, `.env`, or arbitrary config files. The `config.toml` system is a deliberate, scoped exception for organizational defaults (see Section 2 override note)
- **Not stateful** — it does not maintain sessions, connection pools, or application state between calls

---

## Change Log

| Date | Change |
|------|--------|
| 2026-03-15 | Initial draft based on v0.1.3a10 codebase review |
