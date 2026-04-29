# Additory v0.1.3a10 — Project Review

**Reviewed**: 2026-03-15
**Reviewer**: Claude (claude-sonnet-4-6)
**Scope**: Full codebase review — architecture, code quality, security, performance, documentation

---

## Project Overview

Additory is a Rust-powered Python library for DataFrame manipulation supporting both Polars and Pandas. It exposes a 3-function API:

- `add.to()` — Bring columns from external DataFrames using lookup/join semantics
- `add.transform()` — Transform data in-place (11 modes: @calc, @filter, @sort, @aggregate, @round, @bankers_round, @transpose, @extract, @onehotencode, @deduce, @harmonize)
- `add.synthetic()` — Generate synthetic data (@new, @augment)
- `add.scan()` — Inspect and profile DataFrames (@analyze, @lineage)

**Tech stack**: Python 3.9+, Rust (~9,348 lines), PyO3 bindings, Maturin build system

---

## What Works Well

- **Clean API design** — three-function mental model is intuitive and consistent
- **Strong Rust core** — well-typed, solid module structure across core/transform/to/synthetic/scan
- **179 built-in expressions** — organized across finance, medical, physics, engineering, chemistry, statistics domains
- **Excellent user-facing documentation** — 20+ examples, Quarto book, migration guides, quick reference card
- **Cardinality validation** — prevents many:many joins with clear, actionable error messages
- **Lineage tracking** — clever dual-storage design (Polars global registry + Pandas DataFrame.attrs)
- **Expression caching** — namespace resolution cached to <0.1ms after first load
- **Parameter flexibility** — accepts str/list/tuple inputs with consistent normalization

---

## Issues — Prioritized

### CRITICAL — Fix before any production use

| # | Issue | Location | Impact |
|---|-------|----------|--------|
| 1 | **728 `unwrap()`/`expect()` calls in Rust** | Throughout `src/` | Any one can panic and crash the process |
| 2 | **Composite keys only use the first key** | `src/to/lookup.rs` | Multi-column joins silently produce wrong results |
| 3 | **Test suite not present in repository** | — | Cannot verify the claimed "341/341 tests passing" |
| 4 | **No input sanitization on SQL-like where clause** | `@filter`, `@extract` | Injection risk on user-controlled filter strings |
| 5 | **Expression `.add` files loaded without content validation** | `additory/expressions/loader.py` | Malicious expression files could execute arbitrary code |

---

### HIGH — Fix soon

| # | Issue | Location | Impact |
|---|-------|----------|--------|
| 6 | **`rows=` parameter not implemented** — passed but silently ignored | `src/lib.rs` (TODO comment) | API advertises a non-working feature |
| 7 | **`focus=` parameter not implemented** — two TODO comments | `src/scan/mod.rs` | Same — silent no-op |
| 8 | **Expression parser incomplete** — only simple expressions work | `src/transform/calc.rs` (TODO comment) | Complex nested expressions have undefined behavior |
| 9 | **`@deduce` mode calculation incomplete** | `src/transform/deduce.rs` (TODO comment) | Incorrect or incomplete missing value imputation |
| 10 | **Lineage registry not thread-safe** | `additory/lineage_tracker.py` | Race conditions when used in async or multithreaded code |

---

### MEDIUM — Fix in next iteration

| # | Issue | Location | Impact |
|---|-------|----------|--------|
| 11 | **Excessive DataFrame cloning in hot path** | `src/transform/filter.rs:22` (TODO comment exists) | Memory doubles per transform operation |
| 12 | **Regex compiled inside loops** | `src/utils/type_detection.rs` | Performance regression on large DataFrames |
| 13 | **`as_type` validation inconsistent** across `add.to()`, `add.transform()`, `add.synthetic()` | Python API (`additory/__init__.py`) | Partial validation leads to confusing runtime errors |
| 14 | **Orphaned/obsolete files left in codebase** | `src/core/types_old.rs`, `src/core/errors_old.rs`, `src/transform/mod_old.rs`, `src/to/mod_old.rs`, `src/validation/data.rs.backup` | Confusion about which version is authoritative |
| 15 | **481KB `docs/index.tex` build artifact committed to repo** | `docs/index.tex` | Unnecessary repo size bloat; should be in `.gitignore` |
| 16 | **Python version conflict** — `pyproject.toml` requires >=3.9, README states 3.8+ | Config | Potential build failures and user confusion |

---

### LOW — Address when possible

| # | Issue | Location | Impact |
|---|-------|----------|--------|
| 17 | **Function signatures too wide** — 13+ optional parameters on each function | All three main functions | Hard to use correctly, discovery is poor |
| 18 | **No environment variable support** for runtime config | Runtime config | Complicates Docker and cloud deployments |
| 19 | **`add.save()` / `add.load()` referenced in README but not implemented** | `README.md` | Broken documentation promise |
| 20 | **Persistent lineage not supported** — metadata is lost when DataFrame is saved | By design but undocumented | Breaks reproducible pipelines |

---

## Top 5 Immediate Actions

1. **Fix composite key handling** in `src/to/lookup.rs` — currently only uses `join_keys[0]`, silently producing wrong results on multi-column joins.

2. **Replace `unwrap()`/`expect()` calls** with `?` operator and proper error propagation — 728 instances, any one is a production crash waiting to happen.

3. **Add a `tests/` directory** with integration tests — test status is currently unverifiable; add at minimum: `add.to()` cardinality scenarios, `add.transform()` mode coverage, lineage round-trips.

4. **Mark unimplemented parameters as `NotImplementedError`** — `rows=` in `add.scan()` and `focus=` in `add.scan()` should raise a clear error rather than silently doing nothing.

5. **Delete obsolete files** — remove `*_old.rs` and `.backup` files from `src/` to eliminate confusion about the authoritative implementation.

---

## Architecture Observations

### Strengths
- Clean separation between Python API layer and Rust implementation
- PyO3 bindings are well-structured
- Error types in `src/core/errors.rs` are comprehensive and actionable
- Expression namespace system (`inbuilt:`, `user:`) is extensible

### Concerns
- `additory/__init__.py` is ~1,634 lines — consider splitting into submodules per function
- Global mutable state in `lineage_tracker.py` (`_polars_lineage_registry`, `_polars_version_counter`) is not protected by a lock
- No CI/CD pipeline found — no automated test running on commits

---

## Security Summary

| Risk | Severity | Notes |
|------|----------|-------|
| Unsanitized SQL-like where clause | HIGH | Filter strings evaluated without escaping |
| Unvalidated `.add` expression file loading | HIGH | File contents executed without sandboxing |
| Global mutable state race conditions | MEDIUM | Lineage registry unprotected |
| Error messages expose internal paths | LOW | Column names and file paths in error output |

---

## Overall Assessment

**Stability: Beta/Experimental**

The library has a well-designed API and strong architecture foundations. It is well-suited for prototyping and small-scale use. It is **not production-ready** due to:

- Panic risk from 728 unguarded `unwrap()` calls
- Incomplete features being silently ignored rather than raising errors
- Unverifiable test coverage
- Security gaps in input handling

Estimated effort to reach production-ready (v1.0): 3–6 months of focused work on the critical and high-priority items above.
