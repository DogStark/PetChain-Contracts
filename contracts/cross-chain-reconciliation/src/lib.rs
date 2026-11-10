#![no_std]
#![allow(clippy::too_many_arguments)]

//! Cross-chain pet state reconciliation contract & fixture verifier (Issue #1511).
//!
//! Compares Soroban pet representations (owner binding, active flag, medical
//! record commitment digest, and record count) against attested Celo registry
//! snapshots and emits deterministic reconciliation reports.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Maximum batch size for multi-pet cross-chain reconciliation.
pub const MAX_RECONCILE_BATCH: u32 = 32;

/// Maximum length for hex-encoded EVM owner bindings (`0x` + 40 hex chars).
pub const MAX_EVM_ADDRESS_LEN: u32 = 42;

/// Maximum length for hex-encoded 32-byte commitment digests.
pub const MAX_COMMITMENT_HEX_LEN: u32 = 66;

/// Canonical Celo Mainnet chain ID.
pub const CELO_MAINNET_CHAIN_ID: u64 = 42_220;

/// Canonical Celo Alfajores Testnet chain ID.
pub const CELO_ALFAJORES_CHAIN_ID: u64 = 44_787;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReconciliationError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PetNotFound = 4,
    SnapshotNotFound = 5,
    InvalidEvmAddressLength = 6,
    InvalidCommitmentLength = 7,
    UnsupportedChainId = 8,
    StaleBlockNumber = 9,
    ExpiredAttestation = 10,
    ReplayDetected = 11,
    BatchTooLarge = 12,
    EmptyBatch = 13,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DiscrepancyKind {
    OwnerMismatch = 1,
    ActiveStateMismatch = 2,
    RecordCommitmentMismatch = 3,
    RecordCountMismatch = 4,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SorobanPetRepresentation {
    pub pet_id: u64,
    pub owner: Address,
    pub evm_owner_binding: String,
    pub active: bool,
    pub record_commitment: String,
    pub record_count: u32,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CeloPetSnapshot {
    pub pet_id: u64,
    pub chain_id: u64,
    pub owner_evm: String,
    pub is_active: bool,
    pub record_commitment: String,
    pub record_count: u32,
    pub block_number: u64,
    pub attested_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetReconciliationResult {
    pub pet_id: u64,
    pub chain_id: u64,
    pub celo_block_number: u64,
    pub is_consistent: bool,
    pub discrepancies: Vec<DiscrepancyKind>,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReconciliationEvent {
    pub version: u32,
    pub pet_id: u64,
    pub chain_id: u64,
    pub is_consistent: bool,
    pub discrepancy_count: u32,
    pub block_number: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    Relayer(Address),
    SorobanPet(u64),
    CeloSnapshot(u64),
    UsedAttestationNonce((u64, u64)),
}

#[contract]
pub struct CrossChainReconciliationContract;

#[contractimpl]
impl CrossChainReconciliationContract {
    pub fn init(env: Env, admin: Address, relayer: Address) -> Result<(), ReconciliationError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ReconciliationError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::Relayer(relayer), &true);
        Ok(())
    }

    pub fn upsert_soroban_pet(
        env: Env,
        owner: Address,
        pet_id: u64,
        evm_owner_binding: String,
        active: bool,
        record_commitment: String,
        record_count: u32,
    ) -> Result<(), ReconciliationError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        Self::validate_bounds(&evm_owner_binding, &record_commitment)?;

        if let Some(existing) = env
            .storage()
            .persistent()
            .get::<DataKey, SorobanPetRepresentation>(&DataKey::SorobanPet(pet_id))
        {
            if existing.owner != owner {
                return Err(ReconciliationError::Unauthorized);
            }
        }

        let repr = SorobanPetRepresentation {
            pet_id,
            owner,
            evm_owner_binding,
            active,
            record_commitment,
            record_count,
            updated_at: env.ledger().timestamp(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::SorobanPet(pet_id), &repr);
        Ok(())
    }

    pub fn record_celo_snapshot(
        env: Env,
        relayer: Address,
        pet_id: u64,
        chain_id: u64,
        owner_evm: String,
        is_active: bool,
        record_commitment: String,
        record_count: u32,
        block_number: u64,
        nonce: u64,
        expires_at: u64,
    ) -> Result<(), ReconciliationError> {
        Self::require_initialized(&env)?;
        relayer.require_auth();

        let is_relayer: bool = env
            .storage()
            .persistent()
            .get(&DataKey::Relayer(relayer))
            .unwrap_or(false);
        if !is_relayer {
            return Err(ReconciliationError::Unauthorized);
        }

        if chain_id != CELO_MAINNET_CHAIN_ID && chain_id != CELO_ALFAJORES_CHAIN_ID {
            return Err(ReconciliationError::UnsupportedChainId);
        }

        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(ReconciliationError::ExpiredAttestation);
        }

        Self::validate_bounds(&owner_evm, &record_commitment)?;

        let nonce_key = DataKey::UsedAttestationNonce((pet_id, nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(ReconciliationError::ReplayDetected);
        }

        if let Some(prev) = env
            .storage()
            .persistent()
            .get::<DataKey, CeloPetSnapshot>(&DataKey::CeloSnapshot(pet_id))
        {
            if block_number <= prev.block_number {
                return Err(ReconciliationError::StaleBlockNumber);
            }
        }

        let snapshot = CeloPetSnapshot {
            pet_id,
            chain_id,
            owner_evm,
            is_active,
            record_commitment,
            record_count,
            block_number,
            attested_at: now,
        };
        env.storage()
            .persistent()
            .set(&DataKey::CeloSnapshot(pet_id), &snapshot);
        env.storage().persistent().set(&nonce_key, &true);
        Ok(())
    }

    pub fn reconcile_pet(
        env: Env,
        pet_id: u64,
    ) -> Result<PetReconciliationResult, ReconciliationError> {
        Self::require_initialized(&env)?;

        let soroban_pet: SorobanPetRepresentation = env
            .storage()
            .persistent()
            .get(&DataKey::SorobanPet(pet_id))
            .ok_or(ReconciliationError::PetNotFound)?;

        let celo_snap: CeloPetSnapshot = env
            .storage()
            .persistent()
            .get(&DataKey::CeloSnapshot(pet_id))
            .ok_or(ReconciliationError::SnapshotNotFound)?;

        let mut discrepancies = Vec::new(&env);
        if soroban_pet.evm_owner_binding != celo_snap.owner_evm {
            discrepancies.push_back(DiscrepancyKind::OwnerMismatch);
        }
        if soroban_pet.active != celo_snap.is_active {
            discrepancies.push_back(DiscrepancyKind::ActiveStateMismatch);
        }
        if soroban_pet.record_commitment != celo_snap.record_commitment {
            discrepancies.push_back(DiscrepancyKind::RecordCommitmentMismatch);
        }
        if soroban_pet.record_count != celo_snap.record_count {
            discrepancies.push_back(DiscrepancyKind::RecordCountMismatch);
        }

        let is_consistent = discrepancies.is_empty();
        let result = PetReconciliationResult {
            pet_id,
            chain_id: celo_snap.chain_id,
            celo_block_number: celo_snap.block_number,
            is_consistent,
            discrepancies: discrepancies.clone(),
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        };

        env.events().publish(
            (symbol_short!("xrec_pet"), pet_id),
            ReconciliationEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                chain_id: celo_snap.chain_id,
                is_consistent,
                discrepancy_count: discrepancies.len(),
                block_number: celo_snap.block_number,
            },
        );

        Ok(result)
    }

    pub fn reconcile_batch(
        env: Env,
        pet_ids: Vec<u64>,
    ) -> Result<Vec<PetReconciliationResult>, ReconciliationError> {
        Self::require_initialized(&env)?;
        if pet_ids.is_empty() {
            return Err(ReconciliationError::EmptyBatch);
        }
        if pet_ids.len() > MAX_RECONCILE_BATCH {
            return Err(ReconciliationError::BatchTooLarge);
        }

        let mut out = Vec::new(&env);
        for id in pet_ids.iter() {
            let res = Self::reconcile_pet(env.clone(), id)?;
            out.push_back(res);
        }
        Ok(out)
    }

    fn require_initialized(env: &Env) -> Result<(), ReconciliationError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ReconciliationError::NotInitialized);
        }
        Ok(())
    }

    fn validate_bounds(evm_addr: &String, commitment: &String) -> Result<(), ReconciliationError> {
        if evm_addr.is_empty() || evm_addr.len() > MAX_EVM_ADDRESS_LEN {
            return Err(ReconciliationError::InvalidEvmAddressLength);
        }
        if commitment.is_empty() || commitment.len() > MAX_COMMITMENT_HEX_LEN {
            return Err(ReconciliationError::InvalidCommitmentLength);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (
        Env,
        CrossChainReconciliationContractClient<'static>,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, CrossChainReconciliationContract);
        let client = CrossChainReconciliationContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        let relayer = Address::generate(&env);
        client.init(&admin, &relayer);
        (env, client, admin, relayer)
    }

    #[test]
    fn test_shared_json_fixtures_match_contract_reconciliation() {
        let (env, client, _admin, relayer) = setup();
        let raw =
            include_str!("../../../fixtures/cross-chain-reconciliation/pet_state_vectors.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid fixture JSON");
        let vectors = parsed["vectors"].as_array().expect("vectors array");
        assert_eq!(vectors.len(), 3);

        for (idx, vec_item) in vectors.iter().enumerate() {
            let pet_id = vec_item["pet_id"].as_u64().unwrap();
            let s = &vec_item["soroban"];
            let c = &vec_item["celo"];
            let owner = Address::generate(&env);

            client.upsert_soroban_pet(
                &owner,
                &pet_id,
                &String::from_str(&env, s["owner_binding"].as_str().unwrap()),
                &s["active"].as_bool().unwrap(),
                &String::from_str(&env, s["record_commitment"].as_str().unwrap()),
                &(s["record_count"].as_u64().unwrap() as u32),
            );

            client.record_celo_snapshot(
                &relayer,
                &pet_id,
                &c["chain_id"].as_u64().unwrap(),
                &String::from_str(&env, c["owner_evm"].as_str().unwrap()),
                &c["is_active"].as_bool().unwrap(),
                &String::from_str(&env, c["record_commitment"].as_str().unwrap()),
                &(c["record_count"].as_u64().unwrap() as u32),
                &c["block_number"].as_u64().unwrap(),
                &(idx as u64 + 1),
                &5_000,
            );

            let res = client.reconcile_pet(&pet_id);
            let expected_consistent = vec_item["expected_consistent"].as_bool().unwrap();
            assert_eq!(res.is_consistent, expected_consistent);
            let expected_discs = vec_item["expected_discrepancies"].as_array().unwrap();
            assert_eq!(res.discrepancies.len() as usize, expected_discs.len());
        }
    }

    #[test]
    fn test_replay_stale_block_and_expired_snapshot_rejected() {
        let (env, client, _admin, relayer) = setup();
        let owner = Address::generate(&env);
        let evm = String::from_str(&env, "0x1111111111111111111111111111111111111111");
        let comm = String::from_str(
            &env,
            "a3f5c8e1d9b407265184c90123456789abcdef0123456789abcdef0123456789",
        );

        client.upsert_soroban_pet(&owner, &500, &evm, &true, &comm, &1);
        client.record_celo_snapshot(
            &relayer,
            &500,
            &CELO_MAINNET_CHAIN_ID,
            &evm,
            &true,
            &comm,
            &1,
            &100,
            &10,
            &2_000,
        );

        // Replaying the same nonce fails
        let replay = client.try_record_celo_snapshot(
            &relayer,
            &500,
            &CELO_MAINNET_CHAIN_ID,
            &evm,
            &true,
            &comm,
            &1,
            &101,
            &10,
            &2_000,
        );
        assert_eq!(replay, Err(Ok(ReconciliationError::ReplayDetected)));

        // Lower or equal block number fails with StaleBlockNumber
        let stale = client.try_record_celo_snapshot(
            &relayer,
            &500,
            &CELO_MAINNET_CHAIN_ID,
            &evm,
            &true,
            &comm,
            &1,
            &100,
            &11,
            &2_000,
        );
        assert_eq!(stale, Err(Ok(ReconciliationError::StaleBlockNumber)));

        // Expired attestation fails
        let expired = client.try_record_celo_snapshot(
            &relayer,
            &500,
            &CELO_MAINNET_CHAIN_ID,
            &evm,
            &true,
            &comm,
            &1,
            &105,
            &12,
            &999,
        );
        assert_eq!(expired, Err(Ok(ReconciliationError::ExpiredAttestation)));
    }

    #[test]
    fn test_unauthorized_and_boundary_limits() {
        let (env, client, _admin, _relayer) = setup();
        let outsider = Address::generate(&env);
        let evm = String::from_str(&env, "0x1111111111111111111111111111111111111111");
        let comm = String::from_str(&env, "abcd");

        let unauth = client.try_record_celo_snapshot(
            &outsider,
            &1,
            &CELO_MAINNET_CHAIN_ID,
            &evm,
            &true,
            &comm,
            &1,
            &10,
            &1,
            &2_000,
        );
        assert_eq!(unauth, Err(Ok(ReconciliationError::Unauthorized)));

        let mut oversized_batch = Vec::new(&env);
        for i in 0..=MAX_RECONCILE_BATCH {
            oversized_batch.push_back(i as u64);
        }
        let batch_err = client.try_reconcile_batch(&oversized_batch);
        assert_eq!(batch_err, Err(Ok(ReconciliationError::BatchTooLarge)));
    }
}
