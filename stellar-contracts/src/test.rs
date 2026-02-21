use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_register_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "Buddy");
    let birthday = String::from_str(&env, "2020-01-01");
    let breed = String::from_str(&env, "Golden Retriever");

    let pet_id = client.register_pet(
        &owner,
        &name,
        &birthday,
        &Gender::Male,
        &Species::Dog,
        &breed,
        &PrivacyLevel::Public,
    );
    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.name, name);
    assert_eq!(pet.active, false);
    assert_eq!(pet.archived, false);
}

#[test]
fn test_archive_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    assert!(client.get_pet(&pet_id).is_some());
    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id);
    assert_eq!(archived.get(0).unwrap().archived, true);
}

#[test]
fn test_archived_pets_not_in_normal_queries() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &PrivacyLevel::Public,
    );
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let by_owner = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner.len(), 2);

    client.archive_pet(&pet_id1);

    let by_owner_after = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner_after.len(), 1);
    assert_eq!(by_owner_after.get(0).unwrap().id, pet_id2);

    assert!(client.get_pet(&pet_id1).is_none());
    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id1);
}

#[test]
fn test_restore_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    client.restore_pet(&pet_id);
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.archived, false);

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 0);
}

#[test]
fn test_update_pet_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes.len(), 0);

    let notes = String::from_str(&env, "Loves fetch. Allergic to chicken. Needs daily meds at 8am.");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
#[should_panic(expected = "Notes too long")]
fn test_pet_notes_character_limit() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let long_notes = String::from_str(&env, &"x".repeat(1001));
    client.update_pet_notes(&pet_id, &long_notes);
}

#[test]
fn test_only_owner_can_update_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let notes = String::from_str(&env, "Owner notes");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
fn test_register_pet_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let emergency = String::from_str(&env, "555-1234");

    client.register_pet_owner(&owner, &name, &email, &emergency);

    let is_registered = client.is_owner_registered(&owner);
    assert_eq!(is_registered, true);
}

#[test]
fn test_record_and_get_vaccination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Who"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    let now = env.ledger().timestamp();
    let next = now + 1000;

    let vaccine_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &now,
        &next,
        &String::from_str(&env, "BATCH-001"),
    );
    assert_eq!(vaccine_id, 1u64);

    let record = client.get_vaccinations(&vaccine_id).unwrap();

    assert_eq!(record.id, 1);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.veterinarian, vet);
    assert_eq!(record.vaccine_type, VaccineType::Rabies);
    assert_eq!(
        record.batch_number,
        Some(String::from_str(&env, "BATCH-001"))
    );
    assert_eq!(
        record.vaccine_name,
        Some(String::from_str(&env, "Rabies Vaccine"))
    );
}

#[test]
fn test_link_tag_to_pet() {
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
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Verify tag was created
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.pet_id, pet_id);
    assert_eq!(tag.owner, owner);
    assert!(tag.is_active);

    // Verify bidirectional lookup works
    let retrieved_tag_id = client.get_tag_by_pet(&pet_id).unwrap();
    assert_eq!(retrieved_tag_id, tag_id);

    // Verify pet lookup by tag
    let pet = client.get_pet_by_tag(&tag_id).unwrap();
    assert_eq!(pet.id, pet_id);
}

#[test]
fn test_update_tag_message() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Update the tag message
    let message = String::from_str(&env, "If found, call 555-1234");
    let result = client.update_tag_message(&tag_id, &message);
    assert!(result);

    // Verify message was updated
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.message, message);
}

#[test]
fn test_tag_id_uniqueness() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog1"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &PrivacyLevel::Public,
    );
    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog2"),
        &String::from_str(&env, "2020-01-02"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &PrivacyLevel::Public,
    );

    let tag1 = client.link_tag_to_pet(&pet1);
    let tag2 = client.link_tag_to_pet(&pet2);

    assert_ne!(tag1, tag2);
}

#[test]
fn test_pet_privacy_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Secret Pet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "X"),
        &PrivacyLevel::Private, // Encrypted, restricted
    );

    // Owner can access (simulated by contract function always returning Profile in this implementation)
    // In real world, owner holds key. Here get_pet returns Profile.
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Secret Pet")); // Internal decryption works

    // Access control
    let user = Address::generate(&env);
    client.grant_access(&pet_id, &user, &AccessLevel::Full, &None);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);

    client.revoke_access(&pet_id, &user);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::None);
}

#[test]
fn test_vaccination_history_overdue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. What"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    // Set time to future to allow subtraction for past
    let now = 1_000_000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let past = now - 10000;

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Old Rabies"),
        &past,
        &past, // Already overdue
        &String::from_str(&env, "B1"),
    );

    let overdue = client.get_overdue_vaccinations(&pet_id);
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue.get(0).unwrap(), VaccineType::Rabies);

    assert_eq!(
        client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
        false
    );
}

#[test]
fn test_set_and_get_emergency_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Dad"),
        phone: String::from_str(&env, "111-2222"),
        email: String::from_str(&env, "dad@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &String::from_str(&env, "Allergic to bees"),
    );

    let info = client.get_emergency_info(&pet_id).unwrap();
    assert_eq!(info.0.len(), 1);
    assert_eq!(info.0.get(0).unwrap().name, String::from_str(&env, "Dad"));
    assert_eq!(info.0.get(0).unwrap().is_primary, true);
    assert_eq!(info.1, String::from_str(&env, "Allergic to bees"));
}

#[test]
fn test_lab_results() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Patient"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let test_type = String::from_str(&env, "Blood Test");
    let results = String::from_str(&env, "Glucose: 100 mg/dL");
    let reference_ranges = String::from_str(&env, "70-120 mg/dL");
    let attachment_hash = Some(String::from_str(&env, "QmXoyp..."));

    // Add a medical record to link to
    let medical_record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "None"),
        &Vec::new(&env),
    );

    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &test_type,
        &results,
        &reference_ranges,
        &attachment_hash,
        &Some(medical_record_id),
    );

    let res = client.get_lab_result(&lab_id).unwrap();
    assert_eq!(res.test_type, test_type);
    assert_eq!(res.results, results);
    assert_eq!(res.reference_ranges, reference_ranges);
    assert_eq!(res.attachment_hash, attachment_hash);
    assert_eq!(res.medical_record_id, Some(medical_record_id));
    assert_eq!(res.vet_address, vet);

    let list = client.get_lab_results(&pet_id);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_update_medical_record() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Pet"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &PrivacyLevel::Public,
    );

    let mut medications = Vec::new(&env);
    medications.push_back(Medication {
        id: 0,
        pet_id,
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "10mg"),
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: Some(200),
        prescribing_vet: vet.clone(),
        active: true,
    });

    let start_time = 1000;
    env.ledger().with_mut(|l| l.timestamp = start_time);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "Monitor"),
        &medications,
    );

    let created_record = client.get_medical_record(&record_id).unwrap();
    assert_eq!(created_record.created_at, start_time);
    assert_eq!(created_record.updated_at, start_time);

    // Advance time
    let update_time = 2000;
    env.ledger().with_mut(|l| l.timestamp = update_time);

    let mut new_meds = Vec::new(&env);
    new_meds.push_back(Medication {
        id: 0,
        pet_id,
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "20mg"), // Modified dosage
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: Some(200),
        prescribing_vet: vet.clone(),
        active: true,
    });
    new_meds.push_back(Medication {
        id: 0,
        pet_id,
        name: String::from_str(&env, "NewMed"), // New med
        dosage: String::from_str(&env, "5mg"),
        frequency: String::from_str(&env, "Once"),
        start_date: update_time,
        end_date: Some(update_time + 100),
        prescribing_vet: vet.clone(),
        active: true,
    });

    let success = client.update_medical_record(
        &record_id,
        &String::from_str(&env, "Sick"),
        &String::from_str(&env, "Intensive Care"),
        &new_meds,
    );
    assert!(success);

    let updated = client.get_medical_record(&record_id).unwrap();

    // Verify updates
    assert_eq!(updated.diagnosis, String::from_str(&env, "Sick"));
    assert_eq!(updated.treatment, String::from_str(&env, "Intensive Care"));
    assert_eq!(updated.medications.len(), 2);
    assert_eq!(
        updated.medications.get(0).unwrap().dosage,
        String::from_str(&env, "20mg")
    );
    assert_eq!(
        updated.medications.get(1).unwrap().name,
        String::from_str(&env, "NewMed")
    );
    assert_eq!(updated.updated_at, update_time);

    // Verify preserved fields
    assert_eq!(updated.id, record_id);
    assert_eq!(updated.pet_id, pet_id);
    assert_eq!(updated.veterinarian, vet);
    assert_eq!(updated.record_type, String::from_str(&env, "Checkup"));
    assert_eq!(updated.created_at, start_time);
}

#[test]
fn test_update_medical_record_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let meds = Vec::new(&env);
    let success = client.update_medical_record(
        &999,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &meds,
    );
    assert_eq!(success, false);
}

#[test]
fn test_vet_reviews() {
    let env = Env::default();
    env.mock_all_auths();
// === NEW LOST PET ALERT TESTS ===

#[test]
fn test_report_lost() {
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
        &String::from_str(&env, "Labrador"),
        &PrivacyLevel::Public,
    );

    let location = String::from_str(&env, "Central Park, NYC");
    let alert_id = client.report_lost(&pet_id, &location, &Some(500));

    assert_eq!(alert_id, 1);

    let alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(alert.pet_id, pet_id);
    assert_eq!(alert.status, AlertStatus::Active);
    assert_eq!(alert.reward_amount, Some(500));
    assert!(alert.found_date.is_none());
}

#[test]
fn test_report_found() {
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
        &String::from_str(&env, "Labrador"),
        &PrivacyLevel::Public,
    );

    let location = String::from_str(&env, "Brooklyn Bridge");
    let alert_id = client.report_lost(&pet_id, &location, &None);

    let result = client.report_found(&alert_id);
    assert!(result);

    let found_alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(found_alert.status, AlertStatus::Found);
    assert!(found_alert.found_date.is_some());

    let active = client.get_active_alerts();
    assert_eq!(active.len(), 0);
}

#[test]
fn test_cancel_lost_alert() {
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
        &String::from_str(&env, "Labrador"),
        &PrivacyLevel::Public,
    );

    let alert_id = client.report_lost(
        &pet_id,
        &String::from_str(&env, "Times Square"),
        &Some(1000),
    );

    let cancelled = client.cancel_lost_alert(&alert_id);
    assert!(cancelled);

    let alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(alert.status, AlertStatus::Cancelled);
}

#[test]
fn test_get_active_alerts() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    for _ in 0..3 {
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Pet"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Breed"),
            &PrivacyLevel::Public,
        );

        client.report_lost(
            &pet_id,
            &String::from_str(&env, "Location"),
            &None,
        );
    }

    let active = client.get_active_alerts();
    assert_eq!(active.len(), 3);

    client.report_found(&2);

    let active_after = client.get_active_alerts();
    assert_eq!(active_after.len(), 2);

    for alert in active_after.iter() {
        assert_eq!(alert.status, AlertStatus::Active);
    }
}

#[test]
fn test_sighting_report() {
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
        &String::from_str(&env, "Labrador"),
        &PrivacyLevel::Public,
    );

    let alert_id = client.report_lost(
        &pet_id,
        &String::from_str(&env, "Park"),
        &None,
    );

    client.report_sighting(
        &alert_id,
        &String::from_str(&env, "Near the fountain"),
        &String::from_str(&env, "Saw a dog matching description"),
    );

    let sightings = client.get_alert_sightings(&alert_id);
    assert_eq!(sightings.len(), 1);
    
    let sighting = sightings.get(0).unwrap();
    assert_eq!(sighting.location, String::from_str(&env, "Near the fountain"));
}

#[test]
fn test_get_pet_alerts() {
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
        &String::from_str(&env, "Labrador"),
        &PrivacyLevel::Public,
    );

    client.report_lost(&pet_id, &String::from_str(&env, "Loc1"), &None);
    client.report_lost(&pet_id, &String::from_str(&env, "Loc2"), &None);

    let pet_alerts = client.get_pet_alerts(&pet_id);
    assert_eq!(pet_alerts.len(), 2);
}
    #[test]
fn test_set_availability() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Setup vet
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    // Set availability
    let start_time = 1_000_000; // Some timestamp
    let end_time = 1_000_000 + 3600; // 1 hour slot
    let slot_index = client.set_availability(&vet, &start_time, &end_time);
    
    assert_eq!(slot_index, 1);

    // Get available slots for that date
    let date = start_time / 86400;
    let slots = client.get_available_slots(&vet, &date);
    assert_eq!(slots.len(), 1);
    
    let slot = slots.get(0).unwrap();
    assert_eq!(slot.vet_address, vet);
    assert_eq!(slot.start_time, start_time);
    assert_eq!(slot.end_time, end_time);
    assert_eq!(slot.available, true);
}

#[test]
fn test_book_slot() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Setup vet
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    // Set availability
    let start_time = 1_000_000;
    let end_time = 1_000_000 + 3600;
    let slot_index = client.set_availability(&vet, &start_time, &end_time);

    // Book the slot
    let result = client.book_slot(&vet, &slot_index);
    assert!(result);

    // Verify slot is no longer available
    let date = start_time / 86400;
    let slots = client.get_available_slots(&vet, &date);
    assert_eq!(slots.len(), 0);
}
#[test]
fn test_grant_consent() {
 #[test]
fn test_get_version() {
let env = Env::default();
let contract_id = env.register_contract(None, PetChainContract);
let client = PetChainContractClient::new(&env, &contract_id);

let version = client.get_version();
assert_eq!(version.major, 1);
assert_eq!(version.minor, 0);
assert_eq!(version.patch, 0);
}

#[test]
fn test_propose_upgrade() {
let env = Env::default();
let contract_id = env.register_contract(None, PetChainContract);
let client = PetChainContractClient::new(&env, &contract_id);
env.mock_all_auths();

let admin = Address::generate(&env);
client.init_admin(&admin);

let owner = Address::generate(&env);
let pet_id = client.register_pet(
    &owner,
    &String::from_str(&env, "Buddy"),
    &String::from_str(&env, "2020-01-01"),
    &Gender::Male,
    &Species::Dog,
    &String::from_str(&env, "Labrador"),
    &PrivacyLevel::Public,
);

let insurance_company = Address::generate(&env);
let consent_id = client.grant_consent(
    &pet_id,
    &owner,
    &ConsentType::Insurance,
    &insurance_company,
);

assert_eq!(consent_id, 1);
}

#[test]
fn test_revoke_consent() {
let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
let proposal_id = client.propose_upgrade(&admin, &wasm_hash);

assert_eq!(proposal_id, 1);

let proposal = client.get_upgrade_proposal(&proposal_id).unwrap();
assert_eq!(proposal.approved, false);
assert_eq!(proposal.executed, false);
}

#[test]
fn test_approve_upgrade() {
let env = Env::default();
let contract_id = env.register_contract(None, PetChainContract);
let client = PetChainContractClient::new(&env, &contract_id);
env.mock_all_auths();

let admin = Address::generate(&env);
client.init_admin(&admin);

let owner = Address::generate(&env);
let pet_id = client.register_pet(
    &owner,
    &String::from_str(&env, "Buddy"),
    &String::from_str(&env, "2020-01-01"),
    &Gender::Male,
    &Species::Dog,
    &String::from_str(&env, "Labrador"),
    &PrivacyLevel::Public,
);

let research_org = Address::generate(&env);
let consent_id = client.grant_consent(
    &pet_id,
    &owner,
    &ConsentType::Research,
    &research_org,
);

let revoked = client.revoke_consent(&consent_id, &owner);
assert_eq!(revoked, true);
}

#[test]
fn test_consent_history() {
let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
let proposal_id = client.propose_upgrade(&admin, &wasm_hash);

let approved = client.approve_upgrade(&proposal_id);
assert_eq!(approved, true);

let proposal = client.get_upgrade_proposal(&proposal_id).unwrap();
assert_eq!(proposal.approved, true);
}

#[test]
fn test_migrate_version() {
let env = Env::default();
let contract_id = env.register_contract(None, PetChainContract);
let client = PetChainContractClient::new(&env, &contract_id);
env.mock_all_auths();

let admin = Address::generate(&env);
client.init_admin(&admin);

let owner = Address::generate(&env);
let pet_id = client.register_pet(
    &owner,
    &String::from_str(&env, "Buddy"),
    &String::from_str(&env, "2020-01-01"),
    &Gender::Male,
    &Species::Dog,
    &String::from_str(&env, "Labrador"),
    &PrivacyLevel::Public,
);

let insurance_company = Address::generate(&env);
let research_org = Address::generate(&env);

// Grant two consents
client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &insurance_company);
client.grant_consent(&pet_id, &owner, &ConsentType::Research, &research_org);

// Revoke one
client.revoke_consent(&1u64, &owner);

let history = client.get_consent_history(&pet_id);
assert_eq!(history.len(), 2); // both still in history
assert_eq!(history.get(0).unwrap().is_active, false); // first was revoked
assert_eq!(history.get(1).unwrap().is_active, true);  // second still active
}
}
client.migrate_version(&2u32, &1u32, &0u32);

let version = client.get_version();
assert_eq!(version.major, 2);
assert_eq!(version.minor, 1);
assert_eq!(version.patch, 0);
}

#[test]
#[should_panic]
fn test_upgrade_requires_admin() {
let env = Env::default();
let contract_id = env.register_contract(None, PetChainContract);
let client = PetChainContractClient::new(&env, &contract_id);
env.mock_all_auths();

// No admin set - should panic
let wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
client.propose_upgrade(&Address::generate(&env), &wasm_hash);
}
}
#[cfg(test)]
mod test {
use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_register_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "Buddy");
    let birthday = String::from_str(&env, "2020-01-01");
    let breed = String::from_str(&env, "Golden Retriever");

    let pet_id = client.register_pet(
        &owner,
        &name,
        &birthday,
        &Gender::Male,
        &Species::Dog,
        &breed,
        &String::from_str(&env, "Golden"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );
    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.name, name);
    assert_eq!(pet.active, false);
    assert_eq!(pet.archived, false);
}

#[test]
fn test_archive_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    assert!(client.get_pet(&pet_id).is_some());
    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id);
    assert_eq!(archived.get(0).unwrap().archived, true);
}

#[test]
fn test_archived_pets_not_in_normal_queries() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &PrivacyLevel::Public,
    );
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let by_owner = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner.len(), 2);

    client.archive_pet(&pet_id1);

    let by_owner_after = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner_after.len(), 1);
    assert_eq!(by_owner_after.get(0).unwrap().id, pet_id2);

    assert!(client.get_pet(&pet_id1).is_none());
    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id1);
}

#[test]
fn test_restore_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    client.restore_pet(&pet_id);
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.archived, false);

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 0);
}

#[test]
fn test_update_pet_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes.len(), 0);

    let notes = String::from_str(&env, "Loves fetch. Allergic to chicken. Needs daily meds at 8am.");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
#[should_panic(expected = "Notes too long")]
fn test_pet_notes_character_limit() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let long_notes = String::from_str(&env, &"x".repeat(1001));
    client.update_pet_notes(&pet_id, &long_notes);
}

#[test]
fn test_only_owner_can_update_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let notes = String::from_str(&env, "Owner notes");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
fn test_register_pet_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let emergency = String::from_str(&env, "555-1234");

    client.register_pet_owner(&owner, &name, &email, &emergency);

    let is_registered = client.is_owner_registered(&owner);
    assert_eq!(is_registered, true);
}

#[test]
fn test_record_and_get_vaccination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Golden"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Who"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    let now = env.ledger().timestamp();
    let next = now + 1000;

    let vaccine_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &now,
        &next,
        &String::from_str(&env, "BATCH-001"),
    );
    assert_eq!(vaccine_id, 1u64);

    let record = client.get_vaccinations(&vaccine_id).unwrap();

    assert_eq!(record.id, 1);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.veterinarian, vet);
    assert_eq!(record.vaccine_type, VaccineType::Rabies);
    assert_eq!(
        record.batch_number,
        Some(String::from_str(&env, "BATCH-001"))
    );
    assert_eq!(
        record.vaccine_name,
        Some(String::from_str(&env, "Rabies Vaccine"))
    );
}

#[test]
fn test_link_tag_to_pet() {
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

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Verify tag was created
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.pet_id, pet_id);
    assert_eq!(tag.owner, owner);
    assert!(tag.is_active);

    // Verify bidirectional lookup works
    let retrieved_tag_id = client.get_tag_by_pet(&pet_id).unwrap();
    assert_eq!(retrieved_tag_id, tag_id);

    // Verify pet lookup by tag
    let pet = client.get_pet_by_tag(&tag_id).unwrap();
    assert_eq!(pet.id, pet_id);
}

#[test]
fn test_update_tag_message() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Update the tag message
    let message = String::from_str(&env, "If found, call 555-1234");
    let result = client.update_tag_message(&tag_id, &message);
    assert!(result);

    // Verify message was updated
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.message, message);
}

#[test]
fn test_tag_id_uniqueness() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog1"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "Gray"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog2"),
        &String::from_str(&env, "2020-01-02"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let tag1 = client.link_tag_to_pet(&pet1);
    let tag2 = client.link_tag_to_pet(&pet2);

    assert_ne!(tag1, tag2);
}

#[test]
fn test_pet_privacy_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Secret Pet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "X"),
        &String::from_str(&env, "Black"),
        &6u32,
        &None,
        &PrivacyLevel::Private, // Encrypted, restricted
    );

    // Owner can access (simulated by contract function always returning Profile in this implementation)
    // In real world, owner holds key. Here get_pet returns Profile.
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Secret Pet")); // Internal decryption works

    // Access control
    let user = Address::generate(&env);
    client.grant_access(&pet_id, &user, &AccessLevel::Full, &None);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);

    client.revoke_access(&pet_id, &user);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::None);
}

#[test]
fn test_vaccination_history_overdue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. What"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    // Set time to future to allow subtraction for past
    let now = 1_000_000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let past = now - 10000;

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Old Rabies"),
        &past,
        &past, // Already overdue
        &String::from_str(&env, "B1"),
    );

    let overdue = client.get_overdue_vaccinations(&pet_id);
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue.get(0).unwrap(), VaccineType::Rabies);

    assert_eq!(
        client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
        false
    );
}

#[test]
fn test_set_and_get_emergency_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Yellow"),
        &32u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Dad"),
        phone: String::from_str(&env, "111-2222"),
        email: String::from_str(&env, "dad@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &String::from_str(&env, "Allergic to bees"),
    );

    let info = client.get_emergency_info(&pet_id).unwrap();
    assert_eq!(info.0.len(), 1);
    assert_eq!(info.0.get(0).unwrap().name, String::from_str(&env, "Dad"));
    assert_eq!(info.0.get(0).unwrap().is_primary, true);
    assert_eq!(info.1, String::from_str(&env, "Allergic to bees"));
}

#[test]
fn test_lab_results() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Patient"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Point"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );

    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
        &None,
    );

    let res = client.get_lab_result(&lab_id).unwrap();
    assert_eq!(res.test_type, String::from_str(&env, "Blood Test"));
    assert_eq!(res.result_summary, String::from_str(&env, "Normal"));

    let list = client.get_pet_lab_results(&pet_id);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_update_medical_record() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Pet"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Brown"),
        &18u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut medications = Vec::new(&env);
    medications.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "10mg"),
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let start_time = 1000;
    env.ledger().with_mut(|l| l.timestamp = start_time);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "Monitor"),
        &medications,
    );

    let created_record = client.get_medical_record(&record_id).unwrap();
    assert_eq!(created_record.created_at, start_time);
    assert_eq!(created_record.updated_at, start_time);

    // Advance time
    let update_time = 2000;
    env.ledger().with_mut(|l| l.timestamp = update_time);

    let mut new_meds = Vec::new(&env);
    new_meds.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "20mg"), // Modified dosage
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });
    new_meds.push_back(Medication {
        name: String::from_str(&env, "NewMed"), // New med
        dosage: String::from_str(&env, "5mg"),
        frequency: String::from_str(&env, "Once"),
        start_date: update_time,
        end_date: update_time + 100,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let success = client.update_medical_record(
        &record_id,
        &String::from_str(&env, "Sick"),
        &String::from_str(&env, "Intensive Care"),
        &new_meds,
    );
    assert!(success);

    let updated = client.get_medical_record(&record_id).unwrap();

    // Verify updates
    assert_eq!(updated.diagnosis, String::from_str(&env, "Sick"));
    assert_eq!(updated.treatment, String::from_str(&env, "Intensive Care"));
    assert_eq!(updated.medications.len(), 2);
    assert_eq!(
        updated.medications.get(0).unwrap().dosage,
        String::from_str(&env, "20mg")
    );
    assert_eq!(
        updated.medications.get(1).unwrap().name,
        String::from_str(&env, "NewMed")
    );
    assert_eq!(updated.updated_at, update_time);

    // Verify preserved fields
    assert_eq!(updated.id, record_id);
    assert_eq!(updated.pet_id, pet_id);
    assert_eq!(updated.veterinarian, vet);
    assert_eq!(updated.record_type, String::from_str(&env, "Checkup"));
    assert_eq!(updated.created_at, start_time);
}

#[test]
fn test_update_medical_record_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let meds = Vec::new(&env);
    let success = client.update_medical_record(
        &999,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &meds,
    );
    assert_eq!(success, false);
}

#[test]
fn test_register_pet_with_all_new_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Chip"),
        &String::from_str(&env, "2023-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador Retriever"),
        &String::from_str(&env, "Chocolate"),
        &35u32,
        &Some(String::from_str(&env, "982000123456789")),
        &PrivacyLevel::Public,
    );

    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.birthday, String::from_str(&env, "2023-06-15"));
    assert_eq!(pet.breed, String::from_str(&env, "Labrador Retriever"));
    assert_eq!(pet.gender, Gender::Male);
    assert_eq!(pet.color, String::from_str(&env, "Chocolate"));
    assert_eq!(pet.weight, 35);
    assert_eq!(
        pet.microchip_id,
        Some(String::from_str(&env, "982000123456789"))
    );
}

#[test]
fn test_update_pet_profile() {
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
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    let success = client.update_pet_profile(
        &pet_id,
        &String::from_str(&env, "Buddy Updated"),
        &String::from_str(&env, "2020-01-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever Mix"),
        &String::from_str(&env, "Golden Brown"),
        &22u32,
        &Some(String::from_str(&env, "123456789012345")),
        &PrivacyLevel::Public,
    );
    assert!(success);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Buddy Updated"));
    assert_eq!(pet.birthday, String::from_str(&env, "2020-01-15"));
    assert_eq!(pet.breed, String::from_str(&env, "Golden Retriever Mix"));
    assert_eq!(pet.color, String::from_str(&env, "Golden Brown"));
    assert_eq!(pet.weight, 22);
    assert_eq!(
        pet.microchip_id,
        Some(String::from_str(&env, "123456789012345"))
    );
}

#[test]
fn test_gender_enum_values() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_male = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Black"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_male_profile = client.get_pet(&pet_male).unwrap();
    assert_eq!(pet_male_profile.gender, Gender::Male);

    let pet_female = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "White"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_female_profile = client.get_pet(&pet_female).unwrap();
    assert_eq!(pet_female_profile.gender, Gender::Female);

    let pet_unknown = client.register_pet(
        &owner,
        &String::from_str(&env, "Unknown"),
        &String::from_str(&env, "2022-01-01"),
        &Gender::Unknown,
        &Species::Bird,
        &String::from_str(&env, "Parakeet"),
        &String::from_str(&env, "Green"),
        &1u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_unknown_profile = client.get_pet(&pet_unknown).unwrap();
    assert_eq!(pet_unknown_profile.gender, Gender::Unknown);
}

#[test]
fn test_get_pets_by_owner() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Golden"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let pets = client.get_pets_by_owner(&owner);
    assert_eq!(pets.len(), 2);

    let other_owner = Address::generate(&env);
    let other_pets = client.get_pets_by_owner(&other_owner);
    assert_eq!(other_pets.len(), 0);
}

#[test]
fn test_get_pets_by_species() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Golden"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Whiskers"),
        &String::from_str(&env, "2022-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Tabby"),
        &String::from_str(&env, "Orange"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );

    let dogs = client.get_pets_by_species(&String::from_str(&env, "Dog"));
    assert_eq!(dogs.len(), 2);

    let cats = client.get_pets_by_species(&String::from_str(&env, "Cat"));
    assert_eq!(cats.len(), 1);

    let birds = client.get_pets_by_species(&String::from_str(&env, "Bird"));
    assert_eq!(birds.len(), 0);
}

#[test]
fn test_search_empty_results() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    // No pets registered - all searches return empty
    let pets_by_owner = client.get_pets_by_owner(&owner);
    assert_eq!(pets_by_owner.len(), 0);

    let pets_by_species = client.get_pets_by_species(&String::from_str(&env, "Dog"));
    assert_eq!(pets_by_species.len(), 0);

    let active_pets = client.get_active_pets();
    assert_eq!(active_pets.len(), 0);
}

#[test]
fn test_get_active_pets() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Golden"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Initially no pets are active
    let active = client.get_active_pets();
    assert_eq!(active.len(), 0);

    // Activate one pet
    client.activate_pet(&pet1);

    let active = client.get_active_pets();
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(0).unwrap().id, pet1);

    // Activate second pet
    client.activate_pet(&pet2);

    let active = client.get_active_pets();
    assert_eq!(active.len(), 2);
}
}
#[cfg(test)]
mod test {
use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_register_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "Buddy");
    let birthday = String::from_str(&env, "2020-01-01");
    let breed = String::from_str(&env, "Golden Retriever");

    let pet_id = client.register_pet(
        &owner,
        &name,
        &birthday,
        &Gender::Male,
        &Species::Dog,
        &breed,
        &PrivacyLevel::Public,
    );
    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.name, name);
    assert_eq!(pet.active, false);
    assert_eq!(pet.archived, false);
}

#[test]
fn test_archive_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    assert!(client.get_pet(&pet_id).is_some());
    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id);
    assert_eq!(archived.get(0).unwrap().archived, true);
}

#[test]
fn test_archived_pets_not_in_normal_queries() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &PrivacyLevel::Public,
    );
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let by_owner = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner.len(), 2);

    client.archive_pet(&pet_id1);

    let by_owner_after = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner_after.len(), 1);
    assert_eq!(by_owner_after.get(0).unwrap().id, pet_id2);

    assert!(client.get_pet(&pet_id1).is_none());
    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id1);
}

#[test]
fn test_restore_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    client.restore_pet(&pet_id);
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.archived, false);

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 0);
}

#[test]
fn test_update_pet_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes.len(), 0);

    let notes = String::from_str(&env, "Loves fetch. Allergic to chicken. Needs daily meds at 8am.");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
#[should_panic(expected = "Notes too long")]
fn test_pet_notes_character_limit() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let long_notes = String::from_str(&env, &"x".repeat(1001));
    client.update_pet_notes(&pet_id, &long_notes);
}

#[test]
fn test_only_owner_can_update_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let notes = String::from_str(&env, "Owner notes");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
fn test_register_pet_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let emergency = String::from_str(&env, "555-1234");

    client.register_pet_owner(&owner, &name, &email, &emergency);

    let is_registered = client.is_owner_registered(&owner);
    assert_eq!(is_registered, true);
}

#[test]
fn test_record_and_get_vaccination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Who"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let now = env.ledger().timestamp();
    let next = now + 1000;

    let vaccine_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &now,
        &next,
        &String::from_str(&env, "BATCH-001"),
    );
    assert_eq!(vaccine_id, 1u64);

    let record = client.get_vaccinations(&vaccine_id).unwrap();

    assert_eq!(record.id, 1);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.veterinarian, vet);
    assert_eq!(record.vaccine_type, VaccineType::Rabies);
    assert_eq!(
        record.batch_number,
        Some(String::from_str(&env, "BATCH-001"))
    );
    assert_eq!(
        record.vaccine_name,
        Some(String::from_str(&env, "Rabies Vaccine"))
    );
}

#[test]
fn test_link_tag_to_pet() {
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
        &String::from_str(&env, "Retriever"),
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Verify tag was created
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.pet_id, pet_id);
    assert_eq!(tag.owner, owner);
    assert!(tag.is_active);

    // Verify bidirectional lookup works
    let retrieved_tag_id = client.get_tag_by_pet(&pet_id).unwrap();
    assert_eq!(retrieved_tag_id, tag_id);

    // Verify pet lookup by tag
    let pet = client.get_pet_by_tag(&tag_id).unwrap();
    assert_eq!(pet.id, pet_id);
}

#[test]
fn test_update_tag_message() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Update the tag message
    let message = String::from_str(&env, "If found, call 555-1234");
    let result = client.update_tag_message(&tag_id, &message);
    assert!(result);

    // Verify message was updated
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.message, message);
}

#[test]
fn test_tag_id_uniqueness() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog1"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &PrivacyLevel::Public,
    );
    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog2"),
        &String::from_str(&env, "2020-01-02"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &PrivacyLevel::Public,
    );

    let tag1 = client.link_tag_to_pet(&pet1);
    let tag2 = client.link_tag_to_pet(&pet2);

    assert_ne!(tag1, tag2);
}

#[test]
fn test_pet_privacy_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Secret Pet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "X"),
        &PrivacyLevel::Private, // Encrypted, restricted
    );

    // Owner can access (simulated by contract function always returning Profile in this implementation)
    // In real world, owner holds key. Here get_pet returns Profile.
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Secret Pet")); // Internal decryption works

    // Access control
    let user = Address::generate(&env);
    client.grant_access(&pet_id, &user, &AccessLevel::Full, &None);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);

    client.revoke_access(&pet_id, &user);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::None);
}

#[test]
fn test_vaccination_history_overdue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. What"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Set time to future to allow subtraction for past
    let now = 1_000_000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let past = now - 10000;

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Old Rabies"),
        &past,
        &past, // Already overdue
        &String::from_str(&env, "B1"),
    );

    let overdue = client.get_overdue_vaccinations(&pet_id);
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue.get(0).unwrap(), VaccineType::Rabies);

    assert_eq!(
        client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
        false
    );
}

#[test]
fn test_set_and_get_emergency_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Dad"),
        phone: String::from_str(&env, "111-2222"),
        email: String::from_str(&env, "dad@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &String::from_str(&env, "Allergic to bees"),
    );

    let info = client.get_emergency_info(&pet_id).unwrap();
    assert_eq!(info.0.len(), 1);
    assert_eq!(info.0.get(0).unwrap().name, String::from_str(&env, "Dad"));
    assert_eq!(info.0.get(0).unwrap().is_primary, true);
    assert_eq!(info.1, String::from_str(&env, "Allergic to bees"));
}

#[test]
fn test_lab_results() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Patient"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
        &None,
    );

    let res = client.get_lab_result(&lab_id).unwrap();
    assert_eq!(res.test_type, String::from_str(&env, "Blood Test"));
    assert_eq!(res.result_summary, String::from_str(&env, "Normal"));

    let list = list_pet_lab_results(&env, &contract_id, &pet_id);
    assert_eq!(list.len(), 1);
}

fn list_pet_lab_results(env: &Env, contract_id: &Address, pet_id: &u64) -> Vec<LabResult> {
     let client = PetChainContractClient::new(&env, &contract_id);
     client.get_pet_lab_results(pet_id)
}

#[test]
fn test_update_medical_record() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Pet"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &PrivacyLevel::Public,
    );

    let mut medications = Vec::new(&env);
    medications.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "10mg"),
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let start_time = 1000;
    env.ledger().with_mut(|l| l.timestamp = start_time);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "Monitor"),
        &medications,
    );

    let created_record = client.get_medical_record(&record_id).unwrap();
    assert_eq!(created_record.created_at, start_time);
    assert_eq!(created_record.updated_at, start_time);

    // Advance time
    let update_time = 2000;
    env.ledger().with_mut(|l| l.timestamp = update_time);

    let mut new_meds = Vec::new(&env);
    new_meds.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "20mg"), // Modified dosage
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });
    new_meds.push_back(Medication {
        name: String::from_str(&env, "NewMed"), // New med
        dosage: String::from_str(&env, "5mg"),
        frequency: String::from_str(&env, "Once"),
        start_date: update_time,
        end_date: update_time + 100,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let success = client.update_medical_record(
        &record_id,
        &String::from_str(&env, "Sick"),
        &String::from_str(&env, "Intensive Care"),
        &new_meds,
    );
    assert!(success);

    let updated = client.get_medical_record(&record_id).unwrap();

    // Verify updates
    assert_eq!(updated.diagnosis, String::from_str(&env, "Sick"));
    assert_eq!(updated.treatment, String::from_str(&env, "Intensive Care"));
    assert_eq!(updated.medications.len(), 2);
    assert_eq!(
        updated.medications.get(0).unwrap().dosage,
        String::from_str(&env, "20mg")
    );
    assert_eq!(
        updated.medications.get(1).unwrap().name,
        String::from_str(&env, "NewMed")
    );
    assert_eq!(updated.updated_at, update_time);

    // Verify preserved fields
    assert_eq!(updated.id, record_id);
    assert_eq!(updated.pet_id, pet_id);
    assert_eq!(updated.veterinarian, vet);
    assert_eq!(updated.record_type, String::from_str(&env, "Checkup"));
    assert_eq!(updated.created_at, start_time);
}

#[test]
fn test_update_medical_record_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let meds = Vec::new(&env);
    let success = client.update_medical_record(
        &999,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &meds,
    );
    assert_eq!(success, false);
}

#[test]
fn test_ownership_history_tracking() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    
    let start_time = 1000;
    env.ledger().with_mut(|l| l.timestamp = start_time);

    let pet_id = client.register_pet(
        &owner1,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &PrivacyLevel::Public,
    );

    // Verify initial registration history
    let history = client.get_ownership_history(&pet_id);
    assert_eq!(history.len(), 1);
    let reg_record = history.get(0).unwrap();
    assert_eq!(reg_record.previous_owner, owner1);
    assert_eq!(reg_record.new_owner, owner1);
    assert_eq!(reg_record.transfer_date, start_time);
    assert_eq!(reg_record.transfer_reason, String::from_str(&env, "Initial Registration"));

    // Transfer ownership
    let transfer_time = 2000;
    env.ledger().with_mut(|l| l.timestamp = transfer_time);
    
    client.transfer_pet_ownership(&pet_id, &owner2);
    // History shouldn't change yet as transfer is not accepted
    assert_eq!(client.get_ownership_history(&pet_id).len(), 1);

    // Accept transfer
    let accept_time = 3000;
    env.ledger().with_mut(|l| l.timestamp = accept_time);
    client.accept_pet_transfer(&pet_id);

    // Verify updated history
    let history = client.get_ownership_history(&pet_id);
    assert_eq!(history.len(), 2);
    
    // Test chronological order
    let record2 = history.get(1).unwrap();
    assert_eq!(record2.previous_owner, owner1);
    assert_eq!(record2.new_owner, owner2);
    assert_eq!(record2.transfer_date, accept_time);
    assert_eq!(record2.transfer_reason, String::from_str(&env, "Ownership Transfer"));
    
    assert!(history.get(0).unwrap().transfer_date < history.get(1).unwrap().transfer_date);
}

#[test]
fn test_multisig_workflow() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let vet = Address::generate(&env);

    // Initialize Multisig: 2 of 3
    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());
    admins.push_back(admin3.clone());
    
    client.init_multisig(&admin1, &admins, &2);

    // 1. Propose action (VerifyVet)
    let action = ProposalAction::VerifyVet(vet.clone());
    let proposal_id = client.propose_action(&admin1, &action, &3600); // Expires in 1 hour
    
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.approvals.len(), 1); // Proposer counts as 1
    assert_eq!(proposal.required_approvals, 2);
    
    // Register vet first (so it can be verified)
    client.register_vet(&vet, &String::from_str(&env, "Dr. Multi"), &String::from_str(&env, "LIC-999"), &String::from_str(&env, "Expert"));
    assert!(!client.is_verified_vet(&vet));

    // 2. Approve by admin2
    client.approve_proposal(&admin2, &proposal_id);
    
    // 3. Try execute by anyone (threshold met)
    client.execute_proposal(&proposal_id);
    
    // Verify action was executed
    assert!(client.is_verified_vet(&vet));
    
    let proposal_after = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal_after.executed);
}

#[test]
#[should_panic]
fn test_multisig_threshold_not_met() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());
    
    client.init_multisig(&admin1, &admins, &2);

    let action = ProposalAction::VerifyVet(Address::generate(&env));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    
    // Only 1 approval (proposer), execution should fail
    client.execute_proposal(&proposal_id);
}

#[test]
#[should_panic]
fn test_multisig_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    
    client.init_multisig(&admin1, &admins, &1);

    let action = ProposalAction::VerifyVet(Address::generate(&env));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    
    // Advance time past expiry
    env.ledger().with_mut(|l| l.timestamp = env.ledger().timestamp() + 3601);
    
    client.execute_proposal(&proposal_id);
}
}
#[cfg(test)]
mod test {
use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_register_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "Buddy");
    let birthday = String::from_str(&env, "2020-01-01");
    let breed = String::from_str(&env, "Golden Retriever");

    let pet_id = client.register_pet(
        &owner,
        &name,
        &birthday,
        &Gender::Male,
        &Species::Dog,
        &breed,
        &String::from_str(&env, "Golden"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );
    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.name, name);
    assert_eq!(pet.active, false);
    assert_eq!(pet.archived, false);
}

#[test]
fn test_archive_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    assert!(client.get_pet(&pet_id).is_some());
    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id);
    assert_eq!(archived.get(0).unwrap().archived, true);
}

#[test]
fn test_archived_pets_not_in_normal_queries() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &PrivacyLevel::Public,
    );
    let pet_id2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let by_owner = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner.len(), 2);

    client.archive_pet(&pet_id1);

    let by_owner_after = client.get_pets_by_owner(&owner);
    assert_eq!(by_owner_after.len(), 1);
    assert_eq!(by_owner_after.get(0).unwrap().id, pet_id2);

    assert!(client.get_pet(&pet_id1).is_none());
    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 1);
    assert_eq!(archived.get(0).unwrap().id, pet_id1);
}

#[test]
fn test_restore_pet() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    client.archive_pet(&pet_id);
    assert!(client.get_pet(&pet_id).is_none());

    client.restore_pet(&pet_id);
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.archived, false);

    let archived = client.get_archived_pets(&owner);
    assert_eq!(archived.len(), 0);
}

#[test]
fn test_update_pet_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes.len(), 0);

    let notes = String::from_str(&env, "Loves fetch. Allergic to chicken. Needs daily meds at 8am.");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
#[should_panic(expected = "Notes too long")]
fn test_pet_notes_character_limit() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let long_notes = String::from_str(&env, &"x".repeat(1001));
    client.update_pet_notes(&pet_id, &long_notes);
}

#[test]
fn test_only_owner_can_update_notes() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

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
        &PrivacyLevel::Public,
    );

    let notes = String::from_str(&env, "Owner notes");
    client.update_pet_notes(&pet_id, &notes);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.notes, notes);
}

#[test]
fn test_register_pet_owner() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let email = String::from_str(&env, "john@example.com");
    let emergency = String::from_str(&env, "555-1234");

    client.register_pet_owner(&owner, &name, &email, &emergency);

    let is_registered = client.is_owner_registered(&owner);
    assert_eq!(is_registered, true);
}

#[test]
fn test_record_and_get_vaccination() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Golden"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Who"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    let now = env.ledger().timestamp();
    let next = now + 1000;

    let vaccine_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &now,
        &next,
        &String::from_str(&env, "BATCH-001"),
    );
    assert_eq!(vaccine_id, 1u64);

    let record = client.get_vaccinations(&vaccine_id).unwrap();

    assert_eq!(record.id, 1);
    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.veterinarian, vet);
    assert_eq!(record.vaccine_type, VaccineType::Rabies);
    assert_eq!(
        record.batch_number,
        Some(String::from_str(&env, "BATCH-001"))
    );
    assert_eq!(
        record.vaccine_name,
        Some(String::from_str(&env, "Rabies Vaccine"))
    );
}

#[test]
fn test_link_tag_to_pet() {
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

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Verify tag was created
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.pet_id, pet_id);
    assert_eq!(tag.owner, owner);
    assert!(tag.is_active);

    // Verify bidirectional lookup works
    let retrieved_tag_id = client.get_tag_by_pet(&pet_id).unwrap();
    assert_eq!(retrieved_tag_id, tag_id);

    // Verify pet lookup by tag
    let pet = client.get_pet_by_tag(&tag_id).unwrap();
    assert_eq!(pet.id, pet_id);
}

#[test]
fn test_update_tag_message() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let tag_id = client.link_tag_to_pet(&pet_id);

    // Update the tag message
    let message = String::from_str(&env, "If found, call 555-1234");
    let result = client.update_tag_message(&tag_id, &message);
    assert!(result);

    // Verify message was updated
    let tag = client.get_tag(&tag_id).unwrap();
    assert_eq!(tag.message, message);
}

#[test]
fn test_tag_id_uniqueness() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog1"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "Gray"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Dog2"),
        &String::from_str(&env, "2020-01-02"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "White"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let tag1 = client.link_tag_to_pet(&pet1);
    let tag2 = client.link_tag_to_pet(&pet2);

    assert_ne!(tag1, tag2);
}

#[test]
fn test_pet_privacy_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Secret Pet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "X"),
        &String::from_str(&env, "Black"),
        &6u32,
        &None,
        &PrivacyLevel::Private, // Encrypted, restricted
    );

    // Owner can access (simulated by contract function always returning Profile in this implementation)
    // In real world, owner holds key. Here get_pet returns Profile.
    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Secret Pet")); // Internal decryption works

    // Access control
    let user = Address::generate(&env);
    client.grant_access(&pet_id, &user, &AccessLevel::Full, &None);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);

    client.revoke_access(&pet_id, &user);
    assert_eq!(client.check_access(&pet_id, &user), AccessLevel::None);
}

#[test]
fn test_vaccination_history_overdue() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Add a review
    let review_id = client.add_vet_review(
        &owner,
        &vet,
        &5,
        &String::from_str(&env, "Great vet!"),
    );
    assert_eq!(review_id, 1);

    // Get reviews
    let reviews = client.get_vet_reviews(&vet);
    assert_eq!(reviews.len(), 1);
    let review = reviews.get(0).unwrap();
    assert_eq!(review.rating, 5);
    assert_eq!(review.comment, String::from_str(&env, "Great vet!"));
    assert_eq!(review.reviewer, owner);

    // Check average
    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 5);

    // Add another review from different owner
    let owner2 = Address::generate(&env);
    client.add_vet_review(
        &owner2,
        &vet,
        &3,
        &String::from_str(&env, "Okay"),
    );

    let avg = client.get_vet_average_rating(&vet);
    assert_eq!(avg, 4); // (5+3)/2 = 8/2 = 4
}

#[test]
#[should_panic(expected = "You have already reviewed this veterinarian")]
fn test_duplicate_vet_review() {
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. What"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&vet);

    // Set time to future to allow subtraction for past
    let now = 1_000_000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let past = now - 10000;

    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Old Rabies"),
        &past,
        &past, // Already overdue
        &String::from_str(&env, "B1"),
    );

    let overdue = client.get_overdue_vaccinations(&pet_id);
    assert_eq!(overdue.len(), 1);
    assert_eq!(overdue.get(0).unwrap(), VaccineType::Rabies);

    assert_eq!(
        client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
        false
    );
}

#[test]
fn test_set_and_get_emergency_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.add_vet_review(&owner, &vet, &5, &String::from_str(&env, "Good"));
    client.add_vet_review(&owner, &vet, &4, &String::from_str(&env, "Bad"));
}

#[test]
#[should_panic(expected = "Rating must be between 1 and 5")]
fn test_invalid_rating() {
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Yellow"),
        &32u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Dad"),
        phone: String::from_str(&env, "111-2222"),
        email: String::from_str(&env, "dad@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &String::from_str(&env, "Allergic to bees"),
    );

    let info = client.get_emergency_info(&pet_id).unwrap();
    assert_eq!(info.0.len(), 1);
    assert_eq!(info.0.get(0).unwrap().name, String::from_str(&env, "Dad"));
    assert_eq!(info.0.get(0).unwrap().is_primary, true);
    assert_eq!(info.1, String::from_str(&env, "Allergic to bees"));
}

#[test]
fn test_lab_results() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.add_vet_review(&owner, &vet, &6, &String::from_str(&env, "Fake"));
}

#[test]
fn test_standalone_medications() {
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Patient"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Point"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );

    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
        &None,
    );

    let res = client.get_lab_result(&lab_id).unwrap();
    assert_eq!(res.test_type, String::from_str(&env, "Blood Test"));
    assert_eq!(res.result_summary, String::from_str(&env, "Normal"));

    let list = client.get_pet_lab_results(&pet_id);
    assert_eq!(list.len(), 1);
}

#[test]
fn test_update_medical_record() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &PrivacyLevel::Public,
    );

    let now = 1000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let med_id = client.add_medication(
        &pet_id,
        &String::from_str(&env, "Apoquel"),
        &String::from_str(&env, "5.4mg"),
        &String::from_str(&env, "1x daily"),
        &now,
        &None,
        &vet,
    );

    let active = client.get_active_medications(&pet_id);
    assert_eq!(active.len(), 1);
    let med = active.get(0).unwrap();
    assert_eq!(med.id, med_id);
    assert_eq!(med.name, String::from_str(&env, "Apoquel"));
    assert!(med.active);
}

#[test]
fn test_mark_medication_completed() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let vet = Address::generate(&env);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Pet"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Brown"),
        &18u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut medications = Vec::new(&env);
    medications.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "10mg"),
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let start_time = 1000;
    env.ledger().with_mut(|l| l.timestamp = start_time);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "Monitor"),
        &medications,
    );

    let created_record = client.get_medical_record(&record_id).unwrap();
    assert_eq!(created_record.created_at, start_time);
    assert_eq!(created_record.updated_at, start_time);

    // Advance time
    let update_time = 2000;
    env.ledger().with_mut(|l| l.timestamp = update_time);

    let mut new_meds = Vec::new(&env);
    new_meds.push_back(Medication {
        name: String::from_str(&env, "Med1"),
        dosage: String::from_str(&env, "20mg"), // Modified dosage
        frequency: String::from_str(&env, "Daily"),
        start_date: 100,
        end_date: 200,
        prescribing_vet: vet.clone(),
        active: true,
    });
    new_meds.push_back(Medication {
        name: String::from_str(&env, "NewMed"), // New med
        dosage: String::from_str(&env, "5mg"),
        frequency: String::from_str(&env, "Once"),
        start_date: update_time,
        end_date: update_time + 100,
        prescribing_vet: vet.clone(),
        active: true,
    });

    let success = client.update_medical_record(
        &record_id,
        &String::from_str(&env, "Sick"),
        &String::from_str(&env, "Intensive Care"),
        &new_meds,
    );
    assert!(success);

    let updated = client.get_medical_record(&record_id).unwrap();

    // Verify updates
    assert_eq!(updated.diagnosis, String::from_str(&env, "Sick"));
    assert_eq!(updated.treatment, String::from_str(&env, "Intensive Care"));
    assert_eq!(updated.medications.len(), 2);
    assert_eq!(
        updated.medications.get(0).unwrap().dosage,
        String::from_str(&env, "20mg")
    );
    assert_eq!(
        updated.medications.get(1).unwrap().name,
        String::from_str(&env, "NewMed")
    );
    assert_eq!(updated.updated_at, update_time);

    // Verify preserved fields
    assert_eq!(updated.id, record_id);
    assert_eq!(updated.pet_id, pet_id);
    assert_eq!(updated.veterinarian, vet);
    assert_eq!(updated.record_type, String::from_str(&env, "Checkup"));
    assert_eq!(updated.created_at, start_time);
}

#[test]
fn test_update_medical_record_nonexistent() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let meds = Vec::new(&env);
    let success = client.update_medical_record(
        &999,
        &String::from_str(&env, "Diag"),
        &String::from_str(&env, "Treat"),
        &meds,
    );
    assert_eq!(success, false);
}

#[test]
fn test_register_pet_with_all_new_fields() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &PrivacyLevel::Public,
    );

    let now = 1000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let med_id = client.add_medication(
        &pet_id,
        &String::from_str(&env, "TestMed"),
        &String::from_str(&env, "10mg"),
        &String::from_str(&env, "Daily"),
        &now,
        &None,
        &vet,
    );

    client.mark_medication_completed(&med_id);

    let active = client.get_active_medications(&pet_id);
    assert_eq!(active.len(), 0);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &PrivacyLevel::Public,
    );

    // Valid IPFS CIDv0 hash (46 chars)
    let photo_hash = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let success = client.add_pet_photo(&pet_id, &photo_hash);
    assert!(success);

    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 1);
    assert_eq!(photos.get(0).unwrap(), photo_hash);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.photo_hashes.len(), 1);
}

#[test]
fn test_add_multiple_pet_photos() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &PrivacyLevel::Public,
    );

    let hash1 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let hash2 = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    client.add_pet_photo(&pet_id, &hash1);
    client.add_pet_photo(&pet_id, &hash2);

    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 2);
    assert_eq!(photos.get(0).unwrap(), hash1);
    assert_eq!(photos.get(1).unwrap(), hash2);
}

#[test]
#[should_panic(expected = "Invalid IPFS hash")]
fn test_add_pet_photo_invalid_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Chip"),
        &String::from_str(&env, "2023-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador Retriever"),
        &String::from_str(&env, "Chocolate"),
        &35u32,
        &Some(String::from_str(&env, "982000123456789")),
        &PrivacyLevel::Public,
    );

    assert_eq!(pet_id, 1);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, 1);
    assert_eq!(pet.birthday, String::from_str(&env, "2023-06-15"));
    assert_eq!(pet.breed, String::from_str(&env, "Labrador Retriever"));
    assert_eq!(pet.gender, Gender::Male);
    assert_eq!(pet.color, String::from_str(&env, "Chocolate"));
    assert_eq!(pet.weight, 35);
    assert_eq!(
        pet.microchip_id,
        Some(String::from_str(&env, "982000123456789"))
    );
}

#[test]
fn test_update_pet_profile() {
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
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    let success = client.update_pet_profile(
        &pet_id,
        &String::from_str(&env, "Buddy Updated"),
        &String::from_str(&env, "2020-01-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever Mix"),
        &String::from_str(&env, "Golden Brown"),
        &22u32,
        &Some(String::from_str(&env, "123456789012345")),
        &PrivacyLevel::Public,
    );
    assert!(success);

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.name, String::from_str(&env, "Buddy Updated"));
    assert_eq!(pet.birthday, String::from_str(&env, "2020-01-15"));
    assert_eq!(pet.breed, String::from_str(&env, "Golden Retriever Mix"));
    assert_eq!(pet.color, String::from_str(&env, "Golden Brown"));
    assert_eq!(pet.weight, 22);
    assert_eq!(
        pet.microchip_id,
        Some(String::from_str(&env, "123456789012345"))
    );
}

#[test]
fn test_gender_enum_values() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_male = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Black"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_male_profile = client.get_pet(&pet_male).unwrap();
    assert_eq!(pet_male_profile.gender, Gender::Male);

    let pet_female = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "White"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_female_profile = client.get_pet(&pet_female).unwrap();
    assert_eq!(pet_female_profile.gender, Gender::Female);

    let pet_unknown = client.register_pet(
        &owner,
        &String::from_str(&env, "Unknown"),
        &String::from_str(&env, "2022-01-01"),
        &Gender::Unknown,
        &Species::Bird,
        &String::from_str(&env, "Parakeet"),
        &String::from_str(&env, "Green"),
        &1u32,
        &None,
        &PrivacyLevel::Public,
    );
    let pet_unknown_profile = client.get_pet(&pet_unknown).unwrap();
    assert_eq!(pet_unknown_profile.gender, Gender::Unknown);
}
}
