#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

#[test]
fn test_accept_transfer_success() {
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


