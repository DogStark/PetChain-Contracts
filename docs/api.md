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
| `get_custody_history_page` | Returns a single page of custody history with boundary digests (see below) |
| `get_access_logs` | Returns access logs for a pet (owner/admin only) |

> **Audit note:** All `log_access` (storage write) calls were removed from the above functions. Write functions (`add_medical_record`, `update_pet_profile`, `grant_access`, `revoke_access`, `add_attachment`, etc.) retain their access log writes.

---

## Custody History Pagination Proofs (Issue #1339)

Custody history consumers must be able to verify that a page belongs to a
single chain and that no entries were skipped between pages. To make this
possible, every custody history page exposes **boundary digests** that bind the
page to its position in the chain.

### Page shape

`get_custody_history_page(pet_id, cursor, limit)` returns a page with the
following fields:

| Field | Description |
|---|---|
| `entries` | The custody entries in this page, in chain order |
| `prev_digest` | Digest of the entry immediately preceding this page (`None` for the first page) |
| `next_digest` | Digest of the entry immediately following this page (`None` for the terminal page) |
| `page_digest` | Canonical digest over `(domain, version, pet_id, start_seq, end_seq, entries)` |
| `start_seq` / `end_seq` | Inclusive sequence range covered by this page |
| `is_terminal` | `true` iff this page is the last page of the chain |

### Verification rules

A consumer verifies a page against the expected chain as follows:

1. **Chain membership.** Recompute `page_digest` from the returned entries and
   compare it to the returned `page_digest`. A mismatch means the page was
   tampered with.
2. **Linkage.** For consecutive pages `P` and `Q`, require
   `P.next_digest == Q.prev_digest` and `Q.start_seq == P.end_seq + 1`. This
   proves no entries were skipped and that the pages belong to the same chain.
3. **Ordering.** `start_seq` must be strictly greater than the previous page's
   `end_seq`; out-of-order pages fail the linkage check.
4. **Empty pages.** An empty page has `entries == []`, `start_seq == end_seq`,
   and `page_digest` equal to the canonical digest of an empty range. An empty
   page is only valid when it is also terminal.
5. **Terminal pages.** The terminal page has `next_digest == None` and
   `is_terminal == true`. A non-terminal page with `next_digest == None` is
   invalid, and a terminal page with a non-`None` `next_digest` is invalid.

### Proof test plan

Generated custody history fixtures and proof tests cover:

- **Consecutive pages verify.** For a generated chain, every adjacent page pair
  satisfies the linkage rule and each `page_digest` recomputes correctly.
- **Tampered pages fail.** Mutating an entry, `start_seq`, or `page_digest`
  causes verification to fail.
- **Out-of-order pages fail.** Swapping two pages or skipping a page breaks the
  linkage rule.
- **Empty and terminal pages are unambiguous.** An empty page is accepted only
  when terminal; a terminal page is accepted only when `next_digest == None`.

---

## Batch-Operation Atomicity Policy (Issue #1334)

Batch write operations (e.g. `get_pet_full_profile_batch` and any future
multi-item write entrypoints) follow a single, documented atomicity model:

- **All-or-nothing.** A batch is committed only if *every* item succeeds. If
  any item fails validation or authorization, the entire batch is rolled back
  and no storage mutation from that batch is persisted. There is no partial
  commit path.
- **No observable partial state.** Because a failed batch reverts the whole
  transaction, no partial ownership, custody, or consent state is ever
  observable — neither to the caller nor to subsequent reads. A failed batch
  leaves ownership and consent exactly as they were before the call.
- **Per-item results are safe-only.** Per-item results are exposed only on
  full success, or via non-mutating preview/read helpers. A failing batch
  returns a single error and never a mix of applied and rejected items.
- **Bounded item limits.** Every batch entrypoint enforces a documented
  maximum item count (`MAX_BATCH_ITEMS`). Requests exceeding the limit are
  rejected before any item is processed, so a batch can never be used to
  bypass per-transaction resource limits.

### Batch test plan

Batch success, failure, and limit behavior is covered by tests that assert
state snapshots before and after each call:

- **Success:** a valid batch applies all items and the post-state snapshot
  reflects every mutation.
- **Failure:** a batch containing one invalid item reverts entirely; the
  post-state snapshot is byte-for-byte identical to the pre-state snapshot
  (no partial ownership or consent state).
- **Limit:** a batch exceeding `MAX_BATCH_ITEMS` is rejected and the state
  snapshot is unchanged.

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

Cursor pagination is bounded by policy: `get_pet_medical_records_cursor`
accepts an opaque cursor and a page size that is clamped to a maximum, so a
single request cannot scan an unbounded number of records. Callers should
follow the returned cursor until it is exhausted rather than requesting
arbitrarily large pages.

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
