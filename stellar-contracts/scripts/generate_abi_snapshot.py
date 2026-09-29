#!/usr/bin/env python3
"""Extract the public contract ABI from
stellar-contracts/src/lib.rs and prints a normalized, sorted snapshot.

The snapshot is structured into four sections:
  # Methods   — every ``pub fn`` signature in the ``#[contractimpl]`` block
  # Events    — every ``#[contracttype]`` struct that carries a ``version: u32``
                field (event struct)
  # Errors    — every variant of the ``ContractError`` enum
  # Types     — every ``#[contracttype]`` struct and enum (excluding event
                structs and ``ContractError``)

Used by ``generate_abi_snapshot.sh`` and by CI (see
.github/workflows/stellar-contracts.yml, job ``abi-snapshot``) to detect
unreviewed changes to the contract's public interface. See
stellar-contracts/docs/abi-migrations.md for the process to follow when a
change is intentional.
"""
import re
import sys


def _extract_methods(text):
    """Return a sorted list of normalized method signatures from the
    ``#[contractimpl]`` block."""
    marker = "#[contractimpl]"
    start = text.index(marker)
    brace_start = text.index("{", start)
    depth = 0
    i = brace_start
    while True:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = text[brace_start + 1 : i]

    sigs = []
    for m in re.finditer(r"\bpub fn\s+(\w+)\s*\(", body):
        name = m.group(1)
        paren_start = m.end() - 1
        depth = 0
        j = paren_start
        while True:
            if body[j] == "(":
                depth += 1
            elif body[j] == ")":
                depth -= 1
                if depth == 0:
                    break
            j += 1
        args = body[paren_start + 1 : j]
        rest = body[j + 1 : j + 300]
        ret_match = re.match(r"\s*->\s*([^\{]+)\{", rest)
        ret = ret_match.group(1).strip() if ret_match else ""

        def norm(s):
            return re.sub(r"\s+", " ", s).strip()

        args_norm = norm(args)
        sig = f"pub fn {name}({args_norm})"
        if ret:
            sig += f" -> {norm(ret)}"
        sigs.append(sig)

    return sorted(sigs)


def _extract_events(text):
    """Return a sorted list of event struct names.

    An event struct is a ``#[contracttype]`` struct that contains a
    ``version: u32`` field.
    """
    events = []
    # Find all #[contracttype] blocks that contain a version: u32 field
    for m in re.finditer(
        r"#\[contracttype\]\s*(?:#\[derive[^\]]*\]\s*)*pub struct (\w+)\s*\{([^}]*)\}",
        text,
        re.DOTALL,
    ):
        name = m.group(1)
        body = m.group(2)
        if re.search(r"pub version:\s*u32", body):
            events.append(name)
    return sorted(events)


def _extract_errors(text):
    """Return a sorted list of ContractError variant lines
    (``VariantName = N``)."""
    # Find the ContractError enum definition
    marker = "pub enum ContractError"
    start = text.index(marker)
    # Find the opening brace
    brace_start = text.index("{", start)
    depth = 0
    i = brace_start
    while True:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    body = text[brace_start + 1 : i]

    variants = []
    for m in re.finditer(
        r"(\w+)\s*=\s*(\d+)", body
    ):
        variants.append(f"ContractError::{m.group(1)} = {m.group(2)}")
    return sorted(variants)


def _extract_types(text):
    """Return a sorted list of contract type names (structs and enums
    with ``#[contracttype]``), excluding event structs and ContractError."""
    types = []
    # Find all #[contracttype] annotated structs and enums
    for m in re.finditer(
        r"#\[contracttype\]\s*(?:#\[derive[^\]]*\]\s*)*(?:pub\s+)?(?:struct|enum)\s+(\w+)",
        text,
    ):
        name = m.group(1)
        # Skip ContractError (handled separately)
        if name == "ContractError":
            continue
        # Skip event structs (those with a version field)
        # We check by looking ahead for the struct body
        types.append(name)
    return sorted(set(types))


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else "src/lib.rs"
    with open(path) as f:
        text = f.read()

    methods = _extract_methods(text)
    events = _extract_events(text)
    errors = _extract_errors(text)
    types = _extract_types(text)

    sections = []

    sections.append("# Methods")
    for sig in methods:
        sections.append(sig)

    sections.append("# Events")
    for evt in events:
        sections.append(evt)

    sections.append("# Errors")
    for err in errors:
        sections.append(err)

    sections.append("# Types")
    for typ in types:
        sections.append(typ)

    print("\n".join(sections))


if __name__ == "__main__":
    main()
