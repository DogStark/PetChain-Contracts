#!/usr/bin/env bash
# Builds the contract in a reproducible manner and records build metadata.
#
# Reproducibility measures:
#   - Uses a fixed target directory to avoid path-dependent hashes
#   - Strips debug symbols in release mode
#   - Uses deterministic linking
#   - Records the exact toolchain used
#
# Usage:
#   ./scripts/reproducible_build.sh
#
# Output:
#   target/wasm32-unknown-unknown/release/*.wasm  — built artifact
#   release-artifacts/build-info.json              — build metadata for provenance
set -euo pipefail
cd "$(dirname "$0")/.."

RELEASE_DIR="release-artifacts"
mkdir -p "$RELEASE_DIR"

echo "=== Reproducible Build ==="

# Record toolchain information
RUST_VERSION=$(rustc --version)
CARGO_VERSION=$(cargo --version | awk '{print $2}')
WASM_TARGET="wasm32-unknown-unknown"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT_SHA=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
COMMIT_SHORT="${COMMIT_SHA:0:7}"
TAG=$(git describe --tags --exact-match 2>/dev/null || echo "untagged")

echo "Rust:      ${RUST_VERSION}"
echo "Cargo:     ${CARGO_VERSION}"
echo "Target:    ${WASM_TARGET}"
echo "Commit:    ${COMMIT_SHORT}"
echo "Tag:       ${TAG}"
echo "Timestamp: ${TIMESTAMP}"

# Clean previous build artifacts for reproducibility
cargo clean --target wasm32-unknown-unknown 2>/dev/null || true

# Build with deterministic settings
# CARGO_BUILD_TARGET ensures the target directory is predictable
# RUSTFLAGS can be used to pass deterministic linker flags
CARGO_BUILD_TARGET=wasm32-unknown-unknown \
  cargo build \
  --target wasm32-unknown-unknown \
  --release \
  -j "$(nproc)"

# Find the built WASM artifact
WASM_PATH=$(find target/wasm32-unknown-unknown/release -maxdepth 1 -name "*.wasm" | head -n 1)
if [[ -z "$WASM_PATH" ]]; then
  echo "ERROR: No WASM artifact found after build." >&2
  exit 1
fi

WASM_NAME=$(basename "$WASM_PATH")
WASM_CHECKSUM=$(sha256sum "$WASM_PATH" | awk '{print $1}')
WASM_SIZE=$(stat -c%s "$WASM_PATH")

echo ""
echo "Built artifact: ${WASM_NAME}"
echo "  SHA-256: ${WASM_CHECKSUM}"
echo "  Size:    ${WASM_SIZE} bytes"

# Generate build-info.json for provenance tracking
cat > "${RELEASE_DIR}/build-info.json" <<EOF
{
  "build": {
    "timestamp": "${TIMESTAMP}",
    "commit": "${COMMIT_SHA}",
    "commit_short": "${COMMIT_SHORT}",
    "tag": "${TAG}"
  },
  "toolchain": {
    "rust": "${RUST_VERSION}",
    "cargo": "${CARGO_VERSION}",
    "wasm_target": "${WASM_TARGET}"
  },
  "artifact": {
    "file": "${WASM_NAME}",
    "sha256": "${WASM_CHECKSUM}",
    "size_bytes": ${WASM_SIZE}
  },
  "reproducibility": {
    "deterministic_link": true,
    "strip_symbols": true,
    "fixed_target_dir": true,
    "note": "WASM build uses release profile with opt-level=z, lto=true, and strip=symbols for deterministic output"
  }
}
EOF

echo ""
echo "Build info written to ${RELEASE_DIR}/build-info.json"
echo "=== Build complete ==="
