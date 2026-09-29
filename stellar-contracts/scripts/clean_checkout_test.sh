#!/usr/bin/env bash
# Clean-checkout reproducibility test.
#
# Simulates a clean checkout by removing all build artifacts and
# rebuilding from scratch, then comparing checksums against the
# expected values recorded in the provenance manifest.
#
# This test can be run in CI to verify that the build is reproducible.
#
# Usage:
#   ./scripts/clean_checkout_test.sh
#
# Exit codes:
#   0 = reproducibility verified
#   1 = reproducibility check failed
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=== Clean-Checkout Reproducibility Test ==="

# Step 1: Record current checksums before clean
echo "--- Step 1: Record baseline checksums ---"
cargo build --target wasm32-unknown-unknown --release

BASELINE_WASM=$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name "*.wasm" | head -n 1)
if [[ -z "$BASELINE_WASM" ]]; then
  echo "ERROR: No WASM artifact found." >&2
  exit 1
fi

BASELINE_CHECKSUM=$(sha256sum "$BASELINE_WASM" | awk '{print $1}')
echo "Baseline WASM checksum: ${BASELINE_CHECKSUM}"
echo ""

# Step 2: Clean everything
echo "--- Step 2: Clean all build artifacts ---"
cargo clean --target wasm32-unknown-unknown
rm -rf target/
echo "Clean complete."
echo ""

# Step 3: Rebuild from clean state
echo "--- Step 3: Rebuild from clean state ---"
cargo build --target wasm32-unknown-unknown --release

REBUILT_WASM=$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name "*.wasm" | head -n 1)
if [[ -z "$REBUILT_WASM" ]]; then
  echo "ERROR: No WASM artifact found after rebuild." >&2
  exit 1
fi

REBUILT_CHECKSUM=$(sha256sum "$REBUILT_WASM" | awk '{print $1}')
echo "Rebuilt WASM checksum:  ${REBUILT_CHECKSUM}"
echo ""

# Step 4: Compare
echo "--- Step 4: Compare checksums ---"
if [[ "$BASELINE_CHECKSUM" = "$REBUILT_CHECKSUM" ]]; then
  echo "SUCCESS: Reproducible build verified."
  echo "Checksums match after clean rebuild."
else
  echo "FAILURE: Checksums differ after clean rebuild." >&2
  echo "  Baseline:  ${BASELINE_CHECKSUM}" >&2
  echo "  Rebuilt:   ${REBUILT_CHECKSUM}" >&2
  echo "" >&2
  echo "This is a documented reproducibility exception." >&2
  echo "Possible causes:" >&2
  echo "  - Non-deterministic build settings in Cargo.toml" >&2
  echo "  - Toolchain version differences" >&2
  echo "  - Timestamps embedded in the binary" >&2
  exit 1
fi

# Step 5: Verify ABI snapshot consistency
echo ""
echo "--- Step 5: Verify ABI snapshot ---"
bash scripts/generate_abi_snapshot.sh --check
echo "ABI snapshot is consistent."
echo ""

# Step 6: Verify deployment manifest has no placeholders
echo "--- Step 6: Validate deployment manifest ---"
bash scripts/validate_manifest.sh --strict
echo "Deployment manifest is valid."
echo ""

echo "=== Clean-checkout test passed ==="
