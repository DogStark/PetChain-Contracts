//! Storage types for the adoption trial period policy (issues #1454, #1657).
//!
//! Trial expiry is lazy: no background task can run on a blockchain, so the
//! first interaction with an expired trial settles it and publishes a typed
//! `TrialPeriodExpired` event. Keeping the types in a separate module avoids
//! changing any existing storage-key discriminants.

use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum TrialPeriodKey {
    /// `adoption_id` -> the trial record for that adoption.
    Trial(u64),
}

/// Outcome applied when a trial period crosses `trial_expires_at`.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrialOutcome {
    /// Trial succeeded: the adoption becomes permanent and the adopter owns
    /// the pet.
    Converted = 0,
    /// Trial failed: the pet goes back to its original owner.
    Reversed = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptionTrial {
    pub pet_id: u64,
    pub adoption_id: u64,
    /// Candidate adopter who would own the pet after a successful trial.
    pub adopter: Address,
    /// Owner captured when the trial started; the beneficiary of a reversal.
    pub previous_owner: Address,
    pub trial_expires_at: u64,
    pub auto_convert: bool,
    pub settled: bool,
    pub outcome: Option<TrialOutcome>,
    pub settled_at: u64,
}

/// Event published exactly once, on the first interaction after expiry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrialPeriodExpiredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub adoption_id: u64,
    pub outcome: TrialOutcome,
    /// Address that owns the pet after settlement.
    pub new_owner: Address,
    /// Original owner, always included so integrators can reverse side effects.
    pub previous_owner: Address,
    pub timestamp: u64,
}
