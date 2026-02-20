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
            &PrivacyLevel::Public,
        );

        let mut contacts = Vec::new(&env);
        contacts.push_back(EmergencyContactInfo {
            name: String::from_str(&env, "Dad"),
            phone: String::from_str(&env, "111-2222"),
            relationship: String::from_str(&env, "Owner"),
        });

        client.set_emergency_contacts(
            &pet_id,
            &contacts,
            &String::from_str(&env, "Allergic to bees"),
        );

        let info = client.get_emergency_info(&pet_id).unwrap();
        assert_eq!(info.0.len(), 1);
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
}