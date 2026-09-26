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

## Batch Operation Atomicity Policy

Soroban batch operations in `stellar-contracts` follow a single, documented
atomicity model: **all-or-nothing**. A batch either applies every item or
applies none of them. There is no partial-success mode for state-mutating
batches.

### Rationale

Batch writes are cheaper than individual calls, but partial success is
dangerous for medical and ownership records. A half-applied batch could leave
an owner without a corresponding consent record, or a medical entry without its
attachment, producing state that is observable but inconsistent. To prevent
this, batches are treated as a single logical transaction.

### Execution Model

1. **Validate first.** Every item in the batch is validated (authorization,
existence, bounds, and domain invariants) before any state is written. If any
item fails validation, the whole batch is rejected and no state changes.
2. **Apply atomically.** State mutations are staged and committed together. If
any item fails during application, the entire batch is rolled back and the
contract returns an error. No partial ownership or consent state is left
observable.
3. **Report per-item results only when safe.** Per-item results are exposed
only on full success, or through a non-mutating preview/validation entry point
that performs no writes. A failed batch returns a single error and does not
return per-item success flags that could be mistaken for applied state.

### Item Limits

Batch size is bounded by a documented maximum (`MAX_BATCH_SIZE`). Batches that
exceed the limit are rejected before validation, keeping execution cost and
state-snapshot size predictable. The limit is enforced consistently across all
batch entry points.

### Testing

Batch behavior is covered by success, failure, and limit tests that assert
state snapshots before and after each call, verifying that failed batches leave
state unchanged and that over-limit batches are rejected.

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
