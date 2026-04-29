# Contributing to Additory

Thank you for your interest in contributing to Additory! This document provides guidelines for contributing to the project.

## Code of Conduct

Be respectful and constructive in all interactions.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/sekarkrishna/additory/issues)
2. If not, create a new issue with:
   - Clear title and description
   - Steps to reproduce
   - Expected vs actual behavior
   - Python version, OS, and additory version
   - Code sample if applicable

### Suggesting Features

1. Check existing issues for similar suggestions
2. Create a new issue with:
   - Clear description of the feature
   - Use cases and benefits
   - Possible implementation approach

### Pull Requests

1. Fork the repository
2. Create a new branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Add tests if applicable
5. Update documentation
6. Commit with clear messages
7. Push to your fork
8. Create a Pull Request

## Development Setup

### Prerequisites

- Python 3.9+
- Rust (latest stable)
- Maturin

### Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/additory.git
cd additory

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin
pip install maturin

# Build in development mode
export PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1
maturin develop

# Install development dependencies
pip install pytest pandas polars pyarrow
```

### Running Tests

```bash
# Run Python tests
pytest tests/

# Run Rust tests
cargo test
```

### Building Documentation

```bash
cd docs
quarto preview  # Preview locally
quarto render   # Build HTML
```

## Code Style

### Python
- Follow PEP 8
- Use type hints where applicable
- Add docstrings for public functions

### Rust
- Follow Rust standard style
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for issues

## Commit Messages

- Use clear, descriptive commit messages
- Start with a verb (Add, Fix, Update, Remove, etc.)
- Reference issues when applicable: `Fix #123: Description`

## Documentation

- Update documentation for any user-facing changes
- Add examples for new features
- Update CHANGELOG.md

## Questions?

Feel free to open an issue for questions or reach out to:
- Email: krishnamoorthy.sankaran@sekrad.org
- GitHub Issues: [https://github.com/sekarkrishna/additory/issues](https://github.com/sekarkrishna/additory/issues)

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
