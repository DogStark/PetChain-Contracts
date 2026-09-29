#!/usr/bin/env python3
"""Diff two ABI snapshots and emit a compatibility summary.

Groups changes by category (method, event, error, type) and
classifies each as additive or breaking.

Usage:
    python3 scripts/abi_diff.py <old_snapshot> <new_snapshot>

Exit code:
    0 — no breaking changes detected
    1 — breaking changes detected (CI should fail)
    2 — usage error
"""
import sys
import re
from collections import defaultdict


def parse_snapshot(path):
    """Parse an ABI snapshot file into categories.

    The file may use section headers (``# Methods``, ``# Events``,
    ``# Errors``, ``# Types``) or be a flat list of method signatures
    (backward-compatible with the original abi-snapshot.txt format).

    Returns a dict with keys ``methods``, ``events``, ``errors``,
    ``types`` — each a dict mapping entry text to its line number
    (for stable ordering).
    """
    sections = {
        "methods": {},
        "events": {},
        "errors": {},
        "types": {},
    }
    # Map section header names to our keys
    section_map = {
        "methods": "methods",
        "events": "events",
        "errors": "errors",
        "types": "types",
    }

    current_section = "methods"  # default for backward compat

    with open(path) as f:
        for lineno, line in enumerate(f, start=1):
            line = line.rstrip("\n")
            stripped = line.strip()

            if not stripped:
                continue

            if stripped.startswith("#"):
                m = re.match(r"#\s*(Methods|Events|Errors|Types)", stripped, re.IGNORECASE)
                if m:
                    current_section = section_map[m.group(1).lower()]
                continue

            # Store entry with its first-seen line number for stable ordering
            if stripped not in sections[current_section]:
                sections[current_section][stripped] = lineno

    return sections


def _signature_name(entry):
    """Extract the name of a method from its snapshot line.

    For ``pub fn foo(...) -> Bar`` returns ``foo``.
    """
    m = re.match(r"pub fn (\w+)\s*\(", entry)
    if m:
        return m.group(1)
    return entry


def _error_name(entry):
    """Extract the variant name from an error snapshot line.

    For ``ContractError::Foo = 42`` returns ``ContractError::Foo``.
    """
    m = re.match(r"(\w+::\w+)(?:\s*=\s*\d+)?", entry)
    if m:
        return m.group(1)
    return entry


def _type_name(entry):
    """Extract the type name from a type snapshot line.

    For ``pub struct Foo { ... }`` returns ``Foo``.
    For plain names like ``Pet`` returns ``Pet``.
    """
    m = re.match(r"pub (?:struct|enum)\s+(\w+)", entry)
    if m:
        return m.group(1)
    # Plain name (no pub struct/enum prefix)
    m = re.match(r"(\w+)", entry)
    if m:
        return m.group(1)
    return entry


def diff_snapshots(old_path, new_path):
    """Compare two ABI snapshots and return a structured diff.

    Returns a dict with ``additive`` and ``breaking`` keys, each
    containing sub-dicts for ``methods``, ``events``, ``errors``,
    and ``types``.
    """
    old = parse_snapshot(old_path)
    new = parse_snapshot(new_path)

    result = {
        "additive": defaultdict(list),
        "breaking": defaultdict(list),
    }

    for category in ("methods", "events", "errors", "types"):
        old_entries = old[category]
        new_entries = new[category]

        if category == "methods":
            # Methods are keyed by full signature; group by name for
            # change detection.
            old_by_name = {}
            for sig, lineno in old_entries.items():
                name = _signature_name(sig)
                old_by_name[name] = (sig, lineno)

            new_by_name = {}
            for sig, lineno in new_entries.items():
                name = _signature_name(sig)
                new_by_name[name] = (sig, lineno)

            added_names = set(new_by_name.keys()) - set(old_by_name.keys())
            removed_names = set(old_by_name.keys()) - set(new_by_name.keys())
            common_names = set(old_by_name.keys()) & set(new_by_name.keys())

            for name in sorted(added_names):
                result["additive"]["methods"].append(new_by_name[name][0])

            for name in sorted(removed_names):
                result["breaking"]["methods"].append(old_by_name[name][0])

            for name in sorted(common_names):
                old_sig = old_by_name[name][0]
                new_sig = new_by_name[name][0]
                if old_sig != new_sig:
                    # Signature changed — breaking
                    result["breaking"]["methods"].append(
                        f"{old_sig}  →  {new_sig}"
                    )

        else:
            # Events, errors, types: compare by full entry text.
            # For errors, normalise by variant name (ignore discriminant).
            if category == "errors":
                old_keys = {_error_name(e): e for e in old_entries}
                new_keys = {_error_name(e): e for e in new_entries}
            else:
                old_keys = {e: e for e in old_entries}
                new_keys = {e: e for e in new_entries}

            added = set(new_keys.keys()) - set(old_keys.keys())
            removed = set(old_keys.keys()) - set(new_keys.keys())

            for key in sorted(added):
                result["additive"][category].append(new_keys[key])

            for key in sorted(removed):
                result["breaking"][category].append(old_keys[key])

    return result


def format_summary(diff):
    """Format a diff dict as a human-readable compatibility summary."""
    lines = []
    lines.append("## ABI Compatibility Summary")
    lines.append("")

    any_additive = any(diff["additive"].values())
    any_breaking = any(diff["breaking"].values())

    if any_additive:
        lines.append("### ✅ Additive Changes")
        lines.append("")
        for cat in ("methods", "events", "errors", "types"):
            items = diff["additive"][cat]
            if items:
                lines.append(f"#### {cat.capitalize()}")
                for item in items:
                    lines.append(f"+ {item}")
                lines.append("")

    if any_breaking:
        lines.append("### 🚫 Breaking Changes")
        lines.append("")
        for cat in ("methods", "events", "errors", "types"):
            items = diff["breaking"][cat]
            if items:
                lines.append(f"#### {cat.capitalize()}")
                for item in items:
                    lines.append(f"- {item}")
                lines.append("")

    if not any_additive and not any_breaking:
        lines.append("No changes detected.")
        lines.append("")

    total_additive = sum(len(v) for v in diff["additive"].values())
    total_breaking = sum(len(v) for v in diff["breaking"].values())
    lines.append(f"**Summary:** {total_additive} additive, {total_breaking} breaking")

    return "\n".join(lines)


def main():
    if len(sys.argv) != 3:
        print(
            "Usage: abi_diff.py <old_snapshot> <new_snapshot>",
            file=sys.stderr,
        )
        sys.exit(2)

    old_path = sys.argv[1]
    new_path = sys.argv[2]

    diff = diff_snapshots(old_path, new_path)
    summary = format_summary(diff)
    print(summary)

    has_breaking = any(diff["breaking"].values())
    sys.exit(1 if has_breaking else 0)


if __name__ == "__main__":
    main()
