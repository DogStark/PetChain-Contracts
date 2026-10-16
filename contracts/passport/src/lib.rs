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
    Snapshot(Address, u32),
    SnapshotCount(Address),
    SnapshotRevoked(Address, u32),
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
/// Maximum number of snapshots retained per owner (bounded persistent growth).
const MAX_SNAPSHOTS: u32 = 64;

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

/// An immutable, versioned snapshot of a passport's selected fields.
///
/// Once issued, the snapshot contents are never rewritten. Revocation is
/// recorded separately (see `SnapshotRevocation`) so previously issued data
/// remains intact and auditable.
#[contracttype]
#[derive(Clone)]
pub struct PassportSnapshot {
    pub owner: Address,
    pub version: u32,
    pub ledger: u32,
    pub view: PassportView,
}

/// Revocation record for a snapshot. Stored under a distinct key so the
/// original `PassportSnapshot` is never mutated.
#[contracttype]
#[derive(Clone)]
pub struct SnapshotRevocation {
    pub owner: Address,
    pub version: u32,
    pub reason: u32,
    pub ledger: u32,
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
    SnapshotNotFound = 11,
    SnapshotLimitReached = 12,
    AlreadyRevoked = 13,
    InvalidReason = 14,
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
        let view = Self::get_passport_view(env.clone(), owner.clone())?;
        let digest = Self::digest_view(&env, &view);
        let receipt = VerificationReceipt {
            pet_id: owner.clone(),
            passport_version: settings.version,
            issuing_contract: env.current_contract_address(),
            ledger,
            digest,
        };
        env.events().publish(
            (symbol_short!("receipt"), owner),
            (settings.version, receipt.digest.clone()),
        );
        Ok(receipt)
    }

    /// Issue an immutable snapshot of the owner's current passport view.
    ///
    /// The snapshot version is monotonically increasing per owner and is
    /// derived from the current field-selection version. Once written, the
    /// snapshot contents are never mutated; revocation is recorded separately.
    pub fn issue_snapshot(env: Env, owner: Address) -> Result<u32, Error> {
        owner.require_auth();

        let count: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::SnapshotCount(owner.clone()))
            .unwrap_or(0);
        if count >= MAX_SNAPSHOTS {
            return Err(Error::SnapshotLimitReached);
        }

        let view = Self::get_passport_view(env.clone(), owner.clone())?;
        let version = view.version;

        // Reject re-issuing the same version so versions stay strictly increasing.
        if env
            .storage()
            .persistent()
            .has(&DataKey::Snapshot(owner.clone(), version))
        {
            return Err(Error::StaleVersion);
        }

        let snapshot = PassportSnapshot {
            owner: owner.clone(),
            version,
            ledger: env.ledger().sequence(),
            view,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Snapshot(owner.clone(), version), &snapshot);
        env.storage()
            .persistent()
            .set(&DataKey::SnapshotCount(owner.clone()), &(count + 1));

        env.events().publish(
            (symbol_short!("snap_issue"), owner),
            (version, snapshot.ledger),
        );
        Ok(version)
    }

    /// Read an issued snapshot. Returns the immutable snapshot contents.
    pub fn get_snapshot(
        env: Env,
        owner: Address,
        version: u32,
    ) -> Result<PassportSnapshot, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Snapshot(owner, version))
            .ok_or(Error::SnapshotNotFound)
    }

    /// Revoke a previously issued snapshot with a reason code.
    ///
    /// Revocation never rewrites the snapshot: it writes a separate
    /// `SnapshotRevocation` record keyed by (owner, version).
    pub fn revoke_snapshot(
        env: Env,
        owner: Address,
        version: u32,
        reason: u32,
    ) -> Result<(), Error> {
        owner.require_auth();

        if reason == 0 {
            return Err(Error::InvalidReason);
        }
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Snapshot(owner.clone(), version))
        {
            return Err(Error::SnapshotNotFound);
        }
        if env
            .storage()
            .persistent()
            .has(&DataKey::SnapshotRevoked(owner.clone(), version))
        {
            return Err(Error::AlreadyRevoked);
        }

        let revocation = SnapshotRevocation {
            owner: owner.clone(),
            version,
            reason,
            ledger: env.ledger().sequence(),
        };
        env.storage()
            .persistent()
            .set(&DataKey::SnapshotRevoked(owner.clone(), version), &revocation);

        env.events().publish(
            (symbol_short!("snap_revoke"), owner),
            (version, reason),
        );
        Ok(())
    }

    /// Read the revocation record for a snapshot, if any.
    pub fn get_snapshot_revocation(
        env: Env,
        owner: Address,
        version: u32,
    ) -> Result<SnapshotRevocation, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::SnapshotRevoked(owner, version))
            .ok_or(Error::SnapshotNotFound)
    }

    /// Compute the next monotonic version for an owner's field settings.
    fn next_version(env: &Env, owner: &Address) -> u32 {
        let current: Option<FieldSettings> = env
            .storage()
            .persistent()
            .get(&DataKey::FieldSettings(owner.clone()));
        match current {
            Some(settings) => settings.version + 1,
            None => 1,
        }
    }

    /// Deterministic digest over the selected view fields.
    fn digest_view(env: &Env, view: &PassportView) -> BytesN<32> {
        let mut bytes: Vec<u8> = Vec::new(env);
        bytes.push_back((view.version & 0xff) as u8);
        bytes.push_back(((view.version >> 8) & 0xff) as u8);
        bytes.push_back(((view.version >> 16) & 0xff) as u8);
        bytes.push_back(((view.version >> 24) & 0xff) as u8);
        if let Some(name) = view.name.clone() {
            for b in name.to_bytes().iter() {
                bytes.push_back(b);
            }
        }
        if let Some(species) = view.species.clone() {
            for b in species.to_bytes().iter() {
                bytes.push_back(b);
            }
        }
        if let Some(breed) = view.breed.clone() {
            for b in breed.to_bytes().iter() {
                bytes.push_back(b);
            }
        }
        if let Some(microchip_id) = view.microchip_id.clone() {
            for b in microchip_id.to_bytes().iter() {
                bytes.push_back(b);
            }
        }
        env.crypto().sha256(&bytes).into()
    }
}
