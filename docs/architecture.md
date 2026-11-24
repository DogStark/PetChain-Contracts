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

## Canonical Hash Domain Separation

Commitments for medical records, attachments, certificates, and custody digests are
computed over a canonical, domain-separated preimage. Domain separation guarantees
that identical field values hashed under different record types never collide, so a
valid commitment for one record type cannot be reinterpreted as a commitment for
another.

### Versioned Domain Tags

Each domain has an explicit, versioned tag. The tag is the first bytes of every
preimage and is the only thing that differs between domains for otherwise identical
field values.

| Domain | Tag (ASCII) | Version |
| --- | --- | --- |
| Medical record | `PC-MED-REC` | `v1` |
| Attachment | `PC-MED-ATT` | `v1` |
| Certificate | `PC-MED-CERT` | `v1` |
| Custody digest | `PC-CUSTODY` | `v1` |

Version changes are explicit: bumping a domain to `v2` changes the tag bytes (for
example `PC-MED-REC` → `PC-MED-REC2`) and therefore every commitment in that domain.
Old and new versions are never interchangeable.

### Byte-Level Preimage Format

The canonical preimage is the concatenation of the following, in order, with no
separators and no padding:

```text
preimage = domain_tag || 0x00 || version || 0x00 || field_1 || 0x00 || ... || field_n
```

- `domain_tag`: the ASCII tag from the table above.
- `0x00`: a single NUL byte separating the tag from the version.
- `version`: the ASCII version string (for example `v1`).
- `0x00`: a single NUL byte separating the version from the fields.
- `field_i`: the canonical encoding of each field, in the order defined by the
  record type. Integers are big-endian, fixed-width; byte strings are raw; text is
  UTF-8. Fields are separated by a single `0x00` byte.

The commitment is `SHA-256(preimage)`. Because the domain tag and version are part
of the preimage, identical field values in different domains produce different
hashes.

### Determinism

Encoding is deterministic across supported clients: the field order, integer
width, and separators are fixed by this document, so Rust and generated clients
produce byte-identical preimages for the same logical record.

### Test Vectors

Fixed test vectors are published alongside the contract tests and are consumed by
both the Rust tests and the generated client tests. Cross-domain collision
regression tests assert that the same field values under different domain tags
yield different commitments.

### Errors

Verification failures return the documented error for the domain being verified
(see `docs/error-codes.md`); a commitment that does not match the expected domain
or version is rejected with that documented error rather than being accepted.

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
stable code, a documented meaning, and explicit retryability semantics.

The source of truth is [`stellar-contracts/error-codes.json`](../stellar-contracts/error-codes.json),
a hand-maintained fixture with one entry per `#[contracterror]` enum (for
example `ContractError` in the petchain contract). Each entry lists every
error's `code`, `name`, and `retry` policy, plus `retired` codes that stay
reserved forever. Do not copy the table into this document; the fixture and the
policy in [`stellar-contracts/docs/error-compatibility.md`](../stellar-contracts/docs/error-compatibility.md)
are authoritative. For the per-module error reference, see
[`docs/error-codes.md`](error-codes.md).

### Retry policies

| `retry` | Client behaviour |
| --- | --- |
| `never` | Permanent for this request: fix the input, caller, or target. |
| `after_state_change` | A precondition is not met yet. Re-read state and retry once it changes. |
| `backoff` | Transient throttling. Retry with exponential backoff. |
| `treat_as_success` | Idempotent replay: the effect already exists. Do not retry. |

### Excerpt

| Code | Name | `retry` |
| --- | --- | --- |
| 1 | `AdminAlreadyApproved` | `treat_as_success` |
| 5 | `BatchTooLarge` | `never` |
| 8 | `InputStringTooLong` | `never` |
| 9 | `InvalidBreed` | `never` |
| 28 | `Unauthorized` | `never` |
| 46 | `RateLimitExceeded` | `backoff` |

### Stability rules

- Codes are append-only: never renumber a variant or give a code a new meaning.
- Never delete a published code. Retire it by moving its entry from `errors`
  to `retired` in `error-codes.json`.
- CI runs `stellar-contracts/scripts/check_error_compat.py` against the base
  branch's fixture and fails on removed, reused, or renumbered codes, and on
  new errors without a `CHANGELOG.md` entry.

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
