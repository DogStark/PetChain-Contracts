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
