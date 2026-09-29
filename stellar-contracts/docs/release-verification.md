# Release Verification

This document describes the reproducible release archive verification process for the PetChain Stellar contracts.

## Overview

Every release archive must be traceable to the commit, toolchain, manifests, and checksums used to build it. This process ensures:

1. **Reproducibility**: Rebuilding from the tagged commit produces matching checksums.
2. **No secrets**: Archives contain no secrets or sensitive material.
3. **Provenance**: A provenance manifest records the exact build environment and artifact checksums.

## Release Workflow

Releases are triggered by pushing a tag matching `v*`:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The [`stellar-release.yml`](../.github/workflows/stellar-release.yml) workflow runs automatically and:

1. Builds the contract in release mode
2. Generates SHA-256 checksums for WASM, ABI, deployment manifest, and bindings manifest
3. Creates a provenance manifest (`provenance.json`) recording toolchain, commit, and checksums
4. Scans artifacts for secrets
5. Packages the release archive
6. Creates a GitHub Release with all artifacts attached
7. Runs a reproducibility check by rebuilding from the tagged commit and comparing checksums

## Verification Scripts

### `scripts/clean_checkout_test.sh`

Simulates a clean checkout by removing all build artifacts and rebuilding from scratch, then comparing checksums against the baseline. This is run in CI on every push to `main`.

```bash
cd stellar-contracts
bash scripts/clean_checkout_test.sh
```

### `scripts/verify_release.sh <tag>`

Verifies that a specific tagged release can be reproduced. It checks out the tag, rebuilds, and compares all checksums against the provenance manifest.

```bash
cd stellar-contracts
bash scripts/verify_release.sh v0.1.0
```

### `scripts/check_no_secrets.sh [directory]`

Scans release artifacts for common secret patterns (private keys, API keys, passwords, tokens). Defaults to the `release-artifacts` directory.

```bash
cd stellar-contracts
bash scripts/check_no_secrets.sh release-artifacts
```

### `scripts/reproducible_build.sh`

Builds the contract with deterministic settings and records build metadata in `release-artifacts/build-info.json`.

```bash
cd stellar-contracts
bash scripts/reproducible_build.sh
```

## Provenance Manifest

The `provenance.json` file in each release archive records:

| Field | Description |
|-------|-------------|
| `release.tag` | Git tag (e.g., `v0.1.0`) |
| `release.commit` | Full commit SHA |
| `release.commit_short` | Short commit SHA (7 chars) |
| `release.built_at` | ISO-8601 timestamp of the build |
| `toolchain.rust` | Rust compiler version |
| `toolchain.cargo` | Cargo version |
| `toolchain.wasm_target` | WASM target triple |
| `toolchain.stellar_cli` | Stellar CLI version |
| `artifacts.*.sha256` | SHA-256 checksum of each artifact |
| `artifacts.*.size_bytes` | Size of WASM artifact in bytes |

## Reproducibility Exceptions

If a rebuild produces different checksums, this is documented as a reproducibility exception. Common causes include:

- **Non-deterministic build settings**: The `Cargo.toml` release profile uses `opt-level=z`, `lto=true`, `codegen-units=1`, `debug=0`, `strip=symbols`, and `panic=abort` to maximize determinism.
- **Toolchain version differences**: Different Rust versions may produce different WASM output. The provenance manifest records the exact toolchain used.
- **Timestamps embedded in the binary**: Some build tools embed timestamps. The release profile strips debug symbols to minimize this.

All exceptions must be documented in the release notes or in `docs/reproducibility-exceptions.md`.

## CI Integration

The reproducibility check runs as part of the `stellar-contracts` CI workflow:

- **`reproducibility` job**: Runs on every push to `main` and every PR
  - Clean-checkout reproducibility test
  - Secrets scan
  - ABI snapshot consistency check
  - Deployment manifest validation

## Adding a New Release

1. Ensure all tests pass: `cd stellar-contracts && cargo test`
2. Ensure ABI snapshot is up to date: `bash scripts/generate_abi_snapshot.sh`
3. Ensure deployment manifest is valid: `bash scripts/validate_manifest.sh --strict`
4. Create and push a tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
5. The release workflow runs automatically and produces:
   - `petchain-stellar-release-vX.Y.Z.tar.gz`
   - `petchain-stellar-release-vX.Y.Z-with-manifests.tar.gz`
   - `provenance.json` with full build metadata
   - `checksums.txt` with SHA-256 checksums of all artifacts
