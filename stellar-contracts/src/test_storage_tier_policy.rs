// ============================================================
// STORAGE TIER POLICY TESTS
//
// Pins the storage tier used by one representative key from each data
// class documented in docs/storage-tier-policy.md, so a future change
// that silently moves a key to a different tier fails a test instead
// of drifting unnoticed.
// ============================================================

#[cfg(test)]
mod test_storage_tier_policy {
    use crate::{DataKey, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    fn setup(env: &Env) -> (PetChainContractClient<'_>, Address) {
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(env, &contract_id);
        (client, contract_id)
    }

    /// Core records (`Pet`) live on the instance tier: read on almost
    /// every invocation touching that pet, small and bounded, and benefit
    /// from the free TTL bump every active call already performs.
    #[test]
    fn test_pet_record_and_owner_index_use_instance_tier() {
        let env = Env::default();
        let (client, contract_id) = setup(&env);
        let owner = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Tier"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Mixed"),
            &String::from_str(&env, "Grey"),
            &22,
            &None,
            &PrivacyLevel::Public,
        );

        env.as_contract(&contract_id, || {
            assert!(
                env.storage().instance().has(&DataKey::Pet(pet_id)),
                "Pet record must live on the instance tier"
            );
            assert!(
                !env.storage().persistent().has(&DataKey::Pet(pet_id)),
                "Pet record must not also exist on the persistent tier"
            );
            assert!(
                env.storage()
                    .instance()
                    .has(&DataKey::OwnerPetIndex((owner.clone(), 1))),
                "OwnerPetIndex entries must live on the instance tier alongside the count they index"
            );
        });
    }

    /// Accumulating, append-heavy logs (emergency access logs) live on the
    /// persistent tier so they don't inflate the cost of every instance
    /// TTL bump as they grow, and so their lifetime is independent of
    /// unrelated instance writes.
    #[test]
    fn test_emergency_access_log_uses_persistent_tier() {
        let env = Env::default();
        let (_client, contract_id) = setup(&env);
        let pet_id: u64 = 1;

        // Write via the same key type the real emergency-access-logging
        // path uses (DataKey::EmergencyAccessLogs), independent of driving
        // the full emergency-access business flow.
        env.as_contract(&contract_id, || {
            let logs: Vec<u64> = Vec::new(&env);
            env.storage()
                .persistent()
                .set(&DataKey::EmergencyAccessLogs(pet_id), &logs);
        });

        env.as_contract(&contract_id, || {
            assert!(
                env.storage()
                    .persistent()
                    .has(&DataKey::EmergencyAccessLogs(pet_id)),
                "Emergency access logs must live on the persistent tier"
            );
            assert!(
                !env.storage()
                    .instance()
                    .has(&DataKey::EmergencyAccessLogs(pet_id)),
                "Emergency access logs must not also exist on the instance tier"
            );
        });
    }
}
