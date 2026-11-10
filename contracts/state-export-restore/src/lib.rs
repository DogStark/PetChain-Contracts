#![no_std]
#![allow(clippy::too_many_arguments)]

//! Contract state export and restore rehearsal harness (Issue #1505).
//!
//! Exports supported state records (version metadata, admin & vet role
//! authorizations, pet records, and access grants) into a bounded,
//! integrity-tagged `StateExportBundle` and rehearses restoring them into a
//! target instance while preserving authorization and schema version metadata.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 2;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_EXPORTED_PETS: u32 = 32;
pub const MAX_EXPORTED_GRANTS: u32 = 64;
pub const MAX_EXPORTED_VETS: u32 = 32;
pub const MAX_LABEL_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ExportRestoreError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    RehearsalModeDisabled = 4,
    SchemaVersionMismatch = 5,
    BundleExpired = 6,
    BundleAlreadyRestored = 7,
    BundleSizeLimitExceeded = 8,
    ChecksumMismatch = 9,
    LabelTooLong = 10,
    PetNotFound = 11,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionMetadata {
    pub storage_schema_version: u32,
    pub event_schema_version: u32,
    pub contract_semver: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportedPetState {
    pub pet_id: u64,
    pub owner: Address,
    pub name: String,
    pub active: bool,
    pub owner_nonce: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportedAccessGrant {
    pub grant_id: u64,
    pub pet_id: u64,
    pub grantee: Address,
    pub can_write_medical: bool,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateExportBundle {
    pub bundle_id: u64,
    pub version_meta: VersionMetadata,
    pub admin: Address,
    pub verified_vets: Vec<Address>,
    pub pets: Vec<ExportedPetState>,
    pub grants: Vec<ExportedAccessGrant>,
    pub exported_at: u64,
    pub expires_at: u64,
    pub record_checksum: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestoreRehearsalEvent {
    pub version: u32,
    pub bundle_id: u64,
    pub storage_schema_version: u32,
    pub restored_pets: u32,
    pub restored_grants: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    VersionMeta,
    Admin,
    RehearsalEnabled,
    VerifiedVets,
    PetIds,
    Pet(u64),
    GrantIds,
    Grant(u64),
    RestoredBundle(u64),
}

#[contract]
pub struct StateExportRestoreContract;

#[contractimpl]
impl StateExportRestoreContract {
    pub fn init(
        env: Env,
        admin: Address,
        contract_semver: String,
        rehearsal_enabled: bool,
    ) -> Result<(), ExportRestoreError> {
        if env.storage().instance().has(&DataKey::VersionMeta) {
            return Err(ExportRestoreError::AlreadyInitialized);
        }
        admin.require_auth();
        if contract_semver.is_empty() || contract_semver.len() > MAX_LABEL_LEN {
            return Err(ExportRestoreError::LabelTooLong);
        }

        let meta = VersionMetadata {
            storage_schema_version: STORAGE_SCHEMA_VERSION,
            event_schema_version: EVENT_SCHEMA_VERSION,
            contract_semver,
        };
        env.storage().instance().set(&DataKey::VersionMeta, &meta);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RehearsalEnabled, &rehearsal_enabled);
        env.storage()
            .persistent()
            .set(&DataKey::VerifiedVets, &Vec::<Address>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::PetIds, &Vec::<u64>::new(&env));
        env.storage()
            .persistent()
            .set(&DataKey::GrantIds, &Vec::<u64>::new(&env));
        Ok(())
    }

    pub fn add_verified_vet(
        env: Env,
        admin: Address,
        vet: Address,
    ) -> Result<(), ExportRestoreError> {
        Self::require_admin(&env, &admin)?;
        let mut vets: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::VerifiedVets)
            .unwrap_or_else(|| Vec::new(&env));
        if vets.len() >= MAX_EXPORTED_VETS {
            return Err(ExportRestoreError::BundleSizeLimitExceeded);
        }
        if !vets.contains(&vet) {
            vets.push_back(vet);
            env.storage()
                .persistent()
                .set(&DataKey::VerifiedVets, &vets);
        }
        Ok(())
    }

    pub fn put_pet(
        env: Env,
        owner: Address,
        pet_id: u64,
        name: String,
        owner_nonce: u64,
    ) -> Result<(), ExportRestoreError> {
        Self::require_initialized(&env)?;
        owner.require_auth();
        if name.is_empty() || name.len() > MAX_LABEL_LEN {
            return Err(ExportRestoreError::LabelTooLong);
        }

        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PetIds)
            .unwrap_or_else(|| Vec::new(&env));
        if !ids.contains(pet_id) {
            if ids.len() >= MAX_EXPORTED_PETS {
                return Err(ExportRestoreError::BundleSizeLimitExceeded);
            }
            ids.push_back(pet_id);
            env.storage().persistent().set(&DataKey::PetIds, &ids);
        }

        let pet = ExportedPetState {
            pet_id,
            owner,
            name,
            active: true,
            owner_nonce,
        };
        env.storage().persistent().set(&DataKey::Pet(pet_id), &pet);
        Ok(())
    }

    pub fn put_grant(
        env: Env,
        owner: Address,
        grant_id: u64,
        pet_id: u64,
        grantee: Address,
        can_write_medical: bool,
        expires_at: u64,
    ) -> Result<(), ExportRestoreError> {
        Self::require_initialized(&env)?;
        owner.require_auth();

        let pet: ExportedPetState = env
            .storage()
            .persistent()
            .get(&DataKey::Pet(pet_id))
            .ok_or(ExportRestoreError::PetNotFound)?;
        if pet.owner != owner {
            return Err(ExportRestoreError::Unauthorized);
        }

        let mut g_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GrantIds)
            .unwrap_or_else(|| Vec::new(&env));
        if !g_ids.contains(grant_id) {
            if g_ids.len() >= MAX_EXPORTED_GRANTS {
                return Err(ExportRestoreError::BundleSizeLimitExceeded);
            }
            g_ids.push_back(grant_id);
            env.storage().persistent().set(&DataKey::GrantIds, &g_ids);
        }

        let grant = ExportedAccessGrant {
            grant_id,
            pet_id,
            grantee,
            can_write_medical,
            expires_at,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Grant(grant_id), &grant);
        Ok(())
    }

    /// Export all supported state records into a checksummed `StateExportBundle`.
    pub fn export_state_bundle(
        env: Env,
        admin: Address,
        bundle_id: u64,
        expires_at: u64,
    ) -> Result<StateExportBundle, ExportRestoreError> {
        Self::require_admin(&env, &admin)?;
        Self::require_rehearsal_enabled(&env)?;

        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(ExportRestoreError::BundleExpired);
        }

        let version_meta: VersionMetadata = env
            .storage()
            .instance()
            .get(&DataKey::VersionMeta)
            .ok_or(ExportRestoreError::NotInitialized)?;

        let verified_vets: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::VerifiedVets)
            .unwrap_or_else(|| Vec::new(&env));

        let pet_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::PetIds)
            .unwrap_or_else(|| Vec::new(&env));
        let mut pets = Vec::new(&env);
        for pid in pet_ids.iter() {
            if let Some(p) = env
                .storage()
                .persistent()
                .get::<DataKey, ExportedPetState>(&DataKey::Pet(pid))
            {
                pets.push_back(p);
            }
        }

        let grant_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::GrantIds)
            .unwrap_or_else(|| Vec::new(&env));
        let mut grants = Vec::new(&env);
        for gid in grant_ids.iter() {
            if let Some(g) = env
                .storage()
                .persistent()
                .get::<DataKey, ExportedAccessGrant>(&DataKey::Grant(gid))
            {
                grants.push_back(g);
            }
        }

        let record_checksum = Self::compute_checksum(&version_meta, &verified_vets, &pets, &grants);

        Ok(StateExportBundle {
            bundle_id,
            version_meta,
            admin,
            verified_vets,
            pets,
            grants,
            exported_at: now,
            expires_at,
            record_checksum,
        })
    }

    /// Restore a `StateExportBundle` into this contract instance, verifying
    /// rehearsal mode, admin authorization, schema version compatibility,
    /// checksum integrity, expiry, and replay protection.
    pub fn restore_state_bundle(
        env: Env,
        admin: Address,
        bundle: StateExportBundle,
    ) -> Result<(), ExportRestoreError> {
        Self::require_admin(&env, &admin)?;
        Self::require_rehearsal_enabled(&env)?;

        if env.ledger().timestamp() > bundle.expires_at {
            return Err(ExportRestoreError::BundleExpired);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::RestoredBundle(bundle.bundle_id))
        {
            return Err(ExportRestoreError::BundleAlreadyRestored);
        }

        if bundle.version_meta.storage_schema_version != STORAGE_SCHEMA_VERSION
            || bundle.version_meta.event_schema_version != EVENT_SCHEMA_VERSION
        {
            return Err(ExportRestoreError::SchemaVersionMismatch);
        }

        if bundle.verified_vets.len() > MAX_EXPORTED_VETS
            || bundle.pets.len() > MAX_EXPORTED_PETS
            || bundle.grants.len() > MAX_EXPORTED_GRANTS
        {
            return Err(ExportRestoreError::BundleSizeLimitExceeded);
        }

        let expected_checksum = Self::compute_checksum(
            &bundle.version_meta,
            &bundle.verified_vets,
            &bundle.pets,
            &bundle.grants,
        );
        if expected_checksum != bundle.record_checksum {
            return Err(ExportRestoreError::ChecksumMismatch);
        }

        env.storage()
            .instance()
            .set(&DataKey::VersionMeta, &bundle.version_meta);
        env.storage().instance().set(&DataKey::Admin, &bundle.admin);
        env.storage()
            .persistent()
            .set(&DataKey::VerifiedVets, &bundle.verified_vets);

        let mut pet_ids = Vec::new(&env);
        for p in bundle.pets.iter() {
            pet_ids.push_back(p.pet_id);
            env.storage().persistent().set(&DataKey::Pet(p.pet_id), &p);
        }
        env.storage().persistent().set(&DataKey::PetIds, &pet_ids);

        let mut grant_ids = Vec::new(&env);
        for g in bundle.grants.iter() {
            grant_ids.push_back(g.grant_id);
            env.storage()
                .persistent()
                .set(&DataKey::Grant(g.grant_id), &g);
        }
        env.storage()
            .persistent()
            .set(&DataKey::GrantIds, &grant_ids);
        env.storage()
            .persistent()
            .set(&DataKey::RestoredBundle(bundle.bundle_id), &true);

        env.events().publish(
            (symbol_short!("st_rest"), bundle.bundle_id),
            RestoreRehearsalEvent {
                version: EVENT_SCHEMA_VERSION,
                bundle_id: bundle.bundle_id,
                storage_schema_version: bundle.version_meta.storage_schema_version,
                restored_pets: bundle.pets.len(),
                restored_grants: bundle.grants.len(),
            },
        );
        Ok(())
    }

    pub fn get_version_meta(env: Env) -> Option<VersionMetadata> {
        env.storage().instance().get(&DataKey::VersionMeta)
    }

    pub fn get_pet(env: Env, pet_id: u64) -> Option<ExportedPetState> {
        env.storage().persistent().get(&DataKey::Pet(pet_id))
    }

    pub fn is_vet_verified(env: Env, vet: Address) -> bool {
        let vets: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::VerifiedVets)
            .unwrap_or_else(|| Vec::new(&env));
        vets.contains(&vet)
    }

    fn compute_checksum(
        meta: &VersionMetadata,
        vets: &Vec<Address>,
        pets: &Vec<ExportedPetState>,
        grants: &Vec<ExportedAccessGrant>,
    ) -> u64 {
        let mut acc = (meta.storage_schema_version as u64) * 1_000_003
            + (meta.event_schema_version as u64) * 97
            + (vets.len() as u64) * 31;
        for p in pets.iter() {
            acc = acc
                .wrapping_mul(131)
                .wrapping_add(p.pet_id)
                .wrapping_add(p.owner_nonce);
        }
        for g in grants.iter() {
            acc = acc
                .wrapping_mul(137)
                .wrapping_add(g.grant_id)
                .wrapping_add(g.expires_at);
        }
        acc
    }

    fn require_initialized(env: &Env) -> Result<(), ExportRestoreError> {
        if !env.storage().instance().has(&DataKey::VersionMeta) {
            return Err(ExportRestoreError::NotInitialized);
        }
        Ok(())
    }

    fn require_rehearsal_enabled(env: &Env) -> Result<(), ExportRestoreError> {
        let enabled: bool = env
            .storage()
            .instance()
            .get(&DataKey::RehearsalEnabled)
            .unwrap_or(false);
        if !enabled {
            return Err(ExportRestoreError::RehearsalModeDisabled);
        }
        Ok(())
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), ExportRestoreError> {
        Self::require_initialized(env)?;
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ExportRestoreError::NotInitialized)?;
        if admin != *caller {
            return Err(ExportRestoreError::Unauthorized);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    #[test]
    fn test_export_and_restore_rehearsal_preserves_auth_and_version_metadata() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let src_id = env.register_contract(None, StateExportRestoreContract);
        let src = StateExportRestoreContractClient::new(&env, &src_id);

        let dst_id = env.register_contract(None, StateExportRestoreContract);
        let dst = StateExportRestoreContractClient::new(&env, &dst_id);

        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        src.init(&admin, &String::from_str(&env, "1.4.0"), &true);
        src.add_verified_vet(&admin, &vet);
        src.put_pet(&owner, &101, &String::from_str(&env, "Bella"), &7);
        src.put_grant(&owner, &501, &101, &vet, &true, &9_000);

        let bundle = src.export_state_bundle(&admin, &42, &5_000);
        assert_eq!(
            bundle.version_meta.storage_schema_version,
            STORAGE_SCHEMA_VERSION
        );

        dst.init(&admin, &String::from_str(&env, "1.0.0-bootstrap"), &true);
        dst.restore_state_bundle(&admin, &bundle);

        // Verify restored version metadata, pet owner + nonce, and vet verification
        let restored_meta = dst.get_version_meta().unwrap();
        assert_eq!(
            restored_meta.contract_semver,
            String::from_str(&env, "1.4.0")
        );
        assert_eq!(restored_meta.storage_schema_version, STORAGE_SCHEMA_VERSION);

        let restored_pet = dst.get_pet(&101).unwrap();
        assert_eq!(restored_pet.owner, owner);
        assert_eq!(restored_pet.owner_nonce, 7);
        assert!(dst.is_vet_verified(&vet));

        // Restored owner authorization works on destination instance, while non-owner fails
        let outsider = Address::generate(&env);
        let bad_grant = dst.try_put_grant(&outsider, &502, &101, &vet, &false, &9_000);
        assert_eq!(bad_grant, Err(Ok(ExportRestoreError::Unauthorized)));

        // Replaying the same bundle_id is rejected
        let replay = dst.try_restore_state_bundle(&admin, &bundle);
        assert_eq!(replay, Err(Ok(ExportRestoreError::BundleAlreadyRestored)));
    }

    #[test]
    fn test_checksum_tampering_schema_mismatch_and_disabled_rehearsal_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, StateExportRestoreContract);
        let client = StateExportRestoreContractClient::new(&env, &cid);
        let admin = Address::generate(&env);

        client.init(&admin, &String::from_str(&env, "1.4.0"), &true);
        let mut bundle = client.export_state_bundle(&admin, &1, &5_000);

        // Tamper with checksum
        bundle.record_checksum ^= 0xFF;
        let bad_sum = client.try_restore_state_bundle(&admin, &bundle);
        assert_eq!(bad_sum, Err(Ok(ExportRestoreError::ChecksumMismatch)));

        // Incompatible schema version rejected
        let mut bad_ver = client.export_state_bundle(&admin, &2, &5_000);
        bad_ver.version_meta.storage_schema_version = 999;
        let res_ver = client.try_restore_state_bundle(&admin, &bad_ver);
        assert_eq!(res_ver, Err(Ok(ExportRestoreError::SchemaVersionMismatch)));

        // Production instance with rehearsal_enabled = false rejects export/restore
        let prod_id = env.register_contract(None, StateExportRestoreContract);
        let prod = StateExportRestoreContractClient::new(&env, &prod_id);
        prod.init(&admin, &String::from_str(&env, "1.4.0"), &false);
        let disabled = prod.try_export_state_bundle(&admin, &3, &5_000);
        assert_eq!(disabled, Err(Ok(ExportRestoreError::RehearsalModeDisabled)));
    }
}
