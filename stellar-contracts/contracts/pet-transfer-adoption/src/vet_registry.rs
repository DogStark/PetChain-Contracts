// #![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, String, Symbol, Vec,
};

/// ======================================================
/// CONTRACT
/// ======================================================

#[contract]
pub struct VetRegistryContract;

/// ======================================================
/// DATA TYPES
/// ======================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VetStatus {
    Registered,
    Verified,
    Revoked,
}

/// ======================================================
/// STORAGE KEYS
/// ======================================================

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    VetByAddress(Address),
    VetByLicense(String),
    VetCount,
    VetIndex(u64),
    SchemaVersion,
}

/// ======================================================
/// EVENTS
/// ======================================================

const EVT_REGISTERED: Symbol = symbol_short!("reg_vet");
const EVT_VERIFIED: Symbol = symbol_short!("ver_vet");
const EVT_REVOKED: Symbol = symbol_short!("rev_vet");

/// ======================================================
/// ERRORS
/// ======================================================

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    AlreadyInitialized = 0,
    Unauthorized = 1,
    VetAlreadyRegistered = 2,
    VetNotFound = 3,
    LicenseAlreadyUsed = 4,
    VetNotVerified = 5,
    InputTooLong = 6,
    VetAlreadyVerified = 7,
    StaleMigration = 8,
    InvalidMigrationTarget = 9,
}

/// ======================================================
/// INTERNAL HELPERS
/// ======================================================

const MAX_NAME_LEN: u32 = 100;
const MAX_LICENSE_LEN: u32 = 50;
const MAX_SPEC_LEN: u32 = 100;

fn validate_len(env: &Env, s: &String, max: u32) {
    if s.len() > max {
        panic_with_error!(env, ContractError::InputTooLong);
    }
}

fn require_admin(env: &Env) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::Unauthorized));

    admin.require_auth();
}

fn get_vet(env: &Env, vet_address: &Address) -> Vet {
    env.storage()
        .persistent()
        .get(&DataKey::VetByAddress(vet_address.clone()))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::VetNotFound))
}

fn save_vet(env: &Env, vet: &Vet) {
    env.storage()
        .persistent()
        .set(&DataKey::VetByAddress(vet.address.clone()), vet);
}

/// ======================================================
/// CONTRACT IMPLEMENTATION
/// ======================================================

#[contractimpl]
impl VetRegistryContract {
    /// ----------------------------------
    /// INITIALIZATION
    /// ----------------------------------

    pub fn init(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(env, ContractError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// ----------------------------------
    /// REGISTRATION
    /// ----------------------------------

    pub fn register_vet(
        env: Env,
        vet_address: Address,
        name: String,
        license_number: String,
        specialization: String,
    ) {
        vet_address.require_auth();

        validate_len(&env, &name, MAX_NAME_LEN);
        validate_len(&env, &license_number, MAX_LICENSE_LEN);
        validate_len(&env, &specialization, MAX_SPEC_LEN);

        // Prevent duplicate address
        if env
            .storage()
            .persistent()
            .has(&DataKey::VetByAddress(vet_address.clone()))
        {
            panic_with_error!(env, ContractError::VetAlreadyRegistered);
        }

        // Prevent duplicate license
        if env
            .storage()
            .persistent()
            .has(&DataKey::VetByLicense(license_number.clone()))
        {
            panic_with_error!(env, ContractError::LicenseAlreadyUsed);
        }

        let vet = Vet {
            address: vet_address.clone(),
            name,
            license_number: license_number.clone(),
            specialization,
            verified: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::VetByAddress(vet_address.clone()), &vet);

        env.storage()
            .persistent()
            .set(&DataKey::VetByLicense(license_number), &vet_address);

        // Maintain index for pagination
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::VetCount)
            .unwrap_or(0);
        let new_count = count + 1;
        env.storage()
            .persistent()
            .set(&DataKey::VetCount, &new_count);
        env.storage()
            .persistent()
            .set(&DataKey::VetIndex(new_count), &vet_address);

        env.events().publish((EVT_REGISTERED,), vet_address);
    }

    /// ----------------------------------
    /// VERIFICATION (ADMIN)
    /// ----------------------------------

    pub fn verify_vet(env: Env, vet_address: Address) {
        require_admin(&env);

        let mut vet = get_vet(&env, &vet_address);
        if vet.verified {
            panic_with_error!(env, ContractError::VetAlreadyVerified);
        }
        vet.verified = true;
        save_vet(&env, &vet);

        env.events().publish((EVT_VERIFIED,), vet_address);
    }

    pub fn revoke_vet_license(env: Env, vet_address: Address) {
        require_admin(&env);

        let mut vet = get_vet(&env, &vet_address);
        vet.verified = false;
        save_vet(&env, &vet);

        // Remove the license-to-address mapping so the license number can be
        // re-registered after revocation (fixes issue #1019).
        env.storage()
            .persistent()
            .remove(&DataKey::VetByLicense(vet.license_number));

        env.events().publish((EVT_REVOKED,), vet_address);
    }

    /// ----------------------------------
    /// READ HELPERS
    /// ----------------------------------

    pub fn get_vet(env: Env, vet_address: Address) -> Vet {
        get_vet(&env, &vet_address)
    }

    pub fn get_vet_by_license(env: Env, license_number: String) -> Option<Vet> {
        let vet_address: Option<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::VetByLicense(license_number));

        vet_address.and_then(|address| {
            env.storage()
                .persistent()
                .get(&DataKey::VetByAddress(address))
        })
    }

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        let vet = get_vet(&env, &vet_address);
        vet.verified
    }

    pub fn get_vet_count(env: Env) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::VetCount)
            .unwrap_or(0)
    }

    /// List all registered vets with pagination support.
    ///
    /// # Arguments
    /// * `offset` — Number of vets to skip (0-based)
    /// * `limit` — Maximum number of vets to return
    ///
    /// # Returns
    /// `Vec<Vet>` — Paginated list of vets
    pub fn list_vets(env: Env, offset: u64, limit: u32, verified_only: bool) -> Vec<Vet> {
        let count: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::VetCount)
            .unwrap_or(0);

        let mut vets = Vec::new(&env);

        if count == 0 || limit == 0 || offset >= count {
            return vets;
        }

        let start_index = offset + 1; // Indices are 1-based
        let end_index = (offset + limit as u64).min(count);
        let mut matched = 0u64;

        for i in start_index..=end_index {
            if let Some(vet_address) = env
                .storage()
                .persistent()
                .get::<DataKey, Address>(&DataKey::VetIndex(i))
            {
                if let Some(vet) = env
                    .storage()
                    .persistent()
                    .get::<DataKey, Vet>(&DataKey::VetByAddress(vet_address))
                {
                    if verified_only && !vet.verified {
                        continue;
                    }
                    vets.push_back(vet);
                    matched += 1;
                    if matched >= limit as u64 {
                        break;
                    }
                }
            }
        }

        vets
    }

    /// ----------------------------------
    /// SCHEMA MIGRATION  (Issue #1181)
    ///
    /// `get_schema_version` returns the flat `u32` stored under
    /// `DataKey::SchemaVersion`. Absent key -> 0 (pre-versioning: every
    /// `Vet`/`VetByLicense`/`VetIndex` record written before this feature
    /// existed).
    ///
    /// `migrate_schema_version` is:
    ///   - Authorized  — gated by `require_admin`, this contract's existing
    ///                   single-admin `.require_auth()` check (same helper
    ///                   used by `verify_vet`, `revoke_vet_license`, and
    ///                   `transfer_admin`). Called first, before any storage
    ///                   read/write, so an unauthorized caller can neither
    ///                   observe nor mutate migration state.
    ///   - Idempotent  — a replay (`current_version` == already-stored target)
    ///                   or a backward/no-op jump (`target_version <= stored`)
    ///                   panics with `StaleMigration` instead of silently
    ///                   re-running a migration step or corrupting state.
    ///   - Forward-only — `target_version` must strictly exceed `stored`, and
    ///                   the arm that executes it must be a known, reviewed
    ///                   step (unrecognized targets panic with
    ///                   `InvalidMigrationTarget`), preventing silent version
    ///                   skips past undeployed migration logic.
    ///
    /// Threat model:
    ///   An attacker who compromises the admin key could invoke this function
    ///   to advance `SchemaVersion` without the corresponding data actually
    ///   having been migrated, or to attempt to replay a step. The blast
    ///   radius is bounded by two independent properties: (1) the same
    ///   `require_admin` gate already protects every other admin-only
    ///   mutation in this contract (verify/revoke/transfer-admin), so a
    ///   compromised key is already a total-compromise scenario for this
    ///   registry regardless of this function's existence — this function
    ///   adds no new privilege beyond what `transfer_admin` already grants;
    ///   (2) even with the key, the function cannot rewrite history: it can
    ///   only move the version counter strictly forward one recognized step
    ///   at a time, it cannot replay a step already applied (`StaleMigration`),
    ///   and — as this file's migration arm for v0->v1 demonstrates — a step
    ///   that performs no structural change cannot itself destroy or
    ///   resurrect any `Vet` record, since it never touches
    ///   `DataKey::VetByAddress` / `VetByLicense` / `VetIndex` storage at all.
    /// ----------------------------------

    /// Returns the current flat storage-schema version (0 = pre-versioning,
    /// i.e. no migration has ever been run against this contract instance).
    pub fn get_schema_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::SchemaVersion)
            .unwrap_or(0u32)
    }

    /// Advance the storage-schema version from `current_version` to
    /// `target_version`.
    ///
    /// # Behavior
    /// - Panics with `ContractError::StaleMigration` when `current_version`
    ///   does not match the actually-stored version, or when
    ///   `target_version <= stored_version` (covers replay, backward jumps,
    ///   and the exact-equality boundary).
    /// - Panics with `ContractError::InvalidMigrationTarget` when
    ///   `target_version` has no corresponding migration arm.
    /// - Panics via `require_admin` when called by anyone but the
    ///   registered admin.
    ///
    /// # Adding a new migration step
    /// 1. Add a new arm to the `match target_version` block below for the
    ///    next version number.
    /// 2. Keep each arm narrow: touch only the storage that actually needs
    ///    restructuring for that step, and document why existing records
    ///    are (or are not) compatible with the new expected shape.
    pub fn migrate_schema_version(env: Env, current_version: u32, target_version: u32) {
        require_admin(&env);

        let stored: u32 = env
            .storage()
            .instance()
            .get(&DataKey::SchemaVersion)
            .unwrap_or(0u32);

        // Idempotency guard: reject replays and backward/no-op jumps.
        if stored != current_version || target_version <= stored {
            panic_with_error!(&env, ContractError::StaleMigration);
        }

        match target_version {
            1 => {
                // v0 -> v1: no structural changes to Vet/VetByLicense/VetIndex
                // records — every existing record already has every field
                // this version expects. This step exists to establish the
                // version key itself as a baseline for any future structural
                // migration.
            }
            _ => panic_with_error!(&env, ContractError::InvalidMigrationTarget),
        }

        env.storage()
            .instance()
            .set(&DataKey::SchemaVersion, &target_version);
    }
}

/// ======================================================
/// TESTS
/// ======================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, Error, String};

    fn setup() -> (Env, Address, Address, VetRegistryContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, VetRegistryContract);
        let client = VetRegistryContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.init(&admin);
        (env, contract_id, admin, client)
    }

    fn str(env: &Env, s: &str) -> String {
        String::from_str(env, s)
    }

    fn repeat(env: &Env, byte: u8, n: usize) -> String {
        let mut buf = [0u8; 256];
        for b in buf.iter_mut().take(n) {
            *b = byte;
        }
        String::from_bytes(env, &buf[..n])
    }

    // ---- name boundary ----

    #[test]
    fn test_name_at_max_length_accepted() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &repeat(&env, b'a', MAX_NAME_LEN as usize),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
    }

    #[test]
    #[should_panic]
    fn test_name_over_max_length_rejected() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &repeat(&env, b'a', MAX_NAME_LEN as usize + 1),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
    }

    // ---- license_number boundary ----

    #[test]
    fn test_license_at_max_length_accepted() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &str(&env, "Dr. Valid"),
            &repeat(&env, b'L', MAX_LICENSE_LEN as usize),
            &str(&env, "General"),
        );
    }

    #[test]
    #[should_panic]
    fn test_license_over_max_length_rejected() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &str(&env, "Dr. Valid"),
            &repeat(&env, b'L', MAX_LICENSE_LEN as usize + 1),
            &str(&env, "General"),
        );
    }

    // ---- specialization boundary ----

    #[test]
    fn test_specialization_at_max_length_accepted() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &str(&env, "Dr. Valid"),
            &str(&env, "LIC-002"),
            &repeat(&env, b's', MAX_SPEC_LEN as usize),
        );
    }

    #[test]
    #[should_panic]
    fn test_specialization_over_max_length_rejected() {
        let (env, _, _, client) = setup();
        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &str(&env, "Dr. Valid"),
            &str(&env, "LIC-002"),
            &repeat(&env, b's', MAX_SPEC_LEN as usize + 1),
        );
    }

    // ---- error variant ----

    #[test]
    fn test_input_too_long_error_code() {
        assert_eq!(ContractError::InputTooLong as u32, 6);
    }

    // ---- list_vets pagination ----

    #[test]
    fn test_list_vets_empty() {
        let (_, _, _, client) = setup();
        let vets = client.list_vets(&0, &10, &false);
        assert!(vets.is_empty());
    }

    #[test]
    fn test_list_vets_returns_all() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        let vet3 = soroban_sdk::Address::generate(&env);

        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        client.register_vet(
            &vet3,
            &str(&env, "Dr. Three"),
            &str(&env, "LIC-003"),
            &str(&env, "Dermatology"),
        );

        let vets = client.list_vets(&0, &10, &false);
        assert_eq!(vets.len(), 3);
    }

    #[test]
    fn test_list_vets_verified_only_filters_revoked_vets() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        let vet3 = soroban_sdk::Address::generate(&env);

        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        client.register_vet(
            &vet3,
            &str(&env, "Dr. Three"),
            &str(&env, "LIC-003"),
            &str(&env, "Dermatology"),
        );

        client.verify_vet(&vet1);
        client.revoke_vet_license(&vet3);

        let vets = client.list_vets(&0, &10, &true);
        assert_eq!(vets.len(), 1);
        assert_eq!(vets.get(0).unwrap().address, vet1);
    }

    #[test]
    fn test_list_vets_mixed_verified_toggle_preserves_all_vets() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        client.verify_vet(&vet1);

        let all_vets = client.list_vets(&0, &10, &false);
        let filtered = client.list_vets(&0, &10, &true);

        assert_eq!(all_vets.len(), 2);
        assert_eq!(filtered.len(), 1);
        assert!(filtered.get(0).unwrap().verified);
    }

    #[test]
    fn test_list_vets_pagination_limit() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        let vet3 = soroban_sdk::Address::generate(&env);

        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        client.register_vet(
            &vet3,
            &str(&env, "Dr. Three"),
            &str(&env, "LIC-003"),
            &str(&env, "Dermatology"),
        );

        let vets = client.list_vets(&0, &2, &false);
        assert_eq!(vets.len(), 2);
    }

    #[test]
    fn test_list_vets_pagination_offset() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        let vet3 = soroban_sdk::Address::generate(&env);

        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        client.register_vet(
            &vet3,
            &str(&env, "Dr. Three"),
            &str(&env, "LIC-003"),
            &str(&env, "Dermatology"),
        );

        let vets = client.list_vets(&1, &10, &false);
        assert_eq!(vets.len(), 2);
    }

    #[test]
    fn test_list_vets_offset_beyond_count() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );

        let vets = client.list_vets(&5, &10, &false);
        assert!(vets.is_empty());
    }

    #[test]
    fn test_list_vets_zero_limit() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );

        let vets = client.list_vets(&0, &0, &false);
        assert!(vets.is_empty());
    }

    #[test]
    fn test_list_vets_verified_status() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );

        // Verify the vet
        client.verify_vet(&vet1);

        let vets = client.list_vets(&0, &10, &false);
        assert_eq!(vets.len(), 1);
        let retrieved = vets.get(0).unwrap();
        assert!(retrieved.verified);
    }

    #[test]
    fn test_verify_vet_twice_fails_with_vet_already_verified() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );

        client.verify_vet(&vet1);

        let result = client.try_verify_vet(&vet1);
        assert_eq!(
            result,
            Err(Ok(Error::from_contract_error(
                ContractError::VetAlreadyVerified as u32,
            )))
        );
    }

    // ---- get_vet_by_license after revocation ----

    // After revocation the VetByLicense mapping is removed, so
    // get_vet_by_license returns None and the license number is free for
    // re-registration (fixes issue #1019).
    #[test]
    fn test_get_vet_by_license_after_revocation() {
        let (env, _, _, client) = setup();

        let vet = soroban_sdk::Address::generate(&env);
        let license = str(&env, "LIC-REVOKED-001");
        client.register_vet(
            &vet,
            &str(&env, "Dr. Revoked"),
            &license,
            &str(&env, "General"),
        );
        client.verify_vet(&vet);
        client.revoke_vet_license(&vet);

        // License mapping must be cleared — callers can no longer look it up.
        let found = client.get_vet_by_license(&license);
        assert!(found.is_none());
    }

    // ---- #1019: revoked license can be re-registered ----

    #[test]
    fn test_revoked_license_can_be_reregistered_by_same_vet() {
        let (env, _, _, client) = setup();

        let vet = soroban_sdk::Address::generate(&env);
        let license = str(&env, "LIC-REUSE-001");
        client.register_vet(
            &vet,
            &str(&env, "Dr. Same"),
            &license,
            &str(&env, "General"),
        );
        client.revoke_vet_license(&vet);

        // Same vet re-registers with the same license — must succeed.
        client.register_vet(
            &vet,
            &str(&env, "Dr. Same"),
            &license,
            &str(&env, "General"),
        );
        let found = client.get_vet_by_license(&license);
        assert!(found.is_some());
    }

    #[test]
    fn test_revoked_license_can_be_claimed_by_new_vet() {
        let (env, _, _, client) = setup();

        let vet_a = soroban_sdk::Address::generate(&env);
        let vet_b = soroban_sdk::Address::generate(&env);
        let license = str(&env, "LIC-REISSUED-001");

        client.register_vet(&vet_a, &str(&env, "Dr. A"), &license, &str(&env, "General"));
        client.revoke_vet_license(&vet_a);

        // A new vet may now claim the freed license number — must not get LicenseAlreadyUsed.
        client.register_vet(&vet_b, &str(&env, "Dr. B"), &license, &str(&env, "General"));
        let found = client.get_vet_by_license(&license);
        assert!(found.is_some());
        assert_eq!(found.unwrap().address, vet_b);
    }

    // ---- #1180: duplicate license prevention (register, revoke, reissue) ----

    #[test]
    fn test_duplicate_license_on_active_vet_is_rejected() {
        let (env, _, _, client) = setup();
        let vet_a = soroban_sdk::Address::generate(&env);
        let vet_b = soroban_sdk::Address::generate(&env);
        let license = str(&env, "LIC-DUP-001");
        client.register_vet(&vet_a, &str(&env, "Dr. A"), &license, &str(&env, "General"));
        // Same license while vet_a is still active must fail
        let result =
            client.try_register_vet(&vet_b, &str(&env, "Dr. B"), &license, &str(&env, "General"));
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                ContractError::LicenseAlreadyUsed as u32,
            )))
        );
    }

    #[test]
    fn test_license_freed_after_revoke_allows_reissue() {
        let (env, _, _, client) = setup();
        let vet_a = soroban_sdk::Address::generate(&env);
        let vet_b = soroban_sdk::Address::generate(&env);
        let license = str(&env, "LIC-REISSUE-002");
        client.register_vet(&vet_a, &str(&env, "Dr. A"), &license, &str(&env, "General"));
        client.revoke_vet_license(&vet_a);
        // License is now free — new vet may claim it
        client.register_vet(&vet_b, &str(&env, "Dr. B"), &license, &str(&env, "General"));
        let found = client.get_vet_by_license(&license);
        assert!(found.is_some());
        assert_eq!(found.unwrap().address, vet_b);
    }

    // ---- #1023: list_vets with offset >= total vet count returns empty ----

    #[test]
    fn test_list_vets_offset_equal_to_count_returns_empty() {
        let (env, _, _, client) = setup();

        let vet = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet,
            &str(&env, "Dr. Solo"),
            &str(&env, "LIC-SOLO-001"),
            &str(&env, "General"),
        );

        // offset == count (1) — guard must return empty Vec without panicking.
        let vets = client.list_vets(&1, &10, &false);
        assert!(vets.is_empty());
    }

    #[test]
    fn test_list_vets_offset_well_beyond_count_returns_empty() {
        let (env, _, _, client) = setup();

        let vet1 = soroban_sdk::Address::generate(&env);
        let vet2 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );

        // offset >> count — must not panic or attempt an out-of-bounds index.
        let vets = client.list_vets(&1000, &10, &false);
        assert!(vets.is_empty());
    }

    #[test]
    fn test_get_vet_count() {
        let (env, _, _, client) = setup();

        // Initially zero vets
        assert_eq!(client.get_vet_count(), 0);

        // Register one vet
        let vet1 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet1,
            &str(&env, "Dr. One"),
            &str(&env, "LIC-001"),
            &str(&env, "General"),
        );
        assert_eq!(client.get_vet_count(), 1);

        // Register second vet
        let vet2 = soroban_sdk::Address::generate(&env);
        client.register_vet(
            &vet2,
            &str(&env, "Dr. Two"),
            &str(&env, "LIC-002"),
            &str(&env, "Surgery"),
        );
        assert_eq!(client.get_vet_count(), 2);
    }

    // ======================================================
    // #1181: vet-registry schema-version migration tests
    // ======================================================

    // ---- baseline: pre-versioning default ----

    #[test]
    fn test_schema_version_defaults_to_zero_before_migration() {
        let (_, _, _, client) = setup();
        assert_eq!(client.get_schema_version(), 0u32);
    }

    // ---- old-state fixtures: migration preserves pre-existing vet data ----

    // Registers vets under the pre-migration (implicit v0, unversioned)
    // contract state — one verified, one revoked, one plain
    // registered-but-unverified — then runs migrate_schema_version(0, 1)
    // and asserts every vet's data, count, listing, and freed-license
    // behavior (#1019) are byte-for-byte unchanged by the migration.
    #[test]
    fn test_migrate_schema_version_preserves_existing_vet_data() {
        let (env, _, _, client) = setup();

        let verified_vet = soroban_sdk::Address::generate(&env);
        let revoked_vet = soroban_sdk::Address::generate(&env);
        let plain_vet = soroban_sdk::Address::generate(&env);

        let verified_name = str(&env, "Dr. Verified");
        let verified_license = str(&env, "LIC-OLD-VERIFIED");
        let verified_spec = str(&env, "Surgery");

        let revoked_name = str(&env, "Dr. Revoked");
        let revoked_license = str(&env, "LIC-OLD-REVOKED");
        let revoked_spec = str(&env, "Dermatology");

        let plain_name = str(&env, "Dr. Plain");
        let plain_license = str(&env, "LIC-OLD-PLAIN");
        let plain_spec = str(&env, "General");

        // All writes here happen under the old (v0, unversioned) schema —
        // no migration has been run yet.
        client.register_vet(
            &verified_vet,
            &verified_name,
            &verified_license,
            &verified_spec,
        );
        client.verify_vet(&verified_vet);

        client.register_vet(&revoked_vet, &revoked_name, &revoked_license, &revoked_spec);
        client.verify_vet(&revoked_vet);
        client.revoke_vet_license(&revoked_vet);

        client.register_vet(&plain_vet, &plain_name, &plain_license, &plain_spec);

        assert_eq!(client.get_schema_version(), 0u32);
        assert_eq!(client.get_vet_count(), 3);

        // Migrate the old-state fixtures forward.
        client.migrate_schema_version(&0u32, &1u32);

        // Vet count is unchanged.
        assert_eq!(client.get_vet_count(), 3);

        // Full data for each vet is byte-for-byte unchanged via get_vet.
        let verified = client.get_vet(&verified_vet);
        assert_eq!(verified.address, verified_vet);
        assert_eq!(verified.name, verified_name);
        assert_eq!(verified.license_number, verified_license);
        assert_eq!(verified.specialization, verified_spec);
        assert!(verified.verified);

        let revoked = client.get_vet(&revoked_vet);
        assert_eq!(revoked.address, revoked_vet);
        assert_eq!(revoked.name, revoked_name);
        assert_eq!(revoked.license_number, revoked_license);
        assert_eq!(revoked.specialization, revoked_spec);
        assert!(!revoked.verified);

        let plain = client.get_vet(&plain_vet);
        assert_eq!(plain.address, plain_vet);
        assert_eq!(plain.name, plain_name);
        assert_eq!(plain.license_number, plain_license);
        assert_eq!(plain.specialization, plain_spec);
        assert!(!plain.verified);

        // list_vets still returns the right set with the right verified flags.
        let all_vets = client.list_vets(&0, &10, &false);
        assert_eq!(all_vets.len(), 3);

        let verified_only = client.list_vets(&0, &10, &true);
        assert_eq!(verified_only.len(), 1);
        assert_eq!(verified_only.get(0).unwrap().address, verified_vet);

        // The revoked vet's license is still correctly freed (#1019) —
        // migration must not resurrect the VetByLicense mapping.
        let found = client.get_vet_by_license(&revoked_license);
        assert!(found.is_none());

        // Untouched vets' license lookups still resolve correctly.
        assert_eq!(
            client
                .get_vet_by_license(&verified_license)
                .unwrap()
                .address,
            verified_vet
        );
        assert_eq!(
            client.get_vet_by_license(&plain_license).unwrap().address,
            plain_vet
        );
    }

    #[test]
    fn test_migrate_schema_version_updates_version() {
        let (_, _, _, client) = setup();
        client.migrate_schema_version(&0u32, &1u32);
        assert_eq!(client.get_schema_version(), 1u32);
    }

    // ---- idempotency / replay ----

    #[test]
    fn test_migrate_schema_version_replay_fails() {
        let (_, _, _, client) = setup();

        client.migrate_schema_version(&0u32, &1u32);

        let result = client.try_migrate_schema_version(&0u32, &1u32);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                ContractError::StaleMigration as u32,
            )))
        );
    }

    // Exact-boundary case: target_version == stored is rejected too
    // (the `target_version <= stored` check at exact equality).
    #[test]
    fn test_migrate_schema_version_exact_boundary_fails() {
        let (_, _, _, client) = setup();

        client.migrate_schema_version(&0u32, &1u32);

        let result = client.try_migrate_schema_version(&1u32, &1u32);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                ContractError::StaleMigration as u32,
            )))
        );
    }

    #[test]
    fn test_migrate_schema_version_stale_current_version_fails() {
        let (_, _, _, client) = setup();

        // Stored version is actually 0 — passing current_version = 5
        // (out of sync with reality) must be rejected, guarding against
        // racing / out-of-order migration calls.
        let result = client.try_migrate_schema_version(&5u32, &1u32);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                ContractError::StaleMigration as u32,
            )))
        );
    }

    #[test]
    fn test_migrate_schema_version_invalid_target_fails() {
        let (_, _, _, client) = setup();

        let result = client.try_migrate_schema_version(&0u32, &99u32);
        assert_eq!(
            result,
            Err(Ok(soroban_sdk::Error::from_contract_error(
                ContractError::InvalidMigrationTarget as u32,
            )))
        );
    }

    // ---- authorization ----

    // `setup()` calls `env.mock_all_auths()`, which bypasses every
    // `require_auth()` check in the contract — including the admin's, inside
    // `require_admin`. To exercise a genuine "wrong/no caller" rejection we
    // clear all mocked auths with `env.mock_auths(&[])` (the idiom already
    // used elsewhere in this codebase for negative auth tests, see
    // `set_species_adoption_config_requires_admin_auth` in
    // contracts/pet-transfer-adoption/src/test.rs) immediately before the
    // call under test. With no auths mocked, the admin's `.require_auth()`
    // inside `require_admin` cannot be satisfied and the call must fail.
    #[test]
    fn test_migrate_schema_version_without_admin_auth_fails() {
        let (env, _, _, client) = setup();

        env.mock_auths(&[]);

        let result = client.try_migrate_schema_version(&0u32, &1u32);
        assert!(result.is_err());

        // No partial mutation: version must still read as pre-migration.
        env.mock_all_auths();
        assert_eq!(client.get_schema_version(), 0u32);
    }

    // ---- Soroban resource-impact measurement ----

    // Measures CPU/memory cost of migrating a registry with a realistic
    // number of pre-existing vets (20), using the real (non-unlimited)
    // budget so the assertion reflects an actual resource bound rather
    // than a no-op check against an infinite budget. Note this is the
    // *cumulative* budget for the whole `Env` (contract registration, init,
    // the 20 `register_vet` calls, and the migration itself) — the
    // testutils budget API has no per-call isolation — so the numbers below
    // include that fixed setup overhead, not just the migration step alone.
    //
    // Measured on the reference machine when this test was written:
    //   cpu_instruction_cost ≈ 4_407_262
    //   memory_bytes_cost    ≈   992_103
    // The thresholds below are padded roughly 3x-4x over those observed
    // values to absorb SDK/host version drift and machine variance without
    // becoming flaky, while still catching a true regression (e.g. an
    // accidental O(n) rescan of every vet record inside the migration
    // itself, which the v0->v1 arm deliberately does not do).
    #[test]
    fn test_migrate_schema_version_resource_impact_within_bounds() {
        let (env, _, _, client) = setup();

        for i in 0..20u32 {
            let vet = soroban_sdk::Address::generate(&env);
            let name = str(&env, "Dr. Load");
            // Build a unique license number "LIC-LOAD-NN" without pulling in
            // `ToString` (not in scope here) — just write the two decimal
            // digits of `i` (0..20) directly as ASCII bytes.
            let mut buf = *b"LIC-LOAD-00";
            let tens = (i / 10) as u8;
            let ones = (i % 10) as u8;
            buf[9] = b'0' + tens;
            buf[10] = b'0' + ones;
            let license = String::from_bytes(&env, &buf);
            client.register_vet(&vet, &name, &license, &str(&env, "General"));
        }
        assert_eq!(client.get_vet_count(), 20);

        client.migrate_schema_version(&0u32, &1u32);

        let cpu = env.budget().cpu_instruction_cost();
        let mem = env.budget().memory_bytes_cost();

        // Generously padded thresholds — see comment above for observed values.
        assert!(
            cpu < 15_000_000,
            "migrate_schema_version CPU cost regressed: {cpu} instructions"
        );
        assert!(
            mem < 3_000_000,
            "migrate_schema_version memory cost regressed: {mem} bytes"
        );
    }
}
