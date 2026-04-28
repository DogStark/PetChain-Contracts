// ============================================================
// GET LAB RESULTS TESTS
// ============================================================

#[cfg(test)]
mod test_get_lab_results {
    use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Address, Env, String, Vec,
    };

    fn setup() -> (
        Env,
        PetChainContractClient<'static>,
        Address,
        Address,
        u64,
    ) {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        client.init_admin(&admin);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
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

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Smith"),
            &String::from_str(&env, "LIC-001"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&admin, &vet);

        (env, client, owner, vet, pet_id)
    }

    fn add_lab_result(
        client: &PetChainContractClient,
        env: &Env,
        pet_id: u64,
        vet: &Address,
        test_type: &str,
        results: &str,
        timestamp: u64,
    ) -> u64 {
        env.ledger().set_timestamp(timestamp);
        client.add_lab_result(
            &pet_id,
            vet,
            &String::from_str(env, test_type),
            &String::from_str(env, results),
            &String::from_str(env, "0.0-1.0"),
            &None,
            &None,
        )
    }

    #[test]
    fn test_get_lab_results_empty() {
        let (env, client, _owner, _vet, pet_id) = setup();

        // No lab results added - should return empty vector
        let results = client.get_lab_results(&pet_id, &0u64, &10u32);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_lab_results_single() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(
            &client,
            &env,
            pet_id,
            &vet,
            "Blood Test",
            "Normal",
            100,
        );

        let results = client.get_lab_results(&pet_id, &0u64, &10u32);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Blood Test"));
        assert_eq!(results.get(0).unwrap().results, String::from_str(&env, "Normal"));
    }

    #[test]
    fn test_get_lab_results_multiple() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(
            &client,
            &env,
            pet_id,
            &vet,
            "Blood Test",
            "Normal",
            100,
        );
        add_lab_result(
            &client,
            &env,
            pet_id,
            &vet,
            "Urinalysis",
            "Abnormal",
            200,
        );
        add_lab_result(
            &client,
            &env,
            pet_id,
            &vet,
            "X-Ray",
            "Clear",
            300,
        );

        let results = client.get_lab_results(&pet_id, &0u64, &10u32);
        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Blood Test"));
        assert_eq!(results.get(1).unwrap().test_type, String::from_str(&env, "Urinalysis"));
        assert_eq!(results.get(2).unwrap().test_type, String::from_str(&env, "X-Ray"));
    }

    #[test]
    fn test_get_lab_results_pagination_first_page() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);
        add_lab_result(&client, &env, pet_id, &vet, "Test 3", "Result 3", 300);
        add_lab_result(&client, &env, pet_id, &vet, "Test 4", "Result 4", 400);
        add_lab_result(&client, &env, pet_id, &vet, "Test 5", "Result 5", 500);

        // First page: offset 0, limit 2
        let results = client.get_lab_results(&pet_id, &0u64, &2u32);
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Test 1"));
        assert_eq!(results.get(1).unwrap().test_type, String::from_str(&env, "Test 2"));
    }

    #[test]
    fn test_get_lab_results_pagination_second_page() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);
        add_lab_result(&client, &env, pet_id, &vet, "Test 3", "Result 3", 300);
        add_lab_result(&client, &env, pet_id, &vet, "Test 4", "Result 4", 400);
        add_lab_result(&client, &env, pet_id, &vet, "Test 5", "Result 5", 500);

        // Second page: offset 2, limit 2
        let results = client.get_lab_results(&pet_id, &2u64, &2u32);
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Test 3"));
        assert_eq!(results.get(1).unwrap().test_type, String::from_str(&env, "Test 4"));
    }

    #[test]
    fn test_get_lab_results_pagination_last_partial_page() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);
        add_lab_result(&client, &env, pet_id, &vet, "Test 3", "Result 3", 300);
        add_lab_result(&client, &env, pet_id, &vet, "Test 4", "Result 4", 400);
        add_lab_result(&client, &env, pet_id, &vet, "Test 5", "Result 5", 500);

        // Last page: offset 4, limit 2 (only 1 result remaining)
        let results = client.get_lab_results(&pet_id, &4u64, &2u32);
        assert_eq!(results.len(), 1);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Test 5"));
    }

    #[test]
    fn test_get_lab_results_pagination_offset_beyond_count() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);

        // Offset beyond count - should return empty
        let results = client.get_lab_results(&pet_id, &10u64, &5u32);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_lab_results_pagination_zero_limit() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);

        // Zero limit - should return empty
        let results = client.get_lab_results(&pet_id, &0u64, &0u32);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_get_lab_results_pagination_limit_larger_than_remaining() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_lab_result(&client, &env, pet_id, &vet, "Test 1", "Result 1", 100);
        add_lab_result(&client, &env, pet_id, &vet, "Test 2", "Result 2", 200);
        add_lab_result(&client, &env, pet_id, &vet, "Test 3", "Result 3", 300);

        // Offset 1, limit 10 (only 2 results remaining)
        let results = client.get_lab_results(&pet_id, &1u64, &10u32);
        assert_eq!(results.len(), 2);
        assert_eq!(results.get(0).unwrap().test_type, String::from_str(&env, "Test 2"));
        assert_eq!(results.get(1).unwrap().test_type, String::from_str(&env, "Test 3"));
    }
}
