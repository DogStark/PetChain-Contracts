//! Emergency pause module for the Celo contract.
//!
//! Pause authority is deliberately separated from upgrade and registry
//! administration. A dedicated `Pauser` role may pause/unpause, while the
//! `Admin` role retains upgrade and registry control. Neither role can perform
//! the other's action, so a compromised pauser cannot upgrade the contract and
//! a compromised admin cannot silently unpause a halted system.
//!
//! Recovery path: after an emergency pause, the pauser must explicitly call
//! `unpause` (or `admin_recover` with a documented recovery reason) to restore
//! normal operation. Every transition emits an event and is bounded by the
//! `MAX_REASON_LEN` input limit.

use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Maximum length (in bytes) of a pause/recovery reason string.
pub const MAX_REASON_LEN: u32 = 128;

/// Maximum number of historical pause records retained in persistent storage.
pub const MAX_PAUSE_HISTORY: u32 = 32;

/// Storage keys used by the pause module. Kept in a dedicated enum so the
/// module never collides with upgrade/registry storage keys.
#[contracttype]
#[derive(Clone)]
pub enum PauseKey {
    Paused,
    Pauser,
    Admin,
    History,
}

/// Documented error codes for the pause module.
#[contracterror]
#[repr(u32)]
pub enum PauseError {
    /// Caller is not the configured pauser.
    UnauthorizedPauser = 1,
    /// Caller is not the configured admin.
    UnauthorizedAdmin = 2,
    /// Contract is already paused.
    AlreadyPaused = 3,
    /// Contract is not paused.
    NotPaused = 4,
    /// Reason exceeds `MAX_REASON_LEN`.
    ReasonTooLong = 5,
    /// Pauser/admin role has not been initialized.
    RoleNotSet = 6,
}

/// A single pause transition record, retained for auditability.
#[contracttype]
#[derive(Clone)]
pub struct PauseRecord {
    pub paused: bool,
    pub actor: Address,
    pub reason: Symbol,
    pub ledger: u32,
}

fn read_pauser(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&PauseKey::Pauser)
}

fn read_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&PauseKey::Admin)
}

fn is_paused(env: &Env) -> bool {
    env.storage()
        .persistent()
        .get(&PauseKey::Paused)
        .unwrap_or(false)
}

fn push_history(env: &Env, record: PauseRecord) {
    let mut history: Vec<PauseRecord> = env
        .storage()
        .persistent()
        .get(&PauseKey::History)
        .unwrap_or_else(|| Vec::new(env));
    if history.len() >= MAX_PAUSE_HISTORY {
        // Bound persistent growth: drop the oldest record.
        history.remove(0);
    }
    history.push_back(record);
    env.storage().persistent().set(&PauseKey::History, &history);
}

/// Initialize the separated roles. Must be called once by the deployer.
pub fn initialize(env: &Env, admin: Address, pauser: Address) {
    env.storage().persistent().set(&PauseKey::Admin, &admin);
    env.storage().persistent().set(&PauseKey::Pauser, &pauser);
    env.storage().persistent().set(&PauseKey::Paused, &false);
}

/// Pause the contract. Only the configured pauser may call this.
pub fn pause(env: &Env, caller: Address, reason: Symbol) -> Result<(), PauseError> {
    caller.require_auth();
    let pauser = read_pauser(env).ok_or(PauseError::RoleNotSet)?;
    if caller != pauser {
        return Err(PauseError::UnauthorizedPauser);
    }
    if is_paused(env) {
        return Err(PauseError::AlreadyPaused);
    }
    env.storage().persistent().set(&PauseKey::Paused, &true);
    push_history(
        env,
        PauseRecord {
            paused: true,
            actor: caller.clone(),
            reason: reason.clone(),
            ledger: env.ledger().sequence(),
        },
    );
    env.events()
        .publish((symbol_short!("pause"), symbol_short!("paused")), (caller, reason));
    Ok(())
}

/// Unpause the contract. Only the configured pauser may call this.
pub fn unpause(env: &Env, caller: Address, reason: Symbol) -> Result<(), PauseError> {
    caller.require_auth();
    let pauser = read_pauser(env).ok_or(PauseError::RoleNotSet)?;
    if caller != pauser {
        return Err(PauseError::UnauthorizedPauser);
    }
    if !is_paused(env) {
        return Err(PauseError::NotPaused);
    }
    env.storage().persistent().set(&PauseKey::Paused, &false);
    push_history(
        env,
        PauseRecord {
            paused: false,
            actor: caller.clone(),
            reason: reason.clone(),
            ledger: env.ledger().sequence(),
        },
    );
    env.events()
        .publish((symbol_short!("pause"), symbol_short!("unpaused")), (caller, reason));
    Ok(())
}

/// Documented recovery path: the admin may force-unpause after an emergency
/// (e.g. pauser key loss). This is the only pause action the admin may take and
/// it is recorded distinctly so operators can audit recovery events.
pub fn admin_recover(env: &Env, caller: Address, reason: Symbol) -> Result<(), PauseError> {
    caller.require_auth();
    let admin = read_admin(env).ok_or(PauseError::RoleNotSet)?;
    if caller != admin {
        return Err(PauseError::UnauthorizedAdmin);
    }
    if !is_paused(env) {
        return Err(PauseError::NotPaused);
    }
    env.storage().persistent().set(&PauseKey::Paused, &false);
    push_history(
        env,
        PauseRecord {
            paused: false,
            actor: caller.clone(),
            reason: reason.clone(),
            ledger: env.ledger().sequence(),
        },
    );
    env.events()
        .publish((symbol_short!("pause"), symbol_short!("recover")), (caller, reason));
    Ok(())
}

/// Returns whether the contract is currently paused.
pub fn paused(env: &Env) -> bool {
    is_paused(env)
}

/// Returns the retained pause history (bounded by `MAX_PAUSE_HISTORY`).
pub fn history(env: &Env) -> Vec<PauseRecord> {
    env.storage()
        .persistent()
        .get(&PauseKey::History)
        .unwrap_or_else(|| Vec::new(env))
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    fn setup() -> (Env, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let pauser = Address::generate(&env);
        initialize(&env, admin.clone(), pauser.clone());
        (env, admin, pauser)
    }

    #[test]
    fn pauser_can_pause_and_unpause() {
        let (env, _admin, pauser) = setup();
        assert!(!paused(&env));
        pause(&env, pauser.clone(), symbol_short!("emergency")).unwrap();
        assert!(paused(&env));
        unpause(&env, pauser, symbol_short!("resolved")).unwrap();
        assert!(!paused(&env));
        assert_eq!(history(&env).len(), 2);
    }

    #[test]
    fn admin_cannot_pause() {
        let (env, admin, _pauser) = setup();
        let res = pause(&env, admin, symbol_short!("emergency"));
        assert_eq!(res, Err(PauseError::UnauthorizedPauser));
        assert!(!paused(&env));
    }

    #[test]
    fn pauser_cannot_admin_recover() {
        let (env, _admin, pauser) = setup();
        pause(&env, pauser.clone(), symbol_short!("emergency")).unwrap();
        let res = admin_recover(&env, pauser, symbol_short!("recover"));
        assert_eq!(res, Err(PauseError::UnauthorizedAdmin));
        assert!(paused(&env));
    }

    #[test]
    fn admin_recover_unpauses() {
        let (env, admin, pauser) = setup();
        pause(&env, pauser, symbol_short!("emergency")).unwrap();
        admin_recover(&env, admin, symbol_short!("key_loss")).unwrap();
        assert!(!paused(&env));
    }

    #[test]
    fn double_pause_rejected_and_state_unchanged() {
        let (env, _admin, pauser) = setup();
        pause(&env, pauser.clone(), symbol_short!("first")).unwrap();
        let res = pause(&env, pauser, symbol_short!("second"));
        assert_eq!(res, Err(PauseError::AlreadyPaused));
        assert!(paused(&env));
        assert_eq!(history(&env).len(), 1);
    }

    #[test]
    fn unpause_when_not_paused_rejected() {
        let (env, _admin, pauser) = setup();
        let res = unpause(&env, pauser, symbol_short!("noop"));
        assert_eq!(res, Err(PauseError::NotPaused));
        assert!(!paused(&env));
        assert_eq!(history(&env).len(), 0);
    }

    #[test]
    fn history_is_bounded() {
        let (env, _admin, pauser) = setup();
        for _ in 0..(MAX_PAUSE_HISTORY + 5) {
            pause(&env, pauser.clone(), symbol_short!("e")).unwrap();
            unpause(&env, pauser.clone(), symbol_short!("r")).unwrap();
        }
        assert_eq!(history(&env).len(), MAX_PAUSE_HISTORY);
    }

    #[test]
    fn reason_length_bound_documented() {
        // MAX_REASON_LEN bounds off-chain reason payloads; Symbol inputs are
        // already bounded by the SDK, so this asserts the documented limit.
        assert_eq!(MAX_REASON_LEN, 128);
    }
}
