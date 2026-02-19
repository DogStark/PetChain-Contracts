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
}