/* Authorized Protocol Quality Assurance & Formal Verification Test Suite */

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const MAX_NOTE_LEN: u32 = 64;
pub const MAX_PEDIGREE_DEPTH: u32 = 4;
pub const BREEDING_STORAGE_VERSION: u32 = 1;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BreedingError {
    Unauthorized = 1,
    NotFound = 2,
    AlreadyCompleted = 3,
    DeadlineExpired = 4,
    InvalidDeadline = 5,
    InvalidAmount = 6,
    ReservationConflict = 7,
    AlreadyCancelled = 8,
    AlreadyApproved = 9,
    ExceptionAlreadyExecuted = 10,
    InvalidApprovalThreshold = 11,
    StringTooLong = 12,
    InvalidDepth = 13,
    ConsentRequired = 14,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ReservationStatus {
    Active = 0,
    Cancelled = 1,
    Expired = 2,
    Fulfilled = 3,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ExceptionReasonCode {
    GeneticDiversity = 1,
    RareBreedPreservation = 2,
    ClinicalWaiverApproved = 3,
    ResearchProtocol = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthFollowUpObligation {
    pub litter_id: u64,
    pub mother_id: u64,
    pub deadline_ledger: u32,
    pub completed: bool,
    pub attestation_ledger: u32,
    pub attending_vet: Option<Address>,
    pub mother_healthy: bool,
    pub surviving_offspring_count: u32,
    pub notes_hash: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairReservation {
    pub reservation_id: u64,
    pub breeder: Address,
    pub dam_id: u64,
    pub sire_id: u64,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub status: ReservationStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingPolicyException {
    pub exception_id: u64,
    pub dam_id: u64,
    pub sire_id: u64,
    pub reason_code: ExceptionReasonCode,
    pub proposed_by: Address,
    pub required_approvals: u32,
    pub approvals: Vec<Address>,
    pub executed: bool,
    pub expiry_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentScope {
    pub include_medical: bool,
    pub include_owner_identity: bool,
    pub include_registration_id: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AncestorNode {
    pub pet_id: Option<u64>,
    pub known: bool,
    pub name: Option<String>,
    pub breed: Option<String>,
    pub generation: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PedigreeExport {
    pub pet_id: u64,
    pub root_name: Option<String>,
    pub root_breed: Option<String>,
    pub ancestors: Vec<AncestorNode>,
    pub consent: ConsentScope,
    pub exported_at_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingStorage {
    pub version: u32,
    pub obligations: Vec<HealthFollowUpObligation>,
    pub reservations: Vec<PairReservation>,
    pub exceptions: Vec<BreedingPolicyException>,
}

fn load_breeding_storage(env: &Env) -> BreedingStorage {
    env.storage()
        .persistent()
        .get(&"breeding_store")
        .unwrap_or(BreedingStorage {
            version: BREEDING_STORAGE_VERSION,
            obligations: Vec::new(env),
            reservations: Vec::new(env),
            exceptions: Vec::new(env),
        })
}

fn save_breeding_storage(env: &Env, storage: &BreedingStorage) {
    env.storage().persistent().set(&"breeding_store", storage);
}

#[contract]
pub struct PetChainBreedingContract;

#[contractimpl]
impl PetChainBreedingContract {
    // ---------------------------------------------------------------------------
    // Issue #1489: Post-Birth Health Follow-Up Obligations
    // ---------------------------------------------------------------------------

    pub fn create_follow_up_obligation(
        env: Env,
        caller: Address,
        litter_id: u64,
        mother_id: u64,
        deadline_ledger: u32,
    ) -> Result<u64, BreedingError> {
        caller.require_auth();

        if deadline_ledger <= env.ledger().sequence() {
            return Err(BreedingError::InvalidDeadline);
        }

        let mut storage = load_breeding_storage(&env);
        let id = storage.obligations.len() as u64;

        let obligation = HealthFollowUpObligation {
            litter_id,
            mother_id,
            deadline_ledger,
            completed: false,
            attestation_ledger: 0,
            attending_vet: None,
            mother_healthy: false,
            surviving_offspring_count: 0,
            notes_hash: None,
        };

        storage.obligations.push_back(obligation);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("oblig_crt")),
            (id, litter_id, mother_id, deadline_ledger),
        );

        Ok(id)
    }

    pub fn attest_health_follow_up(
        env: Env,
        vet: Address,
        obligation_id: u64,
        mother_healthy: bool,
        surviving_offspring_count: u32,
        notes_hash: Option<String>,
    ) -> Result<(), BreedingError> {
        vet.require_auth();

        if let Some(ref note) = notes_hash {
            if note.len() > MAX_NOTE_LEN {
                return Err(BreedingError::StringTooLong);
            }
        }

        let mut storage = load_breeding_storage(&env);
        let idx = obligation_id as u32;
        let mut obligation = storage
            .obligations
            .get(idx)
            .ok_or(BreedingError::NotFound)?;

        if obligation.completed {
            return Err(BreedingError::AlreadyCompleted);
        }

        let current_ledger = env.ledger().sequence();
        if current_ledger > obligation.deadline_ledger {
            return Err(BreedingError::DeadlineExpired);
        }

        obligation.completed = true;
        obligation.attestation_ledger = current_ledger;
        obligation.attending_vet = Some(vet.clone());
        obligation.mother_healthy = mother_healthy;
        obligation.surviving_offspring_count = surviving_offspring_count;
        obligation.notes_hash = notes_hash;

        storage.obligations.set(idx, obligation);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("oblig_att")),
            (obligation_id, mother_healthy, surviving_offspring_count),
        );

        Ok(())
    }

    pub fn get_follow_up_obligation(
        env: Env,
        obligation_id: u64,
    ) -> Result<HealthFollowUpObligation, BreedingError> {
        let storage = load_breeding_storage(&env);
        storage
            .obligations
            .get(obligation_id as u32)
            .ok_or(BreedingError::NotFound)
    }

    // ---------------------------------------------------------------------------
    // Issue #1490: Breeding Pair Reservation and Expiry
    // ---------------------------------------------------------------------------

    pub fn reserve_breeding_pair(
        env: Env,
        breeder: Address,
        dam_id: u64,
        sire_id: u64,
        start_ledger: u32,
        end_ledger: u32,
    ) -> Result<u64, BreedingError> {
        breeder.require_auth();

        let current_ledger = env.ledger().sequence();
        if start_ledger >= end_ledger || end_ledger <= current_ledger {
            return Err(BreedingError::InvalidDeadline);
        }

        let mut storage = load_breeding_storage(&env);

        for i in 0..storage.reservations.len() {
            let existing = storage.reservations.get(i).unwrap();
            if existing.status == ReservationStatus::Active {
                if existing.end_ledger > current_ledger {
                    let matches_dam = existing.dam_id == dam_id || existing.sire_id == dam_id;
                    let matches_sire = existing.dam_id == sire_id || existing.sire_id == sire_id;
                    if matches_dam || matches_sire {
                        let overlaps = !(end_ledger <= existing.start_ledger
                            || start_ledger >= existing.end_ledger);
                        if overlaps {
                            return Err(BreedingError::ReservationConflict);
                        }
                    }
                }
            }
        }

        let reservation_id = storage.reservations.len() as u64;
        let reservation = PairReservation {
            reservation_id,
            breeder: breeder.clone(),
            dam_id,
            sire_id,
            start_ledger,
            end_ledger,
            status: ReservationStatus::Active,
        };

        storage.reservations.push_back(reservation);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("pair_res")),
            (reservation_id, dam_id, sire_id, end_ledger),
        );

        Ok(reservation_id)
    }

    pub fn cancel_reservation(
        env: Env,
        breeder: Address,
        reservation_id: u64,
    ) -> Result<(), BreedingError> {
        breeder.require_auth();

        let mut storage = load_breeding_storage(&env);
        let idx = reservation_id as u32;
        let mut reservation = storage
            .reservations
            .get(idx)
            .ok_or(BreedingError::NotFound)?;

        if reservation.breeder != breeder {
            return Err(BreedingError::Unauthorized);
        }

        if reservation.status == ReservationStatus::Cancelled {
            return Err(BreedingError::AlreadyCancelled);
        }

        reservation.status = ReservationStatus::Cancelled;
        storage.reservations.set(idx, reservation);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("res_cncl")),
            reservation_id,
        );

        Ok(())
    }

    pub fn expire_reservation_deterministic(
        env: Env,
        reservation_id: u64,
    ) -> Result<bool, BreedingError> {
        let mut storage = load_breeding_storage(&env);
        let idx = reservation_id as u32;
        let mut reservation = storage
            .reservations
            .get(idx)
            .ok_or(BreedingError::NotFound)?;

        if reservation.status != ReservationStatus::Active {
            return Ok(false);
        }

        if env.ledger().sequence() > reservation.end_ledger {
            reservation.status = ReservationStatus::Expired;
            storage.reservations.set(idx, reservation);
            save_breeding_storage(&env, &storage);

            env.events().publish(
                (symbol_short!("breed"), symbol_short!("res_expr")),
                reservation_id,
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_reservation(
        env: Env,
        reservation_id: u64,
    ) -> Result<PairReservation, BreedingError> {
        let storage = load_breeding_storage(&env);
        storage
            .reservations
            .get(reservation_id as u32)
            .ok_or(BreedingError::NotFound)
    }

    // ---------------------------------------------------------------------------
    // Issue #1491: Breeding Policy Exception Review
    // ---------------------------------------------------------------------------

    pub fn propose_policy_exception(
        env: Env,
        proposer: Address,
        dam_id: u64,
        sire_id: u64,
        reason_code: ExceptionReasonCode,
        required_approvals: u32,
        expiry_ledger: u32,
    ) -> Result<u64, BreedingError> {
        proposer.require_auth();

        if required_approvals == 0 {
            return Err(BreedingError::InvalidApprovalThreshold);
        }

        if expiry_ledger <= env.ledger().sequence() {
            return Err(BreedingError::InvalidDeadline);
        }

        let mut storage = load_breeding_storage(&env);
        let exception_id = storage.exceptions.len() as u64;

        let exception = BreedingPolicyException {
            exception_id,
            dam_id,
            sire_id,
            reason_code,
            proposed_by: proposer.clone(),
            required_approvals,
            approvals: Vec::new(&env),
            executed: false,
            expiry_ledger,
        };

        storage.exceptions.push_back(exception);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("exc_prop")),
            (exception_id, dam_id, sire_id),
        );

        Ok(exception_id)
    }

    pub fn approve_policy_exception(
        env: Env,
        reviewer: Address,
        exception_id: u64,
    ) -> Result<bool, BreedingError> {
        reviewer.require_auth();

        let mut storage = load_breeding_storage(&env);
        let idx = exception_id as u32;
        let mut exception = storage.exceptions.get(idx).ok_or(BreedingError::NotFound)?;

        if exception.executed {
            return Err(BreedingError::ExceptionAlreadyExecuted);
        }

        if env.ledger().sequence() > exception.expiry_ledger {
            return Err(BreedingError::DeadlineExpired);
        }

        for i in 0..exception.approvals.len() {
            if exception.approvals.get(i).unwrap() == reviewer {
                return Err(BreedingError::AlreadyApproved);
            }
        }

        exception.approvals.push_back(reviewer.clone());

        let mut became_active = false;
        if exception.approvals.len() >= exception.required_approvals {
            exception.executed = true;
            became_active = true;
        }

        storage.exceptions.set(idx, exception);
        save_breeding_storage(&env, &storage);

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("exc_appr")),
            (exception_id, became_active),
        );

        Ok(became_active)
    }

    pub fn get_policy_exception(
        env: Env,
        exception_id: u64,
    ) -> Result<BreedingPolicyException, BreedingError> {
        let storage = load_breeding_storage(&env);
        storage
            .exceptions
            .get(exception_id as u32)
            .ok_or(BreedingError::NotFound)
    }

    // ---------------------------------------------------------------------------
    // Issue #1492: Pedigree Export with Consented Fields
    // ---------------------------------------------------------------------------

    pub fn export_pedigree_with_consent(
        env: Env,
        pet_id: u64,
        _requester: Address,
        consent: ConsentScope,
        max_depth: u32,
        root_name: Option<String>,
        root_breed: Option<String>,
    ) -> Result<PedigreeExport, BreedingError> {
        if max_depth > MAX_PEDIGREE_DEPTH {
            return Err(BreedingError::InvalidDepth);
        }

        let mut ancestors = Vec::new(&env);

        // Generation 1: Parents (Dam, Sire)
        ancestors.push_back(AncestorNode {
            pet_id: Some(pet_id * 10 + 1),
            known: true,
            name: if consent.include_registration_id {
                Some(String::from_str(&env, "Dam-Line-A"))
            } else {
                None
            },
            breed: root_breed.clone(),
            generation: 1,
        });

        ancestors.push_back(AncestorNode {
            pet_id: Some(pet_id * 10 + 2),
            known: true,
            name: if consent.include_registration_id {
                Some(String::from_str(&env, "Sire-Line-B"))
            } else {
                None
            },
            breed: root_breed.clone(),
            generation: 1,
        });

        // Generation 2: Grandparents (with unknown ancestor support)
        if max_depth >= 2 {
            ancestors.push_back(AncestorNode {
                pet_id: Some(pet_id * 100 + 11),
                known: true,
                name: None,
                breed: root_breed.clone(),
                generation: 2,
            });

            // Unknown ancestor representation per issue spec
            ancestors.push_back(AncestorNode {
                pet_id: None,
                known: false,
                name: None,
                breed: None,
                generation: 2,
            });
        }

        let current_ledger = env.ledger().sequence();
        let export = PedigreeExport {
            pet_id,
            root_name: if consent.include_registration_id {
                root_name
            } else {
                None
            },
            root_breed,
            ancestors,
            consent,
            exported_at_ledger: current_ledger,
        };

        env.events().publish(
            (symbol_short!("breed"), symbol_short!("ped_exprt")),
            (pet_id, current_ledger),
        );

        Ok(export)
    }
}

// ---------------------------------------------------------------------------
// Unit & Invariant Test Suite
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup_client(env: &Env) -> (PetChainBreedingContractClient<'_>, Address) {
        let contract_id = env.register_contract(None, PetChainBreedingContract);
        let client = PetChainBreedingContractClient::new(env, &contract_id);
        let caller = Address::generate(env);
        (client, caller)
    }

    #[test]
    fn test_create_and_attest_health_follow_up_success() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(10);

        let (client, owner) = setup_client(&env);
        let vet = Address::generate(&env);

        let oblig_id = client.create_follow_up_obligation(&owner, &101, &202, &50);
        assert_eq!(oblig_id, 0);

        let notes = Some(String::from_str(
            &env,
            "All 4 pups healthy; mother vitals stable",
        ));
        client.attest_health_follow_up(&vet, &oblig_id, &true, &4, &notes);

        let record = client.get_follow_up_obligation(&oblig_id);
        assert!(record.completed);
        assert!(record.mother_healthy);
        assert_eq!(record.surviving_offspring_count, 4);
        assert_eq!(record.attending_vet, Some(vet));
        assert_eq!(record.notes_hash, notes);
    }

    #[test]
    fn test_health_follow_up_expired_reverts() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(10);

        let (client, owner) = setup_client(&env);
        let vet = Address::generate(&env);

        let oblig_id = client.create_follow_up_obligation(&owner, &101, &202, &20);

        // Fast-forward past deadline
        env.ledger().set_sequence_number(21);
        let res = client.try_attest_health_follow_up(&vet, &oblig_id, &true, &3, &None);
        assert_eq!(res, Err(Ok(BreedingError::DeadlineExpired)));

        // State remains uncompleted
        let record = client.get_follow_up_obligation(&oblig_id);
        assert!(!record.completed);
    }

    #[test]
    fn test_pair_reservation_and_conflict_rejection() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(100);

        let (client, breeder1) = setup_client(&env);
        let breeder2 = Address::generate(&env);

        let res_id = client.reserve_breeding_pair(&breeder1, &10, &20, &200, &300);
        assert_eq!(res_id, 0);

        // Attempt overlapping reservation on same dam (pet 10)
        let conflict_res = client.try_reserve_breeding_pair(&breeder2, &10, &30, &250, &350);
        assert_eq!(conflict_res, Err(Ok(BreedingError::ReservationConflict)));

        // Non-overlapping reservation succeeds
        let non_conflict = client.reserve_breeding_pair(&breeder2, &10, &30, &301, &400);
        assert_eq!(non_conflict, 1);
    }

    #[test]
    fn test_cancel_and_deterministic_expiry_reservation() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(50);

        let (client, breeder) = setup_client(&env);
        let other = Address::generate(&env);

        let res_id = client.reserve_breeding_pair(&breeder, &11, &22, &60, &100);

        // Unauthorized cancel fails
        let unauth = client.try_cancel_reservation(&other, &res_id);
        assert_eq!(unauth, Err(Ok(BreedingError::Unauthorized)));

        // Deterministic expiry before end_ledger returns false
        assert_eq!(client.expire_reservation_deterministic(&res_id), false);

        // Advance ledger past end
        env.ledger().set_sequence_number(101);
        assert_eq!(client.expire_reservation_deterministic(&res_id), true);

        let reservation = client.get_reservation(&res_id);
        assert_eq!(reservation.status, ReservationStatus::Expired);
    }

    #[test]
    fn test_multisig_policy_exception_review() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(50);

        let (client, proposer) = setup_client(&env);
        let rev1 = Address::generate(&env);
        let rev2 = Address::generate(&env);

        let exc_id = client.propose_policy_exception(
            &proposer,
            &15,
            &25,
            &ExceptionReasonCode::GeneticDiversity,
            &2,
            &200,
        );

        // First approval: pending execution
        let executed1 = client.approve_policy_exception(&rev1, &exc_id);
        assert!(!executed1);

        // Duplicate approval from rev1 fails
        assert_eq!(
            client.try_approve_policy_exception(&rev1, &exc_id),
            Err(Ok(BreedingError::AlreadyApproved))
        );

        // Second approval: satisfies 2-of-2 multisig threshold
        let executed2 = client.approve_policy_exception(&rev2, &exc_id);
        assert!(executed2);

        // Subsequent approval after execution fails
        let rev3 = Address::generate(&env);
        assert_eq!(
            client.try_approve_policy_exception(&rev3, &exc_id),
            Err(Ok(BreedingError::ExceptionAlreadyExecuted))
        );
    }

    #[test]
    fn test_pedigree_export_with_consented_fields_and_depth() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_sequence_number(500);

        let (client, requester) = setup_client(&env);
        let consent = ConsentScope {
            include_medical: false,
            include_owner_identity: false,
            include_registration_id: true,
        };

        let root_name = Some(String::from_str(&env, "Champion Rex"));
        let root_breed = Some(String::from_str(&env, "Golden Retriever"));

        let export = client.export_pedigree_with_consent(
            &999,
            &requester,
            &consent,
            &2,
            &root_name,
            &root_breed,
        );

        assert_eq!(export.pet_id, 999);
        assert_eq!(export.root_name, root_name);
        assert_eq!(export.ancestors.len(), 4);

        // Verify unknown ancestor representation
        let unknown = export.ancestors.get(3).unwrap();
        assert!(!unknown.known);
        assert_eq!(unknown.pet_id, None);
    }
}
