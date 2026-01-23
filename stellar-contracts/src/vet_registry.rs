use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, String, Symbol, panic_with_error,
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

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    Unauthorized = 1,
    VetAlreadyRegistered = 2,
    VetNotFound = 3,
    LicenseAlreadyUsed = 4,
    VetNotVerified = 5,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(error: ContractError) -> Self {
        match error {
            ContractError::Unauthorized => soroban_sdk::Error::from((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)),
            ContractError::VetAlreadyRegistered => soroban_sdk::Error::from((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::ExistingValue)),
            ContractError::VetNotFound => soroban_sdk::Error::from((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::MissingValue)),
            ContractError::LicenseAlreadyUsed => soroban_sdk::Error::from((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::ExistingValue)),
            ContractError::VetNotVerified => soroban_sdk::Error::from((soroban_sdk::xdr::ScErrorType::Contract, soroban_sdk::xdr::ScErrorCode::InvalidAction)),
        }
    }
}

/// ======================================================
/// INTERNAL HELPERS
/// ======================================================

fn require_admin(env: &Env) {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("admin not initialized");

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
            panic!("contract already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
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

        env.events().publish(
            (EVT_REGISTERED,),
            vet_address,
        );
    }

    /// ----------------------------------
    /// VERIFICATION (ADMIN)
    /// ----------------------------------

    pub fn verify_vet(env: Env, vet_address: Address) {
        require_admin(&env);

        let mut vet = get_vet(&env, &vet_address);
        vet.verified = true;
        save_vet(&env, &vet);

        env.events().publish(
            (EVT_VERIFIED,),
            vet_address,
        );
    }

    pub fn revoke_vet_license(env: Env, vet_address: Address) {
        require_admin(&env);

        let mut vet = get_vet(&env, &vet_address);
        vet.verified = false;
        save_vet(&env, &vet);

        env.events().publish(
            (EVT_REVOKED,),
            vet_address,
        );
    }

    /// ----------------------------------
    /// READ HELPERS
    /// ----------------------------------

    pub fn get_vet(env: Env, vet_address: Address) -> Vet {
        get_vet(&env, &vet_address)
    }

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        let vet = get_vet(&env, &vet_address);
        vet.verified
    }
}
