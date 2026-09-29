// ============================================================
// MEDICAL EVENT TIMESTAMP VALIDATION TESTS (Issue #1174)
//
// `add_vaccination` accepts caller-supplied `administered_at`,
// `next_due_date`, and `expires_at` values. Without validation, an
// arbitrary far-future (or otherwise nonsensical) timestamp can corrupt
// reminder scheduling and medical history ordering. These tests document
// the allowed domain: `administered_at` may not be further in the future
// than `MAX_EVENT_FUTURE_SKEW` relative to ledger time, and follow-up
// dates (`next_due_date` / `expires_at`, when non-zero) must fall between
// `administered_at` and `administered_at + MAX_EVENT_HORIZON`.
// ============================================================

use crate::{
    ContractError, PetChainContract, PetChainContractClient, PrivacyLevel, Species, VaccineType,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup(env: &Env) -> (PetChainContractClient<'static>, Address, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let vet = Address::generate(env);
    let owner = Address::generate(env);
    let mut admins: Vec<Address> = Vec::new(env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1u32);

    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Test"),
        &String::from_str(env, "LIC-TS-1"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Timestamp"),
        &String::from_str(env, "2020-01-01"),
        &crate::Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Mixed"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );

    (client, vet, owner, pet_id)
}

/// A recent `administered_at` with a sane follow-up schedule succeeds.
#[test]
fn test_recent_administered_at_accepted() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);
    let (client, vet, _owner, pet_id) = setup(&env);

    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &(1_000_000 - 100), // administered_at: just in the past
        &(1_000_000 + 100), // next_due_date
        &0u64,               // expires_at sentinel -> defaults to next_due_date
        &String::from_str(&env, "BATCH-OK"),
    );
    assert!(vax_id > 0);
}

/// `administered_at` exactly at the allowed future-skew boundary succeeds.
#[test]
fn test_administered_at_boundary_accepted() {
    let env = Env::default();
    let now = 1_000_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    let (client, vet, _owner, pet_id) = setup(&env);

    let max_skew = crate::MAX_EVENT_FUTURE_SKEW;
    let vax_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &now.saturating_add(max_skew), // exactly at the boundary: allowed
        &now.saturating_add(max_skew), // next_due_date == administered_at: allowed
        &0u64,
        &String::from_str(&env, "BATCH-BOUND"),
    );
    assert!(vax_id > 0);
}

/// `administered_at` one second past the allowed future-skew boundary is
/// rejected with `InvalidTimestamp`.
#[test]
fn test_administered_at_beyond_skew_rejected() {
    let env = Env::default();
    let now = 1_000_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    let (client, vet, _owner, pet_id) = setup(&env);

    let max_skew = crate::MAX_EVENT_FUTURE_SKEW;
    let result = client.try_add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &now.saturating_add(max_skew).saturating_add(1),
        &now.saturating_add(max_skew).saturating_add(1),
        &0u64,
        &String::from_str(&env, "BATCH-BAD"),
    );
    assert_eq!(
        result,
        Err(Ok(soroban_sdk::Error::from_contract_error(
            ContractError::InvalidTimestamp as u32
        )))
    );
}

/// A `next_due_date` before `administered_at` is rejected: a follow-up
/// cannot be due before the event that scheduled it.
#[test]
fn test_next_due_date_before_administered_at_rejected() {
    let env = Env::default();
    let now = 1_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    let (client, vet, _owner, pet_id) = setup(&env);

    let result = client.try_add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &now,
        &(now - 1), // next_due_date before administered_at
        &0u64,
        &String::from_str(&env, "BATCH-ORDER"),
    );
    assert_eq!(
        result,
        Err(Ok(soroban_sdk::Error::from_contract_error(
            ContractError::InvalidTimestamp as u32
        )))
    );
}

/// An `expires_at` far beyond the allowed horizon past `administered_at`
/// is rejected.
#[test]
fn test_expires_at_beyond_horizon_rejected() {
    let env = Env::default();
    let now = 1_000_000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    let (client, vet, _owner, pet_id) = setup(&env);

    let too_far = now.saturating_add(crate::MAX_EVENT_HORIZON).saturating_add(1);
    let result = client.try_add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabivax"),
        &now,
        &now,
        &too_far,
        &String::from_str(&env, "BATCH-HORIZON"),
    );
    assert_eq!(
        result,
        Err(Ok(soroban_sdk::Error::from_contract_error(
            ContractError::InvalidTimestamp as u32
        )))
    );
}
