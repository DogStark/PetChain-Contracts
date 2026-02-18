#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    panic_with_error, Address, Env, Symbol, Vec,
};

mod vet_reigistry;

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OwnershipRecord {
    pub owner: Address,
    pub acquired_at: u64,
    pub relinquished_at: Option<u64>,
}

/// ======================================================
/// STORAGE KEYS
/// ======================================================

#[contracttype]
enum DataKey {
    Pet(u64),
    PendingTransfer(u64),
    OwnershipHistory(u64),
}

/// ======================================================
/// EVENTS
/// ======================================================

const EVT_TRANSFER_INITIATED: Symbol = symbol_short!("xfer_init");
const EVT_TRANSFER_ACCEPTED: Symbol = symbol_short!("xfer_ok");
const EVT_TRANSFER_CANCELLED: Symbol = symbol_short!("xfer_cancel");

/// ======================================================
/// ERRORS
/// ======================================================

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractError {
    PetNotFound = 1,
    Unauthorized = 2,
    TransferAlreadyPending = 3,
    NoPendingTransfer = 4,
    InvalidRecipient = 5,
}

impl From<ContractError> for soroban_sdk::Error {
    fn from(e: ContractError) -> Self {
        use soroban_sdk::xdr::{ScErrorCode, ScErrorType};
        let code = match e {
            ContractError::PetNotFound => ScErrorCode::MissingValue,
            ContractError::Unauthorized => ScErrorCode::InvalidAction,
            ContractError::TransferAlreadyPending => ScErrorCode::ExistingValue,
            ContractError::NoPendingTransfer => ScErrorCode::MissingValue,
            ContractError::InvalidRecipient => ScErrorCode::InvalidAction,
        };
        soroban_sdk::Error::from((ScErrorType::Contract, code))
    }
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
    env.storage().persistent().set(&DataKey::Pet(pet.pet_id), pet);
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

/// ======================================================
/// CONTRACT IMPLEMENTATION
/// ======================================================

#[contractimpl]
impl PetOwnershipContract {
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
            owner,
            acquired_at: env.ledger().timestamp(),
            relinquished_at: None,
        });

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);
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

        env.events().publish(
            (EVT_TRANSFER_INITIATED, pet_id),
            (pet.current_owner, to),
        );
    }

    /// ----------------------------------
    /// ACCEPT TRANSFER
    /// ----------------------------------

    pub fn accept_transfer(env: Env, pet_id: u64) {
        let transfer: PendingTransfer = env
            .storage()
            .persistent()
            .get(&DataKey::PendingTransfer(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

        transfer.to.require_auth();

        let mut pet = get_pet(&env, pet_id);

        if pet.current_owner != transfer.from {
            panic_with_error!(env, ContractError::Unauthorized);
        }

        let now = env.ledger().timestamp();

        // Update ownership history
        let mut history = get_history(&env, pet_id);
        let last = history.len() - 1;
        let mut prev = history.get(last).unwrap();
        prev.relinquished_at = Some(now);
        history.set(last, prev);

        history.push_back(OwnershipRecord {
            owner: transfer.to.clone(),
            acquired_at: now,
            relinquished_at: None,
        });

        pet.current_owner = transfer.to.clone();

        save_pet(&env, &pet);
        save_history(&env, pet_id, &history);

        env.storage()
            .persistent()
            .remove(&DataKey::PendingTransfer(pet_id));

        env.events().publish(
            (EVT_TRANSFER_ACCEPTED, pet_id),
            (transfer.from, transfer.to),
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

        env.storage()
            .persistent()
            .remove(&DataKey::PendingTransfer(pet_id));

        env.events().publish(
            (EVT_TRANSFER_CANCELLED, pet_id),
            (transfer.from, transfer.to),
        );
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

    pub fn has_pending_transfer(env: Env, pet_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::PendingTransfer(pet_id))
    }
}
