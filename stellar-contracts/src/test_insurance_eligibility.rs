//! Tests for the derived `insurance_eligible` field on `PetHealthSummary`
//! (Issue #769).
//!
//! `insurance_eligible` is true iff the pet currently has an active,
//! non-expired insurance policy AND a non-expired latest vaccination at the
//! ledger timestamp.

use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

// A realistic ledger timestamp after the pet's 2020 birthday so registration
// validation accepts it (2023-11-14T22:13:20Z).
const NOW: u64 = 1_700_000_000;

/// Register admin, a verified vet, and a public pet; pin the ledger to `NOW`.
fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env.ledger().set_timestamp(NOW);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET12345"),
        &String::from_str(&env, "General Practice"),
    );
    client.init_admin(&admin);
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, owner, vet, pet_id)
}

/// Add a vaccination for `pet_id` whose entry expires at `expires_at`.
fn add_vaccination(
    env: &Env,
    client: &PetChainContractClient<'static>,
    pet_id: u64,
    vet: &Address,
    expires_at: u64,
) {
    client.add_vaccination(
        &pet_id,
        vet,
        &VaccineType::Rabies,
        &String::from_str(env, "Rabies Vaccine"),
        &(NOW - 100),  // administered_at
        &expires_at,   // next_due_date
        &expires_at,   // expires_at
        &String::from_str(env, "BATCH-1"),
    );
}

/// Seed an active insurance policy for `pet_id` expiring at `expiry_date`.
/// There is no public writer for policies, so write storage directly.
fn seed_insurance(env: &Env, contract_id: &Address, pet_id: u64, expiry_date: u64) {
    let policy = InsurancePolicy {
        policy_id: String::from_str(env, "POL-1"),
        provider: String::from_str(env, "Acme Pet Insurance"),
        coverage_type: String::from_str(env, "Full"),
        tier: PremiumTier::Standard,
        premium: 100,
        coverage_limit: 10_000,
        start_date: NOW - 1000,
        expiry_date,
        active: true,
    };
    env.as_contract(contract_id, || {
        let store = env.storage().instance();
        store.set(&InsuranceKey::PetPolicyCount(pet_id), &1u64);
        store.set(&InsuranceKey::PetPolicyIndex((pet_id, 1u64)), &policy);
    });
}

/// Seed the immutable enrollment snapshot for `pet_id` (Issue #1472).
/// Eligibility must be derived from this snapshot, not from mutable policy
/// data, so later policy edits cannot silently alter an existing contract.
fn seed_snapshot(
    env: &Env,
    contract_id: &Address,
    pet_id: u64,
    expiry_date: u64,
    coverage: &[&str],
    exclusions: &[&str],
) {
    let mut coverage_vec = Vec::new(env);
    for item in coverage {
        coverage_vec.push_back(String::from_str(env, item));
    }
    let mut exclusion_vec = Vec::new(env);
    for item in exclusions {
        exclusion_vec.push_back(String::from_str(env, item));
    }
    let snapshot = PolicyCoverageSnapshot {
        policy_id: String::from_str(env, "POL-1"),
        coverage_type: String::from_str(env, "Full"),
        tier: PremiumTier::Standard,
        coverage_limit: 10_000,
        start_date: NOW - 1000,
        expiry_date,
        coverage: coverage_vec,
        exclusions: exclusion_vec,
        captured_at: NOW,
    };
    env.as_contract(contract_id, || {
        env.storage()
            .persistent()
            .set(&InsuranceKey::PolicyCoverageSnapshot(pet_id), &snapshot);
    });
}

/// Overwrite the mutable policy record for `pet_id` to simulate a later
/// provider-side policy change (e.g. reduced coverage / new exclusions).
fn mutate_policy(
    env: &Env,
    contract_id: &Address,
    pet_id: u64,
    expiry_date: u64,
    coverage_type: &str,
    coverage_limit: i128,
    active: bool,
) {
    let policy = InsurancePolicy {
        policy_id: String::from_str(env, "POL-1"),
        provider: String::from_str(env, "Acme Pet Insurance"),
        coverage_type: String::from_str(env, coverage_type),
        tier: PremiumTier::Basic,
        premium: 50,
        coverage_limit,
        start_date: NOW - 1000,
        expiry_date,
        active,
    };
    env.as_contract(contract_id, || {
        let store = env.storage().instance();
        store.set(&InsuranceKey::PetPolicyCount(pet_id), &1u64);
        store.set(&InsuranceKey::PetPolicyIndex((pet_id, 1u64)), &policy);
    });
}

/// Active non-expired insurance snapshot + non-expired vaccination => eligible.
#[test]
fn test_insurance_eligible_when_both_valid() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    seed_snapshot(
        &env,
        &contract_id,
        pet_id,
        NOW + 10_000,
        &["Accident", "Illness"],
        &[],
    );

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(summary.insurance_eligible);
}

/// Expired vaccination (with valid snapshot) => not eligible.
#[test]
fn test_not_eligible_when_vaccination_expired() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW - 1);
    seed_snapshot(
        &env,
        &contract_id,
        pet_id,
        NOW + 10_000,
        &["Accident", "Illness"],
        &[],
    );

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(!summary.insurance_eligible);
}

/// No enrollment snapshot (with valid vaccination) => not eligible, even if a
/// mutable policy record exists.
#[test]
fn test_not_eligible_when_no_snapshot() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    // A mutable policy exists but no immutable snapshot was captured.
    seed_insurance(&env, &contract_id, pet_id, NOW + 10_000);

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(!summary.insurance_eligible);
    assert!(summary.active_insurance_policy_id.is_none());
}

/// Expired snapshot (with valid vaccination) => not eligible.
#[test]
fn test_not_eligible_when_snapshot_expired() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    seed_snapshot(&env, &contract_id, pet_id, NOW - 1, &["Accident"], &[]);

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(!summary.insurance_eligible);
}

/// Boundary: snapshot expiring exactly at the ledger timestamp is expired.
#[test]
fn test_not_eligible_when_snapshot_expires_now() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    seed_snapshot(&env, &contract_id, pet_id, NOW, &["Accident"], &[]);

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(!summary.insurance_eligible);
}

/// Later policy edits (reduced coverage, new exclusions, deactivation) must
/// not change the coverage/exclusions captured in the enrollment snapshot.
#[test]
fn test_policy_edits_do_not_change_snapshot() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    seed_snapshot(
        &env,
        &contract_id,
        pet_id,
        NOW + 10_000,
        &["Accident", "Illness"],
        &["Cosmetic"],
    );

    let before = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(before.insurance_eligible);

    // Provider later narrows the policy and adds exclusions.
    mutate_policy(
        &env,
        &contract_id,
        pet_id,
        NOW + 10_000,
        "AccidentOnly",
        1_000,
        true,
    );

    let after = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(after.insurance_eligible);
    assert_eq!(after.active_insurance_policy_id, before.active_insurance_policy_id);

    // The snapshot itself is unchanged.
    let snapshot = env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .get::<InsuranceKey, PolicyCoverageSnapshot>(&InsuranceKey::PolicyCoverageSnapshot(
                pet_id,
            ))
            .unwrap()
    });
    assert_eq!(snapshot.coverage.len(), 2);
    assert_eq!(snapshot.exclusions.len(), 1);
    assert_eq!(
        snapshot.coverage.get(0).unwrap(),
        String::from_str(&env, "Accident")
    );
    assert_eq!(
        snapshot.exclusions.get(0).unwrap(),
        String::from_str(&env, "Cosmetic")
    );
    assert_eq!(snapshot.coverage_limit, 10_000);
    assert_eq!(snapshot.expiry_date, NOW + 10_000);
}

/// Deactivating the mutable policy after enrollment must not revoke the
/// snapshot-backed eligibility of an existing contract.
#[test]
fn test_policy_deactivation_does_not_affect_snapshot() {
    let (env, client, owner, vet, pet_id) = setup();
    let contract_id = client.address.clone();

    add_vaccination(&env, &client, pet_id, &vet, NOW + 10_000);
    seed_snapshot(&env, &contract_id, pet_id, NOW + 10_000, &["Accident"], &[]);

    mutate_policy(&env, &contract_id, pet_id, NOW + 10_000, "Full", 10_000, false);

    let summary = client.get_pet_health_summary(&pet_id, &owner).unwrap();
    assert!(summary.insurance_eligible);
}
