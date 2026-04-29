"""Tests for dynamic expression functions (Task 11.3).

These tests verify __getattr__ resolution, _make_dynamic_function,
column auto-matching, explicit mapping, and error messages.

Updated for unified TOML format: Expression objects have no `sha`, no `format`,
and include `category`.

Note: The actual transform('@calc', ...) delegation requires Rust, so
we test the Python-side wiring up to the Rust call boundary.
"""

import textwrap
from pathlib import Path
from unittest.mock import patch

import pytest

import additory as add
import additory.expressions.loader as _loader
from additory import _make_dynamic_function, __getattr__ as _module_getattr
from additory.expressions.loader import (
    Expression,
    InputDef,
    ExpressionRegistry,
    RESERVED_NAMES,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _fresh_registry():
    """Reset the global registry so tests are isolated."""
    _loader._registry = None
    return _loader.get_registry()


# ---------------------------------------------------------------------------
# __getattr__ resolution
# ---------------------------------------------------------------------------

class TestModuleGetattr:
    def test_resolves_known_expression(self):
        _fresh_registry()
        fn = _module_getattr("bmi")
        assert callable(fn)
        assert fn.__name__ == "bmi"

    def test_raises_attribute_error_for_unknown(self):
        _fresh_registry()
        with pytest.raises(AttributeError, match="no attribute 'zzz_nonexistent'"):
            _module_getattr("zzz_nonexistent")

    def test_raises_for_reserved_name(self):
        _fresh_registry()
        for name in RESERVED_NAMES:
            with pytest.raises(AttributeError):
                _module_getattr(name)

    def test_raises_for_underscore_prefix(self):
        _fresh_registry()
        with pytest.raises(AttributeError):
            _module_getattr("_private")

    def test_error_lists_available_expressions(self):
        _fresh_registry()
        with pytest.raises(AttributeError, match="Available expressions"):
            _module_getattr("zzz_nonexistent")


# ---------------------------------------------------------------------------
# _make_dynamic_function factory
# ---------------------------------------------------------------------------

class TestMakeDynamicFunction:
    def test_sets_name_and_doc(self):
        expr = Expression(
            name="test_expr",
            expression="a + b",
            description="Test expression",
            category="core",
            inputs={"a": InputDef(), "b": InputDef()},
        )
        fn = _make_dynamic_function("test_expr", expr)
        assert fn.__name__ == "test_expr"
        assert "Test expression" in fn.__doc__
        assert "a + b" in fn.__doc__

    def test_rejects_non_dataframe(self):
        expr = Expression(
            name="test_expr",
            expression="a + b",
            description="Test",
            category="core",
            inputs={"a": InputDef(), "b": InputDef()},
        )
        fn = _make_dynamic_function("test_expr", expr)
        with pytest.raises(TypeError, match="DataFrame"):
            fn("not a dataframe")


# ---------------------------------------------------------------------------
# Column auto-matching and explicit mapping
# ---------------------------------------------------------------------------

class TestColumnMapping:
    def _make_fn(self):
        """Create a dynamic function for 'weight / (height ** 2)'."""
        expr = Expression(
            name="bmi",
            expression="weight / (height ** 2)",
            description="BMI",
            category="core",
            output_column="bmi",
            inputs={
                "weight": InputDef(type="numeric", description="kg"),
                "height": InputDef(type="numeric", description="m"),
            },
        )
        return _make_dynamic_function("bmi", expr)

    def test_auto_match_columns_present(self):
        """When df has matching columns, auto-match should work (up to Rust boundary)."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"weight": [70.0], "height": [1.75]})

        if not add.RUST_AVAILABLE:
            with pytest.raises((RuntimeError, ImportError, NotImplementedError)):
                fn(df)
        else:
            result = fn(df)
            assert "bmi" in result.columns

    def test_explicit_mapping(self):
        """Explicit kwargs should remap column names."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"w": [70.0], "h": [1.75]})

        if not add.RUST_AVAILABLE:
            with pytest.raises((RuntimeError, ImportError, NotImplementedError)):
                fn(df, weight="w", height="h")
        else:
            result = fn(df, weight="w", height="h")
            assert "bmi" in result.columns

    def test_partial_mapping(self):
        """Mix of auto-match and explicit mapping."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"weight": [70.0], "h": [1.75]})

        if not add.RUST_AVAILABLE:
            with pytest.raises((RuntimeError, ImportError, NotImplementedError)):
                fn(df, height="h")
        else:
            result = fn(df, height="h")
            assert "bmi" in result.columns

    def test_auto_match_failure_error_message(self):
        """Missing columns without mapping should give a helpful error."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"w": [70.0], "h": [1.75]})

        with pytest.raises(ValueError, match="Column auto-matching failed"):
            fn(df)

    def test_auto_match_error_contains_details(self):
        """Error should list expression name, required cols, actual cols, suggestion."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"w": [70.0], "h": [1.75]})

        with pytest.raises(ValueError) as exc_info:
            fn(df)
        msg = str(exc_info.value)
        assert "bmi" in msg
        assert "weight" in msg
        assert "height" in msg
        assert "add.bmi" in msg

    def test_unknown_mapping_keyword_rejected(self):
        """Kwargs that don't match any input name should be rejected."""
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"weight": [70.0], "height": [1.75]})

        with pytest.raises(ValueError, match="Unknown column mapping"):
            fn(df, nonexistent="col")

    def test_unknown_mapping_lists_valid_names(self):
        import polars as pl
        fn = self._make_fn()
        df = pl.DataFrame({"weight": [70.0], "height": [1.75]})

        with pytest.raises(ValueError, match="weight") as exc_info:
            fn(df, nonexistent="col")
        assert "height" in str(exc_info.value)


# ---------------------------------------------------------------------------
# Expression evaluation wiring (transform delegation)
# ---------------------------------------------------------------------------

class TestTransformDelegation:
    def test_formula_rewrite_with_mapping(self):
        """Verify the formula is rewritten correctly before delegation."""
        import polars as pl

        expr = Expression(
            name="total",
            expression="price * qty",
            description="Total",
            category="core",
            output_column="total",
            inputs={
                "price": InputDef(),
                "qty": InputDef(),
            },
        )
        fn = _make_dynamic_function("total", expr)
        df = pl.DataFrame({"p": [10.0], "q": [5.0]})

        with patch("additory.transform") as mock_transform:
            mock_transform.return_value = df
            fn(df, price="p", qty="q")

            mock_transform.assert_called_once()
            call_kwargs = mock_transform.call_args
            assert call_kwargs[1]["expression"] == "p * q"
            assert call_kwargs[1]["name"] == "total"

    def test_position_kwarg_forwarded(self):
        import polars as pl

        expr = Expression(
            name="x",
            expression="a + b",
            description="test",
            category="core",
            output_column="x",
            inputs={"a": InputDef(), "b": InputDef()},
        )
        fn = _make_dynamic_function("x", expr)
        df = pl.DataFrame({"a": [1.0], "b": [2.0]})

        with patch("additory.transform") as mock_transform:
            mock_transform.return_value = df
            fn(df, position="start")
            assert mock_transform.call_args[1]["position"] == "start"

    def test_logging_lineage_as_type_forwarded(self):
        import polars as pl

        expr = Expression(
            name="x",
            expression="a + b",
            description="test",
            category="core",
            output_column="x",
            inputs={"a": InputDef(), "b": InputDef()},
        )
        fn = _make_dynamic_function("x", expr)
        df = pl.DataFrame({"a": [1.0], "b": [2.0]})

        with patch("additory.transform") as mock_transform:
            mock_transform.return_value = df
            fn(df, logging=True, lineage=True, as_type="pandas")
            kw = mock_transform.call_args[1]
            assert kw["logging"] is True
            assert kw["lineage"] is True
            assert kw["as_type"] == "pandas"

    def test_default_position_is_end(self):
        import polars as pl

        expr = Expression(
            name="x",
            expression="a + b",
            description="test",
            category="core",
            output_column="x",
            inputs={"a": InputDef(), "b": InputDef()},
        )
        fn = _make_dynamic_function("x", expr)
        df = pl.DataFrame({"a": [1.0], "b": [2.0]})

        with patch("additory.transform") as mock_transform:
            mock_transform.return_value = df
            fn(df)
            assert mock_transform.call_args[1]["position"] == "end"
