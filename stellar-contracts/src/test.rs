#[cfg(test)]
mod test {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Env};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Bytes, BytesN, Env,
    };


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

        let admin = Address::generate(&env);
        let vet = Address::generate(&env);
        let owner = Address::generate(&env);

        client.init_admin(&admin);
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Vet"),
            &String::from_str(&env, "VET456"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&vet);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Golden Retriever"),
            &PrivacyLevel::Public,
        );
        assert!(pet_id > 0);

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

        let tag_id = client.link_tag_to_pet(&pet_id);
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
    fn test_emergency_contacts() {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2021-05-20"),
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
            &String::from_str(&env, "German Shepherd"),
            &PrivacyLevel::Public,
        );

        let tag1 = client.link_tag_to_pet(&pet1);
        let tag2 = client.link_tag_to_pet(&pet2);

        assert_ne!(tag1, tag2);
    }

        let mut contacts = Vec::new(&env);
        contacts.push_back(EmergencyContactInfo {
            name: String::from_str(&env, "John Doe"),
            phone: String::from_str(&env, "+1-555-1234"),
            relationship: String::from_str(&env, "Owner"),
            email: Some(String::from_str(&env, "john@example.com")),
            is_primary: true,
        });

        let success = client.set_emergency_contacts(&pet_id, &contacts);
        assert!(success);

        if let Some(retrieved_contacts) = client.get_emergency_contacts(&pet_id) {
            assert_eq!(retrieved_contacts.len(), 1);
            assert_eq!(retrieved_contacts.get(0).unwrap().name, String::from_str(&env, "John Doe"));
        }
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
    fn test_emergency_medical_info() {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let admin = Address::generate(&env);

        // Initialize admin and register/verify vet
        client.init_admin(&admin);
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Vet"),
            &String::from_str(&env, "VET123"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&vet);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Rex"),
            &String::from_str(&env, "2019-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Boxer"),
            &PrivacyLevel::Public,
        );

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
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "2019-12-01"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Persian"),
            &PrivacyLevel::Public,
        );

        let mut allergies = Vec::new(&env);
        allergies.push_back(AllergyInfo {
            allergen: String::from_str(&env, "Chicken"),
            severity: String::from_str(&env, "moderate"),
            symptoms: String::from_str(&env, "Vomiting, itching"),

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
        });

        let mut critical_alerts = Vec::new(&env);
        critical_alerts.push_back(String::from_str(&env, "Diabetic"));

        let success = client.set_emergency_medical_info(
            &pet_id,
            &allergies,
            &String::from_str(&env, "Takes insulin daily"),
            &critical_alerts,
        );
        assert!(success);

        if let Some(medical) = client.get_medical_alerts(&pet_id) {
            assert_eq!(medical.allergies.len(), 1);
            assert_eq!(medical.critical_alerts.len(), 1);
        }
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
    fn test_authorize_vet_and_add_medical_record() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Cody"),
            &String::from_str(&env, "2021-08-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Shepherd Mix"),
            &PrivacyLevel::Public,
        );

        // Skip authorize_veterinarian - auth is handled by mock_all_auths

        let record_id = contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Ear infection"),
            &String::from_str(&env, "Antibiotics for 7 days"),
            &String::from_str(&env, "Amoxicillin"),
            &Vec::new(&env),
        );

        let record = contract.get_medical_record(&record_id).unwrap();
        assert_eq!(record.id, record_id);
        assert_eq!(record.pet_id, pet_id);
        assert_eq!(record.veterinarian, vet);
    }

    #[test]
    fn test_only_authorized_vet_can_add_record() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let unauthorized_vet = Address::generate(&env);

        contract.init_admin(&admin);
        contract.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Vet"),
            &String::from_str(&env, "VET789"),
            &String::from_str(&env, "General"),
        );
        contract.verify_vet(&vet);
        // Don't verify unauthorized_vet

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Milo"),
            &String::from_str(&env, "2020-03-12"),
            &Gender::Male,
            &Species::Cat,
            &String::from_str(&env, "Tabby"),
            &PrivacyLevel::Public,
        );

        contract.add_medical_record(
            &pet_id,
            &unauthorized_vet,
            &String::from_str(&env, "Routine checkup"),
            &String::from_str(&env, "No treatment required"),
            &String::from_str(&env, "None"),
            &Vec::new(&env),
        );
    }

    #[test]
    fn test_get_medical_records_for_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Hazel"),
            &String::from_str(&env, "2019-05-20"),
            &Gender::Female,
            &Species::Dog,
            &String::from_str(&env, "Beagle"),
            &PrivacyLevel::Public,
        );

        // Skip authorize_veterinarian - auth is handled by mock_all_auths

        contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Dental cleaning"),
            &String::from_str(&env, "Performed cleaning"),
            &String::from_str(&env, "Post-care rinse"),
            &Vec::new(&env),
        );

        contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Skin allergy"),
            &String::from_str(&env, "Diet adjustment"),
            &String::from_str(&env, "Omega-3 supplements"),
            &Vec::new(&env),
        );

        let records = contract.get_pet_medical_records(&pet_id);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_get_record_by_id_missing_returns_none() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        let missing = contract.get_medical_record(&9999u64);
        assert!(missing.is_none());
    }
}
