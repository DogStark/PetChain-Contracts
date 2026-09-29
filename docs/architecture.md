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

## Pet Identifier Normalization

Mobile and frontend clients must derive the same canonical pet lookup key from
user-entered chip/microchip values. The canonicalization rules below are the
single source of truth and are published as Rust compatibility vectors in
`stellar-contracts/src/test_pet_identifier_normalization.rs` so contract tests
and generated client fixtures consume the same rules.

### Canonicalization rules

1. **Unicode normalization** — apply NFKC normalization first so compatibility
   characters (full-width digits, ligatures, etc.) fold to their canonical form.
2. **Case folding** — lowercase using Unicode simple case folding (e.g. `A` →
   `a`, `İ` → `i̇`).
3. **Whitespace** — trim leading/trailing whitespace and collapse any internal
   run of Unicode whitespace to a single ASCII space.
4. **Separators** — remove all separator characters (`-`, `_`, `.`, `/`, `\`,
   `:`, and Unicode dash/space separators) so `123-456` and `123 456` both
   canonicalize to `123456`.
5. **Result** — the canonical key is the concatenation of the remaining
   characters; an empty result is rejected as an invalid identifier.

### Compatibility vectors

| Input | Canonical key |
| --- | --- |
| `"  ABC-123  "` | `abc123` |
| `"abc_123"` | `abc123` |
| `"ABC.123"` | `abc123` |
| `"ＡＢＣ１２３"` (full-width) | `abc123` |
| `"abc\u{2009}123"` (thin space) | `abc123` |
| `"ABC\u{2013}123"` (en dash) | `abc123` |
| `""` | rejected |

### Migration path

Existing stored identifiers are read through the same canonicalization
function before lookup, so legacy values that differ only by case, whitespace,
or separators remain readable. New writes persist the canonical key; a
backfill migration can rewrite legacy rows to the canonical form without
changing their meaning.

## Soroban Deterministic Error Registry

Public contract errors are part of the client-facing API: clients branch on the
numeric discriminant returned by Soroban, so every public error must have a
stable code, a documented meaning, and explicit retryability semantics. The
registry below is the single source of truth for those discriminants and is
mirrored by the generated snapshot in
`stellar-contracts/src/test_error_registry.rs`.

### Registry generation

The registry is generated from source rather than hand-maintained. The
`Error` enum in `stellar-contracts/src/lib.rs` is the source of truth; each
variant carries an explicit `#[repr(u32)]` discriminant and a doc comment that
states its meaning and retryability. The generator walks the enum and emits a
snapshot (code, name, meaning, retryable) that is committed alongside the
contract so drift is visible in review.

### Registry

| Code | Error | Meaning | Retryable |
| --- | --- | --- | --- |
| 1 | `Unauthorized` | Caller is not authorized for this operation. | No — fix authorization first. |
| 2 | `NotFound` | Referenced pet, record, or resource does not exist. | No — the resource must be created first. |
| 3 | `InvalidInput` | Input failed validation (malformed or out of range). | No — correct the input before retrying. |
| 4 | `AlreadyExists` | Resource already exists and cannot be created twice. | No — treat as idempotent success or fetch the existing resource. |
| 5 | `Conflict` | Operation conflicts with current contract state. | Yes — re-read state and retry once it is consistent. |
| 6 | `InsufficientFunds` | Account lacks the balance required for the operation. | No — fund the account first. |
| 7 | `Expired` | Consent, grant, or record has passed its validity window. | No — a new grant must be issued. |
| 8 | `RateLimited` | Caller exceeded the allowed request rate. | Yes — retry after the documented backoff. |
| 9 | `Internal` | Unexpected internal failure. | Yes — retry with backoff; report if it persists. |

### Recovery semantics

- **Non-retryable errors** (`Unauthorized`, `NotFound`, `InvalidInput`,
  `AlreadyExists`, `InsufficientFunds`, `Expired`) require a client-side change
  (authorization, input, funding, or a fresh grant) before the call can succeed.
  Retrying the identical request is guaranteed to fail.
- **Retryable errors** (`Conflict`, `RateLimited`, `Internal`) may succeed on a
  later attempt. Clients should re-read contract state, apply the documented
  backoff, and retry with bounded attempts.
- Clients must branch on the numeric code, never on error text, so that
  localization and message changes do not break recovery logic.

### Discriminant stability

Existing codes must not change silently:

- Every public error has an explicit `#[repr(u32)]` discriminant; codes are
  never reassigned or reused, even after a variant is deprecated.
- The generated snapshot is committed and compared in CI. Adding, removing, or
  renumbering a variant changes the snapshot and fails the build until the
  change is reviewed and the snapshot is regenerated intentionally.
- Duplicate discriminants are rejected: the registry test asserts that every
  code is unique, so two variants cannot share a code.
- The intentional-change test documents the expected diff for a deliberate
  renumbering, ensuring such changes are explicit rather than accidental.

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
