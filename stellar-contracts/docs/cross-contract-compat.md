# Cross-Contract Interface Compatibility

The workspace contracts can each stay green while drifting apart. The
`cross-contract` CI job (Issue #1253) deploys them together into one local
Soroban environment and exercises the interfaces they share:

| Contract | Source | How it is deployed in the test |
|---|---|---|
| Main (`PetChainContract`) | `src/lib.rs` | Release WASM (`PETCHAIN_MAIN_WASM`) |
| Ownership / adoption / escrowed transfers (`PetOwnershipContract`) | `contracts/pet-transfer-adoption/src/lib.rs` | Native |
| Vet registry (`VetRegistryContract`) | `contracts/pet-transfer-adoption/src/vet_registry.rs` | Native |

Tests live in `contracts/pet-transfer-adoption/src/test_cross_contract.rs` and
check authorization, error codes, events and state transitions across these
contracts, including the ownership contract's trusted-main-contract binding.

## Compatibility fixture

The `fixture` module in that file fails when a shared interface drifts:

- **Signatures** of the pinned ownership and vet-registry functions are called
  with explicit types, so a changed signature fails to compile.
- **Enums** shared across contracts are matched exhaustively, and the error
  codes callers rely on are pinned numerically.
- **Main contract types** are decoded into local mirrors (e.g.
  `ContractVersion`), so a drifted main-contract type fails at runtime.

## Which interfaces are versioned

- **Intentionally versioned:** the main contract ABI (`abi-snapshot.txt`,
  [abi-migrations.md](abi-migrations.md), `get_version`) and main-contract
  event payloads (`EVENT_SCHEMA_VERSION`, [event-versioning.md](event-versioning.md)).
  Change them only through those processes.
- **Not versioned (append-only):** everything pinned in the fixture —
  ownership/escrowed-transfer and vet-registry functions, `TransferType`, their
  error codes, and event topics. Changing any of them is breaking; update the
  fixture only together with every dependent contract and client.
- **Intentionally different:** `verify_vet` takes an explicit `admin` on the
  main contract but uses the stored admin on the standalone vet registry.

## Running locally

```bash
cd stellar-contracts
cargo build --target wasm32-unknown-unknown --release
cd contracts/pet-transfer-adoption
PETCHAIN_MAIN_WASM=$(realpath ../../../target/wasm32-unknown-unknown/release/petchain_stellar.wasm) \
  cargo test test_cross_contract -- --include-ignored
```

Without `PETCHAIN_MAIN_WASM`, the main-contract test is skipped as ignored and
the remaining cross-contract tests still run.
