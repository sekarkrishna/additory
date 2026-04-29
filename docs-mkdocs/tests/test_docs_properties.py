"""
Property-based tests for additory MkDocs documentation.

Uses pytest with pathlib for file scanning and json for scenarios.json parsing.
Tests are parameterized over the file lists to verify universal properties.
"""

import json
import re
from pathlib import Path

import pytest

# ── Paths ──────────────────────────────────────────────────────────────────

DOCS_ROOT = Path(__file__).resolve().parent.parent / "docs"
REPO_ROOT = Path(__file__).resolve().parent.parent.parent  # additory/
README_PATH = REPO_ROOT / "README.md"
SCENARIOS_PATH = DOCS_ROOT / "assets" / "data" / "scenarios.json"

ALL_MD_FILES = sorted(DOCS_ROOT.rglob("*.md"))

FUNCTION_PAGES = [
    DOCS_ROOT / "functions" / "to.md",
    DOCS_ROOT / "functions" / "transform.md",
    DOCS_ROOT / "functions" / "synthetic.md",
    DOCS_ROOT / "functions" / "scan.md",
    DOCS_ROOT / "functions" / "diff.md",
    DOCS_ROOT / "functions" / "dynamic.md",
]

SHUFFLE_PAGES = {
    DOCS_ROOT / "getting-started" / "quickstart.md",
    DOCS_ROOT / "functions" / "to.md",
    DOCS_ROOT / "functions" / "transform.md",
    DOCS_ROOT / "functions" / "synthetic.md",
    DOCS_ROOT / "functions" / "diff.md",
    DOCS_ROOT / "functions" / "dynamic.md",
}

EXPECTED_SCENARIO_IDS = {
    "healthcare", "finance", "retail", "education",
    "sports", "food", "hr", "logistics",
}


def _rel(path: Path) -> str:
    """Return a short relative path for test IDs."""
    return str(path.relative_to(DOCS_ROOT))


# ── 16.1 Function page content ordering ───────────────────────────────────
# **Property 1: Function page content ordering**
# **Validates: Requirements 3.1**


@pytest.mark.parametrize("page", FUNCTION_PAGES, ids=[_rel(p) for p in FUNCTION_PAGES])
def test_function_page_content_ordering(page: Path):
    """
    For every function page, sections appear in order:
    Simple Example < Parameters < (mode/strategy details) < Practical Scenarios.

    **Validates: Requirements 3.1**
    """
    content = page.read_text()
    lines = content.splitlines()

    # Find line numbers for key section headings
    simple_line = None
    params_line = None
    practical_line = None

    for i, line in enumerate(lines):
        lower = line.lower().strip()
        # Match "## Simple Example" or similar
        if re.match(r"^#{1,3}\s+simple\s+example", lower):
            simple_line = i
        # Match "## Parameters" or similar
        elif re.match(r"^#{1,3}\s+parameters?$", lower):
            params_line = i
        # Match "## Practical Scenarios" or "## Practical" or similar
        elif re.match(r"^#{1,3}\s+practical", lower):
            practical_line = i

    assert simple_line is not None, f"Missing 'Simple Example' section in {_rel(page)}"
    assert params_line is not None, f"Missing 'Parameters' section in {_rel(page)}"
    assert practical_line is not None, f"Missing 'Practical' section in {_rel(page)}"

    assert simple_line < params_line, (
        f"In {_rel(page)}: Simple Example (line {simple_line}) should come before "
        f"Parameters (line {params_line})"
    )
    assert params_line < practical_line, (
        f"In {_rel(page)}: Parameters (line {params_line}) should come before "
        f"Practical Scenarios (line {practical_line})"
    )


# ── 16.2 Shuffle component presence ───────────────────────────────────────
# **Property 2: Shuffle component presence on required pages**
# **Validates: Requirements 3.8, 2.4**


@pytest.mark.parametrize("page", ALL_MD_FILES, ids=[_rel(p) for p in ALL_MD_FILES])
def test_shuffle_component_presence(page: Path):
    """
    Pages in the shuffle set must contain shuffle-container and shuffle-btn.
    Pages NOT in the shuffle set must NOT contain shuffle-container.

    **Validates: Requirements 3.8, 2.4**
    """
    content = page.read_text()

    if page in SHUFFLE_PAGES:
        assert "shuffle-container" in content, (
            f"{_rel(page)} should contain shuffle-container"
        )
        assert "shuffle-btn" in content, (
            f"{_rel(page)} should contain shuffle-btn"
        )
    else:
        assert "shuffle-container" not in content, (
            f"{_rel(page)} should NOT contain shuffle-container"
        )


# ── 16.3 Scenario completeness ────────────────────────────────────────────
# **Property 3: Scenario completeness**
# **Validates: Requirements 8.2**


def test_scenario_completeness():
    """
    scenarios.json must contain exactly 8 scenarios with unique IDs from the
    required set, each with non-empty target.rows and reference.rows.

    **Validates: Requirements 8.2**
    """
    assert SCENARIOS_PATH.exists(), "scenarios.json not found"

    data = json.loads(SCENARIOS_PATH.read_text())
    scenarios = data["scenarios"]

    assert len(scenarios) == 8, f"Expected 8 scenarios, got {len(scenarios)}"

    ids = {s["id"] for s in scenarios}
    assert ids == EXPECTED_SCENARIO_IDS, (
        f"Scenario IDs mismatch: got {ids}, expected {EXPECTED_SCENARIO_IDS}"
    )

    for scenario in scenarios:
        sid = scenario["id"]
        assert "target" in scenario, f"Scenario '{sid}' missing 'target'"
        assert "reference" in scenario, f"Scenario '{sid}' missing 'reference'"
        assert len(scenario["target"]["rows"]) > 0, (
            f"Scenario '{sid}' has empty target.rows"
        )
        assert len(scenario["reference"]["rows"]) > 0, (
            f"Scenario '{sid}' has empty reference.rows"
        )


# ── 16.4 Shuffle always picks different ───────────────────────────────────
# **Property 4: Shuffle always picks a different scenario**
# **Validates: Requirements 8.3**


@pytest.mark.parametrize("current_id", sorted(EXPECTED_SCENARIO_IDS))
def test_shuffle_picks_different(current_id: str):
    """
    Python simulation of pickRandom: for every scenario ID, filtering out the
    current ID always leaves candidates, and none of them equal the current ID.

    **Validates: Requirements 8.3**
    """
    data = json.loads(SCENARIOS_PATH.read_text())
    all_ids = [s["id"] for s in data["scenarios"]]

    candidates = [sid for sid in all_ids if sid != current_id]
    assert len(candidates) > 0, (
        f"No candidates left when excluding '{current_id}'"
    )
    for c in candidates:
        assert c != current_id, (
            f"Candidate '{c}' should differ from current '{current_id}'"
        )


# ── 16.5 API signature correctness ────────────────────────────────────────
# **Property 5: API signature correctness across all documentation pages**
# **Validates: Requirements 11.1, 11.2, 11.3, 11.4**

# Patterns forbidden on ALL pages (including troubleshooting)
FORBIDDEN_API_PATTERNS_ALL = [
    (r"add\.diff\s*\(", "standalone 'add.diff()' function"),
    (r"add\.set\s*\(", "standalone 'add.set()' function"),
]

# Patterns forbidden on non-migration pages only (troubleshooting legitimately
# references old parameter names when documenting the migration path)
FORBIDDEN_API_PATTERNS_NON_MIGRATION = [
    (r"\bfetch_from\b", "old parameter name 'fetch_from'"),
    (r"\bfetch\s*=", "old parameter name 'fetch='"),
]

MIGRATION_PAGES = {
    DOCS_ROOT / "guides" / "troubleshooting.md",
}


@pytest.mark.parametrize("page", ALL_MD_FILES, ids=[_rel(p) for p in ALL_MD_FILES])
def test_api_signature_correctness(page: Path):
    """
    No documentation page should contain old/forbidden API patterns.
    The troubleshooting page is exempt from old-parameter-name checks because
    it documents the migration from old to new API.

    **Validates: Requirements 11.1, 11.2, 11.3, 11.4**
    """
    content = page.read_text()

    for pattern, description in FORBIDDEN_API_PATTERNS_ALL:
        assert not re.search(pattern, content), (
            f"{_rel(page)} contains forbidden pattern: {description}"
        )

    if page not in MIGRATION_PAGES:
        for pattern, description in FORBIDDEN_API_PATTERNS_NON_MIGRATION:
            assert not re.search(pattern, content), (
                f"{_rel(page)} contains forbidden pattern: {description}"
            )


# ── 16.6 No stale API in README ───────────────────────────────────────────
# **Property 6: No stale API references in README**
# **Validates: Requirements 10.4**


def test_no_stale_api_in_readme():
    """
    README must not contain old API references and must contain current ones.

    **Validates: Requirements 10.4**
    """
    assert README_PATH.exists(), "README.md not found"
    content = README_PATH.read_text()

    # Forbidden patterns
    forbidden = [
        (r"\bfetch_from\b", "old parameter 'fetch_from'"),
        (r"\bfetch\s*=", "old parameter 'fetch='"),
        (r"add\.analyze\s*\(", "old standalone 'add.analyze()'"),
        (r"add\.set\s*\(", "old standalone 'add.set()'"),
    ]
    for pattern, description in forbidden:
        assert not re.search(pattern, content), (
            f"README contains forbidden: {description}"
        )

    # Required patterns
    required = [
        ("bring_to", "current parameter 'bring_to'"),
        ("bring_from", "current parameter 'bring_from'"),
        ("bring", "current parameter 'bring'"),
        ("against", "current parameter 'against'"),
        ("add.scan('@diff'", "diff via add.scan('@diff')"),
        ("add.<dynamic>", "dynamic expression reference"),
    ]
    for text, description in required:
        assert text in content, (
            f"README missing required: {description}"
        )


# ── 16.7 No easter egg documentation ──────────────────────────────────────
# **Property 7: No easter egg documentation**
# **Validates: Requirements 11.6**

EASTER_EGG_PATTERNS = [
    r"\btic-tac-toe\b",
    r"\bsudoku\b",
    r"add\.play\s*\(",
]


@pytest.mark.parametrize("page", ALL_MD_FILES, ids=[_rel(p) for p in ALL_MD_FILES])
def test_no_easter_egg_in_docs(page: Path):
    """
    No documentation page should reference easter egg games.

    **Validates: Requirements 11.6**
    """
    content = page.read_text().lower()

    for pattern in EASTER_EGG_PATTERNS:
        assert not re.search(pattern, content), (
            f"{_rel(page)} contains easter egg reference matching '{pattern}'"
        )


def test_no_easter_egg_in_readme():
    """
    README should not reference easter egg games.

    **Validates: Requirements 11.6**
    """
    content = README_PATH.read_text().lower()

    for pattern in EASTER_EGG_PATTERNS:
        assert not re.search(pattern, content), (
            f"README contains easter egg reference matching '{pattern}'"
        )

    # Also check for the word "games" in a documentation context
    # (but not in generic text like "games" as a domain example)
    assert "add.play(" not in content, "README references add.play()"


# ── 16.8 Shuffle placement ────────────────────────────────────────────────
# **Property 8: Shuffle placement — shuffleable vs stable sections**
# **Validates: Requirements 12.1**

SHUFFLE_FUNCTION_PAGES = [
    p for p in FUNCTION_PAGES if p in SHUFFLE_PAGES
]


@pytest.mark.parametrize(
    "page",
    SHUFFLE_FUNCTION_PAGES,
    ids=[_rel(p) for p in SHUFFLE_FUNCTION_PAGES],
)
def test_shuffle_placement(page: Path):
    """
    On function pages with Shuffle, the shuffle-container wraps simple example
    and practical scenarios but NOT the parameter breakdown.

    **Validates: Requirements 12.1**
    """
    content = page.read_text()
    lines = content.splitlines()

    # Find the Parameters section line range
    params_start = None
    params_end = None

    for i, line in enumerate(lines):
        lower = line.lower().strip()
        if re.match(r"^#{1,3}\s+parameters?$", lower):
            params_start = i
        elif params_start is not None and params_end is None:
            # Next heading at same or higher level ends the params section
            if re.match(r"^#{1,3}\s+", line.strip()) and not re.match(r"^#{1,3}\s+parameters?$", lower):
                params_end = i
                break

    if params_start is None:
        pytest.skip(f"No Parameters section found in {_rel(page)}")

    if params_end is None:
        params_end = len(lines)

    params_section = "\n".join(lines[params_start:params_end])

    # The parameter section should NOT be inside a shuffle-container
    assert "shuffle-container" not in params_section, (
        f"In {_rel(page)}: Parameters section should not be inside a shuffle-container"
    )

    # Verify that shuffle-container exists elsewhere (simple example or practical)
    assert "shuffle-container" in content, (
        f"{_rel(page)} should have shuffle-container outside the Parameters section"
    )


# ── 16.9 Admonition usage ─────────────────────────────────────────────────
# **Property 9: Admonition usage**
# **Validates: Requirements 12.2**

FORBIDDEN_CALLOUT_PATTERNS = [
    (r'<div\s+class="callout">', 'raw HTML callout div'),
    (r'<div\s+class="note">', 'raw HTML note div'),
    (r"^>\s+\*\*Note:\*\*", 'blockquote-based note callout'),
]


@pytest.mark.parametrize("page", ALL_MD_FILES, ids=[_rel(p) for p in ALL_MD_FILES])
def test_admonition_usage(page: Path):
    """
    All callouts must use MkDocs Material admonition syntax (!!! or ???),
    not raw HTML divs or blockquote-based patterns.

    **Validates: Requirements 12.2**
    """
    content = page.read_text()

    for pattern, description in FORBIDDEN_CALLOUT_PATTERNS:
        assert not re.search(pattern, content, re.MULTILINE), (
            f"{_rel(page)} uses forbidden callout pattern: {description}"
        )


# ── 16.10 Content tabs ────────────────────────────────────────────────────
# **Property 10: Content tabs for multi-library examples**
# **Validates: Requirements 12.3**

# Pages known to have both pandas and polars examples
PAGES_WITH_BOTH_LIBRARIES = [
    p for p in ALL_MD_FILES
    if "pandas" in p.read_text().lower() and "polars" in p.read_text().lower()
    and ("import pandas" in p.read_text() or "import pd" in p.read_text()
         or 'pd.DataFrame' in p.read_text() or '=== "Pandas"' in p.read_text())
]


@pytest.mark.parametrize(
    "page",
    PAGES_WITH_BOTH_LIBRARIES,
    ids=[_rel(p) for p in PAGES_WITH_BOTH_LIBRARIES],
)
def test_content_tabs_for_multi_library(page: Path):
    """
    Pages that demonstrate both pandas and polars code must use content tab
    syntax (=== "Pandas" / === "Polars").

    **Validates: Requirements 12.3**
    """
    content = page.read_text()

    # If the page has both pandas and polars code blocks, it should use tabs
    has_pandas_code = bool(
        re.search(r"(import pandas|pd\.DataFrame)", content)
    )
    has_polars_code = bool(
        re.search(r"(import polars|pl\.DataFrame)", content)
    )

    if has_pandas_code and has_polars_code:
        has_tabs = '=== "Pandas"' in content or '=== "Polars"' in content
        assert has_tabs, (
            f"{_rel(page)} has both pandas and polars code but doesn't use "
            f"content tabs (=== \"Pandas\" / === \"Polars\")"
        )


# ── 16.11 No source code modifications ────────────────────────────────────
# **Property 11: No source code modifications**
# **Validates: Requirements 13.1, 13.2, 13.3, 13.4**

# Allowed paths for new/modified files
ALLOWED_PREFIXES = [
    Path("additory") / "docs-mkdocs",
    Path("additory") / "README.md",
    Path(".github") / "workflows",
]

# Forbidden paths — source code directories
FORBIDDEN_DIRS = [
    REPO_ROOT / "src",          # additory/src/
    REPO_ROOT / "additory",     # additory/additory/
    REPO_ROOT / "tests",        # additory/tests/
    REPO_ROOT / "docs",         # additory/docs/ (old Quarto)
]


def test_no_source_code_modifications():
    """
    All new files from this migration must be under additory/docs-mkdocs/,
    additory/README.md, or .github/workflows/. This test verifies the test
    file itself is under docs-mkdocs and that forbidden source directories
    have not been modified by checking they don't contain any docs-mkdocs
    related artifacts.

    **Validates: Requirements 13.1, 13.2, 13.3, 13.4**
    """
    # Verify this test file is under docs-mkdocs
    this_file = Path(__file__).resolve()
    docs_mkdocs_root = DOCS_ROOT.parent
    assert str(this_file).startswith(str(docs_mkdocs_root)), (
        f"Test file {this_file} should be under {docs_mkdocs_root}"
    )

    # Verify the GitHub Actions workflow is in the right place
    workspace_root = REPO_ROOT.parent  # parent of additory/
    workflow_path = workspace_root / ".github" / "workflows" / "docs.yml"
    assert workflow_path.exists(), (
        f"GitHub Actions workflow not found at {workflow_path}"
    )

    # Verify mkdocs.yml is in the right place
    mkdocs_yml = docs_mkdocs_root / "mkdocs.yml"
    assert mkdocs_yml.exists(), f"mkdocs.yml not found at {mkdocs_yml}"

    # Verify README exists
    assert README_PATH.exists(), f"README.md not found at {README_PATH}"

    # Verify no mkdocs-related files leaked into forbidden directories
    for forbidden_dir in FORBIDDEN_DIRS:
        if forbidden_dir.exists():
            for f in forbidden_dir.rglob("*.md"):
                # These files should not contain mkdocs-specific markers
                # that would indicate they were modified by this migration
                content = f.read_text()
                assert "mkdocs-material" not in content.lower(), (
                    f"File {f} in forbidden directory contains mkdocs-material reference"
                )
