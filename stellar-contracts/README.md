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

## Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/petchain_stellar.wasm \
  --network testnet
```
