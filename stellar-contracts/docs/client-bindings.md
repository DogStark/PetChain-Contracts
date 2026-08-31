# Client Bindings

## Overview

Client bindings are generated from the pinned contract interface recorded in
`abi-snapshot.txt`. A versioned manifest (`bindings-manifest.json`) tracks the
source commit, contract version, network, and generation tool version so that
consumers can detect when bindings drift from the deployed interface.

## Generating Bindings

```bash
cd stellar-contracts
./scripts/generate_bindings.sh
```

This writes `bindings-manifest.json` with the current metadata.

## CI Check

CI runs the `--check` flag to verify the manifest is consistent with the
current ABI snapshot:

```bash
./scripts/generate_bindings.sh --check
```

If the ABI has changed without updating the manifest, CI fails with an
actionable error message.

## Breaking Changes

When the contract interface changes:

1. Update `abi-snapshot.txt` via `./scripts/generate_abi_snapshot.sh`
2. Bump the `version` field in `bindings-manifest.json`
3. Run `./scripts/generate_bindings.sh` to regenerate the manifest
4. Add a migration note below describing what changed and how consumers
   should update their code

## Update Instructions for Consumers

1. Pull the latest `bindings-manifest.json` from the repository
2. Compare the `source.contract_version` with your local copy
3. If the version has changed, regenerate your client code:
   ```bash
   soroban contract bindings typescript \
     --network testnet \
     --contract-id <CONTRACT_ID> \
     --output-dir ./src/generated
   ```
4. Review the migration notes below for any breaking changes

## Migration Notes

| Version | Date       | Description                          |
|---------|------------|--------------------------------------|
| 1.0.0   | 2026-08-30 | Initial pinned interface release     |
