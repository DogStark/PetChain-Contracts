use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

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

    let custody =
        client.grant_temporary_custody(&pet_id, &custodian, &10u64, &20u64, &permissions);

    assert_eq!(custody.permissions.len(), 1);
    assert_eq!(
        custody.permissions.get(0).unwrap(),
        String::from_str(&env, "medicate")
    );
}
