#![no_std]
#![allow(clippy::too_many_arguments)]

//! Contract release compatibility report generator & registry (Issue #1509).
//!
//! Compares baseline and candidate contract releases across:
//! 1. Public ABI method signatures (added vs removed/breaking)
//! 2. Storage & event schema versions (monotonic progression vs downgrade)
//! 3. Error-code registry entries (added vs removed or renumbered discriminants)
//! 4. Deployment manifest contract bindings (added, updated, or removed endpoints)

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_METHODS_PER_RELEASE: u32 = 64;
pub const MAX_ERROR_CODES_PER_RELEASE: u32 = 64;
pub const MAX_MANIFEST_ENTRIES: u32 = 16;
pub const MAX_IDENTIFIER_LEN: u32 = 96;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReleaseReportError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    ReleaseAlreadyRegistered = 4,
    ReleaseNotFound = 5,
    TooManyMethods = 6,
    TooManyErrorCodes = 7,
    TooManyManifestEntries = 8,
    IdentifierTooLong = 9,
    DuplicateErrorDiscriminant = 10,
    ExpiredReportRequest = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorCodeSpec {
    pub name: String,
    pub code: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestContractEntry {
    pub contract_name: String,
    pub deployed_address: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseDescriptor {
    pub release_tag: String,
    pub storage_schema_version: u32,
    pub event_schema_version: u32,
    pub abi_methods: Vec<String>,
    pub error_codes: Vec<ErrorCodeSpec>,
    pub manifest_entries: Vec<ManifestContractEntry>,
    pub registered_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseCompatibilityReport {
    pub baseline_tag: String,
    pub candidate_tag: String,
    pub is_backward_compatible: bool,
    pub requires_storage_migration: bool,
    pub storage_version_delta: i32,
    pub added_methods: Vec<String>,
    pub removed_methods: Vec<String>,
    pub added_error_codes: Vec<String>,
    pub breaking_error_changes: Vec<String>,
    pub manifest_deltas: Vec<String>,
    pub storage_schema_version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseReportPublishedEvent {
    pub version: u32,
    pub baseline_tag: String,
    pub candidate_tag: String,
    pub is_backward_compatible: bool,
    pub requires_storage_migration: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    Release(String),
    SavedReport((String, String)),
}

#[contract]
pub struct ReleaseCompatibilityContract;

#[contractimpl]
impl ReleaseCompatibilityContract {
    pub fn init(env: Env, admin: Address) -> Result<(), ReleaseReportError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ReleaseReportError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn register_release(
        env: Env,
        admin: Address,
        release_tag: String,
        storage_schema_version: u32,
        event_schema_version: u32,
        abi_methods: Vec<String>,
        error_codes: Vec<ErrorCodeSpec>,
        manifest_entries: Vec<ManifestContractEntry>,
        expires_at: u64,
    ) -> Result<(), ReleaseReportError> {
        Self::require_admin(&env, &admin)?;

        let now = env.ledger().timestamp();
        if expires_at < now {
            return Err(ReleaseReportError::ExpiredReportRequest);
        }

        Self::validate_string(&release_tag)?;
        if env
            .storage()
            .persistent()
            .has(&DataKey::Release(release_tag.clone()))
        {
            return Err(ReleaseReportError::ReleaseAlreadyRegistered);
        }

        if abi_methods.len() > MAX_METHODS_PER_RELEASE {
            return Err(ReleaseReportError::TooManyMethods);
        }
        for m in abi_methods.iter() {
            Self::validate_string(&m)?;
        }

        if error_codes.len() > MAX_ERROR_CODES_PER_RELEASE {
            return Err(ReleaseReportError::TooManyErrorCodes);
        }
        let mut seen_codes = Vec::new(&env);
        for err in error_codes.iter() {
            Self::validate_string(&err.name)?;
            if seen_codes.contains(err.code) {
                return Err(ReleaseReportError::DuplicateErrorDiscriminant);
            }
            seen_codes.push_back(err.code);
        }

        if manifest_entries.len() > MAX_MANIFEST_ENTRIES {
            return Err(ReleaseReportError::TooManyManifestEntries);
        }
        for entry in manifest_entries.iter() {
            Self::validate_string(&entry.contract_name)?;
            Self::validate_string(&entry.deployed_address)?;
        }

        let descriptor = ReleaseDescriptor {
            release_tag: release_tag.clone(),
            storage_schema_version,
            event_schema_version,
            abi_methods,
            error_codes,
            manifest_entries,
            registered_at: now,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Release(release_tag), &descriptor);
        Ok(())
    }

    /// Generate and persist a deterministic compatibility report between `baseline_tag`
    /// and `candidate_tag`.
    pub fn generate_report(
        env: Env,
        baseline_tag: String,
        candidate_tag: String,
    ) -> Result<ReleaseCompatibilityReport, ReleaseReportError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ReleaseReportError::NotInitialized);
        }

        let base: ReleaseDescriptor = env
            .storage()
            .persistent()
            .get(&DataKey::Release(baseline_tag.clone()))
            .ok_or(ReleaseReportError::ReleaseNotFound)?;
        let cand: ReleaseDescriptor = env
            .storage()
            .persistent()
            .get(&DataKey::Release(candidate_tag.clone()))
            .ok_or(ReleaseReportError::ReleaseNotFound)?;

        let mut added_methods = Vec::new(&env);
        let mut removed_methods = Vec::new(&env);

        for m in cand.abi_methods.iter() {
            if !base.abi_methods.contains(&m) {
                added_methods.push_back(m);
            }
        }
        for m in base.abi_methods.iter() {
            if !cand.abi_methods.contains(&m) {
                removed_methods.push_back(m);
            }
        }

        let mut added_error_codes = Vec::new(&env);
        let mut breaking_error_changes = Vec::new(&env);

        for base_err in base.error_codes.iter() {
            let mut found_same_name = false;
            for cand_err in cand.error_codes.iter() {
                if cand_err.name == base_err.name {
                    found_same_name = true;
                    if cand_err.code != base_err.code {
                        breaking_error_changes.push_back(base_err.name.clone());
                    }
                } else if cand_err.code == base_err.code {
                    // Discriminant reused by a different error variant name
                    breaking_error_changes.push_back(cand_err.name.clone());
                }
            }
            if !found_same_name {
                breaking_error_changes.push_back(base_err.name);
            }
        }

        for cand_err in cand.error_codes.iter() {
            let mut existed = false;
            for base_err in base.error_codes.iter() {
                if base_err.name == cand_err.name {
                    existed = true;
                    break;
                }
            }
            if !existed {
                added_error_codes.push_back(cand_err.name);
            }
        }

        let mut manifest_deltas = Vec::new(&env);
        for c_entry in cand.manifest_entries.iter() {
            let mut matched_contract = false;
            for b_entry in base.manifest_entries.iter() {
                if b_entry.contract_name == c_entry.contract_name {
                    matched_contract = true;
                    if b_entry.deployed_address != c_entry.deployed_address {
                        manifest_deltas.push_back(c_entry.contract_name.clone());
                    }
                }
            }
            if !matched_contract {
                manifest_deltas.push_back(c_entry.contract_name);
            }
        }

        let storage_version_delta =
            (cand.storage_schema_version as i32) - (base.storage_schema_version as i32);
        let event_downgrade = cand.event_schema_version < base.event_schema_version;
        let is_backward_compatible = removed_methods.is_empty()
            && breaking_error_changes.is_empty()
            && storage_version_delta >= 0
            && !event_downgrade;
        let requires_storage_migration = storage_version_delta > 0;

        let report = ReleaseCompatibilityReport {
            baseline_tag: baseline_tag.clone(),
            candidate_tag: candidate_tag.clone(),
            is_backward_compatible,
            requires_storage_migration,
            storage_version_delta,
            added_methods,
            removed_methods,
            added_error_codes,
            breaking_error_changes,
            manifest_deltas,
            storage_schema_version: STORAGE_SCHEMA_VERSION,
        };

        env.storage().persistent().set(
            &DataKey::SavedReport((baseline_tag.clone(), candidate_tag.clone())),
            &report,
        );

        env.events().publish(
            (symbol_short!("rel_rpt"),),
            ReleaseReportPublishedEvent {
                version: EVENT_SCHEMA_VERSION,
                baseline_tag,
                candidate_tag,
                is_backward_compatible,
                requires_storage_migration,
            },
        );

        Ok(report)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), ReleaseReportError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(ReleaseReportError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ReleaseReportError::NotInitialized)?;
        if admin != *caller {
            return Err(ReleaseReportError::Unauthorized);
        }
        Ok(())
    }

    fn validate_string(s: &String) -> Result<(), ReleaseReportError> {
        if s.is_empty() || s.len() > MAX_IDENTIFIER_LEN {
            return Err(ReleaseReportError::IdentifierTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, ReleaseCompatibilityContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, ReleaseCompatibilityContract);
        let client = ReleaseCompatibilityContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_compatible_release_report_from_fixture() {
        let (env, client, admin) = setup();
        let raw = include_str!("../fixtures/release_comparison.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

        for key in ["baseline_release", "compatible_candidate"] {
            let rel = &parsed[key];
            let mut methods = Vec::new(&env);
            for m in rel["abi_methods"].as_array().unwrap() {
                methods.push_back(String::from_str(&env, m.as_str().unwrap()));
            }
            let mut errs = Vec::new(&env);
            for e in rel["error_codes"].as_array().unwrap() {
                errs.push_back(ErrorCodeSpec {
                    name: String::from_str(&env, e["name"].as_str().unwrap()),
                    code: e["code"].as_u64().unwrap() as u32,
                });
            }
            let mut manifests = Vec::new(&env);
            for mf in rel["manifest_contracts"].as_array().unwrap() {
                manifests.push_back(ManifestContractEntry {
                    contract_name: String::from_str(&env, mf["name"].as_str().unwrap()),
                    deployed_address: String::from_str(&env, mf["address"].as_str().unwrap()),
                });
            }
            client.register_release(
                &admin,
                &String::from_str(&env, rel["tag"].as_str().unwrap()),
                &(rel["storage_schema_version"].as_u64().unwrap() as u32),
                &(rel["event_schema_version"].as_u64().unwrap() as u32),
                &methods,
                &errs,
                &manifests,
                &5_000,
            );
        }

        let report = client.generate_report(
            &String::from_str(&env, "v1.2.0"),
            &String::from_str(&env, "v1.3.0"),
        );
        assert!(report.is_backward_compatible);
        assert!(report.requires_storage_migration);
        assert_eq!(report.storage_version_delta, 1);
        assert_eq!(report.added_methods.len(), 1);
        assert_eq!(report.removed_methods.len(), 0);
        assert_eq!(report.added_error_codes.len(), 1);
        assert_eq!(report.breaking_error_changes.len(), 0);
        assert_eq!(report.manifest_deltas.len(), 2);
    }

    #[test]
    fn test_breaking_abi_and_error_renumbering_detected() {
        let (env, client, admin) = setup();

        let mut base_methods = Vec::new(&env);
        base_methods.push_back(String::from_str(&env, "init(Env, Address)"));
        base_methods.push_back(String::from_str(&env, "transfer_pet(Env, u64, Address)"));

        let mut base_errs = Vec::new(&env);
        base_errs.push_back(ErrorCodeSpec {
            name: String::from_str(&env, "Unauthorized"),
            code: 1,
        });

        let manifests = Vec::new(&env);
        client.register_release(
            &admin,
            &String::from_str(&env, "v2.0.0"),
            &2,
            &1,
            &base_methods,
            &base_errs,
            &manifests,
            &5_000,
        );

        // Candidate removes `transfer_pet`, downgrades storage version to 1, and renumbers `Unauthorized` to 9
        let mut cand_methods = Vec::new(&env);
        cand_methods.push_back(String::from_str(&env, "init(Env, Address)"));
        let mut cand_errs = Vec::new(&env);
        cand_errs.push_back(ErrorCodeSpec {
            name: String::from_str(&env, "Unauthorized"),
            code: 9,
        });

        client.register_release(
            &admin,
            &String::from_str(&env, "v2.1.0-bad"),
            &1,
            &1,
            &cand_methods,
            &cand_errs,
            &manifests,
            &5_000,
        );

        let report = client.generate_report(
            &String::from_str(&env, "v2.0.0"),
            &String::from_str(&env, "v2.1.0-bad"),
        );
        assert!(!report.is_backward_compatible);
        assert_eq!(report.removed_methods.len(), 1);
        assert_eq!(report.breaking_error_changes.len(), 1);
        assert_eq!(report.storage_version_delta, -1);
    }

    #[test]
    fn test_duplicate_tag_replay_and_unauthorized_rejected() {
        let (env, client, admin) = setup();
        let outsider = Address::generate(&env);
        let methods = Vec::new(&env);
        let errs = Vec::new(&env);
        let manifests = Vec::new(&env);

        let unauth = client.try_register_release(
            &outsider,
            &String::from_str(&env, "v1.0.0"),
            &1,
            &1,
            &methods,
            &errs,
            &manifests,
            &5_000,
        );
        assert_eq!(unauth, Err(Ok(ReleaseReportError::Unauthorized)));

        client.register_release(
            &admin,
            &String::from_str(&env, "v1.0.0"),
            &1,
            &1,
            &methods,
            &errs,
            &manifests,
            &5_000,
        );
        let dup = client.try_register_release(
            &admin,
            &String::from_str(&env, "v1.0.0"),
            &1,
            &1,
            &methods,
            &errs,
            &manifests,
            &5_000,
        );
        assert_eq!(dup, Err(Ok(ReleaseReportError::ReleaseAlreadyRegistered)));
    }
}
