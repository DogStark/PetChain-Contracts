use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init_admin(&admin);
    (client, admin)
}

fn do_register(env: &Env, client: &PetChainContractClient, owner: &Address, nonce: u64) -> u64 {
    client.register_pet_with_nonce(
        owner,
        &nonce,
        &String::from_str(env, "Buddy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &10,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_get_nonce_starts_at_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let caller = Address::generate(&env);
    assert_eq!(client.get_caller_nonce(&caller), 0);
}

#[test]
fn test_nonce_increments_after_successful_call() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let owner = Address::generate(&env);

    assert_eq!(client.get_caller_nonce(&owner), 0);
    do_register(&env, &client, &owner, 0);
    assert_eq!(client.get_caller_nonce(&owner), 1);
    do_register(&env, &client, &owner, 1);
    assert_eq!(client.get_caller_nonce(&owner), 2);
}

#[test]
#[should_panic]
fn test_replayed_nonce_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let owner = Address::generate(&env);

    do_register(&env, &client, &owner, 0); // nonce 0 → increments to 1
    do_register(&env, &client, &owner, 0); // replay with nonce 0 → rejected
}

#[test]
#[should_panic]
fn test_future_nonce_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let owner = Address::generate(&env);

    do_register(&env, &client, &owner, 5); // nonce is 0, supplying 5 → rejected
}

#[test]
fn test_nonces_are_per_caller() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);

    let owner_a = Address::generate(&env);
    let owner_b = Address::generate(&env);

    do_register(&env, &client, &owner_a, 0);
    do_register(&env, &client, &owner_a, 1);

    // owner_b still at nonce 0
    assert_eq!(client.get_caller_nonce(&owner_b), 0);
    do_register(&env, &client, &owner_b, 0);
    assert_eq!(client.get_caller_nonce(&owner_b), 1);
}

#[test]
#[should_panic]
fn test_concurrent_nonce_collision() {
    // Simulates two calls with the same nonce — second must fail.
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let owner = Address::generate(&env);

    // Both supply nonce=0; first succeeds, second is rejected.
    do_register(&env, &client, &owner, 0);
    do_register(&env, &client, &owner, 0);
}
