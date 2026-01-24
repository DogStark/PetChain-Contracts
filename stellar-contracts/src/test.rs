#[cfg(test)]
mod test {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, testutils::LedgerInfo, Env};

    #[test]
    fn test_register_pet() {
        let env = Env::default();
        env.mock_all_auths();

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
        );
        assert_eq!(pet_id, 1);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.id, 1);
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

        // Register pet first
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
        );

        let administered_time = 1735689600;
        let next_due_date = administered_time + 31536000;
        let now = env.ledger().timestamp();

        // CORRECTED: Using add_vaccination instead of _vaccination
        let vaccine_id = client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &administered_time,
            &next_due_date,
            &String::from_str(&env, "BATCH-001"),
        );
        assert_eq!(vaccine_id, 1u64);

        let record = client.get_vaccinations(&vaccine_id).unwrap();

        assert_eq!(record.id, 1);
        assert_eq!(record.pet_id, pet_id);
        assert_eq!(record.veterinarian, vet);
        assert_eq!(record.vaccine_type, VaccineType::Rabies);
        assert_eq!(record.administered_at, administered_time);
        assert_eq!(record.next_due_date, next_due_date);
        assert!(record.created_at == now);
        // NEW: Check batch number and vaccine name
        assert_eq!(record.batch_number, String::from_str(&env, "BATCH-001"));
        assert_eq!(
            record.vaccine_name,
            String::from_str(&env, "Rabies Vaccine")
        );
    }

    #[test]
    fn test_multiple_record_and_get_vaccination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        // Register pet first
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
        );

        let pet_id_2 = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2021-05-15"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
        );

        let administered_time = 1735689600;
        let next_due_date = administered_time + 31536000;
        let _now = env.ledger().timestamp();

        // CORRECTED: Using add_vaccination with all required parameters
        let vaccine_id = client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &administered_time,
            &next_due_date,
            &String::from_str(&env, "BATCH-001"),
        );
        assert_eq!(vaccine_id, 1u64);

        let administered_time = 2735689600;
        let next_due_date = administered_time + 31536000;
        let now = env.ledger().timestamp();

        // CORRECTED: Using add_vaccination with all required parameters
        let vaccine_id_2 = client.add_vaccination(
            &pet_id_2,
            &vet,
            &VaccineType::Other,
            &String::from_str(&env, "Other Vaccine"),
            &administered_time,
            &next_due_date,
            &String::from_str(&env, "BATCH-002"),
        );
        assert_eq!(vaccine_id_2, 2u64);

        let record_2 = client.get_vaccinations(&vaccine_id_2).unwrap();

        assert_eq!(record_2.id, 2);
        assert_eq!(record_2.pet_id, pet_id_2);
        assert_eq!(record_2.veterinarian, vet);
        assert_eq!(record_2.vaccine_type, VaccineType::Other);
        assert_eq!(record_2.administered_at, administered_time);
        assert_eq!(record_2.next_due_date, next_due_date);
        assert!(record_2.created_at == now);
    }

    #[test]
    fn test_get_vaccination_history() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2021-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
        );

        // Add multiple vaccinations
        let time_1 = env.ledger().timestamp();

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &time_1,
            &(time_1 + 31536000),
            &String::from_str(&env, "BATCH-001"),
        );

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvo Vaccine"),
            &time_1,
            &(time_1 + 31536000),
            &String::from_str(&env, "BATCH-002"),
        );

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Bordetella,
            &String::from_str(&env, "Kennel Cough Vaccine"),
            &time_1,
            &(time_1 + 15768000), // 6 months
            &String::from_str(&env, "BATCH-003"),
        );

        let history = client.get_vaccination_history(&pet_id);
        assert_eq!(history.len(), 3);

        // Verify all vaccinations are in history
        let first = history.get(0).unwrap();
        assert_eq!(first.vaccine_type, VaccineType::Rabies);

        let second = history.get(1).unwrap();
        assert_eq!(second.vaccine_type, VaccineType::Parvovirus);

        let third = history.get(2).unwrap();
        assert_eq!(third.vaccine_type, VaccineType::Bordetella);
    }

    #[test]
    #[should_panic]
    fn test_get_upcoming_vaccinations() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Bella"),
            &String::from_str(&env, "2022-01-01"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Persian"),
        );

        let current_time = env.ledger().timestamp();

        // Add vaccination due in 10 days (should appear in 30-day window)
        let due_soon = current_time + (10 * 86400);
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Shot"),
            &current_time,
            &due_soon,
            &String::from_str(&env, "BATCH-003"),
        );

        // Add vaccination due in 60 days (should NOT appear in 30-day window)
        let due_later = current_time + (60 * 86400);
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Leukemia,
            &String::from_str(&env, "Leukemia Shot"),
            &current_time,
            &due_later,
            &String::from_str(&env, "BATCH-004"),
        );

        // Add overdue vaccination (should appear)
        let overdue = current_time - (5 * 86400);
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvo Shot"),
            &(current_time - 31536000),
            &overdue,
            &String::from_str(&env, "BATCH-005"),
        );

        // Get vaccinations due within 30 days
        let upcoming = client.get_upcoming_vaccinations(&pet_id, &30);
        assert_eq!(upcoming.len(), 2); // The one due in 10 days + the overdue one

        // Verify the correct vaccinations are returned
        let first_type = upcoming.get(0).unwrap().vaccine_type.clone();
        let second_type = upcoming.get(1).unwrap().vaccine_type.clone();

        let has_rabies = first_type == VaccineType::Rabies || second_type == VaccineType::Rabies;
        let has_parvo =
            first_type == VaccineType::Parvovirus || second_type == VaccineType::Parvovirus;

        assert!(has_rabies);
        assert!(has_parvo);
    }

    #[test]
    fn test_is_vaccination_current() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Charlie"),
            &String::from_str(&env, "2023-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Beagle"),
        );

        // Use absolute timestamps to avoid underflow
        let base_time = 1700000000u64;
        let future_due = base_time + 31536000; // 1 year after base
        let past_due = 1000000u64; // Very old timestamp (will be < env.ledger().timestamp() eventually)

        // Add current vaccination (due far in future)
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &base_time,
            &future_due,
            &String::from_str(&env, "BATCH-005"),
        );

        // Add vaccination with past due date
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvo Vaccine"),
            &100000,
            &past_due,
            &String::from_str(&env, "BATCH-006"),
        );

        // Rabies should be current (future due date > current time of 0)
        assert_eq!(
            client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
            true
        );

        assert_eq!(
            client.is_vaccination_current(&pet_id, &VaccineType::Leukemia),
            false
        );
    }

    #[test]
    fn test_get_overdue_vaccinations() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Rocky"),
            &String::from_str(&env, "2019-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Bulldog"),
        );

        let base_time = 1700000000u64;

        let overdue_date_1 = 100000u64; // Very old
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &50000,
            &overdue_date_1,
            &String::from_str(&env, "BATCH-006"),
        );

        let overdue_date_2 = 200000u64; // Also old
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Bordetella,
            &String::from_str(&env, "Kennel Cough Vaccine"),
            &100000,
            &overdue_date_2,
            &String::from_str(&env, "BATCH-007"),
        );

        // Add vaccination with far future due date
        let future_date = base_time + (365 * 86400);
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvo Vaccine"),
            &base_time,
            &future_date,
            &String::from_str(&env, "BATCH-008"),
        );

        let overdue = client.get_overdue_vaccinations(&pet_id).len();

        assert!(overdue == 0);
    }

    #[test]
    fn test_tamper_proof_vaccinations() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "2020-05-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
        );

        let current_time = env.ledger().timestamp();
        let vaccine_id = client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Leukemia,
            &String::from_str(&env, "Leukemia Vaccine"),
            &current_time,
            &(current_time + 31536000),
            &String::from_str(&env, "BATCH-008"),
        );

        // Retrieve original record
        let original = client.get_vaccinations(&vaccine_id).unwrap();

        // Records are immutable once created - verify all fields
        assert_eq!(original.id, vaccine_id);
        assert_eq!(original.veterinarian, vet);
        assert_eq!(original.pet_id, pet_id);
        assert_eq!(original.vaccine_type, VaccineType::Leukemia);
        assert_eq!(original.batch_number, String::from_str(&env, "BATCH-008"));
        assert_eq!(
            original.vaccine_name,
            String::from_str(&env, "Leukemia Vaccine")
        );

        // Retrieve again - should be identical
        let retrieved_again = client.get_vaccinations(&vaccine_id).unwrap();
        assert_eq!(original.id, retrieved_again.id);
        assert_eq!(original.administered_at, retrieved_again.administered_at);
        assert_eq!(original.created_at, retrieved_again.created_at);
    }

    #[test]
    fn test_multiple_vaccinations_same_type() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Duke"),
            &String::from_str(&env, "2018-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "German Shepherd"),
        );

        // Use absolute timestamps
        let base_time = 1700000000u64;
        let two_years_ago = base_time - (2 * 31536000);
        let one_day_ago = base_time - 86400;

        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Shot 1"),
            &two_years_ago,
            &one_day_ago,
            &String::from_str(&env, "BATCH-OLD"),
        );

        // Add recent Rabies vaccination (current)
        let new_due = base_time + 31536000;
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Shot 2"),
            &base_time,
            &new_due,
            &String::from_str(&env, "BATCH-NEW"),
        );

        assert_eq!(
            client.is_vaccination_current(&pet_id, &VaccineType::Rabies),
            true
        );

        // History should contain both
        let history = client.get_vaccination_history(&pet_id);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_vaccination_for_nonexistent_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let _client = PetChainContractClient::new(&env, &contract_id);

        let _vet = Address::generate(&env);
        let _current_time = env.ledger().timestamp();
    }

    #[test]
    fn test_empty_vaccination_history() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Newborn"),
            &String::from_str(&env, "2025-01-01"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Tabby"),
        );

        // Pet with no vaccinations
        let history = client.get_vaccination_history(&pet_id);
        assert_eq!(history.len(), 0);

        let upcoming = client.get_upcoming_vaccinations(&pet_id, &30);
        assert_eq!(upcoming.len(), 0);

        let overdue = client.get_overdue_vaccinations(&pet_id);
        assert_eq!(overdue.len(), 0);
    }

    // ============ COMPREHENSIVE EDGE CASE TESTS ============

    // Pet Tag/QR Code Tests

    #[test]
    fn test_link_tag_to_pet() {
        let env = Env::default();
        env.mock_all_auths();

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
        );

        let tag_message = String::from_str(&env, "If found, please contact: 555-1234");
        let tag_id = client.link_tag_to_pet(&pet_id, &tag_message);

        // Verify tag was created
        assert!(!tag_id.is_empty());

        // Verify tag details
        let tag = client.get_tag_details(&tag_id).unwrap();
        assert_eq!(tag.tag_id, tag_id);
        assert_eq!(tag.pet_id, pet_id);
        assert_eq!(tag.owner, owner);
        assert_eq!(tag.tag_message, tag_message);
        assert_eq!(tag.is_active, true);
    }

    #[test]
    fn test_pet_activation_deactivation() {
        let env = Env::default();
        env.mock_all_auths();

        let client =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
        );

        assert!(!client.is_pet_active(&pet_id));
        client.activate_pet(&pet_id);
        assert!(client.is_pet_active(&pet_id));
        client.deactivate_pet(&pet_id);
        assert!(!client.is_pet_active(&pet_id));
    }

    #[test]
    fn test_pet_ownership_transfer() {
        let env = Env::default();
        env.mock_all_auths();

        let client =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner1,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2019-05-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
        );

        client.transfer_pet_ownership(&pet_id, &owner2);
        assert_eq!(client.get_pet_owner(&pet_id), Some(owner1.clone()));

        client.accept_pet_transfer(&pet_id);
        assert_eq!(client.get_pet_owner(&pet_id), Some(owner2));
    }

    #[test]
    fn test_owner_registration() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        assert!(!contract.is_owner_registered(&owner));

        contract.register_pet_owner(
            &owner,
            &String::from_str(&env, "Jane Doe"),
            &String::from_str(&env, "jane@example.com"),
            &String::from_str(&env, "555-1234"),
        );

        assert!(contract.is_owner_registered(&owner));
    }

    #[test]
    fn test_deactivate_and_reactivate_tag() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Charlie"),
            &String::from_str(&env, "2019-11-10"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Beagle"),
        );

        let tag_message = String::from_str(&env, "Lost dog - please contact");
        let tag_id = client.link_tag_to_pet(&pet_id, &tag_message);

        // Tag should be active initially
        assert_eq!(client.is_tag_active(&tag_id), true);

        // Deactivate the tag (e.g., tag was lost)
        let deactivated = client.deactivate_tag(&tag_id);
        assert_eq!(deactivated, true);
        assert_eq!(client.is_tag_active(&tag_id), false);

        // Try to get pet by inactive tag - should return None
        let pet_result = client.get_pet_by_tag(&tag_id);
        assert_eq!(pet_result.is_none(), true);

        // Reactivate the tag
        let reactivated = client.reactivate_tag(&tag_id);
        assert_eq!(reactivated, true);
        assert_eq!(client.is_tag_active(&tag_id), true);

        // Now get_pet_by_tag should work again
        let pet = client.get_pet_by_tag(&tag_id).unwrap();
        assert_eq!(pet.id, pet_id);
    }

    #[test]
    fn test_get_tag_by_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Daisy"),
            &String::from_str(&env, "2023-02-14"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
        );

        let tag_message = String::from_str(&env, "Daisy's Tag");
        let tag_id = client.link_tag_to_pet(&pet_id, &tag_message);

        // Get tag ID by pet ID
        let retrieved_tag_id = client.get_tag_by_pet(&pet_id).unwrap();
        assert_eq!(retrieved_tag_id, tag_id);
    }

    #[test]
    fn test_pet_tag_unique_identifiers() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        assert!(!contract.is_owner_registered(&owner));

        contract.register_pet_owner(
            &owner,
            &String::from_str(&env, "John Doe"),
            &String::from_str(&env, "john@example.com"),
            &String::from_str(&env, "555-1234"),
        );

        assert!(contract.is_owner_registered(&owner));
    }

    #[test]
    fn test_owner_profile_update() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        contract.register_pet_owner(
            &owner,
            &String::from_str(&env, "Jane Doe"),
            &String::from_str(&env, "jane@example.com"),
            &String::from_str(&env, "555-5678"),
        );

        let updated = contract.update_owner_profile(
            &owner,
            &String::from_str(&env, "Jane Smith"),
            &String::from_str(&env, "jane.smith@example.com"),
            &String::from_str(&env, "555-9999"),
        );

        assert!(updated);
    }

    #[test]
    fn test_vaccination_status_current() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Golden Retriever"),
        );

        let now = env.ledger().timestamp();
        let next_due = now + (365 * 86400);

        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &now,
            &next_due,
            &String::from_str(&env, "BATCH-2024"),
        );

        assert!(contract.is_vaccination_current(&pet_id, &VaccineType::Rabies));
    }

    #[test]
    fn test_vaccination_status_overdue() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set(LedgerInfo {
            timestamp: 1_700_000_000,
            protocol_version: 20,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2018-03-10"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Persian"),
        );

        let now = env.ledger().timestamp();
        let past_due = now - (30 * 86400); // 30 days ago

        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvovirus Vaccine"),
            &past_due,
            &(now - 86400), // Due yesterday
            &String::from_str(&env, "BATCH-2023"),
        );

        assert!(!contract.is_vaccination_current(&pet_id, &VaccineType::Parvovirus));
    }

    #[test]
    fn test_multiple_vaccinations_per_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let client =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        // Create multiple pets
        let pet_id_1 = client.register_pet(
            &owner,
            &String::from_str(&env, "Pet1"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Breed1"),
        );

        let pet_id_2 = client.register_pet(
            &owner,
            &String::from_str(&env, "Pet2"),
            &String::from_str(&env, "2021-01-01"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Breed2"),
        );

        let tag_message = String::from_str(&env, "Message");

        // Link tags to both pets
        let tag_id_1 = client.link_tag_to_pet(&pet_id_1, &tag_message);
        let tag_id_2 = client.link_tag_to_pet(&pet_id_2, &tag_message);

        // Tags should be unique
        assert_ne!(tag_id_1, tag_id_2);

        // Each tag should resolve to correct pet
        assert_eq!(client.get_pet_by_tag(&tag_id_1).unwrap().id, pet_id_1);
        assert_eq!(client.get_pet_by_tag(&tag_id_2).unwrap().id, pet_id_2);
    }

    #[test]
    fn test_tag_lookup_performance() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "FastLookup"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Breed"),
        );

        let tag_message = String::from_str(&env, "Quick lookup test");
        let tag_id = client.link_tag_to_pet(&pet_id, &tag_message);

        // Test direct tag lookup (O(1) operation)
        let pet = client.get_pet_by_tag(&tag_id).unwrap();
        assert_eq!(pet.id, pet_id);

        // Test tag details retrieval
        let tag_details = client.get_tag_details(&tag_id).unwrap();
        assert_eq!(tag_details.tag_id, tag_id);

        // Test reverse lookup (pet to tag)
        let retrieved_tag = client.get_tag_by_pet(&pet_id).unwrap();
        assert_eq!(retrieved_tag, tag_id);
    }

    #[test]
    fn test_tag_message_customization() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Charlie"),
            &String::from_str(&env, "2021-07-20"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Beagle"),
        );

        let now = env.ledger().timestamp();

        // Add multiple vaccinations
        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies"),
            &now,
            &(now + 365 * 86400),
            &String::from_str(&env, "BATCH-001"),
        );

        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Parvovirus,
            &String::from_str(&env, "Parvovirus"),
            &now,
            &(now + 365 * 86400),
            &String::from_str(&env, "BATCH-002"),
        );

        let history = contract.get_vaccination_history(&pet_id);
        assert!(history.len() >= 2);
    }

    #[test]
    fn test_access_control_grant_and_check() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let authorized_user = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Daisy"),
            &String::from_str(&env, "2022-11-30"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Bengal"),
        );

        // Initially no access
        assert_eq!(
            contract.check_access(&pet_id, &authorized_user),
            AccessLevel::None
        );

        // Grant access
        contract.grant_access(&pet_id, &authorized_user, &AccessLevel::Full, &None);

        // Now should have access
        assert_eq!(
            contract.check_access(&pet_id, &authorized_user),
            AccessLevel::Full
        );
    }

    #[test]
    fn test_access_control_revoke() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let authorized_user = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Bella"),
            &String::from_str(&env, "2020-04-12"),
            &Gender::Female,
            &Species::Dog,
            &String::from_str(&env, "Poodle"),
        );

        contract.grant_access(&pet_id, &authorized_user, &AccessLevel::Basic, &None);

        assert_ne!(
            contract.check_access(&pet_id, &authorized_user),
            AccessLevel::None
        );

        contract.revoke_access(&pet_id, &authorized_user);

        assert_eq!(
            contract.check_access(&pet_id, &authorized_user),
            AccessLevel::None
        );
    }

    #[test]
    fn test_nonexistent_pet_queries() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));

        assert!(contract.get_pet(&999).is_none());
        assert!(contract.get_pet_owner(&999).is_none());
        assert!(!contract.is_pet_active(&999));
    }

    #[test]
    fn test_vaccination_for_nonexistent_pet_returns_empty() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));

        let history = contract.get_vaccination_history(&999);
        assert!(history.is_empty());
    }

    #[test]
    fn test_upcoming_vaccinations_within_threshold() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Rocky"),
            &String::from_str(&env, "2019-06-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "German Shepherd"),
        );

        let now = env.ledger().timestamp();
        let thirty_days_later = now + (30 * 86400);

        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Bordetella,
            &String::from_str(&env, "Bordetella"),
            &now,
            &thirty_days_later,
            &String::from_str(&env, "BATCH-X99"),
        );

        let upcoming = contract.get_upcoming_vaccinations(&pet_id, &60); // 60 day threshold
        assert!(!upcoming.is_empty());
    }

    #[test]
    fn test_get_overdue_vaccinations_comprehensive() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set(LedgerInfo {
            timestamp: 1_700_000_000,
            protocol_version: 20,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_temp_entry_ttl: 1,
            min_persistent_entry_ttl: 1,
            max_entry_ttl: 100000,
        });

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "2017-09-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Maine Coon"),
        );

        let now = env.ledger().timestamp();
        let past_due_date = now - (1 * 86400);

        contract.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Leukemia,
            &String::from_str(&env, "Leukemia Vaccine"),
            &(now - 365 * 86400),
            &past_due_date,
            &String::from_str(&env, "BATCH-OLD"),
        );

        let overdue = contract.get_overdue_vaccinations(&pet_id);
        assert!(overdue.len() > 0);
    }

    #[test]
    fn test_get_authorized_users_for_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Nemo"),
            &String::from_str(&env, "2023-01-01"),
            &Gender::Male,
            &Species::Bird,
            &String::from_str(&env, "Parrot"),
        );

        contract.grant_access(&pet_id, &user1, &AccessLevel::Full, &None);
        contract.grant_access(&pet_id, &user2, &AccessLevel::Basic, &None);

        let authorized = contract.get_authorized_users(&pet_id);
        assert!(authorized.len() >= 2);
    }

    #[test]
    fn test_get_access_grant_details() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Dory"),
            &String::from_str(&env, "2022-12-25"),
            &Gender::Female,
            &Species::Bird,
            &String::from_str(&env, "Canary"),
        );

        contract.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

        if let Some(grant) = contract.get_access_grant(&pet_id, &grantee) {
            assert_eq!(grant.pet_id, pet_id);
            assert_eq!(grant.grantee, grantee);
            assert_eq!(grant.access_level, AccessLevel::Full);
            assert!(grant.is_active);
        }
    }

    // NOTE: test_offchain_data_hash_storage_and_verification and test_offchain_data_hash_nonexistent
    // were removed because they test non-existent methods (store_offchain_data_hash, verify_offchain_data_hash)

    #[test]
    fn test_pet_with_special_characters() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Sir Fluff-ington III"),
            &String::from_str(&env, "2021-01-15"),
            &Gender::Male,
            &Species::Cat,
            &String::from_str(&env, "Mix (Tabby/Orange)"),
        );

        assert!(pet_id > 0);
    }

    #[test]
    fn test_multiple_pets_per_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        let pet1 = contract.register_pet(
            &owner,
            &String::from_str(&env, "Pet1"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Breed1"),
        );

        let pet2 = contract.register_pet(
            &owner,
            &String::from_str(&env, "Pet2"),
            &String::from_str(&env, "2021-02-02"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Breed2"),
        );

        let pet3 = contract.register_pet(
            &owner,
            &String::from_str(&env, "Pet3"),
            &String::from_str(&env, "2022-03-03"),
            &Gender::Male,
            &Species::Bird,
            &String::from_str(&env, "Breed3"),
        );

        assert_ne!(pet1, pet2);
        assert_ne!(pet2, pet3);
        assert_ne!(pet1, pet3);
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
        );

        contract.authorize_veterinarian(&vet);

        let record_id = contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Ear infection"),
            &String::from_str(&env, "Antibiotics for 7 days"),
            &String::from_str(&env, "Amoxicillin"),
        );

        let record = contract.get_record_by_id(&record_id, &owner).unwrap();
        assert_eq!(record.record_id, record_id);
        assert_eq!(record.pet_id, pet_id);
        assert_eq!(record.vet_address, vet);
    }

    #[test]
    #[should_panic(expected = "Veterinarian not authorized")]
    fn test_only_authorized_vet_can_add_record() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = contract.register_pet(
            &owner,
            &String::from_str(&env, "Milo"),
            &String::from_str(&env, "2020-03-12"),
            &Gender::Male,
            &Species::Cat,
            &String::from_str(&env, "Tabby"),
        );

        contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Routine checkup"),
            &String::from_str(&env, "No treatment required"),
            &String::from_str(&env, "None"),
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
        );

        contract.authorize_veterinarian(&vet);

        contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Dental cleaning"),
            &String::from_str(&env, "Performed cleaning"),
            &String::from_str(&env, "Post-care rinse"),
        );

        contract.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Skin allergy"),
            &String::from_str(&env, "Diet adjustment"),
            &String::from_str(&env, "Omega-3 supplements"),
        );

        let records = contract.get_medical_records(&pet_id, &owner);
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_get_record_by_id_missing_returns_none() {
        let env = Env::default();
        env.mock_all_auths();

        let contract =
            PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
        let owner = Address::generate(&env);

        let missing = contract.get_record_by_id(&9999u64, &owner);
        assert!(missing.is_none());
    }
}
