#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

/// Storage schema version. Bump when the persistent layout changes.
const STORAGE_VERSION: u32 = 2;

/// Maximum length (in bytes) allowed for free-form text fields.
const MAX_TEXT_LEN: u32 = 256;

/// Maximum number of sightings retained per alert.
const MAX_SIGHTINGS: u32 = 32;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NotFound = 4,
    InvalidState = 5,
    InvalidDeadline = 6,
    TextTooLong = 7,
    TooManySightings = 8,
    InvalidVersion = 9,
}

/// Lifecycle states for a lost-pet alert.
///
/// Transitions (all closure transitions are idempotent):
/// - `Active` -> `Resolved`  (owner or resolver)
/// - `Active` -> `Withdrawn` (owner only)
/// - `Active` -> `Expired`   (anyone, once `deadline` has passed)
/// - `Resolved` / `Expired` / `Withdrawn` are terminal.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertState {
    Active = 0,
    Resolved = 1,
    Expired = 2,
    Withdrawn = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Alert {
    pub owner: Address,
    pub resolver: Address,
    pub pet_name: String,
    pub last_seen: String,
    pub deadline: u64,
    pub state: AlertState,
    pub sightings: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlertCreated {
    pub id: u64,
    pub owner: Address,
    pub deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlertClosed {
    pub id: u64,
    pub state: AlertState,
    pub actor: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SightingRecorded {
    pub id: u64,
    pub reporter: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub version: u32,
    pub admin: Address,
    pub next_id: u64,
}

fn config_key(env: &Env) -> soroban_sdk::Symbol {
    soroban_sdk::Symbol::new(env, "config")
}

fn alert_key(env: &Env, id: u64) -> (soroban_sdk::Symbol, u64) {
    (soroban_sdk::Symbol::new(env, "alert"), id)
}

fn load_config(env: &Env) -> Result<Config, Error> {
    let cfg: Config = env
        .storage()
        .persistent()
        .get(&config_key(env))
        .ok_or(Error::NotInitialized)?;
    if cfg.version != STORAGE_VERSION {
        return Err(Error::InvalidVersion);
    }
    Ok(cfg)
}

fn load_alert(env: &Env, id: u64) -> Result<Alert, Error> {
    env.storage()
        .persistent()
        .get(&alert_key(env, id))
        .ok_or(Error::NotFound)
}

fn save_alert(env: &Env, id: u64, alert: &Alert) {
    env.storage().persistent().set(&alert_key(env, id), alert);
}

#[contract]
pub struct LostPetContract;

#[contractimpl]
impl LostPetContract {
    /// Initialize the contract with an admin and the storage version.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().persistent().has(&config_key(&env)) {
            return Err(Error::AlreadyInitialized);
        }
        let cfg = Config {
            version: STORAGE_VERSION,
            admin,
            next_id: 1,
        };
        env.storage().persistent().set(&config_key(&env), &cfg);
        Ok(())
    }

    /// Create a new alert. `deadline` must be strictly in the future.
    pub fn create_alert(
        env: Env,
        owner: Address,
        resolver: Address,
        pet_name: String,
        last_seen: String,
        deadline: u64,
    ) -> Result<u64, Error> {
        owner.require_auth();
        let mut cfg = load_config(&env)?;

        if pet_name.len() > MAX_TEXT_LEN || last_seen.len() > MAX_TEXT_LEN {
            return Err(Error::TextTooLong);
        }
        if deadline <= env.ledger().timestamp() {
            return Err(Error::InvalidDeadline);
        }

        let id = cfg.next_id;
        cfg.next_id = id.checked_add(1).ok_or(Error::InvalidState)?;
        env.storage().persistent().set(&config_key(&env), &cfg);

        let alert = Alert {
            owner: owner.clone(),
            resolver,
            pet_name,
            last_seen,
            deadline,
            state: AlertState::Active,
            sightings: Vec::new(&env),
        };
        save_alert(&env, id, &alert);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "alert_created"), id),
            AlertCreated {
                id,
                owner,
                deadline,
            },
        );
        Ok(id)
    }

    /// Record a sighting on an active alert. Bounded by `MAX_SIGHTINGS`.
    pub fn record_sighting(env: Env, reporter: Address, id: u64) -> Result<(), Error> {
        reporter.require_auth();
        let mut alert = load_alert(&env, id)?;
        if alert.state != AlertState::Active {
            return Err(Error::InvalidState);
        }
        if alert.sightings.len() >= MAX_SIGHTINGS {
            return Err(Error::TooManySightings);
        }
        alert.sightings.push_back(reporter.clone());
        save_alert(&env, id, &alert);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "sighting"), id),
            SightingRecorded { id, reporter },
        );
        Ok(())
    }

    /// Resolve an active alert. Only the owner or the designated resolver may
    /// call this. Idempotent: a second call on an already-resolved alert is a
    /// no-op that emits no event.
    pub fn resolve_alert(env: Env, actor: Address, id: u64) -> Result<(), Error> {
        actor.require_auth();
        let mut alert = load_alert(&env, id)?;

        if alert.state == AlertState::Resolved {
            return Ok(());
        }
        if alert.state != AlertState::Active {
            return Err(Error::InvalidState);
        }
        if actor != alert.owner && actor != alert.resolver {
            return Err(Error::Unauthorized);
        }

        alert.state = AlertState::Resolved;
        save_alert(&env, id, &alert);
        Self::emit_closed(&env, id, AlertState::Resolved, actor);
        Ok(())
    }

    /// Withdraw an active alert. Only the owner may call this. Idempotent.
    pub fn withdraw_alert(env: Env, actor: Address, id: u64) -> Result<(), Error> {
        actor.require_auth();
        let mut alert = load_alert(&env, id)?;

        if alert.state == AlertState::Withdrawn {
            return Ok(());
        }
        if alert.state != AlertState::Active {
            return Err(Error::InvalidState);
        }
        if actor != alert.owner {
            return Err(Error::Unauthorized);
        }

        alert.state = AlertState::Withdrawn;
        save_alert(&env, id, &alert);
        Self::emit_closed(&env, id, AlertState::Withdrawn, actor);
        Ok(())
    }

    /// Expire an active alert once its deadline has passed. Callable by anyone.
    /// Idempotent: repeated calls after expiry are no-ops.
    pub fn expire_alert(env: Env, actor: Address, id: u64) -> Result<(), Error> {
        actor.require_auth();
        let mut alert = load_alert(&env, id)?;

        if alert.state == AlertState::Expired {
            return Ok(());
        }
        if alert.state != AlertState::Active {
            return Err(Error::InvalidState);
        }
        if env.ledger().timestamp() < alert.deadline {
            return Err(Error::InvalidDeadline);
        }

        alert.state = AlertState::Expired;
        save_alert(&env, id, &alert);
        Self::emit_closed(&env, id, AlertState::Expired, actor);
        Ok(())
    }

    /// Read an alert. Returns `NotFound` for unknown ids.
    pub fn get_alert(env: Env, id: u64) -> Result<Alert, Error> {
        load_alert(&env, id)
    }

    fn emit_closed(env: &Env, id: u64, state: AlertState, actor: Address) {
        env.events().publish(
            (soroban_sdk::Symbol::new(env, "alert_closed"), id),
            AlertClosed { id, state, actor },
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

    fn setup() -> (Env, LostPetContractClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, LostPetContract);
        let client = LostPetContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        let owner = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin, owner)
    }

    fn create(client: &LostPetContractClient, env: &Env, owner: &Address) -> u64 {
        let resolver = Address::generate(env);
        client.create_alert(
            owner,
            &resolver,
            &String::from_str(env, "Rex"),
            &String::from_str(env, "park"),
            &(env.ledger().timestamp() + 100),
        )
    }

    #[test]
    fn create_and_resolve_by_owner() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        client.resolve_alert(&owner, &id);
        assert_eq!(client.get_alert(&id).state, AlertState::Resolved);
    }

    #[test]
    fn resolve_is_idempotent() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        client.resolve_alert(&owner, &id);
        client.resolve_alert(&owner, &id);
        assert_eq!(client.get_alert(&id).state, AlertState::Resolved);
    }

    #[test]
    fn withdraw_only_by_owner() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        let stranger = Address::generate(&env);
        assert_eq!(
            client.try_withdraw_alert(&stranger, &id),
            Err(Ok(Error::Unauthorized))
        );
        assert_eq!(client.get_alert(&id).state, AlertState::Active);
        client.withdraw_alert(&owner, &id);
        assert_eq!(client.get_alert(&id).state, AlertState::Withdrawn);
    }

    #[test]
    fn expire_requires_deadline_passed() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        let anyone = Address::generate(&env);
        assert_eq!(
            client.try_expire_alert(&anyone, &id),
            Err(Ok(Error::InvalidDeadline))
        );
        env.ledger().set_timestamp(env.ledger().timestamp() + 101);
        client.expire_alert(&anyone, &id);
        assert_eq!(client.get_alert(&id).state, AlertState::Expired);
        client.expire_alert(&anyone, &id);
        assert_eq!(client.get_alert(&id).state, AlertState::Expired);
    }

    #[test]
    fn closed_alert_rejects_sightings() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        client.resolve_alert(&owner, &id);
        let reporter = Address::generate(&env);
        assert_eq!(
            client.try_record_sighting(&reporter, &id),
            Err(Ok(Error::InvalidState))
        );
    }

    #[test]
    fn create_rejects_past_deadline() {
        let (env, client, _admin, owner) = setup();
        let resolver = Address::generate(&env);
        assert_eq!(
            client.try_create_alert(
                &owner,
                &resolver,
                &String::from_str(&env, "Rex"),
                &String::from_str(&env, "park"),
                &env.ledger().timestamp(),
            ),
            Err(Ok(Error::InvalidDeadline))
        );
    }
}
