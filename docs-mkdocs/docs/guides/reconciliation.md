# Reconciliation

Reconciliation `.add` files define aliases and groups for `add.scan('@diff')`. They let you normalize variant spellings before comparison and detect changes at different levels of a column hierarchy.

---

## Why Reconciliation?

Real-world data is messy. The same value appears in different forms:

- `"Active"`, `"ACTIVE"`, `"active"` — same meaning, different case
- `"North America"`, `"NA"`, `"N. America"` — same region, different abbreviations

Without reconciliation, the diff engine flags these as changes. With reconciliation, they're treated as equivalent.

---

## Reconciliation .add File Format

A reconciliation `.add` file uses TOML format with two sections: `[aliases]` and `[groups]`.

```toml title="my_reconciliation.add"
[aliases]
status = ["Active", "ACTIVE", "active", "Inactive", "INACTIVE", "inactive"]
region = ["NA", "North America", "N. America", "EU", "Europe", "EMEA"]

[groups]
location = ["country", "state", "city"]
product = ["category", "subcategory", "sku"]
```

---

## Aliases

Aliases map variant spellings to a canonical form. During diff comparison, all variants are normalized before checking for changes.

### Inline Aliases

Pass aliases directly in the strategy dict:

```python
import additory as add

diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={
        'aliases': {
            'status': ['Active', 'ACTIVE', 'active'],
            'region': ['NA', 'North America'],
        },
    },
)
```

The first value in each list becomes the canonical form. So `"ACTIVE"` and `"active"` are both normalized to `"Active"` before comparison.

### Aliases from a File

Register a reconciliation file, then reference it by name:

```python
# Load reconciliation definitions
add.scan('@set', './reconciliation/')

# Use by name
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'aliases': 'my_reconciliation'},
)
```

### How Aliases Work

1. Before comparing rows, the diff engine applies alias normalization
2. Each column listed in `[aliases]` has its values mapped to the canonical form
3. The comparison runs on the normalized values
4. The output shows the original values (not the normalized ones)

This means a row where `old.status = "ACTIVE"` and `new.status = "active"` is reported as `no_change` rather than `modified`.

---

## Groups

Groups define column hierarchies for structured change detection. They help answer questions like: "Did the country change, or just the city?"

### Group Definition

```toml
[groups]
location = ["country", "state", "city"]
```

This defines a hierarchy: `country` → `state` → `city`. When the diff engine detects a change in any of these columns, it reports which level of the hierarchy changed.

### Using Groups

```python
diff = add.scan('@diff', old=df_old, new=df_new,
    strategy={'groups': 'my_reconciliation'},
)
```

### How Groups Work

1. The diff engine checks each group's columns in hierarchy order
2. If a higher-level column changed (e.g., `country`), lower-level changes are expected
3. If only a lower-level column changed (e.g., `city`), it's flagged as a minor change
4. This gives you structured visibility into what actually changed

---

## Complete Example

### The reconciliation file

```toml title="clinical_reconciliation.add"
[aliases]
diagnosis = [
    "Hypertension", "HTN", "High Blood Pressure",
    "Diabetes", "DM", "Diabetes Mellitus",
    "Asthma", "Bronchial Asthma",
]
status = ["Active", "ACTIVE", "active", "Discharged", "DISCHARGED"]

[groups]
location = ["hospital", "ward", "bed"]
treatment = ["department", "procedure", "medication"]
```

### Using it

```python
import additory as add
import polars as pl

# Load reconciliation
add.scan('@set', './reconciliation/')

old_patients = pl.DataFrame({
    'patient_id': [1, 2, 3],
    'diagnosis': ['HTN', 'DM', 'Asthma'],
    'status': ['ACTIVE', 'active', 'Active'],
})

new_patients = pl.DataFrame({
    'patient_id': [1, 2, 3],
    'diagnosis': ['Hypertension', 'Diabetes', 'Bronchial Asthma'],
    'status': ['Active', 'Active', 'Discharged'],
})

# Without reconciliation: 3 rows show as modified
diff_raw = add.scan('@diff', old=old_patients, new=new_patients)

# With reconciliation: only patient 3 shows as modified (status changed)
diff_reconciled = add.scan('@diff', old=old_patients, new=new_patients,
    strategy={'aliases': 'clinical_reconciliation'},
)
```

---

## Next Steps

- [add.scan('@diff')](../functions/diff.md) — full diff documentation
- [Expression Files](expression-files.md) — the `.add` file format for expressions
- [Troubleshooting](troubleshooting.md) — common issues
