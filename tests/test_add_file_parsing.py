"""Tests for .add file parsing — unified TOML format (Task 11.1).

Covers:
- Unified TOML format parsing with explicit inputs and inferred inputs
- Validation of required fields (expression, description, category)
- Rejection of removed fields (sha, requires)
- Expression character validation via _EXPRESSION_SAFE_PATTERN
- Invalid TOML error messages include file path
- Reconciliation parsing remains unchanged
- Requirements: 1.1, 1.2, 1.3, 1.6, 1.7, 1.8, 2.1, 2.4
"""

import tempfile
from pathlib import Path

import pytest

from additory.expressions.loader import (
    Expression,
    InputDef,
    load_add_file,
    format_expression_toml,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _write_add_file(content: str) -> Path:
    """Write content to a temporary .add file and return its Path."""
    f = tempfile.NamedTemporaryFile(mode="w", suffix=".add", delete=False)
    f.write(content)
    f.flush()
    f.close()
    return Path(f.name)


# ---------------------------------------------------------------------------
# Unified format — explicit inputs
# ---------------------------------------------------------------------------

class TestUnifiedFormatExplicitInputs:
    def test_basic_expression_with_inputs(self):
        content = """\
[bmi]
expression = "weight_kg / (height_m ** 2)"
description = "Body Mass Index"
category = "core"
output_column = "bmi"

[bmi.inputs]
weight_kg = { type = "numeric", unit = "kg", description = "Weight in kilograms" }
height_m = { type = "numeric", unit = "m", description = "Height in meters" }
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert "bmi" in result
            expr = result["bmi"]
            assert expr.name == "bmi"
            assert expr.description == "Body Mass Index"
            assert expr.category == "core"
            assert expr.output_column == "bmi"
            assert expr.expression == "weight_kg / (height_m ** 2)"
            assert expr.source_file == str(path)
            # Inputs
            assert set(expr.inputs.keys()) == {"weight_kg", "height_m"}
            assert expr.inputs["weight_kg"].type == "numeric"
            assert expr.inputs["weight_kg"].unit == "kg"
            assert expr.inputs["weight_kg"].description == "Weight in kilograms"
            assert expr.inputs["height_m"].type == "numeric"
            assert expr.inputs["height_m"].unit == "m"
        finally:
            path.unlink()

    def test_categorical_input_type(self):
        content = """\
[encoded]
expression = "value * 1"
description = "Encode group"
category = "core"
output_column = "encoded"

[encoded.inputs]
group = { type = "categorical", description = "Patient group" }
value = { type = "numeric", description = "Measurement" }
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert result["encoded"].inputs["group"].type == "categorical"
            assert result["encoded"].inputs["value"].type == "numeric"
        finally:
            path.unlink()

    def test_multiple_expressions_in_one_file(self):
        content = """\
[profit]
expression = "revenue - cost"
description = "Profit"
category = "finance"

[margin]
expression = "(revenue - cost) / revenue * 100"
description = "Margin pct"
category = "finance"
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert "profit" in result
            assert "margin" in result
            assert result["profit"].category == "finance"
            assert result["margin"].category == "finance"
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Unified format — inferred inputs (no [name.inputs] sub-table)
# ---------------------------------------------------------------------------

class TestUnifiedFormatInferredInputs:
    def test_inputs_inferred_from_formula(self):
        content = """\
[profit]
expression = "revenue - cost"
description = "Profit"
category = "core"
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            expr = result["profit"]
            assert set(expr.inputs.keys()) == {"revenue", "cost"}
            # Inferred inputs default to numeric
            assert expr.inputs["revenue"].type == "numeric"
            assert expr.inputs["cost"].type == "numeric"
        finally:
            path.unlink()

    def test_output_column_defaults_to_name(self):
        content = """\
[total]
expression = "price * qty"
description = "Total"
category = "core"
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert result["total"].output_column == "total"
        finally:
            path.unlink()

    def test_output_column_explicit(self):
        content = """\
[total]
expression = "price * qty"
description = "Total"
category = "core"
output_column = "total_price"
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert result["total"].output_column == "total_price"
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Rejected fields (sha, requires)
# ---------------------------------------------------------------------------

class TestRejectedFields:
    def test_sha_field_rejected(self):
        content = """\
[bmi]
expression = "weight / (height ** 2)"
description = "BMI"
category = "core"
sha = "abc123"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="sha.*no longer supported"):
                load_add_file(path)
        finally:
            path.unlink()

    def test_requires_field_rejected(self):
        content = """\
[bmi]
expression = "weight / (height ** 2)"
description = "BMI"
category = "core"
requires = "numpy"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="requires.*no longer supported"):
                load_add_file(path)
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Missing required fields
# ---------------------------------------------------------------------------

class TestMissingRequiredFields:
    def test_missing_expression_field(self):
        content = """\
[bad]
description = "no formula"
category = "core"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="Missing required field 'expression'"):
                load_add_file(path)
        finally:
            path.unlink()

    def test_missing_description_field(self):
        content = """\
[bad]
expression = "a + b"
category = "core"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="Missing required field 'description'"):
                load_add_file(path)
        finally:
            path.unlink()

    def test_missing_category_field(self):
        content = """\
[bad]
expression = "a + b"
description = "test"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="Missing required field 'category'"):
                load_add_file(path)
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Expression character validation
# ---------------------------------------------------------------------------

class TestExpressionValidation:
    def test_invalid_characters_rejected(self):
        content = """\
[bad]
expression = "x + 1; DROP TABLE"
description = "bad expression"
category = "core"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError, match="invalid characters"):
                load_add_file(path)
        finally:
            path.unlink()

    def test_valid_expression_accepted(self):
        content = """\
[ok]
expression = "a + b * 2 - (a / b)"
description = "valid"
category = "core"
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            assert "ok" in result
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Invalid TOML — error includes file path
# ---------------------------------------------------------------------------

class TestInvalidToml:
    def test_invalid_toml_error_includes_path(self):
        content = """\
[bad
expression = "a + b"
"""
        path = _write_add_file(content)
        try:
            with pytest.raises(ValueError) as exc_info:
                load_add_file(path)
            assert str(path) in str(exc_info.value)
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Reconciliation parsing unchanged
# ---------------------------------------------------------------------------

class TestReconciliationParsing:
    def test_reconciliation_file_returns_empty_expressions(self):
        content = """\
[reconciliation]
name = "test_recon"
description = "Test reconciliation"

[aliases]
weight = ["mass", "wt"]

[groups]
vitals = ["weight", "height"]
"""
        path = _write_add_file(content)
        try:
            result = load_add_file(path)
            # Reconciliation files return empty dict (no expressions)
            assert result == {}
        finally:
            path.unlink()


# ---------------------------------------------------------------------------
# Pretty printer round-trip (format_expression_toml)
# ---------------------------------------------------------------------------

class TestFormatExpressionToml:
    """Tests for format_expression_toml() pretty printer."""

    def test_basic_round_trip(self):
        """Format an expression, parse it back, and verify equivalence."""
        expr = Expression(
            name="bmi",
            expression="weight_kg / (height_m ** 2)",
            description="Body Mass Index",
            category="core",
            output_column="bmi",
            inputs={
                "weight_kg": InputDef(type="numeric", unit="kg", description="Weight in kilograms"),
                "height_m": InputDef(type="numeric", unit="m", description="Height in meters"),
            },
        )

        toml_str = format_expression_toml(expr)
        path = _write_add_file(toml_str)
        try:
            result = load_add_file(path)
            assert "bmi" in result
            parsed = result["bmi"]
            assert parsed.name == "bmi"
            assert parsed.description == "Body Mass Index"
            assert parsed.category == "core"
            assert parsed.output_column == "bmi"
            assert parsed.expression == "weight_kg / (height_m ** 2)"
            assert set(parsed.inputs.keys()) == {"weight_kg", "height_m"}
            for k in expr.inputs:
                assert parsed.inputs[k].type == expr.inputs[k].type
                assert parsed.inputs[k].unit == expr.inputs[k].unit
                assert parsed.inputs[k].description == expr.inputs[k].description
        finally:
            path.unlink()

    def test_round_trip_output_column_differs_from_name(self):
        """output_column != name should be preserved."""
        expr = Expression(
            name="calc",
            expression="x + 1",
            description="A calculation",
            category="core",
            output_column="result",
            inputs={"x": InputDef(type="numeric", description="X value")},
        )
        toml_str = format_expression_toml(expr)
        path = _write_add_file(toml_str)
        try:
            result = load_add_file(path)
            assert result["calc"].output_column == "result"
        finally:
            path.unlink()

    def test_round_trip_single_input(self):
        """Round-trip with a single input column."""
        expr = Expression(
            name="double",
            expression="val * 2",
            description="Double it",
            category="core",
            output_column="doubled",
            inputs={"val": InputDef(type="numeric", description="The value")},
        )
        toml_str = format_expression_toml(expr)
        path = _write_add_file(toml_str)
        try:
            result = load_add_file(path)
            parsed = result["double"]
            assert parsed.expression == "val * 2"
            assert parsed.output_column == "doubled"
            assert list(parsed.inputs.keys()) == ["val"]
        finally:
            path.unlink()
