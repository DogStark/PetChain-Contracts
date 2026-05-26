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
        vet.verified = true;
        save_vet(&env, &vet);

        env.events().publish((EVT_VERIFIED,), vet_address);
    }

    pub fn revoke_vet_license(env: Env, vet_address: Address) {
        require_admin(&env);

        let mut vet = get_vet(&env, &vet_address);
        vet.verified = false;
        save_vet(&env, &vet);

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

    /// List all registered vets with pagination support.
    ///
    /// # Arguments
    /// * `offset` — Number of vets to skip (0-based)
    /// * `limit` — Maximum number of vets to return
    ///
    /// # Returns
    /// `Vec<Vet>` — Paginated list of vets
    pub fn list_vets(env: Env, offset: u64, limit: u32) -> Vec<Vet> {
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
                    vets.push_back(vet);
                }
            }
        }

        vets
    }
}

/// ======================================================
/// TESTS
/// ======================================================

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

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
        for i in 0..n {
            buf[i] = byte;
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
        let vets = client.list_vets(&0, &10);
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

        let vets = client.list_vets(&0, &10);
        assert_eq!(vets.len(), 3);
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

        let vets = client.list_vets(&0, &2);
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

        let vets = client.list_vets(&1, &10);
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

        let vets = client.list_vets(&5, &10);
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

        let vets = client.list_vets(&0, &0);
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

        let vets = client.list_vets(&0, &10);
        assert_eq!(vets.len(), 1);
        let retrieved = vets.get(0).unwrap();
        assert!(retrieved.verified);
    }
}
