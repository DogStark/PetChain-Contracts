//! Medication dose schedule validation.
//!
//! Validates dose unit, interval, start time, and end time against documented
//! bounds using checked timestamp arithmetic and species-aware units.

use soroban_sdk::{contracterror, contracttype, Env, String};

/// Documented bounds for dose schedule validation.
///
/// - `MIN_INTERVAL_SECS` / `MAX_INTERVAL_SECS`: allowed dosing interval.
/// - `MAX_SCHEDULE_HORIZON_SECS`: maximum span between start and end time.
/// - `MAX_UNIT_LEN`: maximum length of a dose unit string.
pub const MIN_INTERVAL_SECS: u64 = 60;
pub const MAX_INTERVAL_SECS: u64 = 31_536_000; // 1 year
pub const MAX_SCHEDULE_HORIZON_SECS: u64 = 315_360_000; // 10 years
pub const MAX_UNIT_LEN: u32 = 16;

/// Error codes for dose schedule validation.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DoseValidationError {
    /// Dose unit is empty or exceeds `MAX_UNIT_LEN`.
    InvalidUnit = 1,
    /// Dose unit is not permitted for the given species.
    UnitNotAllowedForSpecies = 2,
    /// Interval is below `MIN_INTERVAL_SECS` or above `MAX_INTERVAL_SECS`.
    IntervalOutOfBounds = 3,
    /// Start time is not strictly before end time.
    InvalidTimeRange = 4,
    /// Schedule horizon exceeds `MAX_SCHEDULE_HORIZON_SECS`.
    HorizonExceeded = 5,
    /// Checked timestamp arithmetic overflowed.
    TimestampOverflow = 6,
}

/// Species categories used to select the allowed dose units.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Species {
    Canine = 0,
    Feline = 1,
    Equine = 2,
    Bovine = 3,
}

/// A validated dose schedule.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoseSchedule {
    pub unit: String,
    pub interval_secs: u64,
    pub start_time: u64,
    pub end_time: u64,
}

/// Returns the dose units permitted for the given species.
fn allowed_units(env: &Env, species: Species) -> soroban_sdk::Vec<String> {
    let mut units = soroban_sdk::Vec::new(env);
    match species {
        Species::Canine | Species::Feline => {
            units.push_back(String::from_str(env, "mg"));
            units.push_back(String::from_str(env, "ml"));
            units.push_back(String::from_str(env, "tablet"));
        }
        Species::Equine | Species::Bovine => {
            units.push_back(String::from_str(env, "mg"));
            units.push_back(String::from_str(env, "ml"));
            units.push_back(String::from_str(env, "g"));
        }
    }
    units
}

/// Validates a dose unit against species-aware allowed units and length bounds.
pub fn validate_unit(
    env: &Env,
    species: Species,
    unit: &String,
) -> Result<(), DoseValidationError> {
    let len = unit.len();
    if len == 0 || len > MAX_UNIT_LEN {
        return Err(DoseValidationError::InvalidUnit);
    }

    let allowed = allowed_units(env, species);
    for candidate in allowed.iter() {
        if candidate == *unit {
            return Ok(());
        }
    }
    Err(DoseValidationError::UnitNotAllowedForSpecies)
}

/// Validates the dosing interval against documented bounds.
pub fn validate_interval(interval_secs: u64) -> Result<(), DoseValidationError> {
    if interval_secs < MIN_INTERVAL_SECS || interval_secs > MAX_INTERVAL_SECS {
        return Err(DoseValidationError::IntervalOutOfBounds);
    }
    Ok(())
}

/// Validates start/end times using checked timestamp arithmetic.
pub fn validate_time_range(start_time: u64, end_time: u64) -> Result<(), DoseValidationError> {
    if start_time >= end_time {
        return Err(DoseValidationError::InvalidTimeRange);
    }
    let horizon = end_time
        .checked_sub(start_time)
        .ok_or(DoseValidationError::TimestampOverflow)?;
    if horizon > MAX_SCHEDULE_HORIZON_SECS {
        return Err(DoseValidationError::HorizonExceeded);
    }
    Ok(())
}

/// Validates a full dose schedule, preserving caller authorization conventions.
///
/// The caller is expected to have already passed the module's authorization
/// checks; this function performs validation only and never mutates state.
pub fn validate_schedule(
    env: &Env,
    species: Species,
    schedule: &DoseSchedule,
) -> Result<(), DoseValidationError> {
    validate_unit(env, species, &schedule.unit)?;
    validate_interval(schedule.interval_secs)?;
    validate_time_range(schedule.start_time, schedule.end_time)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    fn schedule(env: &Env, unit: &str, interval: u64, start: u64, end: u64) -> DoseSchedule {
        DoseSchedule {
            unit: String::from_str(env, unit),
            interval_secs: interval,
            start_time: start,
            end_time: end,
        }
    }

    #[test]
    fn accepts_valid_schedule() {
        let env = Env::default();
        let s = schedule(&env, "mg", 3600, 1_000, 10_000);
        assert_eq!(validate_schedule(&env, Species::Canine, &s), Ok(()));
    }

    #[test]
    fn rejects_unit_not_allowed_for_species() {
        let env = Env::default();
        let s = schedule(&env, "g", 3600, 1_000, 10_000);
        assert_eq!(
            validate_schedule(&env, Species::Feline, &s),
            Err(DoseValidationError::UnitNotAllowedForSpecies)
        );
    }

    #[test]
    fn rejects_empty_and_oversized_unit() {
        let env = Env::default();
        let empty = String::from_str(&env, "");
        assert_eq!(
            validate_unit(&env, Species::Canine, &empty),
            Err(DoseValidationError::InvalidUnit)
        );
        let long = String::from_str(&env, "this-unit-is-way-too-long");
        assert_eq!(
            validate_unit(&env, Species::Canine, &long),
            Err(DoseValidationError::InvalidUnit)
        );
    }

    #[test]
    fn interval_boundaries() {
        assert_eq!(validate_interval(MIN_INTERVAL_SECS), Ok(()));
        assert_eq!(validate_interval(MAX_INTERVAL_SECS), Ok(()));
        assert_eq!(
            validate_interval(MIN_INTERVAL_SECS - 1),
            Err(DoseValidationError::IntervalOutOfBounds)
        );
        assert_eq!(
            validate_interval(MAX_INTERVAL_SECS + 1),
            Err(DoseValidationError::IntervalOutOfBounds)
        );
    }

    #[test]
    fn time_range_boundaries() {
        assert_eq!(validate_time_range(1_000, 1_001), Ok(()));
        assert_eq!(
            validate_time_range(1_000, 1_000),
            Err(DoseValidationError::InvalidTimeRange)
        );
        assert_eq!(
            validate_time_range(1_000, 999),
            Err(DoseValidationError::InvalidTimeRange)
        );
        assert_eq!(
            validate_time_range(0, MAX_SCHEDULE_HORIZON_SECS + 1),
            Err(DoseValidationError::HorizonExceeded)
        );
    }

    #[test]
    fn failed_validation_leaves_input_unchanged() {
        let env = Env::default();
        let s = schedule(&env, "g", 10, 5_000, 1_000);
        let before = s.clone();
        assert!(validate_schedule(&env, Species::Canine, &s).is_err());
        assert_eq!(s, before);
    }
}
