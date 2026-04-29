# Additory Codebase Signatures

## Python Layer (`additory/additory/`)

### `__init__.py` — Public API
- `analyze(mode, df, ...)` — statistical profiling
- `analyse(...)` — alias
- `to(mode, df, ...)` — join/lookup/merge operations
- `transform(mode, df, ...)` — calc, filter, sort, aggregate, etc.
- `synthetic(mode, df, ...)` — data generation
- `scan(mode, df, ...)` — inspection/lineage
- `__getattr__(name)` — dynamic function dispatch from expression registry
- Internal helpers: `_validate_lineage_as_type_exclusion`, `_get_added_columns`, `_get_modified_columns`, `_get_excluded_rows`, `_auto_detect_method`, `_auto_generate_name`, `_make_dynamic_function`

### `scan.py`
- `scan(mode, df, *, columns, where, rows, trace, focus, as_type)` — delegates to Rust `_additory.scan`

### `diff_engine.py`
- Dataclasses: `StrategyConfig`, `CellChange`, `ChangedRow`, `DiffResult`
- `diff(old, new, *, key, strategy, reconciliation, aliases, as_type)` — full diff engine
- Helpers: `_parse_strategy`, `_validate_inputs`, `_parse_key`, `_validate_key`, `_detect_key`, `_handle_duplicates`, `_classify_rows`, `_format_summary`, `_format_detail`, `_resolve_reconciliation`, `_apply_aliases`, `_to_polars`, `_detect_input_type`

### `lineage_tracker.py`
- `Lineage_Tracker` — full lineage tracking with caching, memory overhead, row mapping
  - `record_operation`, `calculate_memory_overhead`, `check_memory_overhead_warning`
  - `sample_rows`, `compress_row_indices`
  - `update_column_sources_for_to/calc/aggregate`
  - `update_row_mapping_for_filter/aggregation`
- `OperationRecorder` — records to/transform/synthetic operations
- `RowMapper` — row-level origin tracing (initialize_mapping, update_for_filter/aggregation, trace_row_origin)
- `DependencyTracker` — formula dependency graph + circular detection (parse_formula, build_dependency_graph, trace_dependencies, detect_circular_dependencies)

### `expressions/loader.py`
- Dataclasses: `InputDef`, `Expression`, `ReconciliationDef`
- `ExpressionRegistry` — resolve by name, list all, namespace support
  - `resolve_by_name(name)`, `list_all_names()`, `resolve(reference)`, `list_expressions(namespace)`
  - `set_user_folder(folder_path)`
- `UserFolder` — user expression folder
- File loaders: `load_add_file`, `_load_simple_add_file`, `_load_structured_add_file`, `_load_reconciliation_add_file`
- Public API: `get_registry()`, `set_user_folder()`, `resolve_expression()`, `list_expressions()`, `format_add_file()`, `compute_sha256()`
- Reconciliation: `load_reconciliation_from_file()`, `resolve_reconciliation_by_name()`, `format_reconciliation_add_file()`

### `core/mode_parser.py`
- `parse_mode(value_string)` → `(mode, match, separator)`
- `validate_mode(mode, match)`
- `parse_strategy_value(value)`

### `core/param_handler.py`
- `normalize_column_input`, `normalize_key_input`, `normalize_by_input`
- `normalize_expression_input`, `normalize_as_input`
- `validate_expression_as_match`

### `validation/`
- `validate_to_params`, `validate_transform_params`, `validate_synthetic_params`
- `validate_list_not_tuple`, `validate_multiple_values`
- `validate_cardinality`, `get_cardinality_type`
- `validate_position`, `validate_string_position`, `normalize_position`
- `validate_synthetic_request`, `validate_strategy_format`, `validate_increment_conflicts`

### `transform/knn.py`
- `perform_knn_imputation(...)` — main entry
- `_knn_impute_inplace`, `_knn_impute_preserve`
- `_calculate_distances`, `_find_k_nearest`, `_compute_weighted_average`
- Distance: `_euclidean_distance`, `_manhattan_distance`, `_cosine_distance`

### `synthetic/strategies/increment.py`
- `parse_increment_strategy(strategy_string)` → `(step, start, pattern)`
- `generate_increment(n, step, start, pattern)`
- `validate_increment_step(step, n)`
- `format_with_pattern(value, pattern)`

### `games/games.py`
- `play(game)`, `tictactoe()`, `sudoku()`, `list_games()` — easter eggs

---

## Rust Core (`additory/src/`)

### `lib.rs` — PyO3 module entry
- `_additory` module registration
- `parse_fetch_parameter`, `parse_strategy_value`

### `core/`
- `DataFrame` wrapper (`from_polars`, `inner`, `inner_mut`)
- `AdditoryError` enum (cardinality, position, validation, mode_parsing)
- `TransformMode` enum, `JoinType` enum

### `config/`
- `ConfigData`, `OrganizationConfig`, `DefaultsConfig`, `SeedConfig`, `LoggingLevel`
- `parse_config_toml`, `load_config`, `merge_config`, `show_config`, `prefix_with_org`

### `expressions/`
- Types: `ExpressionDef`, `InputDef`, `ReconciliationDef`, `ParsedAddFile`, `AddFileFormat`
- Parser: `parse_simple_add_file`, `parse_structured_add_file`, `is_reconciliation_format`, `is_structured_format`
- Scanner: `scan_folder_for_expressions`, `resolve_expression_by_name`, `resolve_reconciliation_by_name`, `list_expression_names`
- Formatter: `format_expression`, `format_reconciliation`
- Identifiers: `extract_identifiers`

### `diff/`
- Types: `DiffResult`, `ChangedRow`, `CellChange`, `StrategyConfig`, `OutputMode`
- `diff(...)` — main entry
- `classify_rows`, `detect_key`, `validate_key`, `handle_duplicates`, `parse_strategy`, `apply_aliases`

### `scan/`
- `execute_scan`, `execute_analyze`, `execute_lineage`
- Types: `ScanMode`, `OutputFormat`, `ScanOutput`, `RowSpec`

### `to/`
- `to(...)` — main entry
- `lookup`, `apply_position`, `aggregate_series`
- `MergeType`, `MergeParams`, `Strategy`

### `transform/`
- `aggregate`, `bankers_round`, `calc` (single + multiple), `datetime`, `deduce` (imputation)
- `extract`, `filter`, `harmonize`, `knn`
- `calc/parser.rs` — expression tokenizer/parser

### `synthetic/`
- `synthetic(...)` — main entry
- `SyntheticMode` enum, `ColumnSchema` enum
- `augment::execute`

### `validation/`
- `DataValidator` — cardinality detection with sampling
- `ParameterValidator` — type/value/required checks
- `StrategyValidator` — strategy requirement detection
- `ValidationError` with suggestions + examples

### `utils/`
- `DistanceCalculator` trait + Euclidean/Manhattan impls
- `TfidfVectorizer` — fit_transform/transform
- `Logger`, type detection, general validation helpers

### `bindings/`
- `expression_cache` — get/set/clear/size for expression caching
- `python_features` — `resolve_expression`, `knn_impute`

---

## CLI (`additory-cli/src/`)

### Commands
- `config` — ConfigArgs + run()
- `diff` — DiffArgs + run()
- `expressions` — ExpressionsArgs + run()
- `synthetic` — SyntheticArgs + run()
- `to` — ToArgs + run()
- `transform` — TransformArgs + run()

### IO
- `read_dataframe(path)` → DataFrame
- `write_output(df, output)` → Result
