use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

const NOW: u64 = 1_700_000_000;

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, u64, u64) {
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
fn test_verify_vet_with_expiry_stores_expiry() {
    let (env, client, admin, vet, _pet_id, _vax_id) = setup();
    let expiry = NOW + 86_400;

    client.verify_vet_with_expiry(&admin, &vet, &Some(expiry));
    assert_eq!(client.get_vet_credentials_expiry(&vet), Some(expiry));
}

#[test]
fn test_is_verified_vet_before_expiry() {
    let (env, client, admin, vet, _pet_id, _vax_id) = setup();
    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW + 86_400));

    assert!(client.is_verified_vet(&vet));
}

#[test]
fn test_is_verified_vet_after_expiry() {
    let (mut env, client, admin, vet, _pet_id, _vax_id) = setup();
    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW + 86_400));

    env.ledger().with_mut(|l| l.timestamp = NOW + 86_401);
    assert!(!client.is_verified_vet(&vet));
}

#[test]
#[should_panic(expected = "Error(Contract, #50)")]
fn test_add_vaccination_rejects_expired_vet() {
    let (env, client, admin, vet, pet_id, _vax_id) = setup();
    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW - 1));

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &NOW,
        &(NOW + 1000),
        &(NOW + 1000),
        &String::from_str(&env, "BATCH-2"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #50)")]
fn test_add_medical_record_rejects_expired_vet() {
    let (env, client, admin, vet, pet_id, _vax_id) = setup();
    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW - 1));

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Fine"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes"),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #50)")]
fn test_anchor_certificate_rejects_expired_vet() {
    let (env, client, admin, vet, pet_id, vax_id) = setup();
    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW - 1));

    let cert_hash = String::from_str(&env, "sha256:abc123");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
}

#[test]
fn test_verify_clears_stale_expiry() {
    let (env, client, admin, vet, _pet_id, _vax_id) = setup();

    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW));
    assert_eq!(client.get_vet_credentials_expiry(&vet), Some(NOW));

    client.verify_vet(&admin, &vet);
    assert_eq!(client.get_vet_credentials_expiry(&vet), None);
}

#[test]
fn test_revoke_clears_stale_expiry() {
    let (env, client, admin, vet, _pet_id, _vax_id) = setup();

    client.verify_vet_with_expiry(&admin, &vet, &Some(NOW + 86_400));
    assert_eq!(client.get_vet_credentials_expiry(&vet), Some(NOW + 86_400));

    client.revoke_vet_license(&admin, &vet);
    assert_eq!(client.get_vet_credentials_expiry(&vet), None);
}
