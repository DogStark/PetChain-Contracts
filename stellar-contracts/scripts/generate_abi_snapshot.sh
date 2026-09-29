#!/usr/bin/env bash
# Regenerates stellar-contracts/abi-snapshot.txt from the current
# `#[contractimpl]` block in src/lib.rs.
#
# Usage:
#   ./scripts/generate_abi_snapshot.sh          # writes abi-snapshot.txt
#   ./scripts/generate_abi_snapshot.sh --check   # exits non-zero if the
#                                                 # committed snapshot is stale
#
# See docs/abi-migrations.md for what to do when this script reports a diff.
set -euo pipefail
cd "$(dirname "$0")/.."

GENERATED="$(python3 scripts/generate_abi_snapshot.py src/lib.rs)"

if [[ "${1:-}" == "--check" ]]; then
  if ! diff -u abi-snapshot.txt <(printf '%s\n' "$GENERATED"); then
    echo "" >&2
    echo "ERROR: the public contract ABI no longer matches abi-snapshot.txt." >&2
    echo "If this change is intentional, run './scripts/generate_abi_snapshot.sh'" >&2
    echo "to update the snapshot and add an entry to docs/abi-migrations.md." >&2
    exit 1
  fi
  echo "abi-snapshot.txt is up to date."
else
  printf '%s\n' "$GENERATED" > abi-snapshot.txt
  echo "Wrote abi-snapshot.txt ($(wc -l < abi-snapshot.txt) functions)."
fi
