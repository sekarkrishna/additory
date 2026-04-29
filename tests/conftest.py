"""Shared Hypothesis strategies for additory property-based tests.

Provides reusable generators for:
- DataFrames (pandas/polars with numeric columns)
- Expression definitions (name, formula, inputs, output_column, category)
- .add file content (unified TOML format)
- Column mappings (random subsets of input names)
- Position values ('start', 'end', 'after:col', 'before:col', int)
"""

from hypothesis import strategies as st

# ---------------------------------------------------------------------------
# Primitives
# ---------------------------------------------------------------------------

# Valid Python/column identifiers — lowercase, short, no collisions with operators
_column_names = st.from_regex(r"[a-z][a-z0-9_]{0,9}", fullmatch=True).filter(
    lambda s: s not in {
        "if_else", "today", "abs", "min", "max", "sum", "mean",
        "sqrt", "log", "exp", "round", "ceil", "floor", "pow",
        "to", "synthetic", "scan", "transform", "harmonize",
    }
)

_numeric_values = st.floats(
    min_value=-1e6, max_value=1e6, allow_nan=False, allow_infinity=False
)

_input_types = st.sampled_from(["numeric", "categorical"])


# ---------------------------------------------------------------------------
# DataFrame generators
# ---------------------------------------------------------------------------

@st.composite
def numeric_dataframes(draw, min_cols=1, max_cols=5, min_rows=1, max_rows=20):
    """Generate a dict of {col_name: [float values]} suitable for DataFrame construction.

    Returns a tuple of (column_dict, column_names) so callers can build
    pandas or polars DataFrames as needed.
    """
    n_cols = draw(st.integers(min_value=min_cols, max_value=max_cols))
    col_names = draw(
        st.lists(_column_names, min_size=n_cols, max_size=n_cols, unique=True)
    )
    n_rows = draw(st.integers(min_value=min_rows, max_value=max_rows))
    data = {}
    for col in col_names:
        data[col] = draw(st.lists(_numeric_values, min_size=n_rows, max_size=n_rows))
    return data, col_names


@st.composite
def pandas_dataframes(draw, min_cols=1, max_cols=5, min_rows=1, max_rows=20):
    """Generate a random pandas DataFrame with numeric columns."""
    import pandas as pd

    data, _ = draw(numeric_dataframes(min_cols=min_cols, max_cols=max_cols,
                                       min_rows=min_rows, max_rows=max_rows))
    return pd.DataFrame(data)


@st.composite
def polars_dataframes(draw, min_cols=1, max_cols=5, min_rows=1, max_rows=20):
    """Generate a random polars DataFrame with numeric columns."""
    import polars as pl

    data, _ = draw(numeric_dataframes(min_cols=min_cols, max_cols=max_cols,
                                       min_rows=min_rows, max_rows=max_rows))
    return pl.DataFrame(data)


# ---------------------------------------------------------------------------
# Expression definition generators
# ---------------------------------------------------------------------------

# Binary operators safe for formula generation
_operators = st.sampled_from([" + ", " - ", " * "])


@st.composite
def expression_inputs(draw, min_inputs=1, max_inputs=4):
    """Generate a dict of input definitions: {name: {type, description}}."""
    n = draw(st.integers(min_value=min_inputs, max_value=max_inputs))
    names = draw(st.lists(_column_names, min_size=n, max_size=n, unique=True))
    inputs = {}
    for name in names:
        inputs[name] = {
            "type": draw(_input_types),
            "description": draw(st.text(min_size=0, max_size=30,
                                        alphabet=st.characters(whitelist_categories=("L", "Nd", "Zs")))),
        }
    return inputs


@st.composite
def expression_definitions(draw):
    """Generate a complete expression definition tuple.

    Returns (name, description, output_column, inputs_dict, formula_str).
    """
    inputs = draw(expression_inputs(min_inputs=1, max_inputs=4))
    input_names = list(inputs.keys())

    # Build a simple formula from the input names
    formula_parts = [input_names[0]]
    for inp in input_names[1:]:
        op = draw(_operators)
        formula_parts.append(op)
        formula_parts.append(inp)
    formula = "".join(formula_parts)

    name = draw(_column_names)
    description = draw(st.text(min_size=1, max_size=50,
                               alphabet=st.characters(whitelist_categories=("L", "Nd", "Zs"))))
    output_column = draw(st.one_of(st.just(name), _column_names))

    return name, description, output_column, inputs, formula


# ---------------------------------------------------------------------------
# .add file content generators
# ---------------------------------------------------------------------------

@st.composite
def unified_add_file_content(draw):
    """Generate valid unified-format .add file TOML content.

    Format: [name] / expression = "..." / description = "..." / category = "..."
    with optional [name.inputs] sub-table.
    """
    name, description, output_column, inputs, formula = draw(expression_definitions())
    category = draw(st.sampled_from(["core", "finance", "medical", "custom"]))

    lines = [
        f'[{name}]',
        f'expression = "{formula}"',
        f'description = "{description}"',
        f'category = "{category}"',
    ]
    if output_column != name:
        lines.append(f'output_column = "{output_column}"')
    lines.append("")

    # Optionally include explicit inputs sub-table
    if draw(st.booleans()):
        lines.append(f'[{name}.inputs]')
        for inp_name, inp_def in inputs.items():
            parts = [f'type = "{inp_def["type"]}"']
            if inp_def.get("description"):
                parts.append(f'description = "{inp_def["description"]}"')
            lines.append(f'{inp_name} = {{ {", ".join(parts)} }}')
        lines.append("")

    content = "\n".join(lines)
    return content, name, description, category, output_column, inputs, formula


# ---------------------------------------------------------------------------
# Column mapping generator
# ---------------------------------------------------------------------------

@st.composite
def column_mappings(draw, input_names):
    """Generate a random subset mapping of input names to alternative column names.

    Args:
        input_names: List of expression input names to potentially remap.

    Returns a dict mapping a subset of input_names to new column names.
    """
    if not input_names:
        return {}
    subset = draw(
        st.lists(st.sampled_from(input_names), unique=True,
                 min_size=0, max_size=len(input_names))
    )
    mapping = {}
    for inp in subset:
        mapping[inp] = draw(_column_names.filter(lambda n, _inp=inp: n != _inp))
    return mapping


# ---------------------------------------------------------------------------
# Position generator
# ---------------------------------------------------------------------------

@st.composite
def position_values(draw, column_names=None):
    """Generate a valid position value for column placement.

    Produces one of: 'start', 'end', 'after:<col>', 'before:<col>', or int index.
    If column_names is provided, 'after:' and 'before:' use those names.
    """
    choices = [
        st.just("start"),
        st.just("end"),
        st.integers(min_value=0, max_value=20),
    ]
    if column_names:
        col_st = st.sampled_from(column_names)
        choices.append(col_st.map(lambda c: f"after:{c}"))
        choices.append(col_st.map(lambda c: f"before:{c}"))
    return draw(st.one_of(*choices))


# ---------------------------------------------------------------------------
# Diff engine generators  (Task 13.1)
# ---------------------------------------------------------------------------

@st.composite
def diff_key_columns(draw, min_keys=1, max_keys=2):
    """Generate a list of key column names (single or composite)."""
    n = draw(st.integers(min_value=min_keys, max_value=max_keys))
    return draw(st.lists(_column_names, min_size=n, max_size=n, unique=True))


@st.composite
def dataframe_pairs_with_key(
    draw,
    min_rows=1,
    max_rows=10,
    min_extra_cols=1,
    max_extra_cols=3,
    as_pandas=False,
):
    """Generate a pair of DataFrames sharing a key column with controlled overlap.

    Returns (old_df, new_df, key_col_name) where:
    - Some keys appear in both (for changed/no_change rows)
    - Some keys appear only in old (for deleted rows)
    - Some keys appear only in new (for new rows)
    """
    import polars as pl

    key_col = draw(_column_names)
    n_extra = draw(st.integers(min_value=min_extra_cols, max_value=max_extra_cols))
    extra_cols = draw(
        st.lists(
            _column_names.filter(lambda n, _k=key_col: n != _k),
            min_size=n_extra,
            max_size=n_extra,
            unique=True,
        )
    )

    n_shared = draw(st.integers(min_value=1, max_value=max_rows))
    n_old_only = draw(st.integers(min_value=0, max_value=max(1, max_rows // 2)))
    n_new_only = draw(st.integers(min_value=0, max_value=max(1, max_rows // 2)))

    total_old = n_shared + n_old_only
    total_new = n_shared + n_new_only

    # Generate unique key values
    all_keys = draw(
        st.lists(
            st.integers(min_value=1, max_value=10000),
            min_size=n_shared + n_old_only + n_new_only,
            max_size=n_shared + n_old_only + n_new_only,
            unique=True,
        )
    )

    shared_keys = all_keys[:n_shared]
    old_only_keys = all_keys[n_shared : n_shared + n_old_only]
    new_only_keys = all_keys[n_shared + n_old_only :]

    old_keys = shared_keys + old_only_keys
    new_keys = shared_keys + new_only_keys

    # Build data dicts
    old_data = {key_col: old_keys}
    new_data = {key_col: new_keys}

    for col in extra_cols:
        old_data[col] = draw(
            st.lists(
                st.text(min_size=1, max_size=5, alphabet="abcdefgh"),
                min_size=total_old,
                max_size=total_old,
            )
        )
        new_data[col] = draw(
            st.lists(
                st.text(min_size=1, max_size=5, alphabet="abcdefgh"),
                min_size=total_new,
                max_size=total_new,
            )
        )

    old_df = pl.DataFrame(old_data)
    new_df = pl.DataFrame(new_data)

    if as_pandas:
        import pandas as pd
        old_data_pd = {k: v for k, v in old_data.items()}
        new_data_pd = {k: v for k, v in new_data.items()}
        return pd.DataFrame(old_data_pd), pd.DataFrame(new_data_pd), key_col

    return old_df, new_df, key_col


@st.composite
def strategy_dicts(draw):
    """Generate a valid strategy dict with random subsets of valid keys."""
    strategy = {}

    if draw(st.booleans()):
        strategy["output"] = draw(st.sampled_from(["summary", "detail"]))

    if draw(st.booleans()):
        strategy["exclude"] = draw(
            st.lists(_column_names, min_size=0, max_size=3, unique=True)
        )

    if draw(st.booleans()):
        # Ensure no overlap with exclude
        excluded = set(strategy.get("exclude", []))
        strategy["carry"] = draw(
            st.lists(
                _column_names.filter(lambda n: n not in excluded),
                min_size=0,
                max_size=3,
                unique=True,
            )
        )

    if draw(st.booleans()):
        strategy["context"] = draw(
            st.lists(_column_names, min_size=0, max_size=3, unique=True)
        )

    return strategy


@st.composite
def alias_dicts(draw, min_groups=1, max_groups=3, min_variants=1, max_variants=3):
    """Generate an alias dict: canonical_name -> [variant1, variant2, ...]."""
    n = draw(st.integers(min_value=min_groups, max_value=max_groups))
    canonicals = draw(
        st.lists(_column_names, min_size=n, max_size=n, unique=True)
    )
    aliases = {}
    for canonical in canonicals:
        n_variants = draw(st.integers(min_value=min_variants, max_value=max_variants))
        variants = draw(
            st.lists(
                _column_names.filter(lambda n, _c=canonical: n != _c),
                min_size=n_variants,
                max_size=n_variants,
                unique=True,
            )
        )
        aliases[canonical] = variants
    return aliases


@st.composite
def reconciliation_defs(draw):
    """Generate a valid ReconciliationDef."""
    from additory.expressions.loader import ReconciliationDef

    name = draw(_column_names)
    description = draw(
        st.text(
            min_size=1,
            max_size=40,
            alphabet=st.characters(whitelist_categories=("L", "Nd", "Zs")),
        )
    )
    aliases = draw(alias_dicts(min_groups=0, max_groups=3))
    groups = draw(alias_dicts(min_groups=0, max_groups=2))

    return ReconciliationDef(
        name=name,
        description=description,
        aliases=aliases,
        groups=groups,
    )


@st.composite
def reconciliation_toml_strings(draw):
    """Generate a valid .add file TOML string with [reconciliation] section."""
    recon_def = draw(reconciliation_defs())
    from additory.expressions.loader import format_reconciliation_add_file

    content = format_reconciliation_add_file(
        name=recon_def.name,
        description=recon_def.description,
        aliases=recon_def.aliases if recon_def.aliases else None,
        groups=recon_def.groups if recon_def.groups else None,
    )
    return content, recon_def
