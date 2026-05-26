use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let owner = Address::generate(env);
    let vet = Address::generate(env);
    let admin = Address::generate(env);

    client.register_vet(
        &vet,
        &String::from_str(env, "Dr Search"),
        &String::from_str(env, "SEARCH-1"),
        &String::from_str(env, "General"),
    );
    client.init_admin(&admin);
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Index"),
        &String::from_str(env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(env, "Gray"),
        &String::from_str(env, "Mixed"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    (client, vet, pet_id)
}

#[test]
fn test_search_by_keyword_uses_index() {
    let env = Env::default();
    let (client, vet, pet_id) = setup(&env);

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Cough"),
        &String::from_str(&env, "Rest"),
        &Vec::new(&env),
        &String::from_str(&env, "Mild cough improved"),
    );
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diet"),
        &String::from_str(&env, "Food change"),
        &Vec::new(&env),
        &String::from_str(&env, "Weight stable"),
    );

    let results = client.search_by_keyword(&pet_id, &String::from_str(&env, "COUGH"));
    assert_eq!(results.len(), 1);
    assert_eq!(
        results.get(0).unwrap().diagnosis,
        String::from_str(&env, "Cough")
    );
}

#[test]
fn test_remove_medical_record_prunes_keyword_index() {
    let env = Env::default();
    let (client, vet, pet_id) = setup(&env);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Allergy"),
        &String::from_str(&env, "Antihistamine"),
        &Vec::new(&env),
        &String::from_str(&env, "Skin allergy rash"),
    );

    assert_eq!(
        client
            .search_by_keyword(&pet_id, &String::from_str(&env, "rash"))
            .len(),
        1
    );
    assert!(client.remove_medical_record(&record_id));
    assert_eq!(
        client
            .search_by_keyword(&pet_id, &String::from_str(&env, "rash"))
            .len(),
        0
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_rejects_oversized_keyword() {
    let env = Env::default();
    let (client, _vet, pet_id) = setup(&env);
    client.search_by_keyword(
        &pet_id,
        &String::from_str(&env, "keyword-that-is-far-too-long-for-index"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #11)")]
fn test_rejects_too_many_tokens() {
    let env = Env::default();
    let (client, vet, pet_id) = setup(&env);
    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Verbose"),
        &String::from_str(&env, "None"),
        &Vec::new(&env),
        &String::from_str(&env, "a b c d e f g h i j k l m n o p q"),
    );
}
