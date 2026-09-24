//! Property tests for the direct-transfer state machine (Issue #1190).
//!
//! Random sequences of transfer operations (including unauthenticated
//! attempts and ledger-time jumps around every window boundary) run against
//! the contract and against a reference model. After every step the tests
//! assert that:
//!
//! - **Allowed states**: each pet is in exactly one of Idle / Pending /
//!   Escrowed / Disputed, an escrow always belongs to the current owner, and
//!   every call returns exactly the result the model predicts (so invalid
//!   transitions fail with the expected error and change nothing).
//! - **Terminality**: a disputed escrow is frozen (never finalized, replaced
//!   or bypassed), and completed transfers are never undone: ownership
//!   history and the custody chain only grow.
//! - **Conservation**: every pet has exactly one owner, listed exactly once
//!   in that owner's index and in no other; history and custody-chain lengths
//!   match the number of completed transfers.

extern crate std;

use crate::{
    ContractError, EscrowedTransfer, OwnershipRecord, PendingTransfer, PetOwnershipContract,
    PetOwnershipContractClient, DEFAULT_TRANSFER_TIMEOUT_SECONDS, DISPUTE_WINDOW_SECONDS,
    TRANSFER_EXPIRY_SECONDS,
};
use proptest::prelude::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    xdr::ScErrorType,
    Address, Env, Error, Vec,
};
// `prop_oneof!` expands to an unqualified `vec!`, which this `no_std` crate
// does not have in scope by default.
use std::vec;
use std::vec::Vec as StdVec;
use Outcome::{Auth, Contract};

const PETS: usize = 2;
const ACTORS: usize = 3;
const DAY: u64 = 24 * 60 * 60;

/// Result of a contract call, reduced to what the model predicts.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Outcome {
    Ok,
    Contract(ContractError),
    /// Host error from a missing authorization.
    Auth,
}

#[derive(Clone, Debug)]
enum Op {
    Initiate { pet: usize, to: usize },
    InitiateWithTimeout { pet: usize, to: usize, days: u32 },
    Accept { pet: usize },
    Cancel { pet: usize },
    Reclaim { pet: usize },
    CancelExpired { pet: usize },
    Dispute { pet: usize, caller: usize },
    Finalize { pet: usize },
    BatchInitiate { pets: StdVec<usize>, to: usize },
    BatchTransfer { pets: StdVec<usize>, to: usize },
    Advance(u64),
}

fn op_strategy() -> impl Strategy<Value = Op> {
    let pet = 0..PETS;
    let actor = 0..ACTORS;
    let batch = prop::collection::vec(0..PETS, 0..=3);
    // Weighted towards the operations that move a transfer forward, so most
    // sequences reach escrow, dispute and finalization.
    prop_oneof![
        3 => (pet.clone(), actor.clone()).prop_map(|(pet, to)| Op::Initiate { pet, to }),
        1 => (pet.clone(), actor.clone(), 0u32..=8)
            .prop_map(|(pet, to, days)| Op::InitiateWithTimeout { pet, to, days }),
        4 => pet.clone().prop_map(|pet| Op::Accept { pet }),
        1 => pet.clone().prop_map(|pet| Op::Cancel { pet }),
        1 => pet.clone().prop_map(|pet| Op::Reclaim { pet }),
        1 => pet.clone().prop_map(|pet| Op::CancelExpired { pet }),
        2 => (pet.clone(), actor.clone()).prop_map(|(pet, caller)| Op::Dispute { pet, caller }),
        3 => pet.clone().prop_map(|pet| Op::Finalize { pet }),
        1 => (batch.clone(), actor.clone()).prop_map(|(pets, to)| Op::BatchInitiate { pets, to }),
        1 => (batch, actor).prop_map(|(pets, to)| Op::BatchTransfer { pets, to }),
        3 => prop::sample::select(vec![
            1,
            DAY - 1,
            DAY,
            DISPUTE_WINDOW_SECONDS - 1,
            DISPUTE_WINDOW_SECONDS,
            TRANSFER_EXPIRY_SECONDS - 1,
            TRANSFER_EXPIRY_SECONDS,
        ])
        .prop_map(Op::Advance),
    ]
}

/// `(op, authenticated)`; unauthenticated steps run with no mocked auths.
fn steps_strategy() -> impl Strategy<Value = StdVec<(Op, bool)>> {
    prop::collection::vec((op_strategy(), prop::bool::weighted(0.85)), 1..40)
}

// ─── Reference model ─────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
struct Pending {
    from: usize,
    to: usize,
    initiated_at: u64,
    timeout_secs: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct Escrow {
    from: usize,
    to: usize,
    escrowed_at: u64,
    disputed: bool,
}

#[derive(Clone, Debug)]
struct PetModel {
    owner: usize,
    pending: Option<Pending>,
    escrow: Option<Escrow>,
    transfers: u32,
}

struct Model {
    now: u64,
    pets: StdVec<PetModel>,
}

impl Model {
    fn new() -> Self {
        let pet = PetModel {
            owner: 0,
            pending: None,
            escrow: None,
            transfers: 0,
        };
        Model {
            now: 0,
            pets: vec![pet; PETS],
        }
    }

    /// Mirrors each entry point's check order and returns the expected
    /// outcome, mutating the model only on success.
    fn apply(&mut self, op: &Op, authed: bool) -> Outcome {
        use ContractError as E;
        let now = self.now;
        match *op {
            Op::Initiate { pet, to } => {
                self.initiate(pet, to, DEFAULT_TRANSFER_TIMEOUT_SECONDS, authed)
            }
            Op::InitiateWithTimeout { pet, to, days } => {
                if days == 0 {
                    return Contract(E::InvalidTimeoutDays);
                }
                self.initiate(pet, to, days as u64 * DAY, authed)
            }
            Op::Accept { pet } => {
                let p = &mut self.pets[pet];
                let Some(pending) = p.pending.clone() else {
                    return Contract(E::NoPendingTransfer);
                };
                if !authed {
                    return Auth;
                }
                if p.owner != pending.from {
                    return Contract(E::Unauthorized);
                }
                if p.escrow.is_some() {
                    return Contract(E::TransferAlreadyPending);
                }
                p.pending = None;
                p.escrow = Some(Escrow {
                    from: pending.from,
                    to: pending.to,
                    escrowed_at: now,
                    disputed: false,
                });
                Outcome::Ok
            }
            Op::Cancel { pet } => {
                let p = &mut self.pets[pet];
                let Some(pending) = &p.pending else {
                    return Contract(E::NoPendingTransfer);
                };
                if !authed {
                    return Auth;
                }
                if p.owner != pending.from {
                    return Contract(E::StaleCancellation);
                }
                p.pending = None;
                Outcome::Ok
            }
            Op::Reclaim { pet } => {
                let p = &mut self.pets[pet];
                let Some(pending) = &p.pending else {
                    return Contract(E::NoPendingTransfer);
                };
                if !authed {
                    return Auth;
                }
                if now - pending.initiated_at < TRANSFER_EXPIRY_SECONDS {
                    return Contract(E::TransferNotExpired);
                }
                p.pending = None;
                Outcome::Ok
            }
            Op::CancelExpired { pet } => {
                let p = &mut self.pets[pet];
                let Some(pending) = &p.pending else {
                    return Contract(E::NoPendingTransfer);
                };
                if now - pending.initiated_at < pending.timeout_secs {
                    return Contract(E::TransferNotExpired);
                }
                p.pending = None;
                Outcome::Ok
            }
            Op::Dispute { pet, caller } => {
                let Some(escrow) = &mut self.pets[pet].escrow else {
                    return Contract(E::NoEscrowedTransfer);
                };
                if escrow.disputed {
                    return Contract(E::TransferAlreadyDisputed);
                }
                if now - escrow.escrowed_at >= DISPUTE_WINDOW_SECONDS {
                    return Contract(E::DisputeWindowNotElapsed);
                }
                if !authed {
                    return Auth;
                }
                if caller != escrow.from && caller != escrow.to {
                    return Contract(E::Unauthorized);
                }
                escrow.disputed = true;
                Outcome::Ok
            }
            Op::Finalize { pet } => {
                let p = &mut self.pets[pet];
                let Some(escrow) = p.escrow.clone() else {
                    return Contract(E::NoEscrowedTransfer);
                };
                if escrow.disputed {
                    return Contract(E::TransferAlreadyDisputed);
                }
                if now - escrow.escrowed_at < DISPUTE_WINDOW_SECONDS {
                    return Contract(E::DisputeWindowNotElapsed);
                }
                if p.owner != escrow.from {
                    return Contract(E::Unauthorized);
                }
                p.escrow = None;
                p.owner = escrow.to;
                p.transfers += 1;
                Outcome::Ok
            }
            Op::BatchInitiate { ref pets, to } => {
                if pets.is_empty() {
                    return Contract(E::EmptyBatch);
                }
                let owner = self.pets[pets[0]].owner;
                for &pet in pets {
                    let p = &self.pets[pet];
                    if p.owner != owner {
                        return Contract(E::BatchOwnerMismatch);
                    }
                    if p.pending.is_some() || p.escrow.is_some() {
                        return Contract(E::TransferAlreadyPending);
                    }
                }
                if !authed {
                    return Auth;
                }
                for &pet in pets {
                    self.pets[pet].pending = Some(Pending {
                        from: owner,
                        to,
                        initiated_at: now,
                        timeout_secs: DEFAULT_TRANSFER_TIMEOUT_SECONDS,
                    });
                }
                Outcome::Ok
            }
            Op::BatchTransfer { ref pets, to } => {
                if pets.is_empty() {
                    return Contract(E::EmptyBatch);
                }
                let owner = self.pets[pets[0]].owner;
                for (i, &pet) in pets.iter().enumerate() {
                    if pets[..i].contains(&pet) {
                        return Contract(E::InvalidBatch);
                    }
                    if self.pets[pet].escrow.is_some() {
                        return Contract(E::TransferAlreadyPending);
                    }
                    if self.pets[pet].owner != owner {
                        return Contract(E::BatchOwnerMismatch);
                    }
                }
                if !authed {
                    return Auth;
                }
                for &pet in pets {
                    self.pets[pet].owner = to;
                    self.pets[pet].transfers += 1;
                }
                Outcome::Ok
            }
            Op::Advance(secs) => {
                self.now += secs;
                Outcome::Ok
            }
        }
    }

    fn initiate(&mut self, pet: usize, to: usize, timeout_secs: u64, authed: bool) -> Outcome {
        let p = &mut self.pets[pet];
        if !authed {
            return Auth;
        }
        if p.pending.is_some() || p.escrow.is_some() {
            return Contract(ContractError::TransferAlreadyPending);
        }
        p.pending = Some(Pending {
            from: p.owner,
            to,
            initiated_at: self.now,
            timeout_secs,
        });
        Outcome::Ok
    }
}

// ─── Contract harness ────────────────────────────────────────────────────────

struct Harness {
    env: Env,
    client: PetOwnershipContractClient<'static>,
    actors: StdVec<Address>,
    /// Last observed history per pet, to check it only ever grows.
    histories: StdVec<Vec<OwnershipRecord>>,
    /// Escrow and owner captured when a pet's escrow became disputed.
    frozen: StdVec<Option<(EscrowedTransfer, Address)>>,
}

fn pet_id(pet: usize) -> u64 {
    pet as u64 + 1
}

fn outcome<T, C>(result: Result<Result<T, C>, Result<Error, soroban_sdk::InvokeError>>) -> Outcome {
    match result {
        Ok(_) => Outcome::Ok,
        Err(Ok(err)) if err.is_type(ScErrorType::Contract) => {
            let code = err.get_code();
            let expected = [
                ContractError::Unauthorized,
                ContractError::TransferAlreadyPending,
                ContractError::NoPendingTransfer,
                ContractError::TransferNotExpired,
                ContractError::StaleCancellation,
                ContractError::EmptyBatch,
                ContractError::BatchOwnerMismatch,
                ContractError::NoEscrowedTransfer,
                ContractError::DisputeWindowNotElapsed,
                ContractError::TransferAlreadyDisputed,
                ContractError::InvalidBatch,
                ContractError::InvalidTimeoutDays,
            ];
            match expected.into_iter().find(|e| *e as u32 == code) {
                Some(e) => Outcome::Contract(e),
                None => panic!("unexpected contract error #{code}"),
            }
        }
        // A failed `require_auth` aborts the call with a host (non-contract)
        // error. The model only predicts this for unauthenticated steps.
        Err(Ok(_)) => Outcome::Auth,
        Err(Err(err)) => panic!("unexpected invoke error: {err:?}"),
    }
}

impl Harness {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();
        let contract_id = env.register_contract(None, PetOwnershipContract);
        let client = PetOwnershipContractClient::new(&env, &contract_id);
        let actors: StdVec<Address> = (0..ACTORS).map(|_| Address::generate(&env)).collect();
        for pet in 0..PETS {
            client.create_pet(&pet_id(pet), &actors[0]);
        }
        let histories = (0..PETS)
            .map(|pet| client.get_ownership_history(&pet_id(pet)))
            .collect();
        Harness {
            env,
            client,
            actors,
            histories,
            frozen: vec![None; PETS],
        }
    }

    fn ids(&self, pets: &[usize]) -> Vec<u64> {
        let mut ids = Vec::new(&self.env);
        for &pet in pets {
            ids.push_back(pet_id(pet));
        }
        ids
    }

    fn run(&self, op: &Op, authed: bool) -> Outcome {
        if !authed {
            self.env.mock_auths(&[]);
        }
        let c = &self.client;
        let a = &self.actors;
        let result = match *op {
            Op::Initiate { pet, to } => outcome(c.try_initiate_transfer(&pet_id(pet), &a[to])),
            Op::InitiateWithTimeout { pet, to, days } => {
                outcome(c.try_initiate_transfer_with_timeout(&pet_id(pet), &a[to], &days))
            }
            Op::Accept { pet } => outcome(c.try_accept_transfer(&pet_id(pet))),
            Op::Cancel { pet } => outcome(c.try_cancel_transfer(&pet_id(pet))),
            Op::Reclaim { pet } => outcome(c.try_reclaim_transfer(&pet_id(pet))),
            Op::CancelExpired { pet } => outcome(c.try_cancel_expired_transfer(&pet_id(pet))),
            Op::Dispute { pet, caller } => outcome(c.try_raise_dispute(&pet_id(pet), &a[caller])),
            Op::Finalize { pet } => outcome(c.try_finalize_transfer(&pet_id(pet))),
            Op::BatchInitiate { ref pets, to } => {
                outcome(c.try_batch_initiate_transfer(&self.ids(pets), &a[to]))
            }
            Op::BatchTransfer { ref pets, to } => {
                outcome(c.try_batch_transfer(&self.ids(pets), &a[to]))
            }
            Op::Advance(secs) => {
                self.env.ledger().with_mut(|l| l.timestamp += secs);
                Outcome::Ok
            }
        };
        self.env.mock_all_auths();
        result
    }

    /// Asserts the contract's observable state matches `model` and that the
    /// allowed-state, terminality and conservation properties hold.
    fn check(&mut self, model: &Model) {
        let c = &self.client;
        let a = &self.actors;
        for (pet, m) in model.pets.iter().enumerate() {
            let id = pet_id(pet);
            let owner = c.get_current_owner(&id);
            assert_eq!(owner, a[m.owner], "owner of pet {pet}");

            let pending = c.get_pending_transfer(&id);
            let expected = m.pending.as_ref().map(|p| PendingTransfer {
                pet_id: id,
                from: a[p.from].clone(),
                to: a[p.to].clone(),
                initiated_at: p.initiated_at,
                timeout_secs: p.timeout_secs,
            });
            assert_eq!(pending, expected, "pending transfer of pet {pet}");

            let escrow = c.get_escrowed_transfer(&id);
            let expected = m.escrow.as_ref().map(|e| EscrowedTransfer {
                pet_id: id,
                from: a[e.from].clone(),
                to: a[e.to].clone(),
                escrowed_at: e.escrowed_at,
                disputed: e.disputed,
            });
            assert_eq!(escrow, expected, "escrowed transfer of pet {pet}");

            // Allowed states: Idle, Pending, Escrowed or Disputed, never two
            // at once, and an escrow always belongs to the current owner.
            assert!(
                pending.is_none() || escrow.is_none(),
                "pet {pet} pending and escrowed"
            );
            if let Some(e) = &escrow {
                assert_eq!(e.from, owner, "escrow of pet {pet} not from its owner");
            }

            // Terminality: a disputed escrow is frozen along with ownership.
            if let Some(frozen) = &self.frozen[pet] {
                assert_eq!(
                    (escrow.clone(), owner.clone()),
                    (Some(frozen.0.clone()), frozen.1.clone()),
                    "disputed escrow of pet {pet} changed"
                );
            } else if let Some(e) = escrow.clone().filter(|e| e.disputed) {
                self.frozen[pet] = Some((e, owner.clone()));
            }

            // Terminality: completed transfers are never undone. History only
            // grows; earlier records are immutable except the previous
            // owner's `relinquished_at` being closed once.
            let history = c.get_ownership_history(&id);
            let before = &self.histories[pet];
            assert!(history.len() >= before.len(), "history of pet {pet} shrank");
            for (i, record) in before.iter().enumerate() {
                let now = history.get(i as u32).unwrap();
                if record.relinquished_at.is_some() {
                    assert_eq!(now, record, "history of pet {pet} rewritten");
                } else {
                    assert_eq!(now.owner, record.owner);
                    assert_eq!(now.acquired_at, record.acquired_at);
                }
            }
            self.histories[pet] = history.clone();

            // Conservation: history and custody chain track completed transfers.
            assert_eq!(
                history.len(),
                m.transfers + 1,
                "history length of pet {pet}"
            );
            assert_eq!(c.get_custody_chain(&id).len(), m.transfers);
            let last = history.last().unwrap();
            assert_eq!((last.owner, last.relinquished_at), (owner, None));
            for record in history.iter().take(history.len() as usize - 1) {
                assert!(record.relinquished_at.is_some());
            }
        }

        // Conservation: each pet is listed exactly once, under its owner.
        for (actor, address) in a.iter().enumerate() {
            let mut listed: StdVec<u64> = c.get_owner_pets(address).iter().collect();
            listed.sort_unstable();
            let owned: StdVec<u64> = (0..PETS)
                .filter(|&pet| model.pets[pet].owner == actor)
                .map(pet_id)
                .collect();
            assert_eq!(listed, owned, "owner index of actor {actor}");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

    #[test]
    fn transfer_state_machine_matches_model(steps in steps_strategy()) {
        let mut harness = Harness::new();
        let mut model = Model::new();
        harness.check(&model);
        for (i, (op, authed)) in steps.iter().enumerate() {
            let expected = model.apply(op, *authed);
            let actual = harness.run(op, *authed);
            prop_assert_eq!(actual, expected, "step {}: {:?} (authed: {})", i, op, authed);
            harness.check(&model);
        }
    }
}

// ─── Focused regressions for the transitions the property tests exercise ─────

/// Escrows a transfer of pet 0 from actor 0 to actor 1 and disputes it.
fn disputed(h: &Harness) {
    h.client.initiate_transfer(&pet_id(0), &h.actors[1]);
    h.client.accept_transfer(&pet_id(0));
    h.client.raise_dispute(&pet_id(0), &h.actors[1]);
}

#[test]
fn disputed_escrow_cannot_be_replaced_by_a_new_transfer() {
    let h = Harness::new();
    disputed(&h);
    assert_eq!(
        outcome(h.client.try_initiate_transfer(&pet_id(0), &h.actors[2])),
        Outcome::Contract(ContractError::TransferAlreadyPending)
    );
    assert!(h.client.get_escrowed_transfer(&pet_id(0)).unwrap().disputed);
}

#[test]
fn escrowed_pet_cannot_be_batch_transferred() {
    let h = Harness::new();
    disputed(&h);
    let ids = h.ids(&[0]);
    assert_eq!(
        outcome(h.client.try_batch_transfer(&ids, &h.actors[2])),
        Outcome::Contract(ContractError::TransferAlreadyPending)
    );
    assert_eq!(
        outcome(h.client.try_batch_initiate_transfer(&ids, &h.actors[2])),
        Outcome::Contract(ContractError::TransferAlreadyPending)
    );
    assert_eq!(h.client.get_current_owner(&pet_id(0)), h.actors[0]);
}
