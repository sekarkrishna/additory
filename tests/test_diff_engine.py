"""Unit tests for the core diff engine logic.

Covers: basic diff, missing old/new, duplicate collapse, summary/detail output,
type preservation, logging, exclude/carry columns.
"""

import sys
sys.path.insert(0, "additory")

import polars as pl
import pandas as pd
import pytest
import logging

from additory.diff_engine import (
    diff,
    _validate_inputs,
    _classify_rows,
    _format_summary,
    _format_detail,
    _handle_duplicates,
    DiffResult,
)


# ── Basic diff ───────────────────────────────────────────────────────

class TestBasicDiff:
    def test_basic_diff_all_statuses(self):
        """Req 1.1, 4.1-4.4: basic diff with all four statuses."""
        old = pl.DataFrame({"id": [1, 2, 3], "val": ["a", "b", "c"]})
        new = pl.DataFrame({"id": [1, 2, 4], "val": ["a", "x", "d"]})
        result = diff(old=old, new=new, key="id")

        statuses = result["_diff_status"].to_list()
        assert "new" in statuses
        assert "deleted" in statuses
        assert "changed" in statuses
        assert "no_change" in statuses

    def test_no_changes(self):
        """All rows identical → all no_change."""
        old = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        new = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        result = diff(old=old, new=new, key="id")
        assert result["_diff_status"].to_list() == ["no_change", "no_change"]

    def test_all_new(self):
        """No overlap → all new + all deleted."""
        old = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        new = pl.DataFrame({"id": [3, 4], "val": ["c", "d"]})
        result = diff(old=old, new=new, key="id")
        statuses = set(result["_diff_status"].to_list())
        assert statuses == {"new", "deleted"}

    def test_changed_cell_marker(self):
        """Req 5.4: changed cells show 'old → new' marker."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        row = result.filter(pl.col("_diff_status") == "changed")
        assert row.height == 1
        assert "a → b" in row["val"][0]


# ── Missing old/new ─────────────────────────────────────────────────

class TestInputValidation:
    def test_missing_old(self):
        """Req 1.4: missing old raises ValueError."""
        new = pl.DataFrame({"id": [1]})
        with pytest.raises(ValueError, match="'old'.*missing"):
            diff(old=None, new=new, key="id")

    def test_missing_new(self):
        """Req 1.4: missing new raises ValueError."""
        old = pl.DataFrame({"id": [1]})
        with pytest.raises(ValueError, match="'new'.*missing"):
            diff(old=old, new=None, key="id")

    def test_invalid_type_old(self):
        """Req 1.5: non-DataFrame old raises TypeError."""
        with pytest.raises(TypeError, match="'old'.*must be"):
            diff(old="not_a_df", new=pl.DataFrame({"id": [1]}), key="id")

    def test_invalid_type_new(self):
        """Req 1.5: non-DataFrame new raises TypeError."""
        with pytest.raises(TypeError, match="'new'.*must be"):
            diff(old=pl.DataFrame({"id": [1]}), new=42, key="id")


# ── Duplicate handling ───────────────────────────────────────────────

class TestDuplicateHandling:
    def test_identical_duplicates_collapsed(self):
        """Req 3.2: identical duplicate rows are collapsed."""
        old = pl.DataFrame({"id": [1, 1, 2], "val": ["a", "a", "b"]})
        new = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        result = diff(old=old, new=new, key="id")
        # Should not have any 'duplicate' status
        assert "duplicate" not in result["_diff_status"].to_list()

    def test_non_identical_duplicates_flagged(self):
        """Req 3.3: non-identical duplicate rows are flagged."""
        old = pl.DataFrame({"id": [1, 1, 2], "val": ["a", "x", "b"]})
        new = pl.DataFrame({"id": [2], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        assert "duplicate" in result["_diff_status"].to_list()


# ── Summary output ───────────────────────────────────────────────────

class TestSummaryOutput:
    def test_summary_has_diff_status(self):
        """Req 5.2: summary output has _diff_status column."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        assert "_diff_status" in result.columns

    def test_summary_column_union(self):
        """Req 5.3: summary output has union of columns."""
        old = pl.DataFrame({"id": [1], "a": ["x"]})
        new = pl.DataFrame({"id": [1], "b": ["y"]})
        result = diff(old=old, new=new, key="id")
        assert "a" in result.columns
        assert "b" in result.columns
        assert "_diff_status" in result.columns

    def test_new_row_values(self):
        """Req 5.5: new rows use New_DataFrame values."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [2], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        new_row = result.filter(pl.col("_diff_status") == "new")
        assert new_row["val"][0] == "b"

    def test_deleted_row_values(self):
        """Req 5.6: deleted rows use Old_DataFrame values."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [2], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        del_row = result.filter(pl.col("_diff_status") == "deleted")
        assert del_row["val"][0] == "a"


# ── Detail output ────────────────────────────────────────────────────

class TestDetailOutput:
    def test_detail_columns(self):
        """Req 6.2: detail output has _key, _column, _old_value, _new_value."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id", strategy={"output": "detail"})
        assert set(result.columns) == {"_key", "_column", "_old_value", "_new_value"}

    def test_detail_one_row_per_cell(self):
        """Req 6.3: one row per changed cell."""
        old = pl.DataFrame({"id": [1], "a": ["x"], "b": ["y"]})
        new = pl.DataFrame({"id": [1], "a": ["X"], "b": ["Y"]})
        result = diff(old=old, new=new, key="id", strategy={"output": "detail"})
        assert result.height == 2

    def test_detail_composite_key(self):
        """Req 6.4: composite keys as comma-separated."""
        old = pl.DataFrame({"k1": [1], "k2": ["a"], "val": ["x"]})
        new = pl.DataFrame({"k1": [1], "k2": ["a"], "val": ["y"]})
        result = diff(old=old, new=new, key="k1,k2", strategy={"output": "detail"})
        assert result["_key"][0] == "1,a"

    def test_detail_context_columns(self):
        """Req 6.5: context columns from New_DataFrame."""
        old = pl.DataFrame({"id": [1], "val": ["a"], "name": ["Alice"]})
        new = pl.DataFrame({"id": [1], "val": ["b"], "name": ["Alice"]})
        result = diff(
            old=old, new=new, key="id",
            strategy={"output": "detail", "context": ["name"]},
        )
        assert "name" in result.columns
        assert result["name"][0] == "Alice"

    def test_detail_missing_context_column(self):
        """Req 6.6: missing context column raises ValueError."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        with pytest.raises(ValueError, match="Context column 'missing'"):
            diff(
                old=old, new=new, key="id",
                strategy={"output": "detail", "context": ["missing"]},
            )


# ── Exclude / Carry columns ─────────────────────────────────────────

class TestExcludeCarry:
    def test_exclude_columns_not_in_output(self):
        """Req 7.1, 7.2: excluded columns omitted from output."""
        old = pl.DataFrame({"id": [1], "val": ["a"], "ts": ["2024-01-01"]})
        new = pl.DataFrame({"id": [1], "val": ["b"], "ts": ["2024-01-02"]})
        result = diff(old=old, new=new, key="id", strategy={"exclude": ["ts"]})
        assert "ts" not in result.columns

    def test_exclude_only_diff_is_no_change(self):
        """Req 4.5: if only excluded columns differ, row is no_change."""
        old = pl.DataFrame({"id": [1], "val": ["a"], "ts": ["2024-01-01"]})
        new = pl.DataFrame({"id": [1], "val": ["a"], "ts": ["2024-01-02"]})
        result = diff(old=old, new=new, key="id", strategy={"exclude": ["ts"]})
        assert result["_diff_status"][0] == "no_change"

    def test_carry_columns_in_output(self):
        """Req 7.3: carry columns appear in output but don't affect comparison."""
        old = pl.DataFrame({"id": [1], "val": ["a"], "comment": ["old note"]})
        new = pl.DataFrame({"id": [1], "val": ["a"], "comment": ["new note"]})
        result = diff(old=old, new=new, key="id", strategy={"carry": ["comment"]})
        assert "comment" in result.columns
        assert result["_diff_status"][0] == "no_change"

    def test_missing_exclude_column(self):
        """Req 7.5: missing exclude column raises ValueError."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        with pytest.raises(ValueError, match="Column 'missing'.*'exclude'"):
            diff(old=old, new=new, key="id", strategy={"exclude": ["missing"]})


# ── Type preservation ────────────────────────────────────────────────

class TestTypePreservation:
    def test_polars_in_polars_out(self):
        """Req 12.2: both polars → polars output."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        assert isinstance(result, pl.DataFrame)

    def test_pandas_in_pandas_out(self):
        """Req 12.1: both pandas → pandas output."""
        old = pd.DataFrame({"id": [1], "val": ["a"]})
        new = pd.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        assert isinstance(result, pd.DataFrame)

    def test_mixed_returns_polars(self):
        """Req 12.3: mixed types → polars output."""
        old = pd.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id")
        assert isinstance(result, pl.DataFrame)

    def test_as_type_override(self):
        """Req 12.4: as_type overrides input type."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        result = diff(old=old, new=new, key="id", as_type="pandas")
        assert isinstance(result, pd.DataFrame)


# ── Logging ──────────────────────────────────────────────────────────

class TestLogging:
    def test_logging_key_detection(self, caplog):
        """Req 13.1: logging reports detected key."""
        old = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        new = pl.DataFrame({"id": [1, 2], "val": ["a", "b"]})
        with caplog.at_level(logging.INFO, logger="additory.diff_engine"):
            diff(old=old, new=new, key="id", logging=True)
        assert any("key column" in r.message.lower() for r in caplog.records)

    def test_logging_row_counts(self, caplog):
        """Req 13.2: logging reports row counts."""
        old = pl.DataFrame({"id": [1, 2, 3], "val": ["a", "b", "c"]})
        new = pl.DataFrame({"id": [1, 2, 4], "val": ["a", "x", "d"]})
        with caplog.at_level(logging.INFO, logger="additory.diff_engine"):
            diff(old=old, new=new, key="id", logging=True)
        assert any("diff result" in r.message.lower() for r in caplog.records)
