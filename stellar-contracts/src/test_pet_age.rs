use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

#[test]
fn test_age_calculation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    env.ledger().with_mut(|l| l.timestamp = 2_000_000_000);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "1963280000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let (years, months) = client.get_pet_age(&pet_id);
    assert_eq!((years, months), (1, 2));
}

#[test]
fn test_age_edge_cases() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let (years, months) = client.get_pet_age(&9999);
    assert_eq!((years, months), (0, 0));

    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let owner = Address::generate(&env);
    let future_pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Future Pet"),
        &String::from_str(&env, "1000100"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let (future_years, future_months) = client.get_pet_age(&future_pet_id);
    assert_eq!((future_years, future_months), (0, 0));
}
