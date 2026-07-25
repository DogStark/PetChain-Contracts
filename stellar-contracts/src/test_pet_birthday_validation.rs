use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn make_client(env: &Env) -> PetChainContractClient {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init_admin(&admin);
    client
}

#[test]
fn test_register_pet_valid_birthday() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let client = make_client(&env);
    let owner = Address::generate(&env);
    // birthday ~3 years ago
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Labrador"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );
    assert!(pet_id > 0);
}

#[test]
#[should_panic]
fn test_register_pet_future_birthday_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let client = make_client(&env);
    let owner = Address::generate(&env);
    // birthday in the future (year 2030)
    client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2030-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Labrador"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );
}

#[test]
#[should_panic]
fn test_register_pet_impossibly_old_birthday_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    // Set ledger to ~year 2200 so that epoch 0 (1970) is >100 years ago
    // 100 years in seconds = 3_153_600_000; set now = 4_000_000_000 so cutoff = ~847M (year 1996)
    // birthday "1" (second after epoch, ~1970) is before the cutoff
    env.ledger().with_mut(|l| l.timestamp = 4_000_000_000);
    let client = make_client(&env);
    let owner = Address::generate(&env);
    client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "1"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Labrador"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );
}
