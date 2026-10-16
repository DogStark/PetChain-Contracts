#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec};

/// Storage keys. Bump `DataKey::Version` when the persisted layout changes.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Version,
    Passport(Address),
    FieldSettings(Address),
}

/// Supported passport fields that an owner may choose to export.
#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum PassportField {
    Name = 0,
    Species = 1,
    Breed = 2,
    BirthDate = 3,
    MicrochipId = 4,
    Vaccinations = 5,
}

/// Current storage layout version.
const STORAGE_VERSION: u32 = 1;
/// Maximum number of selectable fields (bounded iteration / state growth).
const MAX_FIELDS: u32 = 6;

#[contracttype]
#[derive(Clone)]
pub struct Passport {
    pub owner: Address,
    pub name: String,
    pub species: String,
    pub breed: String,
    pub birth_date: u64,
    pub microchip_id: String,
    pub vaccinations: Vec<String>,
}

/// Versioned, owner-controlled selection of exportable passport fields.
#[contracttype]
#[derive(Clone)]
pub struct FieldSettings {
    pub version: u32,
    pub fields: Vec<PassportField>,
}

/// A field-selected projection of a passport. Only selected fields are populated.
#[contracttype]
#[derive(Clone)]
pub struct PassportView {
    pub version: u32,
    pub owner: Address,
    pub name: Option<String>,
    pub species: Option<String>,
    pub breed: Option<String>,
    pub birth_date: Option<u64>,
    pub microchip_id: Option<String>,
    pub vaccinations: Option<Vec<String>>,
}

/// Compact, portable verification receipt. Contains no medical contents — only
/// the identifiers and digest needed to verify an exported passport off-chain.
#[contracttype]
#[derive(Clone)]
pub struct VerificationReceipt {
    pub pet_id: Address,
    pub passport_version: u32,
    pub issuing_contract: Address,
    pub ledger: u32,
    pub digest: BytesN<32>,
}

#[contracterror]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PassportNotFound = 4,
    InvalidField = 5,
    TooManyFields = 6,
    DuplicateField = 7,
    EmptySelection = 8,
    StaleVersion = 9,
    ReceiptExpired = 10,
}

#[contract]
pub struct PassportContract;

#[contractimpl]
impl PassportContract {
    /// One-time initialization. Sets the admin and the storage version.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Version, &STORAGE_VERSION);
        Ok(())
    }

    /// Create or replace a passport. Only the owner may write their passport.
    pub fn set_passport(env: Env, owner: Address, passport: Passport) -> Result<(), Error> {
        owner.require_auth();
        let mut stored = passport;
        stored.owner = owner.clone();
        env.storage().persistent().set(&DataKey::Passport(owner), &stored);
        Ok(())
    }

    /// Owner-controlled selection of exportable fields. Versioned and bounded.
    pub fn set_field_selection(
        env: Env,
        owner: Address,
        fields: Vec<PassportField>,
    ) -> Result<u32, Error> {
        owner.require_auth();

        if fields.is_empty() {
            return Err(Error::EmptySelection);
        }
        if fields.len() > MAX_FIELDS {
            return Err(Error::TooManyFields);
        }

        // Reject duplicates so iteration and state growth stay bounded.
        let mut seen: Vec<u32> = Vec::new(&env);
        for field in fields.iter() {
            let code = field as u32;
            if code >= MAX_FIELDS {
                return Err(Error::InvalidField);
            }
            if seen.contains(&code) {
                return Err(Error::DuplicateField);
            }
            seen.push_back(code);
        }

        let version = Self::next_version(&env, &owner);
        let settings = FieldSettings { version, fields: fields.clone() };
        env.storage()
            .persistent()
            .set(&DataKey::FieldSettings(owner.clone()), &settings);

        env.events().publish(
            (symbol_short!("field_sel"), owner),
            (version, fields),
        );
        Ok(version)
    }

    /// Read the current versioned field selection for an owner.
    pub fn get_field_selection(env: Env, owner: Address) -> Result<FieldSettings, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::FieldSettings(owner))
            .ok_or(Error::PassportNotFound)
    }

    /// Produce a field-selected view. Hidden fields are absent from the response.
    pub fn get_passport_view(env: Env, owner: Address) -> Result<PassportView, Error> {
        let passport: Passport = env
            .storage()
            .persistent()
            .get(&DataKey::Passport(owner.clone()))
            .ok_or(Error::PassportNotFound)?;
        let settings: FieldSettings = env
            .storage()
            .persistent()
            .get(&DataKey::FieldSettings(owner.clone()))
            .ok_or(Error::PassportNotFound)?;

        let mut view = PassportView {
            version: settings.version,
            owner: owner.clone(),
            name: None,
            species: None,
            breed: None,
            birth_date: None,
            microchip_id: None,
            vaccinations: None,
        };

        for field in settings.fields.iter() {
            match field {
                PassportField::Name => view.name = Some(passport.name.clone()),
                PassportField::Species => view.species = Some(passport.species.clone()),
                PassportField::Breed => view.breed = Some(passport.breed.clone()),
                PassportField::BirthDate => view.birth_date = Some(passport.birth_date),
                PassportField::MicrochipId => {
                    view.microchip_id = Some(passport.microchip_id.clone())
                }
                PassportField::Vaccinations => {
                    view.vaccinations = Some(passport.vaccinations.clone())
                }
            }
        }

        env.events().publish(
            (symbol_short!("view_emit"), owner),
            (settings.version, settings.fields.clone()),
        );
        Ok(view)
    }

    /// Emit a verification receipt for a specific settings version (replay/expiry guard).
    pub fn emit_receipt(
        env: Env,
        owner: Address,
        expected_version: u32,
    ) -> Result<(), Error> {
        let settings: FieldSettings = env
            .storage()
            .persistent()
            .get(&DataKey::FieldSettings(owner.clone()))
            .ok_or(Error::PassportNotFound)?;
        if settings.version != expected_version {
            return Err(Error::StaleVersion);
        }
        env.events().publish(
            (symbol_short!("receipt"), owner),
            (settings.version, settings.fields.clone()),
        );
        Ok(())
    }

    /// Produce a compact, portable verification receipt for an exported passport.
    ///
    /// The receipt discloses only the pet ID, passport version, issuing contract,
    /// ledger, and a digest over the selected fields — never the medical contents.
    /// `expected_version` guards against replay of stale exports; `max_ledger`
    /// bounds the receipt's validity window (expiry).
    pub fn get_verification_receipt(
        env: Env,
        owner: Address,
        expected_version: u32,
        max_ledger: u32,
    ) -> Result<VerificationReceipt, Error> {
        let settings: FieldSettings = env
            .storage()
            .persistent()
            .get(&DataKey::FieldSettings(owner.clone()))
            .ok_or(Error::PassportNotFound)?;
        if settings.version != expected_version {
            return Err(Error::StaleVersion);
        }

        let ledger = env.ledger().sequence();
        if ledger > max_ledger {
            return Err(Error::ReceiptExpired);
        }

        // Digest binds the receipt to the exact selected-field set without
        // revealing any field values.
        let mut digest_input = BytesN::from_array(&env, &[0u8; 32]);
        let mut bytes = [0u8; 32];
        bytes[0..4].copy_from_slice(&settings.version.to_be_bytes());
        bytes[4..8].copy_from_slice(&(settings.fields.len() as u32).to_be_bytes());
        for (i, field) in settings.fields.iter().enumerate() {
            if i >= 28 {
                break;
            }
            bytes[8 + i] = field as u8;
        }
        digest_input = BytesN::from_array(&env, &bytes);

        let receipt = VerificationReceipt {
            pet_id: owner.clone(),
            passport_version: settings.version,
            issuing_contract: env.current_contract_address(),
            ledger,
            digest: digest_input,
        };

        env.events().publish(
            (symbol_short!("v_receipt"), owner),
            (receipt.passport_version, receipt.ledger),
        );
        Ok(receipt)
    }

    fn next_version(env: &Env, owner: &Address) -> u32 {
        match env
            .storage()
            .persistent()
            .get::<DataKey, FieldSettings>(&DataKey::FieldSettings(owner.clone()))
        {
            Some(existing) => existing.version.saturating_add(1),
            None => 1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env, String, Vec};

    fn setup() -> (Env, PassportContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PassportContract);
        let client = PassportContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }

    fn sample_passport(env: &Env, owner: &Address) -> Passport {
        let mut vaccinations = Vec::new(env);
        vaccinations.push_back(String::from_str(env, "rabies"));
        Passport {
            owner: owner.clone(),
            name: String::from_str(env, "Rex"),
            species: String::from_str(env, "dog"),
            breed: String::from_str(env, "

/* … truncated 4221 chars — edit only what you need near the top … */
