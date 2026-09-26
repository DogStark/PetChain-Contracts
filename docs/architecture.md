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

## Verification Status

As of this cleanup:

- `cd stellar-contracts && cargo test` passes
- `cd backend-2fa && cargo test` passes
