# Architecture

## Repository Layout

```text
PetChain-Contracts/
├── README.md
├── SECURITY.md
├── CONTRIBUTING.md
├── CHANGELOG.md
├── docs/
│   ├── architecture.md
│   ├── development.md
│   ├── api.md
│   ├── openapi.yaml
│   └── error-codes.md
├── stellar-contracts/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── test_*.rs
│   └── contracts/
│       └── pet-transfer-adoption/
└── backend-2fa/
    ├── Cargo.toml
    ├── src/
    ├── migrations/
    ├── schema.sql
    ├── README.md
    └── examples/
        └── example_integration.rs
```

## Components

### `stellar-contracts`

The main Soroban contract crate. It contains the primary PetChain smart contract, including:

- pet registration and ownership
- veterinary access control
- medical records, vaccinations, and attachments
- emergency data and consent flows
- activity, grooming, and insurance features
- multisig admin and upgrade flows

The nested `contracts/pet-transfer-adoption` package is a smaller ownership-transfer contract with its own tests.

### `backend-2fa`

A Rust support crate for TOTP-based 2FA:

- enrollment and verification handlers
- in-memory and Postgres-backed storage
- request tracing middleware
- in-memory and Redis-backed rate limiting

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
