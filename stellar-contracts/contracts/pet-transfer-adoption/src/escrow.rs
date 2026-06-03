use soroban_sdk::{contracttype, panic_with_error, symbol_short, Address, Env, Symbol};

use crate::ContractError;

// ---------------------------------------------------------------------------
// Storage key
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowDataKey {
    Escrow(u64), // pet_id -> EscrowEntry
}

// ---------------------------------------------------------------------------
// State machine
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowState {
    Held,
    Released,
    Refunded,
    Disputed,
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowEntry {
    pub pet_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub fee_bps: u32, // platform fee in basis points (e.g. 250 = 2.5 %)
    pub state: EscrowState,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

const EVT_FEE_HELD: Symbol = symbol_short!("FEE_HELD");
const EVT_FEE_REL: Symbol = symbol_short!("FEE_REL");
const EVT_FEE_RFND: Symbol = symbol_short!("FEE_RFND");
const EVT_DISPUTED: Symbol = symbol_short!("DISPUTED");

// ---------------------------------------------------------------------------
// Fee helpers
// ---------------------------------------------------------------------------

pub fn compute_platform_fee(amount: i128, fee_bps: u32) -> i128 {
    amount * (fee_bps as i128) / 10_000
}

pub fn compute_seller_amount(amount: i128, fee_bps: u32) -> i128 {
    amount - compute_platform_fee(amount, fee_bps)
}

// ---------------------------------------------------------------------------
// Escrow functions
// ---------------------------------------------------------------------------

/// Buyer deposits XLM into escrow. Creates a `Held` entry.
pub fn deposit_fee(
    env: &Env,
    pet_id: u64,
    buyer: Address,
    seller: Address,
    amount: i128,
    fee_bps: u32,
) {
    buyer.require_auth();

    let entry = EscrowEntry {
        pet_id,
        buyer: buyer.clone(),
        seller,
        amount,
        fee_bps,
        state: EscrowState::Held,
    };

    env.storage()
        .persistent()
        .set(&EscrowDataKey::Escrow(pet_id), &entry);

    env.events()
        .publish((EVT_FEE_HELD, pet_id), (buyer, amount));
}

/// Finalizes transfer: deducts platform fee and releases net amount to seller.
pub fn finalize_transfer(env: &Env, pet_id: u64) {
    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&EscrowDataKey::Escrow(pet_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

    if entry.state != EscrowState::Held {
        panic_with_error!(env, ContractError::Unauthorized);
    }

    let seller_amount = compute_seller_amount(entry.amount, entry.fee_bps);

    entry.state = EscrowState::Released;

    env.storage()
        .persistent()
        .set(&EscrowDataKey::Escrow(pet_id), &entry);

    env.events()
        .publish((EVT_FEE_REL, pet_id), (entry.seller.clone(), seller_amount));
}

/// Full refund to buyer from `Held` or `Disputed` state.
pub fn refund_fee(env: &Env, pet_id: u64) {
    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&EscrowDataKey::Escrow(pet_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

    if entry.state != EscrowState::Held && entry.state != EscrowState::Disputed {
        panic_with_error!(env, ContractError::Unauthorized);
    }

    entry.state = EscrowState::Refunded;

    env.storage()
        .persistent()
        .set(&EscrowDataKey::Escrow(pet_id), &entry);

    env.events()
        .publish((EVT_FEE_RFND, pet_id), (entry.buyer.clone(), entry.amount));
}

/// Buyer or seller raises a dispute, freezing the escrow.
pub fn dispute_transfer(env: &Env, pet_id: u64, caller: Address) {
    caller.require_auth();

    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&EscrowDataKey::Escrow(pet_id))
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NoPendingTransfer));

    if caller != entry.buyer && caller != entry.seller {
        panic_with_error!(env, ContractError::Unauthorized);
    }

    if entry.state != EscrowState::Held {
        panic_with_error!(env, ContractError::Unauthorized);
    }

    entry.state = EscrowState::Disputed;

    env.storage()
        .persistent()
        .set(&EscrowDataKey::Escrow(pet_id), &entry);

    env.events()
        .publish((EVT_DISPUTED, pet_id), caller);
}

/// Returns the escrow entry for a pet, if it exists.
pub fn get_escrow(env: &Env, pet_id: u64) -> Option<EscrowEntry> {
    env.storage()
        .persistent()
        .get(&EscrowDataKey::Escrow(pet_id))
}
