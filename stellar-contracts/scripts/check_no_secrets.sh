#!/usr/bin/env bash
# Checks that release artifacts contain no secrets.
#
# Scans for common secret patterns in all files within the
# release-artifacts directory. This is a best-effort check;
# it should be run as part of the release verification pipeline.
#
# Usage:
#   ./scripts/check_no_secrets.sh [directory]
#
# Defaults to the release-artifacts directory in the stellar-contracts root.
#
# Exit codes:
#   0 = no secrets found
#   1 = potential secrets detected
set -euo pipefail
cd "$(dirname "$0")/.."

TARGET_DIR="${1:-release-artifacts}"

if [[ ! -d "$TARGET_DIR" ]]; then
  echo "ERROR: Directory ${TARGET_DIR} does not exist." >&2
  exit 1
fi

echo "Scanning ${TARGET_DIR} for potential secrets..."

SECRETS_FOUND=0

# Patterns that indicate a secret might be present
PATTERNS=(
  'BEGIN RSA PRIVATE KEY'
  'BEGIN PRIVATE KEY'
  'BEGIN EC PRIVATE KEY'
  'api_key[[:space:]]*='
  'secret_key[[:space:]]*='
  'api-secret[[:space:]]*='
  'authorization[[:space:]]*:'
  'bearer[[:space:]]+[A-Za-z0-9_-]{20,}'
  'password[[:space:]]*='
  'STELLAR_SECRET'
  'SECRET_KEY'
  'PRIVATE_KEY'
  'DATABASE_URL.*://[^:]+:[^@]+@'
)

for pattern in "${PATTERNS[@]}"; do
  matches=$(grep -rlE "$pattern" "$TARGET_DIR" 2>/dev/null || true)
  if [[ -n "$matches" ]]; then
    echo "WARNING: Potential secret match for pattern '${pattern}' in:" >&2
    echo "$matches" >&2
    SECRETS_FOUND=1
  fi
done

# Also check binary WASM files for embedded strings that look like secrets
for wasm_file in "$TARGET_DIR"/*.wasm; do
  [[ -f "$wasm_file" ]] || continue
  # Look for common secret patterns in the binary
  if strings "$wasm_file" 2>/dev/null | grep -qE '(BEGIN RSA PRIVATE KEY|BEGIN PRIVATE KEY|api_key|secret_key|password|SECRET_KEY|PRIVATE_KEY|authorization)'; then
    echo "WARNING: Potential secret string found in WASM file: ${wasm_file}" >&2
    SECRETS_FOUND=1
  fi
done

if [[ $SECRETS_FOUND -eq 1 ]]; then
  echo "" >&2
  echo "ERROR: Potential secrets detected in release artifacts!" >&2
  echo "Aborting release. Remove secrets before proceeding." >&2
  exit 1
fi

echo "No secrets detected in release artifacts."
exit 0
