# Troubleshooting

Common errors, migration steps from older API versions, and Rust binding issues.

---

## Common Errors

### ImportError: Rust bindings not available

```
ImportError: add.to() requires Rust bindings.
Install with: pip install additory[rust]
```

**Cause:** The compiled Rust extension (`_additory`) is not installed or not compatible with your Python version.

**Fix:**

```bash
pip install additory --force-reinstall
```

If building from source:

```bash
cd additory
pip install maturin
maturin develop --release
```

### ValueError: Cannot use 'as_type' with 'lineage=True'

```
ValueError: Cannot use 'as_type' with 'lineage=True' in add.to().
```

**Cause:** Lineage metadata is stored in the DataFrame's native format and would be lost during type conversion.

**Fix:** Use one or the other:

```python
# Option 1: Track lineage without type conversion
result = add.to(df, ref, 'name', 'id', lineage=True)

# Option 2: Convert type without lineage
result = add.to(df, ref, 'name', 'id', as_type='polars')
```

### TypeError: synthetic() got an unexpected keyword argument 'mode'

```
TypeError: The 'mode' keyword argument has been removed in v0.1.3a11.
```

**Cause:** The `add.synthetic()` signature changed. The first argument is now `df_or_mode`, not a separate `mode` keyword.

**Fix:**

```python
# Old (broken)
result = add.synthetic(mode='@new', n=100, strategy={...})

# New (correct)
result = add.synthetic('@new', n=100, strategy={...})
```

### ValueError: inbuilt: prefix is deprecated

```
ValueError: The 'inbuilt:' prefix is deprecated. Use expression names directly.
```

**Cause:** In `@calc` mode, the `inbuilt:` prefix for built-in expressions has been deprecated in favour of direct name resolution.

**Fix:**

```python
# Old
result = add.transform('@calc', df, expression='inbuilt:bmi')

# New — still works but may show a warning
result = add.transform('@calc', df, expression='inbuilt:bmi')

# Or use the dynamic API directly
result = add.bmi(df)
```

!!! note
    The `inbuilt:` prefix still works in v0.1.3a11 but raises a migration warning. It may be removed in a future version.

---

## Migration from Older Versions

### From v0.1.3a8 and earlier → v0.1.3a11

#### Parameter renames in add.to()

| Old Name | New Name | Notes |
|----------|----------|-------|
| `target_df` | `bring_to` | First positional argument |
| `fetch_from` | `bring_from` | Second positional argument |
| `fetch` | `bring` | Third positional argument |
| `by` | `against` | Fourth positional argument |

```python
# Old
result = add.to(df, fetch_from=ref, fetch=['name'], by='id')

# New
result = add.to(df, ref, 'name', 'id')
# Or with keywords:
result = add.to(bring_to=df, bring_from=ref, bring='name', against='id')
```

#### add.synthetic() signature change

The first argument changed from `mode` to `df_or_mode`:

```python
# Old — mode as keyword
result = add.synthetic(mode='@new', n=100, strategy={...})

# New — mode as positional, or DataFrame for augment
result = add.synthetic('@new', n=100, strategy={...})
result = add.synthetic(df, n=100)  # augment mode inferred
```

#### add.analyze() → add.scan('@analyze')

```python
# Old (still works as alias)
result = add.analyze(df)

# New (recommended)
result = add.scan('@analyze', df)
```

#### Tuples → Lists

All list parameters now expect lists, not tuples:

```python
# Old
result = add.to(df, ref, ('name', 'age'), 'id')

# New
result = add.to(df, ref, ['name', 'age'], 'id')
```

### From v0.1.3a4 → v0.1.3a5

The `by` parameter in `add.to()` was renamed to `against`:

```python
# Old (v0.1.3a4)
result = add.to(df, fetch_from=ref, fetch=['age'], by='id')

# New (v0.1.3a5+)
result = add.to(df, ref, 'age', 'id')
```

---

## Rust Binding Issues

### Building from source

If `pip install additory` fails to find a pre-built wheel for your platform:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin
pip install maturin

# Build from source
cd additory
maturin develop --release
```

### Python version compatibility

Additory supports Python 3.9 through 3.13. If you're on an older version:

```bash
python --version  # Check your version
```

### Platform support

Pre-built wheels are available for:

- Linux (x86_64, aarch64)
- macOS (x86_64, arm64)
- Windows (x86_64)

For other platforms, build from source using the steps above.

### ABI compatibility

If you see ABI-related errors:

```bash
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
maturin build --release
pip install target/wheels/*.whl
```

---

## Getting Help

- **GitHub Issues:** [github.com/sekarkrishna/additory/issues](https://github.com/sekarkrishna/additory/issues)
- **Email:** krishnamoorthy.sankaran@sekrad.org

---

## Next Steps

- [Expression Files](expression-files.md) — the `.add` file format
- [API Reference](../reference/api.md) — complete function signatures
- [Changelog](../reference/changelog.md) — version history
