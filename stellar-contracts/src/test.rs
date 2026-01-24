#[cfg(test)]
mod test {
    use crate::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Env, Bytes, BytesN};

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
        assert_eq!(record.batch_number, Some(String::from_str(&env, "BATCH-001")));
        assert_eq!(record.vaccine_name, Some(String::from_str(&env, "Rabies Vaccine")));
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
        
        assert_eq!(client.is_vaccination_current(&pet_id, &VaccineType::Rabies), false);
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
            relationship: String::from_str(&env, "Owner")
        });
        
        client.set_emergency_contacts(&pet_id, &contacts, &String::from_str(&env, "Allergic to bees"));
        
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
}
