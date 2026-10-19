use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

#[test]
fn test_insurance_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    // Attempting to add insurance for non-existent pet should return false
    let expiry = env.ledger().timestamp() + 31536000;
    let result = client.add_insurance_policy(
        &1,
        &String::from_str(&env, "POL-123"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &expiry,
    );
    assert_eq!(result, false);

    // Register a pet
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Labrador"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    // Add insurance
    let expiry2 = env.ledger().timestamp() + 31536000;
    let result = client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-123"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &expiry2,
    );
    assert_eq!(result, true);

    // Get insurance
    let policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(policy.policy_id, String::from_str(&env, "POL-123"));
    assert_eq!(policy.provider, String::from_str(&env, "PetProtect"));
    assert_eq!(
        policy.coverage_type,
        String::from_str(&env, "Comprehensive")
    );
    assert_eq!(policy.premium, 1000);
    assert_eq!(policy.coverage_limit, 50000);
    assert_eq!(policy.active, true);

    // Update insurance status
    let update_result =
        client.update_insurance_status(&owner, &pet_id, &String::from_str(&env, "POL-123"), &false);
    assert_eq!(update_result, true);

    let updated_policy = client.get_pet_insurance(&pet_id).unwrap();
    assert_eq!(updated_policy.active, false);
}

#[test]
fn test_is_insurance_active_no_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // No policy registered — should return false
    assert_eq!(client.is_insurance_active(&99), false);
}

#[test]
fn test_is_insurance_active_with_active_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-10"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Siamese"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    // Policy expires 1 year from now
    let expiry = env.ledger().timestamp() + 31_536_000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-ACTIVE"),
        &String::from_str(&env, "SafePaws"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &expiry,
    );

    assert_eq!(client.is_insurance_active(&pet_id), true);
}

#[test]
fn test_is_insurance_active_with_expired_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2019-03-15"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Poodle"),
        &8,
        &None,
        &PrivacyLevel::Public,
    );

    // Set ledger time to 1000 so we can set an expiry in the past
    env.ledger().set_timestamp(1000);
    // Policy expired at timestamp 500 (before current time of 1000)
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-EXPIRED"),
        &String::from_str(&env, "OldCover"),
        &String::from_str(&env, "Basic"),
        &300,
        &10000,
        &500,
    );

    assert_eq!(client.is_insurance_active(&pet_id), false);
}

#[test]
fn test_is_insurance_active_with_inactive_flag() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2022-07-20"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &String::from_str(&env, "Retriever"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31_536_000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-DEACTIVATED"),
        &String::from_str(&env, "PetShield"),
        &String::from_str(&env, "Comprehensive"),
        &800,
        &40000,
        &expiry,
    );

    // Deactivate the policy
    client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-DEACTIVATED"),
        &false,
    );

    assert_eq!(client.is_insurance_active(&pet_id), false);
}

// ---------------------------------------------------------------------------
// Enrollment workflow tests (#1470)
// ---------------------------------------------------------------------------

fn setup_enrollment(
    env: &Env,
    client: &PetChainContractClient,
) -> (Address, u64, Address) {
    let owner = Address::generate(env);
    let underwriter = Address::generate(env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Rex"),
        &String::from_str(env, "2020-06-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Beagle"),
        &12,
        &None,
        &PrivacyLevel::Public,
    );
    (owner, pet_id, underwriter)
}

#[test]
fn test_enrollment_draft_to_accepted() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let policy_version = 1u32;
    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-1"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &policy_version,
    );

    // Draft state
    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(enrollment.status, crate::InsuranceEnrollmentStatus::Draft);
    assert_eq!(enrollment.policy_version, policy_version);

    // Submit
    client.submit_insurance_enrollment(&owner, &enrollment_id);
    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::Submitted
    );

    // Under review
    client.review_insurance_enrollment(&underwriter, &enrollment_id);
    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::UnderReview
    );

    // Accept
    client.decide_insurance_enrollment(&underwriter, &enrollment_id, &true);
    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::Accepted
    );
    assert_eq!(enrollment.policy_version, policy_version);
}

#[test]
fn test_enrollment_draft_to_declined() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-2"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );

    client.submit_insurance_enrollment(&owner, &enrollment_id);
    client.review_insurance_enrollment(&underwriter, &enrollment_id);
    client.decide_insurance_enrollment(&underwriter, &enrollment_id, &false);

    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::Declined
    );
}

#[test]
fn test_enrollment_unauthorized_submit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (_owner, pet_id, _underwriter) = setup_enrollment(&env, &client);

    let attacker = Address::generate(&env);
    let enrollment_id = client.create_insurance_enrollment(
        &attacker,
        &pet_id,
        &String::from_str(&env, "POL-ENR-3"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );

    // Attacker is not the owner of the enrollment
    let stranger = Address::generate(&env);
    let result = client.try_submit_insurance_enrollment(&stranger, &enrollment_id);
    assert!(result.is_err());

    // State unchanged
    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(enrollment.status, crate::InsuranceEnrollmentStatus::Draft);
}

#[test]
fn test_enrollment_unauthorized_decision() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-4"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );
    client.submit_insurance_enrollment(&owner, &enrollment_id);
    client.review_insurance_enrollment(&underwriter, &enrollment_id);

    // Owner cannot decide their own enrollment
    let result = client.try_decide_insurance_enrollment(&owner, &enrollment_id, &true);
    assert!(result.is_err());

    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::UnderReview
    );
}

#[test]
fn test_enrollment_invalid_transition() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-5"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );

    // Cannot decide a draft enrollment
    let result = client.try_decide_insurance_enrollment(&underwriter, &enrollment_id, &true);
    assert!(result.is_err());

    // Cannot review a draft enrollment
    let result = client.try_review_insurance_enrollment(&underwriter, &enrollment_id);
    assert!(result.is_err());

    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(enrollment.status, crate::InsuranceEnrollmentStatus::Draft);
}

#[test]
fn test_enrollment_replay_decision_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-6"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );
    client.submit_insurance_enrollment(&owner, &enrollment_id);
    client.review_insurance_enrollment(&underwriter, &enrollment_id);
    client.decide_insurance_enrollment(&underwriter, &enrollment_id, &true);

    // Replay decision should fail
    let result = client.try_decide_insurance_enrollment(&underwriter, &enrollment_id, &false);
    assert!(result.is_err());

    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(
        enrollment.status,
        crate::InsuranceEnrollmentStatus::Accepted
    );
}

#[test]
fn test_enrollment_boundary_premium_and_limit() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, _underwriter) = setup_enrollment(&env, &client);

    // Zero premium should be rejected
    let result = client.try_create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-7"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &0,
        &20000,
        &1u32,
    );
    assert!(result.is_err());

    // Zero coverage limit should be rejected
    let result = client.try_create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-8"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &0,
        &1u32,
    );
    assert!(result.is_err());

    // Zero policy version should be rejected
    let result = client.try_create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-9"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &0u32,
    );
    assert!(result.is_err());
}

#[test]
fn test_enrollment_policy_version_binding() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-ENR-10"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &3u32,
    );
    client.submit_insurance_enrollment(&owner, &enrollment_id);
    client.review_insurance_enrollment(&underwriter, &enrollment_id);
    client.decide_insurance_enrollment(&underwriter, &enrollment_id, &true);

    let enrollment = client.get_insurance_enrollment(&enrollment_id).unwrap();
    assert_eq!(enrollment.policy_version, 3u32);
}

// ---------------------------------------------------------------------------
// Coverage snapshot tests (#1472)
// ---------------------------------------------------------------------------

#[test]
fn test_coverage_snapshot_immutable_after_policy_change() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-SNAP-1"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Comprehensive"),
        &1000,
        &50000,
        &1u32,
    );

    // Snapshot should be captured at enrollment creation
    let snapshot = client.get_coverage_snapshot(&enrollment_id).unwrap();
    assert_eq!(snapshot.coverage_limit, 50000);
    assert_eq!(
        snapshot.coverage_type,
        String::from_str(&env, "Comprehensive")
    );
    assert_eq!(snapshot.policy_version, 1u32);

    // Accept the enrollment
    client.submit_insurance_enrollment(&owner, &enrollment_id);
    client.review_insurance_enrollment(&underwriter, &enrollment_id);
    client.decide_insurance_enrollment(&underwriter, &enrollment_id, &true);

    // Snapshot must remain unchanged
    let snapshot_after = client.get_coverage_snapshot(&enrollment_id).unwrap();
    assert_eq!(snapshot_after.coverage_limit, snapshot.coverage_limit);
    assert_eq!(snapshot_after.coverage_type, snapshot.coverage_type);
    assert_eq!(snapshot_after.policy_version, snapshot.policy_version);
}

#[test]
fn test_coverage_snapshot_missing_for_unknown_enrollment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    assert!(client.get_coverage_snapshot(&9999u64).is_none());
}

#[test]
fn test_coverage_snapshot_exclusions_bounded() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let (owner, pet_id, _underwriter) = setup_enrollment(&env, &client);

    let enrollment_id = client.create_insurance_enrollment(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-SNAP-2"),
        &String::from_str(&env, "PetProtect"),
        &String::from_str(&env, "Basic"),
        &500,
        &20000,
        &1u32,
    );

    let snapshot = client.get_coverage_snapshot(&enrollment_id).unwrap();
    // Exclusions list is bounded
    assert!(snapshot.exclusions.len() <= 32);
}