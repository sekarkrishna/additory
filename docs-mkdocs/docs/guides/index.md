# Guides

Practical guides for working with additory's advanced features.

---

## Expression Files

Learn the `.add` file TOML format for defining named expressions. Write custom expression files, load them at runtime with `add.scan('@set', folder)`, and use them via the dynamic API.

→ [Expression Files](expression-files.md)

---

## Reconciliation

Define aliases and groups in reconciliation `.add` files for `add.scan('@diff')`. Normalize variant spellings before comparison and detect changes at different levels of a column hierarchy.

→ [Reconciliation](reconciliation.md)

---

## Troubleshooting

Common errors and solutions, migration steps from older API versions, and Rust binding installation issues.

→ [Troubleshooting](troubleshooting.md)
