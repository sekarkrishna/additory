"""
Expression Loader - Load and manage .add expression files

Handles loading expressions from:
- Inbuilt expressions (core.add, finance.add, medical.add)
- User-defined expressions from custom folder
"""

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Optional

if sys.version_info >= (3, 11):
    import tomllib
else:
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ImportError:
        tomllib = None  # type: ignore[assignment]

# Allowed characters in expression formulas:
# column names (letters, digits, underscore), operators, numbers, whitespace, parentheses, quotes
_EXPRESSION_SAFE_PATTERN = re.compile(r"^[A-Za-z0-9_\s\+\-\*\/\%\(\)\.\,\<\>\=\!\_\^\'\"]+$")

def _validate_expression_content(expression: str, name: str, file_name: str) -> None:
    """Raise ValueError if expression contains disallowed characters."""
    if not _EXPRESSION_SAFE_PATTERN.match(expression):
        raise ValueError(
            f"Expression '{name}' in {file_name} contains invalid characters. "
            f"Expressions may only contain column names, operators (+,-,*,/,%,**), "
            f"numbers, and parentheses. Got: {expression!r}"
        )


# Known function names that should not be treated as column identifiers
_KNOWN_FUNCTIONS = frozenset({
    'if_else', 'today', 'abs', 'min', 'max', 'sum', 'mean',
    'sqrt', 'log', 'exp', 'round', 'ceil', 'floor', 'pow',
})

# Pattern to extract identifiers from a formula string:
# matches word-boundary alpha+alnum sequences, excluding pure numbers
_IDENTIFIER_PATTERN = re.compile(r'\b([A-Za-z_][A-Za-z0-9_]*)\b')


def _extract_identifiers(formula: str) -> List[str]:
    """
    Extract column identifiers from a formula string.

    Finds all identifier tokens (letter/underscore followed by alphanumerics)
    and excludes known function names and Python keywords.

    Args:
        formula: The expression formula string.

    Returns:
        Deduplicated list of identifiers in order of first appearance.
    """
    seen = set()
    result = []
    for match in _IDENTIFIER_PATTERN.finditer(formula):
        ident = match.group(1)
        if ident not in seen and ident not in _KNOWN_FUNCTIONS:
            seen.add(ident)
            result.append(ident)
    return result


@dataclass
class InputDef:
    """Definition of a single expression input."""
    type: str = "numeric"
    unit: str = ""
    description: str = ""


@dataclass
class Expression:
    """Represents a single expression."""
    name: str
    expression: str
    description: str
    category: str = ""
    output_column: str = ""
    inputs: Dict[str, InputDef] = field(default_factory=dict)
    source_file: Optional[str] = None

    def __post_init__(self):
        # Default output_column to name if not set
        if not self.output_column:
            self.output_column = self.name

    def __repr__(self):
        return f"Expression(name='{self.name}', expression='{self.expression}')"


@dataclass
class ReconciliationDef:
    """A reconciliation definition loaded from a .add file."""
    name: str
    description: str
    aliases: Dict[str, List[str]] = field(default_factory=dict)
    groups: Dict[str, List[str]] = field(default_factory=dict)
    source_file: Optional[str] = None


RESERVED_NAMES = frozenset({'to', 'synthetic', 'scan', 'transform', 'harmonize'})


def _scan_folder_for_expressions(folder_path: Path) -> Dict[str, Expression]:
    """Scan a folder for .add files and return all expressions.

    Raises ValueError if any expression uses a reserved name.
    """
    expressions: Dict[str, Expression] = {}
    if not folder_path.exists() or not folder_path.is_dir():
        return expressions

    for add_file in folder_path.glob("*.add"):
        file_expressions = load_add_file(add_file)
        for name, expr in file_expressions.items():
            if name in RESERVED_NAMES:
                raise ValueError(
                    f"Expression name '{name}' in {add_file} is reserved. "
                    f"Reserved names: {', '.join(sorted(RESERVED_NAMES))}"
                )
            if name in expressions:
                raise ValueError(
                    f"Duplicate expression name '{name}' found "
                    f"(defined in multiple .add files in {folder_path})"
                )
            expressions[name] = expr
    return expressions


class ExpressionRegistry:
    """Registry for managing expressions from multiple sources."""
    
    def __init__(self):
        self.inbuilt: Dict[str, Expression] = {}
        self.user_folder: Optional['UserFolder'] = None
        self._load_inbuilt()
    
    def _load_inbuilt(self):
        """Load all inbuilt expressions from .add files."""
        inbuilt_dir = Path(__file__).parent.parent / "inbuilt_expressions"
        
        if not inbuilt_dir.exists():
            return
        
        # Load all .add files
        for add_file in inbuilt_dir.glob("*.add"):
            expressions = load_add_file(add_file)
            
            # Check for duplicates and reserved names
            for name, expr in expressions.items():
                if name in RESERVED_NAMES:
                    raise ValueError(
                        f"Expression name '{name}' in {add_file} is reserved. "
                        f"Reserved names: {', '.join(sorted(RESERVED_NAMES))}"
                    )
                if name in self.inbuilt:
                    raise ValueError(
                        f"Duplicate expression name '{name}' found in builtin files "
                        f"(already defined in another file)"
                    )
                self.inbuilt[name] = expr
    
    def set_user_folder(self, folder_path: str):
        """Set user expressions folder."""
        folder_path_obj = Path(folder_path).resolve()

        if not folder_path_obj.exists():
            raise ValueError(f"Folder does not exist: {folder_path}")

        if not folder_path_obj.is_dir():
            raise ValueError(f"Path is not a directory: {folder_path}")
        
        # Derive namespace from folder name
        namespace = folder_path_obj.name
        
        # Load all .add files from folder (validates reserved names)
        expressions = _scan_folder_for_expressions(folder_path_obj)
        
        # Create user folder (user expressions override inbuilt per Resolution_Order)
        self.user_folder = UserFolder(
            path=folder_path_obj,
            namespace=namespace,
            expressions=expressions
        )

    def resolve_by_name(self, name: str) -> Optional[Expression]:
        """Resolve an expression by name only (not namespace:name).

        Performs a fresh folder scan each time. User folder takes priority
        over inbuilt (user overrides inbuilt).

        Args:
            name: Expression name (e.g. 'bmi')

        Returns:
            Expression if found, None otherwise.

        Raises:
            ValueError: If a reserved name is encountered during scan.
        """
        # Fresh scan of user folder first (user overrides inbuilt)
        if self.user_folder is not None:
            user_expressions = _scan_folder_for_expressions(self.user_folder.path)
            if name in user_expressions:
                return user_expressions[name]

        # Fresh scan of inbuilt folder
        inbuilt_dir = Path(__file__).parent.parent / "inbuilt_expressions"
        inbuilt_expressions = _scan_folder_for_expressions(inbuilt_dir)
        if name in inbuilt_expressions:
            return inbuilt_expressions[name]

        return None

    def list_all_names(self) -> List[str]:
        """Return all available expression names across both folders.

        User folder names come first; inbuilt names that don't conflict are
        appended.
        """
        names: Dict[str, bool] = {}

        # User folder first
        if self.user_folder is not None:
            user_expressions = _scan_folder_for_expressions(self.user_folder.path)
            for n in user_expressions:
                names[n] = True

        # Inbuilt
        inbuilt_dir = Path(__file__).parent.parent / "inbuilt_expressions"
        inbuilt_expressions = _scan_folder_for_expressions(inbuilt_dir)
        for n in inbuilt_expressions:
            if n not in names:
                names[n] = True

        return list(names.keys())
    
    def resolve(self, reference: str) -> Expression:
        """
        Resolve an expression reference.
        
        Args:
            reference: Expression reference (e.g., 'inbuilt:bmi', 'my_folder:custom')
            
        Returns:
            Expression object
            
        Raises:
            ValueError: If reference is invalid or expression not found
        """
        # Parse reference
        parts = reference.split(':', 1)
        if len(parts) != 2:
            raise ValueError(
                f"Invalid expression reference '{reference}'. "
                f"Expected format: 'namespace:name'"
            )
        
        namespace, name = parts
        
        # Check inbuilt namespace
        if namespace == 'inbuilt':
            if name not in self.inbuilt:
                available = ', '.join(sorted(self.inbuilt.keys())[:5])
                raise ValueError(
                    f"Expression '{name}' not found in namespace 'inbuilt'. "
                    f"Available: {available}..."
                )
            return self.inbuilt[name]
        
        # Check user namespace
        if self.user_folder and self.user_folder.namespace == namespace:
            if name not in self.user_folder.expressions:
                available = ', '.join(sorted(self.user_folder.expressions.keys())[:5])
                raise ValueError(
                    f"Expression '{name}' not found in namespace '{namespace}'. "
                    f"Available: {available}..."
                )
            return self.user_folder.expressions[name]
        
        # Unknown namespace
        available_namespaces = ['inbuilt']
        if self.user_folder:
            available_namespaces.append(self.user_folder.namespace)
        
        raise ValueError(
            f"Unknown namespace '{namespace}'. "
            f"Available namespaces: {', '.join(available_namespaces)}"
        )
    
    def list_expressions(self, namespace: Optional[str] = None) -> Dict[str, list]:
        """
        List all expressions, optionally filtered by namespace.
        
        Args:
            namespace: Optional namespace to filter by
            
        Returns:
            Dict with namespace as key and list of expression names as value
        """
        result = {}
        
        if namespace is None or namespace == 'inbuilt':
            result['inbuilt'] = sorted(self.inbuilt.keys())
        
        if self.user_folder and (namespace is None or namespace == self.user_folder.namespace):
            result[self.user_folder.namespace] = sorted(self.user_folder.expressions.keys())
        
        return result


class UserFolder:
    """Represents a user expressions folder."""
    
    def __init__(self, path: Path, namespace: str, expressions: Dict[str, Expression]):
        self.path = path
        self.namespace = namespace
        self.expressions = expressions
    
    def __repr__(self):
        return f"UserFolder(namespace='{self.namespace}', expressions={len(self.expressions)})"



_RECONCILIATION_RE = re.compile(r'^\[reconciliation\]\s*$', re.MULTILINE)


def _is_reconciliation_format(content: str) -> bool:
    """Detect whether .add file content contains a ``[reconciliation]`` section."""
    return bool(_RECONCILIATION_RE.search(content))


def _load_reconciliation_add_file(file_path: Path, content: str) -> ReconciliationDef:
    """Parse a reconciliation-format .add file using TOML.

    Expected sections: ``[reconciliation]``, optional ``[aliases]``, optional ``[groups]``.
    """
    if tomllib is None:
        raise ImportError(
            "Parsing reconciliation .add files requires Python 3.11+ (tomllib) "
            "or the 'tomli' package. Install it with: pip install tomli"
        )

    try:
        data = tomllib.loads(content)
    except Exception as exc:
        raise ValueError(
            f"Failed to parse reconciliation .add file {file_path}: {exc}"
        ) from exc

    recon = data.get("reconciliation", {})

    if "name" not in recon:
        raise ValueError(
            f"Missing required field 'name' in [reconciliation] section of {file_path}.\n"
            f"Every reconciliation .add file must have a name.\n"
            f"Example:\n"
            f"  [reconciliation]\n"
            f'  name = "my_aliases"'
        )

    name = str(recon["name"])
    description = str(recon.get("description", ""))

    # Parse aliases: key = canonical, value = list of variants
    aliases_section = data.get("aliases", {})
    aliases: Dict[str, List[str]] = {}
    for canonical, variants in aliases_section.items():
        if isinstance(variants, list):
            aliases[canonical] = [str(v) for v in variants]
        else:
            aliases[canonical] = [str(variants)]

    # Parse groups: key = parent, value = list of children
    groups_section = data.get("groups", {})
    groups: Dict[str, List[str]] = {}
    for parent, children in groups_section.items():
        if isinstance(children, list):
            groups[parent] = [str(c) for c in children]
        else:
            groups[parent] = [str(children)]

    return ReconciliationDef(
        name=name,
        description=description,
        aliases=aliases,
        groups=groups,
        source_file=str(file_path),
    )


def _load_unified_add_file(file_path: Path, content: str) -> Dict[str, Expression]:
    """Parse a unified-format .add file using TOML.

    Each top-level TOML table (excluding ``reconciliation``, ``aliases``,
    ``groups``) is treated as an expression definition with required
    ``expression``, ``description``, and ``category`` fields and an optional
    ``[name.inputs]`` sub-table.
    """
    if tomllib is None:
        raise ImportError(
            "Parsing .add files requires Python 3.11+ (tomllib) "
            "or the 'tomli' package. Install it with: pip install tomli"
        )

    try:
        data = tomllib.loads(content)
    except Exception as exc:
        raise ValueError(
            f"Failed to parse .add file {file_path}: {exc}"
        ) from exc

    expressions: Dict[str, Expression] = {}
    skip_keys = {"reconciliation", "aliases", "groups"}

    for name, table in data.items():
        if name in skip_keys:
            continue
        if not isinstance(table, dict):
            continue

        # Reject removed fields
        for removed in ("sha", "requires"):
            if removed in table:
                raise ValueError(
                    f"Field '{removed}' in [{name}] of {file_path} is no longer supported. "
                    f"Remove it from the expression definition."
                )

        # Required fields
        for required in ("expression", "description", "category"):
            if required not in table:
                raise ValueError(
                    f"Missing required field '{required}' in [{name}] of {file_path}"
                )

        formula = str(table["expression"])
        description = str(table["description"])
        category = str(table["category"])
        output_column = str(table.get("output_column", name))

        # Validate formula characters
        _validate_expression_content(formula, name, str(file_path))

        # Optional inputs sub-table
        inputs_table = table.get("inputs", {})
        if inputs_table and isinstance(inputs_table, dict):
            inputs: Dict[str, InputDef] = {}
            for inp_name, inp_val in inputs_table.items():
                if isinstance(inp_val, dict):
                    inputs[inp_name] = InputDef(
                        type=str(inp_val.get("type", "numeric")),
                        unit=str(inp_val.get("unit", "")),
                        description=str(inp_val.get("description", "")),
                    )
                else:
                    inputs[inp_name] = InputDef(type="numeric", description=str(inp_val))
        else:
            # Infer inputs from formula identifiers
            identifiers = _extract_identifiers(formula)
            inputs = {ident: InputDef(type="numeric", description="") for ident in identifiers}

        expressions[name] = Expression(
            name=name,
            expression=formula,
            description=description,
            category=category,
            output_column=output_column,
            inputs=inputs,
            source_file=str(file_path),
        )

    return expressions



def load_add_file(file_path: Path) -> Dict[str, Expression]:
    """
    Load expressions from a .add file.

    All expression files use the unified TOML format where each expression
    is a top-level table with ``expression``, ``description``, ``category``
    fields and an optional ``[name.inputs]`` sub-table.

    Reconciliation files (containing a ``[reconciliation]`` section) are
    detected first and return an empty dict (no expressions).

    Args:
        file_path: Path to .add file

    Returns:
        Dict mapping expression name to Expression object
    """
    with open(file_path, 'r') as f:
        content = f.read()

    # Reconciliation-only files have no expressions
    if _is_reconciliation_format(content):
        return {}

    return _load_unified_add_file(file_path, content)


def format_expression_toml(expr: Expression) -> str:
    """Format an Expression into unified TOML format.

    The output uses a ``[name]`` top-level table with ``expression``,
    ``description``, ``category`` fields and an optional ``[name.inputs]``
    sub-table.  It can be parsed back by :func:`load_add_file`.

    Args:
        expr: The Expression object to format.

    Returns:
        A TOML string representing the ``.add`` file content.
    """
    lines: List[str] = []

    # [name] table
    lines.append(f'[{expr.name}]')
    lines.append(f'expression = "{expr.expression}"')
    lines.append(f'description = "{expr.description}"')
    lines.append(f'category = "{expr.category}"')
    if expr.output_column != expr.name:
        lines.append(f'output_column = "{expr.output_column}"')
    lines.append('')

    # [name.inputs] sub-table
    if expr.inputs:
        lines.append(f'[{expr.name}.inputs]')
        for inp_name in sorted(expr.inputs.keys()):
            inp_def = expr.inputs[inp_name]
            parts = [f'type = "{inp_def.type}"']
            if inp_def.unit:
                parts.append(f'unit = "{inp_def.unit}"')
            if inp_def.description:
                parts.append(f'description = "{inp_def.description}"')
            lines.append(f'{inp_name} = {{ {", ".join(parts)} }}')
        lines.append('')

    return '\n'.join(lines)


# Global registry instance
_registry: Optional[ExpressionRegistry] = None


def get_registry() -> ExpressionRegistry:
    """Get the global expression registry (singleton)."""
    global _registry
    if _registry is None:
        _registry = ExpressionRegistry()
    return _registry


def set_user_folder(folder_path: str):
    """Set user expressions folder."""
    registry = get_registry()
    registry.set_user_folder(folder_path)


def resolve_expression(reference: str) -> Expression:
    """Resolve an expression reference."""
    registry = get_registry()
    return registry.resolve(reference)


def list_expressions(namespace: Optional[str] = None) -> Dict[str, list]:
    """List all expressions."""
    registry = get_registry()
    return registry.list_expressions(namespace)


# ═══════════════════════════════════════════════════════════════════════
# Reconciliation helpers
# ═══════════════════════════════════════════════════════════════════════

def load_reconciliation_from_file(file_path: Path) -> Optional[ReconciliationDef]:
    """Load a ReconciliationDef from a .add file, or return None if not reconciliation."""
    with open(file_path, "r") as f:
        content = f.read()
    if not _is_reconciliation_format(content):
        return None
    return _load_reconciliation_add_file(file_path, content)


def resolve_reconciliation_by_name(name: str) -> Optional[ReconciliationDef]:
    """Resolve a reconciliation definition by name from .add files.

    Scans the user folder (if set) and the inbuilt expressions folder for
    ``.add`` files containing a ``[reconciliation]`` section whose ``name``
    field matches *name*.

    Returns ``None`` if no match is found.
    """
    registry = get_registry()

    # Scan user folder first
    if registry.user_folder is not None:
        for add_file in registry.user_folder.path.glob("*.add"):
            recon = load_reconciliation_from_file(add_file)
            if recon is not None and recon.name == name:
                return recon

    # Scan inbuilt folder
    inbuilt_dir = Path(__file__).parent.parent / "inbuilt_expressions"
    if inbuilt_dir.exists():
        for add_file in inbuilt_dir.glob("*.add"):
            recon = load_reconciliation_from_file(add_file)
            if recon is not None and recon.name == name:
                return recon

    return None


def format_reconciliation_add_file(
    name: str,
    description: str,
    aliases: Optional[Dict[str, List[str]]] = None,
    groups: Optional[Dict[str, List[str]]] = None,
) -> str:
    """Format a reconciliation definition into a valid .add file TOML string.

    The output can be parsed back by :func:`load_reconciliation_from_file`.

    Args:
        name: Reconciliation name.
        description: Human-readable description.
        aliases: Mapping of canonical name to list of variant strings.
        groups: Mapping of parent name to list of child member strings.

    Returns:
        A TOML string representing the ``.add`` file content.
    """
    lines: List[str] = []

    lines.append("[reconciliation]")
    lines.append(f'name = "{name}"')
    lines.append(f'description = "{description}"')
    lines.append("")

    if aliases:
        lines.append("[aliases]")
        for canonical, variants in aliases.items():
            variant_strs = ", ".join(f'"{v}"' for v in variants)
            lines.append(f'{canonical} = [{variant_strs}]')
        lines.append("")

    if groups:
        lines.append("[groups]")
        for parent, children in groups.items():
            child_strs = ", ".join(f'"{c}"' for c in children)
            lines.append(f'{parent} = [{child_strs}]')
        lines.append("")

    return "\n".join(lines)
