use crate::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn setup_env() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "TestPet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "TestBreed"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, admin, owner, pet_id)
}

// --- BASIC DUPLICATE DETECTION ---

#[test]
fn test_first_activity_succeeds() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    let record_id = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    assert!(record_id > 0);
}

#[test]
#[should_panic(expected = "DuplicateActivity")]
fn test_duplicate_activity_within_window_rejected() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    // Try to add duplicate immediately (should fail)
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );
}

#[test]
fn test_duplicate_activity_after_window_succeeds() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add first activity
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    // Advance time by 61 seconds (past default 60-second window)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(61);
    });

    // Add same activity again (should succeed)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    assert!(id2 > id1);
}

// --- DIFFERENT ACTIVITY TYPES ---

#[test]
fn test_different_activity_types_allowed() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add Walk activity
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    // Add Run activity immediately (different type, should succeed)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &8,
        &2000,
        &String::from_str(&env, "Morning run"),
    );

    assert!(id2 > id1);
}

#[test]
fn test_all_activity_types_independent() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add one of each activity type
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &8,
        &2000,
        &String::from_str(&env, "Run"),
    );

    let id3 = client.add_activity_record(
        &pet_id,
        &ActivityType::Play,
        &45,
        &6,
        &500,
        &String::from_str(&env, "Play"),
    );

    let id4 = client.add_activity_record(
        &pet_id,
        &ActivityType::Training,
        &60,
        &4,
        &0,
        &String::from_str(&env, "Training"),
    );

    let id5 = client.add_activity_record(
        &pet_id,
        &ActivityType::Other,
        &15,
        &3,
        &100,
        &String::from_str(&env, "Other"),
    );

    assert!(id5 > id4 && id4 > id3 && id3 > id2 && id2 > id1);
}

// --- DIFFERENT PETS ---

#[test]
fn test_different_pets_independent() {
    let (env, client, _admin, owner, pet_id1) = setup_env();

    // Register second pet
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "TestPet2"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "TestBreed2"),
        &String::from_str(&env, "White"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Add same activity for both pets
    let id1 = client.add_activity_record(
        &pet_id1,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    let id2 = client.add_activity_record(
        &pet_id2,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    assert!(id2 > id1);
}

// --- CONFIGURABLE WINDOW ---

#[test]
fn test_set_idempotency_window() {
    let (env, client, admin, _owner, pet_id) = setup_env();

    // Set window to 120 seconds
    client.set_activity_idempotency_window(&admin, &120);

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Advance time by 61 seconds (would be past default 60s window)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(61);
    });

    // Try to add duplicate (should still fail because window is 120s)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "Walk"),
        );
    }));

    assert!(result.is_err());
}

#[test]
fn test_custom_window_expiration() {
    let (env, client, admin, _owner, pet_id) = setup_env();

    // Set window to 30 seconds
    client.set_activity_idempotency_window(&admin, &30);

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Advance time by 31 seconds (past 30s window)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(31);
    });

    // Add duplicate (should succeed)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    assert!(id2 > 0);
}

#[test]
#[should_panic]
fn test_set_window_requires_admin() {
    let (env, client, _admin, _owner, _pet_id) = setup_env();
    let non_admin = Address::generate(&env);

    // Non-admin should not be able to set window
    client.set_activity_idempotency_window(&non_admin, &120);
}

// --- EDGE CASES ---

#[test]
fn test_duplicate_at_exact_window_boundary() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Advance time by exactly 60 seconds
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(60);
    });

    // Try to add duplicate at exact boundary (should still fail)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "Walk"),
        );
    }));

    assert!(result.is_err());
}

#[test]
fn test_multiple_duplicates_within_window() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Try multiple duplicates
    for _ in 0..3 {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.add_activity_record(
                &pet_id,
                &ActivityType::Walk,
                &30,
                &5,
                &1000,
                &String::from_str(&env, "Walk"),
            );
        }));
        assert!(result.is_err());
    }
}

#[test]
fn test_window_zero_allows_immediate_duplicate() {
    let (env, client, admin, _owner, pet_id) = setup_env();

    // Set window to 0 seconds (effectively disables idempotency check)
    client.set_activity_idempotency_window(&admin, &0);

    // Add first activity
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Add duplicate immediately (should succeed with window=0)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    assert!(id2 > id1);
}

// --- IDEMPOTENCY KEY GENERATION ---

#[test]
fn test_same_timestamp_different_types_different_keys() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add Walk
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Activity"),
    );

    // Add Run at same timestamp (different type)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Activity"),
    );

    // Both should succeed
    assert!(id2 > id1);
}

#[test]
#[should_panic(expected = "DuplicateActivity")]
fn test_same_key_components_duplicate_detected() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "First walk"),
    );

    // Add with same pet_id, activity_type, timestamp but different other params
    // Should still be detected as duplicate
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &60,  // Different duration
        &8,   // Different intensity
        &2000, // Different distance
        &String::from_str(&env, "Second walk"), // Different notes
    );
}

// --- REAL-WORLD SCENARIOS ---

#[test]
fn test_rapid_fire_different_activities() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Simulate rapid activity logging of different types
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Play,
        &15,
        &7,
        &0,
        &String::from_str(&env, "Play"),
    );

    let id3 = client.add_activity_record(
        &pet_id,
        &ActivityType::Training,
        &45,
        &4,
        &0,
        &String::from_str(&env, "Training"),
    );

    assert!(id3 > id2 && id2 > id1);
}

#[test]
fn test_activity_sequence_with_time_gaps() {
    let (env, client, _admin, _owner, pet_id) = setup_env();

    // Add activity
    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    // Wait 70 seconds
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(70);
    });

    // Add same activity (should succeed)
    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Afternoon walk"),
    );

    // Wait another 70 seconds
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp.saturating_add(70);
    });

    // Add same activity again (should succeed)
    let id3 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Evening walk"),
    );

    assert!(id3 > id2 && id2 > id1);
}

#[test]
fn test_concurrent_activities_different_pets() {
    let (env, client, _admin, owner, pet_id1) = setup_env();

    // Register second pet
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Pet2"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Breed2"),
        &String::from_str(&env, "White"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Add same activity for both pets at same time
    let id1 = client.add_activity_record(
        &pet_id1,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    let id2 = client.add_activity_record(
        &pet_id2,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    // Try duplicates for each pet (should both fail)
    let result1 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_activity_record(
            &pet_id1,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "Walk"),
        );
    }));

    let result2 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.add_activity_record(
            &pet_id2,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "Walk"),
        );
    }));

    assert!(result1.is_err());
    assert!(result2.is_err());
    assert!(id2 > id1);
}
