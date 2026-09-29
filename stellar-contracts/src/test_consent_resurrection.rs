//! Tests for issue #1202 — prevent consent resurrection after revocation.
//!
//! Threat model: an adversary replays a previously-valid `grant_consent`
//! transaction after the owner has revoked consent, silently restoring
//! access. The revocation-generation counter (`ConsentKey::ConsentRevocationGen`)
//! makes every revocation a permanent on-chain fact: a replayed grant always
//! gets a fresh monotonic ID (generation 0); the old revoked ID stays at
//! generation ≥ 1 forever.

use crate::{
    ConsentKey, ConsentScope, ConsentType, Gender, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String, Vec,
};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, u64) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let owner = Address::generate(env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Sparky"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Husky"),
        &String::from_str(env, "White"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    (client, owner, pet_id)
}

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env
}

// ── Section 0: "before" snapshot — demonstrates the resurrection gap ─────────

#[test]
fn test_revoked_consent_not_resurrected_by_new_grant() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);

    let id1 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    assert!(client.is_consent_active(&id1));

    client.revoke_consent(&id1, &owner);
    assert!(!client.is_consent_active(&id1));

    // Re-grant: must produce a NEW id, not resurrect id1.
    let id2 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    assert!(
        !client.is_consent_active(&id1),
        "revoked consent must stay inactive"
    );
    assert!(client.is_consent_active(&id2));
    assert_ne!(id1, id2);
}

// ── Section 1: success cases ─────────────────────────────────────────────────

#[test]
fn test_grant_revoke_regrant_cycle() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);

    let id1 = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee);
    client.revoke_consent(&id1, &owner);
    let id2 = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee);
    assert!(client.is_consent_active(&id2));
    assert!(!client.is_consent_active(&id1));
    assert_ne!(id1, id2);
}

#[test]
fn test_cascade_revoke_parent_and_child() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    let sub_grantee = Address::generate(&env);

    let root = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    let sub = client.grant_consent_with_parent(
        &pet_id,
        &grantee,
        &ConsentType::Research,
        &sub_grantee,
        &ConsentScope::ReadMedical,
        &Some(root),
    );
    let count = client.revoke_consent_cascade(&pet_id, &root, &owner);
    assert_eq!(count, 2);
    assert!(!client.is_consent_active(&root));
    assert!(!client.is_consent_active(&sub));
}

// ── Section 2: unauthorized cases ────────────────────────────────────────────

#[test]
#[should_panic]
fn test_non_owner_cannot_grant() {
    let env = make_env();
    let (client, _owner, pet_id) = setup(&env);
    let attacker = Address::generate(&env);
    let grantee = Address::generate(&env);
    client.grant_consent(&pet_id, &attacker, &ConsentType::Research, &grantee);
}

#[test]
#[should_panic]
fn test_non_owner_cannot_revoke() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let attacker = Address::generate(&env);
    let grantee = Address::generate(&env);
    let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    client.revoke_consent(&id, &attacker);
}

// ── Section 3: invalid-input cases ───────────────────────────────────────────

#[test]
#[should_panic]
fn test_extend_revoked_consent_rejected() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    env.ledger().set_timestamp(100);
    let id = client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &Some(200u64),
    );
    client.revoke_consent(&id, &owner);
    // Must panic — extending a revoked consent is resurrection.
    client.extend_consent(&pet_id, &id, &owner, &500u64);
}

// ── Section 4: boundary cases ────────────────────────────────────────────────

#[test]
fn test_new_consent_generation_is_zero() {
    let env = make_env();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bolt"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Lab"),
        &String::from_str(&env, "Yellow"),
        &28u32,
        &None,
        &PrivacyLevel::Public,
    );
    let grantee = Address::generate(&env);
    let id = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee);
    env.as_contract(&contract_id, || {
        let gen: u32 = env
            .storage()
            .instance()
            .get::<ConsentKey, u32>(&ConsentKey::ConsentRevocationGen(id))
            .unwrap_or(0);
        assert_eq!(gen, 0);
    });
}

#[test]
fn test_revocation_bumps_generation_to_one() {
    let env = make_env();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "Grey"),
        &15u32,
        &None,
        &PrivacyLevel::Public,
    );
    let grantee = Address::generate(&env);
    let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    client.revoke_consent(&id, &owner);
    env.as_contract(&contract_id, || {
        let gen: u32 = env
            .storage()
            .instance()
            .get::<ConsentKey, u32>(&ConsentKey::ConsentRevocationGen(id))
            .unwrap_or(0);
        assert_eq!(gen, 1);
    });
}

#[test]
fn test_double_revoke_is_idempotent() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    client.revoke_consent(&id, &owner);
    client.revoke_consent(&id, &owner); // second call is no-op
    assert!(!client.is_consent_active(&id));
}

// ── Section 5: overflow / resource-limit cases ───────────────────────────────

#[test]
#[should_panic]
fn test_consent_id_counter_overflow_panics() {
    let env = make_env();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Spike"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Dalmatian"),
        &String::from_str(&env, "Spotted"),
        &22u32,
        &None,
        &PrivacyLevel::Public,
    );
    let grantee = Address::generate(&env);
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&ConsentKey::ConsentCount, &u64::MAX);
    });
    client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
}

#[test]
fn test_many_cycles_stay_bounded() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    for _ in 0..200u32 {
        let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
        client.revoke_consent(&id, &owner);
    }
    assert!(client.get_consent_count(&pet_id) <= 50);
}

// ── Section 6: replay / idempotency cases ────────────────────────────────────

#[test]
fn test_replay_grant_after_revoke_does_not_resurrect() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);

    let id_orig = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    client.revoke_consent(&id_orig, &owner);
    let id_replay = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);

    assert!(
        !client.is_consent_active(&id_orig),
        "original must stay revoked"
    );
    assert!(client.is_consent_active(&id_replay));
    assert_ne!(id_orig, id_replay);
}

#[test]
fn test_multiple_generations_none_resurrected() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    let mut revoked = Vec::new(&env);
    for _ in 0..3u32 {
        let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
        client.revoke_consent(&id, &owner);
        revoked.push_back(id);
    }
    let new_id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    assert!(client.is_consent_active(&new_id));
    for i in 0..revoked.len() {
        assert!(!client.is_consent_active(&revoked.get(i).unwrap()));
    }
}

#[test]
#[should_panic]
fn test_extend_after_revoke_is_rejected() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    env.ledger().set_timestamp(100);
    let id = client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &Some(500u64),
    );
    client.revoke_consent(&id, &owner);
    client.extend_consent(&pet_id, &id, &owner, &1000u64);
}

#[test]
fn test_double_grant_creates_two_records() {
    let env = make_env();
    let (client, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    let id1 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    let id2 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    assert_ne!(id1, id2);
    assert!(client.is_consent_active(&id1));
    assert!(client.is_consent_active(&id2));
    assert_eq!(client.get_consent_count(&pet_id), 2);
}
