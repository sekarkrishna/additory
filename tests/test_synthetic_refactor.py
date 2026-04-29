"""
Tests for add.synthetic() signature refactor.

Verifies:
- First arg accepts DataFrame (augment mode) or '@new' (new mode)
- Mode inference works correctly
- Old mode= keyword raises TypeError with migration message
- Invalid strings raise ValueError with clear message
- logging=True in @new mode logs non-pipeable info
- Signature no longer has a 'mode' parameter

Requirements: 2.1, 2.2, 2.3, 2.5, 2.6, 2.7, 2.8
"""

import inspect
import logging
import pytest

import additory as add


class TestSyntheticSignatureRefactor:
    """Verify the new synthetic() signature shape."""

    def test_first_param_is_df_or_mode(self):
        """First positional parameter must be 'df_or_mode'."""
        sig = inspect.signature(add.synthetic)
        params = list(sig.parameters.keys())
        assert params[0] == "df_or_mode", (
            f"Expected first parameter to be 'df_or_mode', got '{params[0]}'"
        )

    def test_df_or_mode_is_positional(self):
        """df_or_mode must accept positional arguments (required for .pipe())."""
        sig = inspect.signature(add.synthetic)
        param = sig.parameters["df_or_mode"]
        assert param.kind in (
            inspect.Parameter.POSITIONAL_ONLY,
            inspect.Parameter.POSITIONAL_OR_KEYWORD,
        ), "df_or_mode must be positional (required for DataFrame.pipe())"

    def test_no_mode_parameter(self):
        """The old 'mode' parameter must not exist in the signature.
        Requirement 2.7: remove mode parameter entirely."""
        sig = inspect.signature(add.synthetic)
        # 'mode' should not be a named parameter in the signature
        # (it may appear in **kwargs but not as a declared param)
        declared_params = {
            name for name, p in sig.parameters.items()
            if p.kind != inspect.Parameter.VAR_KEYWORD
        }
        assert "mode" not in declared_params, (
            "The 'mode' parameter should be removed from the signature"
        )

    def test_n_is_second_positional(self):
        """n should be the second positional parameter."""
        sig = inspect.signature(add.synthetic)
        params = list(sig.parameters.keys())
        assert params[1] == "n", (
            f"Expected second parameter to be 'n', got '{params[1]}'"
        )

    def test_strategy_is_keyword_only(self):
        """strategy must be keyword-only."""
        sig = inspect.signature(add.synthetic)
        param = sig.parameters["strategy"]
        assert param.kind == inspect.Parameter.KEYWORD_ONLY


class TestSyntheticModeInference:
    """Verify mode is correctly inferred from the first argument."""

    def test_dataframe_infers_augment_mode_polars(self):
        """Passing a polars DataFrame should infer augment mode.
        Requirement 2.1, 2.2."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({"a": [1, 2, 3]})

        # Should not raise ValueError about invalid mode — it should
        # get past mode inference. It may raise ImportError (no Rust)
        # or ValueError about missing 'n', both of which confirm
        # augment mode was inferred.
        with pytest.raises((ImportError, ValueError)) as exc_info:
            add.synthetic(df)

        err_msg = str(exc_info.value)
        # Should NOT say "Invalid first argument"
        assert "Invalid first argument" not in err_msg

    def test_dataframe_infers_augment_mode_pandas(self):
        """Passing a pandas DataFrame should infer augment mode.
        Requirement 2.1, 2.2."""
        pd = pytest.importorskip("pandas")
        df = pd.DataFrame({"a": [1, 2, 3]})

        with pytest.raises((ImportError, ValueError)) as exc_info:
            add.synthetic(df)

        err_msg = str(exc_info.value)
        assert "Invalid first argument" not in err_msg

    def test_at_new_string_infers_new_mode(self):
        """Passing '@new' should infer new mode.
        Requirement 2.1, 2.3."""
        with pytest.raises((ImportError, ValueError)) as exc_info:
            add.synthetic('@new')

        err_msg = str(exc_info.value)
        assert "Invalid first argument" not in err_msg


class TestSyntheticInvalidFirstArg:
    """Verify ValueError for invalid string arguments.
    Requirement 2.6, 9.3."""

    def test_invalid_string_raises_valueerror(self):
        """A string other than '@new' should raise ValueError."""
        with pytest.raises(ValueError) as exc_info:
            add.synthetic('@augment')

        err_msg = str(exc_info.value)
        assert "@augment" in err_msg
        assert "DataFrame" in err_msg
        assert "@new" in err_msg

    def test_random_string_raises_valueerror(self):
        """An arbitrary string should raise ValueError."""
        with pytest.raises(ValueError) as exc_info:
            add.synthetic('foobar')

        err_msg = str(exc_info.value)
        assert "foobar" in err_msg
        assert "@new" in err_msg

    def test_non_dataframe_non_string_raises_valueerror(self):
        """Passing something that is neither DataFrame nor string should raise ValueError."""
        with pytest.raises(ValueError) as exc_info:
            add.synthetic(42)

        err_msg = str(exc_info.value)
        assert "int" in err_msg

    def test_none_raises_valueerror(self):
        """Passing None should raise ValueError."""
        with pytest.raises(ValueError) as exc_info:
            add.synthetic(None)

        err_msg = str(exc_info.value)
        assert "NoneType" in err_msg


class TestSyntheticOldModeKwarg:
    """Verify TypeError for old mode= keyword usage.
    Requirement 2.8."""

    def test_old_mode_augment_raises_typeerror(self):
        """Using mode='@augment' should raise TypeError with migration message."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({"a": [1, 2, 3]})

        with pytest.raises(TypeError) as exc_info:
            add.synthetic(df, n=10, mode='@augment')

        err_msg = str(exc_info.value)
        assert "no longer accepts 'mode'" in err_msg
        assert "New usage" in err_msg

    def test_old_mode_new_raises_typeerror(self):
        """Using mode='@new' should raise TypeError with migration message."""
        with pytest.raises(TypeError) as exc_info:
            add.synthetic('@new', n=10, mode='@new', strategy={'a': 'normal:mean=0:std=1'})

        err_msg = str(exc_info.value)
        assert "no longer accepts 'mode'" in err_msg


class TestSyntheticNewModeLogging:
    """Verify logging behaviour for @new mode.
    Requirement 2.5."""

    def test_new_mode_logs_not_pipeable(self, caplog):
        """When logging=True and mode is @new, should log that it's not pipeable."""
        with caplog.at_level(logging.INFO, logger="additory"):
            try:
                add.synthetic('@new', n=10,
                              strategy={'a': 'normal:mean=0:std=1'},
                              logging=True)
            except (ImportError, RuntimeError):
                pass  # Rust not available — that's fine

        assert any(
            "not pipeable" in rec.message.lower()
            for rec in caplog.records
        ), "Expected an INFO log mentioning 'not pipeable' for @new mode"

    def test_augment_mode_does_not_log_not_pipeable(self, caplog):
        """Augment mode should NOT log the non-pipeable warning."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({"a": [1, 2, 3]})

        with caplog.at_level(logging.INFO, logger="additory"):
            try:
                add.synthetic(df, n=5, logging=True)
            except (ImportError, RuntimeError):
                pass

        assert not any(
            "not pipeable" in rec.message.lower()
            for rec in caplog.records
        ), "Augment mode should not log 'not pipeable'"


class TestSyntheticPipeEquivalence:
    """Verify pipe call produces identical output to direct call.
    Requirement 2.4."""

    @pytest.mark.skipif(
        not add.RUST_AVAILABLE,
        reason="Rust bindings not available",
    )
    def test_pipe_equals_direct_call_polars(self):
        """df.pipe(add.synthetic, n=N, seed=S) must equal add.synthetic(df, n=N, seed=S)."""
        pl = pytest.importorskip("polars")
        df = pl.DataFrame({
            "id": [1, 2, 3],
            "value": [10.0, 20.0, 30.0],
        })

        direct = add.synthetic(df, n=5, seed=42)
        piped = df.pipe(add.synthetic, n=5, seed=42)

        assert direct.frame_equal(piped), (
            "Pipe call and direct call produced different results"
        )

    @pytest.mark.skipif(
        not add.RUST_AVAILABLE,
        reason="Rust bindings not available",
    )
    def test_pipe_equals_direct_call_pandas(self):
        """df.pipe(add.synthetic, n=N, seed=S) must equal add.synthetic(df, n=N, seed=S)."""
        pd = pytest.importorskip("pandas")
        pl = pytest.importorskip("polars")
        df = pd.DataFrame({
            "id": [1, 2, 3],
            "value": [10.0, 20.0, 30.0],
        })

        direct = add.synthetic(df, n=5, seed=42)
        piped = df.pipe(add.synthetic, n=5, seed=42)

        pd.testing.assert_frame_equal(direct, piped)
