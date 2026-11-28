//! Regression tests for proposer-controlled governance cancellation (#1649).

use crate::test_fixtures::{TestEnv, BASE_TIMESTAMP};
use crate::{
    ContractError, ParamKey, PetChainContract, ProposalAction, ProposalCancelledEvent,
    ProposalState, ProposalVetoedEvent, EVENT_SCHEMA_VERSION,
};
use soroban_sdk::{
    testutils::{Address as _, Events},
    Address, Env, Error, Symbol, TryFromVal, Val,
};

const PROPOSAL_TTL: u64 = 72 * 60 * 60;

/// `propose_action` is an internal contract helper rather than a public
/// contract function, so invoke it in contract context (with mocked auth).
fn propose_action(te: &TestEnv<'_>, action: ProposalAction, ttl: u64) -> u64 {
    let contract_id = te.client.address.clone();
    te.env.as_contract(&contract_id, || {
        PetChainContract::propose_action(te.env.clone(), te.admin1.clone(), action, ttl)
    })
}

fn setup_timelocked_proposal<'a>() -> (TestEnv<'a>, u64) {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 4096));
    let proposal_id = propose_action(&te, action, PROPOSAL_TTL);
    te.client.approve_proposal(&te.admin1, &proposal_id);
    te.client.approve_proposal(&te.admin2, &proposal_id);
    (te, proposal_id)
}

fn event_payloads(env: &Env, topic_name: &str) -> soroban_sdk::Vec<Val> {
    let topic = Symbol::new(env, topic_name);
    let mut payloads = soroban_sdk::Vec::new(env);
    for (_, topics, data) in env.events().all().iter() {
        let matches = topics
            .get(0)
            .and_then(|t| Symbol::try_from_val(env, &t).ok())
            .map(|t| t == topic)
            .unwrap_or(false);
        if matches {
            payloads.push_back(data);
        }
    }
    payloads
}

fn assert_state(te: &TestEnv<'_>, proposal_id: u64, expected: ProposalState) {
    let proposal = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, expected);
}

#[test]
fn original_proposer_cancels_pending_proposal() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let proposal = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, ProposalState::TimelockPending);

    // Cancel one second before execution becomes possible.
    te.advance_time(proposal.timelock_end - BASE_TIMESTAMP - 1);
    te.client.cancel_proposal(&te.admin1, &proposal_id);

    let cancelled = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(cancelled.state, ProposalState::Cancelled);
    assert!(!cancelled.executed);
    assert_eq!(cancelled.approvals.len(), 2);
    assert_eq!(cancelled.veto_count, 0);
    assert_eq!(cancelled.proposed_by, te.admin1);
}

#[test]
fn cancellation_emits_only_proposal_cancelled_event() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let timelock_end = te.client.get_proposal(&proposal_id).unwrap().timelock_end;

    te.advance_time(timelock_end - BASE_TIMESTAMP - 1);
    te.client.cancel_proposal(&te.admin1, &proposal_id);

    let cancelled_events = event_payloads(&te.env, "ProposalCancelled");
    assert_eq!(cancelled_events.len(), 1);
    let event =
        ProposalCancelledEvent::try_from_val(&te.env, &cancelled_events.get(0).unwrap()).unwrap();
    assert_eq!(event.version, EVENT_SCHEMA_VERSION);
    assert_eq!(event.proposal_id, proposal_id);
    assert_eq!(event.proposer, te.admin1);
    assert_eq!(event.timestamp, timelock_end - 1);

    assert_eq!(event_payloads(&te.env, "ProposalVetoed").len(), 0);
}

#[test]
fn non_proposer_cannot_cancel() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let before = te.client.get_proposal(&proposal_id).unwrap();

    let result = te.client.try_cancel_proposal(&te.admin2, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::NotProposalProposer as u32,
        )))
    );

    let after = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(after.state, before.state);
    assert_eq!(after.approvals.len(), before.approvals.len());
    assert_eq!(after.veto_count, before.veto_count);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);
}

#[test]
fn cancellation_after_timelock_expiry_is_rejected() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let proposal = te.client.get_proposal(&proposal_id).unwrap();

    // At `timelock_end` execution is already possible, so the cancellation
    // window must be closed even though the proposal has not expired.
    te.advance_time(proposal.timelock_end - BASE_TIMESTAMP);
    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::CancellationWindowClosed as u32,
        )))
    );

    assert_state(&te, proposal_id, ProposalState::TimelockPending);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);

    // The proposal remains executable after the rejected cancellation.
    te.client.execute_proposal(&proposal_id);
    assert_state(&te, proposal_id, ProposalState::Executed);
}

#[test]
fn cancellation_after_proposal_expiry_is_rejected() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let expires_at = te.client.get_proposal(&proposal_id).unwrap().expires_at;

    te.advance_time(expires_at - BASE_TIMESTAMP + 1);
    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalExpired as u32,
        )))
    );

    assert_state(&te, proposal_id, ProposalState::TimelockPending);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);
}

#[test]
fn executed_proposal_cannot_be_cancelled() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let timelock_end = te.client.get_proposal(&proposal_id).unwrap().timelock_end;

    te.advance_time(timelock_end - BASE_TIMESTAMP);
    te.client.execute_proposal(&proposal_id);

    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalAlreadyExecuted as u32,
        )))
    );

    assert_state(&te, proposal_id, ProposalState::Executed);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);
}

#[test]
fn vetoed_proposal_cannot_be_cancelled() {
    let (te, proposal_id) = setup_timelocked_proposal();

    te.client.veto_proposal(&te.admin2, &proposal_id);

    let vetoed = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(vetoed.state, ProposalState::Vetoed);
    assert_eq!(vetoed.veto_count, 1);
    assert_eq!(te.client.get_proposal_veto_count(&proposal_id), 1);
    assert!(te.client.has_admin_vetoed(&proposal_id, &te.admin2));

    let veto_events = event_payloads(&te.env, "ProposalVetoed");
    assert_eq!(veto_events.len(), 1);
    let event = ProposalVetoedEvent::try_from_val(&te.env, &veto_events.get(0).unwrap()).unwrap();
    assert_eq!(event.version, EVENT_SCHEMA_VERSION);
    assert_eq!(event.proposal_id, proposal_id);
    assert_eq!(event.admin, te.admin2);

    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalVetoed as u32,
        )))
    );
    assert_state(&te, proposal_id, ProposalState::Vetoed);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);

    let execution = te.client.try_execute_proposal(&proposal_id);
    assert_eq!(
        execution,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalVetoed as u32,
        )))
    );
    assert_state(&te, proposal_id, ProposalState::Vetoed);
}

#[test]
fn cancelled_proposal_cannot_be_cancelled_approved_or_executed_again() {
    let (te, proposal_id) = setup_timelocked_proposal();
    te.client.cancel_proposal(&te.admin1, &proposal_id);

    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalCancelled as u32,
        )))
    );
    let approval = te.client.try_approve_proposal(&te.admin2, &proposal_id);
    assert_eq!(
        approval,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalCancelled as u32,
        )))
    );
    let execution = te.client.try_execute_proposal(&proposal_id);
    assert_eq!(
        execution,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalCancelled as u32,
        )))
    );

    let after = te.client.get_proposal(&proposal_id).unwrap();
    assert_eq!(after.state, ProposalState::Cancelled);
    assert_eq!(after.approvals.len(), 2);
    assert_eq!(after.veto_count, 0);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 1);
    assert_eq!(event_payloads(&te.env, "ProposalVetoed").len(), 0);
}

#[test]
fn proposal_without_reached_threshold_cannot_be_cancelled() {
    let te = TestEnv::new();
    let action = ProposalAction::ParameterChange((ParamKey::GlobalStorageQuota, 4096));
    let proposal_id = propose_action(&te, action, PROPOSAL_TTL);
    assert_state(&te, proposal_id, ProposalState::Pending);

    let result = te.client.try_cancel_proposal(&te.admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::CancellationWindowClosed as u32,
        )))
    );

    assert_state(&te, proposal_id, ProposalState::Pending);
    assert_eq!(event_payloads(&te.env, "ProposalCancelled").len(), 0);
}

#[test]
fn non_admin_cannot_veto() {
    let (te, proposal_id) = setup_timelocked_proposal();
    let outsider: Address = Address::generate(&te.env);

    let result = te.client.try_veto_proposal(&outsider, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::NotAnAdmin as u32,
        )))
    );

    assert_state(&te, proposal_id, ProposalState::TimelockPending);
    assert_eq!(te.client.get_proposal_veto_count(&proposal_id), 0);
    assert_eq!(event_payloads(&te.env, "ProposalVetoed").len(), 0);
}
