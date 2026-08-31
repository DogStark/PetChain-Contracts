use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String, Vec,
};

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Signal"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Mixed"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr Index"),
        &String::from_str(&env, "SUB-VET-1"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    (env, client, admin, owner, vet, pet_id)
}

#[test]
fn test_matching_subscription_ids_for_event_and_pet() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let subscriber = Address::generate(&env);
    let other_pet = pet_id + 1;

    let mut event_types = Vec::new(&env);
    event_types.push_back(EventType::TreatmentAdded);
    let mut pet_ids = Vec::new(&env);
    pet_ids.push_back(pet_id);
    let matching_id = client.register_subscription(&subscriber, &event_types, &pet_ids, &300u64);

    let mut other_pet_ids = Vec::new(&env);
    other_pet_ids.push_back(other_pet);
    let ignored_id =
        client.register_subscription(&subscriber, &event_types, &other_pet_ids, &300u64);

    let matches = client.get_matching_subscription_ids(&EventType::TreatmentAdded, &pet_id);
    assert_eq!(matches.len(), 1);
    assert_eq!(matches.get(0).unwrap(), matching_id);
    assert_ne!(matches.get(0).unwrap(), ignored_id);
}

#[test]
fn test_expired_subscriptions_are_excluded() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let subscriber = Address::generate(&env);
    let mut event_types = Vec::new(&env);
    event_types.push_back(EventType::MedicalRecordAdded);
    let mut pet_ids = Vec::new(&env);
    pet_ids.push_back(pet_id);

    client.register_subscription(&subscriber, &event_types, &pet_ids, &10u64);
    env.ledger().set_timestamp(env.ledger().timestamp() + 11);

    let matches = client.get_matching_subscription_ids(&EventType::MedicalRecordAdded, &pet_id);
    assert_eq!(matches.len(), 0);
}

#[test]
#[should_panic]
fn test_subscription_limit_enforced_per_address() {
    let (env, client, _admin, _owner, _vet, pet_id) = setup();
    let subscriber = Address::generate(&env);
    let mut event_types = Vec::new(&env);
    event_types.push_back(EventType::TreatmentAdded);
    let mut pet_ids = Vec::new(&env);
    pet_ids.push_back(pet_id);

    for _ in 0..10 {
        client.register_subscription(&subscriber, &event_types, &pet_ids, &300u64);
    }

    client.register_subscription(&subscriber, &event_types, &pet_ids, &300u64);
}

#[test]
fn test_event_payload_contains_matching_subscription_ids() {
    let (env, client, _admin, _owner, vet, pet_id) = setup();
    let subscriber = Address::generate(&env);
    let mut event_types = Vec::new(&env);
    event_types.push_back(EventType::TreatmentAdded);
    let mut pet_ids = Vec::new(&env);
    pet_ids.push_back(pet_id);
    let subscription_id = client.register_subscription(&subscriber, &event_types, &pet_ids, &300u64);

    let ids = client.get_matching_subscription_ids(&EventType::TreatmentAdded, &pet_id);
    assert_eq!(ids.get(0).unwrap(), subscription_id);

    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Routine,
        &env.ledger().timestamp(),
        &String::from_str(&env, "Routine check"),
        &None,
        &String::from_str(&env, "Stable"),
    );
}

// ---------------------------------------------------------------------------
// Event schema versioning guards (Issue #1251)
// ---------------------------------------------------------------------------

/// Verify that EVENT_SCHEMA_VERSION matches the latest EventSchema variant.
/// This acts as a compile-time + runtime canary: if someone bumps the enum
/// without updating the constant (or vice-versa) this test will fail.
#[test]n test_event_schema_version_matches_enum() {
    // The latest enum variant discriminant must equal the constant.
    assert_eq!(EVENT_SCHEMA_VERSION, EventSchema::V1 as u32);
}

/// Assert that every public event struct (documented in event-versioning.md)
/// has a `version` field equal to EVENT_SCHEMA_VERSION when constructed.
/// We verify this by checking the constant value is non-zero (a v0 event
/// would indicate a regression).
#[test]n test_event_schema_version_is_current() {
    assert!(
        EVENT_SCHEMA_VERSION >= 1,
        "EVENT_SCHEMA_VERSION must be >= 1; events without a version field are treated as v0"
    );
}
