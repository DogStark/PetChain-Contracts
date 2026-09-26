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

## Custody History Pagination and Digest Proofs

Custody history is exposed as an ordered, append-only chain of entries. Because
consumers read it page by page, each page carries boundary digests that let a
consumer prove the page belongs to a single chain and that no entries were
skipped between pages.

### Page Boundary Digests

Every custody history page exposes two digests:

- `prev_digest` — the digest of the last entry on the immediately preceding
  page, or the chain's genesis digest for the first page.
- `next_digest` — the digest of the last entry on the current page, which the
  following page must echo as its `prev_digest`.

A page is only valid when `page.prev_digest` equals the `next_digest` of the
page before it. The first page anchors to the genesis digest, and the terminal
page is the one whose `next_digest` equals the chain head digest.

### Verification Rules

- **Consecutive pages verify against the expected chain.** For pages `P(n)` and
  `P(n+1)`, `P(n+1).prev_digest == P(n).next_digest` must hold, and both must
  belong to the same chain identifier.
- **Tampered pages fail.** If any entry is mutated, its recomputed digest no
  longer matches the boundary digest, so the page fails verification.
- **Out-of-order pages fail.** Presenting `P(n+1)` before `P(n)`, or skipping a
  page, breaks the `prev_digest`/`next_digest` linkage and fails verification.
- **Empty and terminal pages are unambiguous.** An empty page reports no
  entries with `prev_digest == next_digest`, so it cannot be mistaken for a page
  that advanced the chain. A terminal page is identified by `next_digest`
  matching the chain head, distinguishing it from a page that was truncated.

### Testing

Custody pagination is covered by generated custody history fixtures and proof
tests that assert: consecutive pages verify against the expected chain;
tampered and out-of-order pages fail; and empty and terminal pages are
unambiguous.

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
