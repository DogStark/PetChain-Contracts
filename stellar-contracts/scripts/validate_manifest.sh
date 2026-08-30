#!/usr/bin/env bash
# Validates the deployment manifest for consistency across networks.
#
# Checks:
#   1. Testnet and mainnet cannot share a contract_id or network_passphrase.
#   2. WASM hashes must not be empty placeholders in CI (when --strict).
#   3. No duplicate contract addresses across networks.
#   4. Interface versions are valid semver.
#   5. Upgrade history entries are ordered chronologically.
#
# Usage:
#   ./scripts/validate_manifest.sh                  # basic validation
#   ./scripts/validate_manifest.sh --strict          # CI: reject placeholders
#   ./scripts/validate_manifest.sh --verify <network> # verify on-chain state
#
# Exit codes:
#   0 = valid
#   1 = validation error
set -euo pipefail
cd "$(dirname "$0")/.."

MANIFEST="deployment-manifest.json"
STRICT=false
VERIFY_NETWORK=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) STRICT=true; shift ;;
    --verify) VERIFY_NETWORK="$2"; shift 2 ;;
    *) echo "Unknown flag: $1" >&2; exit 1 ;;
  esac
done

if [[ ! -f "$MANIFEST" ]]; then
  echo "ERROR: $MANIFEST not found." >&2
  exit 1
fi

ERRORS=0

# Helper: report an error and increment counter
err() {
  echo "ERROR: $1" >&2
  ERRORS=$((ERRORS + 1))
}

# Parse manifest with python3 (available in CI)
validate() {
  python3 - "$MANIFEST" "$STRICT" <<'PYEOF'
import json, sys

manifest_path = sys.argv[1]
strict = sys.argv[2] == "True"
errors = 0

def err(msg):
    global errors
    print(f"ERROR: {msg}", file=sys.stderr)
    errors += 1

with open(manifest_path) as f:
    manifest = json.load(f)

networks = manifest.get("networks", {})
if not networks:
    err("No networks defined in manifest")
    sys.exit(1)

# Collect all contract_ids and passphrases for cross-network checks
contract_ids = {}
passphrases = {}

for name, config in networks.items():
    cid = config.get("contract_id", "")
    passphrase = config.get("network_passphrase", "")
    wasm_hash = config.get("wasm_hash", "")
    iface_version = config.get("interface_version", "")

    # Check for placeholder values in strict mode
    if strict:
        if "PLACEHOLDER" in cid:
            err(f"[{name}] contract_id is a placeholder")
        if "PLACEHOLDER" in wasm_hash:
            err(f"[{name}] wasm_hash is a placeholder")

    # Collect for uniqueness checks
    if cid and "PLACEHOLDER" not in cid:
        if cid in contract_ids:
            err(f"[{name}] contract_id '{cid}' duplicates {contract_ids[cid]}")
        contract_ids[cid] = name

    if passphrase:
        if passphrase in passphrases:
            err(f"[{name}] network_passphrase duplicates {passphrases[passphrase]}")
        passphrases[passphrase] = name

    # Validate interface_version looks like semver
    if iface_version:
        parts = iface_version.split(".")
        if len(parts) != 3 or not all(p.isdigit() for p in parts):
            err(f"[{name}] interface_version '{iface_version}' is not valid semver")

    # Validate upgrade_history is chronologically ordered
    history = config.get("upgrade_history", [])
    for i in range(1, len(history)):
        prev_ts = history[i - 1].get("timestamp", "")
        curr_ts = history[i].get("timestamp", "")
        if curr_ts <= prev_ts:
            err(f"[{name}] upgrade_history is not chronologically ordered at index {i}")

sys.exit(1 if errors > 0 else 0)
PYEOF
}

validate
RESULT=$?

if [[ -n "$VERIFY_NETWORK" ]]; then
  echo "Verifying on-chain state for network: $VERIFY_NETWORK"

  # Extract network config
  RPC_URL=$(python3 -c "import json; m=json.load(open('$MANIFEST')); print(m['networks']['$VERIFY_NETWORK']['rpc_url'])")
  CONTRACT_ID=$(python3 -c "import json; m=json.load(open('$MANIFEST')); print(m['networks']['$VERIFY_NETWORK']['contract_id'])")
  EXPECTED_HASH=$(python3 -c "import json; m=json.load(open('$MANIFEST')); print(m['networks']['$VERIFY_NETWORK']['wasm_hash'])")

  if [[ "$CONTRACT_ID" == *"PLACEHOLDER"* ]]; then
    echo "SKIP: contract_id is a placeholder for $VERIFY_NETWORK"
  elif command -v soroban &>/dev/null; then
    echo "  RPC:         $RPC_URL"
    echo "  Contract:    $CONTRACT_ID"
    echo "  Expected:    $EXPECTED_HASH"

    ACTUAL_VERSION=$(soroban contract invoke \
      --rpc-url "$RPC_URL" \
      --network-passphrase "$(python3 -c "import json; m=json.load(open('$MANIFEST')); print(m['networks']['$VERIFY_NETWORK']['network_passphrase'])")" \
      --id "$CONTRACT_ID" \
      -- get_version 2>/dev/null || echo "UNREACHABLE")

    if [[ "$ACTUAL_VERSION" == "UNREACHABLE" ]]; then
      echo "  WARNING: Could not reach contract. Network may be unavailable."
    else
      echo "  On-chain version: $ACTUAL_VERSION"
    fi
  else
    echo "  SKIP: soroban CLI not installed"
  fi
fi

if [[ $RESULT -eq 0 ]]; then
  echo "Deployment manifest is valid."
else
  echo "" >&2
  echo "Deployment manifest validation FAILED." >&2
fi

exit $RESULT
