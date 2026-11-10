#![no_std]
#![allow(clippy::too_many_arguments)]

//! Contract migration rehearsal command & verification harness (Issue #1503).
//!
//! Loads representative stored state keys across schema versions (`v0`, `v1`,
//! `v2`), executes supported forward migration paths (`v0 -> v1`, `v1 -> v2`,
//! `v2 -> v3`, or multi-hop `v0 -> v3`), and deterministically reports any
//! unmigrated or unrecognized storage keys.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const LATEST_STORAGE_SCHEMA_VERSION: u32 = 3;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_REHEARSAL_KEYS: u32 = 64;
pub const MAX_KEY_NAME_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MigrationRehearsalError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    StaleOrDowngradeMigration = 4,
    UnsupportedTargetVersion = 5,
    TooManyKeys = 6,
    KeyNameTooLong = 7,
    EmptyKeySet = 8,
    ExpiredRehearsalRequest = 9,
    ReplayDetected = 10,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum StorageKeyDomain {
    PetProfile = 1,
    ConsentGrant = 2,
    MedicalRecord = 3,
    InsurancePolicy = 4,
    UnknownLegacyKey = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StoredKeyEntry {
    pub key_name: String,
    pub domain: StorageKeyDomain,
    pub schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationRehearsalReport {
    pub rehearsal_id: u64,
    pub from_version: u32,
    pub to_version: u32,
    pub steps_executed: u32,
    pub migrated_keys_count: u32,
    pub unmigrated_keys: Vec<String>,
    pub all_keys_migrated: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrationRehearsedEvent {
    pub version: u32,
    pub rehearsal_id: u64,
    pub from_version: u32,
    pub to_version: u32,
    pub migrated_keys_count: u32,
    pub unmigrated_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    CurrentSchemaVersion,
    Admin,
    StoredEntries,
    LastReport(u64),
    UsedRehearsalNonce((Address, u64)),
}

#[contract]
pub struct MigrationRehearsalContract;

#[contractimpl]
impl MigrationRehearsalContract {
    pub fn init(
        env: Env,
        admin: Address,
        initial_schema_version: u32,
    ) -> Result<(), MigrationRehearsalError> {
        if env.storage().instance().has(&DataKey::CurrentSchemaVersion) {
            return Err(MigrationRehearsalError::AlreadyInitialized);
        }
        admin.require_auth();
        if initial_schema_version > LATEST_STORAGE_SCHEMA_VERSION {
            return Err(MigrationRehearsalError::UnsupportedTargetVersion);
        }

        env.storage()
            .instance()
            .set(&DataKey::CurrentSchemaVersion, &initial_schema_version);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .persistent()
            .set(&DataKey::StoredEntries, &Vec::<StoredKeyEntry>::new(&env));
        Ok(())
    }

    /// Seed representative stored state entries for migration rehearsal.
    pub fn load_representative_state(
        env: Env,
        admin: Address,
        entries: Vec<StoredKeyEntry>,
    ) -> Result<(), MigrationRehearsalError> {
        Self::require_admin(&env, &admin)?;
        if entries.is_empty() {
            return Err(MigrationRehearsalError::EmptyKeySet);
        }
        if entries.len() > MAX_REHEARSAL_KEYS {
            return Err(MigrationRehearsalError::TooManyKeys);
        }
        for entry in entries.iter() {
            if entry.key_name.is_empty() || entry.key_name.len() > MAX_KEY_NAME_LEN {
                return Err(MigrationRehearsalError::KeyNameTooLong);
            }
        }
        env.storage()
            .persistent()
            .set(&DataKey::StoredEntries, &entries);
        Ok(())
    }

    /// Deterministically run every supported migration step from `CurrentSchemaVersion`
    /// up to `target_version` and report any keys that remain unmigrated.
    pub fn rehearse_migration(
        env: Env,
        admin: Address,
        rehearsal_id: u64,
        target_version: u32,
        nonce: u64,
        expires_at: u64,
    ) -> Result<MigrationRehearsalReport, MigrationRehearsalError> {
        Self::require_admin(&env, &admin)?;

        if expires_at < env.ledger().timestamp() {
            return Err(MigrationRehearsalError::ExpiredRehearsalRequest);
        }

        let nonce_key = DataKey::UsedRehearsalNonce((admin, nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(MigrationRehearsalError::ReplayDetected);
        }

        let from_version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::CurrentSchemaVersion)
            .ok_or(MigrationRehearsalError::NotInitialized)?;

        if target_version <= from_version {
            return Err(MigrationRehearsalError::StaleOrDowngradeMigration);
        }
        if target_version > LATEST_STORAGE_SCHEMA_VERSION {
            return Err(MigrationRehearsalError::UnsupportedTargetVersion);
        }

        let entries: Vec<StoredKeyEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::StoredEntries)
            .unwrap_or_else(|| Vec::new(&env));

        let mut updated_entries = Vec::new(&env);
        let mut migrated_keys_count = 0u32;
        let mut unmigrated_keys = Vec::new(&env);

        for mut entry in entries.iter() {
            if entry.domain == StorageKeyDomain::UnknownLegacyKey {
                unmigrated_keys.push_back(entry.key_name.clone());
                updated_entries.push_back(entry);
                continue;
            }

            // Step through each migration path v -> v+1 up to target_version
            let mut cur = entry.schema_version;
            while cur < target_version {
                cur += 1;
            }
            entry.schema_version = cur;
            migrated_keys_count += 1;
            updated_entries.push_back(entry);
        }

        env.storage()
            .persistent()
            .set(&DataKey::StoredEntries, &updated_entries);
        env.storage()
            .instance()
            .set(&DataKey::CurrentSchemaVersion, &target_version);
        env.storage().persistent().set(&nonce_key, &true);

        let steps_executed = target_version - from_version;
        let all_keys_migrated = unmigrated_keys.is_empty();

        let report = MigrationRehearsalReport {
            rehearsal_id,
            from_version,
            to_version: target_version,
            steps_executed,
            migrated_keys_count,
            unmigrated_keys: unmigrated_keys.clone(),
            all_keys_migrated,
        };
        env.storage()
            .persistent()
            .set(&DataKey::LastReport(rehearsal_id), &report);

        env.events().publish(
            (symbol_short!("mig_reh"), rehearsal_id),
            MigrationRehearsedEvent {
                version: EVENT_SCHEMA_VERSION,
                rehearsal_id,
                from_version,
                to_version: target_version,
                migrated_keys_count,
                unmigrated_count: unmigrated_keys.len(),
            },
        );

        Ok(report)
    }

    pub fn get_schema_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::CurrentSchemaVersion)
            .unwrap_or(0)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), MigrationRehearsalError> {
        if !env.storage().instance().has(&DataKey::CurrentSchemaVersion) {
            return Err(MigrationRehearsalError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(MigrationRehearsalError::NotInitialized)?;
        if admin != *caller {
            return Err(MigrationRehearsalError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup(initial_ver: u32) -> (Env, MigrationRehearsalContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, MigrationRehearsalContract);
        let client = MigrationRehearsalContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin, &initial_ver);
        (env, client, admin)
    }

    #[test]
    fn test_rehearse_full_migration_path_and_report_unmigrated_keys() {
        let (env, client, admin) = setup(0);
        let raw = include_str!("../fixtures/legacy_storage_state.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

        let mut entries = Vec::new(&env);
        entries.push_back(StoredKeyEntry {
            key_name: String::from_str(&env, "pet:101"),
            domain: StorageKeyDomain::PetProfile,
            schema_version: 0,
        });
        entries.push_back(StoredKeyEntry {
            key_name: String::from_str(&env, "consent:101:1"),
            domain: StorageKeyDomain::ConsentGrant,
            schema_version: 0,
        });
        entries.push_back(StoredKeyEntry {
            key_name: String::from_str(&env, "medical_record:501"),
            domain: StorageKeyDomain::MedicalRecord,
            schema_version: 1,
        });
        entries.push_back(StoredKeyEntry {
            key_name: String::from_str(&env, "insurance_policy:801"),
            domain: StorageKeyDomain::InsurancePolicy,
            schema_version: 2,
        });
        entries.push_back(StoredKeyEntry {
            key_name: String::from_str(&env, "legacy_orphan_ experimental:999"),
            domain: StorageKeyDomain::UnknownLegacyKey,
            schema_version: 0,
        });

        client.load_representative_state(&admin, &entries);

        let report = client.rehearse_migration(&admin, &1, &3, &10, &5_000);
        assert_eq!(report.from_version, 0);
        assert_eq!(report.to_version, 3);
        assert_eq!(report.steps_executed, 3);
        assert_eq!(
            report.migrated_keys_count,
            parsed["expected_migrated_count"].as_u64().unwrap() as u32
        );
        assert!(!report.all_keys_migrated);
        assert_eq!(report.unmigrated_keys.len(), 1);
        assert_eq!(client.get_schema_version(), 3);
    }

    #[test]
    fn test_stale_downgrade_unsupported_version_and_unauthorized_rejected() {
        let (env, client, admin) = setup(2);
        let outsider = Address::generate(&env);

        // Stale (same version 2 -> 2) rejected
        let stale = client.try_rehearse_migration(&admin, &1, &2, &1, &5_000);
        assert_eq!(
            stale,
            Err(Ok(MigrationRehearsalError::StaleOrDowngradeMigration))
        );

        // Downgrade (2 -> 1) rejected
        let down = client.try_rehearse_migration(&admin, &2, &1, &2, &5_000);
        assert_eq!(
            down,
            Err(Ok(MigrationRehearsalError::StaleOrDowngradeMigration))
        );

        // Unsupported version (2 -> 99) rejected
        let unsup = client.try_rehearse_migration(&admin, &3, &99, &3, &5_000);
        assert_eq!(
            unsup,
            Err(Ok(MigrationRehearsalError::UnsupportedTargetVersion))
        );

        // Unauthorized caller rejected
        let unauth = client.try_rehearse_migration(&outsider, &4, &3, &4, &5_000);
        assert_eq!(unauth, Err(Ok(MigrationRehearsalError::Unauthorized)));
        assert_eq!(client.get_schema_version(), 2);
    }
}
