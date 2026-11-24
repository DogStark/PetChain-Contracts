#!/usr/bin/env python3
"""Render the error code reference in docs/error-codes.md from error-codes.json (Issue #1593).

error-codes.json is the authoritative registry (checked against the Rust sources
by check_error_compat.py). The section of docs/error-codes.md from the BEGIN marker
up to "## Usage Examples" is generated from it and must not be edited by hand.

Usage:
  python3 stellar-contracts/scripts/generate_error_docs.py          # rewrite the doc
  python3 stellar-contracts/scripts/generate_error_docs.py --check  # fail if stale
"""
import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURE = ROOT / "error-codes.json"
DOC = ROOT.parent / "docs" / "error-codes.md"
BEGIN = "<!-- BEGIN GENERATED ERROR CODES: stellar-contracts/scripts/generate_error_docs.py -->"
START_HEADING = "## Error Code Reference"
END_HEADING = "## Usage Examples"


def render(fixture):
    lines = [
        BEGIN,
        START_HEADING,
        "",
        "Generated from [`stellar-contracts/error-codes.json`](../stellar-contracts/error-codes.json). "
        "Do not edit this section by hand: change the registry, then run "
        "`python3 stellar-contracts/scripts/generate_error_docs.py`.",
        "",
        "### Retry Policies",
        "",
        "| Policy | Meaning |",
        "|--------|---------|",
    ]
    for policy, meaning in fixture["retry_policies"].items():
        lines.append(f"| `{policy}` | {meaning} |")

    for enum in fixture["enums"]:
        lines += [
            "",
            f"### `{enum['enum']}` ({enum['contract']})",
            "",
            f"Source: `stellar-contracts/{enum['source']}`",
            "",
            "| Code | Name | Retry |",
            "|------|------|-------|",
        ]
        for error in enum["errors"]:
            lines.append(f"| {error['code']} | `{error['name']}` | `{error['retry']}` |")
        if enum.get("retired"):
            lines += [
                "",
                "Retired codes (never reuse):",
                "",
                "| Code | Name |",
                "|------|------|",
            ]
            for error in enum["retired"]:
                lines.append(f"| {error['code']} | `{error['name']}` |")

    lines += ["", "---", "", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true", help="fail if the doc is out of date")
    args = parser.parse_args()

    fixture = json.loads(FIXTURE.read_text())
    doc = DOC.read_text()
    start = doc.find(BEGIN)
    if start == -1:
        start = doc.find(START_HEADING)
    end = doc.find(END_HEADING)
    if start == -1 or end == -1 or end < start:
        print(f"ERROR: {DOC} has no '{START_HEADING}' section before '{END_HEADING}'", file=sys.stderr)
        return 1

    updated = doc[:start] + render(fixture) + doc[end:]
    if args.check:
        if updated != doc:
            print("ERROR: docs/error-codes.md is out of date with error-codes.json. "
                  "Run: python3 stellar-contracts/scripts/generate_error_docs.py", file=sys.stderr)
            return 1
        print("docs/error-codes.md matches error-codes.json.")
        return 0

    DOC.write_text(updated)
    print(f"Wrote {DOC.relative_to(ROOT.parent)}.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
