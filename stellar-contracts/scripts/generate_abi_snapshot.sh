#!/usr/bin/env bash
# Regenerates stellar-contracts/abi-snapshot.txt from the current
# `#[contractimpl]` block and type definitions in src/lib.rs.
#
# The snapshot is structured into four sections:
#   # Methods   — every ``pub fn`` signature in the ``#[contractimpl]`` block
#   # Events    — event structs (``#[contracttype]`` structs with a ``version: u32`` field)
#   # Errors    — ``ContractError`` enum variants with discriminants
#   # Types     — all other ``#[contracttype]`` structs and enums
#
# Usage:
#   ./scripts/generate_abi_snapshot.sh          # writes abi-snapshot.txt
#   ./scripts/generate_abi_snapshot.sh --check   # exits non-zero if the
#                                                 # committed snapshot is stale
#   ./scripts/generate_abi_snapshot.sh --diff    # runs abi_diff.py against
#                                                 # the committed snapshot and
#                                                 # the freshly generated one;
#                                                 # exits non-zero if breaking
#                                                 # changes are detected.
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

elif [[ "${1:-}" == "--diff" ]]; then
  # Write the freshly generated snapshot to a temp file and diff it
  # against the committed one using abi_diff.py.
  TMPFILE="$(mktemp)"
  printf '%s\n' "$GENERATED" > "$TMPFILE"
  trap 'rm -f "$TMPFILE"' EXIT

  if ! python3 scripts/abi_diff.py abi-snapshot.txt "$TMPFILE"; then
    echo "" >&2
    echo "ERROR: breaking ABI changes detected." >&2
    echo "Add a migration note to docs/abi-migrations.md describing the change" >&2
    echo "and its ledger-compatibility impact, then regenerate the snapshot with:" >&2
    echo "  ./scripts/generate_abi_snapshot.sh" >&2
    exit 1
  fi
  echo "No breaking ABI changes detected."

else
  printf '%s\n' "$GENERATED" > abi-snapshot.txt
  echo "Wrote abi-snapshot.txt ($(wc -l < abi-snapshot.txt) lines)."
fi
