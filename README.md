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

## Pagination Policy

All Stellar paginated read endpoints (pet, record, vet, consent, custody, and
activity reads) share a single bounded page-size policy. The policy is
centralized in the contracts crate so every endpoint validates page sizes the
same way and returns identical cursor semantics.

| Parameter | Value |
| --- | --- |
| Minimum page size | `1` |
| Default page size | `20` |
| Maximum page size | `100` |

Rules:

- A page size of `0` (or any value below the minimum) is rejected with a
  deterministic error.
- A page size above the maximum is rejected with a deterministic error.
- Omitting the page size uses the documented default of `20`.
- Every paginated endpoint returns the same cursor semantics: the cursor is an
  opaque token identifying the next page, and an empty/absent cursor starts at
  the first page.
- Existing callers that pass a page size within `[1, 100]` remain compatible.

## Documentation

- [Architecture](docs/architecture.md)
- [Development](docs/development.md)
- [API Overview](docs/api.md)
- [Error Codes](docs/error-codes.md)
- [Security Policy](SECURITY.md)
- [Contributing](CONTRIBUTING.md)

## License

MIT
