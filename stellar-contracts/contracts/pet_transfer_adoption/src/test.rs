use super::{
    ContractError, DataKey, OwnershipRecord, PetOwnershipContract, PetOwnershipContractClient,
};
use soroban_sdk::{testutils::Address as _, Address, Env, Error, Vec};

fn setup() -> (Env, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let pet_id = 1;

    (env, owner, new_owner, pet_id)
}

fn create_pending_transfer(
    client: &PetOwnershipContractClient,
    pet_id: u64,
    owner: &Address,
    new_owner: &Address,
) {
    client.create_pet(&pet_id, owner);
    client.initiate_transfer(&pet_id, new_owner);
}

#[test]
fn get_owner_pets_returns_all_pets_for_owner() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&1, &owner);
    client.create_pet(&2, &owner);
    client.create_pet(&3, &new_owner);

    let owner_pets = client.get_owner_pets(&owner);
    assert_eq!(owner_pets.len(), 2);
    assert_eq!(owner_pets.get(0), Some(1));
    assert_eq!(owner_pets.get(1), Some(2));

    let new_owner_pets = client.get_owner_pets(&new_owner);
    assert_eq!(new_owner_pets.len(), 1);
    assert_eq!(new_owner_pets.get(0), Some(3));
}

#[test]
fn get_owner_pets_updates_after_transfer_acceptance() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.create_pet(&2, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    let owner_pets = client.get_owner_pets(&owner);
    assert_eq!(owner_pets.len(), 1);
    assert_eq!(owner_pets.get(0), Some(2));

    let new_owner_pets = client.get_owner_pets(&new_owner);
    assert_eq!(new_owner_pets.len(), 1);
    assert_eq!(new_owner_pets.get(0), Some(pet_id));
}

#[test]
fn create_pet_does_not_duplicate_owner_pet_index() {
    let (env, owner, _, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.create_pet(&pet_id, &owner);

    let owner_pets = client.get_owner_pets(&owner);
    assert_eq!(owner_pets.len(), 1);
    assert_eq!(owner_pets.get(0), Some(pet_id));
}

#[test]
fn accept_transfer_errors_when_history_is_missing() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    create_pending_transfer(&client, pet_id, &owner, &new_owner);

    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .remove(&DataKey::OwnershipHistory(pet_id));
    });

    let result = client.try_accept_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::EmptyOwnershipHistory as u32,
        )))
    );
}

#[test]
fn accept_transfer_errors_when_history_is_empty() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    create_pending_transfer(&client, pet_id, &owner, &new_owner);

    let empty_history = Vec::<OwnershipRecord>::new(&env);
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::OwnershipHistory(pet_id), &empty_history);
    });

    let result = client.try_accept_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::EmptyOwnershipHistory as u32,
        )))
    );
}

#[test]
fn cancel_transfer_errors_when_stale() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    // Create pet and initiate a transfer
    create_pending_transfer(&client, pet_id, &owner, &new_owner);

    // Deliberately alter the pet's current_owner to simulate a divergent state
    let rogue_owner = Address::generate(&env);
    env.as_contract(&contract_id, || {
        let mut pet: super::Pet = env
            .storage()
            .persistent()
            .get(&DataKey::Pet(pet_id))
            .unwrap();
        pet.current_owner = rogue_owner;
        env.storage().persistent().set(&DataKey::Pet(pet_id), &pet);
    });

    // The original owner tries to cancel the transfer, but they no longer match pet.current_owner
    let result = client.try_cancel_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::StaleCancellation as u32,
        )))
    );
}

// ======================================================
// batch_initiate_transfer tests
// ======================================================

#[test]
fn batch_initiate_transfer_creates_pending_transfers_for_all_pets() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&1, &owner);
    client.create_pet(&2, &owner);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(2u64);
    client.batch_initiate_transfer(&ids, &new_owner);

    assert!(client.has_pending_transfer(&1));
    assert!(client.has_pending_transfer(&2));

    let t1 = client.get_pending_transfer(&1).unwrap();
    assert_eq!(t1.from, owner);
    assert_eq!(t1.to, new_owner);

    let t2 = client.get_pending_transfer(&2).unwrap();
    assert_eq!(t2.from, owner);
    assert_eq!(t2.to, new_owner);
}

#[test]
fn batch_initiate_transfer_single_element_behaves_like_initiate_transfer() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(pet_id);
    client.batch_initiate_transfer(&ids, &new_owner);

    assert!(client.has_pending_transfer(&pet_id));
    let transfer = client.get_pending_transfer(&pet_id).unwrap();
    assert_eq!(transfer.from, owner);
    assert_eq!(transfer.to, new_owner);
    assert_eq!(transfer.pet_id, pet_id);
}

#[test]
fn batch_initiate_transfer_errors_on_empty_batch() {
    let (env, _, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let ids = soroban_sdk::Vec::<u64>::new(&env);
    let result = client.try_batch_initiate_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::EmptyBatch as u32,
        )))
    );
}

#[test]
fn batch_initiate_transfer_errors_when_a_pet_is_not_found() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&1, &owner);
    // pet 99 was never created

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(99u64);
    let result = client.try_batch_initiate_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::PetNotFound as u32,
        )))
    );
    // No side effects: pet 1 must not have a pending transfer
    assert!(!client.has_pending_transfer(&1));
}

#[test]
fn batch_initiate_transfer_errors_on_owner_mismatch() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let other_owner = Address::generate(&env);
    client.create_pet(&1, &owner);
    client.create_pet(&2, &other_owner);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(2u64);
    let result = client.try_batch_initiate_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::BatchOwnerMismatch as u32,
        )))
    );
    assert!(!client.has_pending_transfer(&1));
    assert!(!client.has_pending_transfer(&2));
}

#[test]
fn batch_initiate_transfer_errors_when_a_pet_already_has_pending_transfer() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&1, &owner);
    client.create_pet(&2, &owner);
    // pet 1 already has a pending transfer
    client.initiate_transfer(&1, &new_owner);

    let mut ids = soroban_sdk::Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(2u64);
    let result = client.try_batch_initiate_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::TransferAlreadyPending as u32,
        )))
    );
    // Atomicity: pet 2 must remain unaffected
    assert!(!client.has_pending_transfer(&2));
}
