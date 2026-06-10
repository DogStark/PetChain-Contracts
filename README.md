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

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT
