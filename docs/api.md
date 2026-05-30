# API Overview

## Backend 2FA — OpenAPI Specification

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

Backend 2FA endpoints return structured JSON error payloads whenever a request fails. The shared schema is:

```json
{
  "code": "BAD_REQUEST",
  "message": "A human-readable error message",
  "details": null
}
```

| Field | Type | Description |
|---|---|---|
| `code` | `String` | A machine-readable error code |
| `message` | `String` | A user-facing description of the failure |
| `details` | `Option` | Optional structured context for the error |

Common error codes:

- `BAD_REQUEST` — malformed request or invalid payload
- `UNAUTHORIZED` — authentication / login token invalid or missing
- `FORBIDDEN` — authorization failed for the current user
- `NOT_FOUND` — requested resource does not exist
- `CONFLICT` — request conflicts with current state
- `INVALID_TOKEN` — two-factor token invalid or expired
- `INTERNAL_SERVER_ERROR` — unexpected failure on the backend

All unhandled panics are also caught by middleware and translated into a `500 Internal Server Error` with an `ApiError` payload.

---

## Batch Read Operations

Batch read operations reduce the number of round trips required to fetch related data. These functions aggregate multiple data points into a single call while respecting access control.

### `get_pet_full_profile_batch`

Returns comprehensive pet information including profile, owner, active consents, and latest medical record.

**Signature:**
```rust
pub fn get_pet_full_profile_batch(
    env: Env,
    pet_id: u64,
    caller: Address,
) -> Option<PetFullProfileBatch>
```

**Returns:**
```rust
pub struct PetFullProfileBatch {
    pub profile: PetProfile,
    pub owner: Address,
    pub active_consents: Vec<Consent>,
    pub latest_medical_record: Option<MedicalRecord>,
}
```

**Access Control:**
- **Public pets**: Accessible to anyone
- **Restricted pets**: Requires at least Basic access grant
- **Private pets**: Only accessible to owner

**Use Cases:**
- Dashboard views showing complete pet information
- Profile pages requiring owner and consent data
- Applications needing pet data with medical history

**Example:**
```rust
let batch = client.get_pet_full_profile_batch(&pet_id, &caller);
if let Some(data) = batch {
    // Access all data in one call
    let profile = data.profile;
    let owner = data.owner;
    let consents = data.active_consents;
    let latest_record = data.latest_medical_record;
}
```

### `get_pet_health_summary`

Returns health-related information including latest vaccination, lab result, and active insurance policy.

**Signature:**
```rust
pub fn get_pet_health_summary(
    env: Env,
    pet_id: u64,
    caller: Address,
) -> Option<PetHealthSummary>
```

**Returns:**
```rust
pub struct PetHealthSummary {
    pub pet_id: u64,
    pub latest_vaccination: Option<Vaccination>,
    pub latest_lab_result: Option<LabResult>,
    pub active_insurance_policy: Option<InsurancePolicy>,
}
```

**Access Control:**
- **Public pets**: Accessible to anyone
- **Restricted pets**: Requires at least Basic access grant
- **Private pets**: Only accessible to owner

**Use Cases:**
- Health dashboard views
- Veterinary appointment preparation
- Insurance claim verification
- Quick health status checks

**Example:**
```rust
let summary = client.get_pet_health_summary(&pet_id, &caller);
if let Some(health) = summary {
    // Check vaccination status
    if let Some(vax) = health.latest_vaccination {
        // Display vaccination info
    }
    
    // Check lab results
    if let Some(lab) = health.latest_lab_result {
        // Display lab results
    }
    
    // Check insurance coverage
    if let Some(policy) = health.active_insurance_policy {
        // Display insurance info
    }
}
```

### Performance Benefits

**Without Batch Operations:**
```rust
// 5 separate contract calls
let profile = client.get_pet(&pet_id, &caller);
let owner = client.get_pet_owner(&pet_id);
let consents = client.get_active_consents(&pet_id);
let records = client.get_pet_medical_records(&pet_id, &0, &1);
let vaccinations = client.get_vaccination_history(&pet_id, &0, &1);
```

**With Batch Operations:**
```rust
// 1 contract call
let batch = client.get_pet_full_profile_batch(&pet_id, &caller);
```

**Benefits:**
- Reduced network latency (fewer round trips)
- Lower transaction costs
- Atomic data consistency (all data from same ledger state)
- Simplified client code

### Access Control Enforcement

Both batch operations enforce the same access control rules as individual read operations:

1. **Pet existence check**: Returns `None` if pet doesn't exist
2. **Privacy level check**: Enforces Public/Restricted/Private rules
3. **Access grant validation**: Checks for valid access grants on Restricted pets
4. **Owner verification**: Allows owner full access regardless of privacy level

If access is denied, the functions return `None` rather than panicking, allowing graceful handling in client applications.

### Data Freshness

Batch operations return the **most recent** data based on timestamps:
- **Latest medical record**: Highest `recorded_at` timestamp
- **Latest vaccination**: Highest `administered_at` timestamp
- **Latest lab result**: Highest `test_date` timestamp
- **Active insurance**: Most recent active policy (highest index)

### Error Handling

Batch operations return `Option<T>` rather than panicking:
- `Some(data)` - Access granted, data retrieved
- `None` - Pet doesn't exist OR access denied

This design allows clients to handle missing data and access denial uniformly.

