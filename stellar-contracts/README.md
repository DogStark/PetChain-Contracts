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

## Notes

- The main contract source lives in `src/lib.rs`.
- The contract test suite is split across focused `src/test_*.rs` modules.
- High-level repository docs now live in the root `docs/` directory.
