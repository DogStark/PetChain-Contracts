use super::{
    AdoptionState, ContractError, CustodyEntry, DataKey, EscrowedTransfer, OwnershipRecord,
    PetOwnershipContract, PetOwnershipContractClient, TransferType, TrustedUpdateApprovalKey,
    DISPUTE_WINDOW_SECONDS,
    PetOwnershipContract, PetOwnershipContractClient, TransferType, DISPUTE_WINDOW_SECONDS,
    MAX_REJECTION_REASON_LEN,
    MAX_CUSTODY_CHAIN_LENGTH,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, Error, String, Vec,
};

fn setup() -> (Env, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let pet_id = 1;

    (env, owner, new_owner, pet_id)
}

// ======================================================
// cancel_expired_transfer tests (Issue #797)
// ======================================================

#[test]
fn cancel_expired_transfer_before_timeout_is_rejected() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer_with_timeout(&pet_id, &new_owner, &7u32);

    // Within the window — must fail
    let result = client.try_cancel_expired_transfer(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::TransferNotExpired as u32,
        )))
    );
    assert!(client.has_pending_transfer(&pet_id));
}

#[test]
fn cancel_expired_transfer_after_timeout_by_owner_succeeds() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer_with_timeout(&pet_id, &new_owner, &7u32);

    env.ledger().with_mut(|l| {
        l.timestamp += 7 * 24 * 60 * 60 + 1;
    });

    // Owner calls cancel_expired_transfer (no auth requirement, but owner can still call it)
    client.cancel_expired_transfer(&pet_id);

    assert!(!client.has_pending_transfer(&pet_id));
    assert_eq!(client.get_current_owner(&pet_id), owner);
}

#[test]
fn cancel_expired_transfer_after_timeout_by_third_party_succeeds() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let stranger = Address::generate(&env);
    let _ = stranger; // cancel_expired_transfer takes no caller param; anyone can invoke

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer_with_timeout(&pet_id, &new_owner, &3u32);

    env.ledger().with_mut(|l| {
        l.timestamp += 3 * 24 * 60 * 60 + 1;
    });

    // No auth is required by this function — simulating a third party calling it
    // is the same call as the owner calling it, since there's no require_auth gate.
    client.cancel_expired_transfer(&pet_id);

    assert!(!client.has_pending_transfer(&pet_id));
}

#[test]
fn accept_transfer_before_custom_timeout_still_works() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer_with_timeout(&pet_id, &new_owner, &2u32);

    // Accept well within the 2-day window
    client.accept_transfer(&pet_id);

    assert!(!client.has_pending_transfer(&pet_id));
    let escrowed = client.get_escrowed_transfer(&pet_id).unwrap();
    assert_eq!(escrowed.from, owner);
    assert_eq!(escrowed.to, new_owner);
}

#[test]
fn default_initiate_transfer_uses_seven_day_timeout() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.initiate_transfer(&pet_id, &new_owner);

    assert_eq!(
        client.get_transfer_timeout_secs(&pet_id),
        Some(7 * 24 * 60 * 60)
    );
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
fn test_multisig_approval_exactly_at_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let admin_one = Address::generate(&env);
    let admin_two = Address::generate(&env);
    let admin_three = Address::generate(&env);
    let trusted = Address::generate(&env);
    let updated = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(
        &env,
        &[admin_one.clone(), admin_two.clone(), admin_three.clone()],
    );

    // Initialize with threshold = 2 (out of 3 admins)
    client.init_trusted_contract(&trusted, &admins, &2);

    // First approval (1 of 2 threshold) -> should not complete yet
    let res1 = client.update_trusted_contract(&updated, &admin_one);
    assert!(!res1);
    assert_eq!(client.get_trusted_contract_address(), trusted);

    // Second approval (exactly 2 of 2 threshold) -> must complete!
    let res2 = client.update_trusted_contract(&updated, &admin_two);
    assert!(res2);
    assert_eq!(client.get_trusted_contract_address(), updated);
}

#[test]
fn test_multisig_approval_one_below_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let admin_one = Address::generate(&env);
    let admin_two = Address::generate(&env);
    let admin_three = Address::generate(&env);
    let trusted = Address::generate(&env);
    let updated = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(
        &env,
        &[admin_one.clone(), admin_two.clone(), admin_three.clone()],
    );

    // Initialize with threshold = 2 (out of 3 admins)
    client.init_trusted_contract(&trusted, &admins, &2);

    // Submit 1 approval (threshold - 1 = 1) -> must NOT complete
    let completed = client.update_trusted_contract(&updated, &admin_one);
    assert!(!completed);
    assert_eq!(client.get_trusted_contract_address(), trusted);
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
fn trusted_update_approvals_are_keyed_by_proposal_then_approver() {
    let (env, admin_one, admin_two, _) = setup();
    let trusted = Address::generate(&env);
    let proposal = Address::generate(&env);
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let admins = address_vec(&env, &[admin_one.clone(), admin_two.clone()]);

    client.init_trusted_contract(&trusted, &admins, &2);
    assert!(!client.update_trusted_contract(&proposal, &admin_one));

    env.as_contract(&contract_id, || {
        let stored: Option<bool> = env.storage().instance().get(
            &DataKey::TrustedUpdateApproval(TrustedUpdateApprovalKey {
                proposal: proposal.clone(),
                approver: admin_one.clone(),
            }),
        );
        assert_eq!(stored, Some(true));

        // A swapped key must not resolve to the stored approval.
        let swapped: Option<bool> = env.storage().instance().get(
            &DataKey::TrustedUpdateApproval(TrustedUpdateApprovalKey {
                proposal: admin_one.clone(),
                approver: proposal.clone(),
            }),
        );
        assert_eq!(swapped, None);
    });

    // The second approval is collected under the same ordering and
    // reaches the threshold.
    assert!(client.update_trusted_contract(&proposal, &admin_two));
    assert_eq!(client.get_trusted_contract_address(), proposal);
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
    assert_eq!(transfer.timeout_secs, 7 * 24 * 60 * 60);
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

#[test]
fn batch_transfer_moves_all_pets_and_emits_one_event_per_pet() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&1, &owner);
    client.create_pet(&2, &owner);

    let before_events = env.events().all().len();
    let mut ids = Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(2u64);

    client.batch_transfer(&ids, &new_owner);

    assert_eq!(client.get_current_owner(&1), new_owner);
    assert_eq!(client.get_current_owner(&2), new_owner);
    assert_eq!(env.events().all().len(), before_events + 2);
}

#[test]
fn batch_transfer_rejects_owner_mismatch_atomically() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let other_owner = Address::generate(&env);

    client.create_pet(&1, &owner);
    client.create_pet(&2, &other_owner);

    let mut ids = Vec::new(&env);
    ids.push_back(1u64);
    ids.push_back(2u64);

    let result = client.try_batch_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::BatchOwnerMismatch as u32,
        )))
    );
    assert_eq!(client.get_current_owner(&1), owner);
    assert_eq!(client.get_current_owner(&2), other_owner);
}

#[test]
fn batch_transfer_rejects_over_limit() {
    let (env, owner, new_owner, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    let mut ids = Vec::new(&env);
    for pet_id in 1u64..=21u64 {
        client.create_pet(&pet_id, &owner);
        ids.push_back(pet_id);
    }

    let result = client.try_batch_transfer(&ids, &new_owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::BatchTooLarge as u32,
        )))
    );
}

// ======================================================
// adoption multi-party approval tests
// ======================================================

#[test]
fn adoption_without_organization_keeps_two_party_flow() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &None);
    client.approve_adoption(&pet_id, &adopter);
    client.complete_adoption(&pet_id);

    assert_eq!(client.get_current_owner(&pet_id), adopter);
    let record = client.get_adoption_record(&pet_id).unwrap();
    assert_eq!(record.state, AdoptionState::Completed);
    assert!(record.organization.is_none());
}

#[test]
fn adoption_with_organization_requires_org_approval() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let organization = Address::generate(&env);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &Some(organization.clone()));

    // Without adopter approval, complete_adoption must fail
    let blocked = client.try_complete_adoption(&pet_id);
    assert_eq!(
        blocked,
        Err(Ok(Error::from_contract_error(
            ContractError::AdopterApprovalRequired as u32,
        )))
    );

    // Adopter approves but org hasn't — still blocked
    client.approve_adoption(&pet_id, &adopter);
    let blocked = client.try_complete_adoption(&pet_id);
    assert_eq!(
        blocked,
        Err(Ok(Error::from_contract_error(
            ContractError::OrganizationApprovalRequired as u32,
        )))
    );

    // Org approves — now completion succeeds
    client.approve_adoption(&pet_id, &organization);
    client.complete_adoption(&pet_id);

    let record = client.get_adoption_record(&pet_id).unwrap();
    assert_eq!(record.state, AdoptionState::Completed);
    assert_eq!(record.organization, Some(organization));
    assert!(record.organization_approved);
}

#[test]
fn adoption_rejection_cancels_pending_flow() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);
    let organization = Address::generate(&env);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &Some(organization.clone()));
    client.reject_adoption(
        &pet_id,
        &organization,
        &String::from_str(&env, "Rescue declined"),
    );

    assert!(client.get_pending_adoption(&pet_id).is_none());
    let record = client.get_adoption_record(&pet_id).unwrap();
    assert_eq!(record.state, AdoptionState::Rejected);
    assert_eq!(record.rejected_by, Some(organization));
}

#[test]
fn reject_adoption_rejects_oversized_reason() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &None);

    let long_reason = String::from_str(&env, &"x".repeat(MAX_REJECTION_REASON_LEN as usize + 1));
    let result = client.try_reject_adoption(&pet_id, &owner, &long_reason);

    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::InputStringTooLong as u32,
        )))
    );
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

// ======================================================
// update_adoption_config tests (Issue #1007)
// ======================================================

#[test]
fn set_adoption_config_stores_waiting_period_and_admin() {
    let (env, owner, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.set_adoption_config(&7u32, &owner);

    let config = client.get_adoption_config();
    assert_eq!(config.waiting_period_days, 7);
}

#[test]
fn set_adoption_config_is_one_shot_second_call_is_rejected() {
    let (env, owner, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.set_adoption_config(&7u32, &owner);

    // Second call must be rejected
    let result = client.try_set_adoption_config(&14u32, &owner);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::AdoptionNotConfigurable as u32,
        )))
    );
    // Value must remain unchanged
    assert_eq!(client.get_adoption_config().waiting_period_days, 7);
}

#[test]
fn set_species_adoption_config_requires_admin_auth() {
    let (env, owner, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.set_adoption_config(&7u32, &owner);
    env.mock_auths(&[]);

    let result = client.try_set_species_adoption_config(&String::from_str(&env, "dog"), &14u32);

    assert!(result.is_err());
}

#[test]
fn update_adoption_config_changes_waiting_period() {
    let (env, owner, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.set_adoption_config(&7u32, &owner);
    client.update_adoption_config(&14u32);

    assert_eq!(client.get_adoption_config().waiting_period_days, 14);
}

#[test]
fn update_adoption_config_emits_event_with_old_and_new_values() {
    let (env, owner, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.set_adoption_config(&7u32, &owner);
    let events_before = env.events().all().len();

    client.update_adoption_config(&14u32);

    // At least one new event must have been emitted
    assert!(env.events().all().len() > events_before);
}

#[test]
fn update_adoption_config_new_adoption_uses_new_period() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    // Set initial period to 7 days and then update to 0 (instant)
    client.set_adoption_config(&7u32, &owner);
    client.update_adoption_config(&0u32);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &None);
    client.approve_adoption(&pet_id, &adopter);

    // With 0-day waiting period, complete_adoption should succeed immediately
    client.complete_adoption(&pet_id);
    assert_eq!(client.get_current_owner(&pet_id), adopter);
}

#[test]
fn update_adoption_config_fails_without_prior_set() {
    let (env, _, _, _) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    // No set_adoption_config called — update must fail
    let result = client.try_update_adoption_config(&14u32);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::AdoptionConfigNotFound as u32,
        )))
    );
}

// ======================================================
// adopter_approved guard tests (Issue #1008)
// ======================================================

#[test]
fn complete_adoption_without_adopter_approval_is_rejected() {
// cancel_expired_adoption tests (Issue #1009)
// ======================================================

#[test]
fn cancel_expired_adoption_before_expiry_is_rejected() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    // Owner signs adoption — adopter_approved is false
    client.sign_adoption(&pet_id, &adopter, &None);

    // Adopter calls complete_adoption without first calling approve_adoption
    let result = client.try_complete_adoption(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::AdopterApprovalRequired as u32,
        )))
    );

    // Pet ownership must not have changed
    assert_eq!(client.get_current_owner(&pet_id), owner);
    client.sign_adoption(&pet_id, &adopter, &None);

    // Well within the 30-day window — must fail
    env.ledger().with_mut(|l| {
        l.timestamp += 29 * 24 * 60 * 60;
    });

    let result = client.try_cancel_expired_adoption(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::AdoptionNotExpired as u32,
        )))
    );
    assert!(client.get_pending_adoption(&pet_id).is_some());
}

#[test]
fn cancel_expired_adoption_after_expiry_succeeds() {
    let (env, owner, adopter, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);
    client.sign_adoption(&pet_id, &adopter, &None);

    env.ledger().with_mut(|l| {
        l.timestamp += 30 * 24 * 60 * 60 + 1;
    });

    client.cancel_expired_adoption(&pet_id);

    // Pending adoption is cleared and the owner can re-list the pet
    assert!(client.get_pending_adoption(&pet_id).is_none());
    assert_eq!(client.get_current_owner(&pet_id), owner);
    client.sign_adoption(&pet_id, &adopter, &None);
}

#[test]
fn cancel_expired_adoption_without_pending_adoption_is_rejected() {
    let (env, owner, _adopter, pet_id) = setup();
// Custody chain length cap tests (Issue #1011)
// ======================================================

/// Runs one full direct transfer cycle, leaving `to` as the current owner.
fn transfer_once(
    env: &Env,
    client: &PetOwnershipContractClient,
    pet_id: &u64,
    to: &Address,
) {
    // 250+ transfers in one test would otherwise exhaust the test budget.
    env.budget().reset_unlimited();
    client.initiate_transfer(pet_id, to);
    client.accept_transfer(pet_id);
    env.ledger().with_mut(|l| {
        l.timestamp += DISPUTE_WINDOW_SECONDS + 1;
    });
    client.finalize_transfer(pet_id);
}

#[test]
fn custody_chain_is_capped_at_max_length() {
    let (env, owner, new_owner, pet_id) = setup();
    let contract_id = env.register_contract(None, PetOwnershipContract);
    let client = PetOwnershipContractClient::new(&env, &contract_id);

    client.create_pet(&pet_id, &owner);

    let result = client.try_cancel_expired_adoption(&pet_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::NoPendingAdoption as u32,
        )))
    );
    // Exactly MAX_CUSTODY_CHAIN_LENGTH transfers — nothing is dropped yet.
    for i in 0..MAX_CUSTODY_CHAIN_LENGTH {
        let to = if i % 2 == 0 { &new_owner } else { &owner };
        transfer_once(&env, &client, &pet_id, to);
    }
    assert_eq!(client.get_custody_chain(&pet_id).len(), MAX_CUSTODY_CHAIN_LENGTH);

    let second_entry = client.get_custody_chain(&pet_id).get(1).unwrap();

    // One more transfer trims the oldest entry instead of growing the Vec.
    transfer_once(&env, &client, &pet_id, &new_owner);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), MAX_CUSTODY_CHAIN_LENGTH);
    // The previous second entry is now first — the oldest one was dropped.
    assert_eq!(chain.get(0).unwrap(), second_entry);
    // The newest transfer is retained at the tail.
    let newest = chain.get(MAX_CUSTODY_CHAIN_LENGTH - 1).unwrap();
    assert_eq!(newest.to, new_owner);
}
