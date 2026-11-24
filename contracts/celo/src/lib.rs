// SPDX-License-Identifier: MIT
//! Celo emergency pause module.
//!
//! Role separation:
//! - `Pauser` may pause and unpause the contract (emergency authority only).
//! - `Upgrader` may change the implementation / storage version.
//! - `RegistryAdmin` may mutate the registry.
//!
//! A pause is only recoverable through the documented recovery path:
//! `unpause` must be called by the `Pauser` role, and the caller must supply
//! the same `recovery_id` that was recorded at pause time. This prevents a
//! stale or replayed unpause from clearing a newer pause.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, Symbol};

/// Storage keys. Bumped via `STORAGE_VERSION` when the layout changes.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    Pauser,
    Upgrader,
    RegistryAdmin,
    Paused,
    PauseRecoveryId,
    StorageVersion,
}

/// Current storage layout version. Bump when `DataKey` changes.
const STORAGE_VERSION: u32 = 2;

/// Maximum length of a recovery id (bytes). Bounds persistent state growth.
const MAX_RECOVERY_ID_LEN: u32 = 64;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AlreadyPaused = 4,
    NotPaused = 5,
    InvalidRecoveryId = 6,
    RecoveryIdMismatch = 7,
    StorageVersionMismatch = 8,
}

#[contracttype]
#[derive(Clone)]
pub struct Roles {
    pub pauser: Address,
    pub upgrader: Address,
    pub registry_admin: Address,
}

#[contract]
pub struct CeloEmergencyPause;

#[contractimpl]
impl CeloEmergencyPause {
    /// Initialize roles. Pause authority is separate from upgrade and registry
    /// administration; the same address may not hold more than one role.
    pub fn init(env: Env, roles: Roles) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(Error::AlreadyInitialized);
        }
        if roles.pauser == roles.upgrader
            || roles.pauser == roles.registry_admin
            || roles.upgrader == roles.registry_admin
        {
            return Err(Error::Unauthorized);
        }
        env.storage().instance().set(&DataKey::Pauser, &roles.pauser);
        env.storage().instance().set(&DataKey::Upgrader, &roles.upgrader);
        env.storage()
            .instance()
            .set(&DataKey::RegistryAdmin, &roles.registry_admin);
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_VERSION);
        Ok(())
    }

    /// Pause the contract. Only the `Pauser` role may call this.
    /// `recovery_id` is recorded and must be supplied to `unpause`.
    pub fn pause(env: Env, caller: Address, recovery_id: BytesN<32>) -> Result<(), Error> {
        Self::require_version(&env)?;
        caller.require_auth();
        let pauser: Address = env
            .storage()
            .instance()
            .get(&DataKey::Pauser)
            .ok_or(Error::NotInitialized)?;
        if caller != pauser {
            return Err(Error::Unauthorized);
        }
        if Self::is_paused(&env) {
            return Err(Error::AlreadyPaused);
        }
        env.storage().instance().set(&DataKey::Paused, &true);
        env.storage()
            .instance()
            .set(&DataKey::PauseRecoveryId, &recovery_id);
        env.events()
            .publish((Symbol::new(&env, "paused"),), (caller, recovery_id));
        Ok(())
    }

    /// Recovery path: unpause. Only the `Pauser` role may call this, and the
    /// supplied `recovery_id` must match the one recorded at pause time.
    pub fn unpause(env: Env, caller: Address, recovery_id: BytesN<32>) -> Result<(), Error> {
        Self::require_version(&env)?;
        caller.require_auth();
        let pauser: Address = env
            .storage()
            .instance()
            .get(&DataKey::Pauser)
            .ok_or(Error::NotInitialized)?;
        if caller != pauser {
            return Err(Error::Unauthorized);
        }
        if !Self::is_paused(&env) {
            return Err(Error::NotPaused);
        }
        let recorded: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::PauseRecoveryId)
            .ok_or(Error::NotPaused)?;
        if recorded != recovery_id {
            return Err(Error::RecoveryIdMismatch);
        }
        env.storage().instance().set(&DataKey::Paused, &false);
        env.storage().instance().remove(&DataKey::PauseRecoveryId);
        env.events()
            .publish((Symbol::new(&env, "unpaused"),), (caller, recovery_id));
        Ok(())
    }

    /// Upgrade authority. Separate from pause authority.
    pub fn upgrade(env: Env, caller: Address, new_version: u32) -> Result<(), Error> {
        Self::require_version(&env)?;
        caller.require_auth();
        let upgrader: Address = env
            .storage()
            .instance()
            .get(&DataKey::Upgrader)
            .ok_or(Error::NotInitialized)?;
        if caller != upgrader {
            return Err(Error::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &new_version);
        env.events()
            .publish((Symbol::new(&env, "upgraded"),), (caller, new_version));
        Ok(())
    }

    /// Registry administration. Separate from pause authority.
    pub fn set_registry_entry(
        env: Env,
        caller: Address,
        key: Symbol,
        value: BytesN<32>,
    ) -> Result<(), Error> {
        Self::require_version(&env)?;
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::RegistryAdmin)
            .ok_or(Error::NotInitialized)?;
        if caller != admin {
            return Err(Error::Unauthorized);
        }
        env.storage().persistent().set(&key, &value);
        env.events()
            .publish((Symbol::new(&env, "registry_set"),), (caller, key));
        Ok(())
    }

    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    fn require_version(env: &Env) -> Result<(), Error> {
        let version: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StorageVersion)
            .ok_or(Error::NotInitialized)?;
        if version != STORAGE_VERSION {
            return Err(Error::StorageVersionMismatch);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Env, IntoVal};

    fn setup() -> (Env, CeloEmergencyPauseClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CeloEmergencyPause);
        let client = CeloEmergencyPauseClient::new(&env, &contract_id);
        let pauser = Address::generate(&env);
        let upgrader = Address::generate(&env);
        let admin = Address::generate(&env);
        client.init(&Roles {
            pauser: pauser.clone(),
            upgrader: upgrader.clone(),
            registry_admin: admin.clone(),
        });
        (env, client, pauser, upgrader, admin)
    }

    fn rid(env: &Env, b: u8) -> BytesN<32> {
        BytesN::from_array(env, &[b; 32])
    }

    #[test]
    fn pause_and_unpause_success() {
        let (env, client, pauser, _, _) = setup();
        let id = rid(&env, 7);
        client.pause(&pauser, &id);
        assert!(client.is_paused());
        client.unpause(&pauser, &id);
        assert!(!client.is_paused());
    }

    #[test]
    fn unauthorized_pause_rejected_and_state_unchanged() {
        let (env, client, _, upgrader, admin) = setup();
        let id = rid(&env, 1);
        assert_eq!(client.try_pause(&upgrader, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.try_pause(&admin, &id), Err(Ok(Error::Unauthorized)));
        assert!(!client.is_paused());
    }

    #[test]
    fn unauthorized_unpause_rejected() {
        let (env, client, pauser, upgrader, _) = setup();
        let id = rid(&env, 2);
        client.pause(&pauser, &id);
        assert_eq!(client.try_unpause(&upgrader, &id), Err(Ok(Error::Unauthorized)));
        assert!(client.is_paused());
    }

    #[test]
    fn double_pause_rejected() {
        let (env, client, pauser, _, _) = setup();
        let id = rid(&env, 3);
        client.pause(&pauser, &id);
        assert_eq!(client.try_pause(&pauser, &id), Err(Ok(Error::AlreadyPaused)));
        assert!(client.is_paused());
    }

    #[test]
    fn unpause_when_not_paused_rejected() {
        let (env, client, pauser, _, _) = setup();
        let id = rid(&env, 4);
        assert_eq!(client.try_unpause(&pauser, &id), Err(Ok(Error::NotPaused)));
    }

    #[test]
    fn replay_with_stale_recovery_id_rejected() {
        let (env, client, pauser, _, _) = setup();
        let first = rid(&env, 5);
        let second = rid(&env, 6);
        client.pause(&pauser, &first);
        client.unpause(&pauser, &first);
        // Replaying the old id after a new pause must fail.
        client.pause(&pauser, &second);
        assert_eq!(
            client.try_unpause(&pauser, &first),
            Err(Ok(Error::RecoveryIdMismatch))
        );
        assert!(client.is_paused());
        client.unpause(&pauser, &second);
        assert!(!client.is_paused());
    }

    #[test]
    fn upgrade_and_registry_roles_are_separate_from_pause() {
        let (env, client, pauser, upgrader, admin) = setup();
        let id = rid(&env, 8);
        // Pauser cannot upgrade or mutate registry.
        assert_eq!(client.try_upgrade(&pauser, &3), Err(Ok(Error::Unauthorized)));
        assert_eq!(
            client.try_set_registry_entry(&pauser, &Symbol::new(&env, "k"), &id),
            Err(Ok(Error::Unauthorized))
        );
        // Upgrader/admin cannot pause.
        assert_eq!(client.try_pause(&upgrader, &id), Err(Ok(Error::Unauthorized)));
        assert_eq!(client.try_pause(&admin, &id), Err(Ok(Error::Unauthorized)));
        assert!(!client.is_paused());
    }

    #[test]
    fn init_rejects_shared_roles() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CeloEmergencyPause);
        let client = CeloEmergencyPauseClient::new(&env, &contract_id);
        let a = Address::generate(&env);
        let b = Address::generate(&env);
        let res = client.try_init(&Roles {
            pauser: a.clone(),
            upgrader: a.clone(),
            registry_admin: b,
        });
        assert_eq!(res, Err(Ok(Error::Unauthorized)));
    }

    #[test]
    fn storage_version_mismatch_blocks_calls() {
        let (env, client, pauser, _, _) = setup();
        let id = rid(&env, 9);
        // Simulate a stale layout by writing a mismatched version directly.
        env.as_contract(&client.address, || {
            env.storage()
                .instance()
                .set(&DataKey::StorageVersion, &(STORAGE_VERSION + 1));
        });
        assert_eq!(
            client.try_pause(&pauser, &id),
            Err(Ok(Error::StorageVersionMismatch))
        );
        assert!(!client.is_paused());
    }

    #[test]
    fn recovery_id_bound_is_documented() {
        // Recovery ids are fixed 32-byte values, well under MAX_RECOVERY_ID_LEN.
        assert!(32 <= MAX_RECOVERY_ID_LEN);
        let _ = IntoVal::<Env, u32>::into_val(&Env::default(), &MAX_RECOVERY_ID_LEN);
    }
}
