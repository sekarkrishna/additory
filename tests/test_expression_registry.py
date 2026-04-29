"""Tests for ExpressionRegistry - unified TOML format (Task 11.2).

Covers:
- Registry loads all inbuilt files without errors
- resolve_by_name / list_all_names
- User folder override behavior
- Reserved name validation
- Requirements: 4.8, 4.9, 6.5
"""

import tempfile
from pathlib import Path

import pytest

from additory.expressions.loader import (
    ExpressionRegistry,
    Expression,
    InputDef,
    RESERVED_NAMES,
    get_registry,
    _scan_folder_for_expressions,
)


class TestRegistryLoadsInbuilt:
    def test_loads_without_errors(self):
        reg = ExpressionRegistry()
        assert len(reg.inbuilt) > 0

    def test_all_inbuilt_have_category(self):
        reg = ExpressionRegistry()
        for name, expr in reg.inbuilt.items():
            assert expr.category, f"Expression '{name}' has empty category"

    def test_bmi_is_inbuilt(self):
        reg = ExpressionRegistry()
        assert "bmi" in reg.inbuilt
        assert reg.inbuilt["bmi"].category == "core"


class TestResolveByName:
    def test_resolves_inbuilt_expression(self):
        reg = ExpressionRegistry()
        expr = reg.resolve_by_name("bmi")
        assert expr is not None
        assert expr.name == "bmi"

    def test_returns_none_for_unknown(self):
        reg = ExpressionRegistry()
        assert reg.resolve_by_name("nonexistent_xyz_42") is None

    def test_user_folder_overrides_inbuilt(self):
        """User folder version wins when both define the same name."""
        reg = ExpressionRegistry()
        with tempfile.TemporaryDirectory() as td:
            Path(td, "custom.add").write_text(
                '[bmi]\nexpression = "w / (h * h)"\n'
                'description = "Custom BMI"\ncategory = "custom"\n'
            )
            reg.set_user_folder(td)
            expr = reg.resolve_by_name("bmi")
            assert expr is not None
            assert expr.expression == "w / (h * h)"
            assert "Custom BMI" in expr.description

    def test_fresh_scan_picks_up_new_file(self):
        """resolve_by_name re-scans on each call (no caching)."""
        reg = ExpressionRegistry()
        with tempfile.TemporaryDirectory() as td:
            reg.set_user_folder(td)
            assert reg.resolve_by_name("fresh_test") is None
            Path(td, "fresh.add").write_text(
                '[fresh_test]\nexpression = "a + b"\n'
                'description = "Added after set_user_folder"\n'
                'category = "custom"\n'
            )
            expr = reg.resolve_by_name("fresh_test")
            assert expr is not None
            assert expr.name == "fresh_test"


class TestListAllNames:
    def test_includes_inbuilt(self):
        reg = ExpressionRegistry()
        names = reg.list_all_names()
        assert "bmi" in names

    def test_includes_user_expressions(self):
        reg = ExpressionRegistry()
        with tempfile.TemporaryDirectory() as td:
            Path(td, "extra.add").write_text(
                '[my_custom_expr]\nexpression = "x + y"\n'
                'description = "test"\ncategory = "custom"\n'
            )
            reg.set_user_folder(td)
            names = reg.list_all_names()
            assert "my_custom_expr" in names
            assert "bmi" in names


class TestReservedNames:
    @pytest.mark.parametrize("reserved", sorted(RESERVED_NAMES))
    def test_reserved_name_in_user_folder_raises(self, reserved):
        with tempfile.TemporaryDirectory() as td:
            content = f'[{reserved}]\nexpression = "a + b"\ndescription = "should fail"\ncategory = "test"\n'
            Path(td, "bad.add").write_text(content)
            with pytest.raises(ValueError, match="reserved"):
                _scan_folder_for_expressions(Path(td))

    def test_non_reserved_name_ok(self):
        with tempfile.TemporaryDirectory() as td:
            Path(td, "ok.add").write_text(
                '[my_calc]\nexpression = "a + b"\n'
                'description = "fine"\ncategory = "test"\n'
            )
            exprs = _scan_folder_for_expressions(Path(td))
            assert "my_calc" in exprs
