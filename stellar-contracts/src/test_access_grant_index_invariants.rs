// ============================================================
// ACCESS-GRANT INDEX INVARIANT TESTS (Issue #1158)
//
// `grant_access` / `revoke_access` / `compact_storage` maintain several
// related keys per pet:
//   - `AccessGrant((pet_id, grantee))`      -> the grant record itself
//   - `AccessGrantCount(pet_id)`            -> number of index slots
//   - `AccessGrantIndex((pet_id, 1..=count))` -> grantee at each slot
//
// These tests prove:
//   - `AccessGrantIndex` entries (1..=count) are reachable and unique, and
//     every entry resolves to a grant record that actually exists.
//   - A revoked (inactive) or expired grant can never authorize access via
//     `check_access`, even though its index/count bookkeeping may still
//     reference it until `compact_storage` runs.
//   - After `compact_storage`, stale (inactive/expired) entries are fully
//     removed from both the grant map and the index, and the remaining
//     index stays contiguous/consistent with the (now smaller) count.
// ============================================================

use crate::{
    AccessGrant, AccessLevel, DataKey, Gender, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup(env: &Env) -> (PetChainContractClient<'static>, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    (client, contract_id)
}

fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Grantee"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Mixed"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    )
}

fn grant_count(env: &Env, contract_id: &Address, pet_id: u64) -> u64 {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0)
    })
}

/// Read the `AccessGrantIndex(pet_id, 1..=count)` slots directly out of
/// contract instance storage and assert every entry is unique and
/// resolves to a real `AccessGrant` record for that pet.
fn assert_index_consistent(env: &Env, contract_id: &Address, pet_id: u64) {
    let count = grant_count(env, contract_id, pet_id);
    env.as_contract(contract_id, || {
        let mut seen: Vec<Address> = Vec::new(env);
        for i in 1u64..=count {
            let grantee: Address = env
                .storage()
                .instance()
                .get(&DataKey::AccessGrantIndex((pet_id, i)))
                .expect("index slot 1..=count must be reachable");

            assert!(
                !seen.contains(&grantee),
                "grantee must not appear twice in the index"
            );
            seen.push_back(grantee.clone());

            let grant: AccessGrant = env
                .storage()
                .instance()
                .get(&DataKey::AccessGrant((pet_id, grantee)))
                .expect("every index entry must resolve to a real grant record");
            assert_eq!(grant.pet_id, pet_id);
        }
    });
}

#[test]
fn test_grant_index_consistent_after_multiple_grants() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    let g3 = Address::generate(&env);

    client.grant_access(&pet_id, &g1, &AccessLevel::Basic, &None, &0u64);
    client.grant_access(&pet_id, &g2, &AccessLevel::Basic, &None, &1u64);
    client.grant_access(&pet_id, &g3, &AccessLevel::Full, &None, &2u64);

    assert_index_consistent(&env, &contract_id, pet_id);
    assert_eq!(grant_count(&env, &contract_id, pet_id), 3);
}

/// Re-granting to an existing grantee must not add a new index slot
/// (the count/index only grow on brand-new grantees).
#[test]
fn test_regrant_does_not_duplicate_index_entry() {
    let env = Env::default();
    let (client, contract_id) = setup(&env);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let grantee = Address::generate(&env);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None, &0u64);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None, &1u64);

    assert_eq!(grant_count(&env, &contract_id, pet_id), 1);
    assert_index_consistent(&env, &contract_id, pet_id);
}

/// A revoked grant's index/count bookkeeping is untouched by
/// `revoke_access` (compaction is a separate, explicit operation), but
/// `check_access` must never authorize a revoked grantee.
#[test]
fn test_revoked_grant_cannot_authorize_access() {
    let env = Env::default();
    let (client, _contract_id) = setup(&env);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let grantee = Address::generate(&env);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None, &0u64);
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, grantee.clone()),
        AccessLevel::Full
    );

    client.revoke_access(&pet_id, &grantee);
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, grantee),
        AccessLevel::None
    );
}

/// A grant whose `expires_at` has passed cannot authorize access, even
/// though it is still `is_active` and still present in the index.
#[test]
fn test_expired_grant_cannot_authorize_access() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, _contract_id) = setup(&env);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let grantee = Address::generate(&env);
    client.grant_access(
        &pet_id,
        &grantee,
        &AccessLevel::Full,
        &Some(1_000_500u64),
        &0u64,
    );
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, grantee.clone()),
        AccessLevel::Full
    );

    env.ledger().with_mut(|l| l.timestamp = 1_000_500);
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, grantee),
        AccessLevel::None
    );
}

/// After `compact_storage`, a stale (revoked) grant's record and index
/// slot are fully removed, and the remaining index stays contiguous and
/// consistent with the shrunk count.
#[test]
fn test_compact_storage_removes_stale_index_entries() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, contract_id) = setup(&env);
    let owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    let g3 = Address::generate(&env);

    client.grant_access(&pet_id, &g1, &AccessLevel::Basic, &None, &0u64);
    client.grant_access(&pet_id, &g2, &AccessLevel::Basic, &None, &1u64);
    client.grant_access(&pet_id, &g3, &AccessLevel::Basic, &None, &2u64);

    // Revoke the middle grant; it stays in the index/count until
    // compaction runs.
    client.revoke_access(&pet_id, &g2);
    assert_eq!(grant_count(&env, &contract_id, pet_id), 3);

    client.compact_storage(&pet_id, &owner);

    assert_eq!(grant_count(&env, &contract_id, pet_id), 2);
    assert_index_consistent(&env, &contract_id, pet_id);

    // The revoked grantee's record must be gone entirely, not just
    // unreachable via the index.
    env.as_contract(&contract_id, || {
        let still_present: Option<AccessGrant> = env
            .storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, g2.clone())));
        assert!(
            still_present.is_none(),
            "compact_storage must remove the stale grant record"
        );
    });

    // The still-active grantees remain authorized.
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, g1),
        AccessLevel::Basic
    );
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, g3),
        AccessLevel::Basic
    );
    assert_eq!(
        PetChainContract::check_access(env.clone(), pet_id, g2),
        AccessLevel::None
    );
}
