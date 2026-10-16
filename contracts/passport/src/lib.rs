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
            breed: String::from_str(env, "lab"),
            birth_date: 1_600_000_000,
            microchip_id: String::from_str(env, "chip-1"),
            vaccinations,
        }
    }

    #[test]
    fn owner_selects_fields_and_view_hides_others() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        client.set_passport(&owner, &sample_passport(&env, &owner));

        let mut fields = Vec::new(&env);
        fields.push_back(PassportField::Name);
        fields.push_back(PassportField::Species);
        let version = client.set_field_selection(&owner, &fields);
        assert_eq!(version, 1);

        let view = client.get_passport_view(&owner);
        assert_eq!(view.version, 1);
        assert_eq!(view.name, Some(String::from_str(&env, "Rex")));
        assert_eq!(view.species, Some(String::from_str(&env, "dog")));
        // Hidden fields must be absent.
        assert_eq!(view.breed, None);
        assert_eq!(view.birth_date, None);
        assert_eq!(view.microchip_id, None);
        assert_eq!(view.vaccinations, None);
    }

    #[test]
    fn selection_is_versioned() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        client.set_passport(&owner, &sample_passport(&env, &owner));

        let mut first = Vec::new(&env);
        first.push_back(PassportField::Name);
        assert_eq!(client.set_field_selection(&owner, &first), 1);

        let mut second = Vec::new(&env);
        second.push_back(PassportField::Breed);
        assert_eq!(client.set_field_selection(&owner, &second), 2);

        let settings = client.get_field_selection(&owner);
        assert_eq!(settings.version, 2);
        assert_eq!(settings.fields.len(), 1);
    }

    #[test]
    fn unauthorized_selection_is_rejected() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PassportContract);
        let client = PassportContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);

        let owner = Address::generate(&env);
        let mut fields = Vec::new(&env);
        fields.push_back(PassportField::Name);
        // No auth mocked: the call must fail and leave state unchanged.
        let result = client.try_set_field_selection(&owner, &fields);
        assert!(result.is_err());
        assert!(client.try_get_field_selection(&owner).is_err());
    }

    #[test]
    fn boundary_and_invalid_selections_are_rejected() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        client.set_passport(&owner, &sample_passport(&env, &owner));

        // Empty selection.
        let empty: Vec<PassportField> = Vec::new(&env);
        assert!(client.try_set_field_selection(&owner, &empty).is_err());

        // Duplicate field.
        let mut dup = Vec::new(&env);
        dup.push_back(PassportField::Name);
        dup.push_back(PassportField::Name);
        assert!(client.try_set_field_selection(&owner, &dup).is_err());

        // All supported fields is the maximum allowed selection.
        let mut all = Vec::new(&env);
        all.push_back(PassportField::Name);
        all.push_back(PassportField::Species);
        all.push_back(PassportField::Breed);
        all.push_back(PassportField::BirthDate);
        all.push_back(PassportField::MicrochipId);
        all.push_back(PassportField::Vaccinations);
        assert_eq!(client.set_field_selection(&owner, &all), 1);

        // Failed calls above must not have advanced the version.
        assert_eq!(client.get_field_selection(&owner).version, 1);
    }

    #[test]
    fn stale_receipt_version_is_rejected() {
        let (env, client, _admin) = setup();
        let owner = Address::generate(&env);
        client.set_passport(&owner, &sample_passport(&env, &owner));

        let mut fields = Vec::new(&env);
        fields.push_back(PassportField::Name);
        client.set_field_selection(&owner, &fields);

        // Matching version succeeds.
        assert!(client.try_emit_receipt(&owner, &1).is_ok());
        // Replayed/expired version is rejected.
        assert!(client.try_emit_receipt(&owner, &2).is_err());
    }
}
