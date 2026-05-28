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

#[test]
fn test_age_calculation_from_iso_date() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    env.ledger().with_mut(|l| l.timestamp = 1_609_459_200);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Mixed"),
        &String::from_str(&env, "Black"),
        &18u32,
        &None,
        &PrivacyLevel::Public,
    );

    let (years, months) = client.get_pet_age(&pet_id);
    assert_eq!((years, months), (1, 0));
}

#[test]
#[should_panic]
fn test_register_pet_rejects_invalid_birthday_format() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "01/01/2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
}

// --- Issue #622: Breed Metadata and Lifespan ---

#[test]
fn test_add_breed_metadata_and_calculate_lifespan() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    env.ledger().with_mut(|l| l.timestamp = 2_000_000_000);

    // Add breed metadata for Golden Retriever with 12 year lifespan
    client.add_breed_metadata(
        &admin,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Dog"),
        &12,
    );

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

    let age = client.get_pet_age_with_lifespan(&pet_id);
    assert_eq!(age.years, 1);
    assert_eq!(age.months, 2);
    // 1.16 years / 12 years = ~9.7%
    assert!(age.lifespan_pct.is_some());
    assert!(age.lifespan_pct.unwrap() < 15 && age.lifespan_pct.unwrap() > 5);
}

#[test]
fn test_unknown_breed_returns_none_lifespan() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    env.ledger().with_mut(|l| l.timestamp = 2_000_000_000);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Mystery"),
        &String::from_str(&env, "1963280000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "UnknownBreed"),
        &String::from_str(&env, "Black"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let age = client.get_pet_age_with_lifespan(&pet_id);
    assert_eq!(age.years, 1);
    assert_eq!(age.months, 2);
    assert_eq!(age.lifespan_pct, None);
}

#[test]
fn test_update_breed_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let breed_id = String::from_str(&env, "Labrador");

    // Add initial metadata
    client.add_breed_metadata(&admin, &breed_id, &String::from_str(&env, "Dog"), &11);

    // Update metadata
    client.update_breed_metadata(&admin, &breed_id, &String::from_str(&env, "Dog"), &12);

    // Verify update worked
    env.ledger().with_mut(|l| l.timestamp = 2_000_000_000);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "1963280000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );

    let age = client.get_pet_age_with_lifespan(&pet_id);
    assert!(age.lifespan_pct.is_some());
}

#[test]
fn test_delete_breed_metadata() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let breed_id = String::from_str(&env, "Poodle");

    // Add and then delete metadata
    client.add_breed_metadata(&admin, &breed_id, &String::from_str(&env, "Dog"), &15);
    client.delete_breed_metadata(&admin, &breed_id);

    // Verify deletion - should now return None for lifespan_pct
    env.ledger().with_mut(|l| l.timestamp = 2_000_000_000);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Fluffy"),
        &String::from_str(&env, "1963280000"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &22u32,
        &None,
        &PrivacyLevel::Public,
    );

    let age = client.get_pet_age_with_lifespan(&pet_id);
    assert_eq!(age.lifespan_pct, None);
}
