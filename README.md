# PetChain Contracts

This repository contains the Rust code for PetChain’s contract and auth-related backend work.

## What’s Here

- `stellar-contracts/`: the main Soroban contract crate plus the transfer/adoption contract package
- `backend-2fa/`: the Rust 2FA support crate
- `docs/`: the maintained project docs

## Quick Start

### Contracts

```bash
cd stellar-contracts
cargo test
```

### Backend 2FA

```bash
cd backend-2fa
cargo test
```

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)

## Status

The repo has been cleaned so both Rust crates currently test successfully.

## License

MIT
