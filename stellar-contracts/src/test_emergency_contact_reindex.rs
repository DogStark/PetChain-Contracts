//! Regression tests for issue #1641: emergency-contact priorities must stay
//! contiguous and deduplicated after a contact is removed.

use crate::*;
use soroban_sdk::{testutils::Address as _, Env};

fn contact(
    env: &Env,
    name: &str,
    phone: &str,
    is_primary: bool,
    priority: u32,
) -> EmergencyContact {
    EmergencyContact {
        name: String::from_str(env, name),
        phone: String::from_str(env, phone),
        email: String::from_str(env, ""),
        relationship: String::from_str(env, "Owner"),
        is_primary,
        priority,
    }
}

fn setup() -> (Env, PetChainContractClient<'static>, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    (env, client, owner, pet_id)
}

/// Seeds three contacts at priorities 1, 2 and 3 in reverse insertion order so
/// the tests also prove that removal works on the priority-ordered view.
fn seed_contacts(env: &Env, client: &PetChainContractClient, pet_id: u64) {
    let mut contacts = soroban_sdk::Vec::new(env);
    contacts.push_back(contact(env, "Third", "555-0003", false, 3));
    contacts.push_back(contact(env, "Second", "555-0002", false, 2));
    contacts.push_back(contact(env, "First", "555-0001", true, 1));
    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(env),
        &String::from_str(env, ""),
    );
}

fn ordered_names(env: &Env, client: &PetChainContractClient, owner: &Address, pet_id: u64) -> soroban_sdk::Vec<String> {
    client
        .get_contacts_ordered(&pet_id, owner)
        .iter()
        .map(|entry| entry.name)
        .collect()
}

#[test]
fn removing_middle_contact_reindexes_remaining_priorities() {
    let (env, client, owner, pet_id) = setup();
    seed_contacts(&env, &client, pet_id);

    // Priority-ordered view is First(1), Second(2), Third(3); drop Second.
    client.remove_emergency_contact(&pet_id, &1);

    let remaining = client.get_contacts_ordered(&pet_id, &owner);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining.get(0).unwrap().priority, 1);
    assert_eq!(remaining.get(1).unwrap().priority, 2);
    assert_eq!(
        ordered_names(&env, &client, &owner, pet_id),
        soroban_sdk::Vec::from_array(&env, [String::from_str(&env, "First"), String::from_str(&env, "Third")])
    );
    assert_eq!(remaining.get(0).unwrap().is_primary, true);
}

#[test]
fn removing_first_contact_shifts_other_contacts_down() {
    let (env, client, owner, pet_id) = setup();
    seed_contacts(&env, &client, pet_id);

    client.remove_emergency_contact(&pet_id, &0);

    let remaining = client.get_contacts_ordered(&pet_id, &owner);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining.get(0).unwrap().priority, 1);
    assert_eq!(remaining.get(1).unwrap().priority, 2);
    assert_eq!(
        ordered_names(&env, &client, &owner, pet_id),
        soroban_sdk::Vec::from_array(&env, [String::from_str(&env, "Second"), String::from_str(&env, "Third")])
    );
    // The primary contact was removed, so the new first contact is promoted.
    assert_eq!(remaining.get(0).unwrap().is_primary, true);
    assert_eq!(remaining.get(1).unwrap().is_primary, false);
}

#[test]
fn removing_last_contact_decrements_len_without_reindexing_others() {
    let (env, client, owner, pet_id) = setup();
    seed_contacts(&env, &client, pet_id);

    client.remove_emergency_contact(&pet_id, &2);

    let remaining = client.get_contacts_ordered(&pet_id, &owner);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining.get(0).unwrap().priority, 1);
    assert_eq!(remaining.get(1).unwrap().priority, 2);
    assert_eq!(
        ordered_names(&env, &client, &owner, pet_id),
        soroban_sdk::Vec::from_array(&env, [String::from_str(&env, "First"), String::from_str(&env, "Second")])
    );
}

#[test]
fn duplicate_emergency_contact_is_rejected_without_changing_state() {
    let (env, client, owner, pet_id) = setup();
    seed_contacts(&env, &client, pet_id);

    let mut with_duplicate = soroban_sdk::Vec::new(&env);
    with_duplicate.push_back(contact(&env, "First", "555-0001", true, 1));
    with_duplicate.push_back(contact(&env, "First", "555-0001", false, 2));

    let err = client
        .try_set_emergency_contacts(
            &pet_id,
            &with_duplicate,
            &soroban_sdk::Vec::new(&env),
            &String::from_str(&env, ""),
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::InvalidInput.into());

    // Failed calls are atomic: the original three contacts remain untouched.
    let contacts = client.get_contacts_ordered(&pet_id, &owner);
    assert_eq!(contacts.len(), 3);
    assert_eq!(contacts.get(0).unwrap().priority, 1);
    assert_eq!(contacts.get(1).unwrap().priority, 2);
    assert_eq!(contacts.get(2).unwrap().priority, 3);
}

#[test]
fn removing_only_contact_is_rejected() {
    let (env, client, owner, pet_id) = setup();
    seed_contacts(&env, &client, pet_id);
    client.remove_emergency_contact(&pet_id, &0);
    client.remove_emergency_contact(&pet_id, &0);

    let err = client
        .try_remove_emergency_contact(&pet_id, &0)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::InvalidInput.into());

    let contacts = client.get_contacts_ordered(&pet_id, &owner);
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts.get(0).unwrap().priority, 1);
}
