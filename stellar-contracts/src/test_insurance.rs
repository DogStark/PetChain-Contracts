use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

#[test]
fn test_insurance_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    // Attempting to add insurance for non-existent pet should return false
    let expiry = env.ledger().timestamp() + 31536000;
    let result = client.add_insurance_policy(
        &1,
        &String::from_str(&env, "POL-123"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &expiry,
    );
    assert_eq!(result, false);

    // Register a pet
    // fn register_pet(
    //     env: Env,
    //     owner: Address,
    //     name: String,
    //     birthday: String,
    //     gender: Gender,
    //     species: Species,
    //     color: String,
    //     breed: String,
    //     weight: u32,
    //     microchip_id: Option<String>,
    //     privacy_level: PrivacyLevel,
    // ) -> u64
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Labrador"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    // Add insurance
    let expiry2 = env.ledger().timestamp() + 31536000;
    let result = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-123"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &expiry2,
    );
    assert_eq!(result, true);

    // Get insurance
    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.policy_id, String::from_str(&env, "POL-123"));
    assert_eq!(policy.provider, String::from_str(&env, "PetProtect"));
    assert_eq!(
        policy.coverage_type,
        String::from_str(&env, "Comprehensive")
    );
    assert_eq!(policy.premium, 1000);
    assert_eq!(policy.coverage_limit, 50000);
    assert_eq!(policy.active, true);

    // Update insurance status
    let update_result = client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-123"),
        &false,
    );
    assert_eq!(update_result, true);

    let updated_policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(updated_policy.active, false);
}

#[test]
fn test_is_insurance_active_no_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // No policy registered — should return false
    assert_eq!(client.is_insurance_active(&99), false);
}

#[test]
fn test_is_insurance_active_with_active_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-10"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Siamese"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    // Policy expires 1 year from now
    let expiry = env.ledger().timestamp() + 31_536_000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-ACTIVE"),
        &String::from_str(&env, "SafePaws"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &expiry,
    );

    assert_eq!(client.is_insurance_active(&pet_id), true);
}

#[test]
fn test_is_insurance_active_with_expired_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2019-03-15"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Poodle"),
        &8,
        &None,
        &PrivacyLevel::Public,
    );

    // Set ledger time to 1000 so we can set an expiry in the past
    env.ledger().set_timestamp(1000);
    // Policy expired at timestamp 500 (before current time of 1000)
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-EXPIRED"),
        &String::from_str(&env, "OldCover"),
        &String::from_str(&env, "Basic"),
        &300,
        &10000,
        &500,
    );

    assert_eq!(client.is_insurance_active(&pet_id), false);
}

#[test]
fn test_is_insurance_active_with_inactive_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2022-07-20"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &String::from_str(&env, "Retriever"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31_536_000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-DEACTIVATED"),
        &String::from_str(&env, "PetShield"),
        &String::from_str(&env, "Comprehensive"),
        &800,
        &40000,
        &expiry,
    );

    // Deactivate the policy
    client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-DEACTIVATED"),
        &false,
    );

    assert_eq!(client.is_insurance_active(&pet_id), false);
}
