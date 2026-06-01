mod test_vaccination_expiry {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    const DAY: u64 = 86_400;

    fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let vet = Address::generate(&env);
        let owner = Address::generate(&env);

        client.init_admin(&admin);
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Vet"),
            &String::from_str(&env, "LIC-001"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&admin, &vet);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Retriever"),
            &PrivacyLevel::Public,
        );

        (env, client, vet, owner, pet_id)
    }

    #[test]
    fn test_get_expiring_vaccinations_within_window() {
        let (env, client, vet, _owner, pet_id) = setup();
        let now = env.ledger().timestamp();

        // Expires in 10 days — within a 30-day window
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "RabiesVax"),
            &now,
            &(now + 10 * DAY),
            &(now + 10 * DAY),
            &String::from_str(&env, "BATCH-001"),
        );

        let expiring = client.get_expiring_vaccinations(&pet_id, &30u64);
        assert_eq!(expiring.len(), 1);
        assert!(!expiring.get(0).unwrap().already_expired);
        assert_eq!(expiring.get(0).unwrap().days_remaining, 10);
    }

    #[test]
    fn test_get_expiring_vaccinations_already_expired() {
        let (env, client, vet, _owner, pet_id) = setup();
        let now = env.ledger().timestamp();

        // Expired 5 days ago
        let past = now.saturating_sub(5 * DAY);
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Distemper,
            &String::from_str(&env, "DistemperVax"),
            &past,
            &past,
            &past,
            &String::from_str(&env, "BATCH-002"),
        );

        let expiring = client.get_expiring_vaccinations(&pet_id, &30u64);
        assert_eq!(expiring.len(), 1);
        assert!(expiring.get(0).unwrap().already_expired);
        assert_eq!(expiring.get(0).unwrap().days_remaining, 0);
    }

    #[test]
    fn test_get_expiring_vaccinations_outside_window_returns_empty() {
        let (env, client, vet, _owner, pet_id) = setup();
        let now = env.ledger().timestamp();

        // Expires in 60 days — outside a 30-day window
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "ParvoVax"),
            &now,
            &(now + 60 * DAY),
            &(now + 60 * DAY),
            &String::from_str(&env, "BATCH-003"),
        );

        let expiring = client.get_expiring_vaccinations(&pet_id, &30u64);
        assert_eq!(expiring.len(), 0);
    }

    #[test]
    fn test_get_expiring_vaccinations_no_vaccinations_returns_empty() {
        let (_env, client, _vet, _owner, pet_id) = setup();
        let expiring = client.get_expiring_vaccinations(&pet_id, &30u64);
        assert_eq!(expiring.len(), 0);
    }

    #[test]
    fn test_vaccination_summary_overdue() {
        let (env, client, vet, _owner, pet_id) = setup();
        let now = env.ledger().timestamp();
        let past = now.saturating_sub(5 * DAY);

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "RabiesVax"),
            &past,
            &past,
            &past,
            &String::from_str(&env, "BATCH-004"),
        );

        let summary = client.get_vaccination_summary(&pet_id);
        assert!(!summary.is_fully_current);
        assert_eq!(summary.overdue_types.len(), 1);
    }

    #[test]
    fn test_vaccination_summary_current() {
        let (env, client, vet, _owner, pet_id) = setup();
        let now = env.ledger().timestamp();

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "RabiesVax"),
            &now,
            &(now + 365 * DAY),
            &(now + 365 * DAY),
            &String::from_str(&env, "BATCH-005"),
        );

        let summary = client.get_vaccination_summary(&pet_id);
        assert!(summary.is_fully_current);
        assert_eq!(summary.overdue_types.len(), 0);
    }
}
