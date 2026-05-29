use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_env() -> (Env, PetChainContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    client.init_admin(&admin);

    (env, client, admin, owner)
}

fn register_test_pet(
    client: &PetChainContractClient,
    env: &Env,
    owner: &Address,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "TestBreed"),
        &String::from_str(env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

fn register_vet(
    client: &PetChainContractClient,
    env: &Env,
    admin: &Address,
) -> Address {
    let vet = Address::generate(env);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Test"),
        &String::from_str(env, "LIC-TEST-001"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(admin, &vet);
    vet
}

// --- BASIC QUOTA FUNCTIONALITY ---

#[test]
fn test_get_storage_usage_returns_zero_for_new_pet() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.pet_id, pet_id);
    assert_eq!(usage.current_count, 0);
    assert_eq!(usage.quota, 1000); // DEFAULT_STORAGE_QUOTA
}

#[test]
fn test_storage_usage_increments_on_medical_record() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Add a medical record
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis"),
        &String::from_str(&env, "Treatment"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_vaccination() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 31536000), // +1 year
        &String::from_str(&env, "BATCH123"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_behavior_record() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    client.add_behavior_record(
        &pet_id,
        &BehaviorType::Training,
        &5,
        &String::from_str(&env, "Good behavior"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_activity_record() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        &30,
        &5,
        &1000,
        &String::from_str(&env, "Morning walk"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_training_milestone() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    client.add_training_milestone(
        &pet_id,
        &String::from_str(&env, "Sit command"),
        &String::from_str(&env, "Learned to sit"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_medication() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Aspirin"),
        &String::from_str(&env, "10mg"),
        &String::from_str(&env, "Daily"),
        &env.ledger().timestamp(),
        &None,
        &vet,
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_grooming_record() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    client.add_grooming_record(
        &pet_id,
        &String::from_str(&env, "Bath"),
        &String::from_str(&env, "Groomer Name"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 2592000), // +30 days
        &50,
        &String::from_str(&env, "Full grooming"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_lab_result() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
        &String::from_str(&env, "Reference ranges"),
        &None,
        &None,
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_breeding_record() {
    let (env, client, _admin, owner) = setup_env();
    let sire_id = register_test_pet(&client, &env, &owner);
    let dam_id = register_test_pet(&client, &env, &owner);

    client.add_breeding_record(
        &sire_id,
        &dam_id,
        &env.ledger().timestamp(),
        &String::from_str(&env, "Breeding notes"),
    );

    // Both sire and dam should have incremented storage
    let sire_usage = client.get_storage_usage(&sire_id);
    assert_eq!(sire_usage.current_count, 1);

    let dam_usage = client.get_storage_usage(&dam_id);
    assert_eq!(dam_usage.current_count, 1);
}

#[test]
fn test_storage_usage_increments_on_insurance_policy() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-001"),
        &String::from_str(&env, "InsureCo"),
        &String::from_str(&env, "Basic"),
        &100,
        &10000,
        &(env.ledger().timestamp() + 31536000), // +1 year
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 1);
}

// --- QUOTA ENFORCEMENT ---

#[test]
fn test_storage_usage_tracks_multiple_entries() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Add multiple different types of records
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 1"),
    );

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Vaccine"),
        &env.ledger().timestamp(),
        &(env.ledger().timestamp() + 31536000),
        &String::from_str(&env, "BATCH"),
    );

    client.add_behavior_record(
        &pet_id,
        &BehaviorType::Training,
        &5,
        &String::from_str(&env, "Behavior"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 3);
}

#[test]
#[should_panic(expected = "StorageQuotaExceeded")]
fn test_write_rejected_when_quota_exceeded() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Set a very low quota
    client.set_pet_storage_quota(&admin, &pet_id, &2);

    // Add 2 records - should succeed
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 1"),
    );

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 2"),
    );

    // Third record should panic with StorageQuotaExceeded
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 3"),
        &String::from_str(&env, "Treatment 3"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 3"),
    );
}

// --- ADMIN QUOTA MANAGEMENT ---

#[test]
fn test_set_global_storage_quota() {
    let (env, client, admin, owner) = setup_env();

    // Set global quota to 500
    client.set_global_storage_quota(&admin, &500);

    // New pet should have the new global quota
    let pet_id = register_test_pet(&client, &env, &owner);
    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.quota, 500);
}

#[test]
fn test_set_pet_storage_quota_override() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);

    // Default quota
    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.quota, 1000);

    // Set custom quota for this pet
    client.set_pet_storage_quota(&admin, &pet_id, &2000);

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.quota, 2000);
}

#[test]
fn test_per_pet_quota_overrides_global() {
    let (env, client, admin, owner) = setup_env();

    // Set global quota to 100
    client.set_global_storage_quota(&admin, &100);

    let pet_id = register_test_pet(&client, &env, &owner);

    // Should have global quota
    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.quota, 100);

    // Set per-pet override to 500
    client.set_pet_storage_quota(&admin, &pet_id, &500);

    // Should now have per-pet quota
    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.quota, 500);
}

#[test]
#[should_panic]
fn test_set_global_quota_requires_admin() {
    let (env, client, _admin, owner) = setup_env();
    let non_admin = Address::generate(&env);

    // Non-admin should not be able to set global quota
    client.set_global_storage_quota(&non_admin, &500);
}

#[test]
#[should_panic]
fn test_set_pet_quota_requires_admin() {
    let (env, client, _admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let non_admin = Address::generate(&env);

    // Non-admin should not be able to set pet quota
    client.set_pet_storage_quota(&non_admin, &pet_id, &500);
}

#[test]
#[should_panic(expected = "PetNotFound")]
fn test_set_pet_quota_for_nonexistent_pet() {
    let (_env, client, admin, _owner) = setup_env();

    // Should panic when setting quota for non-existent pet
    client.set_pet_storage_quota(&admin, &99999, &500);
}

#[test]
#[should_panic(expected = "PetNotFound")]
fn test_get_storage_usage_for_nonexistent_pet() {
    let (_env, client, _admin, _owner) = setup_env();

    // Should panic when querying non-existent pet
    client.get_storage_usage(&99999);
}

// --- QUOTA BOUNDARY TESTS ---

#[test]
fn test_write_at_quota_limit_succeeds() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Set quota to exactly 3
    client.set_pet_storage_quota(&admin, &pet_id, &3);

    // Add exactly 3 records - all should succeed
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 1"),
    );

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 2"),
    );

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 3"),
        &String::from_str(&env, "Treatment 3"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 3"),
    );

    let usage = client.get_storage_usage(&pet_id);
    assert_eq!(usage.current_count, 3);
    assert_eq!(usage.quota, 3);
}

#[test]
#[should_panic(expected = "StorageQuotaExceeded")]
fn test_write_one_over_quota_fails() {
    let (env, client, admin, owner) = setup_env();
    let pet_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Set quota to 3
    client.set_pet_storage_quota(&admin, &pet_id, &3);

    // Add 3 records
    for i in 1..=3 {
        client.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, &format!("Diagnosis {}", i)),
            &String::from_str(&env, &format!("Treatment {}", i)),
            &Vec::new(&env),
            &String::from_str(&env, &format!("Notes {}", i)),
        );
    }

    // 4th record should fail
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis 4"),
        &String::from_str(&env, "Treatment 4"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 4"),
    );
}

// --- MULTI-PET ISOLATION ---

#[test]
fn test_quota_isolated_per_pet() {
    let (env, client, admin, owner) = setup_env();
    let pet1_id = register_test_pet(&client, &env, &owner);
    let pet2_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Set different quotas
    client.set_pet_storage_quota(&admin, &pet1_id, &2);
    client.set_pet_storage_quota(&admin, &pet2_id, &5);

    // Add records to pet1
    client.add_medical_record(
        &pet1_id,
        &vet,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 1"),
    );

    // Add records to pet2
    client.add_medical_record(
        &pet2_id,
        &vet,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 2"),
    );

    // Check usage is independent
    let usage1 = client.get_storage_usage(&pet1_id);
    assert_eq!(usage1.current_count, 1);
    assert_eq!(usage1.quota, 2);

    let usage2 = client.get_storage_usage(&pet2_id);
    assert_eq!(usage2.current_count, 1);
    assert_eq!(usage2.quota, 5);
}

#[test]
fn test_quota_enforcement_independent_per_pet() {
    let (env, client, admin, owner) = setup_env();
    let pet1_id = register_test_pet(&client, &env, &owner);
    let pet2_id = register_test_pet(&client, &env, &owner);
    let vet = register_vet(&client, &env, &admin);

    // Set pet1 quota to 1
    client.set_pet_storage_quota(&admin, &pet1_id, &1);

    // Fill pet1's quota
    client.add_medical_record(
        &pet1_id,
        &vet,
        &String::from_str(&env, "Diagnosis 1"),
        &String::from_str(&env, "Treatment 1"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 1"),
    );

    // pet2 should still be able to add records (has default quota)
    client.add_medical_record(
        &pet2_id,
        &vet,
        &String::from_str(&env, "Diagnosis 2"),
        &String::from_str(&env, "Treatment 2"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes 2"),
    );

    let usage2 = client.get_storage_usage(&pet2_id);
    assert_eq!(usage2.current_count, 1);
}
