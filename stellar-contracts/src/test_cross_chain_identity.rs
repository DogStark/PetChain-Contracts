#[cfg(test)]
mod test_cross_chain_identity {
    use crate::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, PetChainContractClient<'static>, u64, Address) {
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
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Black"),
            &20u32,
            &None,
            &PrivacyLevel::Public,
        );

        (env, client, pet_id, owner)
    }

    #[test]
    fn test_cross_chain_registration_and_resolution() {
        let (env, client, pet_id, owner) = setup();

        let chain_id = String::from_str(&env, "ethereum");
        let external_id = String::from_str(&env, "0xabc123");

        assert!(client.register_cross_chain_id(
            &pet_id,
            &owner,
            &chain_id,
            &external_id,
        ));

        let resolved = client
            .resolve_cross_chain(&chain_id, &external_id)
            .expect("cross-chain lookup should resolve");
        assert_eq!(resolved, pet_id);
    }

    #[test]
    #[should_panic]
    fn test_cross_chain_duplicate_registration_rejected() {
        let (env, client, pet_id, owner) = setup();
        let chain_id = String::from_str(&env, "ethereum");
        let external_id = String::from_str(&env, "0xabc123");

        assert!(client.register_cross_chain_id(
            &pet_id,
            &owner,
            &chain_id,
            &external_id,
        ));
        let _ = client.register_cross_chain_id(&pet_id, &owner, &chain_id, &external_id);
    }

    #[test]
    #[should_panic]
    fn test_cross_chain_non_owner_registration_rejected() {
        let (env, client, pet_id, owner) = setup();
        let attacker = Address::generate(&env);
        let chain_id = String::from_str(&env, "ethereum");
        let external_id = String::from_str(&env, "0xdeadbeef");

        let _ = owner;
        let _ = client.register_cross_chain_id(
            &pet_id,
            &attacker,
            &chain_id,
            &external_id,
        );
    }

    #[test]
    #[should_panic]
    fn test_cross_chain_duplicate_external_id_on_same_chain_rejected() {
        let (env, client, pet_id, owner) = setup();
        let other_owner = Address::generate(&env);
        let other_pet = client.register_pet(
            &other_owner,
            &String::from_str(&env, "Milo"),
            &String::from_str(&env, "2021-02-02"),
            &Gender::Male,
            &Species::Cat,
            &String::from_str(&env, "Tabby"),
            &String::from_str(&env, "Gray"),
            &12u32,
            &None,
            &PrivacyLevel::Public,
        );

        let chain_id = String::from_str(&env, "ethereum");
        let external_id = String::from_str(&env, "0xabc123");

        assert!(client.register_cross_chain_id(
            &pet_id,
            &owner,
            &chain_id,
            &external_id,
        ));
        let _ = client.register_cross_chain_id(
            &other_pet,
            &other_owner,
            &chain_id,
            &external_id,
        );
    }
}
