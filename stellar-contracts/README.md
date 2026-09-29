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

## ABI Snapshot & Diff

The contract's public ABI is tracked in `abi-snapshot.txt` and can be
regenerated with:

```bash
./scripts/generate_abi_snapshot.sh          # regenerate snapshot
./scripts/generate_abi_snapshot.sh --check  # verify snapshot is up to date
./scripts/generate_abi_snapshot.sh --diff   # check for breaking changes
```

To compare two snapshots and get a categorized compatibility summary:

```bash
python3 scripts/abi_diff.py <old-snapshot> <new-snapshot>
```

The diff tool groups changes by category (methods, events, errors, types)
and labels each as additive or breaking. CI requires a migration note in
`docs/abi-migrations.md` for every breaking change.

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
