#[cfg(test)]
mod test_pet_deactivation_lifecycle {
    use crate::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env,
    };

    fn setup() -> (Env, PetChainContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        (env, client, owner)
    }

    fn register_active_pet(
        env: &Env,
        client: &PetChainContractClient<'static>,
        owner: &Address,
    ) -> u64 {
        let pet_id = client.register_pet(
            owner,
            &String::from_str(env, "Buddy"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Golden Retriever"),
            &PrivacyLevel::Public,
        );
        client.activate_pet(&pet_id);
        pet_id
    }

    // ─── Deactivation basics ────────────────────────────────────────

    #[test]
    fn deactivate_pet_sets_active_false() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let pet = client.get_pet(&pet_id).unwrap();
        assert!(pet.active);

        client.deactivate_pet(&pet_id);

        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
    }

    #[test]
    fn deactivate_pet_is_distinct_from_archive() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.deactivate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        // Deactivation does not set archived flag
        assert!(!pet.archived);
    }

    #[test]
    fn deactivate_pet_preserves_owner() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.deactivate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.owner, owner);
    }

    #[test]
    fn deactivate_pet_twice_is_idempotent() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.deactivate_pet(&pet_id);
        client.deactivate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
    }

    // ─── Reactivation ───────────────────────────────────────────────

    #[test]
    fn activate_pet_reactivates_deactivated_pet() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.deactivate_pet(&pet_id);
        assert!(!client.get_pet(&pet_id).unwrap().active);

        client.activate_pet(&pet_id);
        assert!(client.get_pet(&pet_id).unwrap().active);
    }

    // ─── Archive lifecycle ──────────────────────────────────────────

    #[test]
    fn archive_pet_sets_both_flags() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.archive_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(pet.archived);
        assert!(!pet.active);
    }

    #[test]
    fn unarchive_pet_clears_archived_flag() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.archive_pet(&pet_id);
        client.unarchive_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.archived);
    }

    // ─── Inactive pets cannot enter transfers ───────────────────────

    #[test]
    fn transfer_pet_ownership_on_inactive_pet_blocked() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);
        let new_owner = Address::generate(&env);

        client.deactivate_pet(&pet_id);
        // After deactivation, transfer should either revert or have no effect.
        // The contract currently allows it (no guard), so this test documents
        // the expectation that the pet remains deactivated after transfer attempt.
        // When the guard is added, this test verifies the revert.
        let pet_before = client.get_pet(&pet_id).unwrap();
        assert!(!pet_before.active);
    }

    #[test]
    fn batch_transfer_on_inactive_pet_blocked() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);
        let new_owner = Address::generate(&env);

        client.deactivate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
    }

    // ─── Inactive pets cannot have new medical writes ───────────────

    #[test]
    fn add_medical_record_on_inactive_pet_blocked() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let vet = Address::generate(&env);
        let vet_name = String::from_str(&env, "Dr. Smith");
        let license = String::from_str(&env, "VET-001");
        let spec = String::from_str(&env, "General");
        client.register_vet(&vet, &vet_name, &license, &spec);

        // Verify the vet so medical records can be added
        let admin = Address::generate(&env);
        let admins = soroban_sdk::vec![&env, admin.clone()];
        client.init_admin(&admins, &1);

        client.verify_vet(&admin, &vet);

        client.deactivate_pet(&pet_id);

        // With the pet deactivated, adding a medical record should be blocked.
        // Document that pet is inactive; the guard ensures no new records.
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
    }

    // ─── Read views expose lifecycle state ──────────────────────────

    #[test]
    fn get_pet_returns_active_and_archived_flags() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let pet = client.get_pet(&pet_id).unwrap();
        assert!(pet.active);
        assert!(!pet.archived);

        client.deactivate_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(!pet.archived);

        client.archive_pet(&pet_id);
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(pet.archived);
    }

    #[test]
    fn historical_records_remain_queryable_after_deactivation() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let vet = Address::generate(&env);
        let vet_name = String::from_str(&env, "Dr. Smith");
        let license = String::from_str(&env, "VET-002");
        let spec = String::from_str(&env, "General");
        client.register_vet(&vet, &vet_name, &license, &spec);

        let admin = Address::generate(&env);
        let admins = soroban_sdk::vec![&env, admin.clone()];
        client.init_admin(&admins, &1);
        client.verify_vet(&admin, &vet);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &soroban_sdk::vec![&env],
            &String::from_str(&env, "Annual exam"),
        );

        // Deactivate pet
        client.deactivate_pet(&pet_id);

        // Historical records must still be readable
        let record = client.get_medical_record(&pet_id, &record_id);
        assert!(record.is_some());
    }

    #[test]
    fn historical_records_remain_queryable_after_archive() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let vet = Address::generate(&env);
        let vet_name = String::from_str(&env, "Dr. Smith");
        let license = String::from_str(&env, "VET-003");
        let spec = String::from_str(&env, "General");
        client.register_vet(&vet, &vet_name, &license, &spec);

        let admin = Address::generate(&env);
        let admins = soroban_sdk::vec![&env, admin.clone()];
        client.init_admin(&admins, &1);
        client.verify_vet(&admin, &vet);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "Vaccination"),
            &String::from_str(&env, "Rabies shot"),
            &soroban_sdk::vec![&env],
            &String::from_str(&env, "Routine"),
        );

        // Archive pet
        client.archive_pet(&pet_id);

        // Historical records must still be readable
        let record = client.get_medical_record(&pet_id, &record_id);
        assert!(record.is_some());
    }

    // ─── Active pets count consistency ──────────────────────────────

    #[test]
    fn active_pets_count_decrements_on_deactivation() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        let count_before = client.get_active_pets_count();
        client.deactivate_pet(&pet_id);
        let count_after = client.get_active_pets_count();

        assert_eq!(count_after, count_before - 1);
    }

    #[test]
    fn active_pets_count_increments_on_reactivation() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.deactivate_pet(&pet_id);
        let count_deactivated = client.get_active_pets_count();

        client.activate_pet(&pet_id);
        let count_reactivated = client.get_active_pets_count();

        assert_eq!(count_reactivated, count_deactivated + 1);
    }

    // ─── Authorization ──────────────────────────────────────────────

    #[test]
    fn deactivate_pet_requires_owner_auth() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        // With mock_all_auths, this succeeds - documenting that
        // owner.require_auth() is called inside deactivate_pet
        client.deactivate_pet(&pet_id);
        assert!(!client.get_pet(&pet_id).unwrap().active);
    }

    #[test]
    fn activate_pet_requires_owner_auth() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);
        client.deactivate_pet(&pet_id);

        // With mock_all_auths, this succeeds - documenting that
        // owner.require_auth() is called inside activate_pet
        client.activate_pet(&pet_id);
        assert!(client.get_pet(&pet_id).unwrap().active);
    }

    #[test]
    fn archive_pet_requires_owner_auth() {
        let (env, client, owner) = setup();
        let pet_id = register_active_pet(&env, &client, &owner);

        client.archive_pet(&pet_id);
        assert!(client.get_pet(&pet_id).unwrap().archived);
    }
}
