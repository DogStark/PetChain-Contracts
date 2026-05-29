#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Map, Symbol, Vec,
};

/// Expiry policy: a pending transfer that has not been accepted within
/// this many seconds (~7 days at 5-second ledger close time) may be
/// reclaimed by the original owner via [`PetOwnershipContract::reclaim_transfer`].
/// The constant is expressed in ledger timestamp units (seconds since Unix epoch)
/// so it is independent of ledger sequence numbers.
pub const TRANSFER_EXPIRY_SECONDS: u64 = 7 * 24 * 60 * 60; // 604 800 s

/// Dispute window: after both parties sign, either party has 48 hours to raise
/// a dispute before [`finalize_transfer`] may be called.
pub const DISPUTE_WINDOW_SECONDS: u64 = 48 * 60 * 60; // 172 800 s

#[cfg(test)]
mod test;
mod vet_registry;

/// ======================================================
/// CONTRACT
/// ======================================================

#[contract]
pub struct PetOwnershipContract;

/// ======================================================
/// DATA TYPES
/// ======================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Pet {
    pub pet_id: u64,
    pub current_owner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingTransfer {
    pub pet_id: u64,
    pub from: Address,
    pub to: Address,
    pub initiated_at: u64,
}

/// A transfer that has been accepted by both parties and is now in the
/// 48-hour dispute window before ownership is finalised.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowedTransfer {
    pub pet_id: u64,
    pub from: Address,
    pub to: Address,
    pub escrowed_at: u64,
    pub disputed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipRecord {
    pub owner: Address,
    pub acquired_at: u64,
    pub relinquished_at: Option<u64>,
}

/// Transfer type for chain-of-custody entries.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    Direct,
    Adoption,
    Multisig,
}

/// A single chain-of-custody entry appended on every ownership change.
/// ======================================================
/// ADOPTION WAITING PERIOD (Issue #653)
/// ======================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptionConfig {
    pub waiting_period_days: u32,
}

/// An adoption that has been signed but is still in the waiting period.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingAdoption {
    pub pet_id: u64,
    pub from: Address,
    pub to: Address,
    pub signed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdoptionState {
    Signed,
    Completed,
    Waived,
}

/// A record tracking the full lifecycle of an adoption.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptionRecord {
    pub pet_id: u64,
    pub from: Address,
    pub to: Address,
    pub signed_at: u64,
    pub completed_at: Option<u64>,
    pub state: AdoptionState,
    pub waiver_reason: Option<String>, // set when waived by admin
    pub waived_by: Option<Address>,
}

/// A single chain-of-custody entry appended on every ownership change.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyEntry {
    pub from: Address,
    pub to: Address,
    pub timestamp: u64,
    pub transfer_type: TransferType,
}

/// ======================================================
/// STORAGE KEYS
/// ======================================================

#[contracttype]
enum DataKey {
    Pet(u64),
    PendingTransfer(u64),
    EscrowedTransfer(u64),
    OwnershipHistory(u64),
    OwnerPets(Address),
    CustodyChain(u64), // pet_id -> Vec<CustodyEntry>
    // Adoption waiting period (Issue #653)
    AdoptionConfig,               // global -> AdoptionConfig
    PendingAdoption(u64),         // pet_id -> PendingAdoption
    AdoptionRecord(u64),          // pet_id -> AdoptionRecord
    AdoptionWaitingPeriod(u64),   // pet_id -> waiting_period_days (per-pet override, optional)
    SpeciesAdoptionConfig(String), // species -> waiting_period_days
    JurisdictionAdoptionConfig(String), // jurisdiction -> AdoptionConfig
}

/// ======================================================
/// EVENTS
/// ======================================================

const EVT_TRANSFER_INITIATED: Symbol = symbol_short!("xfer_init");
const EVT_TRANSFER_CANCELLED: Symbol = symbol_short!("xfer_cncl");
const EVT_TRANSFER_ESCROWED: Symbol = symbol_short!("xfer_escr");
const EVT_TRANSFER_FINALIZED: Symbol = symbol_short!("xfer_fin");
const EVT_TRANSFER_DISPUTED: Symbol = symbol_short!("xfer_disp");

/// ======================================================
/// ERRORS
/// ======================================================

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    PetNotFound = 1,
    Unauthorized = 2,
    TransferAlreadyPending = 3,
    NoPendingTransfer = 4,
    InvalidRecipient = 5,
    EmptyOwnershipHistory = 6,
    MissingOwnershipRecord = 7,
    TransferNotExpired = 8,
    StaleCancellation = 9,
    EmptyBatch = 10,
    BatchOwnerMismatch = 11,
    NoEscrowedTransfer = 12,
    DisputeWindowNotElapsed = 13,
    TransferAlreadyDisputed = 14,
    // Adoption waiting period errors (Issue #653)
    NoPendingAdoption = 15,
    WaitingPeriodNotElapsed = 16,
    AdoptionAlreadyCompleted = 17,
    AdoptionNotConfigurable = 18,
    InvalidWaitingPeriod = 19,
    AdoptionConfigNotFound = 20,
}

/// ======================================================
/// INTERNAL HELPERS
/// ======================================================

fn get_pet(env: &Env, pet_id: u64) -> Pet {
    env.storage()
        .persistent()
        .get(&DataKey::Pet(pet_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::PetNotFound))
}

fn save_pet(env: &Env, pet: &Pet) {
    env.storage()
        .persistent()
        .set(&DataKey::Pet(pet.pet_id), pet);
}

fn get_history(env: &Env, pet_id: u64) -> Vec<OwnershipRecord> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnershipHistory(pet_id))
        .unwrap_or_else(|| Vec::new(env))
}

fn save_history(env: &Env, pet_id: u64, history: &Vec<OwnershipRecord>) {
    env.storage()
        .persistent()
        .set(&DataKey::OwnershipHistory(pet_id), history);
}

fn append_custody_entry(env: &Env, pet_id: u64, from: Address, to: Address, transfer_type: TransferType) {
    let mut chain: Vec<CustodyEntry> = env
        .storage()
        .persistent()
        .get(&DataKey::CustodyChain(pet_id))
        .unwrap_or_else(|| Vec::new(env));
    chain.push_back(CustodyEntry {
        from,
        to,
        timestamp: env.ledger().timestamp(),
        transfer_type,
    });
    env.storage()
        .persistent()
        .set(&DataKey::CustodyChain(pet_id), &chain);
}

fn get_owner_pet_ids(env: &Env, owner: &Address) -> Vec<u64> {
    env.storage()
        .persistent()
        .get(&DataKey::OwnerPets(owner.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

fn save_owner_pet_ids(env: &Env, owner: &Address, pet_ids: &Vec<u64>) {
    env.storage()
        .persistent()
        .set(&DataKey::OwnerPets(owner.clone()), pet_ids);
}

fn add_pet_to_owner(env: &Env, owner: &Address, pet_id: u64) {
    let mut pet_ids = get_owner_pet_ids(env, owner);
    for existing_pet_id in pet_ids.iter() {
        if existing_pet_id == pet_id {
            return;
        }
    }

    pet_ids.push_back(pet_id);
    save_owner_pet_ids(env, owner, &pet_ids);
}

fn remove_pet_from_owner(env: &Env, owner: &Address, pet_id: u64) {
    let pet_ids = get_owner_pet_ids(env, owner);
    let mut updated_pet_ids = Vec::new(env);

    for existing_pet_id in pet_ids.iter() {
        if existing_pet_id != pet_id {
            updated_pet_ids.push_back(existing_pet_id);
        }
    }

    save_owner_pet_ids(env, owner, &updated_pet_ids);
}

/// ======================================================
/// CONTRACT IMPLEMENTATION
/// ======================================================

#[contractimpl]
impl PetOwnershipContract {
    /// ----------------------------------
    /// ADOPTION WAITING PERIOD (Issue #653)
    /// ----------------------------------

    /// Set the global adoption waiting period (in days).
    /// Only configurable if no default config has been set yet.
    pub fn set_adoption_config(env: Env, waiting_period_days: u32) {
        if env.storage().persistent().has(&DataKey::AdoptionConfig) {
            panic_with_error!(&env, ContractError::AdoptionNotConfigurable);
        }
        let config = AdoptionConfig { waiting_period_days };
        env.storage().persistent().set(&DataKey::AdoptionConfig, &config);
    }

    /// Get the current adoption config.
    pub fn get_adoption_config(env: Env) -> AdoptionConfig {
        env.storage().persistent().get(&DataKey::AdoptionConfig)
            .unwrap_or(AdoptionConfig { waiting_period_days: 0 })
    }

    /// Set a per-species adoption waiting period override.
    pub fn set_species_adoption_config(env: Env, species: String, waiting_period_days: u32) {
        env.storage().persistent().set(&DataKey::SpeciesAdoptionConfig(species), &waiting_period_days);
    }

    /// Sign an adoption agreement, entering the waiting period.
    pub fn sign_adoption(env: Env, pet_id: u64, to: Address) {
        let pet = get_pet(&env, pet_id);
        pet.current_owner.require_auth();

        if env.storage().persistent().has(&DataKey::PendingAdoption(pet_id)) {
            panic_with_error!(&env, ContractError::TransferAlreadyPending);
        }
        if env.storage().persistent().has(&DataKey::PendingTransfer(pet_id)) {
            panic_with_error!(&env, ContractError::TransferAlreadyPending);
        }

        let now = env.ledger().timestamp();
        let pending = PendingAdoption {
            pet_id,
            from: pet.current_owner.clone(),
            to: to.clone(),
            signed_at: now,
        };
        let record = AdoptionRecord {
            pet_id,
            from: pet.current_owner.clone(),
            to: to.clone(),
            signed_at: now,
            completed_at: None,
            state: AdoptionState::Signed,
            waiver_reason: None,
            waived_by: None,
        };

        env.storage().persistent().set(&DataKey::PendingAdoption(pet_id), &pending);
        env.storage().persistent().set(&DataKey::AdoptionRecord(pet_id), &record);

        env.events().publish(
            (Symbol::new(&env, "adoption_sign"), pet_id),
            (pet.current_owner, to, now),
        );
    }

    /// Complete the adoption after the waiting period has elapsed.
    pub fn complete_adoption(env: Env, pet_id: u64) {
        let pending: PendingAdoption = env.storage().persistent()
            .get(&DataKey::PendingAdoption(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPendingAdoption));

        let now = env.ledger().timestamp();
        let config: Option<AdoptionConfig> = env.storage().persistent().get(&DataKey::AdoptionConfig);
        let wp_seconds = match config {
            Some(c) => (c.waiting_period_days as u64).saturating_mul(86400),
            None => 0,
        };

        if now.saturating_sub(pending.signed_at) < wp_seconds {
            panic_with_error!(&env, ContractError::WaitingPeriodNotElapsed);
        }

        let mut record: AdoptionRecord = env.storage().persistent()
            .get(&DataKey::AdoptionRecord(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPendingAdoption));
        if record.state != AdoptionState::Signed {
            panic_with_error!(&env, ContractError::AdoptionAlreadyCompleted);
        }

        let mut pet = get_pet(&env, pet_id);
        if pet.current_owner != pending.from {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let mut history = get_history(&env, pet_id);
        if history.len() == 0 {
            panic_with_error!(&env, ContractError::EmptyOwnershipHistory);
        }
        let last = history.len() - 1;
        let mut prev = history.get(last)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MissingOwnershipRecord));
        prev.relinquished_at = Some(now);
        history.set(last, prev);
        history.push_back(OwnershipRecord {
            owner: pending.to.clone(),
            acquired_at: now,
            relinquished_at: None,
        });

        remove_pet_from_owner(&env, &pending.from, pet_id);
        add_pet_to_owner(&env, &pending.to, pet_id);
        pet.current_owner = pending.to.clone();
        record.state = AdoptionState::Completed;
        record.completed_at = Some(now);

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);
        env.storage().persistent().remove(&DataKey::PendingAdoption(pet_id));
        env.storage().persistent().set(&DataKey::AdoptionRecord(pet_id), &record);
        append_custody_entry(&env, pet_id, pending.from.clone(), pending.to.clone(), TransferType::Adoption);

        env.events().publish(
            (Symbol::new(&env, "adoption_cmpl"), pet_id),
            (pending.from, pending.to, now),
        );
    }

    /// Allow a multisig admin to waive the waiting period for a specific adoption.
    pub fn waive_waiting_period(env: Env, pet_id: u64, admin: Address, reason: String) {
        admin.require_auth();
        let pending: PendingAdoption = env.storage().persistent()
            .get(&DataKey::PendingAdoption(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPendingAdoption));

        let mut record: AdoptionRecord = env.storage().persistent()
            .get(&DataKey::AdoptionRecord(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPendingAdoption));
        if record.state != AdoptionState::Signed {
            panic_with_error!(&env, ContractError::AdoptionAlreadyCompleted);
        }

        let now = env.ledger().timestamp();
        let mut pet = get_pet(&env, pet_id);
        if pet.current_owner != pending.from {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let mut history = get_history(&env, pet_id);
        if history.len() == 0 {
            panic_with_error!(&env, ContractError::EmptyOwnershipHistory);
        }
        let last = history.len() - 1;
        let mut prev = history.get(last)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MissingOwnershipRecord));
        prev.relinquished_at = Some(now);
        history.set(last, prev);
        history.push_back(OwnershipRecord {
            owner: pending.to.clone(),
            acquired_at: now,
            relinquished_at: None,
        });

        remove_pet_from_owner(&env, &pending.from, pet_id);
        add_pet_to_owner(&env, &pending.to, pet_id);
        pet.current_owner = pending.to.clone();

        record.state = AdoptionState::Waived;
        record.completed_at = Some(now);
        record.waiver_reason = Some(reason.clone());
        record.waived_by = Some(admin.clone());

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);
        env.storage().persistent().remove(&DataKey::PendingAdoption(pet_id));
        env.storage().persistent().set(&DataKey::AdoptionRecord(pet_id), &record);
        append_custody_entry(&env, pet_id, pending.from.clone(), pending.to.clone(), TransferType::Adoption);

        env.events().publish(
            (Symbol::new(&env, "adoption_wavd"), pet_id),
            (admin, pending.from, pending.to, reason, now),
        );
    }

    /// Get the adoption record for a pet.
    pub fn get_adoption_record(env: Env, pet_id: u64) -> Option<AdoptionRecord> {
        env.storage().persistent().get(&DataKey::AdoptionRecord(pet_id))
    }

    /// Get the pending adoption for a pet.
    pub fn get_pending_adoption(env: Env, pet_id: u64) -> Option<PendingAdoption> {
        env.storage().persistent().get(&DataKey::PendingAdoption(pet_id))
    }

    /// ----------------------------------
    /// CREATE PET (bootstrap)
    /// ----------------------------------

    pub fn create_pet(env: Env, pet_id: u64, owner: Address) {
        owner.require_auth();

        let pet = Pet {
            pet_id,
            current_owner: owner.clone(),
        };

        let mut history = Vec::new(&env);
        history.push_back(OwnershipRecord {
            owner: owner.clone(),
            acquired_at: env.ledger().timestamp(),
            relinquished_at: None,
        });

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);
        add_pet_to_owner(&env, &owner, pet_id);
    }

    /// ----------------------------------
    /// INITIATE TRANSFER
    /// ----------------------------------

    pub fn initiate_transfer(env: Env, pet_id: u64, to: Address) {
        let pet = get_pet(&env, pet_id);
        pet.current_owner.require_auth();

        if env
            .storage()
            .persistent()
            .has(&DataKey::PendingTransfer(pet_id))
        {
            panic_with_error!(env, ContractError::TransferAlreadyPending);
        }

        let transfer = PendingTransfer {
            pet_id,
            from: pet.current_owner.clone(),
            to: to.clone(),
            initiated_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::PendingTransfer(pet_id), &transfer);

        env.events()
            .publish((EVT_TRANSFER_INITIATED, pet_id), (pet.current_owner, to));
    }

    /// ----------------------------------
    /// ACCEPT TRANSFER → ESCROW
    /// ----------------------------------
    ///
    /// The recipient accepts the transfer. Ownership does **not** change yet;
    /// the transfer enters an [`EscrowedTransfer`] state and a 48-hour dispute
    /// window begins. Call [`finalize_transfer`] after the window to complete
    /// the ownership change, or [`raise_dispute`] to block finalization.
    pub fn accept_transfer(env: Env, pet_id: u64) {
        let transfer: PendingTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::PendingTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

        transfer.to.require_auth();

        let pet = get_pet(&env, pet_id);
        if pet.current_owner != transfer.from {
            panic_with_error!(env, ContractError::Unauthorized);
        }

        let escrowed = EscrowedTransfer {
            pet_id,
            from: transfer.from.clone(),
            to: transfer.to.clone(),
            escrowed_at: env.ledger().timestamp(),
            disputed: false,
        };

        env.storage()
            .persistent()
            .remove(&DataKey::PendingTransfer(pet_id));
        env.storage()
            .persistent()
            .set(&DataKey::EscrowedTransfer(pet_id), &escrowed);

        env.events().publish(
            (EVT_TRANSFER_ESCROWED, pet_id),
            (transfer.from, transfer.to),
        );
    }

    /// ----------------------------------
    /// FINALIZE TRANSFER
    /// ----------------------------------
    ///
    /// Completes the ownership transfer after the 48-hour dispute window has
    /// elapsed. Either party may call this. Panics if the window has not yet
    /// elapsed or if the transfer has been disputed.
    pub fn finalize_transfer(env: Env, pet_id: u64) {
        let escrowed: EscrowedTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowedTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoEscrowedTransfer));

        if escrowed.disputed {
            panic_with_error!(env, ContractError::TransferAlreadyDisputed);
        }

        let now = env.ledger().timestamp();
        if now.saturating_sub(escrowed.escrowed_at) < DISPUTE_WINDOW_SECONDS {
            panic_with_error!(env, ContractError::DisputeWindowNotElapsed);
        }

        let mut pet = get_pet(&env, pet_id);
        if pet.current_owner != escrowed.from {
            panic_with_error!(env, ContractError::Unauthorized);
        }

        // Update ownership history
        let mut history = get_history(&env, pet_id);
        if history.len() == 0 {
            panic_with_error!(&env, ContractError::EmptyOwnershipHistory);
        }
        let last = history.len() - 1;
        let mut prev = history
            .get(last)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::MissingOwnershipRecord));
        prev.relinquished_at = Some(now);
        history.set(last, prev);
        history.push_back(OwnershipRecord {
            owner: escrowed.to.clone(),
            acquired_at: now,
            relinquished_at: None,
        });

        remove_pet_from_owner(&env, &escrowed.from, pet_id);
        add_pet_to_owner(&env, &escrowed.to, pet_id);
        pet.current_owner = escrowed.to.clone();

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);
        env.storage()
            .persistent()
            .remove(&DataKey::EscrowedTransfer(pet_id));

        append_custody_entry(&env, pet_id, escrowed.from.clone(), escrowed.to.clone(), TransferType::Direct);

        env.events().publish(
            (EVT_TRANSFER_FINALIZED, pet_id),
            (escrowed.from, escrowed.to),
        );
    }

    /// ----------------------------------
    /// RAISE DISPUTE
    /// ----------------------------------
    ///
    /// Either the sender or recipient may raise a dispute during the 48-hour
    /// window. A disputed transfer cannot be finalized; it must be resolved
    /// through the main contract's dispute module.
    ///
    /// `caller` must be either the `from` or `to` address of the escrowed transfer.
    pub fn raise_dispute(env: Env, pet_id: u64, caller: Address) {
        let mut escrowed: EscrowedTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::EscrowedTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoEscrowedTransfer));

        if escrowed.disputed {
            panic_with_error!(env, ContractError::TransferAlreadyDisputed);
        }

        let now = env.ledger().timestamp();
        if now.saturating_sub(escrowed.escrowed_at) >= DISPUTE_WINDOW_SECONDS {
            panic_with_error!(env, ContractError::DisputeWindowNotElapsed);
        }

        // Require auth from the caller and verify they are a party to the transfer.
        caller.require_auth();
        if caller != escrowed.from && caller != escrowed.to {
            panic_with_error!(env, ContractError::Unauthorized);
        }

        escrowed.disputed = true;
        env.storage()
            .persistent()
            .set(&DataKey::EscrowedTransfer(pet_id), &escrowed);

        env.events().publish(
            (EVT_TRANSFER_DISPUTED, pet_id),
            (escrowed.from, escrowed.to),
        );
    }

    /// ----------------------------------
    /// CANCEL TRANSFER
    /// ----------------------------------

    pub fn cancel_transfer(env: Env, pet_id: u64) {
        let transfer: PendingTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::PendingTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

        transfer.from.require_auth();

        let pet = get_pet(&env, pet_id);
        if pet.current_owner != transfer.from {
            panic_with_error!(env, ContractError::StaleCancellation);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::PendingTransfer(pet_id));

        env.events().publish(
            (EVT_TRANSFER_CANCELLED, pet_id),
            (transfer.from, transfer.to),
        );
    }

    /// ----------------------------------
    /// RECLAIM EXPIRED TRANSFER
    /// ----------------------------------

    /// Allows the original owner to cancel a pending transfer that has been
    /// outstanding for longer than [`TRANSFER_EXPIRY_SECONDS`].
    ///
    /// # Expiry policy
    /// A `PendingTransfer` records `initiated_at` (ledger timestamp in seconds).
    /// If `current_timestamp - initiated_at >= TRANSFER_EXPIRY_SECONDS` the
    /// transfer is considered stale and the owner may call this function to
    /// clean it up without requiring the recipient's cooperation.
    ///
    /// # Errors
    /// - [`ContractError::NoPendingTransfer`] – no transfer exists for this pet.
    /// - [`ContractError::Unauthorized`] – caller is not the original sender.
    /// - [`ContractError::TransferNotExpired`] – the expiry window has not elapsed;
    ///   use [`cancel_transfer`] instead if you want to cancel before expiry.
    pub fn reclaim_transfer(env: Env, pet_id: u64) {
        let transfer: PendingTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::PendingTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

        transfer.from.require_auth();

        let now = env.ledger().timestamp();
        if now.saturating_sub(transfer.initiated_at) < TRANSFER_EXPIRY_SECONDS {
            panic_with_error!(env, ContractError::TransferNotExpired);
        }

        env.storage()
            .persistent()
            .remove(&DataKey::PendingTransfer(pet_id));

        env.events().publish(
            (EVT_TRANSFER_CANCELLED, pet_id),
            (transfer.from, transfer.to),
        );
    }

    /// ----------------------------------
    /// BATCH INITIATE TRANSFER
    /// ----------------------------------

    /// Atomically initiates ownership transfers for multiple pets to the same recipient.
    ///
    /// All pets must be owned by a single address. The function validates every pet
    /// before writing anything, so any error rolls back the entire batch cleanly.
    ///
    /// # Errors
    /// - [`ContractError::EmptyBatch`] – `pet_ids` is empty.
    /// - [`ContractError::PetNotFound`] – any pet in the batch does not exist.
    /// - [`ContractError::BatchOwnerMismatch`] – not all pets share the same owner.
    /// - [`ContractError::TransferAlreadyPending`] – any pet already has a pending transfer.
    pub fn batch_initiate_transfer(env: Env, pet_ids: Vec<u64>, to: Address) {
        if pet_ids.is_empty() {
            panic_with_error!(env, ContractError::EmptyBatch);
        }

        // Phase 1: read-only validation — discover owner, ensure all pets are eligible.
        let mut expected_owner: Option<Address> = None;
        for pet_id in pet_ids.iter() {
            let pet = get_pet(&env, pet_id);

            match expected_owner {
                None => expected_owner = Some(pet.current_owner.clone()),
                Some(ref owner) => {
                    if &pet.current_owner != owner {
                        panic_with_error!(env, ContractError::BatchOwnerMismatch);
                    }
                }
            }

            if env
                .storage()
                .persistent()
                .has(&DataKey::PendingTransfer(pet_id))
            {
                panic_with_error!(env, ContractError::TransferAlreadyPending);
            }
        }

        // Safety: pet_ids is non-empty (guarded above), so expected_owner is always Some.
        let owner = expected_owner.unwrap_or_else(|| panic_with_error!(env, ContractError::EmptyBatch));

        // Phase 2: authenticate the single owner once for the entire batch.
        owner.require_auth();

        // Phase 3: write all pending transfers.
        let now = env.ledger().timestamp();
        for pet_id in pet_ids.iter() {
            let transfer = PendingTransfer {
                pet_id,
                from: owner.clone(),
                to: to.clone(),
                initiated_at: now,
            };

            env.storage()
                .persistent()
                .set(&DataKey::PendingTransfer(pet_id), &transfer);

            env.events()
                .publish((EVT_TRANSFER_INITIATED, pet_id), (owner.clone(), to.clone()));
        }
    }

    /// ----------------------------------
    /// READ HELPERS
    /// ----------------------------------

    pub fn get_current_owner(env: Env, pet_id: u64) -> Address {
        get_pet(&env, pet_id).current_owner
    }

    pub fn get_ownership_history(env: Env, pet_id: u64) -> Vec<OwnershipRecord> {
        get_history(&env, pet_id)
    }

    pub fn get_owner_pets(env: Env, owner: Address) -> Vec<u64> {
        get_owner_pet_ids(&env, &owner)
    }

    /// Return the full chain-of-custody log for `pet_id` in chronological order.
    pub fn get_custody_chain(env: Env, pet_id: u64) -> Vec<CustodyEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::CustodyChain(pet_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn has_pending_transfer(env: Env, pet_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::PendingTransfer(pet_id))
    }

    /// Returns the [`PendingTransfer`] for `pet_id`, or `None` if none exists.
    pub fn get_pending_transfer(env: Env, pet_id: u64) -> Option<PendingTransfer> {
        env.storage()
            .persistent()
            .get(&DataKey::PendingTransfer(pet_id))
    }

    /// Returns the [`EscrowedTransfer`] for `pet_id`, or `None` if none exists.
    pub fn get_escrowed_transfer(env: Env, pet_id: u64) -> Option<EscrowedTransfer> {
        env.storage()
            .persistent()
            .get(&DataKey::EscrowedTransfer(pet_id))
    }
}
