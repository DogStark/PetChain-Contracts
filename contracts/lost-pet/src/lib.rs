#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String, Vec};

/// Storage schema version. Bump when the persistent layout changes.
const STORAGE_VERSION: u32 = 2;

/// Maximum length (in bytes) allowed for free-form text fields.
const MAX_TEXT_LEN: u32 = 256;

/// Maximum number of sightings retained per alert.
const MAX_SIGHTINGS: u32 = 32;

/// Maximum number of alerts returned by a single paginated listing call.
const MAX_PAGE_SIZE: u32 = 32;

/// Maximum length (in bytes) allowed for a region identifier.
const MAX_REGION_LEN: u32 = 64;

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
    // Appended in storage version 3. Never renumber existing variants.
    InvalidPageSize = 10,
    InvalidCursor = 11,
    DuplicateAlert = 12,
    RegionTooLong = 13,
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
    pub region: String,
    pub deadline: u64,
    pub state: AlertState,
    pub sightings: Vec<Address>,
    /// Id of the alert this one supersedes, if any (duplicate linking).
    pub linked_to: Option<u64>,
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
pub struct AlertLinked {
    pub id: u64,
    pub supersedes: u64,
    pub owner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageCursor {
    /// Last alert id returned in the previous page.
    pub last_id: u64,
    /// Index generation marker the cursor was issued against.
    pub generation: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub version: u32,
    pub admin: Address,
    pub next_id: u64,
    /// Bumped whenever the active-alert index changes shape, invalidating
    /// outstanding pagination cursors.
    pub index_generation: u32,
}

fn config_key(env: &Env) -> soroban_sdk::Symbol {
    soroban_sdk::Symbol::new(env, "config")
}

fn alert_key(env: &Env, id: u64) -> (soroban_sdk::Symbol, u64) {
    (soroban_sdk::Symbol::new(env, "alert"), id)
}

/// Composite index key: (region, state, alert id).
fn index_key(
    env: &Env,
    region: &String,
    state: AlertState,
    id: u64,
) -> (soroban_sdk::Symbol, String, u32, u64) {
    (
        soroban_sdk::Symbol::new(env, "idx"),
        region.clone(),
        state as u32,
        id,
    )
}

/// Active-alert-by-pet index key: (owner, pet name).
fn pet_key(env: &Env, owner: &Address, pet_name: &String) -> (soroban_sdk::Symbol, Address, String) {
    (
        soroban_sdk::Symbol::new(env, "pet"),
        owner.clone(),
        pet_name.clone(),
    )
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

fn add_index(env: &Env, id: u64, alert: &Alert) {
    env.storage()
        .persistent()
        .set(&index_key(env, &alert.region, alert.state, id), &id);
}

fn remove_index(env: &Env, id: u64, alert: &Alert) {
    env.storage()
        .persistent()
        .remove(&index_key(env, &alert.region, alert.state, id));
}

/// Look up the currently active alert id for (owner, pet name), if any.
fn active_alert_for_pet(env: &Env, owner: &Address, pet_name: &String) -> Option<u64> {
    env.storage()
        .persistent()
        .get(&pet_key(env, owner, pet_name))
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
    ///
    /// Duplicate detection: if an alert for the same `(owner, pet_name)` is
    /// still `Active`, this call fails with `DuplicateAlert`. The owner must
    /// first resolve/withdraw/expire the earlier alert, or use
    /// `create_linked_alert` to explicitly supersede it.
    pub fn create_alert(
        env: Env,
        owner: Address,
        resolver: Address,
        pet_name: String,
        last_seen: String,
        region: String,
        deadline: u64,
    ) -> Result<u64, Error> {
        Self::create_internal(
            env, owner, resolver, pet_name, last_seen, region, deadline, None,
        )
    }

    /// Create a new alert that explicitly supersedes an earlier active alert
    /// for the same pet. The earlier alert is resolved and the new alert is
    /// linked to it. Only the owner of the earlier alert may do this.
    pub fn create_linked_alert(
        env: Env,
        owner: Address,
        resolver: Address,
        pet_name: String,
        last_seen: String,
        region: String,
        deadline: u64,
        supersedes: u64,
    ) -> Result<u64, Error> {
        owner.require_auth();
        let prior = load_alert(&env, supersedes)?;
        if prior.owner != owner {
            return Err(Error::Unauthorized);
        }
        if prior.state != AlertState::Active {
            return Err(Error::InvalidState);
        }
        if prior.pet_name != pet_name {
            return Err(Error::InvalidState);
        }
        Self::create_internal(
            env,
            owner,
            resolver,
            pet_name,
            last_seen,
            region,
            deadline,
            Some(supersedes),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn create_internal(
        env: Env,
        owner: Address,
        resolver: Address,
        pet_name: String,
        last_seen: String,
        region: String,
        deadline: u64,
        supersedes: Option<u64>,
    ) -> Result<u64, Error> {
        owner.require_auth();
        let mut cfg = load_config(&env)?;

        if pet_name.len() > MAX_TEXT_LEN
            || last_seen.len() > MAX_TEXT_LEN
            || region.len() > MAX_REGION_LEN
        {
            return Err(Error::TextTooLong);
        }
        if deadline <= env.ledger().timestamp() {
            return Err(Error::InvalidDeadline);
        }

        // Duplicate detection: reject unless explicitly superseding.
        if let Some(existing) = active_alert_for_pet(&env, &owner, &pet_name) {
            match supersedes {
                Some(prior) if prior == existing => {}
                _ => return Err(Error::DuplicateAlert),
            }
        } else if supersedes.is_some() {
            // Nothing active to supersede.
            return Err(Error::InvalidState);
        }

        // If superseding, close the earlier alert first.
        if let Some(prior_id) = supersedes {
            let mut prior = load_alert(&env, prior_id)?;
            remove_index(&env, prior_id, &prior);
            prior.state = AlertState::Resolved;
            save_alert(&env, prior_id, &prior);
            add_index(&env, prior_id, &prior);
            Self::emit_closed(&env, prior_id, AlertState::Resolved, owner.clone());
        }

        let id = cfg.next_id;
        cfg.next_id = id.checked_add(1).ok_or(Error::InvalidState)?;
        cfg.index_generation = cfg.index_generation.wrapping_add(1);
        env.storage().persistent().set(&config_key(&env), &cfg);

        let alert = Alert {
            owner: owner.clone(),
            resolver,
            pet_name: pet_name.clone(),
            last_seen,
            region,
            deadline,
            state: AlertState::Active,
            sightings: Vec::new(&env),
            linked_to: supersedes,
        };
        save_alert(&env, id, &alert);
        add_index(&env, id, &alert);
        env.storage()
            .persistent()
            .set(&pet_key(&env, &owner, &pet_name), &id);

        env.events().publish(
            (soroban_sdk::Symbol::new(&env, "alert_created"), id),
            AlertCreated {
                id,
                owner: owner.clone(),
                deadline,
            },
        );
        if let Some(prior_id) = supersedes {
            env.events().publish(
                (soroban_sdk::Symbol::new(&env, "alert_linked"), id),
                AlertLinked {
                    id,
                    supersedes: prior_id,
                    owner,
                },
            );
        }
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

        remove_index(&env, id, &alert);
        alert.state = AlertState::Resolved;
        save_alert(&env, id, &alert);
        add_index(&env, id, &alert);
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

        remove_index(&env, id, &alert);
        alert.state = AlertState::Withdrawn;
        save_alert(&env, id, &alert);
        add_index(&env, id, &alert);
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

        remove_index(&env, id, &alert);
        alert.state = AlertState::Expired;
        save_alert(&env, id, &alert);
        add_index(&env, id, &alert);
        Self::emit_closed(&env, id, AlertState::Expired, actor);
        Ok(())
    }

    /// Read an alert. Returns `NotFound` for unknown ids.
    pub fn get_alert(env: Env, id: u64) -> Result<Alert, Error> {
        load_alert(&env, id)
    }

    /// List alerts for a `(region, status)` pair using a stable cursor.
    ///
    /// - `limit` must be in `1..=MAX_PAGE_SIZE`, else `InvalidPageSize`.
    /// - `cursor` is `None` for the first page. A cursor whose generation does
    ///   not match the current index generation is rejected with
    ///   `InvalidCursor` (replay/expiry protection).
    /// - Results are ordered by ascending alert id (deterministic).
    ///
    /// Returns `(ids, next_cursor)`. `next_cursor` is `None` when the page is
    /// the last one.
    pub fn list_active_alerts(
        env: Env,
        region: String,
        status: AlertState,
        cursor: Option<PageCursor>,
        limit: u32,
    ) -> Result<(Vec<u64>, Option<PageCursor>), Error> {
        if limit == 0 || limit > MAX_PAGE_SIZE {
            return Err(Error::InvalidPageSize);
        }
        if region.len() > MAX_REGION_LEN {
            return Err(Error::RegionTooLong);
        }
        let cfg = load_config(&env)?;

        let start_after: u64 = match cursor {
            None => 0,
            Some(c) => {
                if c.generation != cfg.index_generation {
                    return Err(Error::InvalidCursor);
                }
                c.last_id
            }
        };

        let mut ids: Vec<u64> = Vec::new(&env);
        let mut next_id = start_after.checked_add(1).unwrap_or(u64::MAX);
        let mut last_returned: u64 = start_after;
        let mut has_more = false;

        // Bounded scan: walk ids forward from the cursor until we fill the page
        // or exhaust the id space. Each step is O(1) storage lookup.
        while next_id < cfg.next_id {
            if ids.len() >= limit {
                has_more = true;
                break;
            }
            let key = index_key(&env, &region, status, next_id);
            if env.storage().persistent().has(&key) {
                ids.push_back(next_id);
                last_returned = next_id;
            }
            next_id += 1;
        }

        let next_cursor = if has_more {
            Some(PageCursor {
                last_id: last_returned,
                generation: cfg.index_generation,
            })
        } else {
            None
        };
        Ok((ids, next_cursor))
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
        create_named(client, env, owner, "Rex", "north")
    }

    fn create_named(
        client: &LostPetContractClient,
        env: &Env,
        owner: &Address,
        pet: &str,
        region: &str,
    ) -> u64 {
        let resolver = Address::generate(env);
        client.create_alert(
            owner,
            &resolver,
            &String::from_str(env, pet),
            &String::from_str(env, "park"),
            &String::from_str(env, region),
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
                &String::from_str(&env, "north"),
                &env.ledger().timestamp(),
            ),
            Err(Ok(Error::InvalidDeadline))
        );
    }

    #[test]
    fn create_rejects_oversized_region() {
        let (env, client, _admin, owner) = setup();
        let resolver = Address::generate(&env);
        let long = String::from_str(&env, &"r".repeat(65));
        assert_eq!(
            client.try_create_alert(
                &owner,
                &resolver,
                &String::from_str(&env, "Rex"),
                &String::from_str(&env, "park"),
                &long,
                &(env.ledger().timestamp() + 100),
            ),
            Err(Ok(Error::TextTooLong))
        );
    }

    #[test]
    fn duplicate_active_alert_rejected_and_state_unchanged() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        let resolver = Address::generate(&env);
        assert_eq!(
            client.try_create_alert(
                &owner,
                &resolver,
                &String::from_str(&env, "Rex"),
                &String::from_str(&env, "elsewhere"),
                &String::from_str(&env, "north"),
                &(env.ledger().timestamp() + 100),
            ),
            Err(Ok(Error::DuplicateAlert))
        );
        // Original alert untouched.
        assert_eq!(client.get_alert(&id).state, AlertState::Active);
        // A different pet is allowed.
        let other = create_named(&client, &env, &owner, "Fido", "north");
        assert_eq!(client.get_alert(&other).state, AlertState::Active);
    }

    #[test]
    fn duplicate_allowed_after_resolution() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        client.resolve_alert(&owner, &id);
        let id2 = create(&client, &env, &owner);
        assert_ne!(id, id2);
    }

    #[test]
    fn linked_alert_supersedes_prior() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        let resolver = Address::generate(&env);
        let id2 = client.create_linked_alert(
            &owner,
            &resolver,
            &String::from_str(&env, "Rex"),
            &String::from_str(&env, "river"),
            &String::from_str(&env, "north"),
            &(env.ledger().timestamp() + 100),
            &id,
        );
        assert_eq!(client.get_alert(&id).state, AlertState::Resolved);
        let linked = client.get_alert(&id2);
        assert_eq!(linked.state, AlertState::Active);
        assert_eq!(linked.linked_to, Some(id));
    }

    #[test]
    fn linked_alert_requires_owner() {
        let (env, client, _admin, owner) = setup();
        let id = create(&client, &env, &owner);
        let stranger = Address::generate(&env);
        let resolver = Address::generate(&env);
        assert_eq!(
            client.try_create_linked_alert(
                &stranger,
                &resolver,
                &String::from_str(&env, "Rex"),
                &String::from_str(&env, "river"),
                &String::from_str(&env, "north"),
                &(env.ledger().timestamp() + 100),
                &id,
            ),
            Err(Ok(Error::Unauthorized))
        );
        assert_eq!(client.get_alert(&id).state, AlertState::Active);
    }

    #[test]
    fn list_active_alerts_paginates() {
        let (env, client, _admin, owner) = setup();
        let mut ids = Vec::new(&env);
        for i in 0..5u32 {
            let pet = match i {
                0 => "a",
                1 => "b",
                2 => "c",
                3 => "d",
                _ => "e",
            };
            ids.push_back(create_named(&client, &env, &owner, pet, "north"));
        }
        let region = String::from_str(&env, "north");
        let (page1, cursor) =
            client.list_active_alerts(&region, &AlertState::Active, &None, &2);
        assert_eq!(page1.len(), 2);
        assert_eq!(page1.get(0).unwrap(), ids.get(0).unwrap());
        let cursor = cursor.expect("expected next cursor");

        let (page2, cursor2) =
            client.list_active_alerts(&region, &AlertState::Active, &Some(cursor), &2);
        assert_eq!(page2.len(), 2);
        assert_eq!(page2.get(0).unwrap(), ids.get(2).unwrap());
        let cursor2 = cursor2.expect("expected next cursor");

        let (page3, cursor3) =
            client.list_active_alerts(&region, &AlertState::Active, &Some(cursor2), &2);
        assert_eq!(page3.len(), 1);
        assert!(cursor3.is_none());
    }

    #[test]
    fn list_active_alerts_rejects_bad_page_size() {
        let (env, client, _admin, _owner) = setup();
        let region = String::from_str(&env, "north");
        assert_eq!(
            client.try_list_active_alerts(&region, &AlertState::Active, &None, &0),
            Err(Ok(Error::InvalidPageSize))
        );
        assert_eq!(
            client.try_list_active_alerts(&region, &AlertState::Active, &None, &(MAX_PAGE_SIZE + 1)),
            Err(Ok(Error::InvalidPageSize))
        );
    }

    #[test]
    fn list_active_alerts_rejects_stale_cursor() {
        let (env, client, _admin, owner) = setup();
        create_named(&client, &env, &owner, "a", "north");
        create_named(&client, &env, &owner, "b", "north");
        let region = String::from_str(&env, "north");
        let (_page, cursor) =
            client.list_active_alerts(&region, &AlertState::Active, &None, &1);
        let cursor = cursor.expect("expected cursor");

        // Mutating the index bumps the generation, invalidating the cursor.
        create_named(&client, &env, &owner, "c", "north");
        assert_eq!(
            client.try_list_active_alerts(&region, &AlertState::Active, &Some(cursor), &1),
            Err(Ok(Error::InvalidCursor))
        );
    }

    #[test]
    fn list_active_alerts_filters_by_status() {
        let (env, client, _admin, owner) = setup();
        let id = create_named(&client, &env, &owner, "a", "north");
        create_named(&client, &env, &owner, "b", "north");
        client.resolve_alert(&owner, &id);
        let region = String::from_str(&env, "north");
        let (active, _) =
            client.list_active_alerts(&region, &AlertState::Active, &None, &MAX_PAGE_SIZE);
        assert_eq!(active.len(), 1);
        let (resolved, _) =
            client.list_active_alerts(&region, &AlertState::Resolved, &None, &MAX_PAGE_SIZE);
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved.get(0).unwrap(), id);
    }
}
