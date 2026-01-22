// --- TESTS FOR HASH VERIFICATION ---
    #[test]
    fn test_store_and_verify_hash_success() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let data_id: u64 = 123;
        // Sample hash (e.g., 32 bytes for SHA-256)
        let correct_hash_bytes: Vec<u8> = vec![0u8; 32];
        let correct_hash = Bytes::from_array(&env, &correct_hash_bytes);

        // Test storing and verifying correct hash
        client.store_offchain_data_hash(&data_id, &correct_hash);
        let is_verified_correct = client.verify_offchain_data_hash(&data_id, &correct_hash);
        assert_eq!(is_verified_correct, true);
    }

    #[test]
    fn test_verify_hash_incorrect() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let data_id: u64 = 456;
        let correct_hash_bytes: Vec<u8> = vec![0u8; 32];
        let correct_hash = Bytes::from_array(&env, &correct_hash_bytes);

        let wrong_hash_bytes: Vec<u8> = vec![1u8; 32];
        let wrong_hash = Bytes::from_array(&env, &wrong_hash_bytes);

        // Store correct hash
        client.store_offchain_data_hash(&data_id, &correct_hash);

        // Test verifying with wrong hash
        let is_verified_wrong = client.verify_offchain_data_hash(&data_id, &wrong_hash);
        assert_eq!(is_verified_wrong, false);
    }

    #[test]
    fn test_verify_hash_nonexistent_id() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let correct_hash_bytes: Vec<u8> = vec![0u8; 32];
        let correct_hash = Bytes::from_array(&env, &correct_hash_bytes);

        // Test verifying with non-existent data_id
        let non_existent_id: u64 = 999;
        let is_verified_nonexistent = client.verify_offchain_data_hash(&non_existent_id, &correct_hash);
        assert_eq!(is_verified_nonexistent, false);
    }

    #[test]
    fn test_store_multiple_hashes() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let data_id_1: u64 = 101;
        let hash_bytes_1: Vec<u8> = vec![10u8; 32];
        let hash_1 = Bytes::from_array(&env, &hash_bytes_1);

        let data_id_2: u64 = 102;
        let hash_bytes_2: Vec<u8> = vec![20u8; 32];
        let hash_2 = Bytes::from_array(&env, &hash_bytes_2);

        // Store multiple hashes
        client.store_offchain_data_hash(&data_id_1, &hash_1);
        client.store_offchain_data_hash(&data_id_2, &hash_2);

        // Verify each hash correctly
        let is_verified_1 = client.verify_offchain_data_hash(&data_id_1, &hash_1);
        assert_eq!(is_verified_1, true);

        let is_verified_2 = client.verify_offchain_data_hash(&data_id_2, &hash_2);
        assert_eq!(is_verified_2, true);

        // Verify with incorrect hash for one ID
        let wrong_hash_bytes: Vec<u8> = vec![30u8; 32];
        let wrong_hash = Bytes::from_array(&env, &wrong_hash_bytes);
        let is_verified_wrong = client.verify_offchain_data_hash(&data_id_1, &wrong_hash);
        assert_eq!(is_verified_wrong, false);
    }

    // --- OLD TESTS TO BE REMOVED OR UPDATED ---
    // #[should_panic] test_get_upcoming_vaccinations is marked should_panic, needs review
    // test_vaccination_for_nonexistent_pet needs more concrete checks
}