"""Unit tests for alias resolution, group resolution, and .add file parsing.

Covers: alias matching, canonical name output, case-insensitive matching,
group-aware hierarchical detection, .add file parsing, round-trip, missing name.
"""

import sys
sys.path.insert(0, "additory")

import polars as pl
import pytest
import tempfile
from pathlib import Path

from additory.diff_engine import (
    diff,
    _apply_aliases,
    _resolve_reconciliation,
    StrategyConfig,
)
from additory.expressions.loader import (
    ReconciliationDef,
    load_reconciliation_from_file,
    format_reconciliation_add_file,
    _load_reconciliation_add_file,
    resolve_reconciliation_by_name,
)


# ── Alias matching ───────────────────────────────────────────────────

class TestAliasMatching:
    def test_alias_renames_columns(self):
        """Req 8.1: alias variants are treated as the same column."""
        old = pl.DataFrame({"id": [1], "ALT": [10]})
        new = pl.DataFrame({"id": [1], "SGOT": [20]})
        result = diff(
            old=old, new=new, key="id",
            strategy={"aliases": {"ALT": ["SGOT"]}},
        )
        # Both should be renamed to canonical "ALT"
        assert "ALT" in result.columns
        assert "SGOT" not in result.columns

    def test_alias_canonical_name_in_output(self):
        """Req 8.5: output uses canonical name."""
        old = pl.DataFrame({"id": [1], "ALT": [10]})
        new = pl.DataFrame({"id": [1], "sgot": [20]})
        result = diff(
            old=old, new=new, key="id",
            strategy={"aliases": {"ALT": ["sgot"]}},
        )
        assert "ALT" in result.columns

    def test_case_insensitive_alias(self):
        """Req 8.3: case-insensitive alias matching."""
        old = pl.DataFrame({"id": [1], "ALT": [10]})
        new = pl.DataFrame({"id": [1], "Sgot": [20]})
        aliases = {"ALT": ["SGOT"]}
        old_r, new_r = _apply_aliases(old, new, aliases)
        assert "ALT" in new_r.columns

    def test_unresolved_alias_name(self):
        """Req 8.4: unresolved alias name raises ValueError."""
        config = StrategyConfig(aliases="nonexistent_alias")
        with pytest.raises(ValueError, match="nonexistent_alias.*not found"):
            _resolve_reconciliation(config)


# ── Group-aware hierarchical detection ───────────────────────────────

class TestGroupDetection:
    def test_hierarchical_change_detected(self):
        """Req 9.2: parent-child relationship recognized."""
        old = pl.DataFrame({"id": [1], "category": ["Biochemistry"]})
        new = pl.DataFrame({"id": [1], "category": ["creatinine"]})
        # Use inline aliases (empty) and groups via a temp .add file
        # For this test, we pass groups directly through _classify_rows
        from additory.diff_engine import _classify_rows
        groups = {"Biochemistry": ["creatinine", "alt", "ast"]}
        result = _classify_rows(
            old, new, ["id"],
            exclude_cols=[], carry_cols=[],
            groups=groups,
        )
        assert len(result.changed_rows) == 1
        assert result.changed_rows[0].changes[0].is_hierarchical is True

    def test_unresolved_group_name(self):
        """Req 9.4: unresolved group name raises ValueError."""
        config = StrategyConfig(groups="nonexistent_group")
        with pytest.raises(ValueError, match="nonexistent_group.*not found"):
            _resolve_reconciliation(config)


# ── .add file parsing ────────────────────────────────────────────────

class TestAddFileParsing:
    def test_parse_reconciliation_file(self):
        """Req 10.1, 10.2, 10.3: parse [reconciliation] with aliases and groups."""
        content = '''[reconciliation]
name = "lab_aliases"
description = "Lab test aliases"

[aliases]
ALT = ["SGOT", "alanine_transaminase"]
AST = ["SGPT"]

[groups]
Biochemistry = ["creatinine", "alt", "ast"]
'''
        with tempfile.NamedTemporaryFile(suffix=".add", mode="w", delete=False) as f:
            f.write(content)
            f.flush()
            recon = load_reconciliation_from_file(Path(f.name))

        assert recon is not None
        assert recon.name == "lab_aliases"
        assert recon.description == "Lab test aliases"
        assert recon.aliases["ALT"] == ["SGOT", "alanine_transaminase"]
        assert recon.aliases["AST"] == ["SGPT"]
        assert recon.groups["Biochemistry"] == ["creatinine", "alt", "ast"]

    def test_reconciliation_only_file(self):
        """Req 10.5: .add file with only [reconciliation] (no [expression])."""
        content = '''[reconciliation]
name = "simple"
description = "Simple reconciliation"

[aliases]
A = ["a1", "a2"]
'''
        with tempfile.NamedTemporaryFile(suffix=".add", mode="w", delete=False) as f:
            f.write(content)
            f.flush()
            recon = load_reconciliation_from_file(Path(f.name))

        assert recon is not None
        assert recon.name == "simple"

    def test_missing_name_field(self):
        """Req 10.4: missing name field raises ValueError."""
        content = '''[reconciliation]
description = "No name"

[aliases]
A = ["a1"]
'''
        with tempfile.NamedTemporaryFile(suffix=".add", mode="w", delete=False) as f:
            f.write(content)
            f.flush()
            with pytest.raises(ValueError, match="Missing required field 'name'"):
                _load_reconciliation_add_file(Path(f.name), content)

    def test_round_trip(self):
        """Req 10.6: format → parse round-trip."""
        aliases = {"ALT": ["SGOT", "sgpt"], "WBC": ["leukocytes"]}
        groups = {"Biochemistry": ["creatinine", "alt"]}
        content = format_reconciliation_add_file(
            name="test_recon",
            description="Test reconciliation",
            aliases=aliases,
            groups=groups,
        )
        with tempfile.NamedTemporaryFile(suffix=".add", mode="w", delete=False) as f:
            f.write(content)
            f.flush()
            recon = load_reconciliation_from_file(Path(f.name))

        assert recon.name == "test_recon"
        assert recon.description == "Test reconciliation"
        assert recon.aliases == aliases
        assert recon.groups == groups

    def test_non_reconciliation_file_returns_none(self):
        """Non-reconciliation .add file returns None."""
        content = '''[my_expr]
expression = "a + b"
description = "Simple expression"
'''
        with tempfile.NamedTemporaryFile(suffix=".add", mode="w", delete=False) as f:
            f.write(content)
            f.flush()
            recon = load_reconciliation_from_file(Path(f.name))

        assert recon is None


# ── Alias resolution from .add file ─────────────────────────────────

class TestAliasFromFile:
    def test_resolve_from_user_folder(self, tmp_path):
        """Req 8.2: resolve aliases from .add file via registry."""
        content = '''[reconciliation]
name = "my_aliases"
description = "Test"

[aliases]
ALT = ["SGOT"]
'''
        add_file = tmp_path / "recon.add"
        add_file.write_text(content)

        # Set user folder
        from additory.expressions.loader import get_registry
        registry = get_registry()
        old_folder = registry.user_folder
        try:
            registry.set_user_folder(str(tmp_path))
            recon = resolve_reconciliation_by_name("my_aliases")
            assert recon is not None
            assert recon.aliases["ALT"] == ["SGOT"]
        finally:
            registry.user_folder = old_folder
