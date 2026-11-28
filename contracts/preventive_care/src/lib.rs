//! Preventive-care schedule contract.
//!
//! Manages recurring preventive-care schedules with bounded inputs.
//! `interval_days` is bounded by [`MAX_INTERVAL_DAYS`] to prevent schedules
//! from being pushed so far into the future that reminder logic is silently
//! bypassed while the schedule remains `Active`.

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env, String};

/// Maximum allowed value for `interval_days` (10 years).
///
/// Any `interval_days` above this bound is rejected with
/// [`Error::InvalidInput`] on both schedule creation and update.
pub const MAX_INTERVAL_DAYS: u64 = 3650;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidInput = 1,
    NotFound = 2,
    Unauthorized = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleStatus {
    Active,
    Paused,
    Completed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schedule {
    pub owner: Address,
    pub care_type: String,
    pub interval_days: u64,
    pub next_due_at: u64,
    pub notes: String,
    pub status: ScheduleStatus,
}

#[contract]
pub struct PreventiveCareContract;

fn validate_interval_days(interval_days: u64) -> Result<(), Error> {
    if interval_days == 0 || interval_days > MAX_INTERVAL_DAYS {
        return Err(Error::InvalidInput);
    }
    Ok(())
}

#[contractimpl]
impl PreventiveCareContract {
    /// Create a new preventive-care schedule.
    ///
    /// Rejects `interval_days` of `0` or greater than [`MAX_INTERVAL_DAYS`]
    /// with [`Error::InvalidInput`].
    pub fn create_schedule(
        env: Env,
        owner: Address,
        care_type: String,
        interval_days: u64,
        next_due_at: u64,
        notes: String,
    ) -> Result<u64, Error> {
        owner.require_auth();
        validate_interval_days(interval_days)?;

        let id: u64 = env.storage().instance().get(&"next_id").unwrap_or(0);
        let schedule = Schedule {
            owner,
            care_type,
            interval_days,
            next_due_at,
            notes,
            status: ScheduleStatus::Active,
        };
        env.storage().persistent().set(&id, &schedule);
        env.storage().instance().set(&"next_id", &(id + 1));
        Ok(id)
    }

    /// Update the `interval_days` of an existing schedule.
    ///
    /// Rejects `interval_days` of `0` or greater than [`MAX_INTERVAL_DAYS`]
    /// with [`Error::InvalidInput`].
    pub fn update_interval_days(
        env: Env,
        id: u64,
        interval_days: u64,
    ) -> Result<(), Error> {
        let mut schedule: Schedule = env
            .storage()
            .persistent()
            .get(&id)
            .ok_or(Error::NotFound)?;
        schedule.owner.require_auth();
        validate_interval_days(interval_days)?;

        schedule.interval_days = interval_days;
        env.storage().persistent().set(&id, &schedule);
        Ok(())
    }

    pub fn get_schedule(env: Env, id: u64) -> Result<Schedule, Error> {
        env.storage()
            .persistent()
            .get(&id)
            .ok_or(Error::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    fn setup() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        (env, owner)
    }

    #[test]
    fn create_rejects_interval_days_above_max() {
        let (env, owner) = setup();
        let client = PreventiveCareContractClient::new(&env, &env.register(PreventiveCareContract, ()));
        let result = client.try_create_schedule(
            &owner,
            &String::from_str(&env, "vaccination"),
            &(MAX_INTERVAL_DAYS + 1),
            &0,
            &String::from_str(&env, ""),
        );
        assert_eq!(result, Err(Ok(Error::InvalidInput)));
    }

    #[test]
    fn create_accepts_interval_days_at_max() {
        let (env, owner) = setup();
        let client = PreventiveCareContractClient::new(&env, &env.register(PreventiveCareContract, ()));
        let id = client.create_schedule(
            &owner,
            &String::from_str(&env, "vaccination"),
            &MAX_INTERVAL_DAYS,
            &0,
            &String::from_str(&env, ""),
        );
        assert_eq!(client.get_schedule(&id).interval_days, MAX_INTERVAL_DAYS);
    }

    #[test]
    fn update_rejects_interval_days_above_max() {
        let (env, owner) = setup();
        let client = PreventiveCareContractClient::new(&env, &env.register(PreventiveCareContract, ()));
        let id = client.create_schedule(
            &owner,
            &String::from_str(&env, "vaccination"),
            &30,
            &0,
            &String::from_str(&env, ""),
        );
        let result = client.try_update_interval_days(&id, &(MAX_INTERVAL_DAYS + 1));
        assert_eq!(result, Err(Ok(Error::InvalidInput)));
    }
}
