use crate::*;
use soroban_sdk::{
    testutils::Address as _,
    Address, Env, String,
};

const NOW: u64 = 1_700_000_000;

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env.ledger().set_timestamp(NOW);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet(&admin, &vet);

    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &NOW,
        &(NOW + 1000),
        &(NOW + 1000),
        &String::from_str(&env, "BATCH-1"),
    );

    (env, client, admin, vet, pet_id, vax_id)
}

#[test]
fn test_idempotent_same_hash_returns_same_cert_id() {
    let (env, client, _admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    let first = client.anchor_certificate_idempotent(&vet, &pet_id, &vax_id, &cert_hash);
    let second = client.anchor_certificate_idempotent(&vet, &pet_id, &vax_id, &cert_hash);

    assert_eq!(first, second);
    assert!(first > 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #51)")]
fn test_idempotent_different_hash_conflicts() {
    let (env, client, _admin, vet, pet_id, vax_id) = setup();
    let hash1 = String::from_str(&env, "sha256:abc123");
    let hash2 = String::from_str(&env, "sha256:def456");

    client.anchor_certificate_idempotent(&vet, &pet_id, &vax_id, &hash1);
    client.anchor_certificate_idempotent(&vet, &pet_id, &vax_id, &hash2);
}

#[test]
fn test_idempotent_creates_new_certificate() {
    let (env, client, _admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    let cert_id = client.anchor_certificate_idempotent(&vet, &pet_id, &vax_id, &cert_hash);
    assert!(cert_id > 0);
    assert!(client.verify_certificate(&pet_id, &vax_id, &cert_hash));
}
