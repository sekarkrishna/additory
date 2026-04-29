# Installation

## Quick install

```bash
pip install additory
```

## Requirements

- **Python 3.9+** (tested on 3.9, 3.10, 3.11, 3.12, 3.13)
- **polars** ≥ 0.19.0 — required, used internally for all operations
- **pyarrow** ≥ 10.0.0 — required for DataFrame serialization between Python and Rust

## Optional dependencies

### pandas support

additory works with both polars and pandas DataFrames. If you want to pass pandas DataFrames directly:

```bash
pip install pandas
```

When you pass a pandas DataFrame, additory converts it to polars internally, performs the operation, and converts the result back to pandas. You can override this with the `as_type` parameter on any function.

## Verify the installation

```python
import additory as add
print(add.__version__)
# 0.1.3a11
```

Check that the Rust bindings loaded:

```python
print(add.RUST_AVAILABLE)
# True
```

If `RUST_AVAILABLE` is `False`, only Python-only modes (like `@knn` imputation) will work. See [Troubleshooting](../guides/troubleshooting.md) for help with Rust binding issues.

## Building from source

additory's core is written in Rust with Python bindings via PyO3. To build from source you need:

1. **Rust toolchain** — install via [rustup](https://rustup.rs/)
2. **maturin** — the build tool for PyO3 projects

```bash
# Install maturin
pip install maturin

# Clone the repository
git clone https://github.com/sekarkrishna/additory.git
cd additory

# Build and install in development mode
maturin develop --release
```

For a wheel you can distribute:

```bash
maturin build --release
pip install target/wheels/additory-*.whl
```

!!! tip "Development mode"
    Use `maturin develop` (without `--release`) for faster builds during development. The `--release` flag enables Rust optimizations and is recommended for benchmarking and production use.
