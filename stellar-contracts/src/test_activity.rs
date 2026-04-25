use crate::{
    ActivityType, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

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

#[test]
fn test_get_treatment_history_pagination() {
    use crate::{TreatmentType};

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&owner, &vet);

    // Add 3 treatments
    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Routine,
        &1000u64,
        &String::from_str(&env, "Checkup"),
        &None,
        &String::from_str(&env, "Good"),
    );
    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Surgery,
        &2000u64,
        &String::from_str(&env, "Spay"),
        &None,
        &String::from_str(&env, "Successful"),
    );
    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Emergency,
        &3000u64,
        &String::from_str(&env, "Injury"),
        &None,
        &String::from_str(&env, "Recovered"),
    );

    // Get all (offset=0, limit=10)
    let all = client.get_treatment_history(&pet_id, &0u64, &10u32);
    assert_eq!(all.len(), 3);
    assert_eq!(all.get(0).unwrap().treatment_type, TreatmentType::Routine);
    assert_eq!(all.get(1).unwrap().treatment_type, TreatmentType::Surgery);
    assert_eq!(all.get(2).unwrap().treatment_type, TreatmentType::Emergency);

    // Pagination: offset=1, limit=2 → 2nd and 3rd
    let page = client.get_treatment_history(&pet_id, &1u64, &2u32);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().treatment_type, TreatmentType::Surgery);
    assert_eq!(page.get(1).unwrap().treatment_type, TreatmentType::Emergency);

    // Offset beyond count → empty
    let empty = client.get_treatment_history(&pet_id, &10u64, &5u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_get_treatment_history_nonexistent_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    // Non-existent pet should return empty vec
    let result = client.get_treatment_history(&999u64, &0u64, &10u32);
    assert_eq!(result.len(), 0);
}
