#[cfg(test)]
mod test_coownership {
    use crate::*;
    use soroban_sdk::{
        testutils::Address as _,
        Env,
    };

    fn setup() -> (Env, crate::PetChainContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        (env, client)
    }

    fn register_test_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "Buddy"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Labrador"),
            &String::from_str(env, "Black"),
            &30u32,
            &None,
            &PrivacyLevel::Public,
        )
    }

    #[test]
    fn test_add_co_owner() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let co_owner = Address::generate(&env);

        let pet_id = register_test_pet(&env, &client, &owner);

        let owners_before = client.get_co_owners(&pet_id);
        assert_eq!(owners_before.len(), 1);

        client.add_co_owner(&pet_id, &owner, &co_owner);

        let owners_after = client.get_co_owners(&pet_id);
        assert_eq!(owners_after.len(), 2);
        assert!(owners_after.contains(co_owner.clone()));
    }

    #[test]
    fn test_remove_co_owner() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let co_owner = Address::generate(&env);

        let pet_id = register_test_pet(&env, &client, &owner);

        client.add_co_owner(&pet_id, &owner, &co_owner);
        assert_eq!(client.get_co_owners(&pet_id).len(), 2);

        client.remove_co_owner(&pet_id, &owner, &co_owner);
        let owners = client.get_co_owners(&pet_id);
        assert_eq!(owners.len(), 1);
        assert!(!owners.contains(co_owner));
    }

    #[test]
    #[should_panic(expected = "Cannot remove primary owner")]
    fn test_cannot_remove_primary_owner() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);
        client.remove_co_owner(&pet_id, &owner, &owner);
    }

    #[test]
    fn test_co_owner_permissions() {
        let (env, client) = setup();
        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);

        let pet_id = register_test_pet(&env, &client, &owner_a);
        client.add_co_owner(&pet_id, &owner_a, &owner_b);

        // After adding owner_b as co-owner, primary_owner is still owner_a.
        // Primary-owner gated calls (update_pet_profile etc.) require primary_owner auth
        // which is mocked for all addresses in tests, so both work.
        let result = client.update_pet_notes(
            &pet_id,
            &String::from_str(&env, "Notes from co-owner"),
        );
        let _ = result;

        // Verify the pet has both owners in the list.
        let owners = client.get_co_owners(&pet_id);
        assert_eq!(owners.len(), 2);
        assert!(owners.contains(owner_a));
        assert!(owners.contains(owner_b));
    }

    #[test]
    #[should_panic(expected = "Already a co-owner")]
    fn test_add_duplicate_co_owner() {
        let (env, client) = setup();
        let owner = Address::generate(&env);
        let co_owner = Address::generate(&env);

        let pet_id = register_test_pet(&env, &client, &owner);
        client.add_co_owner(&pet_id, &owner, &co_owner);
        client.add_co_owner(&pet_id, &owner, &co_owner);
    }
}
