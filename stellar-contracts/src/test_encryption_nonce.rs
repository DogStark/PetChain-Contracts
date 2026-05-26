use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

fn register_test_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Nonce"),
        &String::from_str(env, "2021-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Mixed"),
        &20,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_rotate_nonce_generates_new_nonce() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);
    let key_id = String::from_str(&env, "profile");

    let first = client.rotate_nonce(&pet_id, &key_id);
    let second = client.rotate_nonce(&pet_id, &key_id);

    assert_ne!(first, second);
    assert_eq!(first.len(), 12);
    assert_eq!(second.len(), 12);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_rejects_nonce_reuse_after_default_limit() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);
    let key_id = String::from_str(&env, "profile");
    let nonce = client.rotate_nonce(&pet_id, &key_id);

    assert!(client.record_nonce_use(&pet_id, &key_id, &nonce));
    client.record_nonce_use(&pet_id, &key_id, &nonce);
}

#[test]
fn test_configurable_nonce_use_limit_and_history_pruning() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = register_test_pet(&env, &client, &owner);
    let key_id = String::from_str(&env, "session");

    client.set_nonce_max_use(&pet_id, &key_id, &2);
    let nonce = client.rotate_nonce(&pet_id, &key_id);
    assert!(client.record_nonce_use(&pet_id, &key_id, &nonce));
    assert!(client.record_nonce_use(&pet_id, &key_id, &nonce));

    for i in 0..10 {
        let mut manual = Bytes::new(&env);
        manual.push_back(i);
        client.record_nonce_use(&pet_id, &key_id, &manual);
    }

    let history = client.get_nonce_history(&pet_id, &key_id);
    assert_eq!(history.len(), 8);
    assert_eq!(history.get(0).unwrap().get(0).unwrap(), 2);
}
