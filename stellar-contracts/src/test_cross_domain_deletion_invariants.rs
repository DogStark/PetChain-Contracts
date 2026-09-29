// ============================================================
// CROSS-DOMAIN PET DELETION INVARIANT TESTS
//
// Issue #1353
//
// Defines soft-delete (deactivate/archive) versus purge
// behaviour and tests each linked domain after lifecycle
// transitions.
//
// Acceptance criteria verified:
//   - Active queries exclude deleted pets.
//   - Historical proofs follow policy.
//   - No orphaned index can authorize access.
// ============================================================

use crate::{
    AccessLevel, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Multi-domain lifecycle fixture
// ---------------------------------------------------------------------------

/// Registers an owner, vet, admin, and one active pet, then populates
/// every linked domain (medical records, vaccinations, access grants,
/// behaviour, activity, breeding, custody) so that deletion
/// invariants can be asserted across all of them.
fn setup_full_lifecycle(env: &Env) -> (PetChainContractClient<'_>, Address, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let vet = Address::generate(env);
    let owner = Address::generate(env);
    let grantee = Address::generate(env);

    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Full"),
        &String::from_str(env, "LIC-FULL"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Cosmo"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Yellow"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.activate_pet(&pet_id);

    // ── Medical records ──────────────────────────────────
    let _record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(env, "Checkup"),
        &String::from_str(env, "Healthy"),
        &Vec::new(env),
        &String::from_str(env, "Annual exam"),
    );

    // ── Vaccination ──────────────────────────────────────
    let _vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(env, "Rabies Vaccine"),
        env.ledger().timestamp(),
        env.ledger().timestamp() + 365 * 86_400,
        env.ledger().timestamp() + 365 * 86_400,
        &String::from_str(env, "BATCH-001"),
    );

    // ── Access grant ─────────────────────────────────────
    client.grant_access(
        &pet_id,
        &grantee,
        &AccessLevel::Basic,
        &None,
        &0u64,
    );

    // ── Behaviour record ─────────────────────────────────
    client.add_behavior_record(
        &pet_id,
        &owner,
        &BehaviorType::Friendly,
        1,
        &String::from_str(env, "Good with children"),
    );

    // ── Activity record ──────────────────────────────────
    client.add_activity_record(
        &pet_id,
        &ActivityType::Walk,
        30,
        5,
        1_000,
        &String::from_str(env, "Morning walk"),
    );

    // ── Breeding record ──────────────────────────────────
    let sire_id = client.register_pet(
        &owner,
        &String::from_str(env, "Sire"),
        &String::from_str(env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.activate_pet(&sire_id);
    let dam_id = client.register_pet(
        &owner,
        &String::from_str(env, "Dam"),
        &String::from_str(env, "2019-06-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Yellow"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.activate_pet(&dam_id);
    client.add_breeding_record(
        &sire_id,
        &dam_id,
        env.ledger().timestamp(),
        &String::from_str(env, "Planned"),
    );

    (client, owner, vet, pet_id)
}

// ---------------------------------------------------------------------------
// Invariant 1 — Active queries exclude deleted pets
// ---------------------------------------------------------------------------

#[cfg(test)]
mod test_cross_domain_deletion_invariants {
    use super::*;

    #[test]
    fn active_pet_count_decrements_on_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let active_before = PetChainContract::get_active_pets_count(&env);

        client.deactivate_pet(&pet_id);
        let active_after = PetChainContract::get_active_pets_count(&env);

        assert_eq!(active_after, active_before - 1);
    }

    #[test]
    fn active_pet_count_decrements_on_archive() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let active_before = PetChainContract::get_active_pets_count(&env);

        client.archive_pet(&pet_id);
        let active_after = PetChainContract::get_active_pets_count(&env);

        assert_eq!(active_after, active_before - 1);
    }

    #[test]
    fn is_pet_active_returns_false_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.deactivate_pet(&pet_id);
        assert!(!PetChainContract::is_pet_active(env.clone(), pet_id));
    }

    #[test]
    fn is_pet_active_returns_false_after_archive() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.archive_pet(&pet_id);
        assert!(!PetChainContract::is_pet_active(env.clone(), pet_id));
    }

    #[test]
    fn reactivating_deactivated_pet_restores_active_count() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.deactivate_pet(&pet_id);
        let active_after_deact = PetChainContract::get_active_pets_count(&env);

        client.activate_pet(&pet_id);
        let active_after_react = PetChainContract::get_active_pets_count(&env);

        assert_eq!(active_after_react, active_after_deact + 1);
    }

    #[test]
    fn owner_pet_count_decrements_on_deactivation() {
        let env = Env::default();
        let (client, owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let count_before = PetChainContract::get_owner_pet_count(&env, &owner);

        client.deactivate_pet(&pet_id);
        let count_after = PetChainContract::get_owner_pet_count(&env, &owner);

        assert_eq!(count_after, count_before - 1);
    }

    #[test]
    fn owner_pet_count_decrements_on_archive() {
        let env = Env::default();
        let (client, owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let count_before = PetChainContract::get_owner_pet_count(&env, &owner);

        client.archive_pet(&pet_id);
        let count_after = PetChainContract::get_owner_pet_count(&env, &owner);

        assert_eq!(count_after, count_before - 1);
    }

    #[test]
    fn species_pet_count_decrements_on_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let count_before = PetChainContract::get_species_pet_count(&env, &Species::Dog);

        client.deactivate_pet(&pet_id);
        let count_after = PetChainContract::get_species_pet_count(&env, &Species::Dog);

        assert_eq!(count_after, count_before - 1);
    }

    #[test]
    fn species_pet_count_decrements_on_archive() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.activate_pet(&pet_id);
        let count_before = PetChainContract::get_species_pet_count(&env, &Species::Dog);

        client.archive_pet(&pet_id);
        let count_after = PetChainContract::get_species_pet_count(&env, &Species::Dog);

        assert_eq!(count_after, count_before - 1);
    }

    #[test]
    fn get_pet_returns_inactive_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.deactivate_pet(&pet_id);

        // Deactivated pets are still retrievable by ID but are not active
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(!pet.archived);
    }

    #[test]
    fn get_pet_returns_archived_after_archive() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.archive_pet(&pet_id);

        // Archived pets are still retrievable by ID but are not active
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(pet.archived);
    }

    // -------------------------------------------------------------------
    // Invariant 2 — Historical proofs follow policy
    // -------------------------------------------------------------------

    #[test]
    fn medical_records_remain_queryable_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        client.deactivate_pet(&pet_id);

        // Historical medical records must still be readable
        let record = client.get_medical_record(&record_id).unwrap();
        assert_eq!(record.pet_id, pet_id);
    }

    #[test]
    fn vaccination_records_remain_queryable_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        let vax_id = client.add_vaccination(
            &pet_id,
            &Address::generate(&env),
            &VaccineType::Rabies,
            &String::from_str(&env, "Rabies Vaccine"),
            env.ledger().timestamp(),
            env.ledger().timestamp() + 365 * 86_400,
            env.ledger().timestamp() + 365 * 86_400,
            &String::from_str(&env, "BATCH-001"),
        );

        client.deactivate_pet(&pet_id);

        // Historical vaccination records must still be readable
        let vax = client.get_vaccinations(&vax_id).unwrap();
        assert_eq!(vax.pet_id, pet_id);
    }

    #[test]
    fn behaviour_records_remain_queryable_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.deactivate_pet(&pet_id);

        // Behaviour records must still be readable for historical proof
        let behaviour_count = client.get_behavior_count(pet_id);
        assert!(behaviour_count > 0);
    }

    #[test]
    fn purge_deleted_records_removes_medical_records() {
        let env = Env::default();
        let (client, admin, _vet, pet_id) = setup_full_lifecycle(&env);

        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        client.deactivate_pet(&pet_id);

        // Set retention period to 0 so purge immediately removes records
        client.set_retention_period(&admin, &0);

        let purge_result = client.purge_deleted_records(&pet_id, &admin, &false);
        assert!(purge_result.deleted.len() > 0);

        // After purge, the medical record should no longer be accessible
        let record = client.get_medical_record(&record_id);
        assert!(record.is_none());
    }

    #[test]
    fn historical_proof_integrity_after_soft_delete() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        // Add a medical record before deactivation
        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        // Deactivate the pet (soft delete)
        client.deactivate_pet(&pet_id);

        // Historical proof: the record should still exist and be
        // attributable to the deleted pet
        let record = client.get_medical_record(&record_id).unwrap();
        assert_eq!(record.pet_id, pet_id);

        // The pet should be deactivated but not purged
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(!pet.archived);
    }

    #[test]
    fn historical_proof_integrity_after_hard_delete() {
        let env = Env::default();
        let (client, admin, _vet, pet_id) = setup_full_lifecycle(&env);

        // Add a medical record before archiving
        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        // Archive the pet (soft delete)
        client.archive_pet(&pet_id);

        // Set retention period to 0 so purge immediately removes records
        client.set_retention_period(&admin, &0);

        // Purge deleted records (hard delete)
        let purge_result = client.purge_deleted_records(&pet_id, &admin, &false);
        assert!(purge_result.deleted.len() > 0);

        // After hard delete, the medical record should no longer be
        // accessible — historical proof policy is enforced
        let record = client.get_medical_record(&record_id);
        assert!(record.is_none());
    }

    // -------------------------------------------------------------------
    // Invariant 3 — No orphaned index can authorize access
    // -------------------------------------------------------------------

    #[test]
    fn revoked_access_grant_cannot_authorize_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);
        let grantee = Address::generate(&env);

        // Grant access, then revoke it
        client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None, &0u64);
        client.revoke_access(&pet_id, &grantee);

        // Deactivate the pet
        client.deactivate_pet(&pet_id);

        // Revoked grant must not authorize access even though the pet is
        // now deleted/deactivated — the orphaned index entry is inert.
        let access = PetChainContract::check_access(env.clone(), pet_id, grantee);
        assert_eq!(access, AccessLevel::None);
    }

    #[test]
    fn expired_access_grant_cannot_authorize_after_deactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);
        let grantee = Address::generate(&env);

        // Grant access with a short TTL
        client.grant_access(
            &pet_id,
            &grantee,
            &AccessLevel::Basic,
            &Some(env.ledger().timestamp() + 100),
            &0u64,
        );

        // Advance time past the grant expiry
        env.ledger().with_mut(|l| l.timestamp += 200);

        // Deactivate the pet
        client.deactivate_pet(&pet_id);

        // Expired grant must not authorize access even though the pet is
        // now deleted/deactivated — the orphaned index entry is inert.
        let access = PetChainContract::check_access(env.clone(), pet_id, grantee);
        assert_eq!(access, AccessLevel::None);
    }

    #[test]
    fn no_orphaned_index_authorizes_access_for_deleted_pet() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);
        let grantee = Address::generate(&env);

        // Grant access then deactivate the pet
        client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None, &0u64);
        client.deactivate_pet(&pet_id);

        // Even though the grant entry may still exist in storage until
        // compact_storage runs, check_access must return None for a
        // deleted pet — no orphaned index can authorize access.
        let access = PetChainContract::check_access(env.clone(), pet_id, grantee);
        assert_eq!(access, AccessLevel::None);
    }

    #[test]
    fn active_query_excludes_pet_with_archived_flag() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        client.archive_pet(&pet_id);

        // Archived pets must not appear in any active query
        assert!(!PetChainContract::is_pet_active(env.clone(), pet_id));

        // Historical queries should still find the pet
        let pet = client.get_pet(&pet_id).unwrap();
        assert!(pet.archived);
    }

    #[test]
    fn cross_domain_consistency_after_reactivation() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        // Deactivate then reactivate
        client.deactivate_pet(&pet_id);
        client.activate_pet(&pet_id);

        // All indexes should be consistent after reactivation
        assert!(PetChainContract::is_pet_active(env.clone(), pet_id));

        let owner = client.get_pet(&pet_id).unwrap().owner;
        let owner_count = PetChainContract::get_owner_pet_count(&env, &owner);
        assert!(owner_count > 0);

        let species_count = PetChainContract::get_species_pet_count(&env, &Species::Dog);
        assert!(species_count > 0);
    }

    #[test]
    fn soft_delete_preserves_data_for_historical_proof() {
        let env = Env::default();
        let (client, _owner, _vet, pet_id) = setup_full_lifecycle(&env);

        // Add records before soft delete
        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        // Soft delete (deactivate)
        client.deactivate_pet(&pet_id);

        // Data must still exist for historical proof
        let record = client.get_medical_record(&record_id).unwrap();
        assert_eq!(record.pet_id, pet_id);

        let pet = client.get_pet(&pet_id).unwrap();
        assert!(!pet.active);
        assert!(!pet.archived);
    }

    #[test]
    fn purge_enforces_historical_proof_policy() {
        let env = Env::default();
        let (client, admin, _vet, pet_id) = setup_full_lifecycle(&env);

        // Add records before archive
        let record_id = client.add_medical_record(
            &pet_id,
            &Address::generate(&env),
            &String::from_str(&env, "Checkup"),
            &String::from_str(&env, "Healthy"),
            &Vec::new(&env),
            &String::from_str(&env, "Annual exam"),
        );

        // Archive (soft delete)
        client.archive_pet(&pet_id);

        // Set retention period to 0 so purge immediately removes records
        client.set_retention_period(&admin, &0);

        // Purge (hard delete)
        let purge_result = client.purge_deleted_records(&pet_id, &admin, &false);
        assert!(purge_result.deleted.len() > 0);

        // After hard delete, data must be gone per policy
        let record = client.get_medical_record(&record_id);
        assert!(record.is_none());
    }
}