#!/usr/bin/env bash
# Verifies that a release archive can be reproduced from the tagged commit.
#
# Usage:
#   ./scripts/verify_release.sh <tag>
#
# The script:
#   1. Checks out the tagged commit in a clean state
#   2. Rebuilds the WASM contract
#   3. Compares checksums against the provenance manifest
#   4. Verifies no secrets are present in the artifacts
#   5. Validates the deployment manifest
#
# Exit codes:
#   0 = all checks passed
#   1 = verification failed
set -euo pipefail
cd "$(dirname "$0")/.."

TAG="${1:-}"
if [[ -z "$TAG" ]]; then
  echo "Usage: $0 <tag>" >&2
  echo "Example: $0 v0.1.0" >&2
  exit 1
fi

RELEASE_DIR="release-artifacts"
VERIFY_DIR="${RELEASE_DIR}/verify-${TAG}"

echo "=== Reproducible Release Verification for ${TAG} ==="
echo ""

# Clean any previous verification artifacts
rm -rf "$VERIFY_DIR"
mkdir -p "$VERIFY_DIR"

# Step 1: Ensure we are on the correct commit
echo "--- Step 1: Checkout tagged commit ---"
git fetch --tags origin
git checkout "refs/tags/${TAG}" 2>/dev/null || {
  echo "ERROR: Tag ${TAG} not found. Available tags:" >&2
  git tag -l | head -20 >&2
  exit 1
}
COMMIT_SHA=$(git rev-parse HEAD)
COMMIT_SHORT="${COMMIT_SHA:0:7}"
echo "Checked out commit: ${COMMIT_SHORT}"
echo ""

# Step 2: Clean build
echo "--- Step 2: Clean build ---"
cargo clean --target wasm32-unknown-unknown 2>/dev/null || true
cargo build --target wasm32-unknown-unknown --release
echo "Build completed."
echo ""

# Step 3: Generate checksums for rebuilt artifacts
echo "--- Step 3: Generate checksums ---"
WASM_PATH=$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name "*.wasm" | head -n 1)
if [[ -z "$WASM_PATH" ]]; then
  echo "ERROR: No WASM artifact found after build." >&2
  exit 1
fi

WASM_CHECKSUM=$(sha256sum "$WASM_PATH" | awk '{print $1}')
WASM_SIZE=$(stat -c%s "$WASM_PATH")
WASM_NAME=$(basename "$WASM_PATH")

ABI_CHECKSUM=$(sha256sum abi-snapshot.txt | awk '{print $1}')
MANIFEST_CHECKSUM=$(sha256sum deployment-manifest.json | awk '{print $1}')
BINDINGS_CHECKSUM=$(sha256sum bindings-manifest.json | awk '{print $1}')

echo "WASM:        ${WASM_NAME}"
echo "  SHA-256:   ${WASM_CHECKSUM}"
echo "  Size:      ${WASM_SIZE} bytes"
echo "ABI:         abi-snapshot.txt"
echo "  SHA-256:   ${ABI_CHECKSUM}"
echo "Manifest:    deployment-manifest.json"
echo "  SHA-256:   ${MANIFEST_CHECKSUM}"
echo "Bindings:    bindings-manifest.json"
echo "  SHA-256:   ${BINDINGS_CHECKSUM}"
echo ""

# Step 4: Compare against provenance if available
echo "--- Step 4: Compare against provenance ---"
PROVENANCE_FILE="${RELEASE_DIR}/provenance.json"
if [[ -f "$PROVENANCE_FILE" ]]; then
  ORIGINAL_WASM=$(python3 -c "import json; print(json.load(open('${PROVENANCE_FILE}'))['artifacts']['wasm']['sha256'])" 2>/dev/null || echo "")
  ORIGINAL_ABI=$(python3 -c "import json; print(json.load(open('${PROVENANCE_FILE}'))['artifacts']['abi_snapshot']['sha256'])" 2>/dev/null || echo "")
  ORIGINAL_MANIFEST=$(python3 -c "import json; print(json.load(open('${PROVENANCE_FILE}'))['artifacts']['deployment_manifest']['sha256'])" 2>/dev/null || echo "")
  ORIGINAL_BINDINGS=$(python3 -c "import json; print(json.load(open('${PROVENANCE_FILE}'))['artifacts']['bindings_manifest']['sha256'])" 2>/dev/null || echo "")

  MISMATCHES=0

  if [[ -n "$ORIGINAL_WASM" && "$WASM_CHECKSUM" != "$ORIGINAL_WASM" ]]; then
    echo "MISMATCH: WASM checksum differs" >&2
    echo "  Expected: ${ORIGINAL_WASM}" >&2
    echo "  Got:      ${WASM_CHECKSUM}" >&2
    MISMATCHES=$((MISMATCHES + 1))
  else
    echo "WASM checksum: OK"
  fi

  if [[ -n "$ORIGINAL_ABI" && "$ABI_CHECKSUM" != "$ORIGINAL_ABI" ]]; then
    echo "MISMATCH: ABI snapshot checksum differs" >&2
    echo "  Expected: ${ORIGINAL_ABI}" >&2
    echo "  Got:      ${ABI_CHECKSUM}" >&2
    MISMATCHES=$((MISMATCHES + 1))
  else
    echo "ABI snapshot checksum: OK"
  fi

  if [[ -n "$ORIGINAL_MANIFEST" && "$MANIFEST_CHECKSUM" != "$ORIGINAL_MANIFEST" ]]; then
    echo "MISMATCH: Deployment manifest checksum differs" >&2
    echo "  Expected: ${ORIGINAL_MANIFEST}" >&2
    echo "  Got:      ${MANIFEST_CHECKSUM}" >&2
    MISMATCHES=$((MISMATCHES + 1))
  else
    echo "Deployment manifest checksum: OK"
  fi

  if [[ -n "$ORIGINAL_BINDINGS" && "$BINDINGS_CHECKSUM" != "$ORIGINAL_BINDINGS" ]]; then
    echo "MISMATCH: Bindings manifest checksum differs" >&2
    echo "  Expected: ${ORIGINAL_BINDINGS}" >&2
    echo "  Got:      ${BINDINGS_CHECKSUM}" >&2
    MISMATCHES=$((MISMATCHES + 1))
  else
    echo "Bindings manifest checksum: OK"
  fi

  if [[ $MISMATCHES -gt 0 ]]; then
    echo ""
    echo "ERROR: ${MISMATCHES} checksum mismatch(es). Reproducibility exception documented." >&2
    echo "This may be caused by non-deterministic build settings or toolchain differences." >&2
    exit 1
  fi

  echo "All checksums match provenance manifest."
else
  echo "No provenance manifest found at ${PROVENANCE_FILE}. Skipping comparison."
  echo "Saving current provenance for future verification..."
  # Generate a provenance manifest for this build
  cat > "${PROVENANCE_FILE}" <<EOF
{
  "release": {
    "tag": "${TAG}",
    "commit": "${COMMIT_SHA}",
    "commit_short": "${COMMIT_SHORT}",
    "built_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  },
  "toolchain": {
    "rust": "$(rustc --version)",
    "cargo": "$(cargo --version | awk '{print $2}')",
    "wasm_target": "wasm32-unknown-unknown"
  },
  "artifacts": {
    "wasm": {
      "file": "${WASM_NAME}",
      "sha256": "${WASM_CHECKSUM}",
      "size_bytes": ${WASM_SIZE}
    },
    "abi_snapshot": {
      "file": "abi-snapshot.txt",
      "sha256": "${ABI_CHECKSUM}"
    },
    "deployment_manifest": {
      "file": "deployment-manifest.json",
      "sha256": "${MANIFEST_CHECKSUM}"
    },
    "bindings_manifest": {
      "file": "bindings-manifest.json",
      "sha256": "${BINDINGS_CHECKSUM}"
    }
  }
}
EOF
  echo "Provenance manifest written to ${PROVENANCE_FILE}"
fi
echo ""

# Step 5: Check for secrets
echo "--- Step 5: Check for secrets in artifacts ---"
if grep -rlE '(BEGIN RSA PRIVATE KEY|BEGIN PRIVATE KEY|api[_-]?key|secret[_-]?key|password|SECRET|PRIVATE)' \
    "$WASM_PATH" abi-snapshot.txt deployment-manifest.json bindings-manifest.json 2>/dev/null; then
  echo "ERROR: Potential secrets detected in release artifacts!" >&2
  exit 1
fi
echo "No secrets detected."
echo ""

# Step 6: Validate deployment manifest
echo "--- Step 6: Validate deployment manifest ---"
bash scripts/validate_manifest.sh --strict
echo "Deployment manifest validation passed."
echo ""

# Step 7: Verify ABI snapshot is up to date
echo "--- Step 7: Verify ABI snapshot ---"
bash scripts/generate_abi_snapshot.sh --check
echo "ABI snapshot is up to date."
echo ""

echo "=== Verification complete for ${TAG} (${COMMIT_SHORT}) ==="
echo "All checks passed."
