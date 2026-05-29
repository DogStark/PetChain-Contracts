#[cfg(test)]
mod test_activity {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);

        let owner = Address::generate(&env);
        let client = PetChainContractClient::new(&env, &contract_id);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Retriever"),
            &PrivacyLevel::Public,
        );

        (env, contract_id, owner, pet_id)
    }

    #[test]
    fn test_bbox_query_returns_subset() {
        let (env, contract_id, _owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);

        let first = client.add_activity_record(
            &pet_id,
            &String::from_str(&env, "Walk"),
            &String::from_str(&env, "Morning walk"),
            &37_774_900,
            &-122_419_400,
        );
        let _second = client.add_activity_record(
            &pet_id,
            &String::from_str(&env, "Park"),
            &String::from_str(&env, "Park time"),
            &37_780_000,
            &-122_420_000,
        );
        let _third = client.add_activity_record(
            &pet_id,
            &String::from_str(&env, "Vet"),
            &String::from_str(&env, "Clinic visit"),
            &40_000_000,
            &-100_000_000,
        );

        let results = client.get_activities_in_bbox(
            &pet_id,
            &37_770_000,
            &37_780_500,
            &-122_421_000,
            &-122_418_000,
        );

        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().id, first);
        assert_eq!(
            results.get(1).unwrap().activity_type,
            String::from_str(&env, "Park")
        );
    }

    #[test]
    #[should_panic(expected = "Latitude out of range")]
    fn test_out_of_range_coordinates_rejected() {
        let (env, contract_id, _owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);

        client.add_activity_record(
            &pet_id,
            &String::from_str(&env, "Walk"),
            &String::from_str(&env, "Bad latitude"),
            &91_000_000,
            &10_000_000,
        );
    }

    #[test]
    #[should_panic(expected = "Coordinates cannot be zero")]
    fn test_zero_coordinate_rejected() {
        let (env, contract_id, _owner, pet_id) = setup();
        let client = PetChainContractClient::new(&env, &contract_id);

        client.add_activity_record(
            &pet_id,
            &String::from_str(&env, "Walk"),
            &String::from_str(&env, "Unset coordinates"),
            &0,
            &10_000_000,
        );
    }
}

use crate::{
    ActivityType, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

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

#[test]
fn test_get_activity_record_by_id_returns_correct_record() {
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

    let record_id = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Morning walk"),
    );

    let record = client.get_activity_record_by_id(&record_id).unwrap();
    assert_eq!(record.id, record_id);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.activity_type, ActivityType::Walk);
    assert_eq!(record.duration_minutes, 30);
    assert_eq!(record.intensity, 5);
    assert_eq!(record.distance_meters, 2000);
}

#[test]
fn test_get_activity_record_by_id_returns_none_for_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let result = client.get_activity_record_by_id(&9999);
    assert!(result.is_none());
}

// --- ACTIVITY STREAK TRACKING TESTS ---

#[test]
fn test_streak_increments_on_consecutive_days() {
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

    // Day 1: Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Day 1 walk"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1);
    assert_eq!(streak.longest_streak, 1);

    // Day 2: Advance time by 1 day and add activity
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 86400;
    });

    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Day 2 run"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 2);
    assert_eq!(streak.longest_streak, 2);

    // Day 3: Advance time by 1 day and add activity
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 86400;
    });

    client.add_activity_record(
        &pet_id,
        &ActivityType::Play,
        &45,
        &6,
        &0,
        &String::from_str(&env, "Day 3 play"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 3);
    assert_eq!(streak.longest_streak, 3);
}

#[test]
fn test_streak_resets_on_gap_greater_than_one_day() {
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

    // Day 1: Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Day 1 walk"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1);

    // Day 2: Advance time by 2 days (gap > 1 day) and add activity
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 172800; // 2 days
    });

    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Day 3 run"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1); // Reset to 1
    assert_eq!(streak.longest_streak, 1); // Longest remains 1
}

#[test]
fn test_milestone_event_at_7_days() {
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

    // Add activities for 7 consecutive days
    for day in 0..7 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Daily walk"),
        );

        if day < 6 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 7);
    assert!(client.has_reached_milestone(&pet_id, &7));
}

#[test]
fn test_milestone_event_at_30_days() {
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

    // Add activities for 30 consecutive days
    for day in 0..30 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Daily walk"),
        );

        if day < 29 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 30);
    assert!(client.has_reached_milestone(&pet_id, &30));
}

#[test]
fn test_milestone_event_at_100_days() {
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

    // Add activities for 100 consecutive days
    for day in 0..100 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Daily walk"),
        );

        if day < 99 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 100);
    assert!(client.has_reached_milestone(&pet_id, &100));
}

#[test]
fn test_longest_streak_tracking() {
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

    // Build a 5-day streak
    for day in 0..5 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Walk"),
        );

        if day < 4 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 5);
    assert_eq!(streak.longest_streak, 5);

    // Break the streak with a 2-day gap
    env.ledger().with_mut(|ledger| {
        ledger.timestamp += 172800; // 2 days
    });

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1); // Reset to 1
    assert_eq!(streak.longest_streak, 5); // Longest remains 5

    // Build a new 8-day streak
    for day in 0..7 {
        env.ledger().with_mut(|ledger| {
            ledger.timestamp += 86400;
        });

        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Walk"),
        );
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 8);
    assert_eq!(streak.longest_streak, 8); // Updated to 8
}

#[test]
fn test_get_current_streak() {
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

    // Initially no streak
    assert_eq!(client.get_current_streak(&pet_id), 0);

    // Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    assert_eq!(client.get_current_streak(&pet_id), 1);
}

#[test]
fn test_get_longest_streak() {
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

    // Initially no streak
    assert_eq!(client.get_longest_streak(&pet_id), 0);

    // Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    assert_eq!(client.get_longest_streak(&pet_id), 1);
}

#[test]
fn test_same_day_activity_no_streak_change() {
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

    // Add first activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk 1"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1);

    // Add second activity on same day (no time advance)
    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Run 1"),
    );

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 1); // No change
}

#[test]
fn test_milestone_not_reached_before_threshold() {
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

    // Add activities for 6 days (below 7-day milestone)
    for day in 0..6 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Walk"),
        );

        if day < 5 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    assert!(!client.has_reached_milestone(&pet_id, &7));
    assert!(!client.has_reached_milestone(&pet_id, &30));
    assert!(!client.has_reached_milestone(&pet_id, &100));
}

#[test]
fn test_multiple_milestones_reached() {
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

    // Add activities for 30 consecutive days
    for day in 0..30 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Walk"),
        );

        if day < 29 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    // Should have reached both 7 and 30 day milestones
    assert!(client.has_reached_milestone(&pet_id, &7));
    assert!(client.has_reached_milestone(&pet_id, &30));
    assert!(!client.has_reached_milestone(&pet_id, &100));
}

#[test]
fn test_streak_with_multiple_pets() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet1_id = client.register_pet(
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

    let pet2_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    // Pet 1: 5-day streak
    for day in 0..5 {
        client.add_activity_record(
            &pet1_id,
            &ActivityType::Walk,
            &30,
            &5,
            &2000,
            &String::from_str(&env, "Walk"),
        );

        if day < 4 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    // Pet 2: 3-day streak (different timeline)
    for day in 0..3 {
        client.add_activity_record(
            &pet2_id,
            &ActivityType::Play,
            &20,
            &4,
            &0,
            &String::from_str(&env, "Play"),
        );

        if day < 2 {
            env.ledger().with_mut(|ledger| {
                ledger.timestamp += 86400;
            });
        }
    }

    let streak1 = client.get_activity_streak(&pet1_id);
    let streak2 = client.get_activity_streak(&pet2_id);

    assert_eq!(streak1.current_streak, 5);
    assert_eq!(streak2.current_streak, 3);
}

#[test]
fn test_streak_persistence_across_calls() {
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

    // Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &2000,
        &String::from_str(&env, "Walk"),
    );

    // Get streak multiple times
    let streak1 = client.get_activity_streak(&pet_id);
    let streak2 = client.get_activity_streak(&pet_id);

    assert_eq!(streak1.current_streak, streak2.current_streak);
    assert_eq!(streak1.longest_streak, streak2.longest_streak);
}

#[test]
fn test_streak_after_gap_resets_to_one() {
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

    // Day 1: Add activity
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Walk"),
    );

    let streak_day1 = client.get_activity_streak(&pet_id);
    assert_eq!(streak_day1.current_streak, 1);

    // Day 2: Add activity (consecutive)
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &87400, // 1 day + 1 second later
        &String::from_str(&env, "Walk"),
    );

    let streak_day2 = client.get_activity_streak(&pet_id);
    assert_eq!(streak_day2.current_streak, 2);

    // Day 4: Add activity (gap > 1 day)
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &261200, // 3 days + 1 second later
        &String::from_str(&env, "Walk"),
    );

    let streak_day4 = client.get_activity_streak(&pet_id);
    assert_eq!(streak_day4.current_streak, 1); // Reset to 1
    assert_eq!(streak_day4.longest_streak, 2); // Longest still 2
}

#[test]
fn test_milestone_events_not_duplicated() {
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

    // Add 7 consecutive days of activity
    for day in 0..7 {
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &((1000 + (day as u64 * 86400)) as u32),
            &String::from_str(&env, "Walk"),
        );
    }

    let streak = client.get_activity_streak(&pet_id);
    assert_eq!(streak.current_streak, 7);
    assert!(streak.milestones_reached.contains(&7));

    // Add one more day - should not emit duplicate 7-day event
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &((1000 + (7 as u64 * 86400)) as u32),
        &String::from_str(&env, "Walk"),
    );

    let streak_after = client.get_activity_streak(&pet_id);
    assert_eq!(streak_after.current_streak, 8);
    // Milestone vector should still only have one entry for 7 days
    let milestone_count = streak_after
        .milestones_reached
        .iter()
        .filter(|m| *m == 7)
        .count();
    assert_eq!(milestone_count, 1);
}

// --- 5 NEW TESTS ---

/// Streak resets to 1 when a gap of exactly 2 days separates two activities,
/// and the previous longest streak is preserved correctly.
#[test]
fn test_streak_reset_preserves_longest() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Streak"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    // Build a 4-day streak
    for day in 0..4u64 {
        env.ledger().with_mut(|l| l.timestamp = 1_000 + day * 86_400);
        client.add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "Walk"),
        );
    }

    let after_four = client.get_activity_streak(&pet_id);
    assert_eq!(after_four.current_streak, 4);
    assert_eq!(after_four.longest_streak, 4);

    // Skip a day (gap = 2 days) then add one activity
    env.ledger().with_mut(|l| l.timestamp = 1_000 + 6 * 86_400);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &6,
        &500,
        &String::from_str(&env, "Run after gap"),
    );

    let after_reset = client.get_activity_streak(&pet_id);
    assert_eq!(after_reset.current_streak, 1);
    assert_eq!(after_reset.longest_streak, 4); // longest must not shrink
}

/// `get_activity_history` returns records in insertion order across
/// different activity types.
#[test]
fn test_activity_history_order_preserved() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Order"),
        &String::from_str(&env, "2021-06-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &4,
        &None,
        &PrivacyLevel::Public,
    );

    let types = [
        ActivityType::Training,
        ActivityType::Play,
        ActivityType::Walk,
        ActivityType::Run,
        ActivityType::Other,
    ];

    for t in types.iter() {
        client.add_activity_record(
            &pet_id,
            t,
            &10,
            &3,
            &100,
            &String::from_str(&env, "note"),
        );
    }

    let history = client.get_activity_history(&pet_id);
    assert_eq!(history.len(), 5);
    for (i, t) in types.iter().enumerate() {
        assert_eq!(history.get(i as u32).unwrap().activity_type, *t);
    }
}

/// `get_activity_stats` only counts records within the requested day window
/// and ignores older records.
#[test]
fn test_activity_stats_respects_day_window() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Window"),
        &String::from_str(&env, "2019-03-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Tricolor"),
        &12,
        &None,
        &PrivacyLevel::Public,
    );

    // Record at t=0 (old, outside a 7-day window from t=30*86400)
    env.ledger().with_mut(|l| l.timestamp = 0);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &60,
        &5,
        &5000,
        &String::from_str(&env, "Old walk"),
    );

    // Record at t=29*86400 (inside a 7-day window from t=30*86400)
    env.ledger().with_mut(|l| l.timestamp = 29 * 86_400);
    client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &20,
        &7,
        &1500,
        &String::from_str(&env, "Recent run"),
    );

    // Query with days=7 from current time t=30*86400
    env.ledger().with_mut(|l| l.timestamp = 30 * 86_400);
    let (duration, distance) = client.get_activity_stats(&pet_id, &7);

    // Only the recent run should be counted
    assert_eq!(duration, 20);
    assert_eq!(distance, 1500);
}

/// Adding activities for two different pets does not cross-contaminate
/// their individual streaks.
#[test]
fn test_streaks_are_isolated_per_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_a = client.register_pet(
        &owner,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "Gray"),
        &28,
        &None,
        &PrivacyLevel::Public,
    );

    let pet_b = client.register_pet(
        &owner,
        &String::from_str(&env, "Beta"),
        &String::from_str(&env, "2021-05-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    // Pet A: 3 consecutive days
    for day in 0..3u64 {
        env.ledger().with_mut(|l| l.timestamp = 1_000 + day * 86_400);
        client.add_activity_record(
            &pet_a,
            &ActivityType::Walk,
            &30,
            &5,
            &1000,
            &String::from_str(&env, "A walk"),
        );
    }

    // Pet B: only 1 day (same timestamps as pet A day 0)
    env.ledger().with_mut(|l| l.timestamp = 1_000);
    client.add_activity_record(
        &pet_b,
        &ActivityType::Play,
        &15,
        &3,
        &0,
        &String::from_str(&env, "B play"),
    );

    let streak_a = client.get_activity_streak(&pet_a);
    let streak_b = client.get_activity_streak(&pet_b);

    assert_eq!(streak_a.current_streak, 3);
    assert_eq!(streak_b.current_streak, 1);
    assert_eq!(streak_a.longest_streak, 3);
    assert_eq!(streak_b.longest_streak, 1);
}

/// `get_activity_record_by_id` returns the correct record when multiple
/// records exist, and returns `None` for an id that was never created.
#[test]
fn test_get_activity_record_by_id_multiple_records() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Multi"),
        &String::from_str(&env, "2022-02-14"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "Apricot"),
        &8,
        &None,
        &PrivacyLevel::Public,
    );

    let id1 = client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &4,
        &2000,
        &String::from_str(&env, "First"),
    );

    let id2 = client.add_activity_record(
        &pet_id,
        &ActivityType::Run,
        &15,
        &8,
        &1200,
        &String::from_str(&env, "Second"),
    );

    let id3 = client.add_activity_record(
        &pet_id,
        &ActivityType::Training,
        &45,
        &6,
        &0,
        &String::from_str(&env, "Third"),
    );

    // Each id resolves to the correct record
    let r1 = client.get_activity_record_by_id(&id1).unwrap();
    assert_eq!(r1.activity_type, ActivityType::Walk);
    assert_eq!(r1.duration_minutes, 30);

    let r2 = client.get_activity_record_by_id(&id2).unwrap();
    assert_eq!(r2.activity_type, ActivityType::Run);
    assert_eq!(r2.distance_meters, 1200);

    let r3 = client.get_activity_record_by_id(&id3).unwrap();
    assert_eq!(r3.activity_type, ActivityType::Training);
    assert_eq!(r3.intensity, 6);

    // An id that was never issued returns None
    let missing_id = id3 + 999;
    assert!(client.get_activity_record_by_id(&missing_id).is_none());
}
