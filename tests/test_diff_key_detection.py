"""Unit tests for key parsing and auto-detection.

Covers: key string parsing, auto-detection (single/composite), missing key column errors.
"""

import sys
sys.path.insert(0, "additory")

import polars as pl
import pytest

from additory.diff_engine import (
    _parse_key,
    _validate_key,
    _detect_key,
    diff,
)


# ── Key string parsing ───────────────────────────────────────────────

class TestParseKey:
    def test_single_key(self):
        """Req 1.3: single key column."""
        assert _parse_key("id") == ["id"]

    def test_composite_key(self):
        """Req 1.2: comma-separated composite key."""
        assert _parse_key("col1,col2") == ["col1", "col2"]

    def test_key_with_spaces(self):
        """Spaces around commas are stripped."""
        assert _parse_key("col1 , col2") == ["col1", "col2"]

    def test_none_key(self):
        """None key returns None (auto-detect)."""
        assert _parse_key(None) is None


# ── Key validation ───────────────────────────────────────────────────

class TestValidateKey:
    def test_valid_key(self):
        """No error when key exists in both DataFrames."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        _validate_key(old, new, ["id"])  # Should not raise

    def test_missing_in_old(self):
        """Req 1.6: key missing from Old_DataFrame."""
        old = pl.DataFrame({"other": [1]})
        new = pl.DataFrame({"id": [1]})
        with pytest.raises(ValueError, match="'id'.*Old_DataFrame"):
            _validate_key(old, new, ["id"])

    def test_missing_in_new(self):
        """Req 1.6: key missing from New_DataFrame."""
        old = pl.DataFrame({"id": [1]})
        new = pl.DataFrame({"other": [1]})
        with pytest.raises(ValueError, match="'id'.*New_DataFrame"):
            _validate_key(old, new, ["id"])

    def test_missing_in_both(self):
        """Key missing from both DataFrames."""
        old = pl.DataFrame({"a": [1]})
        new = pl.DataFrame({"b": [1]})
        with pytest.raises(ValueError, match="'id'"):
            _validate_key(old, new, ["id"])


# ── Auto-detection ───────────────────────────────────────────────────

class TestDetectKey:
    def test_single_unique_column(self):
        """Req 2.2: single unique column detected."""
        old = pl.DataFrame({"id": [1, 2, 3], "val": ["a", "a", "b"]})
        new = pl.DataFrame({"id": [1, 2, 4], "val": ["a", "b", "c"]})
        key = _detect_key(old, new)
        assert key == ["id"]

    def test_composite_key_detection(self):
        """Req 2.3: composite key detected when no single column is unique."""
        old = pl.DataFrame({"a": [1, 1, 2], "b": [1, 2, 1], "val": ["x", "x", "z"]})
        new = pl.DataFrame({"a": [1, 1, 2], "b": [1, 2, 1], "val": ["x", "x", "z"]})
        key = _detect_key(old, new)
        assert len(key) == 2
        assert set(key) == {"a", "b"}

    def test_no_unique_column_raises(self):
        """Req 2.4: no unique column raises ValueError."""
        old = pl.DataFrame({"a": [1, 1], "b": [2, 2]})
        new = pl.DataFrame({"a": [1, 1], "b": [2, 2]})
        with pytest.raises(ValueError, match="Auto-detection failed"):
            _detect_key(old, new)

    def test_no_common_columns_raises(self):
        """No common columns raises ValueError."""
        old = pl.DataFrame({"a": [1]})
        new = pl.DataFrame({"b": [1]})
        with pytest.raises(ValueError, match="No common columns"):
            _detect_key(old, new)

    def test_auto_detect_through_diff(self):
        """Req 2.1: auto-detection works through diff() entry point."""
        old = pl.DataFrame({"uid": [1, 2, 3], "val": ["a", "b", "c"]})
        new = pl.DataFrame({"uid": [1, 2, 4], "val": ["a", "x", "d"]})
        result = diff(old=old, new=new)
        assert "_diff_status" in result.columns
        assert result.height == 4
