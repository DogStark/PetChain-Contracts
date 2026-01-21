# Stellar Smart Contracts

This directory contains the Stellar (Soroban) smart contracts for PetChain.

## Prerequisites

Install Stellar CLI:

```bash
cargo install --locked stellar-cli --features opt
```

Or install Soroban CLI:

```bash
cargo install --locked soroban-cli
```

## Build

```bash
cd stellar-contracts
cargo build --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test
```

The test suite includes 43 comprehensive tests covering:

- All 13 public contract functions
- Edge cases and error conditions
- Integration workflows
- 100% function coverage

For detailed test documentation, see:

- [TEST_DOCUMENTATION.md](TEST_DOCUMENTATION.md) - Complete test documentation
- [TEST_SUMMARY.md](TEST_SUMMARY.md) - Test suite implementation summary

## Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/petchain_stellar.wasm \
  --network testnet
```
