use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
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
fn test_anchor_sets_lifecycle() {
    let (env, client, _admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    let anchor = client.get_certificate_anchor(&pet_id, &vax_id).unwrap();
    let lifecycle = client.get_certificate_lifecycle(&pet_id, &vax_id).unwrap();

    assert_eq!(anchor.pet_id, pet_id);
    assert_eq!(anchor.vaccination_id, vax_id);
    assert_eq!(anchor.cert_hash, cert_hash);
    assert_eq!(anchor.issuer, vet);
    assert!(anchor.anchored_at > 0);

    assert_eq!(lifecycle.cert_id, 1);
    assert_eq!(lifecycle.issue_time, NOW);
    assert_eq!(lifecycle.expiry, NOW + 1000);
    assert!(!lifecycle.revoked);
}

#[test]
fn test_verify_respects_revoked() {
    let (env, client, admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
    client.revoke_certificate(&admin, &pet_id, &vax_id, &String::from_str(&env, "Error"));

    assert!(!client.verify_certificate(&pet_id, &vax_id, &cert_hash));
    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Revoked);
}

#[test]
fn test_verify_respects_expired() {
    let (mut env, client, _admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    // Advance past the certificate expiry (dose expiry = NOW + 1000).
    env.ledger().with_mut(|l| l.timestamp = NOW + 1001);

    assert!(!client.verify_certificate(&pet_id, &vax_id, &cert_hash));
    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Expired);
}

#[test]
#[should_panic(expected = "Error(Contract, #48)")]
fn test_revoke_certificate_twice() {
    let (env, client, admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
    client.revoke_certificate(&admin, &pet_id, &vax_id, &String::from_str(&env, "Once"));
    client.revoke_certificate(&admin, &pet_id, &vax_id, &String::from_str(&env, "Twice"));
}

#[test]
#[should_panic(expected = "Error(Contract, #47)")]
fn test_revoke_missing_certificate() {
    let (env, client, admin, _vet, pet_id, _vax_id) = setup();
    client.revoke_certificate(&admin, &pet_id, &999, &String::from_str(&env, "Missing"));
}

#[test]
fn test_status_transitions() {
    let (env, client, admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::NotAnchored);

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Valid);

    client.revoke_certificate(&admin, &pet_id, &vax_id, &String::from_str(&env, "R"));
    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Revoked);
}

#[test]
fn test_cascade_revocation_on_vaccination() {
    let (env, client, admin, vet, pet_id, vax_id) = setup();
    let cert_hash = String::from_str(&env, "sha256:abc123");

    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Valid);

    client.revoke_vaccination_certificate(&admin, &pet_id, &vax_id, &String::from_str(&env, "Vax error"));

    assert_eq!(client.get_certificate_status(&pet_id, &vax_id), CertificateStatus::Revoked);
    assert!(!client.verify_certificate(&pet_id, &vax_id, &cert_hash));
}
