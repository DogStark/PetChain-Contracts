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


def _event_name(entry):
    """Extract the event name from an event snapshot line.

    Supports forms like ``Event::Foo { ... }``, ``Foo { ... }``,
    ``Foo(...)``, and plain ``Foo``. Returns the bare event name.
    """
    m = re.match(r"(?:\w+::)?(\w+)", entry)
    if m:
        return m.group(1)
    return entry


def _event_fields(entry):
    """Parse the ordered field list of an event snapshot line.

    Fields may be written as ``name: Type`` (struct-style, inside
    ``{ ... }``) or as bare ``Type`` (tuple-style, inside ``( ... )``).
    Returns a list of ``(name, type)`` tuples in declaration order.
    For tuple-style fields the name is ``None``.
    """
    m = re.search(r"[{(](.*)[})]", entry)
    if not m:
        return []
    body = m.group(1).strip()
    if not body:
        return []

    fields = []
    for raw in body.split(","):
        field = raw.strip()
        if not field:
            continue
        if ":" in field:
            name, _, ftype = field.partition(":")
            fields.append((name.strip(), ftype.strip()))
        else:
            fields.append((None, field))
    return fields


def _diff_event(old_entry, new_entry):
    """Classify the change between two versions of the same event.

    Returns a tuple ``(additive, breaking)`` where each is a list of
    human-readable change descriptions.

    Rules:
      * Appended field(s) → additive (index-based consumers must update).
      * Removed field, renamed field, changed field type, or reordered
        field → breaking.
    """
    additive = []
    breaking = []

    old_fields = _event_fields(old_entry)
    new_fields = _event_fields(new_entry)

    # Name-based comparison (struct-style fields).
    old_named = {name: ftype for name, ftype in old_fields if name is not None}
    new_named = {name: ftype for name, ftype in new_fields if name is not None}

    if old_named or new_named:
        old_order = [name for name, _ in old_fields if name is not None]
        new_order = [name for name, _ in new_fields if name is not None]

        for name in old_order:
            if name not in new_named:
                breaking.append(
                    f"{old_entry}  →  {new_entry}  (field '{name}' removed)"
                )
            elif old_named[name] != new_named[name]:
                breaking.append(
                    f"{old_entry}  →  {new_entry}  "
                    f"(field '{name}' type changed: "
                    f"{old_named[name]} → {new_named[name]})"
                )

        for name in new_order:
            if name not in old_named:
                additive.append(
                    f"{old_entry}  →  {new_entry}  "
                    f"(field '{name}' appended; index-based consumers must update)"
                )

        # Reorder detection: compare relative order of common fields.
        common_old = [n for n in old_order if n in new_named]
        common_new = [n for n in new_order if n in old_named]
        if common_old != common_new:
            breaking.append(
                f"{old_entry}  →  {new_entry}  (fields reordered)"
            )
        return additive, breaking

    # Tuple-style (positional) fields: compare by position and type.
    for idx, (_, old_type) in enumerate(old_fields):
        if idx >= len(new_fields):
            breaking.append(
                f"{old_entry}  →  {new_entry}  (field #{idx} removed)"
            )
        elif new_fields[idx][1] != old_type:
            breaking.append(
                f"{old_entry}  →  {new_entry}  "
                f"(field #{idx} type changed: {old_type} → {new_fields[idx][1]})"
            )

    for idx in range(len(old_fields), len(new_fields)):
        additive.append(
            f"{old_entry}  →  {new_entry}  "
            f"(field #{idx} appended; index-based consumers must update)"
        )

    return additive, breaking


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

        elif category == "events":
            # Events are keyed by event name; compare fields by name
            # and position to distinguish additive appends from
            # breaking removals/renames/type changes/reorders.
            old_by_name = {}
            for entry in old_entries:
                old_by_name[_event_name(entry)] = entry

            new_by_name = {}
            for entry in new_entries:
                new_by_name[_event_name(entry)] = entry

            added_names = set(new_by_name.keys()) - set(old_by_name.keys())
            removed_names = set(old_by_name.keys()) - set(new_by_name.keys())
            common_names = set(old_by_name.keys()) & set(new_by_name.keys())

            for name in sorted(added_names):
                result["additive"]["events"].append(new_by_name[name])

            for name in sorted(removed_names):
                result["breaking"]["events"].append(old_by_name[name])

            for name in sorted(common_names):
                old_entry = old_by_name[name]
                new_entry = new_by_name[name]
                if old_entry == new_entry:
                    continue
                additive, breaking = _diff_event(old_entry, new_entry)
                result["additive"]["events"].extend(additive)
                result["breaking"]["events"].extend(breaking)

        else:
            # Errors, types: compare by full entry text.
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
