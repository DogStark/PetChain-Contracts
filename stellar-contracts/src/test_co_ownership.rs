use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Helper: register a pet and return (client, env, owner, pet_id)
fn setup() -> (Env, PetChainContractClient<'static>, Address, u64) {
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

    (env, client, owner, pet_id)
}

// --------------------------------------------------------------------------
// Test: adding a co-owner
// --------------------------------------------------------------------------

#[test]
fn test_add_co_owner() {
    let (env, client, owner, pet_id) = setup();

    let co_owner = Address::generate(&env);

    // Initially only primary owner in the list
    let owners = client.get_co_owners(&pet_id);
    assert_eq!(owners.len(), 1);
    assert_eq!(owners.get(0).unwrap(), owner);

    // Add a co-owner
    client.add_co_owner(&pet_id, &co_owner);

    let owners_after = client.get_co_owners(&pet_id);
    assert_eq!(owners_after.len(), 2);
    assert!(owners_after.contains(owner));
    assert!(owners_after.contains(co_owner.clone()));
}

// --------------------------------------------------------------------------
// Test: removing a co-owner
// --------------------------------------------------------------------------

#[test]
fn test_remove_co_owner() {
    let (env, client, owner, pet_id) = setup();

    let co_owner = Address::generate(&env);

    // Add, then remove
    client.add_co_owner(&pet_id, &co_owner);
    let owners_mid = client.get_co_owners(&pet_id);
    assert_eq!(owners_mid.len(), 2);

    client.remove_co_owner(&pet_id, &co_owner);

    let owners_final = client.get_co_owners(&pet_id);
    assert_eq!(owners_final.len(), 1);
    assert_eq!(owners_final.get(0).unwrap(), owner);
    assert!(!owners_final.contains(co_owner));
}

// --------------------------------------------------------------------------
// Test: all owners have Full access
// --------------------------------------------------------------------------

#[test]
fn test_all_owners_have_full_access() {
    let (env, client, owner, pet_id) = setup();

    let co_owner_1 = Address::generate(&env);
    let co_owner_2 = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.add_co_owner(&pet_id, &co_owner_1);
    client.add_co_owner(&pet_id, &co_owner_2);

    // Primary owner has full access
    assert_eq!(client.check_access(&pet_id, &owner), AccessLevel::Full);
    // Co-owners have full access
    assert_eq!(client.check_access(&pet_id, &co_owner_1), AccessLevel::Full);
    assert_eq!(client.check_access(&pet_id, &co_owner_2), AccessLevel::Full);
    // Stranger has no access
    assert_eq!(client.check_access(&pet_id, &stranger), AccessLevel::None);
}

// --------------------------------------------------------------------------
// Test: primary_owner and owners list are set correctly after registration
// --------------------------------------------------------------------------

#[test]
fn test_primary_owner_set_on_registration() {
    let (env, client, owner, pet_id) = setup();
    let _ = env;

    let profile = client.get_pet(&pet_id).unwrap();
    assert_eq!(profile.primary_owner, owner);
    assert_eq!(profile.owners.len(), 1);
    assert_eq!(profile.owners.get(0).unwrap(), owner);
}

// --------------------------------------------------------------------------
// Test: get_co_owners returns empty Vec for unknown pet
// --------------------------------------------------------------------------

#[test]
fn test_get_co_owners_unknown_pet() {
    let (_env, client, _owner, _pet_id) = setup();

    let owners = client.get_co_owners(&9999u64);
    assert_eq!(owners.len(), 0);
}

// --------------------------------------------------------------------------
// Test: adding multiple co-owners
// --------------------------------------------------------------------------

#[test]
fn test_add_multiple_co_owners() {
    let (env, client, owner, pet_id) = setup();

    let co1 = Address::generate(&env);
    let co2 = Address::generate(&env);
    let co3 = Address::generate(&env);

    client.add_co_owner(&pet_id, &co1);
    client.add_co_owner(&pet_id, &co2);
    client.add_co_owner(&pet_id, &co3);

    let owners = client.get_co_owners(&pet_id);
    assert_eq!(owners.len(), 4);
    assert!(owners.contains(owner));
    assert!(owners.contains(co1));
    assert!(owners.contains(co2));
    assert!(owners.contains(co3));
}

// --------------------------------------------------------------------------
// Test: co-owner can trigger pet operations (mock_all_auths is used)
// --------------------------------------------------------------------------

#[test]
fn test_co_owner_can_add_photo() {
    let (env, client, _owner, pet_id) = setup();

    let co_owner = Address::generate(&env);
    client.add_co_owner(&pet_id, &co_owner);

    // With mock_all_auths any signer is accepted; the key check is that the
    // contract does NOT panic when a co-owner triggers the operation.
    let ok = client.add_pet_photo(
        &pet_id,
        &String::from_str(&env, "QmYwAPJzv5CZsnAg9XxaBv7kNtbfXHs7nL6nKLXA65UmF2"),
    );
    assert!(ok);
}
