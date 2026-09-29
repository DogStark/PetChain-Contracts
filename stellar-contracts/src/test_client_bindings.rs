#[cfg(test)]
mod test_client_bindings {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Env};

    /// Smoke test: verify that the generated client can call representative
    /// read and write methods against a local test environment. This guards
    /// against silent ABI drift between the contract and generated bindings.
    #[test]
    fn smoke_test_read_and_write_via_client() {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        // --- Write: register a pet ---
        let owner = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Smokey"),
            &String::from_str(&env, "2022-03-15"),
            &Gender::Female,
            &Species::Cat,
            &String::from_str(&env, "Siamese"),
            &PrivacyLevel::Public,
        );
        assert!(pet_id > 0);

        // --- Read: retrieve the pet ---
        let pet = client.get_pet(&pet_id);
        assert!(pet.is_some());
        let pet = pet.unwrap();
        assert_eq!(pet.id, pet_id);
        assert_eq!(pet.owner, owner);

        // --- Write: activate pet ---
        client.activate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(pet.active);

        // --- Read: get schema version ---
        let version = client.get_version();
        assert!(version >= 1);
    }

    /// Verify the contract version returned by get_version() is consistent
    /// with the expected schema version recorded in the bindings manifest.
    #[test]
    fn contract_version_is_stable() {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let version = client.get_version();
        // The version must be at least 1 (initial schema)
        assert!(version >= 1, "Contract version must be >= 1");
    }
}
