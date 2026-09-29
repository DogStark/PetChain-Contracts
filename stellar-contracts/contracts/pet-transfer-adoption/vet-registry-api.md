# VetRegistryContract API

`VetRegistryContract` manages veterinarian registration and verification status. It stores vet records by wallet address, prevents duplicate license numbers, and lets an admin verify or revoke a vet.

## Purpose

- Register a vet profile against a wallet address.
- Enforce unique vet addresses and unique license numbers.
- Let a designated admin verify or revoke vets.
- Provide read helpers for vet lookup and verification checks.

## Data Types

### `Vet`

```rust
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
}
```

### `VetStatus`

```rust
pub enum VetStatus {
    Registered,
    Verified,
    Revoked,
}
```

`VetStatus` is defined in the contract source but the current public API uses the `Vet.verified` boolean rather than returning this enum.

### `VetFilter`

```rust
pub struct VetFilter {
    pub verified_only: bool,
    pub specialization: Option<String>,
}
```

### `VetPage`

```rust
pub struct VetPage {
    pub vets: Vec<Vet>,
    pub next_cursor: Option<u64>,
}
```

## Admin Setup

`init(env: Env, admin: Address)` must be called exactly once after deployment.

Recommended setup flow:

1. Deploy the contract.
2. Immediately call `init` with the wallet address that should control vet verification.
3. Use that same admin address for all future `verify_vet` and `revoke_vet_license` calls.

Important implementation detail:

- `init` only checks whether `DataKey::Admin` is already set.
- `init` does not call `require_auth()` on the provided admin address.
- Because of that, initialization should happen immediately after deployment to avoid an unintended party setting the admin first.

## Input Length Limits

The contract enforces these maximum lengths during `register_vet`:

| Field | Max Length |
|---|---|
| `name` | 100 characters |
| `license_number` | 50 characters |
| `specialization` | 100 characters |

Current behavior:

- Inputs longer than these limits fail with `ContractError::InputTooLong`.
- The contract does not currently reject empty strings for these fields; it only enforces the maximum length.

## Public Functions

### `init(env: Env, admin: Address)`

Stores the contract admin address.

- Auth: none
- Can only be called once
- Fails with:
  - `AlreadyInitialized` if the admin was already set

### `register_vet(env: Env, vet_address: Address, name: String, license_number: String, specialization: String)`

Registers a new vet record.

- Auth: `vet_address.require_auth()`
- Validation:
  - `name.len() <= 100`
  - `license_number.len() <= 50`
  - `specialization.len() <= 100`
  - address must not already be registered
  - license number must not already be in use
- Effects:
  - stores the full `Vet` under `VetByAddress`
  - stores the reverse lookup under `VetByLicense`
  - initializes `verified` to `false`
- Emits:
  - `reg_vet`

### `verify_vet(env: Env, vet_address: Address)`

Marks a registered vet as verified.

- Auth: stored admin address via `require_admin`
- Fails with:
  - `Unauthorized` if the admin has not been initialized or the admin auth is missing
  - `VetNotFound` if the target address is not registered
- Emits:
  - `ver_vet`

### `revoke_vet_license(env: Env, vet_address: Address)`

Marks a registered vet as not verified.

- Auth: stored admin address via `require_admin`
- Fails with:
  - `Unauthorized` if the admin has not been initialized or the admin auth is missing
  - `VetNotFound` if the target address is not registered
- Emits:
  - `rev_vet`

### `get_vet(env: Env, vet_address: Address) -> Vet`

Returns the stored vet record for an address.

- Fails with:
  - `VetNotFound` if the address is not registered

### `is_verified_vet(env: Env, vet_address: Address) -> bool`

Returns the current `verified` flag for a registered vet.

- Fails with:
  - `VetNotFound` if the address is not registered

### `get_vet_count(env: Env) -> u64`

Returns the number of registry slots (every vet ever registered, including revoked ones).

### `list_vets_page(env: Env, cursor: u64, limit: u32, filter: VetFilter) -> VetPage`

Cursor-based registry enumeration. See [Pagination](#pagination).

- Auth: none
- Fails with:
  - `InvalidPageLimit` if `limit` is `0` or greater than `MAX_VET_PAGE_SIZE` (15)
  - `InvalidCursor` if `cursor` is greater than `get_vet_count()`
  - `InputTooLong` if `filter.specialization` is longer than 100 characters

### `list_vets(env: Env, offset: u64, limit: u32, verified_only: bool) -> Vec<Vet>`

Offset-based listing kept for compatibility. Inspects the registry slots `offset..offset + limit` and returns the vets among them that pass the `verified_only` filter.

- Auth: none
- `limit` is capped at `MAX_VET_PAGE_SIZE` (15); larger values are treated as 15.
- Returns an empty list when `limit` is `0` or `offset >= get_vet_count()`.

## Pagination

Every registered vet occupies a registry slot (`VetIndex`), numbered from 1 in registration order. Slots are append-only: revoking a vet keeps its slot, and new vets are added at the end.

`list_vets_page` inspects at most `limit` slots after `cursor` and returns the vets among them that match `filter`:

- `verified_only`: only vets whose `verified` flag is set.
- `specialization`: only vets whose specialization equals this value exactly.

To read the whole registry, start with `cursor = 0` and keep passing `next_cursor` back until it is `None`. With a filter set, a page can hold fewer than `limit` vets, or none at all, while `next_cursor` is still `Some`; keep going until it is `None`.

Because slots never move, a cursor stays valid while vets are registered or revoked between calls: no vet is skipped or returned twice, and vets registered after the walk started appear at the end.

### Resource bounds

Each inspected slot costs two persistent reads (`VetIndex` and `VetByAddress`), so a call reads at most `2 * MAX_VET_PAGE_SIZE + 1` entries (30 slot reads plus `VetCount`) no matter how large the registry is. Measured in the Soroban test host (soroban-sdk 21.7.7), a full 15-slot page costs about 0.57M CPU instructions and 42 KB of memory at 20 vets, and about 0.67M CPU instructions and 42 KB at 300 vets, excluding fixed per-invocation overhead. The uncapped `list_vets(0, u32::MAX, false)` used before this change cost about 13.4M CPU instructions and 1.25 MB at 300 vets.

### Migration

- `list_vets` now caps `limit` at 15. Callers that passed a larger `limit` to fetch everything in one call get the first 15 slots only; switch them to `list_vets_page` and follow `next_cursor`.
- No storage changes: `list_vets_page` reads the existing `VetCount`, `VetIndex` and `VetByAddress` entries.

## Events

### `reg_vet`

- Topic: `(symbol_short!("reg_vet"),)`
- Payload: `vet_address`
- Emitted by: `register_vet`

### `ver_vet`

- Topic: `(symbol_short!("ver_vet"),)`
- Payload: `vet_address`
- Emitted by: `verify_vet`

### `rev_vet`

- Topic: `(symbol_short!("rev_vet"),)`
- Payload: `vet_address`
- Emitted by: `revoke_vet_license`

## Error Codes

| Code | Error | Meaning |
|---|---|---|
| 0 | `AlreadyInitialized` | `init` was called after the admin was already set. |
| 1 | `Unauthorized` | Admin authorization is missing or the registry is not initialized. |
| 2 | `VetAlreadyRegistered` | The vet wallet address is already registered. |
| 3 | `VetNotFound` | No vet exists for the provided address. |
| 4 | `LicenseAlreadyUsed` | Another vet already registered the same license number. |
| 5 | `VetNotVerified` | Defined in the contract, but not currently raised by the public API. |
| 6 | `InputTooLong` | One of the input strings exceeded its configured maximum length. |
| 10 | `InvalidPageLimit` | `list_vets_page` was called with `limit` of `0` or above `MAX_VET_PAGE_SIZE`. |
| 11 | `InvalidCursor` | `list_vets_page` was called with a cursor past the end of the registry. |

## Rust Usage Examples

### Initialize the registry and register a vet

```rust
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use crate::vet_registry::{VetRegistryContract, VetRegistryContractClient};

let env = Env::default();
env.mock_all_auths();

let contract_id = env.register_contract(None, VetRegistryContract);
let client = VetRegistryContractClient::new(&env, &contract_id);

let admin = Address::generate(&env);
let vet = Address::generate(&env);

client.init(&admin);
client.register_vet(
    &vet,
    &String::from_str(&env, "Dr. Ada"),
    &String::from_str(&env, "LIC-2026-001"),
    &String::from_str(&env, "Small Animal Surgery"),
);

assert!(!client.is_verified_vet(&vet));
```

### Verify and revoke a vet

```rust
client.verify_vet(&vet);
assert!(client.is_verified_vet(&vet));

client.revoke_vet_license(&vet);
assert!(!client.is_verified_vet(&vet));
```
