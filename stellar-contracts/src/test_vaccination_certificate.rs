use crate::{
    Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species, VaccineType,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, PetChainContractClient, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    (env, client, admin)
}

fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Labrador"),
        &25,
        &None,
        &PrivacyLevel::Public,
    )
}

fn register_and_verify_vet(
    client: &PetChainContractClient,
    env: &Env,
    vet: &Address,
) {
    client.register_vet(
        vet,
        &String::from_str(env, "Dr. Smith"),
        &String::from_str(env, "VET123"),
        &String::from_str(env, "General Practice"),
    );
    client.verify_vet_license(vet, &String::from_str(env, "VET123"));
}

fn add_vaccination(
    client: &PetChainContractClient,
    env: &Env,
    pet_id: u64,
    vet: &Address,
) -> u64 {
    client.add_vaccination(
        &pet_id,
        vet,
        &VaccineType::Rabies,
        &String::from_str(env, "Rabivax"),
        &String::from_str(env, "BATCH123"),
    )
}

#[test]
fn test_anchor_certificate_success() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Anchor certificate
    let cert_hash = String::from_str(&env, "sha256:abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    // Verify certificate was anchored
    let anchor = client.get_certificate_anchor(&pet_id, &vax_id).unwrap();
    assert_eq!(anchor.pet_id, pet_id);
    assert_eq!(anchor.vaccination_id, vax_id);
    assert_eq!(anchor.cert_hash, cert_hash);
    assert_eq!(anchor.issuer, vet);
    assert!(anchor.anchored_at > 0);
}

#[test]
fn test_verify_certificate_valid() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Anchor certificate
    let cert_hash = String::from_str(&env, "sha256:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    // Verify with correct hash
    let is_valid = client.verify_certificate(&pet_id, &vax_id, &cert_hash);
    assert!(is_valid);
}

#[test]
fn test_verify_certificate_invalid_hash() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Anchor certificate
    let cert_hash = String::from_str(&env, "sha256:correct_hash_1234567890abcdef1234567890abcdef1234567890abcdef12");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    // Verify with wrong hash
    let wrong_hash = String::from_str(&env, "sha256:wrong_hash_1234567890abcdef1234567890abcdef1234567890abcdef123");
    let is_valid = client.verify_certificate(&pet_id, &vax_id, &wrong_hash);
    assert!(!is_valid);
}

#[test]
fn test_verify_certificate_not_anchored() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to verify without anchoring
    let cert_hash = String::from_str(&env, "sha256:some_hash_1234567890abcdef1234567890abcdef1234567890abcdef1234");
    let is_valid = client.verify_certificate(&pet_id, &vax_id, &cert_hash);
    assert!(!is_valid);
}

#[test]
fn test_get_certificate_anchor_exists() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Anchor certificate
    let cert_hash = String::from_str(&env, "sha256:anchor_test_hash_1234567890abcdef1234567890abcdef1234567890ab");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    // Get anchor
    let anchor = client.get_certificate_anchor(&pet_id, &vax_id);
    assert!(anchor.is_some());
    
    let anchor_data = anchor.unwrap();
    assert_eq!(anchor_data.pet_id, pet_id);
    assert_eq!(anchor_data.vaccination_id, vax_id);
    assert_eq!(anchor_data.cert_hash, cert_hash);
    assert_eq!(anchor_data.issuer, vet);
}

#[test]
fn test_get_certificate_anchor_not_exists() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to get anchor without anchoring
    let anchor = client.get_certificate_anchor(&pet_id, &vax_id);
    assert!(anchor.is_none());
}

#[test]
#[should_panic(expected = "VetNotVerified")]
fn test_anchor_certificate_unverified_vet() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    
    // Register vet but don't verify
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Unverified"),
        &String::from_str(&env, "VET999"),
        &String::from_str(&env, "General Practice"),
    );
    
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to anchor certificate - should fail
    let cert_hash = String::from_str(&env, "sha256:test_hash_1234567890abcdef1234567890abcdef1234567890abcdef12345");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
}

#[test]
#[should_panic(expected = "VetNotFound")]
fn test_anchor_certificate_non_vet() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let non_vet = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to anchor certificate as non-vet - should fail
    let cert_hash = String::from_str(&env, "sha256:test_hash_1234567890abcdef1234567890abcdef1234567890abcdef12345");
    client.anchor_certificate(&non_vet, &pet_id, &vax_id, &cert_hash);
}

#[test]
#[should_panic(expected = "PetNotFound")]
fn test_anchor_certificate_nonexistent_pet() {
    let (env, client, _admin) = setup();
    let vet = Address::generate(&env);

    register_and_verify_vet(&client, &env, &vet);

    // Try to anchor certificate for nonexistent pet
    let cert_hash = String::from_str(&env, "sha256:test_hash_1234567890abcdef1234567890abcdef1234567890abcdef12345");
    client.anchor_certificate(&vet, &999, &1, &cert_hash);
}

#[test]
#[should_panic(expected = "VaccinationNotFound")]
fn test_anchor_certificate_nonexistent_vaccination() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);

    // Try to anchor certificate for nonexistent vaccination
    let cert_hash = String::from_str(&env, "sha256:test_hash_1234567890abcdef1234567890abcdef1234567890abcdef12345");
    client.anchor_certificate(&vet, &pet_id, &999, &cert_hash);
}

#[test]
#[should_panic(expected = "VaccinationNotFound")]
fn test_anchor_certificate_wrong_pet() {
    let (env, client, _admin) = setup();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id1 = register_pet(&client, &env, &owner1);
    let pet_id2 = register_pet(&client, &env, &owner2);
    register_and_verify_vet(&client, &env, &vet);
    
    let vax_id = add_vaccination(&client, &env, pet_id1, &vet);

    // Try to anchor certificate for wrong pet
    let cert_hash = String::from_str(&env, "sha256:test_hash_1234567890abcdef1234567890abcdef1234567890abcdef12345");
    client.anchor_certificate(&vet, &pet_id2, &vax_id, &cert_hash);
}

#[test]
#[should_panic(expected = "CertificateAlreadyAnchored")]
fn test_anchor_certificate_already_anchored() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Anchor certificate first time
    let cert_hash1 = String::from_str(&env, "sha256:first_hash_1234567890abcdef1234567890abcdef1234567890abcdef123");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash1);

    // Try to anchor again - should fail
    let cert_hash2 = String::from_str(&env, "sha256:second_hash_1234567890abcdef1234567890abcdef1234567890abcdef12");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash2);
}

#[test]
#[should_panic(expected = "InvalidCertificateHash")]
fn test_anchor_certificate_empty_hash() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to anchor with empty hash
    let cert_hash = String::from_str(&env, "");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
}

#[test]
#[should_panic(expected = "InvalidCertificateHash")]
fn test_anchor_certificate_hash_too_long() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    // Try to anchor with hash > 128 characters
    let long_hash = "a".repeat(129);
    let cert_hash = String::from_str(&env, &long_hash);
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);
}

#[test]
fn test_multiple_vaccinations_different_certificates() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);

    // Add multiple vaccinations
    let vax_id1 = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &String::from_str(&env, "BATCH123"),
    );

    let vax_id2 = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Parvovirus,
        &String::from_str(&env, "Parvovax"),
        &String::from_str(&env, "BATCH456"),
    );

    // Anchor different certificates
    let cert_hash1 = String::from_str(&env, "sha256:cert1_hash_1234567890abcdef1234567890abcdef1234567890abcdef123");
    let cert_hash2 = String::from_str(&env, "sha256:cert2_hash_1234567890abcdef1234567890abcdef1234567890abcdef123");

    client.anchor_certificate(&vet, &pet_id, &vax_id1, &cert_hash1);
    client.anchor_certificate(&vet, &pet_id, &vax_id2, &cert_hash2);

    // Verify both certificates
    assert!(client.verify_certificate(&pet_id, &vax_id1, &cert_hash1));
    assert!(client.verify_certificate(&pet_id, &vax_id2, &cert_hash2));

    // Verify cross-verification fails
    assert!(!client.verify_certificate(&pet_id, &vax_id1, &cert_hash2));
    assert!(!client.verify_certificate(&pet_id, &vax_id2, &cert_hash1));
}

#[test]
fn test_different_vets_can_anchor() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    
    // Register and verify both vets
    register_and_verify_vet(&client, &env, &vet1);
    client.register_vet(
        &vet2,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "VET456"),
        &String::from_str(&env, "Surgery"),
    );
    client.verify_vet_license(&vet2, &String::from_str(&env, "VET456"));

    // Add vaccinations from different vets
    let vax_id1 = add_vaccination(&client, &env, pet_id, &vet1);
    let vax_id2 = client.add_vaccination(
        &pet_id,
        &vet2,
        &VaccineType::Leukemia,
        &String::from_str(&env, "Leukemiavax"),
        &String::from_str(&env, "BATCH789"),
    );

    // Both vets can anchor their certificates
    let cert_hash1 = String::from_str(&env, "sha256:vet1_cert_1234567890abcdef1234567890abcdef1234567890abcdef1234");
    let cert_hash2 = String::from_str(&env, "sha256:vet2_cert_1234567890abcdef1234567890abcdef1234567890abcdef1234");

    client.anchor_certificate(&vet1, &pet_id, &vax_id1, &cert_hash1);
    client.anchor_certificate(&vet2, &pet_id, &vax_id2, &cert_hash2);

    // Verify both anchors
    let anchor1 = client.get_certificate_anchor(&pet_id, &vax_id1).unwrap();
    let anchor2 = client.get_certificate_anchor(&pet_id, &vax_id2).unwrap();

    assert_eq!(anchor1.issuer, vet1);
    assert_eq!(anchor2.issuer, vet2);
}

#[test]
fn test_certificate_hash_formats() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);

    // Test various hash formats
    let test_hashes = vec![
        "sha256:abcdef1234567890",
        "0x1234567890abcdef",
        "QmHash1234567890",
        "ipfs://QmHash",
        "simple_hash_123",
    ];

    for (i, hash_str) in test_hashes.iter().enumerate() {
        let vax_id = client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Other,
            &String::from_str(&env, "TestVax"),
            &String::from_str(&env, &format!("BATCH{}", i)),
        );

        let cert_hash = String::from_str(&env, hash_str);
        client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

        // Verify the hash was stored correctly
        let anchor = client.get_certificate_anchor(&pet_id, &vax_id).unwrap();
        assert_eq!(anchor.cert_hash, cert_hash);
    }
}

#[test]
fn test_anchor_timestamp_recorded() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner);
    register_and_verify_vet(&client, &env, &vet);
    let vax_id = add_vaccination(&client, &env, pet_id, &vet);

    let before_time = env.ledger().timestamp();

    // Anchor certificate
    let cert_hash = String::from_str(&env, "sha256:timestamp_test_hash_1234567890abcdef1234567890abcdef12345678");
    client.anchor_certificate(&vet, &pet_id, &vax_id, &cert_hash);

    let after_time = env.ledger().timestamp();

    // Verify timestamp is within range
    let anchor = client.get_certificate_anchor(&pet_id, &vax_id).unwrap();
    assert!(anchor.anchored_at >= before_time);
    assert!(anchor.anchored_at <= after_time);
}
