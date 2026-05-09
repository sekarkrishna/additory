"""
Tests for add.to() pipe compatibility.

Verifies that:
- add.to() accepts a DataFrame as its first positional argument (bring_to)
- df.pipe(add.to, ...) produces identical output to add.to(df, ...)
- logging=True emits a log message when the call is received

Requirements: 1.1, 1.2, 1.3, 1.4
"""

import inspect
import logging
import pytest

import additory as add


class TestToSignaturePipeCompatible:
    """Verify the to() function signature is pipe-compatible."""

    def test_first_param_is_bring_to(self):
        """The first positional parameter of add.to() must be 'bring_to'."""
        sig = inspect.signature(add.to)
        params = list(sig.parameters.keys())
        assert params[0] == "bring_to", (
            f"Expected first parameter to be 'bring_to', got '{params[0]}'"
        )

    def test_bring_to_is_positional(self):
        """bring_to must accept positional arguments (required for .pipe())."""
        sig = inspect.signature(add.to)
        param = sig.parameters["bring_to"]
        assert param.kind in (
            inspect.Parameter.POSITIONAL_ONLY,
            inspect.Parameter.POSITIONAL_OR_KEYWORD,
        ), "bring_to must be positional (required for DataFrame.pipe())"

    def test_bring_to_has_no_default(self):
        """bring_to must be required (no default value)."""
        sig = inspect.signature(add.to)
        param = sig.parameters["bring_to"]
        assert param.default is inspect.Parameter.empty, (
            "bring_to should be a required parameter with no default"
        )


class TestToPipeEquivalence:
    """Verify pipe call produces identical output to direct call.

    Because the Rust backend (_additory) may not be compiled in every
    environment, these tests are skipped when it is unavailable.
    """

    @pytest.fixture
    def sample_data(self):
        """Create sample DataFrames for testing."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({
            "id": [1, 2, 3],
            "name": ["Alice", "Bob", "Charlie"],
        })
        ref = pl.DataFrame({
            "id": [1, 2, 3],
            "age": [30, 25, 35],
        })
        return df, ref

    @pytest.mark.skipif(
        not add.RUST_AVAILABLE,
        reason="Rust bindings not available",
    )
    def test_pipe_equals_direct_call_polars(self, sample_data):
        """df.pipe(add.to, ...) must equal add.to(df, ...) for polars."""
        pl = pytest.importorskip("polars")
        df, ref = sample_data

        direct = add.to(df, bring_from=ref, bring=["age"], against="id")
        piped = df.pipe(add.to, bring_from=ref, bring=["age"], against="id")

        assert direct.equals(piped), (
            "Pipe call and direct call produced different results"
        )

    @pytest.mark.skipif(
        not add.RUST_AVAILABLE,
        reason="Rust bindings not available",
    )
    def test_pipe_equals_direct_call_pandas(self, sample_data):
        """df.pipe(add.to, ...) must equal add.to(df, ...) for pandas."""
        pd = pytest.importorskip("pandas")
        pl = pytest.importorskip("polars")
        df_pl, ref_pl = sample_data

        df = df_pl.to_pandas()
        ref = ref_pl.to_pandas()

        direct = add.to(df, bring_from=ref, bring=["age"], against="id")
        piped = df.pipe(add.to, bring_from=ref, bring=["age"], against="id")

        pd.testing.assert_frame_equal(direct, piped)


class TestToLogging:
    """Verify logging behaviour when logging=True."""

    def test_logging_emits_info_message(self, caplog):
        """When logging=True, add.to() should log an info message before
        doing any real work. We expect the ImportError (no Rust) but the
        log line should already have been emitted."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({"id": [1]})
        ref = pl.DataFrame({"id": [1], "age": [30]})

        with caplog.at_level(logging.INFO, logger="additory"):
            try:
                add.to(df, bring_from=ref, bring=["age"], against="id", logging=True)
            except ImportError:
                pass  # Rust not available — that's fine

        assert any("add.to() called" in rec.message for rec in caplog.records), (
            "Expected an INFO log containing 'add.to() called'"
        )
