use super::{
    ContractError, CustodyEntry, DataKey, EscrowedTransfer, OwnershipRecord, PetOwnershipContract,
    PetOwnershipContractClient, TransferType, DISPUTE_WINDOW_SECONDS,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, Error, Vec,
};

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

fn address_vec(env: &Env, addresses: &[Address]) -> Vec<Address> {
    let mut out = Vec::new(env);
    for address in addresses {
        out.push_back(address.clone());
    }
    out
}

#[test]
fn trusted_contract_validation_rejects_untrusted_callee() {
    let (env, owner, _, _) = setup();
    let trusted = Address::generate(&env);
    let untrusted = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(&env, &[owner.clone()]);

    client.init_trusted_contract(&trusted, &admins, &1);
    assert!(client.validate_trusted_contract(&trusted));

    let result = client.try_validate_trusted_contract(&untrusted);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::UntrustedContract as u32,
        )))
    );
}

#[test]
fn trusted_contract_update_requires_multisig_threshold() {
    let (env, admin_one, admin_two, _) = setup();
    let trusted = Address::generate(&env);
    let updated = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(&env, &[admin_one.clone(), admin_two.clone()]);

    client.init_trusted_contract(&trusted, &admins, &2);
    assert!(!client.update_trusted_contract(&updated, &admin_one));
    assert_eq!(client.get_trusted_contract_address(), trusted);

    assert!(client.update_trusted_contract(&updated, &admin_two));
    assert_eq!(client.get_trusted_contract_address(), updated);
    assert_eq!(env.events().all().len(), 1);
}

#[test]
fn trusted_contract_update_rejects_non_admin_signer() {
    let (env, admin, attacker, _) = setup();
    let trusted = Address::generate(&env);
    let updated = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(&env, &[admin]);

    client.init_trusted_contract(&trusted, &admins, &1);
    let result = client.try_update_trusted_contract(&updated, &attacker);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::NotMultisigAdmin as u32,
        )))
    );
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
    client.accept_transfer(&pet_id); // → Escrowed

    // Advance past the 48-hour dispute window then finalize
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

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
fn finalize_transfer_errors_when_history_is_missing() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    create_pending_transfer(&client, pet_id, &owner, &new_owner);
    client.accept_transfer(&pet_id); // → Escrowed

    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .remove(&DataKey::OwnershipHistory(pet_id));
    });

    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });

    let result = client.try_finalize_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::EmptyOwnershipHistory as u32,
        )))
    );
}

#[test]
fn finalize_transfer_errors_when_history_is_empty() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    create_pending_transfer(&client, pet_id, &owner, &new_owner);
    client.accept_transfer(&pet_id); // → Escrowed

    let empty_history = Vec::<OwnershipRecord>::new(&env);
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&DataKey::OwnershipHistory(pet_id), &empty_history);
    });

    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });

    let result = client.try_finalize_transfer(&pet_id);
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

// ======================================================
// Escrow + dispute window tests
// ======================================================

#[test]
fn accept_transfer_enters_escrowed_state() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    // Ownership must NOT have changed yet
    assert_eq!(client.get_current_owner(&pet_id), owner);
    // Pending transfer is gone; escrowed transfer exists
    assert!(!client.has_pending_transfer(&pet_id));
    let escrowed = client.get_escrowed_transfer(&pet_id).unwrap();
    assert_eq!(escrowed.from, owner);
    assert_eq!(escrowed.to, new_owner);
    assert!(!escrowed.disputed);
}

#[test]
fn finalize_transfer_before_window_is_rejected() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    // Advance time but stay within the window
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS - 1;
    });

    let result = client.try_finalize_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::DisputeWindowNotElapsed as u32,
        )))
    );
    // Ownership unchanged
    assert_eq!(client.get_current_owner(&pet_id), owner);
}

#[test]
fn finalize_transfer_after_window_transfers_ownership() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    assert_eq!(client.get_current_owner(&pet_id), new_owner);
    assert!(client.get_escrowed_transfer(&pet_id).is_none());
}

#[test]
fn raise_dispute_blocks_finalization() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);
    client.raise_dispute(&pet_id, &owner);

    // Escrowed transfer is now marked disputed
    let escrowed = client.get_escrowed_transfer(&pet_id).unwrap();
    assert!(escrowed.disputed);

    // Finalize must fail even after the window
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    let result = client.try_finalize_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::TransferAlreadyDisputed as u32,
        )))
    );
    // Ownership unchanged
    assert_eq!(client.get_current_owner(&pet_id), owner);
}

#[test]
fn raise_dispute_after_window_is_rejected() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });

    let result = client.try_raise_dispute(&pet_id, &owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::DisputeWindowNotElapsed as u32,
        )))
    );
}

#[test]
fn finalize_transfer_no_escrowed_transfer_errors() {
    let (env, _, _, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let result = client.try_finalize_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::NoEscrowedTransfer as u32,
        )))
    );
}

#[test]
fn recipient_can_raise_dispute_during_window() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);
    // Recipient (new_owner / `to`) raises the dispute
    client.raise_dispute(&pet_id, &new_owner);

    let escrowed = client.get_escrowed_transfer(&pet_id).unwrap();
    assert!(escrowed.disputed);
}

#[test]
fn unauthorized_party_cannot_raise_dispute() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    let stranger = Address::generate(&env);
    let result = client.try_raise_dispute(&pet_id, &stranger);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::Unauthorized as u32,
        )))
    );
}

#[test]
fn double_dispute_is_rejected() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);
    client.raise_dispute(&pet_id, &owner);

    let result = client.try_raise_dispute(&pet_id, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::TransferAlreadyDisputed as u32,
        )))
    );
}

// ======================================================
// Chain-of-custody tests (Issue #637)
// ======================================================

#[test]
fn finalize_transfer_appends_direct_custody_entry() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);

    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 1);

    let entry = chain.get(0).unwrap();
    assert_eq!(entry.from, owner);
    assert_eq!(entry.to, new_owner);
    assert_eq!(entry.transfer_type, TransferType::Direct);
}

#[test]
fn multiple_finalizations_produce_ordered_chain() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let third_owner = Address::generate(&env);

    client.create_pet(&pet_id, &owner);

    // First transfer
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    // Second transfer
    client.initiate_transfer(&pet_id, &third_owner);
    client.accept_transfer(&pet_id);
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 2);

    let first = chain.get(0).unwrap();
    assert_eq!(first.from, owner);
    assert_eq!(first.to, new_owner);

    let second = chain.get(1).unwrap();
    assert_eq!(second.from, new_owner);
    assert_eq!(second.to, third_owner);

    assert!(first.timestamp <= second.timestamp);
}

#[test]
fn get_custody_chain_empty_before_any_transfer() {
    let (env, owner, _, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 0);
}

#[test]
fn custody_chain_is_append_only_no_delete_path() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);
    client.accept_transfer(&pet_id);
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    // Chain has one entry; no contract function can remove it
    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 1);

    // A second transfer grows the chain, never shrinks it
    let third_owner = Address::generate(&env);
    client.initiate_transfer(&pet_id, &third_owner);
    client.accept_transfer(&pet_id);
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 2);
}
