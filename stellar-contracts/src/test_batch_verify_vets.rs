#![cfg(test)]
use crate::{PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup_client(env: &Env) -> PetChainContractClient {
    let contract_id = env.register_contract(None, PetChainContract);
    PetChainContractClient::new(env, &contract_id)
}

fn register_test_vet(
    client: &PetChainContractClient,
    env: &Env,
    vet_address: &Address,
    name: &str,
    license: &str,
) {
    client.register_vet(
        vet_address,
        &String::from_str(env, name),
        &String::from_str(env, license),
        &String::from_str(env, "General Practice"),
    );
}

#[test]
fn test_batch_verify_vets_all_valid() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Register 5 vets
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    let vet3 = Address::generate(&env);
    let vet4 = Address::generate(&env);
    let vet5 = Address::generate(&env);

    register_test_vet(&client, &env, &vet1, "Dr. Smith", "LIC-001");
    register_test_vet(&client, &env, &vet2, "Dr. Jones", "LIC-002");
    register_test_vet(&client, &env, &vet3, "Dr. Brown", "LIC-003");
    register_test_vet(&client, &env, &vet4, "Dr. Wilson", "LIC-004");
    register_test_vet(&client, &env, &vet5, "Dr. Taylor", "LIC-005");

    // Verify all vets are not verified initially
    assert!(!client.is_verified_vet(&vet1));
    assert!(!client.is_verified_vet(&vet2));
    assert!(!client.is_verified_vet(&vet3));
    assert!(!client.is_verified_vet(&vet4));
    assert!(!client.is_verified_vet(&vet5));

    // Create batch of vet addresses
    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet1.clone());
    vet_addresses.push_back(vet2.clone());
    vet_addresses.push_back(vet3.clone());
    vet_addresses.push_back(vet4.clone());
    vet_addresses.push_back(vet5.clone());

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify all succeeded
    assert_eq!(result.succeeded.len(), 5);
    assert_eq!(result.failed.len(), 0);

    // Verify all vets are now verified
    assert!(client.is_verified_vet(&vet1));
    assert!(client.is_verified_vet(&vet2));
    assert!(client.is_verified_vet(&vet3));
    assert!(client.is_verified_vet(&vet4));
    assert!(client.is_verified_vet(&vet5));

    // Check that succeeded list contains all addresses
    assert!(result.succeeded.contains(&vet1));
    assert!(result.succeeded.contains(&vet2));
    assert!(result.succeeded.contains(&vet3));
    assert!(result.succeeded.contains(&vet4));
    assert!(result.succeeded.contains(&vet5));
}

#[test]
fn test_batch_verify_vets_some_invalid() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Register only 3 vets
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    let vet3 = Address::generate(&env);
    let vet4 = Address::generate(&env); // Not registered
    let vet5 = Address::generate(&env); // Not registered

    register_test_vet(&client, &env, &vet1, "Dr. Smith", "LIC-001");
    register_test_vet(&client, &env, &vet2, "Dr. Jones", "LIC-002");
    register_test_vet(&client, &env, &vet3, "Dr. Brown", "LIC-003");

    // Create batch including unregistered vets
    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet1.clone());
    vet_addresses.push_back(vet2.clone());
    vet_addresses.push_back(vet4.clone()); // Unregistered
    vet_addresses.push_back(vet3.clone());
    vet_addresses.push_back(vet5.clone()); // Unregistered

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify partial success
    assert_eq!(result.succeeded.len(), 3);
    assert_eq!(result.failed.len(), 2);

    // Verify registered vets are now verified
    assert!(client.is_verified_vet(&vet1));
    assert!(client.is_verified_vet(&vet2));
    assert!(client.is_verified_vet(&vet3));

    // Verify unregistered vets are still not verified
    assert!(!client.is_verified_vet(&vet4));
    assert!(!client.is_verified_vet(&vet5));

    // Check succeeded list
    assert!(result.succeeded.contains(&vet1));
    assert!(result.succeeded.contains(&vet2));
    assert!(result.succeeded.contains(&vet3));

    // Check failed list contains the unregistered addresses
    assert_eq!(result.failed.len(), 2);
    let failed_addresses: Vec<Address> = result
        .failed
        .iter()
        .map(|(addr, _error)| addr)
        .collect();
    assert!(failed_addresses.contains(&vet4));
    assert!(failed_addresses.contains(&vet5));
}

#[test]
#[should_panic(expected = "BatchTooLarge")]
fn test_batch_verify_vets_too_large() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Create batch of 21 vets (exceeds maximum of 20)
    let mut vet_addresses = Vec::new(&env);
    for _ in 0..21 {
        let vet = Address::generate(&env);
        vet_addresses.push_back(vet);
    }

    // This should panic with BatchTooLarge error
    client.batch_verify_vets(&admin, &vet_addresses);
}

#[test]
fn test_batch_verify_vets_exactly_twenty() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Register exactly 20 vets (maximum allowed)
    let mut vet_addresses = Vec::new(&env);
    for i in 0..20 {
        let vet = Address::generate(&env);
        let license = format!("LIC-{:03}", i);
        let name = format!("Dr. Vet{}", i);
        register_test_vet(&client, &env, &vet, &name, &license);
        vet_addresses.push_back(vet.clone());
    }

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify all succeeded
    assert_eq!(result.succeeded.len(), 20);
    assert_eq!(result.failed.len(), 0);

    // Verify all vets are now verified
    for vet_addr in vet_addresses.iter() {
        assert!(client.is_verified_vet(&vet_addr));
    }
}

#[test]
fn test_batch_verify_vets_empty_batch() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Create empty batch
    let vet_addresses = Vec::new(&env);

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify result is empty
    assert_eq!(result.succeeded.len(), 0);
    assert_eq!(result.failed.len(), 0);
}

#[test]
fn test_batch_verify_vets_single_vet() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Register single vet
    let vet = Address::generate(&env);
    register_test_vet(&client, &env, &vet, "Dr. Solo", "LIC-001");

    // Create batch with single vet
    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet.clone());

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify succeeded
    assert_eq!(result.succeeded.len(), 1);
    assert_eq!(result.failed.len(), 0);
    assert!(client.is_verified_vet(&vet));
}

#[test]
fn test_batch_verify_vets_all_unregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Create batch of unregistered vets
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    let vet3 = Address::generate(&env);

    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet1.clone());
    vet_addresses.push_back(vet2.clone());
    vet_addresses.push_back(vet3.clone());

    // Batch verify vets
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Verify all failed
    assert_eq!(result.succeeded.len(), 0);
    assert_eq!(result.failed.len(), 3);

    // Verify vets are still not verified
    assert!(!client.is_verified_vet(&vet1));
    assert!(!client.is_verified_vet(&vet2));
    assert!(!client.is_verified_vet(&vet3));
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_batch_verify_vets_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Non-admin user
    let non_admin = Address::generate(&env);

    // Register a vet
    let vet = Address::generate(&env);
    register_test_vet(&client, &env, &vet, "Dr. Smith", "LIC-001");

    // Create batch
    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet);

    // This should panic with Unauthorized error
    client.batch_verify_vets(&non_admin, &vet_addresses);
}

#[test]
fn test_batch_verify_vets_already_verified() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    // Setup admin
    let admin = Address::generate(&env);
    client.init_admin(&admin);

    // Register and verify a vet individually
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    
    register_test_vet(&client, &env, &vet1, "Dr. Smith", "LIC-001");
    register_test_vet(&client, &env, &vet2, "Dr. Jones", "LIC-002");
    
    // Verify vet1 individually
    client.verify_vet(&admin, &vet1);
    assert!(client.is_verified_vet(&vet1));

    // Create batch including already-verified vet
    let mut vet_addresses = Vec::new(&env);
    vet_addresses.push_back(vet1.clone());
    vet_addresses.push_back(vet2.clone());

    // Batch verify vets (including already verified one)
    let result = client.batch_verify_vets(&admin, &vet_addresses);

    // Both should succeed (re-verification is idempotent)
    assert_eq!(result.succeeded.len(), 2);
    assert_eq!(result.failed.len(), 0);

    // Both should be verified
    assert!(client.is_verified_vet(&vet1));
    assert!(client.is_verified_vet(&vet2));
}
