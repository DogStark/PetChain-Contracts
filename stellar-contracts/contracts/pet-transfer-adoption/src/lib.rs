#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Env, Symbol, Vec,
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
    TrustedContract,
    TrustedAdmins,
    TrustedThreshold,
    TrustedUpdateApprovals((Address, Address)),
}

/// ======================================================
/// EVENTS
/// ======================================================

const EVT_TRANSFER_INITIATED: Symbol = symbol_short!("xfer_init");
const EVT_TRANSFER_CANCELLED: Symbol = symbol_short!("xfer_cncl");
const EVT_TRANSFER_ESCROWED: Symbol = symbol_short!("xfer_escr");
const EVT_TRANSFER_FINALIZED: Symbol = symbol_short!("xfer_fin");
const EVT_TRANSFER_DISPUTED: Symbol = symbol_short!("xfer_disp");
const EVT_TRUSTED_UPDATED: Symbol = symbol_short!("trust_upd");

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
    AlreadyInitialized = 15,
    InvalidThreshold = 16,
    NotMultisigAdmin = 17,
    ThresholdNotMet = 18,
    UntrustedContract = 19,
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

fn append_custody_entry(
    env: &Env,
    pet_id: u64,
    from: Address,
    to: Address,
    transfer_type: TransferType,
) {
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

fn get_trusted_contract(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::TrustedContract)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::UntrustedContract))
}

fn require_trusted_contract(env: &Env, callee: &Address) {
    if &get_trusted_contract(env) != callee {
        panic_with_error!(env, ContractError::UntrustedContract);
    }
}

fn require_trusted_multisig_admin(env: &Env, signer: &Address) -> (Vec<Address>, u32) {
    let admins: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::TrustedAdmins)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotMultisigAdmin));
    let threshold: u32 = env
        .storage()
        .instance()
        .get(&DataKey::TrustedThreshold)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidThreshold));

    if !admins.contains(signer) {
        panic_with_error!(env, ContractError::NotMultisigAdmin);
    }

    signer.require_auth();
    (admins, threshold)
}

fn clear_trusted_update_approvals(env: &Env, admins: &Vec<Address>, new_address: &Address) {
    for admin in admins.iter() {
        env.storage()
            .instance()
            .remove(&DataKey::TrustedUpdateApprovals((
                new_address.clone(),
                admin,
            )));
    }
}

/// ======================================================
/// CONTRACT IMPLEMENTATION
/// ======================================================

#[contractimpl]
impl PetOwnershipContract {
    /// ----------------------------------
    /// INITIALIZE TRUSTED MAIN CONTRACT
    /// ----------------------------------

    pub fn init_trusted_contract(
        env: Env,
        trusted_contract: Address,
        admins: Vec<Address>,
        threshold: u32,
    ) {
        if env.storage().instance().has(&DataKey::TrustedContract) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        if threshold == 0 || threshold > admins.len() {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }

        let first_admin = admins
            .get(0)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidThreshold));
        first_admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::TrustedContract, &trusted_contract);
        env.storage()
            .instance()
            .set(&DataKey::TrustedAdmins, &admins);
        env.storage()
            .instance()
            .set(&DataKey::TrustedThreshold, &threshold);
    }

    /// Approve a trusted main-contract address update. The new address is
    /// stored once configured multisig approvals reach the configured threshold.
    pub fn update_trusted_contract(env: Env, new_address: Address, signer: Address) -> bool {
        let (admins, threshold) = require_trusted_multisig_admin(&env, &signer);
        let approval_key = DataKey::TrustedUpdateApprovals((new_address.clone(), signer));

        if !env.storage().instance().has(&approval_key) {
            env.storage().instance().set(&approval_key, &true);
        }

        let mut approvals = 0u32;
        for admin in admins.iter() {
            if env
                .storage()
                .instance()
                .has(&DataKey::TrustedUpdateApprovals((
                    new_address.clone(),
                    admin,
                )))
            {
                approvals += 1;
            }
        }

        if approvals < threshold {
            return false;
        }

        let previous = get_trusted_contract(&env);
        env.storage()
            .instance()
            .set(&DataKey::TrustedContract, &new_address);
        clear_trusted_update_approvals(&env, &admins, &new_address);

        env.events()
            .publish((EVT_TRUSTED_UPDATED,), (previous, new_address));
        true
    }

    pub fn get_trusted_contract_address(env: Env) -> Address {
        get_trusted_contract(&env)
    }

    pub fn validate_trusted_contract(env: Env, callee: Address) -> bool {
        require_trusted_contract(&env, &callee);
        true
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

        append_custody_entry(
            &env,
            pet_id,
            escrowed.from.clone(),
            escrowed.to.clone(),
            TransferType::Direct,
        );

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
        let owner =
            expected_owner.unwrap_or_else(|| panic_with_error!(env, ContractError::EmptyBatch));

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

            env.events().publish(
                (EVT_TRANSFER_INITIATED, pet_id),
                (owner.clone(), to.clone()),
            );
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
