# pet_transfer_adoption Contract

`pet_transfer_adoption` provides a minimal Soroban ownership-transfer flow for pets. It tracks the current owner, keeps an ownership history per pet, and supports pending transfers that can be accepted, cancelled, or reclaimed after expiry.

## Purpose

- Bootstrap a pet with an initial owner.
- Initiate a pending transfer from the current owner to a recipient.
- Let the recipient accept the transfer.
- Let the sender cancel an active transfer.
- Let the sender reclaim a transfer after the expiry window elapses.
- Expose read helpers for current owner, ownership history, and pending-transfer state.

## Data Types

### `Pet`

```rust
pub struct Pet {
    pub pet_id: u64,
    pub current_owner: Address,
}
```

### `PendingTransfer`

```rust
pub struct PendingTransfer {
    pub pet_id: u64,
    pub from: Address,
    pub to: Address,
    pub initiated_at: u64,
}
```

### `OwnershipRecord`

```rust
pub struct OwnershipRecord {
    pub owner: Address,
    pub acquired_at: u64,
    pub relinquished_at: Option<u64>,
}
```

## Public Functions

### `create_pet(env: Env, pet_id: u64, owner: Address)`

Creates the initial pet record and ownership history entry.

- Auth: `owner.require_auth()`
- Writes:
  - `DataKey::Pet(pet_id)`
  - `DataKey::OwnershipHistory(pet_id)`
- Notes:
  - The initial history record uses the current ledger timestamp as `acquired_at`.
  - This function is intended as the bootstrap step before transfers.

### `initiate_transfer(env: Env, pet_id: u64, to: Address)`

Creates a pending transfer for a pet.

- Auth: current owner of `pet_id`
- Fails if a pending transfer already exists for the pet.
- Stores:
  - `DataKey::PendingTransfer(pet_id)`
- Emits:
  - `xfer_init`

### `accept_transfer(env: Env, pet_id: u64)`

Accepts an existing pending transfer.

- Auth: `PendingTransfer.to`
- Behavior:
  - Confirms the pet is still owned by `PendingTransfer.from`
  - Closes the previous ownership record by setting `relinquished_at`
  - Appends a new `OwnershipRecord` for the recipient
  - Updates `Pet.current_owner`
  - Removes the pending transfer
- Emits:
  - `xfer_ok`

### `cancel_transfer(env: Env, pet_id: u64)`

Cancels an active pending transfer before expiry.

- Auth: `PendingTransfer.from`
- Behavior:
  - Verifies the sender is still the current owner
  - Removes the pending transfer
- Emits:
  - `xfer_cncl`

### `reclaim_transfer(env: Env, pet_id: u64)`

Cancels an expired pending transfer.

- Auth: `PendingTransfer.from`
- Behavior:
  - Requires the transfer age to be at least `TRANSFER_EXPIRY_SECONDS`
  - Removes the pending transfer
- Emits:
  - `xfer_cncl`

### `get_current_owner(env: Env, pet_id: u64) -> Address`

Returns the current owner for `pet_id`.

### `get_ownership_history(env: Env, pet_id: u64) -> Vec<OwnershipRecord>`

Returns the ownership history vector for `pet_id`.

### `has_pending_transfer(env: Env, pet_id: u64) -> bool`

Returns `true` if `DataKey::PendingTransfer(pet_id)` exists.

### `get_pending_transfer(env: Env, pet_id: u64) -> Option<PendingTransfer>`

Returns the current pending transfer, or `None` if there is no active transfer.

## Events

### `xfer_init`

- Topic: `(symbol_short!("xfer_init"), pet_id)`
- Payload: `(from, to)`
- Emitted by: `initiate_transfer`

### `xfer_ok`

- Topic: `(symbol_short!("xfer_ok"), pet_id)`
- Payload: `(from, to)`
- Emitted by: `accept_transfer`

### `xfer_cncl`

- Topic: `(symbol_short!("xfer_cncl"), pet_id)`
- Payload: `(from, to)`
- Emitted by: `cancel_transfer`, `reclaim_transfer`

## Error Codes

| Code | Error | Meaning |
|---|---|---|
| 1 | `PetNotFound` | The requested pet does not exist. |
| 2 | `Unauthorized` | The caller is not allowed to perform the action. |
| 3 | `TransferAlreadyPending` | A transfer already exists for this pet. |
| 4 | `NoPendingTransfer` | No pending transfer exists for this pet. |
| 5 | `InvalidRecipient` | Reserved error variant; not currently raised by the contract. |
| 6 | `EmptyOwnershipHistory` | Ownership history was missing or empty when processing acceptance. |
| 7 | `MissingOwnershipRecord` | The latest ownership record could not be loaded. |
| 8 | `TransferNotExpired` | `reclaim_transfer` was called before the expiry window elapsed. |
| 9 | `StaleCancellation` | The sender tried to cancel a transfer after the pet owner had changed. |

## Transfer Expiry Policy

Pending transfers expire after `TRANSFER_EXPIRY_SECONDS`, which is currently:

```rust
pub const TRANSFER_EXPIRY_SECONDS: u64 = 7 * 24 * 60 * 60; // 604_800 seconds
```

Important details:

- Expiry uses `env.ledger().timestamp()` and `PendingTransfer.initiated_at`.
- The contract compares `current_timestamp - initiated_at >= TRANSFER_EXPIRY_SECONDS`.
- Before expiry, the sender should use `cancel_transfer`.
- After expiry, the sender can use `reclaim_transfer` without recipient cooperation.
- Expiry is based on ledger timestamps, not ledger sequence numbers.

## Rust Usage Examples

### Create a pet and initiate a transfer

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};
use pet_transfer_adoption::{PetOwnershipContract, PetOwnershipContractClient};

let env = Env::default();
env.mock_all_auths();

let contract_id = env.register_contract(None, PetOwnershipContract);
let client = PetOwnershipContractClient::new(&env, &contract_id);

let owner = Address::generate(&env);
let recipient = Address::generate(&env);

client.create_pet(&1, &owner);
client.initiate_transfer(&1, &recipient);

assert!(client.has_pending_transfer(&1));
```

### Accept a transfer

```rust
client.accept_transfer(&1);

let current_owner = client.get_current_owner(&1);
assert_eq!(current_owner, recipient);

let history = client.get_ownership_history(&1);
assert_eq!(history.len(), 2);
```

### Cancel a transfer before expiry

```rust
client.create_pet(&7, &owner);
client.initiate_transfer(&7, &recipient);
client.cancel_transfer(&7);

assert!(!client.has_pending_transfer(&7));
```

### Reclaim a transfer after expiry

```rust
use pet_transfer_adoption::TRANSFER_EXPIRY_SECONDS;

client.create_pet(&9, &owner);
client.initiate_transfer(&9, &recipient);

env.ledger().with_mut(|ledger| {
    ledger.timestamp += TRANSFER_EXPIRY_SECONDS;
});

client.reclaim_transfer(&9);
assert_eq!(client.get_pending_transfer(&9), None);
```
