#![no_std]
#![allow(clippy::too_many_arguments)]

//! Event-to-state reconciliation utility & contract (Issue #1506).
//!
//! Deterministically replays ordered contract domain events (`PetRegistered`,
//! `PetDeactivated`, `ConsentGranted`, `ConsentRevoked`, `VaccinationRecorded`,
//! `InsurancePolicyActivated`, `InsuranceClaimPaid`) and compares the
//! reconstructed pet, consent, vaccination, and insurance state against
//! stored snapshots.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Maximum number of events processed in a single replay batch.
pub const MAX_EVENT_LOG_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EventReconcileError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    EmptyEventLog = 4,
    EventLogTooLong = 5,
    EventSequenceDiscontinuity = 6,
    UnsupportedEventVersion = 7,
    SnapshotNotFound = 8,
    InvalidStateTransition = 9,
    ExpiredReconciliationRequest = 10,
    ReplayDetected = 11,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DomainEventKind {
    PetRegistered = 1,
    PetDeactivated = 2,
    ConsentGranted = 3,
    ConsentRevoked = 4,
    VaccinationRecorded = 5,
    InsurancePolicyActivated = 6,
    InsuranceClaimPaid = 7,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StateDomainMismatch {
    PetActiveMismatch = 1,
    ConsentCountMismatch = 2,
    VaccinationCountMismatch = 3,
    InsuranceActiveMismatch = 4,
    InsurancePayoutMismatch = 5,
    SequenceCursorMismatch = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayableDomainEvent {
    pub seq: u64,
    pub version: u32,
    pub kind: DomainEventKind,
    pub amount_or_delta: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconstructedDomainState {
    pub pet_id: u64,
    pub pet_active: bool,
    pub active_consent_count: u32,
    pub vaccination_count: u32,
    pub insurance_active: bool,
    pub total_insurance_paid: i128,
    pub last_event_seq: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventReconciliationReport {
    pub pet_id: u64,
    pub matches_snapshot: bool,
    pub reconstructed: ReconstructedDomainState,
    pub mismatches: Vec<StateDomainMismatch>,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventReconcileCompletedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub matches_snapshot: bool,
    pub mismatch_count: u32,
    pub last_event_seq: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    Snapshot(u64),
    UsedReplayNonce((Address, u64)),
}

#[contract]
pub struct EventStateReconcilerContract;

#[contractimpl]
impl EventStateReconcilerContract {
    pub fn init(env: Env, admin: Address) -> Result<(), EventReconcileError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(EventReconcileError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn store_snapshot(
        env: Env,
        admin: Address,
        snapshot: ReconstructedDomainState,
    ) -> Result<(), EventReconcileError> {
        Self::require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Snapshot(snapshot.pet_id), &snapshot);
        Ok(())
    }

    /// Deterministically replay an ordered event stream into a `ReconstructedDomainState`.
    pub fn replay_events(
        env: Env,
        pet_id: u64,
        events: Vec<ReplayableDomainEvent>,
    ) -> Result<ReconstructedDomainState, EventReconcileError> {
        let _ = env;
        if events.is_empty() {
            return Err(EventReconcileError::EmptyEventLog);
        }
        if events.len() > MAX_EVENT_LOG_LEN {
            return Err(EventReconcileError::EventLogTooLong);
        }

        let mut state = ReconstructedDomainState {
            pet_id,
            pet_active: false,
            active_consent_count: 0,
            vaccination_count: 0,
            insurance_active: false,
            total_insurance_paid: 0,
            last_event_seq: 0,
        };

        for ev in events.iter() {
            if ev.version != EVENT_SCHEMA_VERSION {
                return Err(EventReconcileError::UnsupportedEventVersion);
            }
            if ev.seq != state.last_event_seq + 1 {
                return Err(EventReconcileError::EventSequenceDiscontinuity);
            }

            match ev.kind {
                DomainEventKind::PetRegistered => {
                    if state.pet_active {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.pet_active = true;
                }
                DomainEventKind::PetDeactivated => {
                    if !state.pet_active {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.pet_active = false;
                }
                DomainEventKind::ConsentGranted => {
                    if !state.pet_active {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.active_consent_count += 1;
                }
                DomainEventKind::ConsentRevoked => {
                    if state.active_consent_count == 0 {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.active_consent_count -= 1;
                }
                DomainEventKind::VaccinationRecorded => {
                    if !state.pet_active {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.vaccination_count += 1;
                }
                DomainEventKind::InsurancePolicyActivated => {
                    if !state.pet_active {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.insurance_active = true;
                }
                DomainEventKind::InsuranceClaimPaid => {
                    if !state.insurance_active || ev.amount_or_delta <= 0 {
                        return Err(EventReconcileError::InvalidStateTransition);
                    }
                    state.total_insurance_paid += ev.amount_or_delta;
                }
            }

            state.last_event_seq = ev.seq;
        }

        Ok(state)
    }

    /// Reconstruct state from `events` and compare it with the stored snapshot for `pet_id`.
    pub fn reconcile_against_snapshot(
        env: Env,
        caller: Address,
        pet_id: u64,
        events: Vec<ReplayableDomainEvent>,
        nonce: u64,
        expires_at: u64,
    ) -> Result<EventReconciliationReport, EventReconcileError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(EventReconcileError::NotInitialized);
        }
        caller.require_auth();

        if expires_at < env.ledger().timestamp() {
            return Err(EventReconcileError::ExpiredReconciliationRequest);
        }

        let nonce_key = DataKey::UsedReplayNonce((caller, nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(EventReconcileError::ReplayDetected);
        }

        let snapshot: ReconstructedDomainState = env
            .storage()
            .persistent()
            .get(&DataKey::Snapshot(pet_id))
            .ok_or(EventReconcileError::SnapshotNotFound)?;

        let reconstructed = Self::replay_events(env.clone(), pet_id, events)?;

        let mut mismatches = Vec::new(&env);
        if reconstructed.pet_active != snapshot.pet_active {
            mismatches.push_back(StateDomainMismatch::PetActiveMismatch);
        }
        if reconstructed.active_consent_count != snapshot.active_consent_count {
            mismatches.push_back(StateDomainMismatch::ConsentCountMismatch);
        }
        if reconstructed.vaccination_count != snapshot.vaccination_count {
            mismatches.push_back(StateDomainMismatch::VaccinationCountMismatch);
        }
        if reconstructed.insurance_active != snapshot.insurance_active {
            mismatches.push_back(StateDomainMismatch::InsuranceActiveMismatch);
        }
        if reconstructed.total_insurance_paid != snapshot.total_insurance_paid {
            mismatches.push_back(StateDomainMismatch::InsurancePayoutMismatch);
        }
        if reconstructed.last_event_seq != snapshot.last_event_seq {
            mismatches.push_back(StateDomainMismatch::SequenceCursorMismatch);
        }

        env.storage().persistent().set(&nonce_key, &true);

        let matches_snapshot = mismatches.is_empty();
        env.events().publish(
            (symbol_short!("evt_rec"), pet_id),
            EventReconcileCompletedEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                matches_snapshot,
                mismatch_count: mismatches.len(),
                last_event_seq: reconstructed.last_event_seq,
            },
        );

        Ok(EventReconciliationReport {
            pet_id,
            matches_snapshot,
            reconstructed,
            mismatches,
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        })
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), EventReconcileError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(EventReconcileError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(EventReconcileError::NotInitialized)?;
        if admin != *caller {
            return Err(EventReconcileError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, EventStateReconcilerContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, EventStateReconcilerContract);
        let client = EventStateReconcilerContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_replay_fixture_events_reconciles_cleanly_with_snapshot() {
        let (env, client, admin) = setup();
        let raw = include_str!("../fixtures/event_replay_snapshots.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

        let pet_id = parsed["pet_id"].as_u64().unwrap();
        let snap = &parsed["expected_snapshot"];
        client.store_snapshot(
            &admin,
            &ReconstructedDomainState {
                pet_id,
                pet_active: snap["pet_active"].as_bool().unwrap(),
                active_consent_count: snap["active_consent_count"].as_u64().unwrap() as u32,
                vaccination_count: snap["vaccination_count"].as_u64().unwrap() as u32,
                insurance_active: snap["insurance_active"].as_bool().unwrap(),
                total_insurance_paid: snap["total_insurance_paid"].as_i64().unwrap() as i128,
                last_event_seq: snap["last_event_seq"].as_u64().unwrap(),
            },
        );

        let mut events = Vec::new(&env);
        for ev in parsed["events"].as_array().unwrap() {
            let kind = match ev["kind"].as_str().unwrap() {
                "PetRegistered" => DomainEventKind::PetRegistered,
                "ConsentGranted" => DomainEventKind::ConsentGranted,
                "VaccinationRecorded" => DomainEventKind::VaccinationRecorded,
                "InsurancePolicyActivated" => DomainEventKind::InsurancePolicyActivated,
                "InsuranceClaimPaid" => DomainEventKind::InsuranceClaimPaid,
                other => panic!("unexpected kind: {}", other),
            };
            events.push_back(ReplayableDomainEvent {
                seq: ev["seq"].as_u64().unwrap(),
                version: ev["version"].as_u64().unwrap() as u32,
                kind,
                amount_or_delta: ev["delta"].as_i64().unwrap() as i128,
            });
        }

        let report = client.reconcile_against_snapshot(&admin, &pet_id, &events, &1, &5_000);
        assert!(report.matches_snapshot);
        assert_eq!(report.mismatches.len(), 0);
        assert_eq!(report.reconstructed.total_insurance_paid, 25_000);
    }

    #[test]
    fn test_sequence_gap_mismatch_and_replay_detection() {
        let (env, client, admin) = setup();
        client.store_snapshot(
            &admin,
            &ReconstructedDomainState {
                pet_id: 9,
                pet_active: true,
                active_consent_count: 2,
                vaccination_count: 1,
                insurance_active: false,
                total_insurance_paid: 0,
                last_event_seq: 1,
            },
        );

        let mut ok_events = Vec::new(&env);
        ok_events.push_back(ReplayableDomainEvent {
            seq: 1,
            version: 1,
            kind: DomainEventKind::PetRegistered,
            amount_or_delta: 1,
        });

        let report = client.reconcile_against_snapshot(&admin, &9, &ok_events, &55, &5_000);
        assert!(!report.matches_snapshot);
        assert_eq!(report.mismatches.len(), 2); // ConsentCountMismatch + VaccinationCountMismatch

        // Replay with same nonce fails
        let replay = client.try_reconcile_against_snapshot(&admin, &9, &ok_events, &55, &5_000);
        assert_eq!(replay, Err(Ok(EventReconcileError::ReplayDetected)));

        // Sequence discontinuity (starts at seq 2) rejected
        let mut gapped = Vec::new(&env);
        gapped.push_back(ReplayableDomainEvent {
            seq: 2,
            version: 1,
            kind: DomainEventKind::PetRegistered,
            amount_or_delta: 1,
        });
        let gap_err = client.try_replay_events(&9, &gapped);
        assert_eq!(
            gap_err,
            Err(Ok(EventReconcileError::EventSequenceDiscontinuity))
        );
    }
}
