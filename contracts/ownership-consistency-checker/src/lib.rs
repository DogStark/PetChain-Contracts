#![no_std]
#![allow(clippy::too_many_arguments)]

//! Cross-contract pet ownership consistency checker (Issue #1504).
//!
//! Verifies consistency across four PetChain contract domains:
//! 1. Core `PetRecordSnapshot` (`pet_id -> owner, active`)
//! 2. `OwnerIndexEntry` (`owner -> Vec<pet_id>`)
//! 3. `TransferProposalSnapshot` (`pet_id -> proposer_owner, proposed_recipient, pending`)
//! 4. `CustodyHistoryHead` (`pet_id -> current_custodian, chain_valid`)
//!
//! Reports all inconsistent pet IDs along with deterministic violation codes.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_PETS_PER_CHECK: u32 = 64;
pub const MAX_OWNERS_PER_CHECK: u32 = 32;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConsistencyError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    EmptyPetSet = 4,
    TooManyPets = 5,
    TooManyOwners = 6,
    ExpiredVerification = 7,
    ReplayDetected = 8,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum OwnershipViolationKind {
    MissingFromOwnerIndex = 1,
    OrphanOwnerIndexEntry = 2,
    DuplicateOwnerClaim = 3,
    TransferProposerNotCurrentOwner = 4,
    CustodyHeadOwnerMismatch = 5,
    BrokenCustodyHistoryLink = 6,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetRecordSnapshot {
    pub pet_id: u64,
    pub owner: Address,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnerIndexSnapshot {
    pub owner: Address,
    pub indexed_pet_ids: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferProposalSnapshot {
    pub pet_id: u64,
    pub proposer_owner: Address,
    pub proposed_recipient: Address,
    pub pending: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyHistorySnapshot {
    pub pet_id: u64,
    pub latest_custodian: Address,
    pub chain_links_valid: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetInconsistencyFinding {
    pub pet_id: u64,
    pub violation: OwnershipViolationKind,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipConsistencyReport {
    pub audit_id: u64,
    pub is_consistent: bool,
    pub inconsistent_pet_ids: Vec<u64>,
    pub findings: Vec<PetInconsistencyFinding>,
    pub checked_pet_count: u32,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsistencyAuditEvent {
    pub version: u32,
    pub audit_id: u64,
    pub is_consistent: bool,
    pub inconsistent_pet_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    SavedAudit(u64),
    UsedAuditNonce((Address, u64)),
}

#[contract]
pub struct OwnershipConsistencyContract;

#[contractimpl]
impl OwnershipConsistencyContract {
    pub fn init(env: Env, admin: Address) -> Result<(), ConsistencyError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ConsistencyError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Pure offline/read-only verification of cross-contract ownership invariants.
    pub fn check_consistency(
        env: Env,
        audit_id: u64,
        pets: Vec<PetRecordSnapshot>,
        owner_indexes: Vec<OwnerIndexSnapshot>,
        proposals: Vec<TransferProposalSnapshot>,
        custody_histories: Vec<CustodyHistorySnapshot>,
    ) -> Result<OwnershipConsistencyReport, ConsistencyError> {
        if pets.is_empty() {
            return Err(ConsistencyError::EmptyPetSet);
        }
        if pets.len() > MAX_PETS_PER_CHECK
            || proposals.len() > MAX_PETS_PER_CHECK
            || custody_histories.len() > MAX_PETS_PER_CHECK
        {
            return Err(ConsistencyError::TooManyPets);
        }
        if owner_indexes.len() > MAX_OWNERS_PER_CHECK {
            return Err(ConsistencyError::TooManyOwners);
        }

        let mut findings = Vec::new(&env);
        let mut inconsistent_pet_ids = Vec::new(&env);

        // 1. Check each pet record against owner_indexes, transfer proposals, and custody history
        for pet in pets.iter() {
            let mut claim_count = 0u32;
            let mut claimed_by_owner = false;

            for idx_snap in owner_indexes.iter() {
                if idx_snap.indexed_pet_ids.contains(pet.pet_id) {
                    claim_count += 1;
                    if idx_snap.owner == pet.owner {
                        claimed_by_owner = true;
                    }
                }
            }

            if !claimed_by_owner {
                Self::record_finding(
                    &mut findings,
                    &mut inconsistent_pet_ids,
                    pet.pet_id,
                    OwnershipViolationKind::MissingFromOwnerIndex,
                );
            }
            if claim_count > 1 {
                Self::record_finding(
                    &mut findings,
                    &mut inconsistent_pet_ids,
                    pet.pet_id,
                    OwnershipViolationKind::DuplicateOwnerClaim,
                );
            }

            for prop in proposals.iter() {
                if prop.pet_id == pet.pet_id && prop.pending && prop.proposer_owner != pet.owner {
                    Self::record_finding(
                        &mut findings,
                        &mut inconsistent_pet_ids,
                        pet.pet_id,
                        OwnershipViolationKind::TransferProposerNotCurrentOwner,
                    );
                }
            }

            for cust in custody_histories.iter() {
                if cust.pet_id == pet.pet_id {
                    if !cust.chain_links_valid {
                        Self::record_finding(
                            &mut findings,
                            &mut inconsistent_pet_ids,
                            pet.pet_id,
                            OwnershipViolationKind::BrokenCustodyHistoryLink,
                        );
                    }
                    if cust.latest_custodian != pet.owner {
                        Self::record_finding(
                            &mut findings,
                            &mut inconsistent_pet_ids,
                            pet.pet_id,
                            OwnershipViolationKind::CustodyHeadOwnerMismatch,
                        );
                    }
                }
            }
        }

        // 2. Check owner_indexes for orphan pet IDs that do not exist in `pets`
        for idx_snap in owner_indexes.iter() {
            for pid in idx_snap.indexed_pet_ids.iter() {
                let mut exists = false;
                for pet in pets.iter() {
                    if pet.pet_id == pid {
                        exists = true;
                        break;
                    }
                }
                if !exists {
                    Self::record_finding(
                        &mut findings,
                        &mut inconsistent_pet_ids,
                        pid,
                        OwnershipViolationKind::OrphanOwnerIndexEntry,
                    );
                }
            }
        }

        Ok(OwnershipConsistencyReport {
            audit_id,
            is_consistent: findings.is_empty(),
            inconsistent_pet_ids,
            findings,
            checked_pet_count: pets.len(),
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        })
    }

    /// Run and persist a signed consistency audit report with replay and expiry guards.
    pub fn record_consistency_audit(
        env: Env,
        caller: Address,
        audit_id: u64,
        pets: Vec<PetRecordSnapshot>,
        owner_indexes: Vec<OwnerIndexSnapshot>,
        proposals: Vec<TransferProposalSnapshot>,
        custody_histories: Vec<CustodyHistorySnapshot>,
        nonce: u64,
        expires_at: u64,
    ) -> Result<OwnershipConsistencyReport, ConsistencyError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ConsistencyError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ConsistencyError::NotInitialized)?;
        if caller != admin {
            return Err(ConsistencyError::Unauthorized);
        }

        if expires_at < env.ledger().timestamp() {
            return Err(ConsistencyError::ExpiredVerification);
        }

        let nonce_key = DataKey::UsedAuditNonce((caller, nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(ConsistencyError::ReplayDetected);
        }

        let report = Self::check_consistency(
            env.clone(),
            audit_id,
            pets,
            owner_indexes,
            proposals,
            custody_histories,
        )?;

        env.storage()
            .persistent()
            .set(&DataKey::SavedAudit(audit_id), &report);
        env.storage().persistent().set(&nonce_key, &true);

        env.events().publish(
            (symbol_short!("own_chk"), audit_id),
            ConsistencyAuditEvent {
                version: EVENT_SCHEMA_VERSION,
                audit_id,
                is_consistent: report.is_consistent,
                inconsistent_pet_count: report.inconsistent_pet_ids.len(),
            },
        );

        Ok(report)
    }

    fn record_finding(
        findings: &mut Vec<PetInconsistencyFinding>,
        inconsistent_pet_ids: &mut Vec<u64>,
        pet_id: u64,
        violation: OwnershipViolationKind,
    ) {
        findings.push_back(PetInconsistencyFinding { pet_id, violation });
        if !inconsistent_pet_ids.contains(pet_id) {
            inconsistent_pet_ids.push_back(pet_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, OwnershipConsistencyContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, OwnershipConsistencyContract);
        let client = OwnershipConsistencyContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_consistent_cross_contract_state_passes() {
        let (env, client, admin) = setup();
        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);

        let mut pets = Vec::new(&env);
        pets.push_back(PetRecordSnapshot {
            pet_id: 1,
            owner: owner_a.clone(),
            active: true,
        });

        let mut a_ids = Vec::new(&env);
        a_ids.push_back(1);
        let mut indexes = Vec::new(&env);
        indexes.push_back(OwnerIndexSnapshot {
            owner: owner_a.clone(),
            indexed_pet_ids: a_ids,
        });

        let mut proposals = Vec::new(&env);
        proposals.push_back(TransferProposalSnapshot {
            pet_id: 1,
            proposer_owner: owner_a.clone(),
            proposed_recipient: owner_b,
            pending: true,
        });

        let mut custody = Vec::new(&env);
        custody.push_back(CustodyHistorySnapshot {
            pet_id: 1,
            latest_custodian: owner_a,
            chain_links_valid: true,
        });

        let report = client.record_consistency_audit(
            &admin, &10, &pets, &indexes, &proposals, &custody, &1, &5_000,
        );
        assert!(report.is_consistent);
        assert_eq!(report.inconsistent_pet_ids.len(), 0);
        assert_eq!(report.findings.len(), 0);
    }

    #[test]
    fn test_detects_owner_index_proposal_and_custody_inconsistencies() {
        let (env, client, _admin) = setup();
        let owner_a = Address::generate(&env);
        let owner_b = Address::generate(&env);

        let mut pets = Vec::new(&env);
        pets.push_back(PetRecordSnapshot {
            pet_id: 10,
            owner: owner_a.clone(),
            active: true,
        });
        pets.push_back(PetRecordSnapshot {
            pet_id: 20,
            owner: owner_b.clone(),
            active: true,
        });

        // Owner A's index claims pet 10, pet 20 (duplicate/wrong owner), and orphan pet 99
        let mut a_ids = Vec::new(&env);
        a_ids.push_back(10);
        a_ids.push_back(20);
        a_ids.push_back(99);
        // Owner B's index also claims pet 20
        let mut b_ids = Vec::new(&env);
        b_ids.push_back(20);

        let mut indexes = Vec::new(&env);
        indexes.push_back(OwnerIndexSnapshot {
            owner: owner_a.clone(),
            indexed_pet_ids: a_ids,
        });
        indexes.push_back(OwnerIndexSnapshot {
            owner: owner_b.clone(),
            indexed_pet_ids: b_ids,
        });

        // Stale transfer proposal on pet 10 from owner_b instead of owner_a
        let mut proposals = Vec::new(&env);
        proposals.push_back(TransferProposalSnapshot {
            pet_id: 10,
            proposer_owner: owner_b.clone(),
            proposed_recipient: owner_a,
            pending: true,
        });

        // Broken custody chain on pet 10
        let mut custody = Vec::new(&env);
        custody.push_back(CustodyHistorySnapshot {
            pet_id: 10,
            latest_custodian: owner_b,
            chain_links_valid: false,
        });

        let report = client.check_consistency(&11, &pets, &indexes, &proposals, &custody);
        assert!(!report.is_consistent);
        assert!(report.inconsistent_pet_ids.contains(10));
        assert!(report.inconsistent_pet_ids.contains(20));
        assert!(report.inconsistent_pet_ids.contains(99));
    }

    #[test]
    fn test_unauthorized_replay_and_empty_inputs_rejected() {
        let (env, client, admin) = setup();
        let outsider = Address::generate(&env);
        let empty_pets = Vec::new(&env);
        let indexes = Vec::new(&env);
        let proposals = Vec::new(&env);
        let custody = Vec::new(&env);

        let empty_err =
            client.try_check_consistency(&1, &empty_pets, &indexes, &proposals, &custody);
        assert_eq!(empty_err, Err(Ok(ConsistencyError::EmptyPetSet)));

        let mut pets = Vec::new(&env);
        pets.push_back(PetRecordSnapshot {
            pet_id: 1,
            owner: admin.clone(),
            active: true,
        });

        let unauth = client.try_record_consistency_audit(
            &outsider, &1, &pets, &indexes, &proposals, &custody, &7, &5_000,
        );
        assert_eq!(unauth, Err(Ok(ConsistencyError::Unauthorized)));
    }
}
