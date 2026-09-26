# Stellar Contracts

This crate contains the main Soroban smart contract for PetChain, plus the nested transfer/adoption contract package under `contracts/pet-transfer-adoption/`.

## Commands

```bash
cargo fmt
cargo test
```

To build a release artifact:

```bash
cargo build --target wasm32-unknown-unknown --release
```

## Release Verification

Every release archive is verified for reproducibility and provenance. See [docs/release-verification.md](docs/release-verification.md) for details.

### Quick checks

```bash
# Verify clean-checkout reproducibility
bash scripts/clean_checkout_test.sh

# Scan artifacts for secrets
bash scripts/check_no_secrets.sh release-artifacts

# Validate deployment manifest
bash scripts/validate_manifest.sh --strict

# Verify ABI snapshot is up to date
bash scripts/generate_abi_snapshot.sh --check
```

## Notes

- The main contract source lives in `src/lib.rs`.
- The contract test suite is split across focused `src/test_*.rs` modules.
- High-level repository docs now live in the root `docs/` directory.
