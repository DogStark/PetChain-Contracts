use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

#[test]
fn test_keyword_index_is_per_pet() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let admin = Address::generate(&env);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr Pager"),
        &String::from_str(&env, "PAGER-1"),
        &String::from_str(&env, "General"),
    );
    client.init_admin(&admin);
    client.verify_vet(&admin, &vet);

    let pet_one = client.register_pet(
        &owner,
        &String::from_str(&env, "One"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Gray"),
        &String::from_str(&env, "Mixed"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_two = client.register_pet(
        &owner,
        &String::from_str(&env, "Two"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Mixed"),
        &15,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_medical_record(
        &pet_one,
        &vet,
        &String::from_str(&env, "Pet one"),
        &String::from_str(&env, "Rest"),
        &Vec::new(&env),
        &String::from_str(&env, "shared keyword"),
    );
    client.add_medical_record(
        &pet_two,
        &vet,
        &String::from_str(&env, "Pet two"),
        &String::from_str(&env, "Rest"),
        &Vec::new(&env),
        &String::from_str(&env, "shared keyword"),
    );

    let pet_one_results = client.search_by_keyword(&pet_one, &String::from_str(&env, "shared"));
    assert_eq!(pet_one_results.len(), 1);
    assert_eq!(
        pet_one_results.get(0).unwrap().diagnosis,
        String::from_str(&env, "Pet one")
    );
}
