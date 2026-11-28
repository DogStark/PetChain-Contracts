# Governance proposal cancellation (#1649)

This document describes the proposer-controlled withdrawal path for multisig
governance proposals in `stellar-contracts/src/lib.rs`.

## Who may cancel

`cancel_proposal(proposer, proposal_id)` requires the explicit `Address` of the
caller to authorize (`require_auth`) and must match the `proposed_by` address
recorded when the proposal was created. No multisig threshold is required, and
the multisig threshold and admin list are not changed by cancellation.

Only the original proposer can use this path. Another admin cannot cancel a
proposal on the proposer's behalf; that admin may use the separate veto path
described below.

## When cancellation is permitted

Cancellation is permitted only while all of the following hold:

- the proposal is in `ProposalState::TimelockPending` (quorum was reached and
  the timelock is running);
- the ledger timestamp is before `proposal.timelock_end`; and
- the ledger timestamp is not past `proposal.expires_at`.

The comparison uses `now < timelock_end`. At exactly `timelock_end`,
`execute_proposal` may succeed, so the cancellation window is already closed.
A successful cancellation writes `ProposalState::Cancelled` and emits
`ProposalCancelled(version, proposal_id, proposer, timestamp)`. The proposal is
not deleted; approvals and audit history remain readable through
`get_proposal`.

## Cancellation versus veto

| Operation | Caller | Authorisation | Result state | Event |
|---|---|---|---|---|
| `cancel_proposal` | original proposer only | `proposer.require_auth()`, proposer identity check | `Cancelled` | `ProposalCancelled` |
| `veto_proposal` | any current admin | `admin.require_auth()`, admin membership check | `Vetoed` | `ProposalVetoed` |

Veto is an administrative intervention and does not require the proposer's
consent. Cancellation is a voluntary withdrawal and never emits
`ProposalVetoed`. Veto continues to record `ProposalVeto((proposal_id, admin))`
and increments both `veto_count` and `get_proposal_veto_count`. Changing the
multisig threshold or veto authorization is out of scope for this feature.

## Errors and lifecycle rules

| Error | Code | Meaning |
|---|---:|---|
| `ProposalNotFound` | 47 | No proposal with the supplied ID exists. |
| `NotProposalProposer` | 229 | The authenticated caller is not the original proposer. |
| `CancellationWindowClosed` | 230 | The proposal is not in `TimelockPending`, or the timelock has elapsed. |
| `ProposalExpired` | 43 | The proposal's `expires_at` timestamp has passed. |
| `ProposalAlreadyExecuted` | 38 | The proposal was already executed. |
| `ProposalCancelled` | 228 | The proposal was already cancelled; approval and execution also reject it with this code. |
| `ProposalVetoed` | 86 | The proposal was vetoed; approval and execution also reject it with this code. |

A failed cancellation leaves the stored proposal and event history unchanged.
After the timelock expires, the proposer can no longer withdraw the proposal:
it must be executed by an admin/quorum call or expire naturally. A cancelled or
vetoed proposal is terminal and can never be approved or executed.
