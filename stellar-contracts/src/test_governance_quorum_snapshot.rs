//! Issue #1211: quorum is computed against the eligible set snapshotted when
//! the proposal was created, so membership changes mid-vote cannot skew it.
//! Assumption: the contract has no delegation; eligible weight is one vote
//! per admin address.

use crate::{test_fixtures::TestEnv, ParamKey, ProposalAction};
use crate::SystemKey;
use soroban_sdk::{testutils::Address as _, Address, Vec};

/// No public add-admin entry point exists, so membership is changed directly
/// in contract storage to emulate a join mid-vote.
fn join(te: &TestEnv, who: &Address) {
    te.env.as_contract(&te.client.address, || {
        let mut admins: Vec<Address> = te.env.storage().instance().get(&SystemKey::Admins).unwrap();
        admins.push_back(who.clone());
        te.env.storage().instance().set(&SystemKey::Admins, &admins);
    });
}

fn setup_with_quorum(percent: u32) -> (TestEnv<'static>, u64) {
    let te = TestEnv::new();
    te.client.set_quorum_percent(&te.admin1, &percent);
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 2000));
    let id = te.client.propose_action(&te.admin1, &action, &(72 * 3_600));
    (te, id)
}

#[test]
fn snapshot_is_recorded_at_proposal_start() {
    let (te, id) = setup_with_quorum(100);
    let snap = te.client.get_proposal_eligible_voters(&id);
    assert_eq!(snap.len(), 2);
    assert!(snap.contains(&te.admin1) && snap.contains(&te.admin2));
}

#[test]
fn joining_admins_do_not_raise_quorum_of_open_proposal() {
    let (te, id) = setup_with_quorum(100);
    let newcomer = Address::generate(&te.env);
    join(&te, &newcomer);
    te.client.approve_proposal(&te.admin1, &id);
    te.client.approve_proposal(&te.admin2, &id);
    let p = te.client.get_proposal(&id).unwrap();
    te.advance_time(p.timelock_end - crate::test_fixtures::BASE_TIMESTAMP + 1);
    // Snapshot had 2 eligible voters, both approved: 100% quorum met.
    te.client.execute_proposal(&id);
}

#[test]
fn late_joiner_approval_does_not_count() {
    let (te, id) = setup_with_quorum(100);
    let newcomer = Address::generate(&te.env);
    join(&te, &newcomer);
    te.client.approve_proposal(&te.admin1, &id);
    te.client.approve_proposal(&newcomer, &id);
    let p = te.client.get_proposal(&id).unwrap();
    te.advance_time(p.timelock_end - crate::test_fixtures::BASE_TIMESTAMP + 1);
    // Threshold (2) met by count, but only 1 snapshotted voter approved (needs 2).
    assert!(te.client.try_execute_proposal(&id).is_err());
}
