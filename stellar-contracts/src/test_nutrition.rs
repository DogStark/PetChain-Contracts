use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

#[test]
fn test_set_and_get_diet_plan() {
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

    let mut restrictions = Vec::new(&env);
    restrictions.push_back(String::from_str(&env, "No corn"));

    let mut allergies = Vec::new(&env);
    allergies.push_back(String::from_str(&env, "Chicken"));

    let ok = client.set_diet_plan(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions,
        &allergies,
    );

    assert!(ok);

    let history = client.get_diet_history(&pet_id);
    assert_eq!(history.len(), 1);
    let plan = history.get(0).unwrap();
    assert_eq!(plan.pet_id, pet_id);
    assert_eq!(plan.food_type, String::from_str(&env, "Dry Kibble"));
}

#[test]
fn test_weight_entries_and_pet_update() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );

    let ok1 = client.add_weight_entry(&pet_id, &7u32);
    assert!(ok1);
    let ok2 = client.add_weight_entry(&pet_id, &8u32);
    assert!(ok2);

    let w_history = client.get_weight_history(&pet_id);
    assert_eq!(w_history.len(), 2);

    let profile = client.get_pet(&pet_id, &owner).unwrap();
    assert_eq!(profile.weight, 8u32);
}

#[test]
fn test_get_medications_pagination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Register and verify vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Add 3 medications
    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Amoxicillin"),
        &String::from_str(&env, "250mg"),
        &String::from_str(&env, "Twice daily"),
        &1000u64,
        &None,
        &vet,
    );
    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Prednisone"),
        &String::from_str(&env, "10mg"),
        &String::from_str(&env, "Once daily"),
        &2000u64,
        &None,
        &vet,
    );
    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Metronidazole"),
        &String::from_str(&env, "500mg"),
        &String::from_str(&env, "Three times daily"),
        &3000u64,
        &None,
        &vet,
    );

    // Get all medications (offset=0, limit=10)
    let all = client.get_medications(&pet_id, &0u64, &10u32);
    assert_eq!(all.len(), 3);
    assert_eq!(
        all.get(0).unwrap().name,
        String::from_str(&env, "Amoxicillin")
    );
    assert_eq!(
        all.get(1).unwrap().name,
        String::from_str(&env, "Prednisone")
    );
    assert_eq!(
        all.get(2).unwrap().name,
        String::from_str(&env, "Metronidazole")
    );

    // Pagination: offset=1, limit=2 → should return 2nd and 3rd
    let page = client.get_medications(&pet_id, &1u64, &2u32);
    assert_eq!(page.len(), 2);
    assert_eq!(
        page.get(0).unwrap().name,
        String::from_str(&env, "Prednisone")
    );
    assert_eq!(
        page.get(1).unwrap().name,
        String::from_str(&env, "Metronidazole")
    );

    // Offset beyond count → empty
    let empty = client.get_medications(&pet_id, &10u64, &5u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_get_active_medications_filter() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2020-07-15"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "Feline"),
    );
    client.verify_vet(&admin, &vet);

    // Add two medications
    let med1_id = client.add_medication(
        &pet_id,
        &String::from_str(&env, "Doxycycline"),
        &String::from_str(&env, "100mg"),
        &String::from_str(&env, "Once daily"),
        &1000u64,
        &None,
        &vet,
    );
    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Furosemide"),
        &String::from_str(&env, "20mg"),
        &String::from_str(&env, "Twice daily"),
        &2000u64,
        &None,
        &vet,
    );

    // Both should be active initially
    let active = client.get_active_medications(&pet_id);
    assert_eq!(active.len(), 2);

    // Mark first medication as completed (inactive)
    client.mark_medication_completed(&med1_id);

    // Now only one should be active
    let active_after = client.get_active_medications(&pet_id);
    assert_eq!(active_after.len(), 1);
    assert_eq!(
        active_after.get(0).unwrap().name,
        String::from_str(&env, "Furosemide")
    );
    assert!(active_after.get(0).unwrap().active);

    // get_medications still returns all (including inactive)
    let all = client.get_medications(&pet_id, &0u64, &10u32);
    assert_eq!(all.len(), 2);
}

#[test]
fn test_discontinue_medication() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let med_id = client.add_medication(
        &pet_id,
        &String::from_str(&env, "Amoxicillin"),
        &String::from_str(&env, "250mg"),
        &String::from_str(&env, "Twice daily"),
        &1000u64,
        &None,
        &vet,
    );

    let end_date = 5000u64;
    client.discontinue_medication(&med_id, &end_date, &vet);

    let all = client.get_medications(&pet_id, &0, &1);
    let med = all.get(0).unwrap();
    assert!(!med.active);
    assert_eq!(med.end_date, Some(end_date));
}

#[test]
fn test_get_diet_plan_count() {
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

    // Initially zero
    assert_eq!(client.get_diet_plan_count(&pet_id), 0);

    let restrictions = Vec::new(&env);
    let allergies = Vec::new(&env);

    // Add first diet plan
    client.set_diet_plan(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions,
        &allergies,
    );
    assert_eq!(client.get_diet_plan_count(&pet_id), 1);

    // Add second diet plan
    client.set_diet_plan(
        &pet_id,
        &String::from_str(&env, "Wet Food"),
        &String::from_str(&env, "150g"),
        &String::from_str(&env, "Three times daily"),
        &restrictions,
        &allergies,
    );
    assert_eq!(client.get_diet_plan_count(&pet_id), 2);

    // Count for a non-existent pet returns 0
    assert_eq!(client.get_diet_plan_count(&9999u64), 0);
}

#[test]
fn test_get_weight_entry_by_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bolt"),
        &String::from_str(&env, "2022-06-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "White"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_weight_entry(&pet_id, &21u32);
    client.add_weight_entry(&pet_id, &22u32);

    // weight_id 1 should exist and have weight 21
    let entry = client.get_weight_entry(&1u64);
    assert!(entry.is_some());
    let e = entry.unwrap();
    assert_eq!(e.pet_id, pet_id);
    assert_eq!(e.weight, 21u32);

    // weight_id 2 should exist and have weight 22
    let entry2 = client.get_weight_entry(&2u64);
    assert!(entry2.is_some());
    assert_eq!(entry2.unwrap().weight, 22u32);
}

#[test]
fn test_get_weight_entry_nonexistent_returns_none() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // No entries added — ID 999 should return None
    let result = client.get_weight_entry(&999u64);
    assert!(result.is_none());
}

// --- VERSIONED NUTRITION TESTS ---

#[test]
fn test_set_nutrition_version_creates_version() {
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

    let mut restrictions = Vec::new(&env);
    restrictions.push_back(String::from_str(&env, "No corn"));

    let mut allergies = Vec::new(&env);
    allergies.push_back(String::from_str(&env, "Chicken"));

    let version = client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions,
        &allergies,
    );

    assert_eq!(version, 1u64);

    let retrieved = client.get_nutrition_version(&pet_id, &version).unwrap();
    assert_eq!(retrieved.version, 1u64);
    assert_eq!(retrieved.food_type, String::from_str(&env, "Dry Kibble"));
    assert_eq!(retrieved.portion_size, String::from_str(&env, "200g"));
    assert_eq!(retrieved.is_active, true);
}

#[test]
fn test_nutrition_version_history_preserved() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut restrictions1 = Vec::new(&env);
    restrictions1.push_back(String::from_str(&env, "No corn"));

    let mut allergies1 = Vec::new(&env);
    allergies1.push_back(String::from_str(&env, "Chicken"));

    // Create version 1
    let v1 = client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions1,
        &allergies1,
    );
    assert_eq!(v1, 1u64);

    let mut restrictions2 = Vec::new(&env);
    restrictions2.push_back(String::from_str(&env, "No wheat"));

    let mut allergies2 = Vec::new(&env);
    allergies2.push_back(String::from_str(&env, "Fish"));

    // Create version 2
    let v2 = client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Wet Food"),
        &String::from_str(&env, "150g"),
        &String::from_str(&env, "Three times daily"),
        &restrictions2,
        &allergies2,
    );
    assert_eq!(v2, 2u64);

    // Verify both versions exist
    let version1 = client.get_nutrition_version(&pet_id, &1u64).unwrap();
    assert_eq!(version1.food_type, String::from_str(&env, "Dry Kibble"));
    assert_eq!(version1.is_active, false); // Previous version should be inactive

    let version2 = client.get_nutrition_version(&pet_id, &2u64).unwrap();
    assert_eq!(version2.food_type, String::from_str(&env, "Wet Food"));
    assert_eq!(version2.is_active, true); // Current version should be active
}

#[test]
fn test_list_nutrition_versions_returns_all_versions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2019-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // Create 5 versions
    for i in 1..=5 {
        let food_type = if i % 2 == 0 {
            String::from_str(&env, "Wet Food")
        } else {
            String::from_str(&env, "Dry Kibble")
        };

        client.set_nutrition_version(
            &pet_id,
            &food_type,
            &String::from_str(&env, "200g"),
            &String::from_str(&env, "Twice daily"),
            &empty_restrictions,
            &empty_allergies,
        );
    }

    let versions = client.list_nutrition_versions(&pet_id);
    assert_eq!(versions.len(), 5);

    // Verify newest version is first (reverse order)
    assert_eq!(versions.get(0).unwrap().version, 5u64);
    assert_eq!(versions.get(4).unwrap().version, 1u64);

    // Verify only latest is active
    assert_eq!(versions.get(0).unwrap().is_active, true);
    for i in 1..5 {
        assert_eq!(versions.get(i).unwrap().is_active, false);
    }
}

#[test]
fn test_rollback_nutrition_restores_correct_state() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Charlie"),
        &String::from_str(&env, "2018-07-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Brown"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // Create version 1
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    // Create version 2
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Wet Food"),
        &String::from_str(&env, "150g"),
        &String::from_str(&env, "Three times daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    // Create version 3
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Mixed Diet"),
        &String::from_str(&env, "175g"),
        &String::from_str(&env, "Twice daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    // Verify current is version 3
    let current = client.get_current_nutrition_version(&pet_id).unwrap();
    assert_eq!(current.version, 3u64);
    assert_eq!(current.food_type, String::from_str(&env, "Mixed Diet"));

    // Rollback to version 1
    let rollback_version = client.rollback_nutrition(&pet_id, &1u64);
    assert_eq!(rollback_version, 4u64); // New version created

    // Verify new version 4 has version 1's data
    let new_current = client.get_current_nutrition_version(&pet_id).unwrap();
    assert_eq!(new_current.version, 4u64);
    assert_eq!(new_current.food_type, String::from_str(&env, "Dry Kibble"));
    assert_eq!(new_current.portion_size, String::from_str(&env, "200g"));
    assert_eq!(new_current.is_active, true);

    // Verify version 3 is now inactive
    let v3 = client.get_nutrition_version(&pet_id, &3u64).unwrap();
    assert_eq!(v3.is_active, false);
}

#[test]
fn test_nutrition_version_pruning_at_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Daisy"),
        &String::from_str(&env, "2017-09-22"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // Create 12 versions (exceeds 10 limit)
    for i in 1..=12 {
        let food_type = format!(&env, "Food Type {}", i);
        client.set_nutrition_version(
            &pet_id,
            &food_type,
            &String::from_str(&env, "200g"),
            &String::from_str(&env, "Twice daily"),
            &empty_restrictions,
            &empty_allergies,
        );
    }

    // Verify version 1 is pruned (oldest)
    let v1 = client.get_nutrition_version(&pet_id, &1u64);
    assert!(v1.is_none());

    // Verify version 2 still exists (oldest after pruning)
    let v2 = client.get_nutrition_version(&pet_id, &2u64);
    assert!(v2.is_some());

    // Verify version 12 exists (newest)
    let v12 = client.get_nutrition_version(&pet_id, &12u64);
    assert!(v12.is_some());

    // Verify list returns only 10 most recent versions
    let versions = client.list_nutrition_versions(&pet_id);
    assert_eq!(versions.len(), 10);
    assert_eq!(versions.get(0).unwrap().version, 12u64); // Newest first
    assert_eq!(versions.get(9).unwrap().version, 3u64);  // Oldest in list (v1 and v2 pruned)
}

#[test]
fn test_get_current_nutrition_version_returns_active() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rocky"),
        &String::from_str(&env, "2016-11-08"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "German Shepherd"),
        &String::from_str(&env, "Brown"),
        &35u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // No version yet
    let current = client.get_current_nutrition_version(&pet_id);
    assert!(current.is_none());

    // Create version 1
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Premium Kibble"),
        &String::from_str(&env, "300g"),
        &String::from_str(&env, "Twice daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    let current = client.get_current_nutrition_version(&pet_id).unwrap();
    assert_eq!(current.version, 1u64);
    assert_eq!(current.food_type, String::from_str(&env, "Premium Kibble"));

    // Create version 2
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Organic Food"),
        &String::from_str(&env, "280g"),
        &String::from_str(&env, "Twice daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    let current = client.get_current_nutrition_version(&pet_id).unwrap();
    assert_eq!(current.version, 2u64);
    assert_eq!(current.food_type, String::from_str(&env, "Organic Food"));
}

#[test]
fn test_nutrition_version_nonexistent_pet_returns_none() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // Try to get version for non-existent pet
    let result = client.get_nutrition_version(&999u64, &1u64);
    assert!(result.is_none());

    let versions = client.list_nutrition_versions(&999u64);
    assert_eq!(versions.len(), 0);

    let current = client.get_current_nutrition_version(&999u64);
    assert!(current.is_none());
}

#[test]
fn test_rollback_to_nonexistent_version_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2015-02-14"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // Create version 1
    client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Cat Food"),
        &String::from_str(&env, "100g"),
        &String::from_str(&env, "Twice daily"),
        &empty_restrictions,
        &empty_allergies,
    );

    // Try to rollback to non-existent version 5
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.rollback_nutrition(&pet_id, &5u64);
    }));
    assert!(result.is_err());
}

#[test]
fn test_nutrition_version_with_restrictions_and_allergies() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2014-04-30"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Bulldog"),
        &String::from_str(&env, "Fawn"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut restrictions = Vec::new(&env);
    restrictions.push_back(String::from_str(&env, "No corn"));
    restrictions.push_back(String::from_str(&env, "No soy"));
    restrictions.push_back(String::from_str(&env, "No wheat"));

    let mut allergies = Vec::new(&env);
    allergies.push_back(String::from_str(&env, "Chicken"));
    allergies.push_back(String::from_str(&env, "Beef"));
    allergies.push_back(String::from_str(&env, "Dairy"));

    let version = client.set_nutrition_version(
        &pet_id,
        &String::from_str(&env, "Hypoallergenic Food"),
        &String::from_str(&env, "250g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions,
        &allergies,
    );

    let retrieved = client.get_nutrition_version(&pet_id, &version).unwrap();
    assert_eq!(retrieved.dietary_restrictions.len(), 3);
    assert_eq!(retrieved.allergies.len(), 3);
    assert_eq!(
        retrieved.dietary_restrictions.get(0).unwrap(),
        &String::from_str(&env, "No corn")
    );
    assert_eq!(
        retrieved.allergies.get(0).unwrap(),
        &String::from_str(&env, "Chicken")
    );
}

#[test]
fn test_multiple_rollbacks_create_new_versions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Zoe"),
        &String::from_str(&env, "2013-06-18"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Bengal"),
        &String::from_str(&env, "Orange"),
        &7u32,
        &None,
        &PrivacyLevel::Public,
    );

    let empty_restrictions = Vec::new(&env);
    let empty_allergies = Vec::new(&env);

    // Create versions 1, 2, 3
    for i in 1..=3 {
        let food = format!(&env, "Food {}", i);
        client.set_nutrition_version(
            &pet_id,
            &food,
            &String::from_str(&env, "100g"),
            &String::from_str(&env, "Twice daily"),
            &empty_restrictions,
            &empty_allergies,
        );
    }

    // Rollback to version 1 (creates version 4)
    let rb1 = client.rollback_nutrition(&pet_id, &1u64);
    assert_eq!(rb1, 4u64);

    // Rollback to version 2 (creates version 5)
    let rb2 = client.rollback_nutrition(&pet_id, &2u64);
    assert_eq!(rb2, 5u64);

    // Verify version 5 has version 2's data
    let v5 = client.get_nutrition_version(&pet_id, &5u64).unwrap();
    assert_eq!(v5.food_type, String::from_str(&env, "Food 2"));

    // Verify all versions exist
    let versions = client.list_nutrition_versions(&pet_id);
    assert_eq!(versions.len(), 5);
}
