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

/// ======================================================
/// VET REGISTRY INITIALIZATION TESTS
/// ======================================================

#[cfg(test)]
mod vet_registry_init_tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_init_success() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);

        // Should successfully initialize
        vet_registry_client.init(&admin);

        // Verify initialization was successful by checking admin is set
        // (Note: There's no getter for admin in current implementation,
        // but this test ensures the init call completes without error)
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #0)")]
    fn test_init_already_initialized() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);

        // First initialization should succeed
        vet_registry_client.init(&admin);

        // Second initialization should fail with AlreadyInitialized error
        // AlreadyInitialized = 0, so it should panic with error code 0
        vet_registry_client.init(&admin);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #0)")]
    fn test_init_already_initialized_assert_error_code() {
        let env = Env::default();
        let vet_registry_contract_id = env.register_contract(None, vet_registry::VetRegistryContract);
        let vet_registry_client = vet_registry::VetRegistryContractClient::new(&env, &vet_registry_contract_id);

        let admin = env.accounts().generate(None);
        let admin2 = env.accounts().generate(None);

        // First initialization
        vet_registry_client.init(&admin);

        // Try to initialize again with different admin - should fail with error code 0
        vet_registry_client.init(&admin2);
    }
}
