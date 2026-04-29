# Changelog

All notable changes to additory will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.3a11] - 2026-04-29

### Added
- **Scan Diff & Reconciliation** — `add.scan('@diff', ...)` for DataFrame comparison
  - Summary mode: inline `"old → new"` change markers with `_diff_status` column
  - Detail mode: one row per changed cell with `_key`, `_column`, `_old_value`, `_new_value`
  - Auto key detection: finds unique single or two-column composite keys
  - Duplicate handling: collapses identical duplicates, flags non-identical
  - Reconciliation via `.add` files: aliases (case-insensitive variant-to-canonical renaming) and groups (hierarchical change detection)
  - `exclude` and `carry` columns in strategy dict
  - Pure Python implementation in `diff_engine.py`
- **Pipe-Friendly Dynamic API**
  - `add.to()` pipe compatibility confirmed and tested (`df.pipe(add.to, ...)`)
  - `add.<dynamic>()` API — module-level `__getattr__` resolution for expression-based column operations
  - Column auto-matching from DataFrame columns, explicit mapping with `**kwargs`, expression evaluation via `@calc`
  - `add.scan('@set', ...)` — runtime expression loading from files and folders
- **Dynamic Expressions — Unified TOML Format**
  - Unified `.add` file format replaces simple/structured split — each top-level TOML table is one expression
  - Added `category` field to expressions
  - Rust expression parser, formatter, scanner, and config system (`src/expressions/`)
- **Rust Migration (partial)**
  - Rust core for diff types, strategy parsing, key detection, duplicate handling, alias application, row classification (`src/diff/`)
  - Config types, parser, and three-tier resolver (`src/config/`)
  - CLI crate (`additory-cli`) with subcommands: `to`, `transform`, `synthetic`, `diff`, `config`, `expressions`
  - `inbuilt:`/`user:` prefix migration error in `@calc` — raises `ValueError` with migration message

### Changed
- **`add.synthetic()` signature refactor** — DataFrame-first, pipe-friendly
  - First argument is now `df_or_mode: Union[DataFrame, str]`
  - Mode inferred from type: DataFrame → augment, `'@new'` → new
  - Old `mode=` kwarg raises `TypeError` with migration message
- **Expression data model** — removed `sha`, `format`, `requires` fields from `ExpressionDef`
- **Inbuilt expression files trimmed** — deleted `advanced.add` and `medical_extended.add`, merged useful expressions into `medical.add`

### Fixed
- 45 Rust test compilation errors fixed (stale `from_str` calls on KNN enums, type mismatches in label.rs/split.rs and across diff/config/expressions modules)
- 12 compiler warnings eliminated (dead code, unused imports, unused methods)
- Invalid regex escape sequences in expression safe pattern
- Synthetic test parameter structure updated (rows moved from strategy to `params.n`)

### Performance
- 404 Rust tests + 157 Python tests = 561 total tests passing
- Zero compiler warnings on `cargo build --lib`

---

## [0.1.3] - 2026-03-09

### Added
- **Lineage Tracking** - Track data transformations across operations
  - `lineage=False` parameter added to all three main functions (add.to, add.transform, add.synthetic)
  - Session-only lineage metadata stored in DataFrame native format
  - View lineage reports with `add.scan('@lineage', df)`
  - Track column sources, row mappings, and operation history
  - Mutual exclusion validation with `as_type` parameter (prevents metadata loss)
- **add.scan() Function** - Unified scanning interface
  - `add.scan('@analyze', df)` - Statistical analysis and data quality reports
  - `add.scan('@lineage', df)` - View lineage tracking reports
  - Replaces standalone `add.analyze()` and `add.analyse()` functions
  - ~95% Rust implementation for performance
- **Helper Functions** - Internal utilities for lineage tracking
  - `_get_added_columns()` - Track columns added in operations
  - `_get_modified_columns()` - Track columns modified (caller-tracked)
  - `_get_excluded_rows()` - Track rows excluded by filters
  - `_validate_lineage_as_type_exclusion()` - Validate mutual exclusion

### Changed
- **API Consolidation** - Moved analyze functionality to scan
  - `add.analyze()` → `add.scan('@analyze')` (analyze still works as alias)
  - `add.analyse()` → `add.scan('@analyse')` (analyse still works as alias)
  - Unified scanning interface for all inspection operations
- **Parameter Order Convention** - Standardized across all functions
  - Order: `logging`, `lineage`, `as_type` (as_type must be last)
  - Consistent keyword parameter ordering for better UX
- **Code Distribution** - Optimized Python/Rust split
  - Python (5%): API layer, lineage storage, configuration
  - Rust (95%): Core operations, statistical calculations, performance-critical code
  - Improved portability for future R/Julia implementations

### Fixed
- **add.to() Parameter Bug** - Fixed critical validation bug
  - Rust validation was receiving wrong parameters (left_keys instead of right_keys)
  - Added string-to-list conversion for `bring` and `against` parameters
  - All tests now passing with correct parameter mapping
- **Compiler Warnings** - Cleaned up Rust codebase
  - Fixed 11 compiler warnings automatically with `cargo fix`
  - Removed unused imports and variables
  - Remaining 4 warnings are intentional (reserved for future features)
- **Duplicate Imports** - Removed duplicate typing imports in Python code

### Removed
- **Orphan Files** - Cleaned up legacy code
  - Deleted `ADD_SCAN_COMPLETE_SPEC.md` (spec completed)
  - Deleted `scanner.py` (migrated to Rust)
  - Deleted `*_old.rs` backup files
  - Deleted `*.backup` files

### Performance
- Lineage overhead: <3ms per operation (84.4% overhead, well under 100ms target)
- Large DataFrame (10k rows): 36.89ms total time
- No performance regressions detected
- All 341 tests passing (17 Python + 324 Rust)

### Documentation
- Added `FUTURE_WORK_SUMMARY.md` - Documents planned features for v0.2.0
- Added `LINEAGE_IMPLEMENTATION_STATUS.md` - Complete lineage feature documentation
- Added `FUNCTION_SIGNATURES_WITH_LINEAGE.md` - Updated function signatures
- Added `QUICK_REFERENCE.md` - Quick reference for all functions
- Updated all docstrings with lineage parameter and examples

### Migration from v0.1.3a9
- **Lineage parameter** - New optional parameter in all three functions
  ```python
  # Enable lineage tracking
  result = add.to(..., lineage=True)
  result = add.transform(..., lineage=True)
  result = add.synthetic(..., lineage=True)
  
  # View lineage
  report = add.scan('@lineage', result)
  ```
- **Mutual exclusion** - Cannot use `lineage=True` with `as_type` parameter
  ```python
  # This will raise ValueError
  result = add.to(..., lineage=True, as_type='polars')
  
  # Instead, convert after tracking
  result = add.to(..., lineage=True)  # Track lineage
  result_pl = pl.from_pandas(result)  # Convert separately (lineage lost)
  ```
- **Analyze function** - Use `add.scan('@analyze')` instead
  ```python
  # Old (still works as alias)
  result = add.analyze(df)
  
  # New (recommended)
  result = add.scan('@analyze', df)
  ```

### Technical Details
- **Version:** 0.1.3 (stable alpha)
- **Python Support:** 3.8, 3.9, 3.10, 3.11, 3.12, 3.13
- **DataFrame Support:** pandas (optional), polars (required)
- **Test Coverage:** 341/341 tests passing (100%)
- **Code Distribution:** ~95% Rust, ~5% Python
- **Philosophy:** No file I/O, no internet, session-only lineage

---

## [0.1.3a10] - 2026-02-10

### Added
- **Pure Rust KNN Imputation** - Complete reimplementation without scikit-learn
  - Multiple distance metrics (Euclidean, Manhattan, Cosine)
  - Uniform and distance-weighted averaging strategies
  - 2x+ performance improvement over Python implementation
  - <500ms for 1000 rows × 10 columns
- **Pure Rust Label Deduction** - TF-IDF based label deduction
  - TF-IDF vectorization with unigrams and bigrams
  - Cosine similarity-based label assignment
  - 2x+ performance improvement over Python implementation
  - <1000ms for 1000 rows × 5 text columns
- **Python Synthetic Wrapper** - PyO3 bindings for synthetic data generation
  - Support for all 10 strategies (increment, range, choice, linked_list, normal, uniform, lognormal, exponential, poisson, categorical)
  - Seamless pandas/polars DataFrame conversion
  - Arrow IPC serialization for efficient data transfer
  - <200ms for 10,000 synthetic rows

### Removed
- **scikit-learn dependency** - Eliminated 60-70 MB dependency
  - Package size reduced from ~95-115 MB to ~35-45 MB (50+ MB savings)
  - Installation time reduced by 30%+
  - No external ML library dependencies
- Removed Python implementation files:
  - `python-specific/transform/knn.py`
  - `python-specific/additory/synthetic/strategies/deduce.py`

### Changed
- **R Portability Foundation** - Core logic separated from Python bindings
- **Zero-copy serialization** - Arrow IPC format for efficient data transfer
- **Modular utilities** - Reusable distance calculators and TF-IDF vectorizer

### Fixed
- **API Compatibility** - 100% backward compatible with existing Python API
- **Cross-platform support** - Tested on Linux, macOS, and Windows

### Performance Metrics
| Metric | Result | Target | Status |
|--------|--------|--------|--------|
| KNN imputation (1000×10) | <500ms | <500ms | ✅ Met |
| Label deduction (1000×5) | <1000ms | <1000ms | ✅ Met |
| Synthetic generation (10k rows) | <200ms | <200ms | ✅ Met |
| Package size reduction | 50+ MB | 50+ MB | ✅ Met |
| Installation time reduction | 30%+ | 30%+ | ✅ Met |

---

## [0.1.3a6] - 2026-02-13

### Fixed
- Added pyarrow as dependency for pandas users (required for pandas-to-polars conversion)

### Changed
- Updated optional dependencies to include pyarrow when installing with pandas support

---

## [0.1.3a5] - 2026-02-13

### Added
- **Complete PyO3 bindings** for Rust-Python integration
- **add.to()** - Data joining and lookup operations
  - Lookup mode with intelligent key matching
  - Multiple column fetching
  - Aggregation support (sum, mean, first, last, concat)
  - Position control for new columns
- **add.transform()** - Data transformation operations
  - `@calc` mode - Calculate new columns from expressions
  - `@filter` mode - Filter rows and select columns
  - `@sort` mode - Sort by column(s)
  - `@aggregate` mode - Group and aggregate data
  - `@transpose` mode - Transpose DataFrames
  - `@split` mode - Split text columns
  - `@extract` mode - Extract datetime components
  - `@onehot` mode - One-hot encoding
  - `@label` mode - Label encoding
  - `@harmonize` mode - Unit conversions
  - `@knn` mode - K-Nearest Neighbors imputation
- **add.synthetic()** - Synthetic data generation
  - `@new` mode - Create synthetic DataFrames from scratch
    - 7 statistical distributions (normal, lognormal, uniform, exponential, poisson, binomial, beta)
    - Categorical data (simple and weighted)
    - Sequences and date/time ranges
    - Pattern generation (email, phone, UUID, regex)
  - `augment` mode - Add synthetic rows to existing DataFrames
  - `@analyze` mode - Data quality analysis and statistics
- **Rust-powered performance** - 3-20x faster than pure Python
- **Polars and Pandas support** - Works seamlessly with both libraries
- **Comprehensive test coverage** - 106 Rust tests passing (100%)

### Changed
- Renamed `by` parameter to `against` in add.to() for clarity
- Updated to use `on` parameter for expressions (compatibility with v0.1.3a4)
- Improved error messages and validation
- Enhanced documentation with pandas examples (more familiar to users)
- Made pandas optional (polars is required for internal operations)

### Fixed
- Parameter name mismatches between Python wrapper and Rust code
- PyO3 bindings syntax issues
- DataFrame serialization edge cases

### Performance
- 3-5x faster transformations vs pure Python
- 5-10x faster data joining operations
- 10-20x faster synthetic data generation
- Efficient memory usage with Arrow IPC serialization

### Documentation
- Production-ready README with comprehensive examples
- Windows build guide for cross-platform support
- API reference documentation
- Integration test examples

### Technical Details
- **Language:** Rust + Python
- **Python Support:** 3.9, 3.10, 3.11, 3.12, 3.13
- **DataFrame Support:** pandas (optional), polars (required)
- **Dependencies:** polars>=0.19.0, pyarrow>=10.0.0 (required for DataFrame conversions)
- **Build System:** Maturin
- **License:** MIT

---

## [Unreleased]

### Planned Features
- Additional transform modes
- Enhanced expression parsing for complex formulas
- More synthetic data distributions
- Performance optimizations
- Extended documentation and tutorials

---

## Version History

- **0.1.3a6** (2026-02-13) - Fixed pyarrow dependency for pandas users
- **0.1.3a5** (2026-02-13) - First beta release with complete PyO3 bindings
- **0.1.3a4** (2026-02-12) - Alpha release with partial Rust implementation
- **0.1.3a3** (2026-02-11) - Alpha release with synthetic data features
- **0.1.3a2** (2026-02-10) - Alpha release with transform modes
- **0.1.3a1** (2026-02-09) - Initial alpha release

---

## Migration Guide

### From 0.1.3a4 to 0.1.3a5

**Breaking Changes:**
- `by` parameter in add.to() renamed to `against`
  ```python
  # Old (0.1.3a4)
  add.to(df, fetch_from=ref, fetch=['age'], by='id')
  
  # New (0.1.3a5)
  add.to(df, fetch_from=ref, fetch=['age'], against='id')
  ```

**Note:** The Python wrapper maintains backward compatibility by accepting `by` and converting it to `against` internally.

---

## Support

- **Issues:** https://github.com/sekarkrishna/additory/issues
- **Email:** krishnamoorthy.sankaran@sekrad.org
- **Repository:** https://github.com/sekarkrishna/additory

---

**Author:** Krishnamoorthy Sankaran  
**License:** MIT
