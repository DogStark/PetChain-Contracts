use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn register_test_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Buddy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Golden Retriever"),
        &String::from_str(env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_register_pet_and_get_profile() {
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
        &Some(String::from_str(&env, "982000123456789")),
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.owner, owner);
    assert_eq!(pet.name, String::from_str(&env, "Buddy"));
    assert_eq!(pet.breed, String::from_str(&env, "Golden Retriever"));
    assert_eq!(pet.color, String::from_str(&env, "Golden"));
    assert_eq!(pet.weight, 25u32);
}

#[test]
fn test_grant_custody() {
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

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "feed"));
    permissions.push_back(String::from_str(&env, "walk"));

    let custody =
        client.grant_temporary_custody(&pet_id, &custodian, &100u64, &200u64, &permissions);

    assert!(custody.is_active);
    assert_eq!(custody.pet_id, pet_id);
    assert_eq!(custody.owner, owner);
    assert_eq!(custody.custodian, custodian);
    assert_eq!(custody.permissions.len(), 2);
}

#[test]
fn test_auto_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    env.ledger().with_mut(|l| l.timestamp = 1000);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "feed"));

    client.grant_temporary_custody(&pet_id, &custodian, &900u64, &1100u64, &permissions);
    assert!(client.is_custody_valid(&pet_id));

    env.ledger().with_mut(|l| l.timestamp = 1200);
    assert!(!client.is_custody_valid(&pet_id));
}

#[test]
fn test_limited_permissions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2022-06-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Tricolor"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "medicate"));

    let custody = client.grant_temporary_custody(&pet_id, &custodian, &10u64, &20u64, &permissions);

    assert_eq!(custody.permissions.len(), 1);
    assert_eq!(
        custody.permissions.get(0).unwrap(),
        String::from_str(&env, "medicate")
    );
}

#[test]
fn test_log_all_actions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let grantee = Address::generate(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let meds = Vec::new(&env);
    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "No treatment"),
        &meds,
    );
    client.get_medical_record(&record_id);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);
    client.revoke_access(&pet_id, &grantee);

    let logs = client.get_access_logs(&pet_id);
    assert!(logs.len() >= 4);

    let mut has_read = false;
    let mut has_write = false;
    let mut has_grant = false;
    let mut has_revoke = false;
    for log in logs.iter() {
        match log.action {
            AccessAction::Read => has_read = true,
            AccessAction::Write => has_write = true,
            AccessAction::Grant => has_grant = true,
            AccessAction::Revoke => has_revoke = true,
        }
    }

    assert!(has_read);
    assert!(has_write);
    assert!(has_grant);
    assert!(has_revoke);
}

#[test]
fn test_log_immutability() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let meds = Vec::new(&env);
    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Stable"),
        &String::from_str(&env, "Observe"),
        &meds,
    );
    let logs_before = client.get_access_logs(&pet_id);
    let initial_count = logs_before.len();

    client.get_medical_record(&record_id);
    let logs_after = client.get_access_logs(&pet_id);
    assert!(logs_after.len() > initial_count);

    assert_eq!(
        logs_before.get(0).unwrap().id,
        logs_after.get(0).unwrap().id
    );
    assert_eq!(
        logs_before.get(0).unwrap().details,
        logs_after.get(0).unwrap().details
    );
}

#[test]
fn test_log_retrieval_by_pet_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let pet_1 = register_test_pet(&client, &env, &owner);
    let pet_2 = client.register_pet(
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

    let meds = Vec::new(&env);
    client.add_medical_record(
        &pet_1,
        &vet,
        &String::from_str(&env, "Exam"),
        &String::from_str(&env, "Good"),
        &String::from_str(&env, "None"),
        &meds,
    );

    let logs_pet_1 = client.get_access_logs(&pet_1);
    let logs_pet_2 = client.get_access_logs(&pet_2);

    assert!(logs_pet_1.len() > 0);
    assert_eq!(logs_pet_2.len(), 0);
}
