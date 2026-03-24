#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

#[test]
fn test_accept_transfer_success() {
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env,
};

// -------------------------------------------------------
// Helpers
// -------------------------------------------------------

fn setup() -> (
    Env,
    PetOwnershipContractClient<'static>,
    Address,
    Address,
    u64,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let pet_id = 101u64;

    // 1. Create Pet
    client.create_pet(&pet_id, &owner);

    // 2. Initiate Transfer
    client.initiate_transfer(&pet_id, &new_owner);

    // 3. Accept Transfer
    client.accept_transfer(&pet_id);

    // 4. Verify Ownership
    assert_eq!(client.get_current_owner(&pet_id), new_owner);

    // 5. Verify History
    let history = client.get_ownership_history(&pet_id);
    assert_eq!(history.len(), 2);
    
    let original_record = history.get(0).expect("Original record missing");
    assert_eq!(original_record.owner, owner);
    assert!(original_record.relinquished_at.is_some());

    let new_record = history.get(1).expect("New record missing");
    assert_eq!(new_record.owner, new_owner);
    assert!(new_record.relinquished_at.is_none());
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // NoPendingTransfer
fn test_accept_transfer_no_pending() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let pet_id = 101u64;
    client.accept_transfer(&pet_id);
    let recipient = Address::generate(&env);
    let pet_id: u64 = 1;

    client.create_pet(&pet_id, &owner);

    (env, client, owner, recipient, pet_id)
}

/// Advance the ledger timestamp by `seconds`.
fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|l| l.timestamp += seconds);
}

// -------------------------------------------------------
// Basic transfer lifecycle
// -------------------------------------------------------

#[test]
fn test_create_pet_sets_owner() {
    let (_env, client, owner, _recipient, pet_id) = setup();
    assert_eq!(client.get_current_owner(&pet_id), owner);
}

#[test]
fn test_initiate_and_accept_transfer() {
    let (_env, client, owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);
    assert!(client.has_pending_transfer(&pet_id));

    client.accept_transfer(&pet_id);
    assert!(!client.has_pending_transfer(&pet_id));
    assert_eq!(client.get_current_owner(&pet_id), recipient);

    // Ownership history should have two entries
    let history = client.get_ownership_history(&pet_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().owner, owner);
    assert!(history.get(0).unwrap().relinquished_at.is_some());
    assert_eq!(history.get(1).unwrap().owner, recipient);
    assert!(history.get(1).unwrap().relinquished_at.is_none());
}

#[test]
fn test_cancel_transfer_by_owner() {
    let (_env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);
    assert!(client.has_pending_transfer(&pet_id));

    client.cancel_transfer(&pet_id);
    assert!(!client.has_pending_transfer(&pet_id));
}

#[test]
#[should_panic]
fn test_duplicate_initiate_panics() {
    let (_env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);
    // Second initiate must panic with TransferAlreadyPending
    client.initiate_transfer(&pet_id, &recipient);
}

#[test]
#[should_panic]
fn test_accept_without_pending_panics() {
    let (_env, client, _owner, _recipient, pet_id) = setup();
    client.accept_transfer(&pet_id);
}

#[test]
#[should_panic]
fn test_cancel_without_pending_panics() {
    let (_env, client, _owner, _recipient, pet_id) = setup();
    client.cancel_transfer(&pet_id);
}

// -------------------------------------------------------
// Expiry / reclaim tests  (issue #302)
// -------------------------------------------------------

/// A stale pending transfer (older than TRANSFER_EXPIRY_SECONDS) can be
/// cleared by the original owner via reclaim_transfer, proving that
/// has_pending_transfer cannot remain true indefinitely.
#[test]
fn test_reclaim_expired_transfer_clears_pending_state() {
    let (env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);
    assert!(client.has_pending_transfer(&pet_id));

    // Advance time past the expiry window
    advance_time(&env, TRANSFER_EXPIRY_SECONDS);

    client.reclaim_transfer(&pet_id);

    // Pending state must be cleared — no longer stuck
    assert!(!client.has_pending_transfer(&pet_id));
    // Ownership must remain with the original owner
    assert_eq!(client.get_current_owner(&pet_id), _owner);
}

/// Reclaim at exactly the expiry boundary (initiated_at + TRANSFER_EXPIRY_SECONDS)
/// must succeed.
#[test]
fn test_reclaim_at_exact_expiry_boundary_succeeds() {
    let (env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);

    advance_time(&env, TRANSFER_EXPIRY_SECONDS);
    client.reclaim_transfer(&pet_id);

    assert!(!client.has_pending_transfer(&pet_id));
}

/// Reclaim one second before expiry must be rejected.
#[test]
#[should_panic]
fn test_reclaim_before_expiry_panics() {
    let (env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);

    // One second short of the expiry window
    advance_time(&env, TRANSFER_EXPIRY_SECONDS - 1);
    client.reclaim_transfer(&pet_id);
}

/// After a successful reclaim the owner can initiate a fresh transfer,
/// proving the state machine is fully reset.
#[test]
fn test_new_transfer_possible_after_reclaim() {
    let (env, client, _owner, recipient, pet_id) = setup();

    client.initiate_transfer(&pet_id, &recipient);
    advance_time(&env, TRANSFER_EXPIRY_SECONDS);
    client.reclaim_transfer(&pet_id);

    // Should be able to initiate again without TransferAlreadyPending
    let new_recipient = Address::generate(&env);
    client.initiate_transfer(&pet_id, &new_recipient);
    assert!(client.has_pending_transfer(&pet_id));

    let pending = client.get_pending_transfer(&pet_id).unwrap();
    assert_eq!(pending.to, new_recipient);
}

/// reclaim_transfer on a pet with no pending transfer must panic.
#[test]
#[should_panic]
fn test_reclaim_without_pending_panics() {
    let (env, client, _owner, _recipient, pet_id) = setup();
    advance_time(&env, TRANSFER_EXPIRY_SECONDS);
    client.reclaim_transfer(&pet_id);
}

/// get_pending_transfer returns None when no transfer is pending.
#[test]
fn test_get_pending_transfer_returns_none_when_absent() {
    let (_env, client, _owner, _recipient, pet_id) = setup();
    assert!(client.get_pending_transfer(&pet_id).is_none());
}

/// get_pending_transfer returns the correct record when a transfer is pending.
#[test]
fn test_get_pending_transfer_returns_record() {
    let (env, client, owner, recipient, pet_id) = setup();

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    client.initiate_transfer(&pet_id, &recipient);

    let pending = client.get_pending_transfer(&pet_id).unwrap();
    assert_eq!(pending.pet_id, pet_id);
    assert_eq!(pending.from, owner);
    assert_eq!(pending.to, recipient);
    assert_eq!(pending.initiated_at, 1_000_000);
}

#[test]
fn test_ownership_history_empty_graceful() {
    // This test simulates a case where history is missing but a transfer exists.
    // While this shouldn't happen in normal operation, our fix handles it.
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let pet_id = 101u64;

    // 1. Create Pet manually without history
    env.as_contract(&contract_id, || {
        let pet = Pet {
            pet_id,
            current_owner: owner.clone(),
        };
        env.storage().persistent().set(&DataKey::Pet(pet_id), &pet);
        // Explicitly NOT setting History
    });

    // 2. Initiate Transfer
    client.initiate_transfer(&pet_id, &new_owner);

    // 3. Accept Transfer - should NOT panic despite empty history
    client.accept_transfer(&pet_id);

    // 4. Verify Ownership and History
    assert_eq!(client.get_current_owner(&pet_id), new_owner);
    let history = client.get_ownership_history(&pet_id);
    assert_eq!(history.len(), 1); // Only the new record since original was missing
    assert_eq!(history.get(0).expect("Record missing").owner, new_owner);
}


