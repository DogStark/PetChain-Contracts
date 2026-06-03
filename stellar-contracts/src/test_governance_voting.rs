//! # Governance Voting for Parameter Changes — Issue #699
//!
//! Tests for the `ParameterChange` proposal action:
//! - Vote passes only after quorum
//! - Expired vote is rejected
//! - Parameter is updated atomically on vote passage
//! - Governance workflow uses the shared `TestEnv` fixture

use crate::{
    test_fixtures::{TestEnv, BASE_TIMESTAMP},
    ParamKey, ProposalAction, ProposalState,
};
use soroban_sdk::{testutils::Ledger, Env};

// ─── Helper: propose + approve + execute a parameter change ──────────────────

/// Propose a `ParameterChange` as admin1, approve with admin2 (reaching quorum),
/// advance past the timelock, then execute. Returns the proposal ID.
fn propose_param_and_execute(te: &TestEnv, key: ParamKey, value: u64) -> u64 {
    let action = ProposalAction::ParameterChange((key, value));
    // 72-hour voting window as required by the issue
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    te.client.approve_proposal(&te.admin2, &proposal_id);

    // Advance past the default 24-hour timelock
    let proposal = te.client.get_proposal(&proposal_id).unwrap();
    te.advance_time(proposal.timelock_end - BASE_TIMESTAMP + 1);

    te.client.execute_proposal(&proposal_id);
    proposal_id
}

// ─── Vote passes only after quorum ───────────────────────────────────────────

#[test]
fn param_change_cannot_execute_without_quorum() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 2000));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));

    // Only admin1 has approved (count = 1, required = 2) — must not execute
    let result = te.client.try_execute_proposal(&proposal_id);
    assert!(result.is_err(), "must fail when quorum not reached");
}

#[test]
fn param_change_executes_after_quorum_and_timelock() {
    let te = TestEnv::new();
    let proposal_id = propose_param_and_execute(&te, ParamKey::GlobalStorageQuota, 2000);

    let proposal = te.client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
    assert_eq!(proposal.state, ProposalState::Executed);
}

#[test]
fn param_change_proposal_enters_timelock_pending_after_quorum() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 1500));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));

    // Before second approval: Pending
    let p = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(p.state, ProposalState::Pending);

    // After second approval: TimelockPending (quorum reached)
    te.client.approve_proposal(&te.admin2, &proposal_id);
    let p = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(p.state, ProposalState::TimelockPending);
}

// ─── Parameter updated atomically ────────────────────────────────────────────

#[test]
fn global_storage_quota_updated_atomically_on_execution() {
    let te = TestEnv::new();
    // Verify that the parameter is only changed after execute, not before.
    let initial_quota = te.client.get_global_storage_quota();

    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 9999));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    te.client.approve_proposal(&te.admin2, &proposal_id);

    // After approval (in timelock) the quota must NOT have changed yet.
    let after_approval = te.client.get_global_storage_quota();
    assert_eq!(initial_quota, after_approval, "quota must not change before execution");

    // Advance past timelock and execute
    let p = te.client.get_proposal(&proposal_id).unwrap();
    te.advance_time(p.timelock_end - BASE_TIMESTAMP + 1);
    te.client.execute_proposal(&proposal_id);

    // Now the quota must reflect the new value.
    let final_quota = te.client.get_global_storage_quota();
    assert_eq!(final_quota, 9999, "quota must be updated after execution");
}

#[test]
fn admin_threshold_updated_via_governance() {
    // Build an env with 3 admins so we can lower the threshold to 2 via governance.
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register(crate::PetChainContract, ());
    let client = crate::PetChainContractClient::new(&env, &contract_id);

    let admin1 = soroban_sdk::Address::generate(&env);
    let admin2 = soroban_sdk::Address::generate(&env);
    let admin3 = soroban_sdk::Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());
    admins.push_back(admin3.clone());
    client.init_multisig(&admin1, &admins, &3); // threshold = 3

    let action = ProposalAction::ParameterChange((ParamKey::AdminThreshold, 2));
    let proposal_id = client.propose_action(&admin1, &action, &(72 * 3_600));
    client.approve_proposal(&admin2, &proposal_id);
    client.approve_proposal(&admin3, &proposal_id); // third approval reaches quorum

    // Advance past default 24-hour timelock
    let p = client.get_proposal(&proposal_id).unwrap();
    env.ledger().with_mut(|l| l.timestamp = p.timelock_end + 1);
    client.execute_proposal(&proposal_id);

    assert_eq!(client.get_admin_threshold(), 2);
}

// ─── Expired vote rejected ────────────────────────────────────────────────────

#[test]
#[should_panic]
fn param_change_approve_after_expiry_panics() {
    let te = TestEnv::new();
    // 72-hour voting window
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 500));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));

    // Advance past the voting window
    te.advance_time(72 * 3_600 + 1);

    // admin2 tries to approve an expired vote — must panic
    te.client.approve_proposal(&te.admin2, &proposal_id);
}

#[test]
#[should_panic]
fn param_change_execute_after_expiry_panics() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 500));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    te.client.approve_proposal(&te.admin2, &proposal_id);

    // Advance past the voting window (note: timelock + expiry window overlap)
    te.advance_time(72 * 3_600 + 1);

    te.client.execute_proposal(&proposal_id);
}

// ─── Cannot execute before timelock elapses ──────────────────────────────────

#[test]
#[should_panic]
fn param_change_execute_before_timelock_panics() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 1234));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    te.client.approve_proposal(&te.admin2, &proposal_id);

    // Quorum reached but timelock has NOT elapsed — must panic
    te.client.execute_proposal(&proposal_id);
}

// ─── Non-admin cannot propose ────────────────────────────────────────────────

#[test]
#[should_panic]
fn non_admin_cannot_propose_param_change() {
    let te = TestEnv::new();
    let outsider = soroban_sdk::Address::generate(&te.env);
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 100));
    te.client.propose_action(&outsider, &action, &(72 * 3_600));
}

// ─── Duplicate approval rejected ─────────────────────────────────────────────

#[test]
#[should_panic]
fn duplicate_approval_on_param_change_panics() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 800));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    te.client.approve_proposal(&te.admin2, &proposal_id);
    // admin2 approves a second time — must panic
    te.client.approve_proposal(&te.admin2, &proposal_id);
}

// ─── Cannot execute twice ────────────────────────────────────────────────────

#[test]
#[should_panic]
fn param_change_cannot_execute_twice() {
    let te = TestEnv::new();
    let proposal_id = propose_param_and_execute(&te, ParamKey::GlobalStorageQuota, 500);
    // Second execution must panic
    te.client.execute_proposal(&proposal_id);
}

// ─── HealthScoreCacheTtl parameter ───────────────────────────────────────────

#[test]
fn health_score_cache_ttl_updated_via_governance() {
    let te = TestEnv::new();
    propose_param_and_execute(&te, ParamKey::HealthScoreCacheTtl, 7_200); // 2 hours
    // No panic == success; the key is stored in instance storage.
    // If the contract panics on an unknown key the test would fail here.
}

// ─── 72-hour voting window enforced ─────────────────────────────────────────

#[test]
fn param_change_window_is_72_hours() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 300));
    let proposal_id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));

    let p = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(
        p.expires_at,
        BASE_TIMESTAMP + 72 * 3_600,
        "voting window must be 72 hours"
    );
}
