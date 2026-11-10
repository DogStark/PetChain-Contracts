#![no_std]
#![allow(clippy::too_many_arguments)]

//! Resource budgets for adoption and custody workflows (Issue #1510).
//!
//! Implements bounded custody transfers, adoption dispute raising/resolution,
//! and bounded batch custody handoffs with storage-footprint tracking and
//! committed worst-case CPU, memory, and storage budget verification.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Maximum pets processed in a single batch custody handoff.
pub const MAX_BATCH_CUSTODY_SIZE: u32 = 16;

/// Maximum length (in bytes) of dispute evidence or transfer notes.
pub const MAX_NOTE_BYTES: u32 = 128;

/// Maximum custody history entries retained per pet.
pub const MAX_CUSTODY_HISTORY: u32 = 32;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CustodyBudgetError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PetNotFound = 4,
    CustodyLockedByDispute = 5,
    DisputeAlreadyOpen = 6,
    DisputeNotFound = 7,
    BatchTooLarge = 8,
    EmptyBatch = 9,
    NoteTooLong = 10,
    ExpiredTransfer = 11,
    ReplayDetected = 12,
    HistoryLimitReached = 13,
    BudgetRegression = 14,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeStatus {
    Open = 1,
    ResolvedForCurrentCustodian = 2,
    ResolvedForChallenger = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyState {
    pub pet_id: u64,
    pub custodian: Address,
    pub transfer_count: u32,
    pub disputed: bool,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyDispute {
    pub dispute_id: u64,
    pub pet_id: u64,
    pub challenger: Address,
    pub evidence_uri: String,
    pub status: DisputeStatus,
    pub opened_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStorageMetrics {
    pub workflow_name: String,
    pub storage_entries_written: u32,
    pub pet_count_processed: u32,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyWorkflowEvent {
    pub version: u32,
    pub pet_id: u64,
    pub storage_entries_written: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    Custody(u64),
    ActiveDispute(u64),
    UsedTransferNonce((Address, u64)),
}

#[contract]
pub struct AdoptionCustodyBudgetContract;

#[contractimpl]
impl AdoptionCustodyBudgetContract {
    pub fn init(env: Env, admin: Address) -> Result<(), CustodyBudgetError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(CustodyBudgetError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn register_custody(
        env: Env,
        custodian: Address,
        pet_id: u64,
    ) -> Result<(), CustodyBudgetError> {
        Self::require_initialized(&env)?;
        custodian.require_auth();

        let state = CustodyState {
            pet_id,
            custodian,
            transfer_count: 0,
            disputed: false,
            updated_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Custody(pet_id), &state);
        Ok(())
    }

    /// Execute a single pet custody transfer with replay/expiry guards and return
    /// the deterministic storage-write footprint.
    pub fn transfer_custody(
        env: Env,
        from: Address,
        to: Address,
        pet_id: u64,
        note: String,
        nonce: u64,
        expires_at: u64,
    ) -> Result<WorkflowStorageMetrics, CustodyBudgetError> {
        Self::require_initialized(&env)?;
        from.require_auth();

        if note.len() > MAX_NOTE_BYTES {
            return Err(CustodyBudgetError::NoteTooLong);
        }
        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(CustodyBudgetError::ExpiredTransfer);
        }

        let nonce_key = DataKey::UsedTransferNonce((from.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(CustodyBudgetError::ReplayDetected);
        }

        let mut state: CustodyState = env
            .storage()
            .persistent()
            .get(&DataKey::Custody(pet_id))
            .ok_or(CustodyBudgetError::PetNotFound)?;

        if state.custodian != from {
            return Err(CustodyBudgetError::Unauthorized);
        }
        if state.disputed {
            return Err(CustodyBudgetError::CustodyLockedByDispute);
        }
        if state.transfer_count >= MAX_CUSTODY_HISTORY {
            return Err(CustodyBudgetError::HistoryLimitReached);
        }

        state.custodian = to;
        state.transfer_count += 1;
        state.updated_at = now;

        env.storage()
            .persistent()
            .set(&DataKey::Custody(pet_id), &state);
        env.storage().persistent().set(&nonce_key, &true);

        env.events().publish(
            (symbol_short!("cust_xfr"), pet_id),
            CustodyWorkflowEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                storage_entries_written: 2,
                timestamp: now,
            },
        );

        Ok(WorkflowStorageMetrics {
            workflow_name: String::from_str(&env, "custody_transfer"),
            storage_entries_written: 2,
            pet_count_processed: 1,
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        })
    }

    /// Open or resolve an adoption custody dispute with bounded evidence.
    pub fn raise_and_resolve_dispute(
        env: Env,
        admin: Address,
        challenger: Address,
        pet_id: u64,
        dispute_id: u64,
        evidence_uri: String,
        award_to_challenger: bool,
    ) -> Result<WorkflowStorageMetrics, CustodyBudgetError> {
        Self::require_initialized(&env)?;
        challenger.require_auth();
        admin.require_auth();

        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(CustodyBudgetError::NotInitialized)?;
        if stored_admin != admin {
            return Err(CustodyBudgetError::Unauthorized);
        }
        if evidence_uri.len() > MAX_NOTE_BYTES {
            return Err(CustodyBudgetError::NoteTooLong);
        }

        let mut state: CustodyState = env
            .storage()
            .persistent()
            .get(&DataKey::Custody(pet_id))
            .ok_or(CustodyBudgetError::PetNotFound)?;
        if state.disputed {
            return Err(CustodyBudgetError::DisputeAlreadyOpen);
        }

        let now = env.ledger().timestamp();
        let status = if award_to_challenger {
            state.custodian = challenger.clone();
            DisputeStatus::ResolvedForChallenger
        } else {
            DisputeStatus::ResolvedForCurrentCustodian
        };
        state.updated_at = now;

        let dispute = CustodyDispute {
            dispute_id,
            pet_id,
            challenger,
            evidence_uri,
            status,
            opened_at: now,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Custody(pet_id), &state);
        env.storage()
            .persistent()
            .set(&DataKey::ActiveDispute(pet_id), &dispute);

        Ok(WorkflowStorageMetrics {
            workflow_name: String::from_str(&env, "adoption_dispute"),
            storage_entries_written: 2,
            pet_count_processed: 1,
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        })
    }

    /// Execute a bounded batch custody handoff for up to `MAX_BATCH_CUSTODY_SIZE` pets.
    pub fn batch_transfer_custody(
        env: Env,
        from: Address,
        to: Address,
        pet_ids: Vec<u64>,
        nonce: u64,
        expires_at: u64,
    ) -> Result<WorkflowStorageMetrics, CustodyBudgetError> {
        Self::require_initialized(&env)?;
        from.require_auth();

        if pet_ids.is_empty() {
            return Err(CustodyBudgetError::EmptyBatch);
        }
        if pet_ids.len() > MAX_BATCH_CUSTODY_SIZE {
            return Err(CustodyBudgetError::BatchTooLarge);
        }

        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(CustodyBudgetError::ExpiredTransfer);
        }

        let nonce_key = DataKey::UsedTransferNonce((from.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(CustodyBudgetError::ReplayDetected);
        }

        // Pre-validate all pets before mutating any state so a failure leaves storage unchanged.
        let mut loaded = Vec::new(&env);
        for pet_id in pet_ids.iter() {
            let state: CustodyState = env
                .storage()
                .persistent()
                .get(&DataKey::Custody(pet_id))
                .ok_or(CustodyBudgetError::PetNotFound)?;
            if state.custodian != from {
                return Err(CustodyBudgetError::Unauthorized);
            }
            if state.disputed {
                return Err(CustodyBudgetError::CustodyLockedByDispute);
            }
            if state.transfer_count >= MAX_CUSTODY_HISTORY {
                return Err(CustodyBudgetError::HistoryLimitReached);
            }
            loaded.push_back(state);
        }

        let mut writes = 0u32;
        for mut state in loaded.iter() {
            state.custodian = to.clone();
            state.transfer_count += 1;
            state.updated_at = now;
            env.storage()
                .persistent()
                .set(&DataKey::Custody(state.pet_id), &state);
            writes += 1;
        }
        env.storage().persistent().set(&nonce_key, &true);
        writes += 1;

        Ok(WorkflowStorageMetrics {
            workflow_name: String::from_str(&env, "bounded_batch_custody_handoff"),
            storage_entries_written: writes,
            pet_count_processed: pet_ids.len(),
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        })
    }

    pub fn get_custody(env: Env, pet_id: u64) -> Option<CustodyState> {
        env.storage().persistent().get(&DataKey::Custody(pet_id))
    }

    fn require_initialized(env: &Env) -> Result<(), CustodyBudgetError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(CustodyBudgetError::NotInitialized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, AdoptionCustodyBudgetContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, AdoptionCustodyBudgetContract);
        let client = AdoptionCustodyBudgetContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_worst_case_workflows_within_committed_budget_snapshot() {
        let (env, client, admin) = setup();
        let raw = include_str!("../snapshots/resource_budgets.json");
        let snap: serde_json::Value = serde_json::from_str(raw).expect("valid snapshot JSON");

        let shelter = Address::generate(&env);
        let adopter = Address::generate(&env);
        let challenger = Address::generate(&env);

        for id in 1..=MAX_BATCH_CUSTODY_SIZE as u64 {
            client.register_custody(&shelter, &id);
        }
        client.register_custody(&shelter, &999);

        // 1. Worst-case single custody transfer with max-length note
        let max_note = String::from_str(&env, &"n".repeat(MAX_NOTE_BYTES as usize));
        env.budget().reset_default();
        let m_xfr = client.transfer_custody(&shelter, &adopter, &999, &max_note, &1, &5_000);
        let cpu_xfr = env.budget().cpu_instruction_cost();
        let mem_xfr = env.budget().memory_bytes_cost();
        let b_xfr = &snap["workflows"]["custody_transfer"];
        assert!(cpu_xfr <= b_xfr["max_cpu_instructions"].as_u64().unwrap());
        assert!(mem_xfr <= b_xfr["max_memory_bytes"].as_u64().unwrap());
        assert!(
            m_xfr.storage_entries_written <= b_xfr["max_storage_entries"].as_u64().unwrap() as u32
        );

        // 2. Worst-case adoption dispute with max-length evidence URI
        env.budget().reset_default();
        let m_disp =
            client.raise_and_resolve_dispute(&admin, &challenger, &999, &501, &max_note, &true);
        let cpu_disp = env.budget().cpu_instruction_cost();
        let mem_disp = env.budget().memory_bytes_cost();
        let b_disp = &snap["workflows"]["adoption_dispute"];
        assert!(cpu_disp <= b_disp["max_cpu_instructions"].as_u64().unwrap());
        assert!(mem_disp <= b_disp["max_memory_bytes"].as_u64().unwrap());
        assert!(
            m_disp.storage_entries_written
                <= b_disp["max_storage_entries"].as_u64().unwrap() as u32
        );

        // 3. Worst-case bounded batch custody handoff (16 pets)
        let mut batch = Vec::new(&env);
        for id in 1..=MAX_BATCH_CUSTODY_SIZE as u64 {
            batch.push_back(id);
        }
        env.budget().reset_default();
        let m_batch = client.batch_transfer_custody(&shelter, &adopter, &batch, &2, &5_000);
        let cpu_batch = env.budget().cpu_instruction_cost();
        let mem_batch = env.budget().memory_bytes_cost();
        let b_batch = &snap["workflows"]["bounded_batch_custody_handoff"];
        assert!(cpu_batch <= b_batch["max_cpu_instructions"].as_u64().unwrap());
        assert!(mem_batch <= b_batch["max_memory_bytes"].as_u64().unwrap());
        assert!(
            m_batch.storage_entries_written
                <= b_batch["max_storage_entries"].as_u64().unwrap() as u32
        );
    }

    #[test]
    fn test_unauthorized_replay_expiry_and_atomic_batch_rollback() {
        let (env, client, _admin) = setup();
        let shelter = Address::generate(&env);
        let adopter = Address::generate(&env);
        let outsider = Address::generate(&env);

        client.register_custody(&shelter, &1);
        client.register_custody(&outsider, &2);

        // Batch containing one unauthorized pet fails atomically and leaves pet 1 with shelter
        let mut mixed_batch = Vec::new(&env);
        mixed_batch.push_back(1);
        mixed_batch.push_back(2);
        let res = client.try_batch_transfer_custody(&shelter, &adopter, &mixed_batch, &10, &5_000);
        assert_eq!(res, Err(Ok(CustodyBudgetError::Unauthorized)));
        assert_eq!(client.get_custody(&1).unwrap().custodian, shelter);

        // Expired transfer rejected
        let exp = client.try_transfer_custody(
            &shelter,
            &adopter,
            &1,
            &String::from_str(&env, "ok"),
            &11,
            &500,
        );
        assert_eq!(exp, Err(Ok(CustodyBudgetError::ExpiredTransfer)));

        // Replay nonce rejected when same sender reuses nonce 12
        client.register_custody(&shelter, &3);
        client.transfer_custody(
            &shelter,
            &adopter,
            &1,
            &String::from_str(&env, "ok"),
            &12,
            &5_000,
        );
        let replay = client.try_transfer_custody(
            &shelter,
            &adopter,
            &3,
            &String::from_str(&env, "replay"),
            &12,
            &5_000,
        );
        assert_eq!(replay, Err(Ok(CustodyBudgetError::ReplayDetected)));
    }
}
