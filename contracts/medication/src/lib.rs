#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, Symbol};

/// Storage schema version. Bump only on breaking storage layout changes.
const STORAGE_VERSION: u32 = 1;

/// Documented bounds for a medication dose schedule.
///
/// - `MIN_INTERVAL_SECS` / `MAX_INTERVAL_SECS`: allowed dosing interval.
/// - `MAX_SCHEDULE_HORIZON_SECS`: maximum allowed span between start and end.
/// - `MAX_UNIT_LEN`: maximum length of a dose unit symbol.
const MIN_INTERVAL_SECS: u64 = 60; // 1 minute
const MAX_INTERVAL_SECS: u64 = 31 * 24 * 60 * 60; // 31 days
const MAX_SCHEDULE_HORIZON_SECS: u64 = 366 * 24 * 60 * 60; // 1 year
const MAX_UNIT_LEN: u32 = 16;

/// Error codes for the medication contract.
///
/// | Code | Name                  | Meaning                                              |
/// |------|-----------------------|------------------------------------------------------|
/// | 1    | NotAuthorized         | Caller is not the registered owner.                  |
/// | 2    | InvalidUnit           | Dose unit is not allowed for the given species.      |
/// | 3    | InvalidInterval       | Interval is outside documented bounds.               |
/// | 4    | InvalidStartTime      | Start time is in the past or otherwise invalid.      |
/// | 5    | InvalidEndTime        | End time precedes start or exceeds the horizon.      |
/// | 6    | ScheduleNotFound      | No schedule stored for the given id.                 |
/// | 7    | ScheduleAlreadyExists | A schedule with the given id already exists.         |
/// | 8    | ScheduleDiscontinued  | Schedule is already discontinued.                    |
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MedicationError {
    NotAuthorized = 1,
    InvalidUnit = 2,
    InvalidInterval = 3,
    InvalidStartTime = 4,
    InvalidEndTime = 5,
    ScheduleNotFound = 6,
    ScheduleAlreadyExists = 7,
    ScheduleDiscontinued = 8,
}

/// Species supported by the contract. Units are validated per species.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Species {
    Canine = 0,
    Feline = 1,
    Equine = 2,
}

/// A validated medication dose schedule.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoseSchedule {
    pub id: u64,
    pub species: Species,
    pub unit: Symbol,
    pub interval_secs: u64,
    pub start_time: u64,
    pub end_time: u64,
}

/// Discontinuation record. Presence of this key marks a schedule as
/// discontinued and invalidates reminders and refill eligibility.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discontinuation {
    pub id: u64,
    pub discontinued_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct DataKey {
    pub owner: Address,
    pub id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct DiscontinuationKey {
    pub owner: Address,
    pub id: u64,
}

#[contract]
pub struct MedicationContract;

#[contractimpl]
impl MedicationContract {
    /// Initialize the contract with an owner and record the storage version.
    pub fn init(env: Env, owner: Address) {
        owner.require_auth();
        env.storage().instance().set(&Symbol::new(&env, "owner"), &owner);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "version"), &STORAGE_VERSION);
    }

    /// Return the current storage schema version.
    pub fn storage_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, "version"))
            .unwrap_or(0)
    }

    /// Create a validated dose schedule.
    ///
    /// Authorization: only the registered owner may create schedules.
    /// Validation is performed before any state is written, so a failed call
    /// leaves storage unchanged.
    pub fn create_schedule(
        env: Env,
        id: u64,
        species: Species,
        unit: Symbol,
        interval_secs: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<(), MedicationError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "owner"))
            .ok_or(MedicationError::NotAuthorized)?;
        owner.require_auth();

        let key = DataKey {
            owner: owner.clone(),
            id,
        };
        if env.storage().persistent().has(&key) {
            return Err(MedicationError::ScheduleAlreadyExists);
        }

        Self::validate_schedule(&env, species, &unit, interval_secs, start_time, end_time)?;

        let schedule = DoseSchedule {
            id,
            species,
            unit,
            interval_secs,
            start_time,
            end_time,
        };
        env.storage().persistent().set(&key, &schedule);
        env.events().publish(
            (Symbol::new(&env, "schedule_created"), owner),
            (id, species, interval_secs, start_time, end_time),
        );
        Ok(())
    }

    /// Fetch a stored schedule for the owner.
    pub fn get_schedule(env: Env, owner: Address, id: u64) -> Result<DoseSchedule, MedicationError> {
        let key = DataKey { owner, id };
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(MedicationError::ScheduleNotFound)
    }

    /// Discontinue a medication schedule.
    ///
    /// Authorization: only the registered owner may discontinue schedules.
    /// In the same transaction this removes the active schedule (invalidating
    /// reminders) and records a discontinuation marker so refill eligibility
    /// checks fail. All validation happens before any write, so a failed call
    /// leaves storage unchanged.
    pub fn discontinue_schedule(env: Env, id: u64) -> Result<(), MedicationError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "owner"))
            .ok_or(MedicationError::NotAuthorized)?;
        owner.require_auth();

        let key = DataKey {
            owner: owner.clone(),
            id,
        };
        if !env.storage().persistent().has(&key) {
            return Err(MedicationError::ScheduleNotFound);
        }

        let dkey = DiscontinuationKey {
            owner: owner.clone(),
            id,
        };
        if env.storage().persistent().has(&dkey) {
            return Err(MedicationError::ScheduleDiscontinued);
        }

        let now = env.ledger().timestamp();
        let record = Discontinuation {
            id,
            discontinued_at: now,
        };

        // Same-transaction propagation: invalidate reminders by removing the
        // active schedule and mark refill eligibility as ineligible.
        env.storage().persistent().remove(&key);
        env.storage().persistent().set(&dkey, &record);
        env.events().publish(
            (Symbol::new(&env, "schedule_discontinued"), owner),
            (id, now),
        );
        Ok(())
    }

    /// Returns true when the schedule is eligible for refills.
    ///
    /// A schedule is eligible only while it exists and has not been
    /// discontinued.
    pub fn is_refill_eligible(env: Env, owner: Address, id: u64) -> bool {
        let key = DataKey {
            owner: owner.clone(),
            id,
        };
        if !env.storage().persistent().has(&key) {
            return false;
        }
        let dkey = DiscontinuationKey { owner, id };
        !env.storage().persistent().has(&dkey)
    }

    /// Validate a dose schedule against documented bounds.
    ///
    /// Uses checked timestamp arithmetic so that overflow cannot bypass the
    /// horizon check. Units are validated against species-aware allowlists.
    fn validate_schedule(
        env: &Env,
        species: Species,
        unit: &Symbol,
        interval_secs: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<(), MedicationError> {
        // Bound the unit symbol length before comparing.
        if unit.len() == 0 || unit.len() > MAX_UNIT_LEN {
            return Err(MedicationError::InvalidUnit);
        }
        if !Self::is_allowed_unit(env, species, unit) {
            return Err(MedicationError::InvalidUnit);
        }

        if interval_secs < MIN_INTERVAL_SECS || interval_secs > MAX_INTERVAL_SECS {
            return Err(MedicationError::InvalidInterval);
        }

        let now = env.ledger().timestamp();
        if start_time < now {
            return Err(MedicationError::InvalidStartTime);
        }

        if end_time <= start_time {
            return Err(MedicationError::InvalidEndTime);
        }

        // Checked arithmetic: reject overflow rather than wrapping.
        let horizon = start_time
            .checked_add(MAX_SCHEDULE_HORIZON_SECS)
            .ok_or(MedicationError::InvalidEndTime)?;
        if end_time > horizon {
            return Err(MedicationError::InvalidEndTime);
        }

        Ok(())
    }

    /// Species-aware allowlist of dose units.
    fn is_allowed_unit(env: &Env, species: Species, unit: &Symbol) -> bool {
        let mg = Symbol::new(env, "mg");
        let ml = Symbol::new(env, "ml");
        let iu = Symbol::new(env, "iu");
        let mcg = Symbol::new(env, "mcg");
        match species {
            Species::Canine => unit == &mg || unit == &ml || unit == &mcg,
            Species::Feline => unit == &mg || unit == &ml,
            Species::Equine => unit == &mg || unit == &ml || unit == &iu,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Env, Symbol};

    fn setup() -> (Env, MedicationContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, MedicationContract);
        let client = MedicationContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        client.init(&owner);
        (env, client, owner)
    }

    #[test]
    fn test_create_schedule_success() {
        let (env, client, owner) = setup();
        env.ledger().set_timestamp(1_000);
        let unit = Symbol::new(&env, "mg");
        let res = client.try_create_schedule(&1, &Species::Canine, &unit, &3_600, &2_000, &100_000);
        assert!(res.is_ok());
        let stored = client.get_schedule(&owner, &1);
        assert_eq!(stored.interval_secs, 3_600);
    }

    #[test]
    fn test_discontinue_success_invalidates_reminders_and_refills() {
        let (env, client, owner) = setup();
        env.ledger().set_timestamp(1_000);
        let unit = Symbol::new(&env, "mg");
        client.create_schedule(&1, &Species::Canine, &unit, &3_600, &2_000, &100_000);
        assert!(client.is_refill_eligible(&owner, &1));

        let res = client.try_discontinue_schedule(&1);
        assert!(res.is_ok());

        // Reminders invalidated: active schedule no longer retrievable.
        assert!(client.try_get_schedule(&owner, &1).is_err());
        // Refill eligibility invalidated.
        assert!(!client.is_refill_eligible(&owner, &1));
    }

    #[test]
    fn test_discontinue_unauthorized() {
        let env = Env::default();
        let contract_id = env.register_contract(None, MedicationContract);
        let client = MedicationContractClient::new(&env, &contract_id);
        let owner = Address::generate(&env);
        client.init(&owner);
        env.ledger().set_timestamp(1_000);
        let unit = Symbol::new(&env, "mg");
        client.create_schedule(&1, &Species::Canine, &unit, &3_600, &2_000, &100_000);

        // No auth mocked: the call must fail and leave state unchanged.
        let res = client.try_discontinue_schedule(&1);
        assert!(res.is_err());
        assert!(client.is_refill_eligible(&owner, &1));
        assert!(client.try_get_schedule(&owner, &1).is_ok());
    }

    #[test]
    fn test_discontinue_not_found_leaves_state_unchanged() {
        let (env, client, owner) = setup();
        env.ledger().set_timestamp(1_000);
        let res = client.try_discontinue_schedule(&99);
        assert!(res.is_err());
        assert!(!client.is_refill_eligible(&owner, &99));
    }

    #[test]
    fn test_discontinue_replay_rejected() {
        let (env, client, owner) = setup();
        env.ledger().set_timestamp(1_000);
        let unit = Symbol::new(&env, "mg");
        client.create_schedule(&1, &Species::Canine, &unit, &3_600, &2_000, &100_000);
        client.discontinue_schedule(&1);

        // Replay of the same discontinuation must be rejected.
        let res = client.try_discontinue_schedule(&1);
        assert!(res.is_err());
        assert!(!client.is_refill_eligible(&owner, &1));
    }

    #[test]
    fn test_discontinue_boundary_id_zero() {
        let (env, client, owner) = setup();
        env.ledger().set_timestamp(1_000);
        let unit = Symbol::new(&env, "mg");
        client.create_schedule(&0, &Species::Canine, &unit, &3_600, &2_000, &100_000);
        assert!(client.is_refill_eligible(&owner, &0));
        client.discontinue_schedule(&0);
        assert!(!client.is_refill_eligible(&owner, &0));
    }
}
