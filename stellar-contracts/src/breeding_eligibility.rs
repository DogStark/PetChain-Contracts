//! Storage keys and value types for centralized breeding eligibility and
//! cooldown invariants (Issue #1342).
//!
//! `record_mating` (in `lib.rs`) is the strict entry point: it evaluates
//! every rule below through one function (`evaluate_breeding_eligibility`),
//! records the parent/cooldown relationship, and treats a replay of the
//! same `(sire_id, dam_id, breeding_date)` mating as idempotent.
//!
//! Rules, in evaluation order:
//! 1. `sire_id != dam_id`.
//! 2. Both pets exist, are active and are not archived.
//! 3. Same species; sire is `Male`, dam is `Female`.
//! 4. `breeding_date` is not in the future and not before either birth.
//! 5. Each parent is at least `min_age_secs` old at `breeding_date`
//!    (exact boundary is eligible).
//! 6. `breeding_date >= last_mating + cooldown` for each parent, using the
//!    sire/dam cooldown respectively (exact boundary is eligible). This also
//!    makes mating dates monotonic per pet: back-dating is rejected.
//! 7. Coefficient of inbreeding is below `max_coi_bp`.

use soroban_sdk::{contracttype, Address};

pub const SECONDS_PER_DAY: u64 = 86_400;
pub const DEFAULT_MIN_BREEDING_AGE_SECS: u64 = 365 * SECONDS_PER_DAY;
pub const DEFAULT_SIRE_COOLDOWN_SECS: u64 = 7 * SECONDS_PER_DAY;
pub const DEFAULT_DAM_COOLDOWN_SECS: u64 = 180 * SECONDS_PER_DAY;
/// 25% — parent/offspring and full-sibling pairings are rejected.
pub const DEFAULT_MAX_COI_BP: u32 = 2_500;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BreedingEligibilityKey {
    /// Admin-configured `BreedingPolicy`; defaults apply when absent.
    Policy,
    /// pet_id -> breeding_date of that pet's most recent recorded mating.
    LastMating(u64),
    /// (sire_id, dam_id, breeding_date) -> breeding record id (replay guard).
    Mating((u64, u64, u64)),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingPolicy {
    pub min_age_secs: u64,
    pub sire_cooldown_secs: u64,
    pub dam_cooldown_secs: u64,
    /// Basis points (0-10000); a pairing with COI >= this is rejected.
    pub max_coi_bp: u32,
}

impl BreedingPolicy {
    pub fn default_policy() -> Self {
        BreedingPolicy {
            min_age_secs: DEFAULT_MIN_BREEDING_AGE_SECS,
            sire_cooldown_secs: DEFAULT_SIRE_COOLDOWN_SECS,
            dam_cooldown_secs: DEFAULT_DAM_COOLDOWN_SECS,
            max_coi_bp: DEFAULT_MAX_COI_BP,
        }
    }
}

/// Outcome of `check_breeding_eligibility`. Authorization is not part of
/// this view (it cannot be evaluated read-only); `record_mating` checks it
/// before evaluating these rules.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BreedingCheck {
    Eligible,
    SelfBreeding,
    ParentNotFound,
    ParentInactive,
    IncompatibleSpecies,
    IncompatibleSex,
    InvalidBreedingDate,
    ParentTooYoung,
    SireCooldownActive,
    DamCooldownActive,
    InbreedingThresholdExceeded,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MatingRecordedEvent {
    pub version: u32,
    pub record_id: u64,
    pub sire_id: u64,
    pub dam_id: u64,
    pub breeding_date: u64,
    pub breeder: Address,
    pub timestamp: u64,
}
