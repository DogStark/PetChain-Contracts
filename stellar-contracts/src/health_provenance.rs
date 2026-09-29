//! Storage keys, value types and pure scoring/encoding functions for
//! health-score input provenance (Issue #1343).
//!
//! A recorded health score is bound to:
//! - `input_digest`: SHA-256 of the canonical encoding of the exact inputs
//!   (component values + the set of source records they were derived from);
//! - `algorithm_version`: which scoring function produced it;
//! - `scorer` and `scored_at`.
//!
//! Records are append-only. `(pet_id, algorithm_version, input_digest)` is
//! unique: recording the same inputs again under the same version returns
//! the existing record, and a different version always creates a new
//! record, so no version can overwrite another's history.
//!
//! Canonical input encoding (all integers big-endian, no separators):
//!
//! ```text
//! sha256(
//!     b"petchain:health-score-input:v1"
//!  || pet_id            u64
//!  || as_of             u64
//!  || vaccination       u32
//!  || lab_results       u32
//!  || activity          u32
//!  || insurance         u32
//!  || source_count      u32
//!  || for each source (strictly ascending by (kind, id)):
//!         kind          u32   (MedicalRecord=0, Vaccination=1, LabResult=2)
//!      || id            u64
//! )
//! ```

use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env, Vec};

pub const HEALTH_SCORE_INPUT_DOMAIN: &[u8] = b"petchain:health-score-input:v1";
pub const MAX_HEALTH_SCORE_SOURCES: u32 = 64;
pub const MAX_HEALTH_COMPONENT_SCORE: u32 = 100;
/// Algorithm versions this contract can evaluate. See `score_for_version`.
pub const HEALTH_SCORE_ALGORITHM_V1: u32 = 1;
pub const HEALTH_SCORE_ALGORITHM_V2: u32 = 2;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HealthScoreKey {
    /// (pet_id, seq) -> HealthScoreRecord. seq starts at 1.
    Record((u64, u64)),
    /// pet_id -> number of records.
    Count(u64),
    /// (pet_id, algorithm_version, input_digest) -> seq.
    ByInput((u64, u32, BytesN<32>)),
    /// (pet_id, algorithm_version) -> seq of the most recent record.
    Latest((u64, u32)),
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum HealthInputKind {
    MedicalRecord = 0,
    Vaccination = 1,
    LabResult = 2,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthInputRef {
    pub kind: HealthInputKind,
    pub id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthScoreInputs {
    /// Snapshot time the component values were evaluated at.
    pub as_of: u64,
    pub vaccination: u32,
    pub lab_results: u32,
    pub activity: u32,
    pub insurance: u32,
    /// Source records, strictly ascending by `(kind, id)`.
    pub sources: Vec<HealthInputRef>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthScoreRecord {
    pub pet_id: u64,
    pub seq: u64,
    pub score: u32,
    pub algorithm_version: u32,
    pub input_digest: BytesN<32>,
    pub inputs: HealthScoreInputs,
    pub scorer: Address,
    pub scored_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthScoreRecordedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub seq: u64,
    pub score: u32,
    pub algorithm_version: u32,
    pub input_digest: BytesN<32>,
    pub scorer: Address,
    pub timestamp: u64,
}

/// Why a set of inputs is structurally invalid.
pub enum HealthInputError {
    ComponentOutOfRange,
    TooManySources,
    NotCanonical,
}

/// Structural validation: component bounds, source count, and strict
/// `(kind, id)` ordering (which also rules out duplicates), so every input
/// set has exactly one valid encoding.
pub fn validate_inputs(inputs: &HealthScoreInputs) -> Result<(), HealthInputError> {
    for c in [
        inputs.vaccination,
        inputs.lab_results,
        inputs.activity,
        inputs.insurance,
    ] {
        if c > MAX_HEALTH_COMPONENT_SCORE {
            return Err(HealthInputError::ComponentOutOfRange);
        }
    }
    if inputs.sources.len() > MAX_HEALTH_SCORE_SOURCES {
        return Err(HealthInputError::TooManySources);
    }
    let mut prev: Option<(u32, u64)> = None;
    for s in inputs.sources.iter() {
        let cur = (s.kind as u32, s.id);
        if let Some(p) = prev {
            if cur <= p {
                return Err(HealthInputError::NotCanonical);
            }
        }
        prev = Some(cur);
    }
    Ok(())
}

/// SHA-256 over the canonical encoding documented at the top of this file.
/// Callers must run `validate_inputs` first.
pub fn input_digest(env: &Env, pet_id: u64, inputs: &HealthScoreInputs) -> BytesN<32> {
    let mut buf = Bytes::from_slice(env, HEALTH_SCORE_INPUT_DOMAIN);
    buf.extend_from_array(&pet_id.to_be_bytes());
    buf.extend_from_array(&inputs.as_of.to_be_bytes());
    buf.extend_from_array(&inputs.vaccination.to_be_bytes());
    buf.extend_from_array(&inputs.lab_results.to_be_bytes());
    buf.extend_from_array(&inputs.activity.to_be_bytes());
    buf.extend_from_array(&inputs.insurance.to_be_bytes());
    buf.extend_from_array(&inputs.sources.len().to_be_bytes());
    for s in inputs.sources.iter() {
        buf.extend_from_array(&(s.kind as u32).to_be_bytes());
        buf.extend_from_array(&s.id.to_be_bytes());
    }
    env.crypto().sha256(&buf).into()
}

/// Pure scoring function. Returns `None` for an unsupported version.
///
/// - v1: unweighted mean of the four components (floor).
/// - v2: weighted mean — vaccination 35%, lab results 25%, activity 15%,
///   insurance 25% (floor).
pub fn score_for_version(algorithm_version: u32, inputs: &HealthScoreInputs) -> Option<u32> {
    match algorithm_version {
        HEALTH_SCORE_ALGORITHM_V1 => {
            Some((inputs.vaccination + inputs.lab_results + inputs.activity + inputs.insurance) / 4)
        }
        HEALTH_SCORE_ALGORITHM_V2 => Some(
            (inputs.vaccination * 35
                + inputs.lab_results * 25
                + inputs.activity * 15
                + inputs.insurance * 25)
                / 100,
        ),
        _ => None,
    }
}
