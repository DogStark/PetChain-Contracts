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

## Handsoff notes

<!-- handsoff-issue-1188 -->
- #1188: [pet-transfer-adoption] Prevent adoption of nonexistent or inactive pets

<!-- handsoff-issue-1197 -->
- #1197: [stellar-contracts] Add emergency-override least-privilege scopes

<!-- handsoff-issue-1200 -->
- #1200: [stellar-contracts] Minimize public emergency-profile data

<!-- handsoff-issue-1212 -->
- #1212: [stellar-contracts] Add governance timelock and cancellation policy

<!-- handsoff-issue-1213 -->
- #1213: [stellar-contracts] Secure contract-upgrade authorization and hash binding

<!-- handsoff-issue-1227 -->
- #1227: [backend-2fa] Enforce TOTP replay protection atomically

<!-- handsoff-issue-1261 -->
- #1261: [backend-2fa] Add webhook signature key versioning

<!-- handsoff-issue-1320 -->
- #1320: [Contracts] Add Celo event indexing compatibility fixtures
