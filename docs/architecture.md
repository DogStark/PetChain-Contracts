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

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
