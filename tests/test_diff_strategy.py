"""Unit tests for strategy validation.

Covers: unrecognized keys, invalid output, groups as inline dict,
exclude/carry conflict, missing exclude/carry columns.
"""

import sys
sys.path.insert(0, "additory")

import polars as pl
import pytest

from additory.diff_engine import _parse_strategy, StrategyConfig, diff


# ── Strategy parsing ─────────────────────────────────────────────────

class TestParseStrategy:
    def test_none_strategy(self):
        """Default strategy when None."""
        config = _parse_strategy(None)
        assert config.output == "summary"
        assert config.exclude == []
        assert config.carry == []

    def test_valid_strategy(self):
        """Valid strategy dict is parsed correctly."""
        config = _parse_strategy({
            "output": "detail",
            "exclude": ["ts"],
            "carry": ["comment"],
            "context": ["name"],
        })
        assert config.output == "detail"
        assert config.exclude == ["ts"]
        assert config.carry == ["comment"]
        assert config.context == ["name"]

    def test_unrecognized_key(self):
        """Req 11.5: unrecognized key raises ValueError."""
        with pytest.raises(ValueError, match="Unrecognised strategy key.*bogus"):
            _parse_strategy({"bogus": "value"})

    def test_invalid_output_mode(self):
        """Req 11.6: invalid output mode raises ValueError."""
        with pytest.raises(ValueError, match="Invalid output mode 'csv'"):
            _parse_strategy({"output": "csv"})

    def test_groups_as_inline_dict(self):
        """Req 11.4: groups as inline dict raises ValueError."""
        with pytest.raises(ValueError, match="Groups cannot be passed as an inline dict"):
            _parse_strategy({"groups": {"parent": ["child"]}})

    def test_exclude_carry_conflict(self):
        """Req 7.4: column in both exclude and carry raises ValueError."""
        with pytest.raises(ValueError, match="both 'exclude' and 'carry'.*ts"):
            _parse_strategy({"exclude": ["ts"], "carry": ["ts"]})

    def test_strategy_not_dict(self):
        """Strategy must be a dict."""
        with pytest.raises(TypeError, match="strategy must be a dict"):
            _parse_strategy("not_a_dict")

    def test_aliases_as_string(self):
        """Aliases as string is accepted (registry name)."""
        config = _parse_strategy({"aliases": "lab_aliases"})
        assert config.aliases == "lab_aliases"

    def test_aliases_as_dict(self):
        """Aliases as dict is accepted (inline)."""
        config = _parse_strategy({"aliases": {"ALT": ["SGOT"]}})
        assert config.aliases == {"ALT": ["SGOT"]}

    def test_groups_as_string(self):
        """Groups as string is accepted (registry name)."""
        config = _parse_strategy({"groups": "lab_categories"})
        assert config.groups == "lab_categories"


# ── Missing exclude/carry columns (through diff) ────────────────────

class TestMissingColumns:
    def test_missing_exclude_column(self):
        """Req 7.5: missing exclude column raises ValueError."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        with pytest.raises(ValueError, match="Column 'missing'.*'exclude'"):
            diff(old=old, new=new, key="id", strategy={"exclude": ["missing"]})

    def test_missing_carry_column(self):
        """Req 7.5: missing carry column raises ValueError."""
        old = pl.DataFrame({"id": [1], "val": ["a"]})
        new = pl.DataFrame({"id": [1], "val": ["b"]})
        with pytest.raises(ValueError, match="Column 'missing'.*'carry'"):
            diff(old=old, new=new, key="id", strategy={"carry": ["missing"]})
