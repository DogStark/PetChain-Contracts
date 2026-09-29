//! Issue #1210: a single voting identity must never be counted twice.

use crate::{
    test_fixtures::TestEnv, MultiSigProposal, ParamKey, ProposalAction, SystemKey,
};
use soroban_sdk::Vec;

fn new_proposal(te: &TestEnv) -> u64 {
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 2000));
    te.client.propose_action(&te.admin1, &action, &(72 * 3_600))
}

#[test]
fn approving_twice_is_rejected() {
    let te = TestEnv::new();
    let id = new_proposal(&te);
    te.client.approve_proposal(&te.admin1, &id);
    assert!(te.client.try_approve_proposal(&te.admin1, &id).is_err());
    assert_eq!(te.client.get_proposal(&id).unwrap().approvals.len(), 1);
}

#[test]
fn duplicated_approval_entries_count_once_at_execution() {
    let te = TestEnv::new();
    let id = new_proposal(&te);
    // Corrupt/legacy state: the same identity recorded twice must not satisfy
    // a 2-of-2 threshold.
    te.env.as_contract(&te.client.address, || {
        let mut p: MultiSigProposal = te
            .env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(id))
            .unwrap();
        let mut dup = Vec::new(&te.env);
        dup.push_back(te.admin1.clone());
        dup.push_back(te.admin1.clone());
        p.approvals = dup;
        te.env.storage().instance().set(&SystemKey::Proposal(id), &p);
    });
    assert!(te.client.try_execute_proposal(&id).is_err());
}
