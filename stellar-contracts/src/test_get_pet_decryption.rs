// ============================================================
// get_pet DECRYPTION ERROR PROPAGATION TESTS
// ============================================================
//
// Comprehensive test coverage for pet data decryption including:
// 1. Successful decryption scenarios with valid encrypted data
// 2. Privacy level enforcement (Public, Restricted, Private)
// 3. Corrupted ciphertext handling (returns None, not partial profiles)
// 4. Access control integration with decryption
//
// decrypt_sensitive_data validates nonce length and XDR decoding.
// Corrupt data is simulated by storing invalid bytes that cannot be
// XDR-decoded as the expected type (String / Vec<Allergy>).
// The contract ensures get_pet returns None rather than partial profiles
// with sentinel "Error" strings.

#[cfg(test)]
mod test_get_pet_decryption {
    use crate::{
        AccessLevel, DataKey, EncryptedData, Gender, Pet, PetChainContract, PetChainContractClient,
        PrivacyLevel, Species,
    };
    use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Vec};

    // ---- helpers ----

    fn setup() -> (Env, PetChainContractClient<'static>, soroban_sdk::Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        (env, client, contract_id)
    }

    fn register_pet(
        client: &PetChainContractClient,
        env: &Env,
        owner: &Address,
        privacy: PrivacyLevel,
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
            &privacy,
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
    fn corrupt_pet_name(env: &Env, contract_id: &Address, pet_id: u64) {
        env.as_contract(contract_id, || {
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

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        });
    }

    fn corrupt_pet_birthday(env: &Env, contract_id: &Address, pet_id: u64) {
        env.as_contract(contract_id, || {
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

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        });
    }

    fn corrupt_pet_breed(env: &Env, contract_id: &Address, pet_id: u64) {
        env.as_contract(contract_id, || {
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

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        });
    }

    fn corrupt_pet_allergies(env: &Env, contract_id: &Address, pet_id: u64) {
        env.as_contract(contract_id, || {
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

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        });
    }

    /// Corrupt the nonce to an invalid length (not 12 bytes)
    fn corrupt_pet_nonce_length(env: &Env, contract_id: &Address, pet_id: u64) {
        env.as_contract(contract_id, || {
            let mut pet: Pet = env
                .storage()
                .instance()
                .get(&DataKey::Pet(pet_id))
                .expect("pet must exist");

            // Invalid nonce length (should be 12 bytes)
            let invalid_nonce = Bytes::from_array(env, &[0x01, 0x02, 0x03]);
            pet.encrypted_name = EncryptedData {
                ciphertext: pet.encrypted_name.ciphertext.clone(),
                nonce: invalid_nonce,
            };

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        });
    }

    // ================================================================
    // SUCCESSFUL DECRYPTION TESTS
    // ================================================================
    // Verify that valid encrypted data decrypts correctly and returns
    // a complete PetProfile with all fields properly decoded.

    #[test]
    fn test_get_pet_valid_data_returns_some() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some(), "valid pet must return Some");
        let profile = result.unwrap();
        assert_eq!(profile.id, pet_id);
        // Confirm no sentinel "Error" strings leak through
        assert_ne!(profile.name, String::from_str(&env, "Error"));
        assert_ne!(profile.birthday, String::from_str(&env, "Error"));
        assert_ne!(profile.breed, String::from_str(&env, "Error"));
    }

    #[test]
    fn test_get_pet_decryption_preserves_data_integrity() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some());
        let profile = result.unwrap();

        // Verify decrypted data matches original registration
        assert_eq!(profile.name, String::from_str(&env, "Buddy"));
        assert_eq!(profile.birthday, String::from_str(&env, "2020-01-01"));
        assert_eq!(profile.breed, String::from_str(&env, "Labrador"));
        assert_eq!(profile.species, Species::Dog);
        assert_eq!(profile.gender, Gender::Male);
        assert_eq!(profile.color, String::from_str(&env, "Brown"));
        assert_eq!(profile.weight, 25u32);
    }

    // ================================================================
    // PRIVACY LEVEL ENFORCEMENT TESTS
    // ================================================================
    // Verify that privacy levels are correctly enforced during decryption.
    // Public pets should be readable by anyone, Restricted by grantees,
    // and Private by owner only.

    #[test]
    fn test_public_pet_decryption_by_owner() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some(), "owner must decrypt their own public pet");
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Public);
    }

    #[test]
    fn test_public_pet_decryption_by_stranger() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let stranger = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        let result = client.get_pet(&pet_id, &stranger);
        assert!(
            result.is_some(),
            "stranger must decrypt a public pet without access grant"
        );
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Public);
    }

    #[test]
    fn test_restricted_pet_decryption_by_owner() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_some(),
            "owner must decrypt their own restricted pet"
        );
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Restricted);
    }

    #[test]
    fn test_restricted_pet_decryption_by_stranger_denied() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let stranger = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

        let result = client.get_pet(&pet_id, &stranger);
        assert!(
            result.is_none(),
            "stranger must not decrypt restricted pet without access grant"
        );
    }

    #[test]
    fn test_restricted_pet_decryption_with_basic_access_grant() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

        client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None);

        let result = client.get_pet(&pet_id, &grantee);
        assert!(
            result.is_some(),
            "grantee with basic access must decrypt restricted pet"
        );
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Restricted);
    }

    #[test]
    fn test_restricted_pet_decryption_with_full_access_grant() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Restricted);

        client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

        let result = client.get_pet(&pet_id, &grantee);
        assert!(
            result.is_some(),
            "grantee with full access must decrypt restricted pet"
        );
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Restricted);
    }

    #[test]
    fn test_private_pet_decryption_by_owner() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some(), "owner must decrypt their own private pet");
        assert_eq!(result.unwrap().privacy_level, PrivacyLevel::Private);
    }

    #[test]
    fn test_private_pet_decryption_by_stranger_denied() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let stranger = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

        let result = client.get_pet(&pet_id, &stranger);
        assert!(
            result.is_none(),
            "stranger must not decrypt private pet even with access grant"
        );
    }

    #[test]
    fn test_private_pet_decryption_with_full_access_grant_still_denied() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let grantee = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Private);

        client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

        let result = client.get_pet(&pet_id, &grantee);
        assert!(
            result.is_none(),
            "full access grant cannot override private privacy level"
        );
    }

    // ================================================================
    // CORRUPTED CIPHERTEXT HANDLING TESTS
    // ================================================================
    // Verify that corrupted encrypted fields cause get_pet to return None
    // rather than partial profiles with sentinel "Error" strings.

    #[test]
    fn test_corrupt_name_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_name(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt name ciphertext must yield None, not a partial profile"
        );
    }

    #[test]
    fn test_corrupt_birthday_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_birthday(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt birthday ciphertext must yield None"
        );
    }

    #[test]
    fn test_corrupt_breed_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_breed(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_none(), "corrupt breed ciphertext must yield None");
    }

    #[test]
    fn test_corrupt_allergies_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_allergies(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "corrupt allergies ciphertext must yield None"
        );
    }

    #[test]
    fn test_corrupt_nonce_length_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_nonce_length(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        assert!(
            result.is_none(),
            "invalid nonce length must yield None (decrypt_sensitive_data validates nonce length)"
        );
    }

    /// Verify the old sentinel value "Error" is never returned for any field,
    /// even when all fields are corrupted simultaneously.
    #[test]
    fn test_all_fields_corrupt_never_returns_error_sentinel() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        corrupt_pet_name(&env, &contract_id, pet_id);
        corrupt_pet_birthday(&env, &contract_id, pet_id);
        corrupt_pet_breed(&env, &contract_id, pet_id);
        corrupt_pet_allergies(&env, &contract_id, pet_id);

        let result = client.get_pet(&pet_id, &owner);
        // Must be None — never a profile containing "Error" strings
        assert!(result.is_none());
    }

    // ================================================================
    // EDGE CASES AND REGRESSION TESTS
    // ================================================================

    /// A non-existent pet must still return None (regression guard).
    #[test]
    fn test_nonexistent_pet_returns_none() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        assert!(client.get_pet(&9999u64, &owner).is_none());
    }

    /// Verify that decryption works correctly after pet profile update.
    #[test]
    fn test_decryption_after_pet_update() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner, PrivacyLevel::Public);

        // Update pet profile with new data
        client.update_pet_profile(
            &pet_id,
            &String::from_str(&env, "UpdatedBuddy"),
            &String::from_str(&env, "2019-06-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Persian"),
            &String::from_str(&env, "White"),
            &15u32,
            &None,
            &PrivacyLevel::Public,
        );

        let result = client.get_pet(&pet_id, &owner);
        assert!(result.is_some(), "decryption must work after update");
        let profile = result.unwrap();
        assert_eq!(profile.name, String::from_str(&env, "UpdatedBuddy"));
        assert_eq!(profile.birthday, String::from_str(&env, "2019-06-15"));
        assert_eq!(profile.breed, String::from_str(&env, "Persian"));
        assert_eq!(profile.species, Species::Cat);
        assert_eq!(profile.gender, Gender::Female);
    }

    /// Verify multiple pets can be decrypted independently with correct data.
    #[test]
    fn test_multiple_pets_decryption_independence() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);

        let pet1_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        );

        let pet2_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Whiskers"),
            &String::from_str(&env, "2021-03-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
            &String::from_str(&env, "Cream"),
            &8u32,
            &None,
            &PrivacyLevel::Public,
        );

        let result1 = client.get_pet(&pet1_id, &owner);
        let result2 = client.get_pet(&pet2_id, &owner);

        assert!(result1.is_some());
        assert!(result2.is_some());

        let profile1 = result1.unwrap();
        let profile2 = result2.unwrap();

        assert_eq!(profile1.name, String::from_str(&env, "Buddy"));
        assert_eq!(profile2.name, String::from_str(&env, "Whiskers"));
        assert_eq!(profile1.breed, String::from_str(&env, "Labrador"));
        assert_eq!(profile2.breed, String::from_str(&env, "Siamese"));
    }

    /// Verify that corrupting one pet's data doesn't affect another pet's decryption.
    #[test]
    fn test_corruption_isolation_between_pets() {
        let (env, client, contract_id) = setup();
        let owner = Address::generate(&env);

        let pet1_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        );

        let pet2_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Whiskers"),
            &String::from_str(&env, "2021-03-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
            &String::from_str(&env, "Cream"),
            &8u32,
            &None,
            &PrivacyLevel::Public,
        );

        // Corrupt only pet1
        corrupt_pet_name(&env, &contract_id, pet1_id);

        let result1 = client.get_pet(&pet1_id, &owner);
        let result2 = client.get_pet(&pet2_id, &owner);

        assert!(result1.is_none(), "corrupted pet1 must return None");
        assert!(result2.is_some(), "pet2 must still decrypt successfully");
        assert_eq!(result2.unwrap().name, String::from_str(&env, "Whiskers"));
    }
}
