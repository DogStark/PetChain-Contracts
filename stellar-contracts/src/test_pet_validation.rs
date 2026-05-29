use crate::{ContractError, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (PetChainContractClient, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init_admin(&admin);
    (client, admin)
}

fn register(env: &Env, client: &PetChainContractClient, name: &str, species: &Species, breed: &str) -> u64 {
    let owner = Address::generate(env);
    client.register_pet(
        &owner,
        &String::from_str(env, name),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        species,
        &String::from_str(env, breed),
        &String::from_str(env, "Brown"),
        &10,
        &None,
        &PrivacyLevel::Public,
    )
}

// --- Name validation ---

#[test]
fn test_valid_name_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let id = register(&env, &client, "Buddy-2", &Species::Dog, "Labrador");
    assert!(id > 0);
}

#[test]
#[should_panic]
fn test_empty_name_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    register(&env, &client, "", &Species::Dog, "Labrador");
}

#[test]
#[should_panic]
fn test_name_too_long_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    // 65 chars
    register(&env, &client, "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCDDDDDDDDDDEEEEEEEEEEFFFFF123456789012", &Species::Dog, "Labrador");
}

#[test]
#[should_panic]
fn test_name_invalid_chars_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    register(&env, &client, "Buddy@!", &Species::Dog, "Labrador");
}

#[test]
fn test_name_max_length_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    // exactly 64 chars
    let id = register(&env, &client, "AAAAAAAAAABBBBBBBBBBCCCCCCCCCCDDDDDDDDDDEEEEEEEEEEFFFFF1234567890", &Species::Dog, "Labrador");
    assert!(id > 0);
}

// --- Breed whitelist ---

#[test]
fn test_breed_accepted_when_no_whitelist() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let id = register(&env, &client, "Rex", &Species::Dog, "AnythingGoes");
    assert!(id > 0);
}

#[test]
fn test_breed_accepted_when_in_whitelist() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let mut breeds = Vec::new(&env);
    breeds.push_back(String::from_str(&env, "Labrador"));
    breeds.push_back(String::from_str(&env, "Poodle"));
    client.set_breed_list(&admin, &Species::Dog, &breeds);

    let id = register(&env, &client, "Rex", &Species::Dog, "Labrador");
    assert!(id > 0);
}

#[test]
#[should_panic]
fn test_breed_rejected_when_not_in_whitelist() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let mut breeds = Vec::new(&env);
    breeds.push_back(String::from_str(&env, "Labrador"));
    client.set_breed_list(&admin, &Species::Dog, &breeds);

    register(&env, &client, "Rex", &Species::Dog, "Poodle");
}

#[test]
fn test_breed_whitelist_is_species_specific() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    // Set dog whitelist only
    let mut dog_breeds = Vec::new(&env);
    dog_breeds.push_back(String::from_str(&env, "Labrador"));
    client.set_breed_list(&admin, &Species::Dog, &dog_breeds);

    // Cat has no whitelist — any breed accepted
    let id = register(&env, &client, "Whiskers", &Species::Cat, "AnyCatBreed");
    assert!(id > 0);
}

#[test]
fn test_get_breed_list_returns_set_list() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let mut breeds = Vec::new(&env);
    breeds.push_back(String::from_str(&env, "Persian"));
    breeds.push_back(String::from_str(&env, "Siamese"));
    client.set_breed_list(&admin, &Species::Cat, &breeds);

    let result = client.get_breed_list(&Species::Cat);
    assert_eq!(result.len(), 2);
}

#[test]
fn test_clear_breed_list_allows_any_breed() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin) = setup(&env);

    let mut breeds = Vec::new(&env);
    breeds.push_back(String::from_str(&env, "Labrador"));
    client.set_breed_list(&admin, &Species::Dog, &breeds);

    // Clear by passing empty vec
    client.set_breed_list(&admin, &Species::Dog, &Vec::new(&env));

    // Now any breed is accepted
    let id = register(&env, &client, "Rex", &Species::Dog, "Poodle");
    assert!(id > 0);
}
