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

## Batch Operation Atomicity Policy

Soroban batch operations in this repository follow a single, documented
atomicity model: **all-or-nothing**. A batch either applies every item or
applies none of them. There is no partial-success mode.

- **Atomicity model.** Each batch is executed as one atomic unit. If any item
  fails validation or execution, the entire batch is rolled back and no state
  change from that batch is committed. Ownership, consent, and custody records
  therefore never expose a partially applied batch.
- **Rollback / error behavior.** On the first failing item the batch aborts and
  returns an error identifying the failing item index and reason. Because the
  whole batch is rolled back, no partial ownership or consent state is
  observable after a failed batch.
- **Per-item results.** Per-item results are only exposed when they are safe:
  on full success (each item's committed result) or as a non-mutating preview
  that performs no writes. Failed batches do not return per-item success
  results, since none of the items were committed.
- **Item limits.** Batch size is bounded by a documented maximum. Batches that
  exceed the limit are rejected before any state is touched, so oversized
  batches cannot partially apply.

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

<!-- handsoff-issue-1340 -->
- #1340: [Contracts] Add Soroban insurance reserve accounting audit view
