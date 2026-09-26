# PetChain Contracts

Rust code for PetChain's on-chain contracts and backend authentication work.

## Repository Layout

```text
PetChain-Contracts/
├── stellar-contracts/       # Soroban smart contract crate
│   ├── src/lib.rs           # Main contract
│   ├── src/test_*.rs        # Test modules
│   └── contracts/
│       └── pet-transfer-adoption/
├── backend-2fa/             # TOTP 2FA support crate
│   ├── src/
│   ├── migrations/
│   ├── schema.sql
│   ├── README.md
│   └── examples/
│       └── example_integration.rs
└── docs/
    ├── architecture.md
    ├── development.md
    ├── api.md
    ├── openapi.yaml
    └── error-codes.md
```

## Quick Start

### Stellar contracts

```bash
cd stellar-contracts
cargo test
```

### Backend 2FA

```bash
cd backend-2fa
cargo test
```

## Soroban Error Registry

Public contract errors are documented in a deterministic registry so clients can
map a numeric discriminant to its meaning and recovery semantics without parsing
error text. The registry is generated from the contract source (the `#[contracterror]`
enums) rather than hand-maintained, so it stays in sync with the contracts.

Each entry records:

- **code** — the stable numeric discriminant returned on-chain.
- **meaning** — a human-readable description of the failure.
- **retryability** — whether a client may safely retry, and under what conditions.

See [Error Codes](docs/error-codes.md) for the full registry and recovery
guidance. Discriminants are treated as a stable public interface: existing codes
must not change silently, duplicate codes fail CI, and any intentional change
requires updating the generated snapshot and the accompanying intentional-change
test.

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT
