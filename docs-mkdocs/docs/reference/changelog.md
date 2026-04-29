# Changelog

All notable changes to additory. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## 0.1.3a11 — 2026-04-29

### Added

- **Scan Diff & Reconciliation** — `add.scan('@diff', ...)` for DataFrame comparison
    - Summary mode: inline `"old → new"` change markers with `_diff_status` column
    - Detail mode: one row per changed cell with `_key`, `_column`, `_old_value`, `_new_value`
    - Auto key detection: finds unique single or two-column composite keys
    - Duplicate handling: collapses identical duplicates, flags non-identical
    - Reconciliation via `.add` files: aliases and groups
    - `exclude` and `carry` columns in strategy dict
- **Pipe-Friendly Dynamic API**
    - `add.to()` pipe compatibility (`df.pipe(add.to, ...)`)
    - `add.<dynamic>()` API — module-level `__getattr__` resolution for expression-based column operations
    - Column auto-matching, explicit mapping with `**kwargs`
    - `add.scan('@set', ...)` — runtime expression loading
- **Dynamic Expressions — Unified TOML Format**
    - Unified `.add` file format — each top-level TOML table is one expression
    - Added `category` field to expressions
    - Rust expression parser, formatter, scanner, and config system
- **Rust Migration (partial)**
    - Rust core for diff types, strategy parsing, key detection, duplicate handling
    - Config types, parser, and three-tier resolver
    - CLI crate (`additory-cli`) with subcommands: `to`, `transform`, `synthetic`, `diff`, `config`, `expressions`

### Changed

- **`add.synthetic()` signature refactor** — DataFrame-first, pipe-friendly
    - First argument is now `df_or_mode: Union[DataFrame, str]`
    - Mode inferred from type: DataFrame → augment, `'@new'` → new
    - Old `mode=` kwarg raises `TypeError` with migration message
- **Expression data model** — removed `sha`, `format`, `requires` fields
- **Inbuilt expression files trimmed** — deleted `advanced.add` and `medical_extended.add`

### Fixed

- 45 Rust test compilation errors fixed
- 12 compiler warnings eliminated
- Invalid regex escape sequences in expression safe pattern
- Synthetic test parameter structure updated

### Performance

- 404 Rust tests + 157 Python tests = 561 total tests passing
- Zero compiler warnings on `cargo build --lib`

---

## 0.1.3 — 2026-03-09

### Added

- **Lineage Tracking** — `lineage=True` parameter on all core functions
    - Session-only lineage metadata stored in DataFrame native format
    - View lineage reports with `add.scan('@lineage', df)`
    - Track column sources, row mappings, and operation history
    - Mutual exclusion validation with `as_type` parameter
- **add.scan() Function** — unified scanning interface
    - `add.scan('@analyze', df)` — statistical analysis and data quality
    - `add.scan('@lineage', df)` — view lineage tracking reports
    - Replaces standalone `add.analyze()` and `add.analyse()`

### Changed

- **API Consolidation** — `add.analyze()` → `add.scan('@analyze')`
- **Parameter Order Convention** — standardized: `logging`, `lineage`, `as_type`
- **Code Distribution** — ~95% Rust, ~5% Python

### Fixed

- `add.to()` parameter validation bug (wrong parameters sent to Rust)
- String-to-list conversion for `bring` and `against` parameters
- 11 compiler warnings fixed with `cargo fix`

### Performance

- Lineage overhead: <3ms per operation
- All 341 tests passing

---

## 0.1.3a10 — 2026-02-10

### Added

- **Pure Rust KNN Imputation** — no scikit-learn dependency
    - Multiple distance metrics (Euclidean, Manhattan, Cosine)
    - Uniform and distance-weighted averaging
    - 2x+ performance improvement over Python
- **Pure Rust Label Deduction** — TF-IDF based
    - Unigrams and bigrams, cosine similarity
    - 2x+ performance improvement over Python
- **Python Synthetic Wrapper** — PyO3 bindings for all 10 strategies

### Removed

- **scikit-learn dependency** — 50+ MB package size reduction

### Performance

| Metric | Result |
|--------|--------|
| KNN imputation (1000×10) | <500ms |
| Label deduction (1000×5) | <1000ms |
| Synthetic generation (10k rows) | <200ms |
| Package size reduction | 50+ MB |

---

## 0.1.3a6 — 2026-02-13

### Fixed

- Added pyarrow as dependency for pandas users

---

## 0.1.3a5 — 2026-02-13

### Added

- **Complete PyO3 bindings** for Rust-Python integration
- **add.to()** — lookup, multi-column fetch, aggregation, position control
- **add.transform()** — all 12 modes: `@calc`, `@filter`, `@sort`, `@aggregate`, `@transpose`, `@split`, `@extract`, `@onehot`, `@label`, `@harmonize`, `@deduce`, `@round`
- **add.synthetic()** — `@new` mode (7 distributions, patterns, sequences) and augment mode
- Polars and Pandas support

### Changed

- Renamed `by` parameter to `against` in `add.to()`

### Performance

- 3-5x faster transformations vs pure Python
- 5-10x faster data joining operations
- 10-20x faster synthetic data generation

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| 0.1.3a11 | 2026-04-29 | Diff, dynamic API, pipe compatibility, reconciliation |
| 0.1.3 | 2026-03-09 | Lineage tracking, `add.scan()`, 95% Rust |
| 0.1.3a10 | 2026-02-10 | Pure Rust KNN/TF-IDF, no scikit-learn |
| 0.1.3a6 | 2026-02-13 | pyarrow dependency fix |
| 0.1.3a5 | 2026-02-13 | First beta with complete PyO3 bindings |
| 0.1.3a4 | 2026-02-12 | Alpha with partial Rust |
| 0.1.3a3 | 2026-02-11 | Synthetic data features |
| 0.1.3a2 | 2026-02-10 | Transform modes |
| 0.1.3a1 | 2026-02-09 | Initial alpha release |
