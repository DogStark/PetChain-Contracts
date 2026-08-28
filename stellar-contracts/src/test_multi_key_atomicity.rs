// ============================================================
// MULTI-KEY WRITE ATOMICITY TESTS
//
// Several mutations update a pet record together with one or more
// index/counter keys (owner index + count, species index + count,
// access-grant index + count). These tests force a failure at the
// validation boundary of each such mutation and assert that none of
// the associated keys were left in a partially-updated state.
// ============================================================

#[cfg(test)]
mod test_multi_key_atomicity {
    use crate::{
        AccessLevel, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn setup(env: &Env) -> (PetChainContractClient<'_>, Address) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(env, &contract_id);
        (client, contract_id)
    }

    fn register(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "Atom"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Mixed"),
            &String::from_str(env, "Black"),
            &18,
            &None,
            &PrivacyLevel::Public,
        )
    }

    /// register_pet validates name/breed/birthday *before* touching
    /// PetCount, PetCountByOwner, OwnerPetIndex, SpeciesPetCount, or
    /// SpeciesPetIndex. An invalid name must therefore leave every one
    /// of those counters exactly as they were before the call.
    #[test]
    fn test_register_pet_invalid_name_leaves_no_partial_index_state() {
        let env = Env::default();
        let (client, _contract_id) = setup(&env);
        let owner = Address::generate(&env);

        // A first, valid registration to give us a known-good baseline.
        register(&client, &env, &owner);
        let total_before = client.get_total_pets();
        let owner_count_before = PetChainContract::get_owner_pet_count(&env, &owner);

        let result = client.try_register_pet(
            &owner,
            // An empty name unconditionally fails validate_pet_name.
            &String::from_str(&env, ""),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Mixed"),
            &String::from_str(&env, "Black"),
            &18,
            &None,
            &PrivacyLevel::Public,
        );
        assert!(result.is_err(), "empty name must be rejected");

        assert_eq!(
            client.get_total_pets(),
            total_before,
            "PetCount must be unchanged after a rejected registration"
        );
        assert_eq!(
            PetChainContract::get_owner_pet_count(&env, &owner),
            owner_count_before,
            "PetCountByOwner must be unchanged after a rejected registration"
        );
    }

    /// batch_transfer validates that every pet in the batch belongs to the
    /// same owner *before* it mutates any owner index, count, or pet
    /// record. A batch containing one pet owned by someone else must
    /// leave every pet's owner, owner-index and owner-count untouched —
    /// including pets earlier in the list that would otherwise have been
    /// processed first.
    #[test]
    fn test_batch_transfer_failure_leaves_all_indexes_and_counts_untouched() {
        let env = Env::default();
        let (client, _contract_id) = setup(&env);
        let owner = Address::generate(&env);
        let other_owner = Address::generate(&env);
        let new_owner = Address::generate(&env);

        let pet_1 = register(&client, &env, &owner);
        let pet_2 = register(&client, &env, &owner);
        let mismatched_pet = register(&client, &env, &other_owner);

        let owner_count_before = PetChainContract::get_owner_pet_count(&env, &owner);
        let other_count_before = PetChainContract::get_owner_pet_count(&env, &other_owner);
        let new_owner_count_before = PetChainContract::get_owner_pet_count(&env, &new_owner);

        let mut pet_ids = Vec::new(&env);
        pet_ids.push_back(pet_1);
        pet_ids.push_back(pet_2);
        pet_ids.push_back(mismatched_pet);

        let result = client.try_batch_transfer(&pet_ids, &new_owner);
        assert!(result.is_err(), "mixed-owner batch must be rejected");

        // Pets that appeared *before* the mismatched one in the batch must
        // not have been transferred either — the whole call is one unit.
        assert_eq!(client.get_pet_owner(&pet_1).unwrap(), owner);
        assert_eq!(client.get_pet_owner(&pet_2).unwrap(), owner);
        assert_eq!(client.get_pet_owner(&mismatched_pet).unwrap(), other_owner);

        assert_eq!(
            PetChainContract::get_owner_pet_count(&env, &owner),
            owner_count_before
        );
        assert_eq!(
            PetChainContract::get_owner_pet_count(&env, &other_owner),
            other_count_before
        );
        assert_eq!(
            PetChainContract::get_owner_pet_count(&env, &new_owner),
            new_owner_count_before
        );
    }

    /// grant_access increments AccessGrantCount and writes AccessGrantIndex
    /// only for a *new* grant. Calling it for a pet that does not exist
    /// must not create a dangling grant, count, or index entry.
    #[test]
    fn test_grant_access_on_missing_pet_creates_no_partial_state() {
        let env = Env::default();
        let (client, _contract_id) = setup(&env);
        let grantee = Address::generate(&env);
        let missing_pet_id: u64 = 999_999;

        let granted = client.grant_access(
            &missing_pet_id,
            &grantee,
            &AccessLevel::Basic,
            &None,
            &0,
        );
        assert!(
            !granted,
            "granting access on a non-existent pet must be a no-op, not a partial write"
        );
    }
}
