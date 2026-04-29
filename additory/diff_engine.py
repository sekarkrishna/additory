"""
Diff Engine — Compare two DataFrames and classify every row.

Public entry point: ``diff()`` orchestrates the full pipeline:
validate → parse strategy → resolve reconciliation → apply aliases →
detect/validate key → handle duplicates → classify → format.

All comparison logic uses Polars internally; the result is converted
back to match the caller's DataFrame type.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from itertools import combinations
from typing import Any, Dict, List, Optional, Union

import polars as pl

logger = logging.getLogger(__name__)

# ── Valid strategy keys ────────────────────────────────────────────────
_VALID_STRATEGY_KEYS = frozenset({
    "output", "exclude", "carry", "context", "aliases", "groups",
})

_VALID_OUTPUT_MODES = ("summary", "detail")


# ═══════════════════════════════════════════════════════════════════════
# Data-model dataclasses
# ═══════════════════════════════════════════════════════════════════════

@dataclass
class StrategyConfig:
    """Parsed and validated strategy dictionary."""
    output: str = "summary"
    exclude: List[str] = field(default_factory=list)
    carry: List[str] = field(default_factory=list)
    context: List[str] = field(default_factory=list)
    aliases: Optional[Union[Dict[str, List[str]], str]] = None
    groups: Optional[str] = None


@dataclass
class CellChange:
    """A single cell-level difference."""
    column: str
    old_value: Any
    new_value: Any
    is_hierarchical: bool = False


@dataclass
class ChangedRow:
    """A row present in both DataFrames with at least one cell difference."""
    key_values: Dict[str, Any]
    changes: List[CellChange]
    old_row: Dict[str, Any]
    new_row: Dict[str, Any]


@dataclass
class DiffResult:
    """Internal representation of the diff before formatting."""
    key_cols: List[str]
    new_rows: pl.DataFrame
    deleted_rows: pl.DataFrame
    changed_rows: List[ChangedRow]
    no_change_rows: pl.DataFrame
    duplicate_rows: pl.DataFrame



# ═══════════════════════════════════════════════════════════════════════
# 1 · Strategy parsing  (Task 1.1)
# ═══════════════════════════════════════════════════════════════════════

def _parse_strategy(strategy: Optional[Dict[str, Any]]) -> StrategyConfig:
    """Validate and destructure a strategy dict into a :class:`StrategyConfig`.

    Raises
    ------
    ValueError
        • Unrecognised key in *strategy*.
        • ``output`` value not ``'summary'`` or ``'detail'``.
        • ``groups`` passed as an inline dict (must be a registry name string).
        • A column appears in both ``exclude`` and ``carry``.
    """
    if strategy is None:
        return StrategyConfig()

    if not isinstance(strategy, dict):
        raise TypeError(
            f"strategy must be a dict, got {type(strategy).__name__}.\n"
            f"Example: strategy={{'output': 'detail', 'exclude': ['timestamp']}}"
        )

    # ── reject unknown keys ──────────────────────────────────────────
    unknown = set(strategy.keys()) - _VALID_STRATEGY_KEYS
    if unknown:
        raise ValueError(
            f"Unrecognised strategy key(s): {', '.join(sorted(unknown))}.\n"
            f"Valid keys are: {', '.join(sorted(_VALID_STRATEGY_KEYS))}.\n"
            f"Example: strategy={{'output': 'detail', 'exclude': ['timestamp']}}"
        )

    # ── output mode ──────────────────────────────────────────────────
    output = strategy.get("output", "summary")
    if output not in _VALID_OUTPUT_MODES:
        raise ValueError(
            f"Invalid output mode '{output}'.\n"
            f"Valid modes are: {', '.join(_VALID_OUTPUT_MODES)}.\n"
            f"Example: strategy={{'output': 'summary'}}"
        )

    # ── groups must be a string (registry name), not inline dict ─────
    groups = strategy.get("groups")
    if groups is not None and isinstance(groups, dict):
        raise ValueError(
            "Groups cannot be passed as an inline dict. "
            "Groups must be loaded from a .add file via the Expression Registry.\n"
            "Example: strategy={'groups': 'lab_categories'}\n"
            "See the .add file format documentation for how to define groups."
        )

    # ── exclude / carry lists ────────────────────────────────────────
    exclude = list(strategy.get("exclude", []))
    carry = list(strategy.get("carry", []))

    conflict = set(exclude) & set(carry)
    if conflict:
        raise ValueError(
            f"Column(s) appear in both 'exclude' and 'carry': "
            f"{', '.join(sorted(conflict))}.\n"
            f"A column can be excluded (omitted entirely) or carried "
            f"(included but not compared), but not both.\n"
            f"Remove the conflicting column(s) from one of the lists."
        )

    context = list(strategy.get("context", []))
    aliases = strategy.get("aliases")

    return StrategyConfig(
        output=output,
        exclude=exclude,
        carry=carry,
        context=context,
        aliases=aliases,
        groups=groups,
    )


# ═══════════════════════════════════════════════════════════════════════
# 2 · Input validation and key handling  (Tasks 2.1, 2.3)
# ═══════════════════════════════════════════════════════════════════════

def _validate_inputs(old: Any, new: Any) -> None:
    """Type-check *old* and *new* arguments.

    Raises
    ------
    ValueError  – if either argument is ``None``.
    TypeError   – if either argument is not a pandas/polars DataFrame.
    """
    try:
        import pandas as pd
        HAS_PANDAS = True
    except ImportError:
        HAS_PANDAS = False
        pd = None

    for arg_name, arg_val in [("old", old), ("new", new)]:
        if arg_val is None:
            raise ValueError(
                f"The '{arg_name}' argument is missing.\n"
                f"add.scan('@diff', ...) requires both old and new DataFrames.\n"
                f"Example: add.scan('@diff', old=df_old, new=df_new)"
            )
        is_pandas = HAS_PANDAS and pd is not None and isinstance(arg_val, pd.DataFrame)
        is_polars = isinstance(arg_val, pl.DataFrame)
        if not is_pandas and not is_polars:
            raise TypeError(
                f"The '{arg_name}' argument must be a pandas or polars DataFrame, "
                f"got {type(arg_val).__name__}.\n"
                f"Example: add.scan('@diff', old=pd.DataFrame(...), new=pd.DataFrame(...))"
            )


def _parse_key(key: Optional[str]) -> Optional[List[str]]:
    """Split a comma-separated key string into a list of column names.

    Returns ``None`` when *key* is ``None`` (auto-detect).
    """
    if key is None:
        return None
    parts = [k.strip() for k in key.split(",")]
    return [p for p in parts if p]


def _validate_key(
    old_pl: pl.DataFrame,
    new_pl: pl.DataFrame,
    key_cols: List[str],
) -> None:
    """Check that every key column exists in both DataFrames.

    Raises
    ------
    ValueError – with the missing column name and which DataFrame lacks it.
    """
    for col in key_cols:
        missing_in = []
        if col not in old_pl.columns:
            missing_in.append("Old_DataFrame")
        if col not in new_pl.columns:
            missing_in.append("New_DataFrame")
        if missing_in:
            raise ValueError(
                f"Key column '{col}' not found in {' or '.join(missing_in)}.\n"
                f"Available columns — Old: {old_pl.columns}, New: {new_pl.columns}.\n"
                f"Example: add.scan('@diff', old=df1, new=df2, key='{old_pl.columns[0]}')"
            )


def _detect_key(old_pl: pl.DataFrame, new_pl: pl.DataFrame) -> List[str]:
    """Auto-detect primary key columns.

    Strategy:
    1. Find columns present in both DataFrames.
    2. For each common column, check if values are unique in both.
    3. If exactly one unique column found, use it.
    4. If none found, try all 2-column combinations of common columns.
    5. If still none, raise ``ValueError`` with diagnostic info.
    """
    common = [c for c in old_pl.columns if c in new_pl.columns]
    if not common:
        raise ValueError(
            "No common columns between Old_DataFrame and New_DataFrame — "
            "cannot auto-detect a primary key.\n"
            f"Old columns: {old_pl.columns}\n"
            f"New columns: {new_pl.columns}\n"
            "Provide an explicit key: add.scan('@diff', old=df1, new=df2, key='my_key')"
        )

    # ── single-column candidates ─────────────────────────────────────
    unique_cols: List[str] = []
    for col in common:
        old_unique = old_pl[col].n_unique() == old_pl.height
        new_unique = new_pl[col].n_unique() == new_pl.height
        if old_unique and new_unique:
            unique_cols.append(col)

    if len(unique_cols) == 1:
        return unique_cols[:1]
    if len(unique_cols) > 1:
        # Multiple candidates — pick the first (deterministic)
        return unique_cols[:1]

    # ── two-column composite candidates ──────────────────────────────
    for c1, c2 in combinations(common, 2):
        old_unique = old_pl.select(pl.struct(c1, c2)).n_unique() == old_pl.height
        new_unique = new_pl.select(pl.struct(c1, c2)).n_unique() == new_pl.height
        if old_unique and new_unique:
            return [c1, c2]

    # ── give up ──────────────────────────────────────────────────────
    raise ValueError(
        f"Auto-detection failed: no single column or two-column combination "
        f"is unique in both DataFrames.\n"
        f"Columns tested: {common}\n"
        f"Provide an explicit key: add.scan('@diff', old=df1, new=df2, key='my_key')"
    )



# ═══════════════════════════════════════════════════════════════════════
# 4 · Duplicate handling  (Task 4.1)
# ═══════════════════════════════════════════════════════════════════════

def _handle_duplicates(
    old_pl: pl.DataFrame,
    new_pl: pl.DataFrame,
    key_cols: List[str],
    do_log: bool = False,
) -> tuple[pl.DataFrame, pl.DataFrame, pl.DataFrame]:
    """Detect and handle duplicate key values.

    Returns
    -------
    (cleaned_old, cleaned_new, duplicate_rows_df)
        *cleaned_old* / *cleaned_new* have identical-duplicate rows collapsed.
        *duplicate_rows_df* contains non-identical duplicate rows with a
        ``_diff_status`` column set to ``'duplicate'``.
    """
    dup_frames: List[pl.DataFrame] = []

    def _process(df: pl.DataFrame, label: str) -> pl.DataFrame:
        # Count occurrences of each key combination
        key_counts = df.group_by(key_cols).agg(pl.len().alias("_cnt"))
        dup_keys = key_counts.filter(pl.col("_cnt") > 1).select(key_cols)

        if dup_keys.height == 0:
            return df

        # Join to get all duplicate rows
        dup_rows = df.join(dup_keys, on=key_cols, how="inner")

        # For each duplicate key group, check if all rows are identical
        keep_rows: List[pl.DataFrame] = []
        flag_rows: List[pl.DataFrame] = []

        for key_vals in dup_keys.iter_rows(named=True):
            # Build filter for this key combination
            mask = pl.lit(True)
            for kc in key_cols:
                mask = mask & (pl.col(kc) == key_vals[kc])
            group = dup_rows.filter(mask)

            if group.n_unique() == 1:
                # All identical — collapse to one row
                if do_log:
                    logger.info(
                        "Collapsed %d identical duplicate rows for key %s in %s",
                        group.height, key_vals, label,
                    )
                keep_rows.append(group.head(1))
            else:
                # Non-identical — flag all
                flag_rows.append(group.with_columns(pl.lit("duplicate").alias("_diff_status")))

        # Build cleaned DataFrame: non-dup rows + collapsed rows
        non_dup = df.join(dup_keys, on=key_cols, how="anti")
        if keep_rows:
            non_dup = pl.concat([non_dup] + keep_rows)
        if flag_rows:
            dup_frames.append(pl.concat(flag_rows))

        return non_dup

    cleaned_old = _process(old_pl, "Old_DataFrame")
    cleaned_new = _process(new_pl, "New_DataFrame")

    if dup_frames:
        duplicate_rows = pl.concat(dup_frames)
    else:
        # Empty DataFrame with _diff_status column
        all_cols = list(dict.fromkeys(old_pl.columns + new_pl.columns))
        duplicate_rows = pl.DataFrame(
            {c: pl.Series([], dtype=pl.Utf8) for c in all_cols + ["_diff_status"]}
        )

    return cleaned_old, cleaned_new, duplicate_rows


# ═══════════════════════════════════════════════════════════════════════
# 5 · Row classification  (Task 5.1)
# ═══════════════════════════════════════════════════════════════════════

def _classify_rows(
    old_pl: pl.DataFrame,
    new_pl: pl.DataFrame,
    key_cols: List[str],
    exclude_cols: List[str],
    carry_cols: List[str],
    groups: Optional[Dict[str, List[str]]] = None,
) -> DiffResult:
    """Outer-join on *key_cols* and classify every row.

    Parameters
    ----------
    groups
        Optional parent→children mapping for hierarchical change detection.

    Raises
    ------
    ValueError – if an exclude or carry column doesn't exist in either DataFrame.
    """
    all_cols = set(old_pl.columns) | set(new_pl.columns)

    # ── validate exclude / carry columns ─────────────────────────────
    for label, col_list in [("exclude", exclude_cols), ("carry", carry_cols)]:
        for col in col_list:
            if col not in all_cols:
                raise ValueError(
                    f"Column '{col}' specified in '{label}' does not exist "
                    f"in either DataFrame.\n"
                    f"Available columns: {sorted(all_cols)}.\n"
                    f"Remove '{col}' from the '{label}' list."
                )

    # ── build the set of columns to compare ──────────────────────────
    skip = set(key_cols) | set(exclude_cols) | set(carry_cols)
    compare_cols = [c for c in old_pl.columns if c in new_pl.columns and c not in skip]

    # ── build group lookup (case-insensitive) ────────────────────────
    parent_of: Dict[str, str] = {}  # child_lower -> parent
    children_of: Dict[str, List[str]] = {}  # parent_lower -> [children]
    if groups:
        for parent, children in groups.items():
            pl_lower = parent.lower()
            children_of[pl_lower] = [c.lower() for c in children]
            for child in children:
                parent_of[child.lower()] = parent

    # ── classify rows ────────────────────────────────────────────────
    # Rows only in new
    new_only = new_pl.join(old_pl.select(key_cols).unique(), on=key_cols, how="anti")
    # Rows only in old
    deleted_only = old_pl.join(new_pl.select(key_cols).unique(), on=key_cols, how="anti")
    # Rows in both
    matched_old = old_pl.join(new_pl.select(key_cols).unique(), on=key_cols, how="semi")
    matched_new = new_pl.join(old_pl.select(key_cols).unique(), on=key_cols, how="semi")

    # Sort both matched sets by key for aligned iteration
    matched_old = matched_old.sort(key_cols)
    matched_new = matched_new.sort(key_cols)

    changed_rows: List[ChangedRow] = []
    no_change_indices_old: List[int] = []

    for i in range(matched_old.height):
        old_row = matched_old.row(i, named=True)
        # Find matching new row by key
        mask = pl.lit(True)
        for kc in key_cols:
            mask = mask & (pl.col(kc) == old_row[kc])
        new_match = matched_new.filter(mask)
        if new_match.height == 0:
            continue
        new_row = new_match.row(0, named=True)

        changes: List[CellChange] = []
        for col in compare_cols:
            old_val = old_row.get(col)
            new_val = new_row.get(col)
            # Handle None/null comparison
            if old_val is None and new_val is None:
                continue
            if old_val != new_val:
                is_hier = False
                if groups and old_val is not None and new_val is not None:
                    ov_lower = str(old_val).lower()
                    nv_lower = str(new_val).lower()
                    # parent→child or child→parent
                    if (ov_lower in children_of and nv_lower in children_of.get(ov_lower, [])):
                        is_hier = True
                    elif (nv_lower in children_of and ov_lower in children_of.get(nv_lower, [])):
                        is_hier = True
                    elif (ov_lower in parent_of and parent_of[ov_lower].lower() == nv_lower):
                        is_hier = True
                    elif (nv_lower in parent_of and parent_of[nv_lower].lower() == ov_lower):
                        is_hier = True
                changes.append(CellChange(
                    column=col,
                    old_value=old_val,
                    new_value=new_val,
                    is_hierarchical=is_hier,
                ))

        if changes:
            key_values = {kc: old_row[kc] for kc in key_cols}
            changed_rows.append(ChangedRow(
                key_values=key_values,
                changes=changes,
                old_row=old_row,
                new_row=new_row,
            ))
        else:
            no_change_indices_old.append(i)

    # Build no_change DataFrame from matched_old rows that had no changes
    if no_change_indices_old:
        no_change_rows = matched_old[no_change_indices_old]
    else:
        no_change_rows = matched_old.head(0)

    return DiffResult(
        key_cols=key_cols,
        new_rows=new_only,
        deleted_rows=deleted_only,
        changed_rows=changed_rows,
        no_change_rows=no_change_rows,
        duplicate_rows=pl.DataFrame(),  # filled by caller
    )



# ═══════════════════════════════════════════════════════════════════════
# 7 · Summary mode output  (Task 7.1)
# ═══════════════════════════════════════════════════════════════════════

def _format_summary(
    diff_result: DiffResult,
    old_pl: pl.DataFrame,
    new_pl: pl.DataFrame,
    exclude_cols: List[str],
) -> pl.DataFrame:
    """Build a Summary-mode output DataFrame.

    Columns: union of both DataFrames (minus excluded) + ``_diff_status``.
    Changed cells are formatted as ``"old_value → new_value"``.
    """
    all_cols_ordered: List[str] = []
    seen: set = set()
    for c in list(old_pl.columns) + list(new_pl.columns):
        if c not in seen and c not in exclude_cols:
            all_cols_ordered.append(c)
            seen.add(c)

    output_cols = all_cols_ordered + ["_diff_status"]
    frames: List[pl.DataFrame] = []

    # ── new rows ─────────────────────────────────────────────────────
    if diff_result.new_rows.height > 0:
        row_data: Dict[str, List] = {c: [] for c in output_cols}
        for row in diff_result.new_rows.iter_rows(named=True):
            for c in all_cols_ordered:
                row_data[c].append(row.get(c))
            row_data["_diff_status"].append("new")
        frames.append(pl.DataFrame(row_data))

    # ── deleted rows ─────────────────────────────────────────────────
    if diff_result.deleted_rows.height > 0:
        row_data = {c: [] for c in output_cols}
        for row in diff_result.deleted_rows.iter_rows(named=True):
            for c in all_cols_ordered:
                row_data[c].append(row.get(c))
            row_data["_diff_status"].append("deleted")
        frames.append(pl.DataFrame(row_data))

    # ── changed rows ─────────────────────────────────────────────────
    if diff_result.changed_rows:
        row_data = {c: [] for c in output_cols}
        for cr in diff_result.changed_rows:
            change_map = {ch.column: ch for ch in cr.changes}
            for c in all_cols_ordered:
                if c in change_map:
                    ch = change_map[c]
                    marker = f"{ch.old_value} → {ch.new_value}"
                    if ch.is_hierarchical:
                        marker += " (hierarchy)"
                    row_data[c].append(marker)
                else:
                    # Use new_row value (or old_row if column only in old)
                    row_data[c].append(
                        cr.new_row.get(c, cr.old_row.get(c))
                    )
            row_data["_diff_status"].append("changed")
        frames.append(pl.DataFrame(row_data))

    # ── no_change rows ───────────────────────────────────────────────
    if diff_result.no_change_rows.height > 0:
        row_data = {c: [] for c in output_cols}
        for row in diff_result.no_change_rows.iter_rows(named=True):
            for c in all_cols_ordered:
                row_data[c].append(row.get(c))
            row_data["_diff_status"].append("no_change")
        frames.append(pl.DataFrame(row_data))

    # ── duplicate rows ───────────────────────────────────────────────
    if diff_result.duplicate_rows.height > 0:
        row_data = {c: [] for c in output_cols}
        for row in diff_result.duplicate_rows.iter_rows(named=True):
            for c in all_cols_ordered:
                row_data[c].append(row.get(c))
            row_data["_diff_status"].append("duplicate")
        frames.append(pl.DataFrame(row_data))

    if not frames:
        return pl.DataFrame({c: pl.Series([], dtype=pl.Utf8) for c in output_cols})

    # Cast all frames to Utf8 for uniform concat (change markers are strings)
    cast_frames = []
    for f in frames:
        cast_frames.append(
            f.select([pl.col(c).cast(pl.Utf8) for c in output_cols])
        )
    return pl.concat(cast_frames)


# ═══════════════════════════════════════════════════════════════════════
# 8 · Detail mode output  (Task 8.1)
# ═══════════════════════════════════════════════════════════════════════

def _format_detail(
    diff_result: DiffResult,
    context_cols: List[str],
    new_pl: pl.DataFrame,
) -> pl.DataFrame:
    """Build a Detail-mode output DataFrame.

    Columns: ``_key``, ``_column``, ``_old_value``, ``_new_value``
    plus any requested *context_cols* from *new_pl*.

    Raises
    ------
    ValueError – if a context column does not exist in *new_pl*.
    """
    for col in context_cols:
        if col not in new_pl.columns:
            raise ValueError(
                f"Context column '{col}' does not exist in New_DataFrame.\n"
                f"Available columns: {new_pl.columns}.\n"
                f"Remove '{col}' from strategy['context']."
            )

    keys: List[str] = []
    columns: List[str] = []
    old_values: List[str] = []
    new_values: List[str] = []
    ctx_data: Dict[str, List] = {c: [] for c in context_cols}

    key_cols = diff_result.key_cols

    for cr in diff_result.changed_rows:
        key_str = ",".join(str(cr.key_values[k]) for k in key_cols)

        # Look up context values from new_pl
        ctx_vals: Dict[str, Any] = {}
        if context_cols:
            mask = pl.lit(True)
            for kc in key_cols:
                mask = mask & (pl.col(kc) == cr.key_values[kc])
            ctx_row = new_pl.filter(mask)
            if ctx_row.height > 0:
                ctx_vals = ctx_row.row(0, named=True)

        for ch in cr.changes:
            keys.append(key_str)
            columns.append(ch.column)
            old_values.append(str(ch.old_value) if ch.old_value is not None else None)
            new_values.append(str(ch.new_value) if ch.new_value is not None else None)
            for cc in context_cols:
                ctx_data[cc].append(ctx_vals.get(cc))

    result_data: Dict[str, Any] = {
        "_key": keys,
        "_column": columns,
        "_old_value": old_values,
        "_new_value": new_values,
    }
    result_data.update(ctx_data)

    return pl.DataFrame(result_data)



# ═══════════════════════════════════════════════════════════════════════
# 10 · Reconciliation resolution and alias application  (Task 10.3)
# ═══════════════════════════════════════════════════════════════════════

def _resolve_reconciliation(
    strategy_config: StrategyConfig,
) -> tuple[Optional[Dict[str, List[str]]], Optional[Dict[str, List[str]]]]:
    """Resolve aliases and groups from inline dict or registry name.

    Returns
    -------
    (aliases_dict, groups_dict)
        Both may be ``None`` if not specified.

    Raises
    ------
    ValueError – if a registry name cannot be resolved.
    """
    from .expressions.loader import resolve_reconciliation_by_name

    aliases: Optional[Dict[str, List[str]]] = None
    groups: Optional[Dict[str, List[str]]] = None

    # ── aliases ──────────────────────────────────────────────────────
    if strategy_config.aliases is not None:
        if isinstance(strategy_config.aliases, dict):
            aliases = strategy_config.aliases
        elif isinstance(strategy_config.aliases, str):
            recon = resolve_reconciliation_by_name(strategy_config.aliases)
            if recon is None:
                raise ValueError(
                    f"Reconciliation name '{strategy_config.aliases}' not found "
                    f"in the Expression Registry.\n"
                    f"Check that a .add file with [reconciliation] name = "
                    f"'{strategy_config.aliases}' exists in your expression folder.\n"
                    f"Example .add file:\n"
                    f"  [reconciliation]\n"
                    f"  name = \"{strategy_config.aliases}\"\n"
                    f"  description = \"...\"\n"
                    f"  [aliases]\n"
                    f"  ALT = [\"SGOT\", \"alanine_transaminase\"]"
                )
            aliases = recon.aliases

    # ── groups ───────────────────────────────────────────────────────
    if strategy_config.groups is not None:
        recon = resolve_reconciliation_by_name(strategy_config.groups)
        if recon is None:
            raise ValueError(
                f"Reconciliation name '{strategy_config.groups}' not found "
                f"in the Expression Registry.\n"
                f"Check that a .add file with [reconciliation] name = "
                f"'{strategy_config.groups}' exists in your expression folder.\n"
                f"Example .add file:\n"
                f"  [reconciliation]\n"
                f"  name = \"{strategy_config.groups}\"\n"
                f"  description = \"...\"\n"
                f"  [groups]\n"
                f"  Biochemistry = [\"creatinine\", \"alt\", \"ast\"]"
            )
        groups = recon.groups

    return aliases, groups


def _apply_aliases(
    old_pl: pl.DataFrame,
    new_pl: pl.DataFrame,
    aliases: Dict[str, List[str]],
) -> tuple[pl.DataFrame, pl.DataFrame]:
    """Rename columns in both DataFrames to canonical names using aliases.

    Matching is case-insensitive: a column named ``"sgot"`` matches variant
    ``"SGOT"`` and is renamed to the canonical key (e.g. ``"ALT"``).
    """
    # Build case-insensitive lookup: variant_lower -> canonical
    variant_to_canonical: Dict[str, str] = {}
    for canonical, variants in aliases.items():
        for v in variants:
            variant_to_canonical[v.lower()] = canonical

    def _rename(df: pl.DataFrame) -> pl.DataFrame:
        rename_map: Dict[str, str] = {}
        for col in df.columns:
            cl = col.lower()
            if cl in variant_to_canonical:
                rename_map[col] = variant_to_canonical[cl]
        if rename_map:
            return df.rename(rename_map)
        return df

    return _rename(old_pl), _rename(new_pl)


# ═══════════════════════════════════════════════════════════════════════
# 12 · Public entry point  (Task 12.1)
# ═══════════════════════════════════════════════════════════════════════

def _to_polars(df: Any) -> pl.DataFrame:
    """Convert a pandas or polars DataFrame to polars."""
    if isinstance(df, pl.DataFrame):
        return df
    # Check for pandas DataFrame
    _pd = None
    try:
        import pandas as _pd  # noqa: N813
    except ImportError:
        _pd = None
    if _pd is not None and isinstance(df, _pd.DataFrame):
        # Build polars DataFrame column-by-column to avoid pyarrow dependency
        data: Dict[str, list] = {}
        for col in df.columns:
            data[str(col)] = df[col].tolist()
        return pl.DataFrame(data)
    raise TypeError(
        f"Cannot convert {type(df).__module__}.{type(df).__name__} "
        f"to polars DataFrame"
    )


def _detect_input_type(old: Any, new: Any) -> str:
    """Return ``'pandas'``, ``'polars'``, or ``'mixed'``."""
    try:
        import pandas as pd
        HAS_PANDAS = True
    except ImportError:
        HAS_PANDAS = False
        pd = None

    old_is_pandas = HAS_PANDAS and pd is not None and isinstance(old, pd.DataFrame)
    new_is_pandas = HAS_PANDAS and pd is not None and isinstance(new, pd.DataFrame)
    old_is_polars = isinstance(old, pl.DataFrame)
    new_is_polars = isinstance(new, pl.DataFrame)

    if old_is_pandas and new_is_pandas:
        return "pandas"
    if old_is_polars and new_is_polars:
        return "polars"
    return "mixed"


def diff(
    *,
    old: Any,
    new: Any,
    key: Optional[str] = None,
    strategy: Optional[Dict[str, Any]] = None,
    logging: bool = False,
    as_type: Optional[str] = None,
) -> Any:
    """Compare two DataFrames and return a diff result.

    This is the public entry point called by ``scan('@diff', ...)``.
    """
    do_log = logging

    # 1. Validate inputs
    _validate_inputs(old, new)

    # 2. Detect input types for output preservation
    input_type = _detect_input_type(old, new)
    if do_log and input_type == "mixed":
        logger.info(
            "Mixed DataFrame types (pandas + polars). Output will be polars."
        )

    # 3. Convert to polars internally
    old_pl = _to_polars(old)
    new_pl = _to_polars(new)

    # 4. Parse strategy
    config = _parse_strategy(strategy)

    # 5. Resolve reconciliation (aliases / groups)
    aliases, groups = _resolve_reconciliation(config)

    # 6. Apply aliases
    if aliases:
        old_pl, new_pl = _apply_aliases(old_pl, new_pl, aliases)
        if do_log:
            logger.info(
                "Applied %d alias group(s): %s",
                len(aliases), list(aliases.keys()),
            )

    # 7. Detect or validate key
    key_cols = _parse_key(key)
    if key_cols is None:
        key_cols = _detect_key(old_pl, new_pl)
        if do_log:
            logger.info("Auto-detected key column(s): %s", key_cols)
    else:
        _validate_key(old_pl, new_pl, key_cols)
        if do_log:
            logger.info("Using provided key column(s): %s", key_cols)

    # 8. Handle duplicates
    old_pl, new_pl, dup_rows = _handle_duplicates(
        old_pl, new_pl, key_cols, do_log=do_log,
    )

    # 9. Classify rows
    result = _classify_rows(
        old_pl, new_pl, key_cols,
        exclude_cols=config.exclude,
        carry_cols=config.carry,
        groups=groups,
    )
    result.duplicate_rows = dup_rows

    if do_log:
        logger.info(
            "Diff result — new: %d, deleted: %d, changed: %d, "
            "no_change: %d, duplicate: %d",
            result.new_rows.height,
            result.deleted_rows.height,
            len(result.changed_rows),
            result.no_change_rows.height,
            result.duplicate_rows.height,
        )

    # 10. Format output
    if config.output == "detail":
        output_pl = _format_detail(result, config.context, new_pl)
    else:
        output_pl = _format_summary(result, old_pl, new_pl, config.exclude)

    # 11. Convert output to match input type (or as_type override)
    target_type = as_type or (input_type if input_type != "mixed" else "polars")
    if target_type == "pandas":
        return _polars_to_pandas(output_pl)
    return output_pl


def _polars_to_pandas(df: pl.DataFrame) -> Any:
    """Convert a polars DataFrame to pandas without requiring pyarrow."""
    try:
        import pandas as _pd
    except ImportError:
        return df  # fallback to polars if pandas not available
    data: Dict[str, list] = {}
    for col in df.columns:
        data[col] = df[col].to_list()
    return _pd.DataFrame(data)
