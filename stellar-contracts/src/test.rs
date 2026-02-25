// ============================================================
// VET REGISTRATION & VERIFICATION TESTS
// ============================================================

#[cfg(test)]
mod test_vet {
    use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn register_test_pet(
        client: &PetChainContractClient,
        env: &Env,
        owner: &Address,
    ) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "Buddy"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Labrador"),
            &String::from_str(env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        )
    }

    #[test]
    fn test_register_vet_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        let result = client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Sarah Connor"),
            &String::from_str(&env, "LIC-2024-001"),
            &String::from_str(&env, "Surgery"),
        );

        assert!(result);

        let stored = client.get_vet(&vet).unwrap();
        assert_eq!(stored.address, vet);
        assert_eq!(stored.name, String::from_str(&env, "Dr. Sarah Connor"));
        assert_eq!(stored.license_number, String::from_str(&env, "LIC-2024-001"));
        assert_eq!(stored.specialization, String::from_str(&env, "Surgery"));
        assert_eq!(stored.verified, false);
    }

    #[test]
    fn test_register_vet_is_unverified_by_default() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. John Doe"),
            &String::from_str(&env, "LIC-2024-002"),
            &String::from_str(&env, "General Practice"),
        );

        assert!(!client.is_verified_vet(&vet));
    }

    #[test]
    fn test_verify_vet_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Jane Smith"),
            &String::from_str(&env, "LIC-2024-003"),
            &String::from_str(&env, "Cardiology"),
        );

        assert!(!client.is_verified_vet(&vet));

        client.verify_vet(&admin, &vet);

        assert!(client.is_verified_vet(&vet));

        let stored = client.get_vet(&vet).unwrap();
        assert_eq!(stored.verified, true);
    }

    #[test]
    fn test_revoke_vet_license_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Mark Evans"),
            &String::from_str(&env, "LIC-2024-004"),
            &String::from_str(&env, "Orthopedics"),
        );

        client.verify_vet(&admin, &vet);
        assert!(client.is_verified_vet(&vet));

        let result = client.revoke_vet_license(&admin, &vet);
        assert!(result);
        assert!(!client.is_verified_vet(&vet));

        let stored = client.get_vet(&vet).unwrap();
        assert_eq!(stored.verified, false);
    }

    #[test]
    fn test_revoke_then_reverify_vet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Alice Brown"),
            &String::from_str(&env, "LIC-2024-005"),
            &String::from_str(&env, "Neurology"),
        );

        client.verify_vet(&admin, &vet);
        assert!(client.is_verified_vet(&vet));

        client.revoke_vet_license(&admin, &vet);
        assert!(!client.is_verified_vet(&vet));

        // Can be re-verified after revocation
        client.verify_vet(&admin, &vet);
        assert!(client.is_verified_vet(&vet));
    }

    #[test]
    #[should_panic(expected = "License already registered")]
    fn test_duplicate_license_prevention() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet1 = Address::generate(&env);
        let vet2 = Address::generate(&env);

        client.register_vet(
            &vet1,
            &String::from_str(&env, "Dr. First"),
            &String::from_str(&env, "LIC-DUPLICATE"),
            &String::from_str(&env, "General Practice"),
        );

        // Different address, same license — should panic
        client.register_vet(
            &vet2,
            &String::from_str(&env, "Dr. Second"),
            &String::from_str(&env, "LIC-DUPLICATE"),
            &String::from_str(&env, "Surgery"),
        );
    }

    #[test]
    #[should_panic(expected = "Vet already registered")]
    fn test_duplicate_address_prevention() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Clone"),
            &String::from_str(&env, "LIC-2024-006"),
            &String::from_str(&env, "Dermatology"),
        );

        // Same address, different license — should panic
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Clone Again"),
            &String::from_str(&env, "LIC-2024-007"),
            &String::from_str(&env, "Dermatology"),
        );
    }

    #[test]
    fn test_get_vet_returns_none_for_unknown_address() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let unknown = Address::generate(&env);

        assert!(client.get_vet(&unknown).is_none());
    }

    #[test]
    fn test_is_verified_vet_returns_false_for_unregistered() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let unknown = Address::generate(&env);

        assert!(!client.is_verified_vet(&unknown));
    }

    #[test]
    fn test_get_vet_by_license_success() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);
        let license = String::from_str(&env, "LIC-2024-008");

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Maria Garcia"),
            &license,
            &String::from_str(&env, "Ophthalmology"),
        );

        let found = client.get_vet_by_license(&license).unwrap();
        assert_eq!(found.address, vet);
        assert_eq!(found.license_number, license);
    }

    #[test]
    fn test_get_vet_by_license_returns_none_for_unknown() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        assert!(client.get_vet_by_license(&String::from_str(&env, "NONEXISTENT")).is_none());
    }

    #[test]
    fn test_multiple_vets_independent_verification() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let vet1 = Address::generate(&env);
        let vet2 = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        client.register_vet(
            &vet1,
            &String::from_str(&env, "Dr. One"),
            &String::from_str(&env, "LIC-2024-009"),
            &String::from_str(&env, "Surgery"),
        );
        client.register_vet(
            &vet2,
            &String::from_str(&env, "Dr. Two"),
            &String::from_str(&env, "LIC-2024-010"),
            &String::from_str(&env, "Therapy"),
        );

        // Verify only vet1
        client.verify_vet(&admin, &vet1);

        assert!(client.is_verified_vet(&vet1));
        assert!(!client.is_verified_vet(&vet2));
    }

    #[test]
    fn test_verified_vet_can_add_vaccination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Rex"),
            &String::from_str(&env, "2021-06-15"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "German Shepherd"),
            &String::from_str(&env, "Black"),
            &30u32,
            &None,
            &PrivacyLevel::Public,
        );

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Verified"),
            &String::from_str(&env, "LIC-VERIFIED-001"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&admin, &vet);

        // Verified vet should be able to add a vaccination
        use crate::VaccineType;
        let vaccine_id = client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "RabiesVax Pro"),
            &env.ledger().timestamp(),
            &(env.ledger().timestamp() + 31536000),
            &String::from_str(&env, "BATCH-001"),
        );

        assert_eq!(vaccine_id, 1);
    }

    #[test]
    #[should_panic(expected = "Veterinarian not verified")]
    fn test_unverified_vet_cannot_add_vaccination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "2022-03-10"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
            &String::from_str(&env, "White"),
            &5u32,
            &None,
            &PrivacyLevel::Public,
        );

        // Registered but NOT verified
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Unverified"),
            &String::from_str(&env, "LIC-UNVERIFIED-001"),
            &String::from_str(&env, "General"),
        );

        use crate::VaccineType;
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &String::from_str(&env, "RabiesVax Pro"),
            &env.ledger().timestamp(),
            &(env.ledger().timestamp() + 31536000),
            &String::from_str(&env, "BATCH-001"),
        );
    }

    #[test]
    fn test_revoke_vet_returns_false_for_unregistered_vet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let unknown_vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        // Revoking a non-existent vet should return false, not panic
        let result = client.revoke_vet_license(&admin, &unknown_vet);
        assert!(!result);
    }

    #[test]
    fn test_verify_vet_returns_false_for_unregistered_vet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let unknown_vet = Address::generate(&env);

        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        let result = client.verify_vet(&admin, &unknown_vet);
        assert!(!result);
    }

    #[test]
fn test_add_vaccination_success() {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    let env = Env::default();
    env.mock_all_auths();

    // --- Setup accounts ---
    let admin = Address::generate(&env);
    let veterinarian = Address::generate(&env);
    let pet_owner = Address::generate(&env);

    // --- Deploy contract ---
    let contract_id = env.register_contract(None, PetRegistry {});
    let client = PetRegistryClient::new(&env, &contract_id);

    // --- Setup prerequisite state ---
    // 1. Verify veterinarian
    client.verify_vet(&veterinarian);

    // 2. Register pet
    let pet_id = client.register_pet(
        &pet_owner,
        &String::from_str(&env, "Rex"),
    );

    // --- Call function under test ---
    let vaccine_name = String::from_str(&env, "Rabies");
    let batch_number = String::from_str(&env, "BATCH-001");

    let administered_at = 1_700_000_000;
    let next_due_date = 1_800_000_000;

    let vaccine_id = client.add_vaccination(
        &pet_id,
        &veterinarian,
        &VaccineType::Core,
        &vaccine_name,
        &administered_at,
        &next_due_date,
        &batch_number,
    );

    // --- Assertions ---

    // 1. Vaccine ID should start at 1
    assert_eq!(vaccine_id, 1);

    // 2. VaccinationCount updated
    let count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::VaccinationCount)
        .unwrap();
    assert_eq!(count, 1);

    // 3. Vaccination record stored
    let record: Vaccination = env
        .storage()
        .instance()
        .get(&DataKey::Vaccination(vaccine_id))
        .unwrap();

    assert_eq!(record.pet_id, pet_id);
    assert_eq!(record.veterinarian, veterinarian);
    assert_eq!(record.vaccine_type, VaccineType::Core);
    assert_eq!(record.administered_at, administered_at);
    assert_eq!(record.next_due_date, next_due_date);

    // 4. Ensure plaintext fields are NOT stored
    assert!(record.vaccine_name.is_none());
    assert!(record.batch_number.is_none());

    // 5. Ensure encrypted fields exist
    assert!(record.encrypted_vaccine_name.ciphertext.len() > 0);
    assert!(record.encrypted_batch_number.ciphertext.len() > 0);

    // 6. Pet vaccination index updated
    let pet_vax_count: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PetVaccinationCount(pet_id))
        .unwrap();
    assert_eq!(pet_vax_count, 1);

    let indexed_id: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PetVaccinationByIndex((pet_id, 1)))
        .unwrap();
    assert_eq!(indexed_id, vaccine_id);
}

#[test]
fn test_is_vaccination_current() {
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    let env = Env::default();
    env.mock_all_auths();

    // Set ledger timestamp
    env.ledger().set_timestamp(1_700_000_000);

    let veterinarian = Address::generate(&env);
    let pet_owner = Address::generate(&env);

    let contract_id = env.register_contract(None, PetRegistry {});
    let client = PetRegistryClient::new(&env, &contract_id);

    client.verify_vet(&veterinarian);

    let pet_id = client.register_pet(
        &pet_owner,
        &String::from_str(&env, "Rex"),
    );

    client.add_vaccination(
        &pet_id,
        &veterinarian,
        &VaccineType::Core,
        &String::from_str(&env, "Rabies"),
        &1_600_000_000,
        &1_800_000_000, // future due date
        &String::from_str(&env, "BATCH-001"),
    );

    let result = client.is_vaccination_current(&pet_id, &VaccineType::Core);
    assert!(result);
}

#[test]
fn test_get_overdue_vaccinations_none() {
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_timestamp(1_700_000_000);

    let veterinarian = Address::generate(&env);
    let pet_owner = Address::generate(&env);

    let contract_id = env.register_contract(None, PetRegistry {});
    let client = PetRegistryClient::new(&env, &contract_id);

    client.verify_vet(&veterinarian);

    let pet_id = client.register_pet(
        &pet_owner,
        &String::from_str(&env, "Rex"),
    );

    // Not overdue
    client.add_vaccination(
        &pet_id,
        &veterinarian,
        &VaccineType::Core,
        &String::from_str(&env, "Rabies"),
        &1_600_000_000,
        &1_800_000_000, // future
        &String::from_str(&env, "BATCH-1"),
    );

    let result = client.get_overdue_vaccinations(&pet_id);

    assert_eq!(result.len(), 0);
}



}