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

use soroban_sdk::{contracttype, token, Address, Env};
use soroban_sdk::{contracterror, contracttype, panic_with_error, Address, Env};

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
}

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
    pub transfer_id: u64,
    pub buyer: Address,
    pub seller: Address,
    pub amount: i128,
    pub platform_fee_bps: u32,
    pub status: EscrowStatus,
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

pub fn init_escrow_config(env: &Env, fee_bps: u32, fee_recipient: Address, token_address: Address) {
    assert!(fee_bps <= 10_000, "fee_bps must be <= 10000");
    env.storage()
        .instance()
        .set(&EscrowDataKey::FeeBps, &fee_bps);
    env.storage()
        .instance()
        .set(&EscrowDataKey::FeeRecipient, &fee_recipient);
    if fee_bps > 10_000 {
        panic_with_error!(env, EscrowError::FeeBpsTooHigh);
    }
    env.storage().instance().set(&EscrowDataKey::FeeBps, &fee_bps);
    env.storage().instance().set(&EscrowDataKey::FeeRecipient, &fee_recipient);
    env.storage().instance().set(&EscrowDataKey::TokenAddress, &token_address);
    // Store the initial caller as Admin (first call sets the admin)
    if !env.storage().instance().has(&EscrowDataKey::Admin) {
        env.storage()
            .instance()
            .set(&EscrowDataKey::Admin, &fee_recipient);
    }
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
    assert!(
        !env.storage().persistent().has(&key),
        "escrow already exists"
    );
    if env.storage().persistent().has(&key) {
        panic_with_error!(env, EscrowError::EscrowAlreadyExists);
    }
    let entry = EscrowEntry {
        transfer_id,
        buyer: buyer.clone(),
        seller: seller.clone(),
        amount,
        platform_fee_bps: get_platform_fee_bps(env),
        status: EscrowStatus::Held,
    };
    env.storage().persistent().set(&key, &entry);
    token_client(env).transfer(&buyer, &env.current_contract_address(), &amount);
    env.events().publish(
        (soroban_sdk::symbol_short!("FEE_HELD"), transfer_id),
        (buyer, amount),
    );
}

/// Releases escrowed fee to seller minus platform fee.
pub fn finalize_transfer(env: &Env, transfer_id: u64) {
    let key = EscrowDataKey::Entry(transfer_id);
    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&key)
        .expect("escrow not found");
    assert!(
        entry.status == EscrowStatus::Held,
        "cannot finalize: not in Held state"
    );
    let platform_fee = compute_platform_fee(entry.amount, entry.platform_fee_bps);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
    let platform_fee  = compute_platform_fee(entry.amount, entry.platform_fee_bps);
    let seller_amount = entry.amount - platform_fee;
    let contract = env.current_contract_address();
    let client   = token_client(env);
    client.transfer(&contract, &entry.seller, &seller_amount);
    if platform_fee > 0 {
        let fee_recipient = get_fee_recipient(env).expect("fee recipient not configured");
        client.transfer(&contract, &fee_recipient, &platform_fee);
    }
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
    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&key)
        .expect("escrow not found");
    assert!(
        entry.status == EscrowStatus::Held || entry.status == EscrowStatus::Disputed,
        "cannot refund: invalid state"
    );
    token_client(env).transfer(&env.current_contract_address(), &entry.buyer, &entry.amount);
    let mut entry: EscrowEntry = match env.storage().persistent().get(&key) {
        Some(entry) => entry,
        None => panic_with_error!(env, EscrowError::EscrowNotFound),
    };
    if entry.status != EscrowStatus::Held && entry.status != EscrowStatus::Disputed {
        panic_with_error!(env, EscrowError::InvalidEscrowState);
    }
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
    let mut entry: EscrowEntry = env
        .storage()
        .persistent()
        .get(&key)
        .expect("escrow not found");
    assert!(
        entry.status == EscrowStatus::Held,
        "cannot dispute: not in Held state"
    );
    assert!(
        initiator == entry.buyer || initiator == entry.seller,
        "only buyer or seller may dispute"
    );
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

pub fn get_escrow(env: &Env, transfer_id: u64) -> Option<EscrowEntry> {
    env.storage()
        .persistent()
        .get(&EscrowDataKey::Entry(transfer_id))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PetOwnershipContract;
    use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env};

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
    use soroban_sdk::{testutils::Address as _, Address, Env};

    // Storage access requires a contract context in tests, so setup
    // registers a contract and tests run inside env.as_contract.
    fn setup() -> (Env, Address, Address, Address, Address) {
        let env      = Env::default();
        env.mock_all_auths();
        let contract = env.register_contract(None, crate::PetOwnershipContract);
    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let buyer = Address::generate(&env);
        let seller = Address::generate(&env);
    /// Escrow config lives in instance storage, so every test body must run
    /// inside a contract frame — `setup` returns the registered contract id.
    fn setup() -> (Env, Address, Address, Address, Address) {
        let env      = Env::default();
        env.mock_all_auths();
        let contract = env.register_contract(None, PetOwnershipContract);
        let buyer    = Address::generate(&env);
        let seller   = Address::generate(&env);
        let platform = Address::generate(&env);
        (env, contract, buyer, seller, platform)
    }

    fn deposit_creates_held_entry() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 1, c.buyer.clone(), c.seller.clone(), 10_000_000);
            let e = get_escrow(&c.env, 1).unwrap();
            assert_eq!(e.status, EscrowStatus::Held);
            assert_eq!(e.amount, 10_000_000);
            assert_eq!(e.buyer, c.buyer);
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
        let (env, buyer, seller, platform) = setup();
        init_escrow_config(&env, 250, platform);
        deposit_fee(&env, 1, buyer.clone(), seller, 10_000_000);
        let e = get_escrow(&env, 1).unwrap();
        assert_eq!(e.status, EscrowStatus::Held);
        assert_eq!(e.amount, 10_000_000);
        assert_eq!(e.buyer, buyer);
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 1, buyer.clone(), seller, 10_000_000);
            let e = get_escrow(&env, 1).unwrap();
            assert_eq!(e.status, EscrowStatus::Held);
            assert_eq!(e.amount, 10_000_000);
            assert_eq!(e.buyer,  buyer);
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
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 2, buyer, seller, 10_000_000);
            finalize_transfer(&env, 2);
            assert_eq!(get_escrow(&env, 2).unwrap().status, EscrowStatus::Released);
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
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 3, buyer, seller, 5_000_000);
            refund_fee(&env, 3);
            assert_eq!(get_escrow(&env, 3).unwrap().status, EscrowStatus::Refunded);
        });
    }

    #[test]
    #[should_panic]
    fn cannot_finalize_twice() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 4, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 4);
            finalize_transfer(&c.env, 4);
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 4, buyer, seller, 1_000_000);
            finalize_transfer(&env, 4);
            finalize_transfer(&env, 4);
        });
    }

    #[test]
    #[should_panic]
    fn cannot_refund_after_release() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 100, c.platform.clone(), c.token.clone());
            deposit_fee(&c.env, 5, c.buyer.clone(), c.seller.clone(), 1_000_000);
            finalize_transfer(&c.env, 5);
            refund_fee(&c.env, 5);
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 5, buyer, seller, 1_000_000);
            finalize_transfer(&env, 5);
            refund_fee(&env, 5);
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
        let (env, contract, buyer, seller, platform) = setup();
        // require_auth may only run once per frame; each call gets its own.
        env.as_contract(&contract, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 6, buyer.clone(), seller, 2_000_000);
        });
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 6, buyer.clone(), seller, 2_000_000);
        });
        env.as_contract(&cid, || {
            dispute_transfer(&env, 6, buyer);
            assert_eq!(get_escrow(&env, 6).unwrap().status, EscrowStatus::Disputed);
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
        let (env, contract, buyer, seller, platform) = setup();
        env.as_contract(&contract, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 7, buyer.clone(), seller, 3_000_000);
        });
        env.as_contract(&contract, || {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 100, platform);
            deposit_fee(&env, 7, buyer.clone(), seller, 3_000_000);
        });
        env.as_contract(&cid, || {
            dispute_transfer(&env, 7, buyer);
            refund_fee(&env, 7);
            assert_eq!(get_escrow(&env, 7).unwrap().status, EscrowStatus::Refunded);
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
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            // transfer_id 999 was never deposited
            finalize_transfer(&env, 999);
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
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            // transfer_id 999 was never deposited
            refund_fee(&env, 999);
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
        let (env, cid, buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            // transfer_id 999 was never deposited
            dispute_transfer(&env, 999, buyer);
        });
    }

    // ── Issue #1004: get_fee_bps and get_fee_recipient view functions ─────────

    #[test]
    fn get_fee_bps_returns_configured_value() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 300, c.platform.clone(), c.token.clone());
            assert_eq!(get_fee_bps(&c.env), 300);
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 300, platform);
            assert_eq!(get_fee_bps(&env), 300);
        });
    }

    #[test]
    fn get_fee_bps_returns_zero_when_not_configured() {
        let c = setup();
        c.run(|| assert_eq!(get_fee_bps(&c.env), 0));
        let (env, cid, _buyer, _seller, _platform) = setup();
        env.as_contract(&cid, || {
            assert_eq!(get_fee_bps(&env), 0);
        });
    }

    #[test]
    fn get_fee_recipient_returns_configured_address() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 150, c.platform.clone(), c.token.clone());
            assert_eq!(get_fee_recipient(&c.env), Some(c.platform.clone()));
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 150, platform.clone());
            assert_eq!(get_fee_recipient(&env), Some(platform));
        });
    }

    #[test]
    fn get_fee_recipient_returns_none_when_not_configured() {
        let c = setup();
        c.run(|| assert_eq!(get_fee_recipient(&c.env), None));
        let (env, cid, _buyer, _seller, _platform) = setup();
        env.as_contract(&cid, || {
            assert_eq!(get_fee_recipient(&env), None);
        });
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
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            let new_recipient = Address::generate(&env);
            init_escrow_config(&env, 250, platform.clone());

            update_fee_config(&env, 100, new_recipient.clone());

            assert_eq!(get_fee_bps(&env), 100);
            assert_eq!(get_fee_recipient(&env), Some(new_recipient));
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
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            let new_recipient = Address::generate(&env);
            init_escrow_config(&env, 250, platform.clone());

            // Update fee to 500 bps (5%)
            update_fee_config(&env, 500, new_recipient.clone());

            // New deposit captures the updated rate
            deposit_fee(&env, 10, buyer.clone(), seller.clone(), 10_000_000);
            let entry = get_escrow(&env, 10).unwrap();
            assert_eq!(entry.platform_fee_bps, 500);
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
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            let new_recipient = Address::generate(&env);
            init_escrow_config(&env, 250, platform.clone());

            // Deposit before update — captures 250 bps
            deposit_fee(&env, 20, buyer.clone(), seller.clone(), 10_000_000);

            // Update fee to 500 bps
            update_fee_config(&env, 500, new_recipient.clone());

            // Existing escrow is unchanged
            let entry = get_escrow(&env, 20).unwrap();
            assert_eq!(entry.platform_fee_bps, 250);
        });
    }

    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn update_fee_config_rejects_fee_bps_over_10000() {
        let c = setup();
        c.run(|| {
            init_escrow_config(&c.env, 250, c.platform.clone(), c.token.clone());
            update_fee_config(&c.env, 10_001, c.platform.clone());
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform.clone());
            update_fee_config(&env, 10_001, platform);
        });
    }

    // ── Issue #1002: typed EscrowError variants ──────────────────────────────

    /// init_escrow_config rejects a fee above 100% with FeeBpsTooHigh.
    #[test]
    #[should_panic(expected = "Error(Contract, #1)")]
    fn init_escrow_config_rejects_fee_bps_over_10000() {
        let (env, cid, _buyer, _seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 10_001, platform);
        });
    }

    /// deposit_fee rejects a non-positive amount with InvalidAmount.
    #[test]
    #[should_panic(expected = "Error(Contract, #2)")]
    fn deposit_fee_rejects_non_positive_amount() {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 30, buyer, seller, 0);
        });
    }

    /// A second deposit under the same transfer_id fails with EscrowAlreadyExists.
    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn deposit_fee_rejects_duplicate_transfer_id() {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 31, buyer.clone(), seller.clone(), 1_000_000);
        });
        env.as_contract(&cid, || {
            deposit_fee(&env, 31, buyer, seller, 1_000_000);
        });
    }

    /// Finalizing an already-released escrow fails with InvalidEscrowState.
    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn finalize_transfer_rejects_non_held_state() {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 32, buyer, seller, 1_000_000);
            finalize_transfer(&env, 32);
            finalize_transfer(&env, 32);
        });
    }

    /// Refunding an already-released escrow fails with InvalidEscrowState.
    #[test]
    #[should_panic(expected = "Error(Contract, #5)")]
    fn refund_fee_rejects_non_refundable_state() {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 33, buyer, seller, 1_000_000);
            finalize_transfer(&env, 33);
            refund_fee(&env, 33);
        });
    }

    /// A third party cannot dispute an escrow — Unauthorized.
    #[test]
    #[should_panic(expected = "Error(Contract, #6)")]
    fn dispute_transfer_rejects_third_party() {
        let (env, cid, buyer, seller, platform) = setup();
        env.as_contract(&cid, || {
            let stranger = Address::generate(&env);
            init_escrow_config(&env, 250, platform);
            deposit_fee(&env, 34, buyer, seller, 1_000_000);
            dispute_transfer(&env, 34, stranger);
        });
    }
}
