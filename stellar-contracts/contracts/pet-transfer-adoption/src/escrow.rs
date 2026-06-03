//! Fee escrow for the PetChain pet transfer/adoption contract.
//!
//! Flow:
//!   1. Buyer calls `deposit_fee()` — XLM (in stroops) held in contract storage
//!   2. `finalize_transfer()` — platform fee deducted; remainder released to seller
//!   3. `refund_fee()` — full amount back to buyer (Held or Disputed state)
//!
//! Platform fee: configurable basis points (e.g. 250 = 2.50%)
//!   platform_fee   = amount * fee_bps / 10_000
//!   seller_amount  = amount - platform_fee
//!
//! Storage keys added (no conflict with existing DataKey variants):
//!   DataKey::EscrowEntry(transfer_id) → EscrowEntry
//!   DataKey::PlatformFeeBps          → u32
//!   DataKey::PlatformFeeRecipient    → Address

use soroban_sdk::{contracttype, Address, Env};

// ─── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum EscrowStatus {
    Held,
    Released,
    Refunded,
    Disputed,
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct EscrowEntry {
    pub transfer_id:      u64,
    pub buyer:            Address,
    pub seller:           Address,
    pub amount:           i128,
    pub platform_fee_bps: u32,
    pub status:           EscrowStatus,
}

#[contracttype]
pub enum EscrowDataKey {
    Entry(u64),
    FeeBps,
    FeeRecipient,
}

// ─── Fee helpers ──────────────────────────────────────────────────────────────

pub fn compute_platform_fee(amount: i128, fee_bps: u32) -> i128 {
    amount * fee_bps as i128 / 10_000
}

pub fn compute_seller_amount(amount: i128, fee_bps: u32) -> i128 {
    amount - compute_platform_fee(amount, fee_bps)
}

// ─── Config ───────────────────────────────────────────────────────────────────

pub fn init_escrow_config(env: &Env, fee_bps: u32, fee_recipient: Address) {
    assert!(fee_bps <= 10_000, "fee_bps must be <= 10000");
    env.storage().instance().set(&EscrowDataKey::FeeBps, &fee_bps);
    env.storage().instance().set(&EscrowDataKey::FeeRecipient, &fee_recipient);
}

pub fn get_platform_fee_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get::<EscrowDataKey, u32>(&EscrowDataKey::FeeBps)
        .unwrap_or(0)
}

// ─── Operations ───────────────────────────────────────────────────────────────

/// Buyer deposits adoption fee into escrow.
pub fn deposit_fee(env: &Env, transfer_id: u64, buyer: Address, seller: Address, amount: i128) {
    buyer.require_auth();
    assert!(amount > 0, "amount must be positive");
    let key = EscrowDataKey::Entry(transfer_id);
    assert!(!env.storage().persistent().has(&key), "escrow already exists");
    let entry = EscrowEntry {
        transfer_id,
        buyer:            buyer.clone(),
        seller:           seller.clone(),
        amount,
        platform_fee_bps: get_platform_fee_bps(env),
        status:           EscrowStatus::Held,
    };
    env.storage().persistent().set(&key, &entry);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_HELD"), transfer_id),
        (buyer, amount),
    );
}

/// Releases escrowed fee to seller minus platform fee.
pub fn finalize_transfer(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = env.storage().persistent().get(&key).expect("escrow not found");
    assert!(entry.status == EscrowStatus::Held, "cannot finalize: not in Held state");
    let platform_fee  = compute_platform_fee(entry.amount, entry.platform_fee_bps);
    let seller_amount = entry.amount - platform_fee;
    // Wire-up: token_client.transfer(&contract, &entry.seller, &seller_amount);
    //          token_client.transfer(&contract, &fee_recipient, &platform_fee);
    entry.status = EscrowStatus::Released;
    env.storage().persistent().set(&key, &entry);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_REL"), transfer_id),
        (entry.seller.clone(), seller_amount, platform_fee),
    );
}

/// Refunds the full fee to the buyer (from Held or Disputed state).
pub fn refund_fee(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = env.storage().persistent().get(&key).expect("escrow not found");
    assert!(
        entry.status == EscrowStatus::Held || entry.status == EscrowStatus::Disputed,
        "cannot refund: invalid state"
    );
    // Wire-up: token_client.transfer(&contract, &entry.buyer, &entry.amount);
    entry.status = EscrowStatus::Refunded;
    env.storage().persistent().set(&key, &entry);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_RFND"), transfer_id),
        (entry.buyer.clone(), entry.amount),
    );
}

/// Freezes escrow for admin resolution; only buyer or seller may dispute.
pub fn dispute_transfer(env: &Env, transfer_id: u64, initiator: Address) {
    initiator.require_auth();
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = env.storage().persistent().get(&key).expect("escrow not found");
    assert!(entry.status == EscrowStatus::Held, "cannot dispute: not in Held state");
    assert!(initiator == entry.buyer || initiator == entry.seller, "only buyer or seller may dispute");
    entry.status = EscrowStatus::Disputed;
    env.storage().persistent().set(&key, &entry);
}

pub fn get_escrow(env: &Env, transfer_id: u64) -> Option<EscrowEntry> {
    env.storage().persistent().get(&EscrowDataKey::Entry(transfer_id))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, Address, Address, Address) {
        let env      = Env::default();
        env.mock_all_auths();
        let buyer    = Address::generate(&env);
        let seller   = Address::generate(&env);
        let platform = Address::generate(&env);
        (env, buyer, seller, platform)
    }

    #[test]
    fn deposit_creates_held_entry() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 250, platform);
        deposit_fee(&env, 1, buyer.clone(), seller, 10_000_000);
        let e = get_escrow(&env, 1).unwrap();
        assert_eq!(e.status, EscrowStatus::Held);
        assert_eq!(e.amount, 10_000_000);
        assert_eq!(e.buyer,  buyer);
    }

    #[test]
    fn finalize_sets_released() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 250, platform);
        deposit_fee(&env, 2, buyer, seller, 10_000_000);
        finalize_transfer(&env, 2);
        assert_eq!(get_escrow(&env, 2).unwrap().status, EscrowStatus::Released);
    }

    #[test]
    fn fee_calculation_correct() {
        assert_eq!(compute_platform_fee(10_000_000, 250), 250_000);
        assert_eq!(compute_seller_amount(10_000_000, 250), 9_750_000);
    }

    #[test]
    fn refund_sets_refunded() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 100, platform);
        deposit_fee(&env, 3, buyer, seller, 5_000_000);
        refund_fee(&env, 3);
        assert_eq!(get_escrow(&env, 3).unwrap().status, EscrowStatus::Refunded);
    }

    #[test]
    #[should_panic]
    fn cannot_finalize_twice() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 100, platform);
        deposit_fee(&env, 4, buyer, seller, 1_000_000);
        finalize_transfer(&env, 4);
        finalize_transfer(&env, 4);
    }

    #[test]
    #[should_panic]
    fn cannot_refund_after_release() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 100, platform);
        deposit_fee(&env, 5, buyer, seller, 1_000_000);
        finalize_transfer(&env, 5);
        refund_fee(&env, 5);
    }

    #[test]
    fn dispute_freezes_escrow() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 100, platform);
        deposit_fee(&env, 6, buyer.clone(), seller, 2_000_000);
        dispute_transfer(&env, 6, buyer);
        assert_eq!(get_escrow(&env, 6).unwrap().status, EscrowStatus::Disputed);
    }

    #[test]
    fn refund_works_from_disputed() {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 100, platform);
        deposit_fee(&env, 7, buyer.clone(), seller, 3_000_000);
        dispute_transfer(&env, 7, buyer);
        refund_fee(&env, 7);
        assert_eq!(get_escrow(&env, 7).unwrap().status, EscrowStatus::Refunded);
    }

    #[test]
    fn zero_fee_bps_full_amount_to_seller() {
        assert_eq!(compute_platform_fee(10_000_000, 0), 0);
        assert_eq!(compute_seller_amount(10_000_000, 0), 10_000_000);
    }
}
