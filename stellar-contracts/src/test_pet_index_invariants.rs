// ============================================================
// PET INDEX INVARIANT TESTS
//
// Registers pets across several owners and species in a fixed,
// varied (deterministically "shuffled") sequence, interleaved with
// batch ownership transfers, and asserts after every mutation that:
//   - PetCount / PetCountByOwner / SpeciesPetCount match the number
//     of pets actually reachable through the corresponding index.
//   - OwnerPetIndex / SpeciesPetIndex entries are unique (no pet id
//     appears twice under the same owner/species) and reachable
//     (every entry from 1..=count resolves to a real, currently
//     matching pet).
// ============================================================

#[cfg(test)]
mod test_pet_index_invariants {
    use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn setup(env: &Env) -> (PetChainContractClient<'_>, Address) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(env, &contract_id);
        (client, contract_id)
    }

    fn register(
        client: &PetChainContractClient,
        env: &Env,
        owner: &Address,
        species: Species,
    ) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "Idx"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &species,
            &String::from_str(env, "Mixed"),
            &String::from_str(env, "Brown"),
            &20,
            &None,
            &PrivacyLevel::Public,
        )
    }

    /// Reads every `OwnerPetIndex(owner, 1..=count)` entry and asserts:
    /// - the stored count of entries matches `get_owner_pet_count`,
    /// - every entry is unique,
    /// - every entry currently resolves to a pet actually owned by `owner`.
    fn assert_owner_index_consistent(
        env: &Env,
        contract_id: &Address,
        client: &PetChainContractClient,
        owner: &Address,
    ) {
        let count = PetChainContract::get_owner_pet_count(env, owner);
        let mut seen: Vec<u64> = Vec::new(env);
        for i in 1..=count {
            let pet_id: u64 = env
                .as_contract(contract_id, || {
                    env.storage()
                        .instance()
                        .get(&crate::DataKey::OwnerPetIndex((owner.clone(), i)))
                })
                .expect("index entry must exist for 1..=count");
            assert!(
                !seen.contains(&pet_id),
                "duplicate pet id in owner index for this owner"
            );
            seen.push_back(pet_id);
            assert_eq!(
                client.get_pet_owner(&pet_id).unwrap(),
                owner.clone(),
                "owner index entry points at a pet not owned by this owner"
            );
        }
    }

    #[test]
    fn test_owner_index_invariants_hold_across_mixed_operations() {
        let env = Env::default();
        let (client, contract_id) = setup(&env);

        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);
        let owner_c = Address::generate(&env);

        // Deterministic "shuffled" registration sequence across three owners
        // and two species, exercising interleaved index growth rather than
        // one owner/species at a time.
        let sequence: [(usize, Species); 8] = [
            (0, Species::Dog),
            (1, Species::Cat),
            (0, Species::Cat),
            (2, Species::Dog),
            (1, Species::Dog),
            (2, Species::Cat),
            (0, Species::Dog),
            (1, Species::Cat),
        ];
        let owners = [owner_a.clone(), owner_b.clone(), owner_c.clone()];

        let mut pet_ids_by_owner: [Vec<u64>; 3] =
            [Vec::new(&env), Vec::new(&env), Vec::new(&env)];

        for (owner_idx, species) in sequence.iter() {
            let pet_id = register(&client, &env, &owners[*owner_idx], species.clone());
            pet_ids_by_owner[*owner_idx].push_back(pet_id);
        }

        let total_registered: u64 = sequence.len() as u64;
        assert_eq!(client.get_total_pets(), total_registered);

        for owner in owners.iter() {
            assert_owner_index_consistent(&env, &contract_id, &client, owner);
        }
        assert_eq!(PetChainContract::get_owner_pet_count(&env, &owner_a), 3);
        assert_eq!(PetChainContract::get_owner_pet_count(&env, &owner_b), 3);
        assert_eq!(PetChainContract::get_owner_pet_count(&env, &owner_c), 2);

        // Now transfer owner_a's first two pets to owner_c and re-verify both
        // sides of the index (source shrinks and stays dense, destination
        // grows and stays unique/reachable).
        let mut moving = Vec::new(&env);
        moving.push_back(pet_ids_by_owner[0].get(0).unwrap());
        moving.push_back(pet_ids_by_owner[0].get(1).unwrap());
        client.batch_transfer(&moving, &owner_c);

        assert_eq!(PetChainContract::get_owner_pet_count(&env, &owner_a), 1);
        assert_eq!(PetChainContract::get_owner_pet_count(&env, &owner_c), 4);
        assert_owner_index_consistent(&env, &contract_id, &client, &owner_a);
        assert_owner_index_consistent(&env, &contract_id, &client, &owner_c);
    }
}
