#![no_std]
#![allow(clippy::too_many_arguments)]

//! Deterministic chain-mismatch guard and deployment manifest validator (Issue #1508).
//!
//! Ensures contract calls and deployment manifests fail with explicit,
//! deterministic error variants whenever the deployment network name, network
//! passphrase, Celo/EVM chain ID, or contract address metadata do not match.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_NETWORK_NAME_LEN: u32 = 32;
pub const MAX_PASSPHRASE_LEN: u32 = 128;
pub const MAX_ADDRESS_STR_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ChainGuardError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NetworkNameMismatch = 4,
    NetworkPassphraseMismatch = 5,
    ChainIdMismatch = 6,
    ContractAddressMismatch = 7,
    MetadataFieldTooLong = 8,
    EmptyMetadataField = 9,
    ExpiredCallEnvelope = 10,
    ReplayDetected = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainDomainMetadata {
    pub network_name: String,
    pub network_passphrase: String,
    pub chain_id: u64,
    pub contract_address: String,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCallReceipt {
    pub sequence_no: u64,
    pub caller: Address,
    pub chain_id: u64,
    pub network_name: String,
    pub verified_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainVerifiedEvent {
    pub version: u32,
    pub sequence_no: u64,
    pub chain_id: u64,
    pub caller: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    BoundDomain,
    VerifiedCallCount,
    UsedEnvelopeNonce((Address, u64)),
}

#[contract]
pub struct ChainMismatchGuardContract;

#[contractimpl]
impl ChainMismatchGuardContract {
    pub fn init(
        env: Env,
        admin: Address,
        network_name: String,
        network_passphrase: String,
        chain_id: u64,
        contract_address: String,
    ) -> Result<(), ChainGuardError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ChainGuardError::AlreadyInitialized);
        }
        admin.require_auth();

        Self::validate_metadata_bounds(&network_name, &network_passphrase, &contract_address)?;

        let domain = ChainDomainMetadata {
            network_name,
            network_passphrase,
            chain_id,
            contract_address,
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        };

        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::BoundDomain, &domain);
        env.storage()
            .instance()
            .set(&DataKey::VerifiedCallCount, &0u64);
        Ok(())
    }

    /// Validate a deployment manifest or cross-network call envelope against the
    /// contract's bound domain metadata. Fails deterministically on the first
    /// mismatched field without mutating state.
    pub fn assert_domain_match(
        env: Env,
        network_name: String,
        network_passphrase: String,
        chain_id: u64,
        contract_address: String,
    ) -> Result<ChainDomainMetadata, ChainGuardError> {
        Self::validate_metadata_bounds(&network_name, &network_passphrase, &contract_address)?;

        let bound: ChainDomainMetadata = env
            .storage()
            .instance()
            .get(&DataKey::BoundDomain)
            .ok_or(ChainGuardError::NotInitialized)?;

        if bound.network_name != network_name {
            return Err(ChainGuardError::NetworkNameMismatch);
        }
        if bound.network_passphrase != network_passphrase {
            return Err(ChainGuardError::NetworkPassphraseMismatch);
        }
        if bound.chain_id != chain_id {
            return Err(ChainGuardError::ChainIdMismatch);
        }
        if bound.contract_address != contract_address {
            return Err(ChainGuardError::ContractAddressMismatch);
        }

        Ok(bound)
    }

    /// Execute a state-changing domain-guarded call, incrementing `VerifiedCallCount`
    /// only when all chain/manifest metadata, expiry, and replay checks pass.
    pub fn execute_guarded_call(
        env: Env,
        caller: Address,
        network_name: String,
        network_passphrase: String,
        chain_id: u64,
        contract_address: String,
        nonce: u64,
        expires_at: u64,
    ) -> Result<VerifiedCallReceipt, ChainGuardError> {
        caller.require_auth();

        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(ChainGuardError::ExpiredCallEnvelope);
        }

        let nonce_key = DataKey::UsedEnvelopeNonce((caller.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(ChainGuardError::ReplayDetected);
        }

        let bound = Self::assert_domain_match(
            env.clone(),
            network_name,
            network_passphrase,
            chain_id,
            contract_address,
        )?;

        let prev_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VerifiedCallCount)
            .unwrap_or(0);
        let sequence_no = prev_count + 1;

        env.storage()
            .instance()
            .set(&DataKey::VerifiedCallCount, &sequence_no);
        env.storage().persistent().set(&nonce_key, &true);

        env.events().publish(
            (symbol_short!("chn_ok"), sequence_no),
            ChainVerifiedEvent {
                version: EVENT_SCHEMA_VERSION,
                sequence_no,
                chain_id: bound.chain_id,
                caller: caller.clone(),
            },
        );

        Ok(VerifiedCallReceipt {
            sequence_no,
            caller,
            chain_id: bound.chain_id,
            network_name: bound.network_name,
            verified_at: now,
        })
    }

    pub fn get_verified_call_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::VerifiedCallCount)
            .unwrap_or(0)
    }

    fn validate_metadata_bounds(
        network_name: &String,
        network_passphrase: &String,
        contract_address: &String,
    ) -> Result<(), ChainGuardError> {
        if network_name.is_empty() || network_passphrase.is_empty() || contract_address.is_empty() {
            return Err(ChainGuardError::EmptyMetadataField);
        }
        if network_name.len() > MAX_NETWORK_NAME_LEN
            || network_passphrase.len() > MAX_PASSPHRASE_LEN
            || contract_address.len() > MAX_ADDRESS_STR_LEN
        {
            return Err(ChainGuardError::MetadataFieldTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, ChainMismatchGuardContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, ChainMismatchGuardContract);
        let client = ChainMismatchGuardContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(
            &admin,
            &String::from_str(&env, "testnet"),
            &String::from_str(&env, "Test SDF Network ; September 2015"),
            &44_787,
            &String::from_str(&env, "CCAA0001TESTNETPETCHAIN"),
        );
        (env, client, admin)
    }

    #[test]
    fn test_deterministic_chain_mismatch_vectors_fail_with_exact_errors_and_preserve_state() {
        let (env, client, _admin) = setup();
        let caller = Address::generate(&env);
        let raw = include_str!("../fixtures/deployment_manifests.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

        for (idx, item) in parsed["mismatch_cases"]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            let net = String::from_str(&env, item["network"].as_str().unwrap());
            let pass = String::from_str(&env, item["network_passphrase"].as_str().unwrap());
            let chain_id = item["chain_id"].as_u64().unwrap();
            let addr = String::from_str(&env, item["contract_address"].as_str().unwrap());
            let expected = match item["expected_error"].as_str().unwrap() {
                "NetworkNameMismatch" => ChainGuardError::NetworkNameMismatch,
                "NetworkPassphraseMismatch" => ChainGuardError::NetworkPassphraseMismatch,
                "ChainIdMismatch" => ChainGuardError::ChainIdMismatch,
                "ContractAddressMismatch" => ChainGuardError::ContractAddressMismatch,
                other => panic!("unexpected error name: {}", other),
            };

            let res = client.try_execute_guarded_call(
                &caller,
                &net,
                &pass,
                &chain_id,
                &addr,
                &(idx as u64 + 1),
                &5_000,
            );
            assert_eq!(res, Err(Ok(expected)));
            assert_eq!(client.get_verified_call_count(), 0);
        }
    }

    #[test]
    fn test_matching_call_succeeds_and_replay_or_expiry_rejected() {
        let (env, client, _admin) = setup();
        let caller = Address::generate(&env);

        let net = String::from_str(&env, "testnet");
        let pass = String::from_str(&env, "Test SDF Network ; September 2015");
        let addr = String::from_str(&env, "CCAA0001TESTNETPETCHAIN");

        let receipt =
            client.execute_guarded_call(&caller, &net, &pass, &44_787, &addr, &99, &5_000);
        assert_eq!(receipt.sequence_no, 1);
        assert_eq!(client.get_verified_call_count(), 1);

        // Replay with same nonce fails and leaves count at 1
        let replay =
            client.try_execute_guarded_call(&caller, &net, &pass, &44_787, &addr, &99, &5_000);
        assert_eq!(replay, Err(Ok(ChainGuardError::ReplayDetected)));
        assert_eq!(client.get_verified_call_count(), 1);

        // Expired envelope fails
        let expired =
            client.try_execute_guarded_call(&caller, &net, &pass, &44_787, &addr, &100, &500);
        assert_eq!(expired, Err(Ok(ChainGuardError::ExpiredCallEnvelope)));
        assert_eq!(client.get_verified_call_count(), 1);
    }

    #[test]
    fn test_boundary_field_lengths_rejected() {
        let (env, client, _admin) = setup();
        let long_net = String::from_str(&env, &"n".repeat((MAX_NETWORK_NAME_LEN + 1) as usize));
        let pass = String::from_str(&env, "Test SDF Network ; September 2015");
        let addr = String::from_str(&env, "CCAA0001TESTNETPETCHAIN");

        let res = client.try_assert_domain_match(&long_net, &pass, &44_787, &addr);
        assert_eq!(res, Err(Ok(ChainGuardError::MetadataFieldTooLong)));
    }
}
