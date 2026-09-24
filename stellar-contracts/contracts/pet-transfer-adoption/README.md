# Pet Transfer Adoption Contract

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
  - `DataKey::OwnershipCount(pet_id)` (set to 1)
  - `DataKey::OwnershipEntry((pet_id, 1))`
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

Accepts an existing pending transfer and moves it into escrow. Ownership does not change yet; see [Two-Party Custody Confirmation](#two-party-custody-confirmation).

- Auth: `PendingTransfer.to`
- Behavior:
  - Confirms the pet is still owned by `PendingTransfer.from`
  - Fails with `TransferAlreadyPending` if the pet already has an escrowed transfer
  - Removes the pending transfer and stores an `EscrowedTransfer`
- Emits:
  - `xfer_escr`

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

Returns the ownership history for `pet_id` using a paginated index pattern.
Entries are stored individually at `DataKey::OwnershipEntry((pet_id, seq))` with
a count tracked at `DataKey::OwnershipCount(pet_id)`. The history is capped at
`MAX_OWNERSHIP_HISTORY_LEN` (512) entries; oldest entries are trimmed on save.

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
| 32 | `InputStringTooLong` | A string argument exceeded its maximum allowed length. |
| 35 | `CustodyNotConfirmed` | `finalize_transfer` was called before both parties confirmed custody. |
| 36 | `CustodyAlreadyConfirmed` | The caller already confirmed custody, or both parties have confirmed and the transfer can no longer be cancelled. |
| 37 | `CustodyConfirmationExpired` | `confirm_custody` was called after the confirmation window closed. |
| 38 | `TransferNotDisputed` | `resolve_custody_dispute` was called on a transfer that is not disputed. |

## Two-Party Custody Confirmation

Ownership only moves once both the transferor (`from`) and the recipient (`to`) confirm that the pet has physically changed hands. Neither party can complete the transfer alone.

1. `initiate_transfer` (owner) creates a `PendingTransfer`.
2. `accept_transfer` (recipient) moves it into an `EscrowedTransfer`. This starts the 48-hour dispute window (`DISPUTE_WINDOW_SECONDS`) and the 7-day confirmation window (`CUSTODY_CONFIRMATION_TIMEOUT_SECONDS`).
3. `confirm_custody` is called once by `from` and once by `to` before the confirmation window closes.
4. `finalize_transfer` moves ownership once both parties have confirmed, the dispute window has elapsed and there is no dispute. Anyone may call it.

Two other paths end an escrowed transfer:

- **Timeout.** If both confirmations aren't in by the end of the confirmation window, anyone may call `cancel_unconfirmed_transfer`. The escrow is cleared and ownership stays with `from`.
- **Arbitration.** A transfer disputed with `raise_dispute` cannot be confirmed, finalized or timed out. A trusted multisig admin (configured with `init_trusted_contract`) settles it with `resolve_custody_dispute`: `complete = true` moves ownership to `to`, and `false` cancels the transfer.

While a pet has an escrowed transfer, `initiate_transfer`, `initiate_transfer_with_timeout`, `batch_initiate_transfer`, `batch_transfer`, `accept_transfer`, `sign_adoption`, `complete_adoption` and `waive_waiting_period` fail with `TransferAlreadyPending`. Ownership cannot move by any path other than the three above.

### `confirm_custody(env: Env, pet_id: u64, caller: Address)`

- Auth: `caller`, which must be `EscrowedTransfer.from` or `.to`
- Fails with `NoEscrowedTransfer`, `Unauthorized`, `TransferAlreadyDisputed`, `CustodyConfirmationExpired`, or `CustodyAlreadyConfirmed` (the same party confirming twice)
- Emits `cust_conf` with topic `(symbol_short!("cust_conf"), pet_id)` and payload `caller`

### `get_custody_confirmation(env: Env, pet_id: u64) -> Option<CustodyConfirmation>`

Returns `{ from_confirmed, to_confirmed }` for the escrowed transfer, or `None` if there is none.

### `finalize_transfer(env: Env, pet_id: u64)`

- Auth: none (both parties have already confirmed)
- Fails with `NoEscrowedTransfer`, `TransferAlreadyDisputed`, `DisputeWindowNotElapsed`, or `CustodyNotConfirmed`
- Emits `xfer_fin`

### `cancel_unconfirmed_transfer(env: Env, pet_id: u64)`

- Auth: none
- Fails with `NoEscrowedTransfer`, `TransferAlreadyDisputed`, `TransferNotExpired` (window still open), or `CustodyAlreadyConfirmed` (both confirmed; use `finalize_transfer`)
- Emits `cust_exp` with payload `(from, to)`

### `resolve_custody_dispute(env: Env, pet_id: u64, arbitrator: Address, complete: bool)`

- Auth: `arbitrator`, which must be one of the trusted multisig admins
- Fails with `NotMultisigAdmin`, `NoEscrowedTransfer`, or `TransferNotDisputed`
- Emits `xfer_arb` with payload `(arbitrator, from, to, complete)`, plus `xfer_fin` when `complete` is `true`

### Storage

Confirmations are stored under a new key, `DataKey::CustodyConfirmation(pet_id)`, which is removed when the escrow ends. The layouts of `EscrowedTransfer` and all existing keys are unchanged, and existing error codes keep their values.

### Threat model

- A recipient cannot take ownership without the transferor confirming the handover, and a transferor cannot push ownership onto a recipient who hasn't confirmed.
- A dispute freezes the pet. It can only be settled by a trusted admin, and neither party can bypass it by starting a new transfer, a batch transfer or an adoption.
- A party that never confirms can only delay the transfer until the confirmation window closes. After that, anyone can cancel it and the pet returns to its pre-transfer state.
- Arbitration trusts any single trusted multisig admin. If no trusted admins are configured, disputed transfers stay frozen.

### Resource impact

Measured in the Soroban test host (soroban-sdk 21.7.7), CPU instructions / memory bytes:

| Call | Before | After |
|---|---|---|
| `initiate_transfer` | 75,073 / 11,968 | 86,319 / 14,085 |
| `accept_transfer` | 91,144 / 15,132 | 94,885 / 14,345 |
| `confirm_custody` (each party) | — | ~84,000 / ~13,100 |
| `finalize_transfer` | 344,733 / 50,062 | 374,781 / 57,155 |
| `batch_transfer` (1 pet) | 356,427 / 54,930 | 384,507 / 62,875 |

Each guarded entry point makes one extra `has` read per pet. `finalize_transfer` reads and removes one extra entry.

### Migration

- Escrowed transfers created before this upgrade need both `confirm_custody` calls before they can be finalized. Their confirmation window is counted from the original `escrowed_at`. If the window has already closed, they can only be cancelled with `cancel_unconfirmed_transfer`, or resolved by arbitration if they are disputed.
- Clients that called `finalize_transfer` straight after the dispute window must now have both parties call `confirm_custody` first.

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

### Accept, confirm custody, and finalize a transfer

```rust
use pet_transfer_adoption::DISPUTE_WINDOW_SECONDS;

client.accept_transfer(&1);
client.confirm_custody(&1, &owner);
client.confirm_custody(&1, &recipient);

env.ledger().with_mut(|ledger| {
    ledger.timestamp += DISPUTE_WINDOW_SECONDS;
});
client.finalize_transfer(&1);

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
