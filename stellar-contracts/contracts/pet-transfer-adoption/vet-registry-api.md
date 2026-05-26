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
