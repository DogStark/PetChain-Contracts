# Deployment Manifest

## Overview

The deployment manifest (`deployment-manifest.json`) provides a single source
of truth for contract deployments across all networks. It records the contract
identifier, WASM hash, interface version, initialization state, and upgrade
history for each target network.

## Structure

```json
{
  "manifest_version": "1.0.0",
  "networks": {
    "<network-name>": {
      "network_passphrase": "...",
      "rpc_url": "...",
      "contract_id": "C...",
      "wasm_hash": "<hex>",
      "interface_version": "1.0.0",
      "deployed_at": "ISO-8601",
      "initialized": true,
      "upgrade_history": [
        {
          "from_version": "1.0.0",
          "to_version": "1.1.0",
          "wasm_hash": "<hex>",
          "timestamp": "ISO-8601",
          "tx_hash": "<hex>"
        }
      ]
    }
  }
}
```

## Validation Rules

The `validate_manifest.sh` script enforces:

1. **No shared addresses**: Testnet and mainnet cannot share a `contract_id`.
2. **No shared passphrases**: Each network must have a unique `network_passphrase`.
3. **No placeholder hashes** (strict mode): CI rejects `PLACEHOLDER_*` values.
4. **Valid semver**: `interface_version` must follow `MAJOR.MINOR.PATCH`.
5. **Ordered history**: `upgrade_history` entries must be chronologically sorted.

## CI Integration

Add to your CI workflow:

```yaml
- name: Validate deployment manifest
  run: ./scripts/validate_manifest.sh --strict
```

## Promoting a Deployment

1. Deploy the contract to testnet
2. Update `deployment-manifest.json` with the new contract ID and WASM hash
3. Run `./scripts/validate_manifest.sh` to verify
4. After testnet validation, update the mainnet entry
5. Create a PR with the manifest changes

## Rolling Back

1. Find the previous WASM hash in `upgrade_history`
2. Use `soroban contract invoke ... -- execute_upgrade` with the old hash
3. Update the manifest to reflect the rollback
4. Add an entry to `upgrade_history` documenting the rollback

## Verifying On-Chain State

```bash
./scripts/validate_manifest.sh --verify testnet
```

This queries the deployed contract's version and compares it against the
manifest. Mismatches are reported with actionable details.
