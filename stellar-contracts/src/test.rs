#[cfg(test)]
mod test {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_pet() {
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
        assert!(pet_id > 0);
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
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "German Shepherd"),
            &PrivacyLevel::Public,
        );

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
    }

    #[test]
    fn test_emergency_medical_info() {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
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
    }
}
