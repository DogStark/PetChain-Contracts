# Vet Credential Issuers: Key Versions, Rotation and Revocation

Issue #1336. Types live in `src/credential_issuers.rs`, methods in `src/lib.rs`,
tests in `src/test_vet_credential_issuer_rotation.rs`.

## Model

An **issuer** (e.g. a licensing board) is an `Address` registered by an admin.
Each issuer has one or more numbered **key versions**, each recording a
32-byte public key, an activation time and a hard expiry. The newest version
is the issuer's `current_version`.

A **credential** is minted on-chain by the issuer for a registered vet under
a specific key version. It records the issuer, key version, vet, issue time
and expiry. Off-chain verifiers resolve the key through
`get_issuer_key_version` and check status with `verify_vet_credential`.

## Operations

| Operation | Who | Effect |
|---|---|---|
| `register_credential_issuer` | admin | Creates the issuer with key version 1. |
| `rotate_credential_issuer_key` | issuer or admin | Adds version `n+1` and makes it current. The previous version gets a mint overlap window. |
| `issue_vet_credential` | issuer | Mints a credential under a mintable key version. |
| `revoke_credential_issuer_key` | admin | Revokes one key version (key compromise). |
| `revoke_credential_issuer` | admin | Revokes the issuer entirely. |
| `verify_vet_credential` | anyone | Returns `Valid`, `Expired`, `KeyVersionRevoked`, `IssuerRevoked` or `NotFound`. |

## Minting rules

A key version can mint only while all of these hold:

- the issuer is not revoked;
- the version is not revoked;
- `now < key.expires_at`;
- if the version was rotated out, `now < mint_until`.

The credential's own expiry must satisfy `now < expires_at <= key.expires_at`,
so a credential never outlives the key that minted it.

## Rotation and overlap

Rotation is routine and **not retroactive**. When version `n` is rotated to
`n+1`, version `n` may keep minting until
`mint_until = min(now + overlap_secs, key_n.expires_at)`. This lets issuers
finish in-flight signing with the old key. `overlap_secs` is capped at
`MAX_ISSUER_ROTATION_OVERLAP_SECS` (7 days), and `0` retires the old key
immediately. A version that is already revoked gets no overlap.

Credentials minted under an older version **stay `Valid` until their own
expiry**, as long as neither that version nor the issuer is revoked.

A public key can be used by only one version of an issuer. Rotating back to
an earlier key fails with `IssuerKeyReused`.

## Revocation

Revocation is for compromise and **is retroactive**:

- **Key version revocation** makes every credential minted under that version
  report `KeyVersionRevoked`. It also blocks further minting with it,
  including during an overlap window. Credentials under other versions are
  unaffected. If the revoked version was current, the issuer recovers by
  rotating to a new key.
- **Issuer revocation** makes every credential of the issuer report
  `IssuerRevoked`, and blocks minting and rotation permanently.

Both revocations are idempotent: repeating one keeps the original
`revoked_at`.

## Historical verification

`verify_vet_credential` reports status **as of the current ledger time**.
Precedence is `IssuerRevoked` > `KeyVersionRevoked` > `Expired` > `Valid`.
Because revocation is retroactive, a credential that verified yesterday may
not verify today. Consumers that need a point-in-time record should keep the
`issued_at` and `revoked_at` timestamps, which are exposed through
`get_vet_credential`, `get_issuer_key_version` and `get_credential_issuer`.

## Relationship to vet verification

This registry is separate from the admin-managed vet verification flag
(`verify_vet`, `verify_vet_with_expiry`, `is_verified_vet`). Minting requires
only that the vet is registered, and issuing a credential does not change
`is_verified_vet`.
