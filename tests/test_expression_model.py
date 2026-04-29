"""Tests for Expression and InputDef dataclasses (Task 11.4).

Updated for unified TOML format:
- No `sha`, no `format`, no `compute_sha256`
- Added `category` field on Expression
- Added `unit` field on InputDef
- Requirements: 1.1, 2.2, 7.4, 7.5
"""

import tempfile
from pathlib import Path

from additory.expressions.loader import (
    Expression,
    InputDef,
    _extract_identifiers,
    load_add_file,
)


class TestInputDef:
    def test_defaults(self):
        inp = InputDef()
        assert inp.type == "numeric"
        assert inp.unit == ""
        assert inp.description == ""

    def test_custom_values(self):
        inp = InputDef(type="categorical", unit="kg", description="Patient weight")
        assert inp.type == "categorical"
        assert inp.unit == "kg"
        assert inp.description == "Patient weight"

    def test_unit_field(self):
        inp = InputDef(unit="m/s")
        assert inp.unit == "m/s"
        assert inp.type == "numeric"  # default


class TestExpressionFields:
    def test_output_column_defaults_to_name(self):
        expr = Expression(
            name="bmi", expression="w / (h ** 2)", description="BMI", category="core"
        )
        assert expr.output_column == "bmi"
        assert expr.inputs == {}
        assert expr.source_file is None

    def test_output_column_explicit(self):
        expr = Expression(
            name="bmi",
            expression="w / (h ** 2)",
            description="BMI",
            category="core",
            output_column="body_mass_index",
        )
        assert expr.output_column == "body_mass_index"

    def test_category_field(self):
        expr = Expression(
            name="bmi", expression="w / (h ** 2)", description="BMI", category="medical"
        )
        assert expr.category == "medical"

    def test_category_defaults_to_empty(self):
        expr = Expression(name="x", expression="a + b", description="test")
        assert expr.category == ""

    def test_inputs_populated(self):
        inputs = {
            "weight": InputDef(type="numeric", unit="kg", description="kg"),
            "height": InputDef(type="numeric", unit="m", description="m"),
        }
        expr = Expression(
            name="bmi",
            expression="weight / (height ** 2)",
            description="BMI",
            category="core",
            inputs=inputs,
        )
        assert "weight" in expr.inputs
        assert "height" in expr.inputs
        assert expr.inputs["weight"].unit == "kg"
        assert expr.inputs["weight"].description == "kg"

    def test_source_file_field(self):
        expr = Expression(
            name="x",
            expression="a + b",
            description="test",
            category="core",
            source_file="/tmp/test.add",
        )
        assert expr.source_file == "/tmp/test.add"

    def test_repr_unchanged(self):
        expr = Expression(
            name="bmi", expression="w / (h ** 2)", description="BMI", category="core"
        )
        assert "Expression(name='bmi'" in repr(expr)


class TestExtractIdentifiers:
    def test_simple_formula(self):
        assert _extract_identifiers("a + b") == ["a", "b"]

    def test_deduplication(self):
        assert _extract_identifiers("a + a * b") == ["a", "b"]

    def test_excludes_known_functions(self):
        result = _extract_identifiers("abs(x) + sqrt(y)")
        assert "abs" not in result
        assert "sqrt" not in result
        assert result == ["x", "y"]

    def test_excludes_today(self):
        result = _extract_identifiers("(today() - birth_date).days / 365.25")
        assert "today" not in result
        assert "birth_date" in result

    def test_complex_formula(self):
        result = _extract_identifiers("weight / (height ** 2)")
        assert result == ["weight", "height"]

    def test_numeric_literals_excluded(self):
        result = _extract_identifiers("0.007184 * (height ** 0.725) * (weight ** 0.425)")
        assert result == ["height", "weight"]

    def test_empty_formula(self):
        assert _extract_identifiers("") == []

    def test_preserves_order(self):
        result = _extract_identifiers("z + a + m")
        assert result == ["z", "a", "m"]


class TestLoadAddFileUnifiedFormat:
    def test_unified_format_populates_all_fields(self):
        content = """\
[bmi]
expression = "weight / (height ** 2)"
description = "Body Mass Index"
category = "core"
"""
        with tempfile.NamedTemporaryFile(mode="w", suffix=".add", delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)

        try:
            expressions = load_add_file(path)
            expr = expressions["bmi"]

            assert expr.output_column == "bmi"
            assert expr.category == "core"
            assert expr.source_file == str(path)
            assert "weight" in expr.inputs
            assert "height" in expr.inputs
            assert expr.inputs["weight"].type == "numeric"
            assert expr.inputs["height"].type == "numeric"
        finally:
            path.unlink()

    def test_multiple_expressions_in_file(self):
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
        with tempfile.NamedTemporaryFile(mode="w", suffix=".add", delete=False) as f:
            f.write(content)
            f.flush()
            path = Path(f.name)

        try:
            expressions = load_add_file(path)

            assert expressions["profit"].output_column == "profit"
            assert expressions["profit"].category == "finance"
            assert set(expressions["profit"].inputs.keys()) == {"revenue", "cost"}

            assert expressions["margin"].output_column == "margin"
            assert expressions["margin"].category == "finance"
            assert set(expressions["margin"].inputs.keys()) == {"revenue", "cost"}
        finally:
            path.unlink()
