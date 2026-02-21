#[cfg(test)]
mod test {
    use crate::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env, String, Address, Vec,
    };

    fn setup_test() -> (Env, Address, PetChainContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.init_admin(&admin);

        (env, admin, client)
    }

    #[test]
    fn test_register_pet() {
        let (env, _admin, client) = setup_test();
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
            &String::from_str(&env, "Brown"),
            &5,
            &None,
            &PrivacyLevel::Public,
        );
        assert_eq!(pet_id, 1);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.id, 1);
        assert_eq!(pet.name, name);
    }

    #[test]
    fn test_register_pet_owner() {
        let (env, _admin, client) = setup_test();
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
        let (env, admin, client) = setup_test();
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
            &10,
            &None,
            &PrivacyLevel::Public,
        );

        client.register_vet(
            &admin,
            &vet,
            &String::from_str(&env, "Dr. Who"),
            &String::from_str(&env, "LIC-001"),
            &String::from_str(&env, "Clinic Name"),
            &String::from_str(&env, "Clinic Address"),
            &Vec::new(&env),
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
    }

    #[test]
    fn test_link_tag_to_pet() {
        let (env, _admin, client) = setup_test();
        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Golden Retriever"),
            &String::from_str(&env, "Golden"),
            &10,
            &None,
            &PrivacyLevel::Public,
        );

        let tag_id = client.link_tag_to_pet(&pet_id);
        let tag = client.get_tag(&tag_id).unwrap();
        assert_eq!(tag.pet_id, pet_id);
    }

    #[test]
    fn test_pet_privacy_flow() {
        let (env, _admin, client) = setup_test();
        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Secret Pet"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Cat,
            &String::from_str(&env, "X"),
            &String::from_str(&env, "Black"),
            &0,
            &None,
            &PrivacyLevel::Private,
        );

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.name, String::from_str(&env, "Secret Pet"));

        let user = Address::generate(&env);
        client.grant_access(&pet_id, &user, &AccessLevel::Full, &None);
        assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);

        client.revoke_access(&pet_id, &user);
        assert_eq!(client.check_access(&pet_id, &user), AccessLevel::None);
    }

    #[test]
    fn test_lab_results() {
        let (env, _admin, client) = setup_test();
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Patient"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
            &String::from_str(&env, "White"),
            &5,
            &None,
            &PrivacyLevel::Public,
        );

        let test_type = String::from_str(&env, "Blood Test");
        let results = String::from_str(&env, "Glucose: 100 mg/dL");

        let lab_id = client.add_lab_result(
            &pet_id,
            &vet,
            &test_type,
            &results,
            &100,
        );

        let res = client.get_lab_result(&lab_id).unwrap();
        assert_eq!(res.test_name, test_type);
        assert_eq!(res.result, results);
    }

    #[test]
    fn test_update_medical_record() {
        let (env, _admin, client) = setup_test();
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Pet"),
            &String::from_str(&env, "2020"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Breed"),
            &String::from_str(&env, "Black"),
            &20,
            &None,
            &PrivacyLevel::Public,
        );

        let start_time = 1000;
        env.ledger().with_mut(|l| l.timestamp = start_time);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &start_time,
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &String::from_str(&env, "Monitor"),
        );

        let created_record = client.get_medical_record(&record_id).unwrap();
        assert_eq!(created_record.updated_at, start_time);

        let update_time = 2000;
        env.ledger().with_mut(|l| l.timestamp = update_time);

        let mut new_meds = Vec::new(&env);
        new_meds.push_back(Medication {
            id: 1,
            pet_id,
            name: String::from_str(&env, "Med1"),
            dosage: String::from_str(&env, "20mg"),
            frequency: String::from_str(&env, "Daily"),
            start_date: 100,
            end_date: Some(200),
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
        assert_eq!(updated.diagnosis, String::from_str(&env, "Sick"));
        assert_eq!(updated.updated_at, update_time);
    }

    #[test]
    fn test_report_lost_found() {
        let (env, _admin, client) = setup_test();
        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Golden"),
            &10,
            &None,
            &PrivacyLevel::Public,
        );

        let location = String::from_str(&env, "Central Park, NYC");
        let alert_id = client.report_lost(&pet_id, &location, &Some(500));
        assert_eq!(alert_id, 1);

        let alert = client.get_alert(&alert_id).unwrap();
        assert_eq!(alert.status, AlertStatus::Active);

        let result = client.report_found(&alert_id);
        assert!(result);

        let found_alert = client.get_alert(&alert_id).unwrap();
        assert_eq!(found_alert.status, AlertStatus::Found);
    }

    #[test]
    fn test_vet_reviews() {
        let (env, admin, client) = setup_test();
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        client.register_vet(
            &admin,
            &vet,
            &String::from_str(&env, "Dr. Smith"),
            &String::from_str(&env, "VET-001"),
            &String::from_str(&env, "General"),
            &String::from_str(&env, "Clinic Address"),
            &Vec::new(&env),
        );

        client.add_vet_review(&owner, &vet, &5, &String::from_str(&env, "Excellent vet!"));
        
        let vet_data = client.get_vet(&vet).unwrap();
        assert_eq!(vet_data.rating, 5);
        assert_eq!(vet_data.review_count, 1);
    }

    #[test]
    fn test_multisig_workflow() {
        let (env, admin, client) = setup_test();
        let admin1 = admin;
        let admin2 = Address::generate(&env);
        let admin3 = Address::generate(&env);
        let vet = Address::generate(&env);

        let mut admins = Vec::new(&env);
        admins.push_back(admin1.clone());
        admins.push_back(admin2.clone());
        admins.push_back(admin3.clone());
        
        client.init_multisig(&admin1, &admins, &2);

        let action = ProposalAction::VerifyVet(vet.clone());
        let proposal_id = client.propose_action(&admin1, &action, &3600);
        
        client.register_vet(&admin1, &vet, &String::from_str(&env, "Dr. Multi"), &String::from_str(&env, "LIC-999"), &String::from_str(&env, "Clinic"), &String::from_str(&env, "Address"), &Vec::new(&env));
        
        client.approve_proposal(&admin2, &proposal_id);
        client.execute_proposal(&proposal_id);
        
        assert!(client.is_verified_vet(&vet));
    }

    #[test]
    fn test_pet_transfer_flow() {
        let (env, _admin, client) = setup_test();
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner1,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Retriever"),
            &String::from_str(&env, "Golden"),
            &10,
            &None,
            &PrivacyLevel::Public,
        );

        client.initiate_transfer(&pet_id, &owner2, &(env.ledger().timestamp() + 3600));
        client.accept_transfer(&pet_id, &owner2);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.owner, owner2);
    }
}
