use crate::{
    AccessLevel, ConsentScope, ConsentType, Gender, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species, VaccineType,
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

fn register_pet(
    client: &PetChainContractClient,
    env: &Env,
    owner: &Address,
    privacy: PrivacyLevel,
) -> u64 {
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
        &privacy,
    )
}

#[test]
fn test_get_pet_full_profile_batch_public_pet() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add a consent
    let grantee = Address::generate(&env);
    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &ConsentScope::ReadMedical,
    );

    // Add a medical record
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "None"),
    );

    // Get batch profile
    let batch = client.get_pet_full_profile_batch(&pet_id, &caller).unwrap();

    assert_eq!(batch.profile.id, pet_id);
    assert_eq!(batch.owner, owner);
    assert_eq!(batch.active_consents.len(), 1);
    assert!(batch.latest_medical_record.is_some());
    assert_eq!(batch.latest_medical_record.unwrap().id, record_id);
}

#[test]
fn test_get_pet_full_profile_batch_owner_access() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

    // Owner should have access
    let batch = client.get_pet_full_profile_batch(&pet_id, &owner).unwrap();

    assert_eq!(batch.profile.id, pet_id);
    assert_eq!(batch.owner, owner);
}

#[test]
fn test_get_pet_full_profile_batch_private_pet_denied() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

    // Stranger should not have access to private pet
    let result = client.get_pet_full_profile_batch(&pet_id, &stranger);

    assert!(result.is_none());
}

#[test]
fn test_get_pet_full_profile_batch_restricted_with_grant() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

    // Grant access
    client.grant_access(&owner, &pet_id, &grantee, &AccessLevel::Basic, &None);

    // Grantee should have access
    let batch = client.get_pet_full_profile_batch(&pet_id, &grantee).unwrap();

    assert_eq!(batch.profile.id, pet_id);
    assert_eq!(batch.owner, owner);
}

#[test]
fn test_get_pet_full_profile_batch_restricted_without_grant() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

    // Stranger without grant should not have access
    let result = client.get_pet_full_profile_batch(&pet_id, &stranger);

    assert!(result.is_none());
}

#[test]
fn test_get_pet_full_profile_batch_multiple_consents() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add multiple consents
    let grantee1 = Address::generate(&env);
    let grantee2 = Address::generate(&env);
    let grantee3 = Address::generate(&env);

    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee1,
        &ConsentScope::ReadMedical,
    );
    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Insurance,
        &grantee2,
        &ConsentScope::WriteMedical,
    );
    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::DataSharing,
        &grantee3,
        &ConsentScope::ReadMedical,
    );

    let batch = client.get_pet_full_profile_batch(&pet_id, &caller).unwrap();

    assert_eq!(batch.active_consents.len(), 3);
}

#[test]
fn test_get_pet_full_profile_batch_latest_medical_record() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    // Add multiple medical records
    let _record_id1 = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup 1"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "None"),
    );

    // Advance time
    env.ledger().with_mut(|li| {
        li.timestamp += 1000;
    });

    let record_id2 = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup 2"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "None"),
    );

    let batch = client.get_pet_full_profile_batch(&pet_id, &caller).unwrap();

    // Should return the most recent record
    assert!(batch.latest_medical_record.is_some());
    assert_eq!(batch.latest_medical_record.unwrap().id, record_id2);
}

#[test]
fn test_get_pet_full_profile_batch_no_medical_records() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    let batch = client.get_pet_full_profile_batch(&pet_id, &caller).unwrap();

    assert!(batch.latest_medical_record.is_none());
}

#[test]
fn test_get_pet_full_profile_batch_nonexistent_pet() {
    let (env, client, _admin) = setup();
    let caller = Address::generate(&env);

    let result = client.get_pet_full_profile_batch(&999, &caller);

    assert!(result.is_none());
}

// ============================================================================
// PetHealthSummary Tests
// ============================================================================

#[test]
fn test_get_pet_health_summary_complete() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add vaccination
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &String::from_str(&env, "BATCH123"),
    );

    // Add lab result
    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
    );

    // Add insurance policy
    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-001"),
        &String::from_str(&env, "PetInsure"),
        &String::from_str(&env, "Standard"),
        &100,
        &10000,
        &expiry,
    );

    // Get health summary
    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    assert_eq!(summary.pet_id, pet_id);
    assert!(summary.latest_vaccination.is_some());
    assert_eq!(summary.latest_vaccination.unwrap().id, vax_id);
    assert!(summary.latest_lab_result.is_some());
    assert_eq!(summary.latest_lab_result.unwrap().id, lab_id);
    assert!(summary.active_insurance_policy.is_some());
}

#[test]
fn test_get_pet_health_summary_partial_data() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add only vaccination
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &String::from_str(&env, "BATCH123"),
    );

    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    assert_eq!(summary.pet_id, pet_id);
    assert!(summary.latest_vaccination.is_some());
    assert_eq!(summary.latest_vaccination.unwrap().id, vax_id);
    assert!(summary.latest_lab_result.is_none());
    assert!(summary.active_insurance_policy.is_none());
}

#[test]
fn test_get_pet_health_summary_no_data() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    assert_eq!(summary.pet_id, pet_id);
    assert!(summary.latest_vaccination.is_none());
    assert!(summary.latest_lab_result.is_none());
    assert!(summary.active_insurance_policy.is_none());
}

#[test]
fn test_get_pet_health_summary_latest_vaccination() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    // Add first vaccination
    let _vax_id1 = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &String::from_str(&env, "BATCH123"),
    );

    // Advance time
    env.ledger().with_mut(|li| {
        li.timestamp += 1000;
    });

    // Add second vaccination
    let vax_id2 = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Parvovirus,
        &String::from_str(&env, "Parvovax"),
        &String::from_str(&env, "BATCH456"),
    );

    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    // Should return the most recent vaccination
    assert!(summary.latest_vaccination.is_some());
    assert_eq!(summary.latest_vaccination.unwrap().id, vax_id2);
}

#[test]
fn test_get_pet_health_summary_latest_lab_result() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    // Add first lab result
    let _lab_id1 = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test 1"),
        &String::from_str(&env, "Normal"),
    );

    // Advance time
    env.ledger().with_mut(|li| {
        li.timestamp += 1000;
    });

    // Add second lab result
    let lab_id2 = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test 2"),
        &String::from_str(&env, "Normal"),
    );

    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    // Should return the most recent lab result
    assert!(summary.latest_lab_result.is_some());
    assert_eq!(summary.latest_lab_result.unwrap().id, lab_id2);
}

#[test]
fn test_get_pet_health_summary_inactive_insurance() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add insurance policy
    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-001"),
        &String::from_str(&env, "PetInsure"),
        &String::from_str(&env, "Standard"),
        &100,
        &10000,
        &expiry,
    );

    // Deactivate the policy
    client.update_insurance_status(&owner, &pet_id, &String::from_str(&env, "POL-001"), &false);

    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    // Should not return inactive insurance
    assert!(summary.active_insurance_policy.is_none());
}

#[test]
fn test_get_pet_health_summary_private_pet_owner_access() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

    // Owner should have access
    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();

    assert_eq!(summary.pet_id, pet_id);
}

#[test]
fn test_get_pet_health_summary_private_pet_denied() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

    // Stranger should not have access
    let result = client.get_pet_health_summary(&pet_id, &stranger);

    assert!(result.is_none());
}

#[test]
fn test_get_pet_health_summary_restricted_with_grant() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

    // Grant access
    client.grant_access(&owner, &pet_id, &grantee, &AccessLevel::Basic, &None);

    // Grantee should have access
    let summary = client.get_pet_health_summary(&pet_id, &grantee).unwrap();

    assert_eq!(summary.pet_id, pet_id);
}

#[test]
fn test_get_pet_health_summary_restricted_without_grant() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

    // Stranger without grant should not have access
    let result = client.get_pet_health_summary(&pet_id, &stranger);

    assert!(result.is_none());
}

#[test]
fn test_get_pet_health_summary_nonexistent_pet() {
    let (env, client, _admin) = setup();
    let caller = Address::generate(&env);

    let result = client.get_pet_health_summary(&999, &caller);

    assert!(result.is_none());
}

#[test]
fn test_batch_operations_reduce_calls() {
    let (env, client, _admin) = setup();
    let owner = Address::generate(&env);
    let caller = Address::generate(&env);

    let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

    // Add data
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET123"),
        &String::from_str(&env, "General Practice"),
    );
    client.verify_vet_license(&vet, &String::from_str(&env, "VET123"));

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &String::from_str(&env, "BATCH123"),
    );

    client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "None"),
    );

    let grantee = Address::generate(&env);
    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &ConsentScope::ReadMedical,
    );

    // Single batch call gets all data
    let batch = client.get_pet_full_profile_batch(&pet_id, &caller).unwrap();
    let summary = client.get_pet_health_summary(&pet_id, &caller).unwrap();

    // Verify all data is present
    assert_eq!(batch.profile.id, pet_id);
    assert_eq!(batch.active_consents.len(), 1);
    assert!(batch.latest_medical_record.is_some());
    assert!(summary.latest_vaccination.is_some());
}
