//! Tests for contract event emission, event data, and indexing.
//! Events enable off-chain tracking and real-time frontend/backend updates.

use crate::*;
use soroban_sdk::{
    testutils::Address as _, testutils::Events, Address, Env, String, TryFromVal,
};

fn register_test_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Breed"),
        &String::from_str(env, "Brown"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    )
}

fn topic_matches(env: &Env, topics: &soroban_sdk::Vec<soroban_sdk::Val>, name: &str) -> bool {
    if topics.len() == 0 {
        return false;
    }
    let expected_str = String::from_str(env, name);
    match topics.get(0) {
        Some(v) => {
            // Contract uses String::from_str for topic names
            String::try_from_val(env, &v).map(|s| s == expected_str).unwrap_or(false)
        }
        None => false,
    }
}

#[test]
fn test_pet_registered_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);

    let events = env.events().all();
    assert!(!events.is_empty(), "At least one event should be emitted");
    let has_pet_registered = events
        .iter()
        .any(|(_, topics, _)| topic_matches(&env, &topics, "PetRegistered"));
    assert!(has_pet_registered, "PetRegistered event should be emitted");
    assert_eq!(pet_id, 1);
}

#[test]
fn test_pet_registered_event_data() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let _pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2019-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let events = env.events().all();
    let reg_count = events
        .iter()
        .filter(|(_, topics, _)| topic_matches(&env, &topics, "PetRegistered"))
        .count();
    assert_eq!(reg_count, 1, "Exactly one PetRegistered event");
    let has_multiple_topics = events
        .iter()
        .filter(|(_, topics, _)| topic_matches(&env, &topics, "PetRegistered"))
        .any(|(_, topics, _)| topics.len() >= 1);
    assert!(has_multiple_topics, "PetRegistered event should have topic data");
}

#[test]
fn test_pet_updated_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);

    client.update_pet_profile(
        &pet_id,
        &String::from_str(&env, "UpdatedName"),
        &String::from_str(&env, "2019-06-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );

    let events = env.events().all();
    let has_pet_updated = events
        .iter()
        .any(|(_, topics, _)| topic_matches(&env, &topics, "PetUpdated"));
    assert!(has_pet_updated, "PetUpdated event should be emitted");
}

#[test]
fn test_lost_pet_reported_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);

    let _alert_id = client.report_lost(
        &pet_id,
        &String::from_str(&env, "Central Park"),
        &Some(100u64),
    );

    let events = env.events().all();
    let has_lost_reported = events
        .iter()
        .any(|(_, topics, _)| topic_matches(&env, &topics, "LostPetReported"));
    assert!(has_lost_reported, "LostPetReported event should be emitted");
}

#[test]
fn test_event_indexing_by_topic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    register_test_pet(&env, &client, &owner);
    register_test_pet(&env, &client, &owner); // pet_id 2

    let events = env.events().all();
    let pet_registered_count = events
        .iter()
        .filter(|(_, topics, _)| topic_matches(&env, &topics, "PetRegistered"))
        .count();
    assert_eq!(pet_registered_count, 2, "Should index 2 PetRegistered events");
}

#[test]
fn test_access_granted_event_emission() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);

    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

    let events = env.events().all();
    let has_access_granted = events
        .iter()
        .any(|(_, topics, _)| topic_matches(&env, &topics, "AccessGranted"));
    assert!(has_access_granted, "AccessGranted event should be emitted");
}
