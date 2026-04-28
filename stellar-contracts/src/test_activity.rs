use crate::{
    ActivityType, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, String};

#[test]
fn test_add_activity_record() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let activity_id = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Morning walk in the park"),
    );

    assert_eq!(activity_id, 1);
}

#[test]
fn test_get_activity_history() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Morning walk"),
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &15,
        &8,
        &1500,
        &String::from_str(&env, "Evening run"),
    );

    let history = client.get_activity_history(&pet_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().activity_type, ActivityType::Walk);
    assert_eq!(history.get(1).unwrap().activity_type, ActivityType::Run);
}

#[test]
fn test_activity_stats() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk 1"),
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &8,
        &1500,
        &String::from_str(&env, "Run 1"),
    );

    let (total_duration, total_distance) = client.get_activity_stats(&pet_id, &7);
    assert_eq!(total_duration, 50);
    assert_eq!(total_distance, 3500);
}

#[test]
fn test_multiple_activity_types() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &15,
        &8,
        &1500,
        &String::from_str(&env, "Run"),
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Play,
        &45,
        &6,
        &0,
        &String::from_str(&env, "Play time"),
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Training,
        &20,
        &4,
        &0,
        &String::from_str(&env, "Training session"),
    );

    let history = client.get_activity_history(&pet_id);
    assert_eq!(history.len(), 4);
}

#[test]
#[should_panic]
fn test_add_activity_nonexistent_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    client.add_activity_record(
        &999,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );
}

#[test]
#[should_panic]
fn test_invalid_intensity() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &15,
        &2000,
        &String::from_str(&env, "Walk"),
    );
}

#[test]
fn test_activity_stats_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let (total_duration, total_distance) = client.get_activity_stats(&pet_id, &7);
    assert_eq!(total_duration, 0);
    assert_eq!(total_distance, 0);
}

#[test]
fn test_archive_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.activate_pet(&pet_id);
    assert_eq!(client.get_active_pets_count(), 1);

    client.archive_pet(&pet_id);

    // Archived pet should not be active
    assert!(!client.is_pet_active(&pet_id));
    // Active count should decrease
    assert_eq!(client.get_active_pets_count(), 0);
    // Archived pet excluded from get_active_pets
    let active = client.get_active_pets();
    assert_eq!(active.len(), 0);
}

#[test]
fn test_unarchive_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.archive_pet(&pet_id);
    client.unarchive_pet(&pet_id);

    // After unarchive, pet is no longer archived (active state unchanged)
    assert!(!client.is_pet_active(&pet_id));
    // Can re-activate after unarchiving
    client.activate_pet(&pet_id);
    assert!(client.is_pet_active(&pet_id));
}

#[test]
fn test_archive_decrements_active_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "White"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    client.activate_pet(&pet_id);
    assert_eq!(client.get_active_pets_count(), 1);

    client.archive_pet(&pet_id);
    assert_eq!(client.get_active_pets_count(), 0);
}

#[test]
#[should_panic]
fn test_archive_nonexistent_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    client.archive_pet(&999);
}

// ── get_activity_summary tests ────────────────────────────────────────────────

/// Helper: register a pet and return its id.
fn setup_pet(env: &Env, client: &PetChainContractClient) -> u64 {
    let owner = Address::generate(env);
    client.init_admin(&owner);
    client.register_pet(
        &owner,
        &String::from_str(env, "Max"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Golden Retriever"),
        &String::from_str(env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_activity_summary_valid_range() {
    // Activities within the range should be summed correctly.
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let pet_id = setup_pet(&env, &client);

    // Both records land at timestamp 1_000 (current ledger time).
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );
    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Run"),
    );

    let (duration, distance) = client.get_activity_summary(&pet_id, &500, &2000);
    assert_eq!(duration, 50);
    assert_eq!(distance, 3500);
}

#[test]
fn test_activity_summary_partial_overlap() {
    // Only records whose timestamp falls inside [from, to] should be counted.
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // Record 1 at t=100
    env.ledger().with_mut(|l| l.timestamp = 100);
    let pet_id = setup_pet(&env, &client);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Inside range"),
    );

    // Record 2 at t=5000 – outside the query range below
    env.ledger().with_mut(|l| l.timestamp = 5000);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Outside range"),
    );

    // Query only covers t=0..=200
    let (duration, distance) = client.get_activity_summary(&pet_id, &0, &200);
    assert_eq!(duration, 30);
    assert_eq!(distance, 2000);
}

#[test]
fn test_activity_summary_empty_range() {
    // No activities exist in the queried window → (0, 0).
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 9999);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let pet_id = setup_pet(&env, &client);

    client.add_activity_record(
        &pet_id,
        &ActivityType::Play,
        &45,
        &3,
        &0,
        &String::from_str(&env, "Play"),
    );

    // Query a window that doesn't include t=9999
    let (duration, distance) = client.get_activity_summary(&pet_id, &0, &100);
    assert_eq!(duration, 0);
    assert_eq!(distance, 0);
}

#[test]
fn test_activity_summary_single_activity_on_boundary() {
    // A record exactly on from_date or to_date must be included (inclusive).
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 500);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let pet_id = setup_pet(&env, &client);

    client.add_activity_record(
        &pet_id,
        &ActivityType::Training,
        &60,
        &4,
        &3000,
        &String::from_str(&env, "Boundary activity"),
    );

    // Exact lower boundary
    let (d1, dist1) = client.get_activity_summary(&pet_id, &500, &500);
    assert_eq!(d1, 60);
    assert_eq!(dist1, 3000);

    // Exact upper boundary
    let (d2, dist2) = client.get_activity_summary(&pet_id, &0, &500);
    assert_eq!(d2, 60);
    assert_eq!(dist2, 3000);
}

#[test]
fn test_activity_summary_invalid_range() {
    // from_date > to_date → (0, 0) without panicking.
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let pet_id = setup_pet(&env, &client);

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    let (duration, distance) = client.get_activity_summary(&pet_id, &2000, &500);
    assert_eq!(duration, 0);
    assert_eq!(distance, 0);
}
