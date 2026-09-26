ckend 2FA — OpenAPI Specification

The Backend 2FA service is fully documented as an **OpenAPI 3.0** spec.

- **Machine-readable spec:** [`docs/openapi.yaml`](./openapi.yaml)
- **Validation:** The spec is validated automatically on every PR via the
  `backend-2fa.yml` CI workflow using `@stoplight/spectral-cli`.

### Endpoints at a glance

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/2fa/enable` | Enable 2FA — returns secret and backup codes |
| `POST` | `/2fa/disable` | Disable 2FA (requires current TOTP token) |
| `POST` | `/2fa/verify` | Verify a TOTP token |
| `POST` | `/2fa/login` | Complete login with 2FA |
| `POST` | `/2fa/recover` | Recover access with a backup code |
| `GET`  | `/2fa/recovery-log` | Paginated backup-code usage log |
| `GET`  | `/2fa/audit-log/{user_id}` | Paginated 2FA audit log for a user |
| `POST` | `/admin/quota` | Set per-user storage quota (admin) |
| `POST` | `/admin/quota/unlimited` | Grant unlimited quota (admin) |
| `POST` | `/admin/canary` | Create a canary user (admin) |
| `GET`  | `/admin/flagged` | List all flagged submissions (admin) |
| `GET`  | `/admin/flagged/{user_id}` | Flagged submissions for a user (admin) |
| `GET`  | `/admin/users/{user_id}/2fa-summary` | 2FA summary for a user (admin) |
| `POST` | `/tenant/provision` | Provision a new tenant (admin) |
| `GET`  | `/ws/leaderboard` | WebSocket leaderboard feed |
| `GET`  | `/health` | Health check |

### Authentication
All write and sensitive read endpoints require a Bearer JWT (`Authorization: Bearer <token>`).

### Error format
```json
{ "error": "INVALID_TOKEN", "message": "The provided TOTP token has expired" }
```

---

## Pagination policy (Issue #1306)

All paginated Stellar contract reads share a single, centralized page-size
policy. The policy is enforced by one shared validator so every endpoint
behaves identically.

| Bound | Value | Meaning |
|---|---|---|
| Minimum | `1` | Smallest accepted page size |
| Default | `20` | Used when the caller omits the page size |
| Maximum | `100` | Largest accepted page size (Soroban budget bound) |

Rules:

- A page size of `0` (or any value below the minimum) returns a deterministic
  `InvalidPageSize` error — it is never silently coerced.
- A page size above the maximum returns the same deterministic
  `InvalidPageSize` error.
- Omitting the page size uses the documented default of `20`, which is stable
  across releases.
- All paginated endpoints return the same cursor semantics: an opaque cursor
  is returned alongside the page and passed back unchanged to fetch the next
  page. A `null`/absent cursor starts from the beginning.
- Existing valid callers (page sizes within `[1, 100]`) remain fully
  compatible.

This policy applies to the pet, record, vet, consent, custody, and activity
paginated reads.

---

## View Functions (pure reads — no storage writes or event emissions)

The following functions are guaranteed to have no side effects. They do not write to storage, emit events, or update access timestamps.

| Function | Description |
|---|---|
| `get_pet` | Returns decrypted pet profile; enforces privacy level |
| `get_pet_data` | Returns minimal pet data (name, species, breed) |
| `get_pet_age` | Computes age from birthday timestamp |
| `get_pet_full_profile` | Returns full profile with vaccination/medication summary |
| `get_pet_full_profile_batch` | **[Batch]** Returns pet profile, owner, active consents, and latest medical record in one call |
| `get_pet_health_summary` | **[Batch]** Returns latest vaccination, lab result, and active insurance in one call |
| `is_pet_active` | Returns whether a pet is active |
| `get_pet_owner` | Returns the owner address for a pet |
| `get_pet_photos` | Returns all photo hashes for a pet |
| `get_pet_photo_count` | Returns photo count |
| `get_pet_photos_paginated` | Returns paginated photo hashes |
| `get_total_pets` | Returns global pet count |
| `get_species_count` | Returns pet count for a species |
| `get_active_pets_count` | Returns count of active pets |
| `get_vet_stats` | Returns vet treatment/vaccination statistics |
| `get_vet_treatment_history` | Returns paginated treatment history for a vet |
| `get_vet_vaccination_history` | Returns paginated vaccination history for a vet |
| `get_pets_overdue_vaccinations` | Returns pet IDs with overdue vaccinations |
| `get_admins` | Returns list of admin addresses |
| `get_admin_threshold` | Returns multisig approval threshold |
| `get_verified_vets` | Returns paginated list of verified vets |
| `is_vet_registered` | Returns whether a vet address is registered |
| `is_verified_vet` | Returns whether a vet is verified |
| `get_vet` | Returns vet record by address |
| `get_vet_by_license` | Returns vet record by license number |
| `get_vaccinations` | Returns a vaccination record by ID |
| `get_vaccination_history` | Returns paginated vaccination history for a pet |
| `get_upcoming_vaccinations` | Returns upcoming vaccinations for a pet |
| `is_vaccination_current` | Returns whether a vaccine type is current |
| `get_medical_record` | Returns a medical record by ID |
| `get_pet_medical_records` | Returns paginated medical records for a pet |
| `get_pet_medical_records_cursor` | Cursor-based paged medical records; stable under concurrent inserts/deletes (Issue #1173) |
| `search_medical_records` | Returns filtered medical records |
| `get_attachments` | Returns attachments for a record (Malicious hidden from non-admins) |
| `get_attachment_by_index` | Returns a single attachment by index |
| `get_attachment_count` | Returns attachment count for a record |
| `verify_attachment` | Verifies content hash of an attachment |
| `is_owner_registered` | Returns whether an owner address is registered |
| `get_pet_count_by_owner` | Returns pet count for an owner |
| `get_pets_by_owner` | Returns paginated pets for an owner |
| `get_pets_by_species` | Returns paginated pets by species |
| `is_custody_valid` | Returns whether temporary custody is active |
| `get_custody_history` | Returns custody history for a pet |
| `get_custody_chain` | Returns the chain-of-custody log for a pet (chronological, append-only, capped at 100 entries) |
| `verify_custody_chain` | Checks chain-of-custody internal consistency (links, creator, current owner) |
| `get_custody_chain_digest` | Returns the canonical SHA-256 digest of the custody chain (domain, version, pet ID, sequence, entries in order) for completeness/ordering proofs |
| `get_access_logs` | Returns access logs for a pet (owner/admin only) |

> **Audit note:** All `log_access` (storage write) calls were removed from the above functions. Write functions (`add_medical_record`, `update_pet_profile`, `grant_access`, `revoke_access`, `add_attachment`, etc.) retain their access log writes.

---

## Smart Contracts

### Main contract

The primary contract lives in `stellar-contracts/src/lib.rs` and exposes functionality around:

- pet registration and profile management
- ownership and access grants
- vet registration and verification
- medical records, lab results, and attachments
- insurance, grooming, nutrition, activity, and behavior records
- emergency contacts and emergency access logs
- multisig administration and upgrade proposals

**Medical-record soft-delete & pagination (Issues #1170–#1173):**
Medical-record reads are delegated through a shared soft-delete filter so a
soft-deleted record never resurfaces in `get_medical_record`,
`get_pet_medical_records`, `get_pet_medical_records_cursor`,
`search_medical_records`, `search_by_keyword`, or
`get_pet_full_profile_batch`. Deletion preserves provenance (only the pet
owner, the record's vet, or an admin may delete) and publishes a
`MedicalRecordDeleted` audit event. Purging is split into a bounded,
resumable `purge_deleted_records_bounded` (Issue #1172) so large pets can be
drained without hitting transaction resource limits.

**Compatibility / migration notes:**
- `set_max_subscriptions_per_address` was renamed to `set_max_subscriptions`
  because the previous name (33 chars) exceeded Soroban's 32-char contract
  function-name limit, which prevented the contract from compiling. Callers
  must target the new name.
- The `ProposalNotFound` contract error discriminant moved from `39` to `42`
  to resolve a collision with `InvalidNonce`; `ProposalAlreadyExecuted`
  remains `38`. Error-code consumers should rely on the symbol, not the raw
  discriminant.

### Transfer and adoption contract

The transfer-focused contract lives in `stellar-contracts/contracts/pet-transfer-adoption/src/lib.rs` and handles:

- pet creation
- transfer initiation and acceptance
- transfer cancellation and reclaim flows
- ownership history tracking

## Backend 2FA

The backend crate provides:

- 2FA enrollment
- token verification and activation
- login-time token checks
- disable and recovery flows
- request tracing middleware
- in-memory and Redis-backed rate limiting
- standardized JSON error responses via `ApiError`

For implementation details, read the crate sources in `backend-2fa/src/`.

### Error response format

Backend 2FA endpoints return structured J

/* … truncated 5440 chars — edit only what you need near the top … */
