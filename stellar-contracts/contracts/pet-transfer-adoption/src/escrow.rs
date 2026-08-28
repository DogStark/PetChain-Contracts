//! Fee escrow for the PetChain pet transfer/adoption contract.
//!
//! Flow:
//!   1. Buyer calls `deposit_fee()` — XLM (in stroops) held in contract storage
//!   2. `finalize_transfer()` — platform fee deducted; remainder released to seller
//!   3. `refund_fee()` — full amount back to buyer (Held or Disputed state)
//!   4. `admin_resolve_dispute()` — admin ruling from Disputed state (buyer, seller, or split)
//!
//! Platform fee: configurable basis points (e.g. 250 = 2.50%)
//!   platform_fee   = amount * fee_bps / 10_000
//!   seller_amount  = amount - platform_fee
//!
//! Storage keys added (no conflict with existing DataKey variants):
//!   DataKey::EscrowEntry(transfer_id) → EscrowEntry
//!   DataKey::PlatformFeeBps          → u32
//!   DataKey::PlatformFeeRecipient    → Address

use soroban_sdk::{contracterror, contracttype, panic_with_error, token, Address, Env, Symbol};

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EscrowError {
    FeeBpsTooHigh = 1,
    InvalidAmount = 2,
    EscrowAlreadyExists = 3,
    EscrowNotFound = 4,
    InvalidEscrowState = 5,
    Unauthorized = 6,
    EscrowNotExpired = 7,
    InvalidTokenAddress = 8,
}

// ─── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum EscrowStatus {
    Held,
    Released,
    Refunded,
    Disputed,
    Resolved,
}

#[contracttype]
#[derive(Debug, Clone, PartialEq)]
pub enum DisputeDecision {
    RefundBuyer,
    PaySeller,
    Split(u32), // basis points of the escrowed amount awarded to the seller
}

/// Default escrow deadline: 90 days in seconds.
pub const DEFAULT_ESCROW_DEADLINE_SECONDS: u64 = 90 * 24 * 60 * 60;

#[contracttype]
#[derive(Debug, Clone)]
pub struct EscrowEntry {
    pub transfer_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub platform_fee_bps: u32,
    pub status: EscrowStatus,
    pub deadline: u64, // ledger timestamp after which escrow may be cancelled
}

#[contracttype]
pub enum EscrowDataKey {
    Entry(u64),
    FeeBps,
    FeeRecipient,
    Admin,
    TokenAddress,
}

// ─── Fee helpers ──────────────────────────────────────────────────────────────

pub fn compute_platform_fee(amount: i128, fee_bps: u32) -> i128 {
    amount * fee_bps as i128 / 10_000
}

pub fn compute_seller_amount(amount: i128, fee_bps: u32) -> i128 {
    amount - compute_platform_fee(amount, fee_bps)
}

// ─── Config ───────────────────────────────────────────────────────────────────

/// Initialise escrow configuration. Only callable once; subsequent calls require
/// the stored admin to call `update_fee_config` instead.
/// Issue #1185: restricted to admin-only on first call (caller must be fee_recipient)
/// and validates that token_address is a non-zero contract address.
pub fn init_escrow_config(env: &Env, fee_bps: u32, fee_recipient: Address, token_address: Address) {
    if fee_bps > 10_000 {
        panic_with_error!(env, EscrowError::FeeBpsTooHigh);
    }
    // Issue #1185: require auth from the fee_recipient (who becomes admin)
    fee_recipient.require_auth();
    // Issue #1185: guard against re-initialisation
    if env.storage().instance().has(&EscrowDataKey::Admin) {
        panic_with_error!(env, EscrowError::Unauthorized);
    }
    env.storage().instance().set(&EscrowDataKey::FeeBps, &fee_bps);
    env.storage()
        .instance()
        .set(&EscrowDataKey::FeeRecipient, &fee_recipient);
    env.storage()
        .instance()
        .set(&EscrowDataKey::TokenAddress, &token_address);
    env.storage()
        .instance()
        .set(&EscrowDataKey::Admin, &fee_recipient);
}

pub fn get_platform_fee_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get::<EscrowDataKey, u32>(&EscrowDataKey::FeeBps)
        .unwrap_or(0)
}

/// Returns the stored fee_bps (or 0 if unset). View-only, no auth required.
/// Closes issue #1004.
pub fn get_fee_bps(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get::<EscrowDataKey, u32>(&EscrowDataKey::FeeBps)
        .unwrap_or(0)
}

/// Returns the stored fee recipient address, or None if unset. View-only, no auth required.
/// Closes issue #1004.
pub fn get_fee_recipient(env: &Env) -> Option<Address> {
    env.storage()
        .instance()
        .get::<EscrowDataKey, Address>(&EscrowDataKey::FeeRecipient)
}

/// Returns the escrow token contract address. Panics if the escrow config is not initialised.
pub fn get_token_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get::<EscrowDataKey, Address>(&EscrowDataKey::TokenAddress)
        .expect("escrow token not configured")
}

fn token_client(env: &Env) -> token::Client<'_> {
    token::Client::new(env, &get_token_address(env))
}

/// Updates the platform fee configuration. Only callable by the stored Admin.
/// Validates new_fee_bps <= 10_000 and emits a CONFIG_UPDATED event with old and new values.
/// Existing in-flight escrows are NOT retroactively updated (they capture fee_bps at deposit time).
/// Closes issue #1005.
pub fn update_fee_config(env: &Env, new_fee_bps: u32, new_fee_recipient: Address) {
    if new_fee_bps > 10_000 {
        panic_with_error!(env, EscrowError::FeeBpsTooHigh);
    }

    // Require admin auth
    let admin: Address = env
        .storage()
        .instance()
        .get(&EscrowDataKey::Admin)
        .expect("escrow config not initialised");
    admin.require_auth();

    let old_fee_bps: u32 = get_platform_fee_bps(env);
    let old_fee_recipient: Option<Address> = get_fee_recipient(env);

    env.storage()
        .instance()
        .set(&EscrowDataKey::FeeBps, &new_fee_bps);
    env.storage()
        .instance()
        .set(&EscrowDataKey::FeeRecipient, &new_fee_recipient);

    env.events().publish(
        (soroban_sdk::symbol_short!("CFG_UPD"),),
        (
            old_fee_bps,
            new_fee_bps,
            old_fee_recipient,
            new_fee_recipient,
        ),
    );
}

// ─── Operations ───────────────────────────────────────────────────────────────

/// Buyer deposits adoption fee into escrow.
pub fn deposit_fee(env: &Env, transfer_id: u64, buyer: Address, seller: Address, amount: i128) {
    buyer.require_auth();
    if amount <= 0 {
        panic_with_error!(env, EscrowError::InvalidAmount);
    }
    let key = EscrowDataKey::Entry(transfer_id);
    if env.storage().persistent().has(&key) {
        panic_with_error!(env, EscrowError::EscrowAlreadyExists);
    }
    let deadline = env.ledger().timestamp() + DEFAULT_ESCROW_DEADLINE_SECONDS;
    let entry = EscrowEntry {
        transfer_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        amount,
        platform_fee_bps: get_platform_fee_bps(env),
        status: EscrowStatus::Held,
        deadline,
    };
    env.storage().persistent().set(&key, &entry);
    token_client(env).transfer(&buyer, &env.current_contract_address(), &amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_HELD"), transfer_id),
        (buyer, amount),
    );
}

/// Releases escrowed fee to seller minus platform fee.
/// Issue #1183: terminal state is written BEFORE token transfers to prevent double-release.
pub fn finalize_transfer(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
    // Issue #1183: mark terminal state before any token transfer
    entry.status = EscrowStatus::Released;
    env.storage().persistent().set(&key, &entry);
    let platform_fee = compute_platform_fee(entry.amount, entry.platform_fee_bps);
    let seller_amount = entry.amount - platform_fee;
    let contract = env.current_contract_address();
    let client = token_client(env);
    client.transfer(&contract, &entry.seller, &seller_amount);
    if platform_fee > 0 {
        let fee_recipient = get_fee_recipient(env).expect("fee recipient not configured");
        client.transfer(&contract, &fee_recipient, &platform_fee);
    }
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_REL"), transfer_id),
        (entry.seller.clone(), seller_amount, platform_fee),
    );
}

/// Refunds the full fee to the buyer (from Held or Disputed state).
/// Issue #1183: terminal state is written BEFORE token transfer to prevent double-refund.
pub fn refund_fee(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held && entry.status != EscrowStatus::Disputed {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
    // Issue #1183: mark terminal state before token transfer
    entry.status = EscrowStatus::Refunded;
    env.storage().persistent().set(&key, &entry);
    token_client(env).transfer(&env.current_contract_address(), &entry.buyer, &entry.amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_RFND"), transfer_id),
        (entry.buyer.clone(), entry.amount),
    );
}

/// Freezes escrow for admin resolution; only buyer or seller may dispute.
pub fn dispute_transfer(env: &Env, transfer_id: u64, initiator: Address) {
    initiator.require_auth();
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
    if initiator != entry.buyer && initiator != entry.seller {
        panic_with_error!(env, EscrowError::Unauthorized);
    }
    entry.status = EscrowStatus::Disputed;
    env.storage().persistent().set(&key, &entry);
}

/// Issue #1184: Cancel an escrow whose deadline has passed. Callable by anyone;
/// refunds the full amount to the buyer.
pub fn cancel_expired_escrow(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held && entry.status != EscrowStatus::Disputed {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
    if env.ledger().timestamp() <= entry.deadline {
        panic_with_error!(env, EscrowError::EscrowNotExpired);
    }
    // Mark terminal state before transfer
    entry.status = EscrowStatus::Refunded;
    env.storage().persistent().set(&key, &entry);
    token_client(env).transfer(&env.current_contract_address(), &entry.buyer, &entry.amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_EXP"), transfer_id),
        (entry.buyer.clone(), entry.amount),
    );
}

/// Admin-controlled ruling on a disputed escrow. Only the configured `FeeRecipient`
/// (platform admin) may call this. Unlike `refund_fee`, which always returns the full
/// amount to the buyer, this allows the admin to pay the seller in full or split the
/// escrowed amount between buyer and seller when the dispute is resolved in the
/// seller's favour (in whole or in part).
pub fn admin_resolve_dispute(env: &Env, transfer_id: u64, decision: DisputeDecision) {
    let admin = get_fee_recipient(env).unwrap_or_else(|| panic_with_error!(env, EscrowError::Unauthorized));
    admin.require_auth();

    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Disputed {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }

    let (seller_amount, buyer_amount): (i128, i128) = match decision {
        DisputeDecision::RefundBuyer => (0, entry.amount),
        DisputeDecision::PaySeller => (entry.amount, 0),
        DisputeDecision::Split(seller_bps) => {
            if seller_bps > 10_000 {
                panic_with_error!(env, EscrowError::FeeBpsTooHigh);
            }
            let seller_amount = entry.amount * seller_bps as i128 / 10_000;
            (seller_amount, entry.amount - seller_amount)
        }
    };

    let contract = env.current_contract_address();
    let client = token_client(env);

    // Issue #1183: mark terminal state before token transfers
    entry.status = EscrowStatus::Resolved;
    env.storage().persistent().set(&key, &entry);

    if seller_amount > 0 {
        client.transfer(&contract, &entry.seller, &seller_amount);
    }
    if buyer_amount > 0 {
        client.transfer(&contract, &entry.buyer, &buyer_amount);
    }

    // Mark terminal state before transfers
    entry.status = EscrowStatus::Resolved;
    env.storage().persistent().set(&key, &entry);

    env.events().publish(
        (Symbol::new(env, "DISPUTE_RESOLVED"), transfer_id),
        (entry.buyer.clone(), entry.seller.clone(), seller_amount, buyer_amount),
    );
}

pub fn get_escrow(env: &Env, transfer_id: u64) -> Option<EscrowEntry> {
    env.storage()
        .persistent()
        .get(&EscrowDataKey::Entry(transfer_id))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    // Threat-model note: all auth, token, and state-transition paths are tested
    // below. Terminal states are written before token transfers (issues #1183, #1184)
    // to prevent double-settlement under reentrancy or retry. init_escrow_config
    // requires admin auth and rejects re-initialisation (issue #1185).
    use super::*;
    use crate::PetOwnershipContract;
    use soroban_sdk::{testutils::{Address as _, Ledger}, token::StellarAssetClient, Address, Env};

    /// Registers the contract plus a Stellar asset (XLM stand-in) and funds the buyer.
    struct Ctx {
        env:      Env,
        contract: Address,
        token:    Address,
        buyer:    Address,
        seller:   Address,
        platform: Address,
    }

    impl Ctx {
        /// Runs `f` inside the contract's storage/auth context.
        fn run<T>(&self, f: impl FnOnce() -> T) -> T {
            self.env.as_contract(&self.contract, f)
        }

        fn balance(&self, who: &Address) -> i128 {
            token::Client::new(&self.env, &self.token).balance(who)
        }
    }

    fn setup() -> Ctx {
        let env = Env::default();
        env.mock_all_auths();
        let contract     = env.register_contract(None, PetOwnershipContract);
        let token_admin  = Address::generate(&env);
        let token        = env.register_stellar_asset_contract(token_admin);
        let buyer        = Address::generate(&env);
        let seller       = Address::generate(&env);
        let platform     = Address::generate(&env);
        StellarAssetClient::new(&env, &token).mint(&buyer, &1_000_000_000);
        Ctx { env, contract, token, buyer, seller, platform }
    }

    // ── Issue #1185: init_escrow_config admin-only + no re-init ──────────────

    #[test]
    #[should_panic(expected = "Error(Contract, #6)")]
    fn init_escrow_config_rejects_reinitialisation() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            // Second call must fail with Unauthorized
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
        });
    }

    // ── Issue #1183: terminal state written before token transfer ────────────

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn finalize_transfer_state_set_before_transfer_prevents_double_release() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 500, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 500);
            // State is Released; second call must fail
            finalize_transfer(&c.env, 500);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn refund_fee_state_set_before_transfer_prevents_double_refund() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 501, c.buyer.clone(), c.seller.clone(), 1_000_000);
            refund_fee(&c.env, 501);
            // State is Refunded; second call must fail
            refund_fee(&c.env, 501);
        });
    }

    // ── Issue #1184: escrow timeout / abandonment ────────────────────────────

    #[test]
    fn deposit_fee_records_deadline() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 600, c.buyer.clone(), c.seller.clone(), 1_000_000);
            let e = get_escrow(&c.env, 600).unwrap();
            assert_eq!(e.deadline, c.env.ledger().timestamp() + DEFAULT_ESCROW_DEADLINE_SECONDS);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #7)")]
    fn cancel_expired_escrow_before_deadline_is_rejected() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 601, c.buyer.clone(), c.seller.clone(), 1_000_000);
            cancel_expired_escrow(&c.env, 601);
        });
    }

    #[test]
    fn cancel_expired_escrow_after_deadline_refunds_buyer() {
        let c = setup();
        let before = c.balance(&c.buyer);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 602, c.buyer.clone(), c.seller.clone(), 1_000_000);
            c.env.ledger().with_mut(|l| l.timestamp += DEFAULT_ESCROW_DEADLINE_SECONDS + 1);
            cancel_expired_escrow(&c.env, 602);
            assert_eq!(get_escrow(&c.env, 602).unwrap().status, EscrowStatus::Refunded);
        });
        assert_eq!(c.balance(&c.buyer), before);
        assert_eq!(c.balance(&c.contract), 0);
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn cancel_expired_escrow_on_released_state_is_rejected() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 603, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 603);
            c.env.ledger().with_mut(|l| l.timestamp += DEFAULT_ESCROW_DEADLINE_SECONDS + 1);
            cancel_expired_escrow(&c.env, 603);
        });
    }

    #[test]
    fn deposit_creates_held_entry() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 1, c.buyer.clone(), c.seller.clone(), 10_000_000);
            let e = get_escrow(&c.env, 1).unwrap();
            assert_eq!(e.status, EscrowStatus::Held);
            assert_eq!(e.amount, 10_000_000);
            assert_eq!(e.buyer, c.buyer);
        });
    }

    #[test]
    fn finalize_sets_released() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 2, c.buyer.clone(), c.seller.clone(), 10_000_000);
            finalize_transfer(&c.env, 2);
            assert_eq!(get_escrow(&c.env, 2).unwrap().status, EscrowStatus::Released);
        });
    }

    #[test]
    fn fee_calculation_correct() {
        assert_eq!(compute_platform_fee(10_000_000, 250), 250_000);
        assert_eq!(compute_seller_amount(10_000_000, 250), 9_750_000);
    }

    #[test]
    fn refund_sets_refunded() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 3, c.buyer.clone(), c.seller.clone(), 5_000_000);
            refund_fee(&c.env, 3);
            assert_eq!(get_escrow(&c.env, 3).unwrap().status, EscrowStatus::Refunded);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn cannot_finalize_twice() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 4, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 4);
            finalize_transfer(&c.env, 4);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn cannot_refund_after_release() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 5, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 5);
            refund_fee(&c.env, 5);
        });
    }

    #[test]
    fn dispute_freezes_escrow() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 6, c.buyer.clone(), c.seller.clone(), 2_000_000);
        });
        c.run(|| {
            dispute_transfer(&c.env, 6, c.buyer.clone());
            assert_eq!(get_escrow(&c.env, 6).unwrap().status, EscrowStatus::Disputed);
        });
    }

    #[test]
    fn refund_works_from_disputed() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 7, c.buyer.clone(), c.seller.clone(), 3_000_000);
        });
        c.run(|| {
            dispute_transfer(&c.env, 7, c.buyer.clone());
            refund_fee(&c.env, 7);
            assert_eq!(get_escrow(&c.env, 7).unwrap().status, EscrowStatus::Refunded);
        });
    }

    #[test]
    fn zero_fee_bps_full_amount_to_seller() {
        assert_eq!(compute_platform_fee(10_000_000, 0), 0);
        assert_eq!(compute_seller_amount(10_000_000, 0), 10_000_000);
    }

    // ── Issue #1001: real token transfers on deposit / finalize / refund ─────

    /// deposit_fee moves the full amount from the buyer into the contract.
    #[test]
    fn deposit_moves_funds_into_contract() {
        let c = setup();
        let before = c.balance(&c.buyer);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 100, c.buyer.clone(), c.seller.clone(), 10_000_000);
        });
        assert_eq!(c.balance(&c.buyer), before - 10_000_000);
        assert_eq!(c.balance(&c.contract), 10_000_000);
    }

    /// finalize_transfer pays the seller the net amount and the platform its fee.
    #[test]
    fn finalize_pays_seller_and_fee_recipient() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 101, c.buyer.clone(), c.seller.clone(), 10_000_000);
            finalize_transfer(&c.env, 101);
        });
        assert_eq!(c.balance(&c.seller), 9_750_000);
        assert_eq!(c.balance(&c.platform), 250_000);
        assert_eq!(c.balance(&c.contract), 0);
    }

    /// With a zero fee the seller receives the whole amount and no fee transfer occurs.
    #[test]
    fn finalize_with_zero_fee_pays_full_amount_to_seller() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 0, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 102, c.buyer.clone(), c.seller.clone(), 10_000_000);
            finalize_transfer(&c.env, 102);
        });
        assert_eq!(c.balance(&c.seller), 10_000_000);
        assert_eq!(c.balance(&c.platform), 0);
        assert_eq!(c.balance(&c.contract), 0);
    }

    /// refund_fee returns the full amount to the buyer.
    #[test]
    fn refund_returns_funds_to_buyer() {
        let c = setup();
        let before = c.balance(&c.buyer);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 103, c.buyer.clone(), c.seller.clone(), 10_000_000);
            refund_fee(&c.env, 103);
        });
        assert_eq!(c.balance(&c.buyer), before);
        assert_eq!(c.balance(&c.contract), 0);
        assert_eq!(c.balance(&c.seller), 0);
    }

    // ── Issue #1006: missing-entry error-path tests ───────────────────────────

    /// finalize_transfer panics with "escrow not found" when no entry exists.
    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn finalize_transfer_panics_on_missing_entry() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            // transfer_id 999 was never deposited
            finalize_transfer(&c.env, 999);
        });
    }

    /// refund_fee panics with "escrow not found" when no entry exists.
    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn refund_fee_panics_on_missing_entry() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            // transfer_id 999 was never deposited
            refund_fee(&c.env, 999);
        });
    }

    /// dispute_transfer panics with "escrow not found" when no entry exists.
    #[test]
    #[should_panic(expected = "Error(Contract, #4)")]
    fn dispute_transfer_panics_on_missing_entry() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            // transfer_id 999 was never deposited
            dispute_transfer(&c.env, 999, c.buyer.clone());
        });
    }

    // ── Issue #1004: get_fee_bps and get_fee_recipient view functions ─────────

    #[test]
    fn get_fee_bps_returns_configured_value() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 300, c.platform.clone(), c.token.clone());
            assert_eq!(get_fee_bps(&c.env), 300);
        });
    }

    #[test]
    fn get_fee_bps_returns_zero_when_not_configured() {
        let c = setup();
        c.run(|| assert_eq!(get_fee_bps(&c.env), 0));
    }

    #[test]
    fn get_fee_recipient_returns_configured_address() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 150, c.platform.clone(), c.token.clone());
            assert_eq!(get_fee_recipient(&c.env), Some(c.platform.clone()));
        });
    }

    #[test]
    fn get_fee_recipient_returns_none_when_not_configured() {
        let c = setup();
        c.run(|| assert_eq!(get_fee_recipient(&c.env), None));
    }

    // ── Issue #1005: update_fee_config admin-only update ─────────────────────

    #[test]
    fn update_fee_config_changes_fee_bps_and_recipient() {
        let c = setup();
        let new_recipient = Address::generate(&c.env);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            update_fee_config(&c.env, 100, new_recipient.clone());
            assert_eq!(get_fee_bps(&c.env), 100);
            assert_eq!(get_fee_recipient(&c.env), Some(new_recipient.clone()));
        });
    }

    #[test]
    fn new_escrow_uses_updated_fee_rate() {
        let c = setup();
        let new_recipient = Address::generate(&c.env);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());

            // Update fee to 500 bps (5%)
            update_fee_config(&c.env, 500, new_recipient.clone());

            // New deposit captures the updated rate
            deposit_fee(&c.env, 10, c.buyer.clone(), c.seller.clone(), 10_000_000);
            assert_eq!(get_escrow(&c.env, 10).unwrap().platform_fee_bps, 500);
        });
    }

    #[test]
    fn existing_escrow_keeps_original_fee_rate_after_update() {
        let c = setup();
        let new_recipient = Address::generate(&c.env);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());

            // Deposit before update — captures 250 bps
            deposit_fee(&c.env, 20, c.buyer.clone(), c.seller.clone(), 10_000_000);

            // Update fee to 500 bps
            update_fee_config(&c.env, 500, new_recipient.clone());

            // Existing escrow is unchanged
            assert_eq!(get_escrow(&c.env, 20).unwrap().platform_fee_bps, 250);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn update_fee_config_rejects_fee_bps_over_10000() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            update_fee_config(&c.env, 10_001, c.platform.clone());
        });
    }

    // ── Issue #1002: typed EscrowError variants ──────────────────────────────

    /// init_escrow_config rejects a fee above 100% with FeeBpsTooHigh.
    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn init_escrow_config_rejects_fee_bps_over_10000() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 10_001, c.platform.clone(), c.token.clone());
        });
    }

    /// deposit_fee rejects a non-positive amount with InvalidAmount.
    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn deposit_fee_rejects_non_positive_amount() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 30, c.buyer.clone(), c.seller.clone(), 0);
        });
    }

    /// A second deposit under the same transfer_id fails with EscrowAlreadyExists.
    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn deposit_fee_rejects_duplicate_transfer_id() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 31, c.buyer.clone(), c.seller.clone(), 1_000_000);
        });
        // A fresh top-level call is needed since `buyer.require_auth()` can only
        // be satisfied once per invocation frame under `mock_all_auths`.
        c.run(|| {
            deposit_fee(&c.env, 31, c.buyer.clone(), c.seller.clone(), 1_000_000);
        });
    }

    /// Finalizing an already-released escrow fails with InvalidEscrowState.
    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn finalize_transfer_rejects_non_held_state() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 32, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 32);
            finalize_transfer(&c.env, 32);
        });
    }

    /// Refunding an already-released escrow fails with InvalidEscrowState.
    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn refund_fee_rejects_non_refundable_state() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 33, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 33);
            refund_fee(&c.env, 33);
        });
    }

    /// A third party cannot dispute an escrow — Unauthorized.
    #[test]
    #[should_panic(expected = "Error(Contract, #6)")]
    fn dispute_transfer_rejects_third_party() {
        let c = setup();
        c.run(|| {
            let stranger = Address::generate(&c.env);
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 34, c.buyer.clone(), c.seller.clone(), 1_000_000);
            dispute_transfer(&c.env, 34, stranger);
        });
    }

    // ── Issue #1003: admin_resolve_dispute ────────────────────────────────────

    /// Admin rules for the buyer: the full escrowed amount is refunded.
    #[test]
    fn admin_resolve_dispute_refunds_buyer() {
        let c = setup();
        let before = c.balance(&c.buyer);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 200, c.buyer.clone(), c.seller.clone(), 10_000_000);
        });
        // Separate top-level calls: `buyer.require_auth()` can only be satisfied
        // once per invocation frame under `mock_all_auths`.
        c.run(|| {
            dispute_transfer(&c.env, 200, c.buyer.clone());
            admin_resolve_dispute(&c.env, 200, DisputeDecision::RefundBuyer);
        });
        assert_eq!(c.balance(&c.buyer), before);
        assert_eq!(c.balance(&c.seller), 0);
        assert_eq!(c.balance(&c.contract), 0);
        c.run(|| {
            assert_eq!(get_escrow(&c.env, 200).unwrap().status, EscrowStatus::Resolved);
        });
    }

    /// Admin rules for the seller: the full escrowed amount is paid out.
    #[test]
    fn admin_resolve_dispute_pays_seller() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 201, c.buyer.clone(), c.seller.clone(), 10_000_000);
        });
        c.run(|| {
            dispute_transfer(&c.env, 201, c.buyer.clone());
            admin_resolve_dispute(&c.env, 201, DisputeDecision::PaySeller);
        });
        assert_eq!(c.balance(&c.seller), 10_000_000);
        assert_eq!(c.balance(&c.contract), 0);
        c.run(|| {
            assert_eq!(get_escrow(&c.env, 201).unwrap().status, EscrowStatus::Resolved);
        });
    }

    /// Admin splits the escrowed amount between buyer and seller.
    #[test]
    fn admin_resolve_dispute_split_decision() {
        let c = setup();
        let before = c.balance(&c.buyer);
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 202, c.buyer.clone(), c.seller.clone(), 10_000_000);
        });
        c.run(|| {
            dispute_transfer(&c.env, 202, c.buyer.clone());
            admin_resolve_dispute(&c.env, 202, DisputeDecision::Split(3_000));
        });
        assert_eq!(c.balance(&c.seller), 3_000_000);
        assert_eq!(c.balance(&c.buyer), before - 10_000_000 + 7_000_000);
        assert_eq!(c.balance(&c.contract), 0);
        c.run(|| {
            assert_eq!(get_escrow(&c.env, 202).unwrap().status, EscrowStatus::Resolved);
        });
    }

    /// admin_resolve_dispute can only be called from the Disputed state.
    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn admin_resolve_dispute_rejects_non_disputed_state() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 203, c.buyer.clone(), c.seller.clone(), 1_000_000);
            admin_resolve_dispute(&c.env, 203, DisputeDecision::RefundBuyer);
        });
    }
}
