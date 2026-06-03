// Regression tests for bugs discovered during fuzz analysis.
// Each test documents the exact input that triggered the bug and asserts
// the fixed behaviour.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (PetChainContractClient, Address, Address, u64) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let owner = Address::generate(env);
    client.init_admin(&admin);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Rex"),
        &String::from_str(env, "2020-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Husky"),
        &String::from_str(env, "White"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );
    (client, admin, owner, pet_id)
}

fn setup_with_vet(env: &Env) -> (PetChainContractClient, Address, Address, Address, u64) {
    let (client, admin, owner, pet_id) = setup(env);
    let vet = Address::generate(env);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Fuzz"),
        &String::from_str(env, "LIC-FUZZ-001"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);
    (client, admin, owner, vet, pet_id)
}

// ── Bug 1: add_vet_review – missing comment length guard ─────────────────────
// Before fix: a comment longer than MAX_REVIEW_COMMENT_LEN (500) was stored
// without error, allowing unbounded on-chain data growth.
// After fix: panics with ContractError::CommentTooLong.

#[test]
fn regression_review_comment_exactly_at_limit_accepted() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, owner, _pet_id) = setup(&env);
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Limit"),
        &String::from_str(&env, "LIC-R1-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);
    let comment = String::from_bytes(&env, &[b'x'; 500]);
    let id = client.add_vet_review(&owner, &vet, &5, &comment);
    assert!(id > 0);
}

#[test]
fn regression_review_comment_over_limit_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, owner, _pet_id) = setup(&env);
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Over"),
        &String::from_str(&env, "LIC-R1-002"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);
    let comment = String::from_bytes(&env, &[b'x'; 501]);
    let result = client.try_add_vet_review(&owner, &vet, &5, &comment);
    assert!(result.is_err());
}

// ── Bug 2: get_upcoming_vaccinations – arithmetic overflow ───────────────────
// Before fix: `current_time + (days_threshold * 86400)` could overflow u64
// when days_threshold = u64::MAX, causing a panic in release builds with
// overflow-checks = true.
// After fix: saturating arithmetic is used; the function returns normally.

#[test]
fn regression_upcoming_vaccinations_max_days_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    // u64::MAX days – must not panic
    let result = client.get_upcoming_vaccinations(&pet_id, &u64::MAX);
    assert_eq!(result.len(), 0);
}

#[test]
fn regression_upcoming_vaccinations_large_days_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    let result = client.get_upcoming_vaccinations(&pet_id, &(u64::MAX / 86400 + 1));
    assert_eq!(result.len(), 0);
}

// ── Bug 3: get_consent_history_page – page * size overflow cast to u32 ───────
// Before fix: `(page * size as u64) as u32` silently truncated when the
// product exceeded u32::MAX, returning wrong page data.
// After fix: returns an empty Vec when the computed start index overflows u32.

#[test]
fn regression_consent_page_overflow_returns_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, pet_id) = setup(&env);
    // page = u64::MAX, page_size = 2 → product overflows u32
    let result = client.get_consent_history_page(&pet_id, &u64::MAX, &2u32);
    assert_eq!(result.len(), 0);
}

#[test]
fn regression_consent_page_zero_page_works() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, owner, pet_id) = setup(&env);
    let grantee = Address::generate(&env);
    client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee);
    let result = client.get_consent_history_page(&pet_id, &0u64, &10u32);
    assert_eq!(result.len(), 1);
}

// ── Bug 4: get_medications – offset + 1 could overflow u64 ──────────────────
// Before fix: `offset + 1` and `offset + limit as u64` were unchecked
// additions that panic on overflow in debug / overflow-checks builds.
// After fix: saturating_add is used; large offsets return an empty Vec.

#[test]
fn regression_get_medications_max_offset_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    let result = client.get_medications(&pet_id, &u64::MAX, &10u32);
    assert_eq!(result.len(), 0);
}

#[test]
fn regression_get_medications_max_limit_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    let result = client.get_medications(&pet_id, &0u64, &u32::MAX);
    assert_eq!(result.len(), 0);
}

// ── Bug 5: get_treatment_history – same overflow class as get_medications ────

#[test]
fn regression_get_treatment_history_max_offset_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    let result = client.get_treatment_history(&pet_id, &u64::MAX, &10u32);
    assert_eq!(result.len(), 0);
}

#[test]
fn regression_get_treatment_history_max_limit_no_panic() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _admin, _owner, _vet, pet_id) = setup_with_vet(&env);
    let result = client.get_treatment_history(&pet_id, &0u64, &u32::MAX);
    assert_eq!(result.len(), 0);
}
