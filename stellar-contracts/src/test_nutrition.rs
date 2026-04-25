use crate::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String, Vec};

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
fn test_get_current_diet_plan() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let restrictions = Vec::new(&env);
    let allergies = Vec::new(&env);

    env.ledger().with_mut(|li| li.timestamp = 1000);
    client.set_diet_plan(
        &pet_id,
        &String::from_str(&env, "Dry Kibble"),
        &String::from_str(&env, "200g"),
        &String::from_str(&env, "Twice daily"),
        &restrictions,
        &allergies,
    );

    env.ledger().with_mut(|li| li.timestamp = 2000);
    client.set_diet_plan(
        &pet_id,
        &String::from_str(&env, "Wet Food"),
        &String::from_str(&env, "300g"),
        &String::from_str(&env, "Three times daily"),
        &restrictions,
        &allergies,
    );

    let current = client.get_current_diet_plan(&pet_id).unwrap();
    assert_eq!(current.food_type, String::from_str(&env, "Wet Food"));
    assert_eq!(current.created_at, 2000);
}

#[test]
fn test_get_current_diet_plan_no_plans() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2022-01-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    let result = client.get_current_diet_plan(&pet_id);
    assert!(result.is_none());
}
