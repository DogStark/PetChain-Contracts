#!/usr/bin/env python3
"""Test suite for abi_diff.py using snapshot fixtures.

Each fixture pair (old/new) exercises one category of ABI change:
  - additive-method, breaking-method
  - additive-event, breaking-event
  - additive-error, breaking-error
  - additive-type, breaking-type

Run with:  python3 scripts/test_abi_diff.py
"""
import os
import subprocess
import sys

FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "..", "fixtures")

PAIRS = [
    ("additive-method", 0),
    ("breaking-method", 1),
    ("additive-event", 0),
    ("breaking-event", 1),
    ("additive-error", 0),
    ("breaking-error", 1),
    ("additive-type", 0),
    ("breaking-type", 1),
]


def main():
    failures = []
    for category, expected_exit in PAIRS:
        old = os.path.join(FIXTURES_DIR, f"{category}-old.txt")
        new = os.path.join(FIXTURES_DIR, f"{category}-new.txt")

        result = subprocess.run(
            [sys.executable, os.path.join(os.path.dirname(__file__), "abi_diff.py"), old, new],
            capture_output=True,
            text=True,
        )

        if result.returncode != expected_exit:
            failures.append(
                f"{category}: expected exit code {expected_exit}, got {result.returncode}"
            )
            print(f"FAIL: {category}")
            print(result.stdout)
        else:
            print(f"PASS: {category}")

    if failures:
        print(f"\n{len(failures)} test(s) failed:")
        for f in failures:
            print(f"  - {f}")
        sys.exit(1)

    print(f"\nAll {len(PAIRS)} tests passed.")


if __name__ == "__main__":
    main()
