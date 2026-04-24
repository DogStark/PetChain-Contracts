// ============================================================
// get_pet DECRYPTION ERROR PROPAGATION TESTS
// ============================================================
//
// decrypt_sensitive_data is currently a passthrough (Ok(ciphertext.clone())).
// Corrupt data is therefore simulated by storing a Pet whose encrypted fields
// contain raw bytes that cannot be XDR-decoded as the expected type (String /
// Vec<Allergy>).  The fix ensures get_pet returns None rather than a partial
// profile with sentinel "Error" strings.

#[cfg(test)]
mod test_get_pet_decryption {
    use crate::{
        DataKey, EncryptedData, Gender, Pet, PetChainContract, PetChainContractClient,
        PrivacyLevel, Species,
    };
    use soroban_sdk::{
        testutils::Address as _, Address, Bytes, Env, String, Vec,
    };

    // ---- helpers ----

    fn setup() -> (Env, PetChainContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
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

    fn setup_verified_vet(client: &PetChainContractClient, env: &Env) -> Address {
        let admin = Address::generate(env);
        let vet = Address::generate(env);
        let mut admins = soroban_sdk::Vec::new(env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);
        client.register_vet(
            &vet,
            &String::from_str(env, "Dr. Test"),
            &String::from_str(env, "LIC-001"),
            &String::from_str(env, "General"),
        );
        client.verify_vet(&admin, &vet);
        vet
    }

    /// Overwrite a stored Pet's encrypted_name with bytes that are not valid
    /// XDR for a soroban String, then assert get_pet returns None.
    fn corrupt_pet_name(env: &Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("pet must exist before corruption");

        // 0xFF bytes are not valid XDR for a soroban String
        let garbage = Bytes::from_array(env, &[0xFF, 0xFE, 0xFD, 0xFC]);
        pet.encrypted_name = EncryptedData {
            ciphertext: garbage.clone(),
            nonce: garbage,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pet(pet_id), &pet);
    }

    fn corrupt_pet_birthday(env: &Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("pet must exist");

        let garbage = Bytes::from_array(env, &[0xDE, 0xAD, 0xBE, 0xEF]);
        pet.encrypted_birthday = EncryptedData {
            ciphertext: garbage.clone(),
            nonce: garbage,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pet(pet_id), &pet);
    }

    fn corrupt_pet_breed(env: &Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("pet must exist");

        let garbage = Bytes::from_array(env, &[0x00, 0x01, 0x02, 0x03]);
        pet.encrypted_breed = EncryptedData {
            ciphertext: garbage.clone(),
            nonce: garbage,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pet(pet_id), &pet);
    }

    fn corrupt_pet_allergies(env: &Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("pet must exist");

        let garbage = Bytes::from_array(env, &[0xAB, 0xCD, 0xEF, 0x01]);
        pet.encrypted_allergies = EncryptedData {
            ciphertext: garbage.clone(),
            nonce: garbage,
        };

        env.storage()
            .instance()
            .set(&DataKey::Pet(pet_id), &pet);
    }

    // ---- happy path: valid data still works ----

    #[test]
    fn test_get_pet_valid_data_returns_some() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some(), "valid pet must return Some");
        let profile = result.unwrap();
        assert_eq!(profile.id, pet_id);
        // Confirm no sentinel "Error" strings leak through
        assert_ne!(profile.name, String::from_str(&env, "Error"));
        assert_ne!(profile.birthday, String::from_str(&env, "Error"));
        assert_ne!(profile.breed, String::from_str(&env, "Error"));
    }

    // ---- corrupt fields return None, not masked output ----

    #[test]
    fn test_corrupt_name_returns_none() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        corrupt_pet_name(&env, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt name ciphertext must yield None, not a partial profile"
        );
    }

    #[test]
    fn test_corrupt_birthday_returns_none() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        corrupt_pet_birthday(&env, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt birthday ciphertext must yield None"
        );
    }

    #[test]
    fn test_corrupt_breed_returns_none() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        corrupt_pet_breed(&env, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt breed ciphertext must yield None"
        );
    }

    #[test]
    fn test_corrupt_allergies_returns_none() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        corrupt_pet_allergies(&env, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt allergies ciphertext must yield None"
        );
    }

    /// Verify the old sentinel value "Error" is never returned for any field,
    /// even when all fields are corrupted simultaneously.
    #[test]
    fn test_all_fields_corrupt_never_returns_error_sentinel() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        corrupt_pet_name(&env, pet_id);
        corrupt_pet_birthday(&env, pet_id);
        corrupt_pet_breed(&env, pet_id);
        corrupt_pet_allergies(&env, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        // Must be None — never a profile containing "Error" strings
        assert!(result.is_none());
    }

    /// A non-existent pet must still return None (regression guard).
    #[test]
    fn test_nonexistent_pet_returns_none() {
        let (env, client) = setup();
        let viewer = Address::generate(&env);
        assert!(client.get_pet(&9999u64, &viewer).is_none());
    }

    // ============================================================
    // get_pet_full_profile TESTS
    // ============================================================

    #[test]
    fn test_get_pet_full_profile_valid_data_returns_some() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some(), "valid pet must return Some");
        let full_profile = result.unwrap();
        assert_eq!(full_profile.profile.id, pet_id);
        assert_eq!(full_profile.profile.owner, owner);
        assert!(full_profile.latest_vaccination_id.is_none());
        assert_eq!(full_profile.active_medications_count, 0);
        assert!(!full_profile.has_insurance);
    }

    #[test]
    fn test_get_pet_full_profile_includes_latest_vaccination() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_verified_vet(&client, &env);

        let vax_id = client.add_vaccination(
            &pet_id,
            &vet,
            &crate::VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            &1000u64,
            &2000u64,
            &String::from_str(&env, "BATCH123"),
        );

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();
        assert!(full_profile.latest_vaccination_id.is_some());
        assert_eq!(full_profile.latest_vaccination_id.unwrap(), vax_id);
    }

    #[test]
    fn test_get_pet_full_profile_includes_active_medications() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_verified_vet(&client, &env);

        client.add_medication(
            &pet_id,
            &String::from_str(&env, "Amoxicillin"),
            &String::from_str(&env, "500mg"),
            &String::from_str(&env, "Twice daily"),
            &1000u64,
            &None,
            &vet,
        );

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();
        assert_eq!(full_profile.active_medications_count, 1);
    }

    #[test]
    fn test_get_pet_full_profile_includes_insurance() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        client.add_insurance_policy(
            &pet_id,
            &String::from_str(&env, "POL123"),
            &String::from_str(&env, "PetCare Inc"),
            &String::from_str(&env, "Comprehensive"),
            &5000u64,
            &100000u64,
            &5000u64,
        );

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();
        assert!(full_profile.has_insurance);
    }

    #[test]
    fn test_get_pet_full_profile_aggregates_all_data() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_verified_vet(&client, &env);

        client.add_vaccination(
            &pet_id,
            &vet,
            &crate::VaccineType::Parvovirus,
            &String::from_str(&env, "Parvovirus Vaccine"),
            &1000u64,
            &2000u64,
            &String::from_str(&env, "BATCH456"),
        );

        client.add_medication(
            &pet_id,
            &String::from_str(&env, "Lisinopril"),
            &String::from_str(&env, "10mg"),
            &String::from_str(&env, "Once daily"),
            &1000u64,
            &None,
            &vet,
        );

        client.add_insurance_policy(
            &pet_id,
            &String::from_str(&env, "POL456"),
            &String::from_str(&env, "VetShield"),
            &String::from_str(&env, "Premium"),
            &7500u64,
            &150000u64,
            &5000u64,
        );

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();

        assert_eq!(full_profile.profile.id, pet_id);
        assert!(full_profile.latest_vaccination_id.is_some());
        assert_eq!(full_profile.active_medications_count, 1);
        assert!(full_profile.has_insurance);
    }

    #[test]
    fn test_get_pet_full_profile_respects_access_control() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let stranger = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        let result = client.get_pet_full_profile(&pet_id, &stranger);
        assert!(result.is_none(), "stranger must not access private pet");
    }

    #[test]
    fn test_get_pet_full_profile_with_public_pet() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let viewer = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "PublicPet"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Poodle"),
            &String::from_str(&env, "White"),
            &20u32,
            &None,
            &PrivacyLevel::Public,
        );

        let result = client.get_pet_full_profile(&pet_id, &viewer);
        assert!(result.is_some(), "viewer must access public pet");
        let full_profile = result.unwrap();
        assert_eq!(full_profile.profile.id, pet_id);
    }

    #[test]
    fn test_get_pet_full_profile_with_granted_access() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);

        client.grant_access(&pet_id, &grantee, &crate::AccessLevel::Full, &None);

        let result = client.get_pet_full_profile(&pet_id, &grantee);
        assert!(result.is_some(), "grantee with Full access must access pet");
        let full_profile = result.unwrap();
        assert_eq!(full_profile.profile.id, pet_id);
    }

    #[test]
    fn test_get_pet_full_profile_nonexistent_pet_returns_none() {
        let (env, client) = setup();
        let viewer = Address::generate(&env);
        let result = client.get_pet_full_profile(&9999u64, &viewer);
        assert!(result.is_none(), "nonexistent pet must return None");
    }

    #[test]
    fn test_get_pet_full_profile_with_multiple_vaccinations_returns_latest() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_verified_vet(&client, &env);

        client.add_vaccination(
            &pet_id,
            &vet,
            &crate::VaccineType::Rabies,
            &String::from_str(&env, "Rabies V1"),
            &1000u64,
            &2000u64,
            &String::from_str(&env, "BATCH1"),
        );

        let latest_vax_id = client.add_vaccination(
            &pet_id,
            &vet,
            &crate::VaccineType::Leukemia,
            &String::from_str(&env, "Leukemia V1"),
            &3000u64,
            &4000u64,
            &String::from_str(&env, "BATCH2"),
        );

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();
        assert!(full_profile.latest_vaccination_id.is_some());
        assert_eq!(full_profile.latest_vaccination_id.unwrap(), latest_vax_id);
    }

    #[test]
    fn test_get_pet_full_profile_excludes_inactive_medications() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_verified_vet(&client, &env);

        let med_id = client.add_medication(
            &pet_id,
            &String::from_str(&env, "Aspirin"),
            &String::from_str(&env, "100mg"),
            &String::from_str(&env, "Once daily"),
            &1000u64,
            &None,
            &vet,
        );

        client.mark_medication_completed(&med_id);

        let result = client.get_pet_full_profile(&pet_id, &owner);
        assert!(result.is_some());
        let full_profile = result.unwrap();
        assert_eq!(full_profile.active_medications_count, 0, "inactive medications must be excluded");
    }
}
