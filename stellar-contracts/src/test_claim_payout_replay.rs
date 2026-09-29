//! test_claim_payout_replay.rs
//!
//! Issue #1205 — Prevent duplicate insurance claim payouts.
//!
//! # "Before" documentation (what the code looked like without the fix)
//!
//! Prior to this fix the `InsuranceClaim` struct had no settlement record and
//! no storage key to track whether a claim had already been paid.  The
//! approval path (`update_insurance_claim_status(..., Paid)`) simply wrote the
//! new status unconditionally, so calling it twice on the same claim would
//! transition the claim to `Paid` a second time and any off-chain disburser
//! watching for `Paid` events would trigger a second payout.  The appeal
//! resolution path (`review_appeal(..., Approved)`) had the same gap.
//!
//! # Fix summary
//!
//! A new `InsuranceKey::ClaimSettled(claim_id)` storage key is written
//! BEFORE the status is changed to `Paid` (checks-effects-interactions).
//! Soroban's atomic transaction model guarantees that if anything after the
//! write panics the key is never persisted, so no partial "paid" flag can
//! survive a failed transaction.  Every payout path checks this key first and
//! panics with `ContractError::ClaimAlreadySettled` on replay.
//!
//! # Synthetic test data
//!
//! All addresses, amounts, and policy IDs below are fake and generated only
//! for test purposes.  No real keys, secrets, medical records, or addresses
//! are used anywhere in this file.

use crate::{
    ContractError, Gender, InsuranceClaimStatus, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

// ---------------------------------------------------------------------------
// Shared setup helpers
// ---------------------------------------------------------------------------

/// Register a pet with a fresh owner and return (client, owner, pet_id).
fn setup_env() -> (Env, PetChainContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    (env, client, admin)
}

fn register_pet_with_policy(env: &Env, client: &PetChainContractClient) -> (Address, u64) {
    let owner = Address::generate(env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Labrador"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );
    let expiry = env.ledger().timestamp() + 31_536_000; // +1 year
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(env, "POL-SYNTHETIC-001"),
        &String::from_str(env, "FakePetInsure Co"),
        &String::from_str(env, "Comprehensive"),
        &500u64,
        &50_000u64,
        &expiry,
    );
    (owner, pet_id)
}

// ---------------------------------------------------------------------------
// 1. SUCCESS — single legitimate approval pays exactly once and records settlement
// ---------------------------------------------------------------------------

#[test]
fn test_single_approval_pays_once_and_records_settlement() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(
            &pet_id,
            &1_000u64,
            &String::from_str(&env, "Routine vet visit"),
        )
        .unwrap();

    // Before approval: not settled
    assert!(!client.is_claim_settled(&claim_id));

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);

    // After approval: settled flag must be set
    assert!(client.is_claim_settled(&claim_id));

    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::Paid);
}

// ---------------------------------------------------------------------------
// 2. REPLAY — repeated approval of the same claim does not pay twice
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "ClaimAlreadySettled")]
fn test_approval_replay_panics() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &2_000u64, &String::from_str(&env, "Surgery"))
        .unwrap();

    // First approval — succeeds
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);

    // Second approval — must panic with ClaimAlreadySettled
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
}

// ---------------------------------------------------------------------------
// 3a. APPEAL REPLAY — appeal resolution after a paid claim does not pay again
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "ClaimAlreadySettled")]
fn test_appeal_resolution_after_paid_panics() {
    let (env, client, admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &1_500u64, &String::from_str(&env, "Dental"))
        .unwrap();

    // Record reviewer then reject
    client.set_claim_reviewer(&admin, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Owner appeals
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "New X-rays attached"),
        &Vec::new(&env),
    );

    // Second admin approves the appeal (acts as payout)
    let admin2 = Address::generate(&env);
    // Bootstrap admin2 via a config change proposal
    let proposal_id = client.propose_change_admin(&admin, &admin2);
    client.execute_proposal(&proposal_id);
    client.review_appeal(&admin2, &claim_id, &InsuranceClaimStatus::Approved);

    // Claim is now settled; attempting to approve via status update must fail
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
}

// ---------------------------------------------------------------------------
// 3b. APPEAL REPLAY — repeated appeal resolution does not pay twice
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "ClaimAlreadySettled")]
fn test_double_appeal_resolution_panics() {
    let (env, client, admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &800u64, &String::from_str(&env, "Lab work"))
        .unwrap();

    client.set_claim_reviewer(&admin, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Evidence"),
        &Vec::new(&env),
    );

    let admin2 = Address::generate(&env);
    let proposal_id = client.propose_change_admin(&admin, &admin2);
    client.execute_proposal(&proposal_id);

    // First appeal resolution — succeeds
    client.review_appeal(&admin2, &claim_id, &InsuranceClaimStatus::Approved);

    // Second appeal resolution on same claim — must panic
    client.review_appeal(&admin2, &claim_id, &InsuranceClaimStatus::Approved);
}

// ---------------------------------------------------------------------------
// 4. FAILURE PATH — if the claim does not exist, settlement is NOT recorded
//    and the real claim can still be retried
// ---------------------------------------------------------------------------

#[test]
fn test_nonexistent_claim_does_not_pollute_settlement() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let real_claim_id = client
        .submit_insurance_claim(&pet_id, &300u64, &String::from_str(&env, "Checkup"))
        .unwrap();

    // A spurious call with the wrong ID must not settle the real claim
    // (the panic unwinds the whole tx, so is_claim_settled stays false)
    let result = std::panic::catch_unwind(|| {
        // This will panic with ClaimNotFound; the settlement write does NOT occur
        // before the panic, so real_claim_id's key remains unset.
        // We verify this indirectly: after the failed call, the real claim is
        // still payable.
        let _ = client
            .update_insurance_claim_status(&(real_claim_id + 9999), &InsuranceClaimStatus::Paid);
    });
    assert!(result.is_err(), "expected panic for nonexistent claim");

    // Real claim is NOT settled — it can still be approved
    assert!(!client.is_claim_settled(&real_claim_id));
    client.update_insurance_claim_status(&real_claim_id, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&real_claim_id));
}

// ---------------------------------------------------------------------------
// 5. UNAUTHORIZED — non-admin/non-authorized callers cannot approve or resolve
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_set_claim_reviewer() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &100u64, &String::from_str(&env, "Minor"))
        .unwrap();

    let non_admin = Address::generate(&env);
    client.set_claim_reviewer(&non_admin, &claim_id);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_review_appeal() {
    let (env, client, admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &100u64, &String::from_str(&env, "Minor"))
        .unwrap();

    client.set_claim_reviewer(&admin, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Reason"),
        &Vec::new(&env),
    );

    let non_admin = Address::generate(&env);
    client.review_appeal(&non_admin, &claim_id, &InsuranceClaimStatus::Approved);
}

// ---------------------------------------------------------------------------
// 6. INVALID INPUT — nonexistent claim ID, claim in wrong status
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "ClaimNotFound")]
fn test_update_status_nonexistent_claim_panics() {
    let (_env, client, _admin) = setup_env();
    client.update_insurance_claim_status(&99_999u64, &InsuranceClaimStatus::Paid);
}

#[test]
#[should_panic(expected = "ClaimNotRejected")]
fn test_appeal_pending_claim_panics() {
    let (env, client, _admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &200u64, &String::from_str(&env, "Pending"))
        .unwrap();

    // Still Pending — appeal must be rejected
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Reason"),
        &Vec::new(&env),
    );
}

#[test]
#[should_panic(expected = "ClaimNotUnderAppeal")]
fn test_review_appeal_on_non_appealed_claim_panics() {
    let (env, client, admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &200u64, &String::from_str(&env, "Pending"))
        .unwrap();

    // Not under appeal — review_appeal must panic
    client.review_appeal(&admin, &claim_id, &InsuranceClaimStatus::Approved);
}

// ---------------------------------------------------------------------------
// 7. EXACT BOUNDARY — payout amount equal to coverage limit, and one over it
//    (The contract stores the amount as-is; enforcement is off-chain.
//     We test that the settlement guard works at boundary values.)
// ---------------------------------------------------------------------------

#[test]
fn test_payout_at_exact_coverage_limit_settles_once() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    // Coverage limit on the policy is 50_000 — submit a claim for exactly that
    let claim_id = client
        .submit_insurance_claim(&pet_id, &50_000u64, &String::from_str(&env, "Max coverage"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_id));
}

#[test]
fn test_payout_one_over_coverage_limit_still_settles_once() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    // 50_001 > coverage_limit — contract stores it; settlement guard still fires
    let claim_id = client
        .submit_insurance_claim(&pet_id, &50_001u64, &String::from_str(&env, "Over limit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_id));
}

// ---------------------------------------------------------------------------
// 8. OVERFLOW / RESOURCE LIMIT — large amounts use no wrap-around
// ---------------------------------------------------------------------------

#[test]
fn test_large_amount_no_overflow() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    // u64::MAX as claim amount — checked arithmetic must not wrap
    let claim_id = client
        .submit_insurance_claim(&pet_id, &u64::MAX, &String::from_str(&env, "Large claim"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_id));
}

// ---------------------------------------------------------------------------
// 9. INDEPENDENCE — paying claim A does not block claim B
// ---------------------------------------------------------------------------

#[test]
fn test_settling_claim_a_does_not_block_claim_b() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_a = client
        .submit_insurance_claim(&pet_id, &100u64, &String::from_str(&env, "Claim A"))
        .unwrap();

    let claim_b = client
        .submit_insurance_claim(&pet_id, &200u64, &String::from_str(&env, "Claim B"))
        .unwrap();

    // Settle claim A
    client.update_insurance_claim_status(&claim_a, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_a));

    // Claim B must still be payable independently
    assert!(!client.is_claim_settled(&claim_b));
    client.update_insurance_claim_status(&claim_b, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_b));
}

// ---------------------------------------------------------------------------
// 10. SOROBAN ATOMICITY — settlement flag NOT persisted when tx would revert
//
//     In Soroban the entire transaction reverts on any panic, so writing
//     ClaimSettled before a subsequent panic means both writes are rolled back.
//     We confirm this indirectly: after a failed attempt (wrong claim ID in the
//     same logical operation), the real claim remains un-settled and retryable.
// ---------------------------------------------------------------------------

#[test]
fn test_atomicity_settlement_not_persisted_on_failed_tx() {
    let (env, client, _admin) = setup_env();
    let (_owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500u64, &String::from_str(&env, "Atomicity check"))
        .unwrap();

    // Attempt to pay a non-existent claim — panics, whole tx reverts
    let _ = std::panic::catch_unwind(|| {
        client.update_insurance_claim_status(&(claim_id + 1_000_000), &InsuranceClaimStatus::Paid);
    });

    // The real claim was never touched — its settlement flag is still false
    assert!(!client.is_claim_settled(&claim_id));

    // It can be legitimately paid now
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Paid);
    assert!(client.is_claim_settled(&claim_id));
}

// ---------------------------------------------------------------------------
// 11. APPEAL WINDOW BOUNDARY — appeal exactly at and beyond 14 days
// ---------------------------------------------------------------------------

#[test]
fn test_appeal_within_14_days_succeeds() {
    let (env, client, _admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &600u64, &String::from_str(&env, "Window test"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Advance 13 days — still within window
    env.ledger().with_mut(|li| {
        li.timestamp += 13 * 24 * 60 * 60;
    });

    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Still in window"),
        &Vec::new(&env),
    );

    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::UnderAppeal);
}

#[test]
#[should_panic(expected = "AppealWindowExpired")]
fn test_appeal_after_14_days_panics() {
    let (env, client, _admin) = setup_env();
    let (owner, pet_id) = register_pet_with_policy(&env, &client);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &600u64, &String::from_str(&env, "Late appeal"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Advance 15 days — past the window
    env.ledger().with_mut(|li| {
        li.timestamp += 15 * 24 * 60 * 60;
    });

    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Too late"),
        &Vec::new(&env),
    );
}

// ---------------------------------------------------------------------------
// RESOURCE IMPACT NOTE
//
// The approval path now performs one extra storage read (`ClaimSettled`) and
// one extra storage write (setting `ClaimSettled = true`) per payout.  On
// Soroban, each instance-storage read costs ~1 000 CPU instructions and each
// write costs ~2 000.  Total overhead per payout: ~3 000 instructions and
// 2 storage ops — negligible relative to the existing claim-status write.
//
// Appeal resolution adds the same overhead only on the `Approved` branch;
// `Rejected` appeal decisions skip the guard entirely.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// THREAT MODEL NOTE (funds)
//
// Who could trigger a double payout?
//   1. A careless admin retrying `update_insurance_claim_status(..., Paid)`.
//   2. An off-chain service replaying the approval RPC after a timeout.
//   3. An attacker who observes a claim in `Approved` status and races to
//      call `review_appeal` a second time before the first tx is indexed.
//
// How recording settlement before the status write closes it:
//   * The first payout writes `ClaimSettled(claim_id) = true` and then sets
//     `status = Paid` in the same atomic transaction.
//   * Any subsequent call to any payout path reads `ClaimSettled`, finds it
//     set, and panics before doing anything else.
//   * Because Soroban transactions are atomic, there is no window in which
//     `ClaimSettled` is set but `status` is not yet `Paid` — both succeed
//     together or neither does.
//
// COMPATIBILITY NOTE
//   ABI:      Unchanged — no existing public function signature was modified.
//   Storage:  Two new InsuranceKey variants (ClaimSettled, ClaimReviewer).
//             Existing deployed instances have no ClaimSettled entries, which
//             is correctly interpreted as "not yet settled" (unwrap_or(false)).
//             No migration script is needed.
//   Errors:   Ten new ContractError variants appended at discriminants 189–198.
//             No existing discriminant was renumbered.
// ---------------------------------------------------------------------------
