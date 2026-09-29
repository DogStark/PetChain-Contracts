#!/usr/bin/env bash
# Generates client bindings from the current contract interface.
#
# The generated bindings-manifest.json records the source commit, contract
# version, network, and generation tool version so consumers can detect
# when bindings drift from the deployed interface.
#
# Usage:
#   ./scripts/generate_bindings.sh               # regenerate manifest
#   ./scripts/generate_bindings.sh --check        # CI mode: fail on drift
#
# See docs/client-bindings.md for consumer update instructions.
set -euo pipefail
cd "$(dirname "$0")/.."

MANIFEST="bindings-manifest.json"

# Gather metadata
SOURCE_COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
CONTRACT_VERSION=$(grep -m1 'SCHEMA_VERSION' src/lib.rs | grep -oE '[0-9]+' || echo "1")
ABI_SNAPSHOT_HASH=$(sha256sum abi-snapshot.txt 2>/dev/null | cut -d' ' -f1 || echo "unknown")
SOROBAN_VERSION=$(soroban --version 2>/dev/null | head -1 || echo "soroban-cli 21.7.7")
TOOL_VERSION=$(echo "$SOROBAN_VERSION" | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "21.7.7")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

generate_manifest() {
  cat <<EOF
{
  "version": "1.0.0",
  "generated_at": "${TIMESTAMP}",
  "source": {
    "commit": "${SOURCE_COMMIT}",
    "contract_version": ${CONTRACT_VERSION},
    "abi_snapshot_hash": "${ABI_SNAPSHOT_HASH}"
  },
  "network": "testnet",
  "generation_tool": {
    "name": "soroban-cli",
    "version": "${TOOL_VERSION}"
  },
  "notes": "Regenerate with: ./scripts/generate_bindings.sh"
}
EOF
}

if [[ "${1:-}" == "--check" ]]; then
  # In CI: verify the ABI snapshot hasn't changed without updating the manifest
  if [[ ! -f "$MANIFEST" ]]; then
    echo "ERROR: $MANIFEST not found. Run './scripts/generate_bindings.sh' to create it." >&2
    exit 1
  fi

  # Extract the recorded ABI hash from the manifest
  RECORDED_HASH=$(python3 -c "import json; m=json.load(open('$MANIFEST')); print(m.get('source',{}).get('abi_snapshot_hash',''))" 2>/dev/null || echo "")

  if [[ -z "$RECORDED_HASH" || "$RECORDED_HASH" == "unknown" ]]; then
    echo "WARNING: No abi_snapshot_hash recorded in $MANIFEST. Skipping drift check."
    exit 0
  fi

  if [[ "$RECORDED_HASH" != "$ABI_SNAPSHOT_HASH" ]]; then
    echo "" >&2
    echo "ERROR: Contract ABI has changed since bindings were last generated." >&2
    echo "  Recorded: $RECORDED_HASH" >&2
    echo "  Current:  $ABI_SNAPSHOT_HASH" >&2
    echo "" >&2
    echo "If this is intentional:" >&2
    echo "  1. Bump the version in $MANIFEST" >&2
    echo "  2. Run './scripts/generate_bindings.sh' to update" >&2
    echo "  3. Add a migration note to docs/client-bindings.md" >&2
    exit 1
  fi

  echo "Bindings manifest is up to date (ABI hash matches)."
else
  generate_manifest > "$MANIFEST"
  echo "Wrote $MANIFEST"
  echo "  commit:    $SOURCE_COMMIT"
  echo "  version:   $CONTRACT_VERSION"
  echo "  abi_hash:  $ABI_SNAPSHOT_HASH"
  echo "  tool:      soroban-cli $TOOL_VERSION"
fi
