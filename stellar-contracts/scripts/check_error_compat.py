#!/usr/bin/env python3
"""Error compatibility check for every `#[contracterror]` enum (Issue #1255).

Compares the error enums in the Rust sources against the client mapping
fixture `error-codes.json` and fails when a discriminant is removed, reused,
or renumbered, or when a new error is missing from the fixture.

Usage:
  python3 scripts/check_error_compat.py [--base BASE_FIXTURE]

With `--base` (the fixture from the target branch) it also fails if a
previously published code disappeared from the fixture, or if a newly added
error has no CHANGELOG.md entry. See docs/error-compatibility.md.
"""
import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "error-codes.json"
CHANGELOG = ROOT.parent / "CHANGELOG.md"

ENUM_RE = re.compile(r"#\[contracterror[^\]]*\].*?pub enum (\w+)\s*\{(.*?)\n\}", re.S)
VARIANT_RE = re.compile(r"^\s*(\w+)\s*=\s*(\d+)\s*,", re.M)


def parse_sources():
    """Return {(source, enum): [(name, code), ...]} for every contracterror enum."""
    found = {}
    for path in sorted(ROOT.rglob("*.rs")):
        if "target" in path.parts:
            continue
        text = re.sub(r"//[^\n]*", "", path.read_text())
        for enum, body in ENUM_RE.findall(text):
            source = path.relative_to(ROOT).as_posix()
            found[(source, enum)] = [(n, int(c)) for n, c in VARIANT_RE.findall(body)]
    return found


def check(fixture, base, changelog):
    errors = []
    policies = fixture["retry_policies"]
    sources = parse_sources()
    listed = {(e["source"], e["enum"]): e for e in fixture["enums"]}

    for key in sources.keys() - listed.keys():
        errors.append(f"{key[0]}: enum {key[1]} is not registered in error-codes.json")

    for key, entry in listed.items():
        where = f"{key[0]}::{key[1]}"
        if key not in sources:
            errors.append(f"{where}: enum not found in sources")
            continue
        variants = sources[key]
        by_code = {}
        for name, code in variants:
            if code in by_code:
                errors.append(f"{where}: code {code} reused by {by_code[code]} and {name}")
            by_code[code] = name
        in_source = dict(variants)
        documented = {e["name"]: e for e in entry["errors"]}
        retired = {r["code"]: r["name"] for r in entry.get("retired", [])}

        for name, e in documented.items():
            if e.get("retry") not in policies:
                errors.append(f"{where}: {name} has no valid retry guidance")
            if name not in in_source:
                errors.append(f"{where}: {name} = {e['code']} was removed or renamed "
                              "(move it to `retired` instead of deleting it)")
            elif in_source[name] != e["code"]:
                errors.append(f"{where}: {name} renumbered {e['code']} -> {in_source[name]}")
        for name, code in variants:
            if code in retired:
                errors.append(f"{where}: {name} reuses retired code {code} ({retired[code]})")
            elif name not in documented:
                errors.append(f"{where}: new error {name} = {code} missing from "
                              "error-codes.json (add it with retry guidance)")

    if base is not None:
        current = {(e["source"], e["enum"], x["code"]): x["name"]
                   for e in fixture["enums"] for x in e["errors"] + e.get("retired", [])}
        previous = {(e["source"], e["enum"], x["code"]): x["name"]
                    for e in base["enums"] for x in e["errors"] + e.get("retired", [])}
        for key, name in previous.items():
            if current.get(key) != name:
                errors.append(f"{key[0]}::{key[1]}: published code {key[2]} ({name}) "
                              "was removed or reassigned in error-codes.json")
        for key, name in current.items():
            if key not in previous and not re.search(rf"\b{name}\b", changelog):
                errors.append(f"{key[0]}::{key[1]}: new error {name} = {key[2]} "
                              "needs a CHANGELOG.md entry")
    return errors


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--base", type=Path, help="error-codes.json from the base branch")
    args = parser.parse_args()

    fixture = json.loads(FIXTURE.read_text())
    base = json.loads(args.base.read_text()) if args.base else None
    errors = check(fixture, base, CHANGELOG.read_text())
    for e in errors:
        print(f"ERROR: {e}", file=sys.stderr)
    if errors:
        print("\nSee stellar-contracts/docs/error-compatibility.md", file=sys.stderr)
        return 1
    total = sum(len(e["errors"]) for e in fixture["enums"])
    print(f"error-codes.json is compatible ({len(fixture['enums'])} enums, {total} errors).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
