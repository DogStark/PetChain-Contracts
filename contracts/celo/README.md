# Celo Contracts

Soroban/Celo contract modules for the PetChain platform.

## Modules

- `pet_medical_history` — stores pet medical records with cursor-based pagination.

## Pet Medical History Pagination

Medical records are returned using **cursor-based pagination** with deterministic
ordering and bounded page sizes.

### Ordering

Records are ordered deterministically by `(created_at, record_id)` ascending.
This guarantees a stable, reproducible order across pages and nodes.

### Page size

- `DEFAULT_PAGE_SIZE = 20`
- `MAX_PAGE_SIZE = 100`

Requests with `limit == 0` fall back to `DEFAULT_PAGE_SIZE`; requests above
`MAX_PAGE_SIZE` are clamped to `MAX_PAGE_SIZE`. This bounds iteration and
persistent state growth per call.

### Cursor

A cursor is an opaque `u64` encoding the last returned `record_id`. Passing
`None` starts from the beginning. The cursor is validated against the pet's
record set; an unknown cursor returns `Error::InvalidCursor`.

### Authorization

All read calls require the caller to be the pet owner or an authorized
veterinarian, preserving the existing authorization conventions. Unauthorized
callers receive `Error::Unauthorized` and no state is mutated.

### Error codes

| Code | Name             | Meaning                                   |
|------|------------------|-------------------------------------------|
| 1    | `Unauthorized`   | Caller lacks access to the pet's records  |
| 2    | `InvalidCursor`  | Cursor does not belong to the pet's set   |
| 3    | `InvalidLimit`   | Limit out of accepted bounds              |

### Events

- `MedicalHistoryPage { pet_id, cursor, limit, returned }` — emitted on each
  successful paginated read.

### Storage version

The storage layout is unchanged; the existing storage-version key is preserved
and no migration is required for this feature.

### Compatibility

Public ABI adds the paginated read entry point and the `MedicalHistoryPage`
event. Update compatibility fixtures when the ABI snapshot changes.

## Emergency Pause Role Separation

Pause authority is separated from upgrade and registry administration so that
no single role can both halt the contract and mutate its code or registry.

### Roles

| Role          | Can pause/unpause | Can upgrade | Can edit registry |
|---------------|-------------------|-------------|-------------------|
| `Pauser`      | yes               | no          | no                |
| `Upgrader`    | no                | yes         | no                |
| `RegistryAdmin` | no              | no          | yes               |

Each role is stored under its own key and checked independently. A caller
holding one role cannot exercise another; attempts return `Error::Unauthorized`
and leave state unchanged.

### Pause / unpause

- `pause(caller)` — requires `Pauser`; sets the paused flag and emits
  `Paused { caller }`. Re-pausing an already-paused contract is a no-op that
  returns `Error::AlreadyPaused` without mutating state.
- `unpause(caller)` — requires `Pauser`; clears the paused flag and emits
  `Unpaused { caller }`. Unpausing a non-paused contract returns
  `Error::NotPaused` without mutating state.

While paused, state-mutating entry points (including upgrade and registry edits)
return `Error::Paused` and perform no writes. Read-only calls remain available.

### Recovery path

Unpausing requires the `Pauser` role. If the `Pauser` key is lost, recovery is
performed by the `Upgrader` via `rotate_pauser(new_pauser)`, which is the only
operation the `Upgrader` may perform while paused. This keeps a documented,
bounded recovery path without granting the `Upgrader` direct pause authority.
`rotate_pauser` emits `PauserRotated { old, new }` and rejects the zero address
with `Error::InvalidAddress`.

### Error codes (pause)

| Code | Name            | Meaning                                    |
|------|-----------------|--------------------------------------------|
| 4    | `Paused`        | Contract is paused; mutation rejected      |
| 5    | `AlreadyPaused` | Pause requested while already paused       |
| 6    | `NotPaused`     | Unpause requested while not paused         |
| 7    | `InvalidAddress`| Zero/invalid address supplied              |

### Events (pause)

- `Paused { caller }`
- `Unpaused { caller }`
- `PauserRotated { old, new }`

### Storage version

Role keys are added under the existing storage-version key; the version is
bumped and a migration initializes `Pauser`, `Upgrader`, and `RegistryAdmin`
from the prior admin key. No other layout changes are required.

### Compatibility

Public ABI adds `pause`, `unpause`, and `rotate_pauser` entry points plus the
`Paused`, `Unpaused`, and `PauserRotated` events. Update compatibility fixtures
when the ABI snapshot changes.

## Two-Step Ownership Transfer

Ownership changes require explicit recipient acceptance. The current owner
proposes a recipient; ownership only moves once that recipient accepts. This
prevents transfers to wrong or unreachable addresses.

### Flow

- `propose_owner(caller, new_owner)` — requires `caller == current owner`;
  stores a pending transfer `{ new_owner, expires_at }` and emits
  `OwnershipProposed { current, new_owner, expires_at }`. Proposing while a
  pending transfer exists overwrites it (bounded to a single pending slot).
- `accept_owner(caller)` — requires `caller == pending.new_owner` and
  `ledger_time <= pending.expires_at`. On success, the prior owner is appended
  to the owner history, ownership moves to `caller`, the pending slot is
  cleared, and `OwnershipTransferred { previous, new_owner }` is emitted.
- `cancel_owner_transfer(caller)` — requires `caller == current owner`; clears
  the pending slot and emits `OwnershipTransferCancelled { current }`.
- `owner_history()` — read-only view returning the bounded, append-only list of
  prior owners (oldest first).

### Expiry

Pending transfers carry an `expires_at` ledger timestamp. Proposals with a
zero or past deadline are rejected with `Error::InvalidDeadline`. Accepting an
expired proposal returns `Error::TransferExpired` and leaves state unchanged;
re-proposing replaces the expired slot. This bounds how long a stale proposal
can be replayed.

### Bounded history

Owner history is append-only and capped at `MAX_OWNER_HISTORY = 32` entries.
When the cap is reached the oldest entry is dropped, keeping persistent state
growth bounded while retaining a verifiable recent chain of prior owners.

### Authorization

Only the current owner may propose or cancel; only the nominated recipient may
accept. Unauthorized callers receive `Error::Unauthorized` and no state is
mutated. Replay of an already-accepted or cancelled proposal fails because the
pending slot is cleared on success.

### Error codes (ownership)

| Code | Name               | Meaning                                        |
|------|--------------------|------------------------------------------------|
| 8    | `NoPendingTransfer`| No pending ownership transfer to act on        |
| 9    | `TransferExpired`  | Pending transfer deadline has passed           |
| 10   | `InvalidDeadline`  | Proposed deadline is zero or in the past       |

### Events (ownership)

- `OwnershipProposed { current, new_owner, expires_at }`
- `OwnershipTransferred { previous, new_owner }`
- `OwnershipTransferCancelled { current }`

### Storage version

Ownership adds a pending-transfer key and an owner-history key under the
existing storage-version key; the version is bumped and a migration seeds the
history with the current owner. No other layout changes are required.

### Compatibility

Public ABI adds `propose_owner`, `accept_owner`, `cancel_owner_transfer`, and
`owner_history` entry points plus the `OwnershipProposed`,
`OwnershipTransferred`, and `OwnershipTransferCancelled` events. Update
compatibility fixtures when the ABI snapshot changes.

## Celo Cross-Chain Pet Identifier Mapping

An explicit mapping from Celo pet identifiers to canonical PetChain identifiers
lets off-chain Celo records resolve to a single canonical pet. Both directions
are stored so uniqueness can be enforced in either direction.

### Mapping rules

- `set_pet_mapping(caller, celo_pet_id, canonical_pet_id)` — requires the
  `RegistryAdmin` role (preserving existing authorization conventions). Stores
  `celo_pet_id -> canonical_pet_id` and `canonical_pet_id -> celo_pet_id` and
  emits `PetMappingSet { celo_pet_id, canonical_pet_id }`.
- A `celo_pet_id` maps to at most one canonical id. Re-mapping a `celo_pet_id`
  to a *different* canonical id returns `Error::MappingConflict` and leaves
  state unchanged; re-mapping to the *same* canonical id is an idempotent no-op.
- A `canonical_pet_id` cannot be double-mapped to a different `celo_pet_id`;
  this also returns `Error::MappingConflict` without mutating state.
- `revoke_pet_mapping(caller, celo_pet_id)` — requires `RegistryAdmin`; clears
  both directions of the mapping and emits `PetMappingRevoked { celo_pet_id,
  canonical_pet_id }`. Revoking a missing mapping returns
  `Error::MappingNotFound` and leaves state unchanged (explicit error, not a
  silent no-op).
- `canonical_for(celo_pet_id)` / `celo_for(canonical_pet_id)` — read-only
  lookups returning `Option<u64>`; missing entries return `None`.

### Bounds

Identifiers are `u64`; the zero id is rejected with `Error::InvalidPetId` to
keep the mapping space well-defined. Each mapping occupies exactly two storage
slots, so persistent state growth is bounded by the number of live mappings and
fully reclaimed on revocation. No iteration over the mapping set is performed.

### Authorization

Only `RegistryAdmin` may set or revoke mappings. Unauthorized callers receive
`Error::Unauthorized` and no state is mutated. While the contract is paused,
set/revoke return `Error::Paused` and perform no writes; reads remain available.

### Error codes (mapping)

| Code | Name              | Meaning                                          |
|------|-------------------|--------------------------------------------------|
| 11   | `MappingConflict` | Id already mapped to a different counterpart     |
| 12   | `MappingNotFound` | No mapping exists for the supplied id            |
| 13   | `InvalidPetId`    | Zero/invalid pet identifier supplied             |

### Events (mapping)

- `PetMappingSet { celo_pet_id, canonical_pet_id }`
- `PetMappingRevoked { celo_pet_id, canonical_pet_id }`

### Storage version

Mapping keys are added under the existing storage-version key; the version is
bumped and a migration initializes the (empty) mapping tables. No other layout
changes are required.

### Compatibility

Public ABI adds `set_pet_mapping`, `revoke_pet_mapping`, `canonical_for`, and
`celo_for` entry points plus the `PetMappingSet` and `PetMappingRevoked`
events. Update compatibility fixtures when the ABI snapshot changes.
