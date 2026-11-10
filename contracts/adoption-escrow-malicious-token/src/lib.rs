#![no_std]
#![allow(clippy::too_many_arguments)]

//! Malicious-token integration fixture and hardened adoption escrow (Issue #1507).
//!
//! Provides:
//! 1. `MaliciousTokenContract`: a configurable Soroban test token that can act
//!    honestly, silently no-op on transfer ("lying transfer"), under-deliver
//!    tokens ("short transfer"), revert explicitly, or attempt a reentrant
//!    callback into the escrow contract during `transfer`.
//! 2. `HardenedAdoptionEscrowContract`: an adoption fee escrow enforcing
//!    checks-effects-interactions, an explicit reentrancy lock, pre/post
//!    balance delta verification (`actual_delta == amount`), and replay/expiry
//!    guards.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, IntoVal,
    Symbol, Vec,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 1;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const MAX_ESCROW_AMOUNT: i128 = 1_000_000_000_000;
pub const MAX_ACTIVE_ESCROWS: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum EscrowError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    EscrowNotFound = 5,
    EscrowAlreadySettled = 6,
    EscrowExpired = 7,
    EscrowNotExpired = 8,
    ReentrancyDetected = 9,
    TokenTransferFailed = 10,
    TokenBalanceDeltaMismatch = 11,
    ReplayDetected = 12,
    ActiveEscrowLimitReached = 13,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TokenBehaviorMode {
    Honest = 0,
    SilentNoOpLie = 1,
    ShortDeliveryHalf = 2,
    ExplicitRevert = 3,
    ReentrantReleaseAttempt = 4,
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum EscrowState {
    Funded = 1,
    Released = 2,
    Refunded = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptionEscrowRecord {
    pub escrow_id: u64,
    pub pet_id: u64,
    pub adopter: Address,
    pub shelter: Address,
    pub token: Address,
    pub amount: i128,
    pub state: EscrowState,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EscrowSettledEvent {
    pub version: u32,
    pub escrow_id: u64,
    pub pet_id: u64,
    pub state: EscrowState,
    pub amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowKey {
    StorageVersion,
    Admin,
    ReentrancyLock,
    ActiveCount,
    Escrow(u64),
    UsedNonce((Address, u64)),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKey {
    Mode,
    Balance(Address),
    CallbackEscrow,
    CallbackEscrowId,
    CallbackActor,
}

#[contract]
pub struct MaliciousTokenContract;

#[contractimpl]
impl MaliciousTokenContract {
    pub fn init_token(env: Env, mode: TokenBehaviorMode) {
        env.storage().instance().set(&TokenKey::Mode, &mode);
    }

    pub fn set_mode(env: Env, mode: TokenBehaviorMode) {
        env.storage().instance().set(&TokenKey::Mode, &mode);
    }

    pub fn configure_reentry(env: Env, escrow_contract: Address, escrow_id: u64, actor: Address) {
        env.storage()
            .instance()
            .set(&TokenKey::CallbackEscrow, &escrow_contract);
        env.storage()
            .instance()
            .set(&TokenKey::CallbackEscrowId, &escrow_id);
        env.storage()
            .instance()
            .set(&TokenKey::CallbackActor, &actor);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let prev = Self::balance(env.clone(), to.clone());
        env.storage()
            .persistent()
            .set(&TokenKey::Balance(to), &(prev + amount));
    }

    pub fn balance(env: Env, owner: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&TokenKey::Balance(owner))
            .unwrap_or(0)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> bool {
        let mode: TokenBehaviorMode = env
            .storage()
            .instance()
            .get(&TokenKey::Mode)
            .unwrap_or(TokenBehaviorMode::Honest);

        match mode {
            TokenBehaviorMode::Honest => {
                let from_bal = Self::balance(env.clone(), from.clone());
                if from_bal < amount {
                    return false;
                }
                let to_bal = Self::balance(env.clone(), to.clone());
                env.storage()
                    .persistent()
                    .set(&TokenKey::Balance(from), &(from_bal - amount));
                env.storage()
                    .persistent()
                    .set(&TokenKey::Balance(to), &(to_bal + amount));
                true
            }
            TokenBehaviorMode::SilentNoOpLie => {
                // Lies: returns true without moving any balances
                true
            }
            TokenBehaviorMode::ShortDeliveryHalf => {
                // Lies: deducts `amount` from sender but only credits `amount / 2` to recipient
                let from_bal = Self::balance(env.clone(), from.clone());
                if from_bal < amount {
                    return false;
                }
                let to_bal = Self::balance(env.clone(), to.clone());
                env.storage()
                    .persistent()
                    .set(&TokenKey::Balance(from), &(from_bal - amount));
                env.storage()
                    .persistent()
                    .set(&TokenKey::Balance(to), &(to_bal + (amount / 2)));
                true
            }
            TokenBehaviorMode::ExplicitRevert => false,
            TokenBehaviorMode::ReentrantReleaseAttempt => {
                if let (Some(escrow_addr), Some(escrow_id), Some(actor)) = (
                    env.storage()
                        .instance()
                        .get::<TokenKey, Address>(&TokenKey::CallbackEscrow),
                    env.storage()
                        .instance()
                        .get::<TokenKey, u64>(&TokenKey::CallbackEscrowId),
                    env.storage()
                        .instance()
                        .get::<TokenKey, Address>(&TokenKey::CallbackActor),
                ) {
                    // Attempt callback reentry into `release_escrow` while inside `transfer`
                    let mut args = Vec::new(&env);
                    args.push_back(actor.into_val(&env));
                    args.push_back(escrow_id.into_val(&env));
                    args.push_back(999u64.into_val(&env));
                    let _ = env.try_invoke_contract::<(), EscrowError>(
                        &escrow_addr,
                        &Symbol::new(&env, "release_escrow"),
                        args,
                    );
                }
                false
            }
        }
    }
}

#[contract]
pub struct HardenedAdoptionEscrowContract;

#[contractimpl]
impl HardenedAdoptionEscrowContract {
    pub fn init(env: Env, admin: Address) -> Result<(), EscrowError> {
        if env.storage().instance().has(&EscrowKey::StorageVersion) {
            return Err(EscrowError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&EscrowKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&EscrowKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&EscrowKey::ReentrancyLock, &false);
        env.storage().instance().set(&EscrowKey::ActiveCount, &0u32);
        Ok(())
    }

    pub fn fund_escrow(
        env: Env,
        adopter: Address,
        shelter: Address,
        token: Address,
        escrow_id: u64,
        pet_id: u64,
        amount: i128,
        nonce: u64,
        expires_at: u64,
    ) -> Result<AdoptionEscrowRecord, EscrowError> {
        Self::enter_non_reentrant(&env)?;
        let res = Self::fund_escrow_inner(
            &env, adopter, shelter, token, escrow_id, pet_id, amount, nonce, expires_at,
        );
        Self::exit_non_reentrant(&env);
        res
    }

    pub fn release_escrow(
        env: Env,
        caller: Address,
        escrow_id: u64,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        Self::enter_non_reentrant(&env)?;
        let res = Self::release_escrow_inner(&env, caller, escrow_id, nonce);
        Self::exit_non_reentrant(&env);
        res
    }

    pub fn refund_expired_escrow(
        env: Env,
        caller: Address,
        escrow_id: u64,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        Self::enter_non_reentrant(&env)?;
        let res = Self::refund_escrow_inner(&env, caller, escrow_id, nonce);
        Self::exit_non_reentrant(&env);
        res
    }

    pub fn get_escrow(env: Env, escrow_id: u64) -> Option<AdoptionEscrowRecord> {
        env.storage()
            .persistent()
            .get(&EscrowKey::Escrow(escrow_id))
    }

    fn fund_escrow_inner(
        env: &Env,
        adopter: Address,
        shelter: Address,
        token: Address,
        escrow_id: u64,
        pet_id: u64,
        amount: i128,
        nonce: u64,
        expires_at: u64,
    ) -> Result<AdoptionEscrowRecord, EscrowError> {
        if !env.storage().instance().has(&EscrowKey::StorageVersion) {
            return Err(EscrowError::NotInitialized);
        }
        adopter.require_auth();

        if amount <= 0 || amount > MAX_ESCROW_AMOUNT {
            return Err(EscrowError::InvalidAmount);
        }
        let now = env.ledger().timestamp();
        if expires_at <= now {
            return Err(EscrowError::EscrowExpired);
        }

        let nonce_key = EscrowKey::UsedNonce((adopter.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(EscrowError::ReplayDetected);
        }

        let active_count: u32 = env
            .storage()
            .instance()
            .get(&EscrowKey::ActiveCount)
            .unwrap_or(0);
        if active_count >= MAX_ACTIVE_ESCROWS {
            return Err(EscrowError::ActiveEscrowLimitReached);
        }

        let contract_addr = env.current_contract_address();
        Self::verified_token_transfer(env, &token, &adopter, &contract_addr, amount)?;

        let record = AdoptionEscrowRecord {
            escrow_id,
            pet_id,
            adopter,
            shelter,
            token,
            amount,
            state: EscrowState::Funded,
            expires_at,
        };
        env.storage()
            .persistent()
            .set(&EscrowKey::Escrow(escrow_id), &record);
        env.storage()
            .instance()
            .set(&EscrowKey::ActiveCount, &(active_count + 1));
        env.storage().persistent().set(&nonce_key, &true);

        Ok(record)
    }

    fn release_escrow_inner(
        env: &Env,
        caller: Address,
        escrow_id: u64,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        if !env.storage().instance().has(&EscrowKey::StorageVersion) {
            return Err(EscrowError::NotInitialized);
        }
        caller.require_auth();

        let nonce_key = EscrowKey::UsedNonce((caller.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(EscrowError::ReplayDetected);
        }

        let mut record: AdoptionEscrowRecord = env
            .storage()
            .persistent()
            .get(&EscrowKey::Escrow(escrow_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if record.state != EscrowState::Funded {
            return Err(EscrowError::EscrowAlreadySettled);
        }
        if caller != record.adopter {
            return Err(EscrowError::Unauthorized);
        }
        if env.ledger().timestamp() > record.expires_at {
            return Err(EscrowError::EscrowExpired);
        }

        // Checks-effects-interactions: mark Released before external token call
        record.state = EscrowState::Released;
        env.storage()
            .persistent()
            .set(&EscrowKey::Escrow(escrow_id), &record);
        env.storage().persistent().set(&nonce_key, &true);

        let contract_addr = env.current_contract_address();
        Self::verified_token_transfer(
            env,
            &record.token,
            &contract_addr,
            &record.shelter,
            record.amount,
        )?;

        env.events().publish(
            (symbol_short!("esc_done"), escrow_id),
            EscrowSettledEvent {
                version: EVENT_SCHEMA_VERSION,
                escrow_id,
                pet_id: record.pet_id,
                state: EscrowState::Released,
                amount: record.amount,
            },
        );
        Ok(())
    }

    fn refund_escrow_inner(
        env: &Env,
        caller: Address,
        escrow_id: u64,
        nonce: u64,
    ) -> Result<(), EscrowError> {
        if !env.storage().instance().has(&EscrowKey::StorageVersion) {
            return Err(EscrowError::NotInitialized);
        }
        caller.require_auth();

        let nonce_key = EscrowKey::UsedNonce((caller.clone(), nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(EscrowError::ReplayDetected);
        }

        let mut record: AdoptionEscrowRecord = env
            .storage()
            .persistent()
            .get(&EscrowKey::Escrow(escrow_id))
            .ok_or(EscrowError::EscrowNotFound)?;

        if record.state != EscrowState::Funded {
            return Err(EscrowError::EscrowAlreadySettled);
        }
        if caller != record.adopter {
            return Err(EscrowError::Unauthorized);
        }
        if env.ledger().timestamp() <= record.expires_at {
            return Err(EscrowError::EscrowNotExpired);
        }

        record.state = EscrowState::Refunded;
        env.storage()
            .persistent()
            .set(&EscrowKey::Escrow(escrow_id), &record);
        env.storage().persistent().set(&nonce_key, &true);

        let contract_addr = env.current_contract_address();
        Self::verified_token_transfer(
            env,
            &record.token,
            &contract_addr,
            &record.adopter,
            record.amount,
        )?;
        Ok(())
    }

    fn verified_token_transfer(
        env: &Env,
        token: &Address,
        from: &Address,
        to: &Address,
        amount: i128,
    ) -> Result<(), EscrowError> {
        let token_client = MaliciousTokenContractClient::new(env, token);
        let from_before = token_client.balance(from);
        let to_before = token_client.balance(to);

        let ok = token_client.transfer(from, to, &amount);
        if !ok {
            return Err(EscrowError::TokenTransferFailed);
        }

        let from_after = token_client.balance(from);
        let to_after = token_client.balance(to);

        if from_before - from_after != amount || to_after - to_before != amount {
            return Err(EscrowError::TokenBalanceDeltaMismatch);
        }
        Ok(())
    }

    fn enter_non_reentrant(env: &Env) -> Result<(), EscrowError> {
        let locked: bool = env
            .storage()
            .instance()
            .get(&EscrowKey::ReentrancyLock)
            .unwrap_or(false);
        if locked {
            return Err(EscrowError::ReentrancyDetected);
        }
        env.storage()
            .instance()
            .set(&EscrowKey::ReentrancyLock, &true);
        Ok(())
    }

    fn exit_non_reentrant(env: &Env) {
        env.storage()
            .instance()
            .set(&EscrowKey::ReentrancyLock, &false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (
        Env,
        HardenedAdoptionEscrowContractClient<'static>,
        Address,
        MaliciousTokenContractClient<'static>,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let escrow_id = env.register_contract(None, HardenedAdoptionEscrowContract);
        let escrow_client = HardenedAdoptionEscrowContractClient::new(&env, &escrow_id);

        let token_id = env.register_contract(None, MaliciousTokenContract);
        let token_client = MaliciousTokenContractClient::new(&env, &token_id);

        let admin = Address::generate(&env);
        escrow_client.init(&admin);
        token_client.init_token(&TokenBehaviorMode::Honest);

        (env, escrow_client, escrow_id, token_client, token_id)
    }

    #[test]
    fn test_honest_token_fund_and_release_preserves_invariants() {
        let (env, escrow, _escrow_addr, token, token_addr) = setup();
        let adopter = Address::generate(&env);
        let shelter = Address::generate(&env);

        token.mint(&adopter, &1_000);
        escrow.fund_escrow(&adopter, &shelter, &token_addr, &1, &101, &500, &1, &5_000);
        assert_eq!(token.balance(&adopter), 500);

        escrow.release_escrow(&adopter, &1, &2);
        assert_eq!(token.balance(&shelter), 500);
        assert_eq!(escrow.get_escrow(&1).unwrap().state, EscrowState::Released);
    }

    #[test]
    fn test_lying_and_short_transfer_tokens_rejected_and_escrow_state_unchanged() {
        let (env, escrow, _escrow_addr, token, token_addr) = setup();
        let adopter = Address::generate(&env);
        let shelter = Address::generate(&env);
        token.mint(&adopter, &1_000);

        // 1. Silent no-op lie
        token.set_mode(&TokenBehaviorMode::SilentNoOpLie);
        let res_lie = escrow.try_fund_escrow(
            &adopter,
            &shelter,
            &token_addr,
            &10,
            &101,
            &500,
            &10,
            &5_000,
        );
        assert_eq!(res_lie, Err(Ok(EscrowError::TokenBalanceDeltaMismatch)));
        assert!(escrow.get_escrow(&10).is_none());

        // 2. Short delivery (credits half)
        token.set_mode(&TokenBehaviorMode::ShortDeliveryHalf);
        let res_short = escrow.try_fund_escrow(
            &adopter,
            &shelter,
            &token_addr,
            &11,
            &101,
            &500,
            &11,
            &5_000,
        );
        assert_eq!(res_short, Err(Ok(EscrowError::TokenBalanceDeltaMismatch)));
        assert!(escrow.get_escrow(&11).is_none());
        // Atomic rollback ensures adopter still has 1_000
        assert_eq!(token.balance(&adopter), 1_000);
    }

    #[test]
    fn test_reentrant_callback_and_explicit_revert_preserve_funded_escrow() {
        let (env, escrow, escrow_addr, token, token_addr) = setup();
        let adopter = Address::generate(&env);
        let shelter = Address::generate(&env);
        token.mint(&adopter, &1_000);

        // Fund honestly first
        escrow.fund_escrow(
            &adopter,
            &shelter,
            &token_addr,
            &20,
            &202,
            &500,
            &20,
            &5_000,
        );

        // Configure token to attempt callback reentry during `release_escrow`
        token.set_mode(&TokenBehaviorMode::ReentrantReleaseAttempt);
        token.configure_reentry(&escrow_addr, &20, &adopter);

        let res_reentry = escrow.try_release_escrow(&adopter, &20, &21);
        assert_eq!(res_reentry, Err(Ok(EscrowError::TokenTransferFailed)));
        // Escrow remains Funded because outer transaction reverted cleanly
        assert_eq!(escrow.get_escrow(&20).unwrap().state, EscrowState::Funded);
    }
}
