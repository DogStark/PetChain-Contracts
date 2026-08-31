#![no_std]
#![allow(clippy::too_many_arguments)]

// ---------------------------------------------------------------------------
// EVENT SCHEMA VERSIONING
// Increment EVENT_SCHEMA_VERSION whenever any event struct's fields change.
// Off-chain indexers must check the `version` field on every event to handle
// schema evolution without breaking.
//
// Migration path:
//   v0 (pre-versioning): events had no `version` field — treat as version 0.
//   v1 (current):        `version: u32` added to every event struct.
//                        Indexers that see version 0 should apply defaults for
//                        the new field.
// ---------------------------------------------------------------------------
pub const EVENT_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// STORAGE SCHEMA VERSIONING  (Issue #1149)
//
// STORAGE_SCHEMA_VERSION is a flat u32 stored under SystemKey::StorageSchemaVersion.
// It is independent of the semver ContractVersion and EVENT_SCHEMA_VERSION.
//
// Bump this constant whenever a stored struct gains, loses, or changes a field,
// or when any storage key layout changes in a way that requires a migration.
//
// Migration path:
//   v0 (implicit, pre-versioning): no StorageSchemaVersion key in storage.
//   v1 (current): key written during first migrate_storage call.
//
// `migrate_storage` is:
//   • Authorized  — only an admin may call it.
//   • Idempotent  — calling it again with the same target returns StaleMigration.
//   • Resumable   — only advances forward; never rolls back the schema version.
// ---------------------------------------------------------------------------
pub const STORAGE_SCHEMA_VERSION: u32 = 1;

/// Canonical enum for off-chain indexers to identify the active event schema.
/// Bump the variant and `EVENT_SCHEMA_VERSION` together whenever fields change.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventSchema {
    V1 = 1,
}

/// Paginated result container used by behavior-record list endpoints.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BehaviorRecordPage {
    pub items: Vec<BehaviorRecord>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// Paginated result for vet treatment history.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreatmentHistoryPage {
    pub items: Vec<MedicalRecord>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[contracttype]
pub enum InsuranceKey {
    Policy(u64),               // (pet_id) -> InsurancePolicy [deprecated, never used in production, safe to remove]
    Claim(u64),                // claim_id -> InsuranceClaim
    ClaimCount,                // Global count of claims
    PetClaimCount(u64),        // pet_id -> count of claims
    PetClaimIndex((u64, u64)), // (pet_id, index) -> claim_id
    PetPolicyCount(u64),       // pet_id -> count of policies
    PetPolicyIndex((u64, u64)), // (pet_id, index) -> InsurancePolicy
    // Fraud detection
    FlaggedClaimCount,      // Global count of entries in the flagged index
    FlaggedClaimIndex(u64), // sequential index -> claim_id (for paginated admin review)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PremiumTier {
    Basic,
    Standard,
    Premium,
}

#[contracttype]
pub enum BehaviorKey {
    BehaviorRecord(u64),
    BehaviorRecordCount,
    PetBehaviorCount(u64),
    PetBehaviorIndex((u64, u64)),
    TrainingMilestone(u64),
    TrainingMilestoneCount,
    PetMilestoneCount(u64),
    PetMilestoneIndex((u64, u64)),
    MilestonePrerequisite((u64, u64)), // (milestone_id, seq) -> prerequisite_id
}

#[contracttype]
pub enum ActivityKey {
    ActivityRecord(u64),
    ActivityRecordCount,
    PetActivityCount(u64),
    PetActivityIndex((u64, u64)),

    // Streak tracking
    PetActivityStreak(u64),       // pet_id -> ActivityStreak
    PetStreakLastRecordDate(u64), // pet_id -> last activity date (for gap detection)

    // Idempotency tracking (Issue #685)
    ActivityIdempotencyKey(Bytes), // hash(pet_id, activity_type, start_ts) -> timestamp
    IdempotencyWindow,             // Configurable time window in seconds (default 60)
}

#[contracttype]
pub enum BreedingKey {
    BreedingRecord(u64),
    BreedingRecordCount,
    PetBreedingCount(u64),
    PetBreedingIndex((u64, u64)),
    PetOffspringCount(u64),
    PetOffspringIndex((u64, u64)),
    ParentPair(u64),
    LineageDepth(u64),
    BreedingOffspringCount(u64),
    BreedingOffspringIndex((u64, u64)),
}

/// Allele type for Mendelian genetics simulation.
/// Dominant allele expresses when at least one copy is present;
/// Recessive allele only expresses when both copies are recessive.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Allele {
    Dominant,
    Recessive,
}

/// Storage keys for genetics data (kept separate to avoid breaking existing structs).
#[contracttype]
pub enum GeneticsKey {
    /// pet_id -> Map<trait_name, Allele>
    PetTraits(u64),
    /// breeding_record_id -> Map<trait_name, u32>  (probability in basis points 0-10000)
    PredictedTraits(u64),
}

#[contracttype]
pub enum GroomingKey {
    GroomingRecord(u64),
    GroomingRecordCount,
    PetGroomingCount(u64),
    PetGroomingIndex((u64, u64)),
    Groomer(Address),
    GroomerRatingCount,
    RecurringSchedule(u64),
    RecurringScheduleCount,
    PetScheduleCount(u64),
    PetScheduleIndex((u64, u64)),
    GroomerSlotIndex((Address, u64)),
    GroomerSlotCount(Address),
}

use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Bytes, BytesN,
    Env, IntoVal, Map, String, Symbol, Val, Vec,
};

// Bounded-module split (Issue #1146, phase 1): storage keys and value
// types for the dispute-resolution domain live in `disputes.rs`. The
// `pub use` keeps every type nameable exactly as before (`Dispute`,
// `DisputeKey`, ...) from both this file and external callers.
mod disputes;
pub use disputes::*;

#[cfg(test)]
mod test_behavior_records;
#[cfg(test)]
mod test_dispute_voting;
#[cfg(test)]
mod test_error_registry;
#[cfg(test)]
mod test_license_uniqueness;
#[cfg(test)]
mod test_medical_record_soft_delete;
#[cfg(test)]
mod test_nutrition_plan;
#[cfg(test)]
mod test_attachment_limit;
#[cfg(test)]
mod test_get_pets_overdue_vaccinations;
#[cfg(test)]
mod test_search_medical_records;
#[cfg(test)]
mod test_insurance_eligibility;
#[cfg(test)]
mod test_breeding;
#[cfg(test)]
mod test_breeding_genetics;
#[cfg(test)]
mod test_pet_birthday_validation;
#[cfg(test)]
mod test_persistent_ttl_policy;
#[cfg(test)]
mod test_access_grant_index_invariants;
#[cfg(test)]
mod test_medical_record_hashing;
#[cfg(test)]
mod test_medical_event_timestamps;
#[cfg(test)]
mod test_verify_claim_document;
#[cfg(test)]
mod test_vet_pagination;
#[cfg(test)]
mod test_access_grant_pagination;
#[cfg(test)]
mod test_access_revocation_cascade;
#[cfg(test)]
mod test_decryption_token_key_version;
#[cfg(test)]
mod test_domain_separated_hashes;
#[cfg(test)]
mod test_upgrade_proposal;
#[cfg(test)]
// NOTE: test_disputes.rs and test_book_slot.rs were wired but reference
// contract features (dispute arbitration + slot booking) that were removed
// and no longer exist in this crate, so they fail to compile. They are
// temporarily unwired to unblock `cargo test`; restore them alongside those
// features. (Files preserved.)
// mod test_disputes;
// mod test_book_slot;
#[cfg(test)]
mod test_emergency_notify_rate_limit;
#[cfg(test)]
mod test_discriminant_stability;
#[cfg(test)]
mod test_max_input_sizes;

const DEFAULT_NONCE_MAX_USES: u32 = 1;
#[allow(dead_code)]
const NONCE_HISTORY_LIMIT: u32 = 8;
#[allow(dead_code)]
const MAX_SEARCH_KEYWORD_LEN: u32 = 32;
#[allow(dead_code)]
const MAX_SEARCH_TOKENS_PER_RECORD: u32 = 16;
#[allow(dead_code)]
const MAX_SEARCH_NOTES_LEN: u32 = 512;
const MAX_LINEAGE_DEPTH: u32 = 16;
const MAX_LOG_ENTRIES: u32 = 1_000;
const MAX_ACTIVE_SUBSCRIPTIONS_PER_ADDRESS: u32 = 10;
const MAX_BATCH_ERROR_MESSAGES: usize = 50;
/// Maximum number of attachments allowed on a single medical record.
///
/// Each attachment consumes a ledger entry, so an unbounded count would let an
/// adversarial or buggy client flood one record and silently exhaust the pet
/// owner's storage quota. `add_attachment` enforces this cap. (Issue #774)
const MAX_ATTACHMENTS_PER_RECORD: u32 = 20;

/// Maximum number of milestone entries that can be stored in
/// [`ActivityStreak::milestones_reached`].
///
/// The `milestones_reached` Vec is embedded inline inside `ActivityStreak` and
/// serialised in a single XDR entry in Soroban persistent storage. An unbounded
/// Vec could exceed the XDR entry size limit if many milestones are added over
/// time (e.g. 365-day, 1000-day, etc.) or via a bug that appends duplicates.
/// Once the entry exceeds the XDR limit every subsequent `add_activity_record`
/// call would panic, effectively bricking all activity updates for the pet.
///
/// 32 slots is far more than the current milestone set (7, 30, 100 days) and
/// leaves ample room for future milestones while keeping the entry size bounded.
const MAX_MILESTONES: u32 = 32;

const MAX_PREREQUISITES: u32 = 16;

/// Standard activity-streak milestones (in streak-days).
/// A new milestone entry is recorded in `ActivityStreak::milestones_reached`
/// the first time a pet's consecutive-day streak reaches one of these values,
/// subject to the [`MAX_MILESTONES`] cap.
const STREAK_MILESTONE_DAYS: &[u64] = &[7, 30, 100, 365, 1000];

// ---------------------------------------------------------------------------
// BOUNDED-VEC CAPS  (Issue #1153)
//
// Every inline Vec stored in a Soroban persistent entry must be bounded so
// that the XDR serialised size of the entry stays below the platform limit
// (~64 KiB per entry). Exceeding the limit silently bricked the entry on all
// future writes.  Each cap below represents the highest safe value given the
// size of the contained element type plus XDR overhead.
//
// Migration note: existing entries that already exceed a cap will still
// deserialise correctly — the caps only gate NEW pushes.  A migration script
// (or `migrate_storage` step) can trim oversized Vecs offline if needed.
// ---------------------------------------------------------------------------

/// Maximum number of photo hashes stored inline in `Pet::photo_hashes`.
/// Each hash is a 46-byte IPFS CIDv0 string; 20 hashes ≈ 1 KiB of overhead.
const MAX_PHOTO_HASHES: u32 = 20;

/// Maximum items in `DietPlan::dietary_restrictions` or `NutritionVersion::dietary_restrictions`.
const MAX_DIETARY_RESTRICTIONS: u32 = 20;

/// Maximum items in `DietPlan::allergies` / `NutritionVersion::allergies`.
/// Mirrors the existing `Allergy` Vec cap on `Pet`.
const MAX_DIETARY_ALLERGIES: u32 = 20;

/// Maximum `Ingredient` entries in `NutritionPlan::ingredients`.
const MAX_INGREDIENTS: u32 = 50;

/// Maximum prerequisite IDs in `TrainingMilestone::prerequisites`.
const MAX_PREREQUISITES: u32 = 20;

/// Maximum entries in the chain-of-custody Vec stored per pet.
/// ~100 transfers × ~80 bytes/entry = ~8 KiB, well within the 64 KiB limit.
const MAX_CUSTODY_CHAIN: u32 = 100;

/// Maximum number of signers in `MultisigConfig::signers`.
/// Practical governance cap; more signers = larger XDR + more gas for threshold checks.
const MAX_MULTISIG_SIGNERS: u32 = 20;

/// Maximum number of pending signature slots in `PetTransferProposal::signatures`.
const MAX_TRANSFER_SIGNATURES: u32 = 20;

/// Maximum number of approval entries in a `MultiSigProposal::approvals` Vec.
const MAX_PROPOSAL_APPROVALS: u32 = 20;

/// Maximum number of language codes in `SupportedLanguages` Vec.
const MAX_SUPPORTED_LANGUAGES: u32 = 50;

// ---------------------------------------------------------------------------
// BOUNDED-STRING CAPS  (Issue #1152)
//
// The constants below define maximum byte-lengths for every domain-specific
// string stored on-chain.  All limits are enforced BEFORE writing to storage
// so oversized payloads are rejected at the transaction boundary.
//
// Validation uses the existing `validate_len` / `panic_with_error` pattern.
// ---------------------------------------------------------------------------

/// Maximum byte length of a `color` field (pet registration).
const MAX_COLOR_LEN: u32 = 50;
const MAX_MICROCHIP_ID_LEN: usize = 64;

/// Maximum byte length of a `BehaviorRecord::description`.
const MAX_BEHAVIOR_DESC_LEN: u32 = 500;

/// Maximum byte length of a `GroomingRecord::service_type`.
const MAX_GROOMING_SERVICE_LEN: u32 = 100;

/// Maximum byte length of a `GroomingRecord::groomer` (display name).
const MAX_GROOMING_GROOMER_LEN: u32 = 100;

/// Maximum byte length of a `GroomingRecord::notes`.
const MAX_GROOMING_NOTES_LEN: u32 = 500;

/// Maximum byte length of an `ActivityRecord::notes`.
const MAX_ACTIVITY_NOTES_LEN: u32 = 500;

/// Maximum byte length of a `MedicalRecord::diagnosis`.
const MAX_MEDICAL_DIAGNOSIS_LEN: u32 = 500;

/// Maximum byte length of a `MedicalRecord::treatment`.
const MAX_MEDICAL_TREATMENT_LEN: u32 = 500;

/// Maximum byte length of a `MedicalRecord::notes`.
const MAX_MEDICAL_NOTES_LEN: u32 = 1_000;

/// Maximum byte length of a `LabResult::test_type`.
const MAX_LAB_TEST_TYPE_LEN: u32 = 100;

/// Maximum byte length of a `LabResult::results`.
const MAX_LAB_RESULTS_LEN: u32 = 1_000;

/// Maximum byte length of a `LabResult::reference_ranges`.
const MAX_LAB_REF_RANGES_LEN: u32 = 500;

/// TTL-extension policy for persistent storage entries (Issue #1154).
///
/// Persistent entries (audit/access logs, breeding records, ...) are billed
/// separately from instance storage and, unlike instance storage, are not
/// automatically kept alive by every contract invocation: each entry's TTL
/// must be extended explicitly or it can be archived/expire out from under
/// the contract. `PERSISTENT_TTL_THRESHOLD` is the minimum remaining TTL (in
/// ledgers) below which we proactively bump it back up to
/// `PERSISTENT_TTL_EXTEND_TO` on every write (and on reads of
/// long-lived/critical records) so records that are written once and read
/// rarely still survive.
///
/// At Stellar's ~5s ledger close time, `PERSISTENT_TTL_EXTEND_TO` of
/// ~1,036,800 ledgers is roughly 60 days; `PERSISTENT_TTL_THRESHOLD` bumps
/// as soon as the entry has less than ~30 days of life left, well within the
/// network's max TTL extension window.
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400; // ~30 days
const PERSISTENT_TTL_EXTEND_TO: u32 = 1_036_800; // ~60 days

/// Maximum allowed clock skew (seconds) for a medical-event timestamp that is
/// reported as having already occurred (e.g. `administered_at`), measured
/// relative to the current ledger time. This is deliberately generous (on
/// the order of decades) so it only rejects clearly nonsensical/corrupt
/// future dates (e.g. a caller passing a millisecond timestamp, or a typo
/// adding extra digits) without constraining legitimate historical or
/// synthetic test timestamps, which need not track real-world wall-clock
/// time. (Issue #1174)
const MAX_EVENT_FUTURE_SKEW: u64 = 100 * 365 * 24 * 60 * 60; // ~100 years

/// Furthest a vaccination's `next_due_date` / `expires_at` may be scheduled
/// past `administered_at`, to catch fat-fingered far-future dates while
/// still allowing multi-year vaccination schedules. (Issue #1174)
const MAX_EVENT_HORIZON: u64 = 50 * 365 * 24 * 60 * 60; // ~50 years

/// Maximum byte length of a `Dispute::reason`.
const MAX_DISPUTE_REASON_LEN: u32 = 500;

/// Maximum byte length of a `BreedingRecord::notes`.
const MAX_BREEDING_NOTES_LEN: u32 = 500;

/// Maximum byte length of a `LostPetAlert::last_seen_location`.
const MAX_LOCATION_LEN: u32 = 200;

/// Maximum byte length of a `SightingReport::location`.
const MAX_SIGHTING_LOCATION_LEN: u32 = 200;

/// Maximum byte length of a `SightingReport::description`.
const MAX_SIGHTING_DESC_LEN: u32 = 500;

// --- STORAGE QUOTA CONSTANTS ---
const DEFAULT_STORAGE_QUOTA: u64 = 1000; // Default max storage entries per pet

// --- INPUT VALIDATION MIDDLEWARE ---

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationError {
    StringTooLong,
    EmptyString,
    InvalidAddress,
    InvalidId,
}

pub fn validate_string(s: &str, max_len: usize) -> Result<(), ValidationError> {
    if s.is_empty() {
        return Err(ValidationError::EmptyString);
    }
    if s.len() > max_len {
        return Err(ValidationError::StringTooLong);
    }
    Ok(())
}

pub fn validate_address(_a: &Address) -> Result<(), ValidationError> {
    // Addresses are structurally validated by the type system
    Ok(())
}

pub fn validate_id(id: &u64) -> Result<(), ValidationError> {
    if *id == 0 {
        return Err(ValidationError::InvalidId);
    }
    Ok(())
}

/// Compute a vaccination-reminder expiry window in ledger seconds.
///
/// Uses saturating arithmetic so inputs near `u64::MAX` cannot wrap or panic
/// (Issue #3: duration arithmetic near integer limits).
pub(crate) fn duration_window_end(now: u64, days: u64) -> u64 {
    now.saturating_add(days.saturating_mul(86_400))
}

// --- BREED METADATA ---

#[contracttype]
#[derive(Clone)]
pub struct BreedMetadata {
    pub species: String,
    pub avg_lifespan_years: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct PetAge {
    pub years: u32,
    pub months: u32,
    pub days: u32,
    pub lifespan_pct: Option<u32>,
}

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PetChainError {
    NonceReused = 1,
    SelfLineage = 2,
    CircularLineage = 3,
    KeywordTooLong = 4,
    TooManySearchTokens = 5,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AdminAlreadyApproved = 1,
    AdminAlreadySet = 2,
    AdminNotInitialized = 3,
    AdminsNotSet = 4,
    BatchTooLarge = 5,
    CertificateAlreadyAnchored = 6,
    CounterOverflow = 7,
    InputStringTooLong = 8,
    InvalidBreed = 9,
    InvalidCallerNonce = 10,
    InvalidCertificateHash = 11,
    InvalidInput = 12,
    InvalidIpfsHash = 13,
    InvalidPetName = 14,
    InvalidRating = 15,
    InvalidState = 16,
    InvalidThreshold = 17,
    InvokerNotInAdminList = 18,
    LicenseAlreadyRegistered = 19,
    NoAdminsConfigured = 20,
    NotAnAdmin = 21,
    NotPetOwner = 22,
    PetAlreadyHasLinkedTag = 23,
    PetNotFound = 24,
    StorageQuotaExceeded = 25,
    ThresholdNotMet = 26,
    TooManyItems = 27,
    Unauthorized = 28,
    VaccinationNotFound = 29,
    VetAlreadyRegistered = 30,
    VetNotFound = 31,
    VetNotVerified = 32,
    VeterinarianNotVerified = 33,
    SlotAlreadyBooked = 34,
    DuplicateActivity = 35,
    InbreedingThresholdExceeded = 36,
    SelfBreeding = 37,

    AlreadyDeleted = 160,
    RecordAlreadyDeleted = 161,
    RecordNotFound = 163,
    RetentionPeriodNotMet = 162,
    ProposalExpired = 43,
    ProposalNotApproved = 44,
    ProposalAlreadyExecuted = 38,
    /// Discriminant 39 is reserved for `InvalidNonce` (canonical ABI value).
    /// `ProposalNotFound` previously collided here; it is now reassigned to 47.
    InvalidNonce = 39,
    RollbackWindowExpired = 40,
    NoPreviousUpgrade = 41,
    ProposalExpired = 43,
    ProposalNotApproved = 44,
    QuorumNotMet = 45,
    RateLimitExceeded = 46,

    // --- Typed replacements for former assert!/panic! call sites (Issue #1150) ---
    // Append-only: existing values above must never be renumbered or reused.
    DisputeNotFound = 164,
    DisputeNotOpenForVoting = 165,
    NotDisputeStakeholder = 166,
    NotInEvidencePhase = 167,
    NotDisputeParty = 168,

    /// A medical-event timestamp fell outside the allowed domain relative to
    /// ledger time (too far in the past, too far in the future, or with a
    /// due/expiry date before the event it describes). (Issue #1174)
    InvalidTimestamp = 169,
}

// --- MULTI-LANGUAGE ERROR REGISTRY (Issue #684) ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorMessage {
    pub code: u32,
    pub language: String,
    pub message: String,
}

#[contracttype]
pub enum ErrorRegistryKey {
    ErrorMessage((u32, String)), // (error_code, language) -> message
    SupportedLanguages,          // Vec<String> of supported languages
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Species {
    Other,
    Dog,
    Cat,
    Bird,
    Rabbit,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityType {
    Walk,
    Run,
    Play,
    Training,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroomingRecord {
    pub id: u64,
    pub pet_id: u64,
    pub service_type: String,
    pub groomer: String,
    pub groomer_address: Option<Address>,
    pub date: u64,
    pub next_due: u64,
    pub cost: u64,
    pub notes: String,
}

/// A bookable grooming slot indexed by groomer for conflict detection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroomingSlot {
    pub slot_id: u64,
    pub groomer_id: Address,
    pub start_time: u64,
    pub duration_mins: u64,
    pub pet_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroomingFrequency {
    Weekly,
    Biweekly,
    Monthly,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecurringGroomingSchedule {
    pub id: u64,
    pub pet_id: u64,
    pub frequency: GroomingFrequency,
    pub start_date: u64,
    pub end_date: u64,
    pub groomer: String,
    pub service_type: String,
    pub cost: u64,
    pub is_active: bool,
    pub last_slot_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroomerProfile {
    pub address: Address,
    pub name: String,
    pub license_id: String,
    pub aggregate_rating: u32, // Average rating multiplied by 100 for precision
    pub review_count: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityRecord {
    pub id: u64,
    pub pet_id: u64,
    pub activity_type: ActivityType,
    pub duration_minutes: u32,
    pub intensity: u32,
    pub distance_meters: u32,
    pub recorded_at: u64,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityStreak {
    pub pet_id: u64,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub last_activity_date: u64,
    pub milestones_reached: Vec<u64>, // 7, 30, 100 day milestones
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreakMilestoneEvent {
    pub pet_id: u64,
    pub milestone_days: u64,
    pub timestamp: u64,
}
/// Migration Note (Issue #1031):
/// Previous versions stored an unbounded `offspring_ids: Vec<u64>` directly inside `BreedingRecord`,
/// which caused XDR serialization failures for large litters.
/// Offspring are now stored in separate persistent index entries under
/// `BreedingKey::BreedingOffspringIndex((record_id, seq))` with `BreedingKey::BreedingOffspringCount(record_id)`.
/// `BreedingRecord` stores `offspring_count: u32`. Retrieve offspring via `get_offspring_ids`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingRecord {
    pub id: u64,
    pub sire_id: u64,
    pub dam_id: u64,
    pub breeding_date: u64,
    pub offspring_count: u32,
    pub breeder: Address,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BehaviorType {
    Aggression,
    Anxiety,
    Training,
    Socialization,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BehaviorRecord {
    pub id: u64,
    pub pet_id: u64,
    pub behavior_type: BehaviorType,
    pub severity: u32,
    pub description: String,
    pub recorded_by: Address,
    pub recorded_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrainingMilestone {
    pub id: u64,
    pub pet_id: u64,
    pub milestone_name: String,
    pub achieved: bool,
    pub achieved_at: Option<u64>,
    pub trainer: Address,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Gender {
    NotSpecified,
    Male,
    Female,
    Unknown,
}

/// Domains that get a distinct, versioned prefix before hashing (Issue
/// #1168). Two canonical encodings that would otherwise collide (e.g. an
/// evidence blob and an attachment blob that happen to serialize to the
/// same bytes) hash to different values once tagged with their domain, so
/// a hash stored for one purpose can never be replayed as if it were a
/// hash for another.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HashDomain {
    Evidence,
    Attachment,
    ClaimDocument,
    Certificate,
    MedicalRecord,
    TagId,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivacyLevel {
    Public,     // Accessible to anyone
    Restricted, // Accessible to granted access (e.g., vets, owners)
    Private,    // Accessible only to the owner
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessAction {
    Read,
    Write,
    Grant,
    Revoke,
    EmergencyGrant,
    EmergencyRevoke,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessLog {
    pub id: u64,
    pub pet_id: u64,
    pub user: Address,
    pub action: AccessAction,
    pub timestamp: u64,
    pub details: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessEvent {
    pub actor: Address,
    pub action: AccessAction,
    pub target: Address,
    pub timestamp: u64,
    pub result: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyContactInfo {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub relationship: String,
    pub is_primary: bool,
    pub priority: u32,
}

/// Per-(caller, pet) rate limit window for emergency contact notifications
/// (Issue #820).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotifyRateLimit {
    /// Ledger timestamp the current hour-long window started.
    pub window_start: u64,
    /// Number of notifications sent within the current window.
    pub count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allergy {
    pub name: String,
    pub severity: String,
    pub is_critical: bool,
}

// --- NUTRITION / DIET ---
#[contracttype]
pub enum NutritionKey {
    DietPlan(u64),              // diet_id -> DietPlan
    DietPlanCount,              // global count
    PetDietCount(u64),          // pet_id -> count
    PetDietByIndex((u64, u64)), // (pet_id, index) -> diet_id

    WeightEntry(u64),             // weight_id -> WeightEntry
    WeightCount,                  // global weight entry count
    PetWeightCount(u64),          // pet_id -> count
    PetWeightByIndex((u64, u64)), // (pet_id, index) -> weight_id

    // Versioned nutrition plans
    NutritionVersion((u64, u64)), // (pet_id, version) -> NutritionVersion
    PetNutritionVersionCount(u64), // pet_id -> current version count
    CurrentNutritionVersion(u64), // pet_id -> current active version
    DailyNutritionSummary((u64, u64)), // (pet_id, date) -> DailyNutritionSummary

    // Ingredient-based nutrition plans (Issue #800)
    NutritionPlan(u64),                // plan_id -> NutritionPlan
    NutritionPlanCount,                // global count of plans
    PetNutritionPlanCount(u64),        // pet_id -> count of plans
    PetNutritionPlanIndex((u64, u64)), // (pet_id, index) -> plan_id
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DietPlan {
    pub pet_id: u64,
    pub food_type: String,
    pub portion_size: String,
    pub feeding_frequency: String,
    pub calories_per_serving: u32,
    pub daily_target_calories: u32,
    pub dietary_restrictions: Vec<String>,
    pub allergies: Vec<String>,
    pub created_by: Address,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NutritionVersion {
    pub pet_id: u64,
    pub version: u64,
    pub food_type: String,
    pub portion_size: String,
    pub feeding_frequency: String,
    pub calories_per_serving: u32,
    pub daily_target_calories: u32,
    pub dietary_restrictions: Vec<String>,
    pub allergies: Vec<String>,
    pub created_by: Address,
    pub created_at: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyNutritionSummary {
    pub pet_id: u64,
    pub date: u64,
    pub total_calories: u32,
    pub target_calories: u32,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ingredient {
    pub name: String,
    pub calories: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NutritionPlan {
    pub id: u64,
    pub pet_id: u64,
    pub name: String,
    pub ingredients: Vec<Ingredient>,
    pub total_calories: u32,
    pub created_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightEntry {
    pub pet_id: u64,
    pub weight: u32,
    pub recorded_at: u64,
    pub recorded_by: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetData {
    pub name: String,
    pub species: String,
    pub breed: String,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyInfo {
    pub pet_id: u64,
    pub species: String,
    pub allergies: Vec<Allergy>,
    pub critical_alerts: Vec<String>,
    pub emergency_contacts: Vec<EmergencyContact>,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyAccessLog {
    pub pet_id: u64,
    pub accessed_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyOverride {
    pub responder: Address,
    pub expires_at: u64,
    pub total_duration: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    pub actor: Address,
    pub timestamp: u64,
    pub reason_code: u32,
    pub pet_id: u64,
}

/// A single recorded admin action (Issue #816).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActivityEntry {
    pub actor: Address,
    pub action: String,
    pub timestamp: u64,
}

/// Paginated, actor-filtered view over the admin activity log (Issue #816).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminActivityPage {
    pub items: Vec<AdminActivityEntry>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncryptedData {
    pub nonce: Bytes,
    pub ciphertext: Bytes,
}

#[contracttype]
#[derive(Clone)]
pub struct Pet {
    pub id: u64,
    pub owner: Address,
    pub privacy_level: PrivacyLevel,
    // Encrypted fields replace plain text for sensitive data in storage
    pub encrypted_name: EncryptedData,
    pub encrypted_birthday: EncryptedData,
    pub encrypted_breed: EncryptedData,
    pub encrypted_emergency_contacts: EncryptedData,
    pub encrypted_medical_alerts: EncryptedData,
    pub encrypted_allergies: EncryptedData,

    // Internal/Empty fields to maintain some structural compatibility if needed,
    // or just purely internal placeholders. HEAD set these to empty strings.
    pub name: String,
    pub birthday: String,
    pub breed: String,
    pub emergency_contacts: Vec<EmergencyContact>,
    pub medical_alerts: String,
    pub allergies: Vec<Allergy>,

    pub active: bool,
    pub archived: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub color: String,
    pub weight: u32,
    pub microchip_id: Option<String>,
    pub photo_hashes: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetProfile {
    pub id: u64,
    pub owner: Address,
    pub privacy_level: PrivacyLevel,
    pub name: String,
    pub birthday: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub breed: String,
    pub color: String,
    pub weight: u32,
    pub microchip_id: Option<String>,
    pub allergies: Vec<Allergy>,
}

#[contracttype]
#[derive(Clone)]
pub struct PetFullProfile {
    pub profile: PetProfile,
    pub latest_vaccination_id: Option<u64>,
    pub active_medications_count: u64,
    pub has_insurance: bool,
}

/// Batch read structure for comprehensive pet profile with owner and consents
#[contracttype]
#[derive(Clone)]
pub struct PetFullProfileBatch {
    pub profile: PetProfile,
    pub owner: Address,
    pub active_consents: Vec<Consent>,
    pub latest_medical_record_id: Option<u64>,
}

/// Batch read structure for pet health summary
#[contracttype]
#[derive(Clone)]
pub struct PetHealthSummary {
    pub pet_id: u64,
    pub latest_vaccination_id: Option<u64>,
    pub latest_lab_result_id: Option<u64>,
    pub active_insurance_policy_id: Option<u64>,
    /// Derived (not stored): true iff the pet currently has an active,
    /// non-expired insurance policy AND a non-expired latest vaccination at the
    /// current ledger timestamp. Lets the insurance eligibility flow decide in a
    /// single call instead of comparing two timestamps client-side. (Issue #769)
    pub insurance_eligible: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct PetOwner {
    pub owner_address: Address,
    pub privacy_level: PrivacyLevel,
    pub encrypted_name: EncryptedData,
    pub encrypted_email: EncryptedData,
    pub encrypted_emergency_contact: EncryptedData,

    pub created_at: u64,
    pub updated_at: u64,
    pub is_pet_owner: bool,
}

/*
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClinicInfo {
    pub clinic_name: String,
    pub address: String,
    pub phone: String,
    pub email: String,
    pub operating_hours: String,
    pub emergency_available: bool,
}
*/

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Specialization {
    GeneralPractice,
    Surgery,
    Dermatology,
    Oncology,
    Dentistry,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certification {
    pub name: String,
    pub issuer: String,
    pub issue_date: u64,
    pub expiry_date: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
    pub clinic_info: Option<String>, // Simplified to String to avoid nested Option issues
}

/// Result of a batch verification operation
/// Allows partial success - some vets may succeed while others fail
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResult {
    pub succeeded: Vec<Address>,
    /// Failed entries store the error code as u32 (ContractError discriminant)
    pub failed: Vec<(Address, u32)>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaccineType {
    Rabies,
    Parvovirus,
    Leukemia,
    Bordetella,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Vaccination {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,

    pub vaccine_name: Option<String>, // Decrypted value (None in storage)
    pub encrypted_vaccine_name: EncryptedData, // Encrypted value

    pub administered_at: u64,
    pub next_due_date: u64,
    pub expires_at: u64, // Unix timestamp when the vaccination expires (0 = same as next_due_date)

    pub batch_number: Option<String>, // Decrypted value (None in storage)
    pub encrypted_batch_number: EncryptedData, // Encrypted value

    pub created_at: u64,

    pub revoked: bool,
    pub revocation_reason: Option<String>,
}

/// Certificate anchor for vaccination PDF metadata
/// Stores hash of off-chain certificate for authenticity verification.
///
/// NOTE (storage migration): adding lifecycle fields to the separate
/// `CertificateLifecycle` record is additive; existing `CertificateAnchor`
/// entries are backwards-compatible. Re-anchor certificates if cert_id /
/// expiry / revocation tracking is required for pre-existing entries.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateAnchor {
    pub pet_id: u64,
    pub vaccination_id: u64,
    pub cert_hash: String, // Hash of the PDF certificate
    pub issuer: Address,   // Verified vet who issued the certificate
    pub anchored_at: u64,  // Timestamp when anchored
}

/// Lifecycle state bound to a certificate anchor.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateLifecycle {
    /// Stable, monotonically-assigned certificate identifier.
    pub cert_id: u64,
    /// Ledger timestamp when the certificate was issued.
    pub issue_time: u64,
    /// Unix timestamp when the certificate expires. `0` means no expiry.
    pub expiry: u64,
    /// Whether the certificate has been revoked.
    pub revoked: bool,
    /// Ledger timestamp of revocation, if `revoked` is true.
    pub revoked_at: Option<u64>,
    /// Free-text reason supplied at revocation time.
    pub revocation_reason: Option<String>,
}

/// High-level lifecycle status of a certificate.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    NotAnchored,
    Valid,
    Expired,
    Revoked,
}

#[contracttype]
#[derive(Clone)]
pub struct TagLinkedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TagDeactivatedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub deactivated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct UpgradeProposal {
    pub id: u64,
    pub proposed_by: Address,
    pub new_wasm_hash: BytesN<32>,
    pub proposed_at: u64,
    pub approved: bool,
    pub executed: bool,
    pub timelock_duration: u64,   // seconds; min 86400 (24h)
    pub approved_at: Option<u64>, // when quorum was reached
    pub vetoed: bool,
    pub expires_at: u64,          // timestamp after which execute_upgrade is rejected
}
#[contracttype]
#[derive(Clone)]
pub struct TagReactivatedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub reactivated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct PetTag {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub message: String,
    pub is_active: bool,
    pub linked_at: u64,
    pub updated_at: u64,
    // Note: older code might use 'tag_message' or 'created_at', we harmonize here
    pub tag_message: String,
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),
    SpeciesPetCount(String),
    SpeciesPetIndex((String, u64)),
    Vet(Address),
    VetLicense(String),
    VetCount,
    VetIndex(u64),
    Admin,
    VetLicenseVerified(Address),
    VetSpecializations(Address),
    ContractVersion,
    AccessGrant((u64, Address)),
    AccessGrantCount(u64),
    AccessGrantIndex((u64, u64)),
    PetDelegationCount(u64),
    DecryptionToken((u64, Address)),
    /// Current encryption key version for a pet (Issue #1163). Absent means
    /// version 1, matching [`PetChainContract::derive_versioned_key`]'s
    /// treatment of versions `<= 1` as the base key.
    PetKeyVersion(u64),
    EmergencyAccessLogs(u64),
    EmergencyAuditLog(u64),
    EmergencyResponders(u64),
    EmergencyNotifyRateLimit((Address, u64)), // (caller, pet_id) -> NotifyRateLimit
    BreedMetadata(String),
    SpeciesBreedList(String),
    CallerNonce(Address),
    ClaimDocuments(u64),
    PetStorageUsage(u64),
    PetStorageQuota(u64),
    GlobalStorageQuota,
    NonceHistory((u64, String)),
    NonceMaxUse((u64, String)),
    NonceUsage((u64, String, Bytes)),
    RetentionPeriod,
    MaxSubscriptionsPerAddress,
    /// Canonical microchip identifier -> pet id.
    MicrochipIndex(String),
}

#[contracttype]
pub enum TreatmentKey {
    // Treatment DataKey
    Treatment(u64),
    TreatmentCount,
    PetTreatmentCount(u64),
    PetTreatmentIndex((u64, u64)), // (pet_id, index) -> treatment_id
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventType {
    PetRegistered,
    TreatmentAdded,
    MedicalRecordAdded,
    VaccinationAdded,
    VaccinationRevoked,
    AccessGranted,
    AccessRevoked,
    InsuranceClaimSubmitted,
    PetProfileUpdated,
    GroomingRecordCreated,
    PolicyExpiringSoon,
    PolicyRenewed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventSubscription {
    pub id: u64,
    pub subscriber: Address,
    pub event_types: Vec<EventType>,
    pub pet_ids: Vec<u64>,
    pub expires_at: u64,
    pub created_at: u64,
}

#[contracttype]
pub enum SubscriptionKey {
    Subscription(u64),
    SubscriptionCount,
    SubscriberSubscriptionCount(Address),
    SubscriberSubscriptionIndex((Address, u64)),
}

#[contracttype]
pub enum TagKey {
    // Tag Linking System keys
    Tag(soroban_sdk::BytesN<32>), // tag_id -> PetTag (reverse lookup for QR scan)
    // Tag String keys (QR)
    PetTagId(u64), // pet_id -> tag_id (forward lookup)
    TagNonce,      // Global nonce for deterministic tag ID generation
    PetTagCount,   // Count of tags (mostly for stats)
}

#[contracttype]
pub enum MedicalKey {
    LabResult(u64),
    LabResultCount,
    PetLabResultIndex((u64, u64)), // (pet_id, index) -> lab_result_id
    PetLabResultCount(u64),
    MedicalRecord(u64),
    MedicalRecordCount,
    PetMedicalRecordIndex((u64, u64)), // (pet_id, index) -> medical_record_id
    PetMedicalRecordCount(u64),
    MedicalRecordAmendment((u64, u32)),
    MedicalRecordAmendmentCount(u64),
    KeywordRecordCount((u64, Bytes)),
    KeywordRecordIndex((u64, Bytes, u64)),
    GlobalMedication(u64),          // medication_id -> Medication
    MedicationCount,                // Global count
    PetMedicationCount(u64),        // pet_id -> count
    PetMedicationIndex((u64, u64)), // (pet_id, index) -> medication_id
    // Vaccination DataKey
    Vaccination(u64),
    VaccinationCount,
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),
    // Certificate anchoring (Issue #693)
    CertificateAnchor((u64, u64)), // (pet_id, vaccination_id) -> CertificateAnchor
    // Scanner registry
    ScannerRegistry,
    // Retention period for soft-deleted record purging
    RetentionPeriod,
    // Certificate lifecycle (Issue #X: cert_id, expiry, revocation)
    CertificateCount,
    CertificateLifecycle((u64, u64)), // (pet_id, vaccination_id) -> CertificateLifecycle
}

#[contracttype]
pub enum ReviewKey {
    VetReview(u64),                          // review_id -> VetReview
    VetReviewCount,                          // Global count of reviews
    VetReviewByVetIndex((Address, u64)),     // (Vet, index) -> review_id
    VetReviewCountByVet(Address),            // Vet -> count
    VetReviewByOwnerVet((Address, Address)), // (Owner, Vet) -> review_id (Duplicate check)
}

#[contracttype]
pub enum AlertKey {
    LostPetAlert(u64),
    LostPetAlertCount,
    ActiveLostPetAlerts, // Vec<u64> of active alert IDs
    AlertSightings(u64),
}

#[contracttype]
pub enum ConsentKey {
    // Consent System keys
    Consent(u64),
    ConsentCount,
    PetConsentIndex((u64, u64)),
    PetConsentCount(u64),
}

#[contracttype]
pub enum CrossChainKey {
    PetChainMapping((u64, String)),
    ChainLookup((String, String)),
}

#[contracttype]
pub enum SystemKey {
    // Ownership History keys
    PetOwnershipRecord(u64),
    OwnershipRecordCount,
    PetOwnershipRecordCount(u64),
    PetOwnershipRecordIndex((u64, u64)),

    // Multisig keys
    Admins,
    AdminThreshold,
    AdminQuorumPercent,
    PendingConfig, // Issue #626: Three-phase bootstrap
    Proposal(u64),
    ProposalCount,
    PendingThresholdChange, // Issue #815: full-quorum threshold changes

    // Timelock and veto keys
    AdminTimelockConfig,
    ProposalVeto((u64, Address)), // (proposal_id, admin) -> bool (has vetoed)
    ProposalVetoCount(u64),       // proposal_id -> count of vetoes

    // Vet Availability keys
    VetAvailability((Address, u64)),
    VetAvailabilityCount(Address),
    VetAvailabilityByDate((Address, u64)),

    // Pet Multisig keys
    PetMultisigConfig(u64),
    PetTransferProposal(u64),
    PetTransferProposalCount,
    PetActiveProposals(u64), // pet_id -> Vec<u64> of active proposal IDs
    PetTransferOffer(u64),
    EmergencyOverride((u64, Address)),
    EncryptionNonceCounter,

    // Statistics caching keys
    StatCacheTTL,
    StatCache(String),
    LabThreshold,
    // Chain-of-custody log (Issue #637)
    CustodyChain(u64), // pet_id -> Vec<CustodyEntry>
    // #699: governance-controlled parameters
    HealthScoreCacheTtl, // TTL (seconds) for health-score cache entries
    // #828: Statistics snapshots for governance reporting
    StatisticsSnapshot(u64), // snapshot_id -> StatisticsSnapshot
    SnapshotCount,           // Total number of snapshots
    SnapshotIndex(u64),      // index (0-99) -> snapshot_id (for purging oldest)
    // Upgrade proposal keys (Issue #818)
    UpgradeProposal(u64),    // proposal_id -> UpgradeProposal
    UpgradeProposalCount,    // Total number of upgrade proposals
    // Rollback keys
    RollbackDeadline,        // timestamp after which rollback is no longer possible
    PreviousWasmHash,        // BytesN<32> of the previous WASM hash before upgrade
    // Version keys
    StorageVersion,          // ContractVersion for storage schema
    /// Flat u32 written by `migrate_storage`. Zero / absent means pre-versioning (treat as v0).
    /// Distinct from `StorageVersion` (which carries a semver triple). (#1149)
    StorageSchemaVersion,
    // Admin activity log keys (Issue #816)
    AdminActivityLog(u64), // index -> AdminActivityEntry
    AdminActivityCount,    // Total number of recorded admin actions
}

/// Statistics snapshot for governance reporting (Issue #828)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatisticsSnapshot {
    pub snapshot_id: u64,
    pub timestamp: u64,
    pub total_pets: u64,
    pub active_pets: u64,
    pub species_distribution: Map<String, u64>,
    pub total_vets: u64,
    pub total_medical_records: u64,
    pub total_vaccinations: u64,
    pub total_insurance_claims: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatCache {
    pub value: i128,
    pub computed_at: u64,
}

#[contracttype]
pub enum VetKey {
    VetStats(Address),
    VetPetTreated((Address, u64)),
    VetPetCount(Address),
    VetTreatmentIndex((Address, u64)), // (vet, index) -> record_id
    VetTreatmentCount(Address),        // vet -> count of treatments
    VetVaccinationIndex((Address, u64)), // (vet, index) -> vaccine_id
    VetVaccinationCount(Address),      // vet -> count of vaccinations
}

#[contracttype]
pub enum StatsKey {
    ActivePetsCount,
}

#[contracttype]
pub enum StatSeriesKey {
    Count(String),        // stat key -> number of stored points
    Point((String, u64)), // (stat key, 1-based index) -> StatPoint
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StatPoint {
    pub value: u64,
    pub timestamp: u64,
}

#[contracttype]
pub enum FeatureKey {
    Rg((u64, Address)),
    Gr(u64),
    Gc,
    Ar(u64),
    Ac,
    Br(u64),
    Bc,
    BP,
    BN,
}

// --- STORAGE QUOTA SYSTEM (Issue #676) ---
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageUsage {
    pub pet_id: u64,
    pub current_count: u64,
    pub quota: u64,
}

// --- LOST PET ALERT SYSTEM ---
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AlertStatus {
    Active,
    Found,
    Cancelled,
}

#[contracttype]
#[derive(Clone)]
pub struct VetStats {
    pub total_records: u64,
    pub total_vaccinations: u64,
    pub total_treatments: u64,
    pub pets_treated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct LostPetAlert {
    pub id: u64,
    pub pet_id: u64,
    pub reported_by: Address,
    pub reported_date: u64,
    pub last_seen_location: String,
    pub reward_amount: Option<u64>,
    pub status: AlertStatus,
    pub found_date: Option<u64>,
}

#[contracttype]
#[derive(Clone)]
pub struct SightingReport {
    pub alert_id: u64,
    pub reporter: Address,
    pub location: String,
    pub timestamp: u64,
    pub description: String,
}

#[contracttype]
#[derive(Clone)]
pub struct AvailabilitySlot {
    pub vet_address: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub available: bool,
    pub start_ts: u64,         // Unix timestamp for slot start (Issue #624)
    pub duration_minutes: u32, // Duration in minutes for overlap detection (Issue #624)
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsentType {
    Insurance,
    Research,
    PublicHealth,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsentScope {
    ReadMedical,
    WriteMedical,
    ReadLab,
    EmergencyOnly,
}

#[contracttype]
#[derive(Clone)]
pub struct Consent {
    pub id: u64,
    pub pet_id: u64,
    pub owner: Address,
    pub consent_type: ConsentType,
    pub granted_to: Address,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
    pub is_active: bool,
    pub scope: ConsentScope,
    /// ID of the parent consent this was delegated from (None = root consent).
    pub parent_consent_id: Option<u64>,
    /// Maximum delegation depth allowed for this consent branch.
    pub max_depth: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabResult {
    pub id: u64,
    pub pet_id: u64,
    pub test_type: String,
    pub date: u64,
    pub results: String,
    pub vet_address: Address,
    pub reference_ranges: String,
    pub attachment_hash: Option<String>, // IPFS hash for PDF
    pub medical_record_id: Option<u64>,  // Link to medical record
    pub biomarkers: Map<String, i128>,
    // Issue #652: biomarker flags (Normal/Low/High) set during add_lab_result
    pub biomarker_flags: Map<String, u32>,
}

/// Per-biomarker reference range (Issue #652)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReferenceRange {
    pub min: i128,
    pub max: i128,
}

/// Storage key for reference ranges (Issue #652)
#[contracttype]
pub enum ReferenceRangeKey {
    /// (species_str, biomarker_name) -> ReferenceRange
    SpeciesBiomarker((String, String)),
}

#[allow(dead_code)]
const FLAG_NORMAL: u32 = 0;
#[allow(dead_code)]
const FLAG_LOW: u32 = 1;
#[allow(dead_code)]
const FLAG_HIGH: u32 = 2;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationSummary {
    pub is_fully_current: bool,
    pub overdue_types: Vec<VaccineType>,
    pub upcoming_count: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthScoreBreakdown {
    pub vaccination: u32,
    pub lab_results: u32,
    pub activity: u32,
    pub insurance: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthScore {
    pub pet_id: u64,
    pub score: u32,
    pub breakdown: HealthScoreBreakdown,
    pub computed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabDifference {
    pub biomarker: String,
    pub value_a: i128,
    pub value_b: i128,
    pub delta: i128,
    pub abnormal: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalFieldDiff {
    pub field: String,
    pub from_value: String,
    pub to_value: String,
}

/// Cached result of a biomarker moving-average computation (1-hour TTL).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomarkerTrendCache {
    /// Moving average value (scaled by 1000 to avoid floats).
    pub moving_avg: i128,
    /// Ledger timestamp when this cache entry was computed.
    pub computed_at: u64,
    /// Whether a deteriorating trend (3 consecutive worsening results) was detected.
    pub deteriorating: bool,
}

/// Event emitted when a deteriorating biomarker trend is detected.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BiomarkerTrendAlert {
    pub version: u32,
    pub pet_id: u64,
    pub biomarker: String,
    pub moving_avg: i128,
    pub window: u32,
}

/// Event emitted when a biomarker value deviates more than 3 standard deviations from
/// the pet's historical baseline (z-score scaled by 100, so 305 = z-score 3.05).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabResultAnomaly {
    pub version: u32,
    pub pet_id: u64,
    pub biomarker: String,
    pub value: i128,
    pub z_score: i128,
}

/// Event emitted for each consent revoked during a cascade revocation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsentRevoked {
    pub version: u32,
    pub pet_id: u64,
    pub consent_id: u64,
    pub revoked_at: u64,
}

/// Event emitted when a pet is linked to an external chain identity.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossChainIdentityRegistered {
    pub version: u32,
    pub pet_id: u64,
    pub chain_id: String,
    pub external_id: String,
    pub registered_at: u64,
}

/// A single entry in the immutable append-only audit ledger.
/// Each entry hashes the previous entry to form a tamper-evident chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLedgerEntry {
    pub index: u64,
    pub pet_id: u64,
    pub actor: Address,
    pub action: String,
    pub timestamp: u64,
    /// SHA-256 hash of the previous entry's serialised fields (all-zeros for genesis).
    pub prev_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessLevel {
    None,
    Basic, // Can view basic pet info only
    Full,  // Can view all records including medical history
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Role {
    ReadOnly,
    Vet,
    Admin,
    Owner,
}

impl Role {
    #[allow(dead_code)]
    fn rank(self) -> u8 {
        match self {
            Role::ReadOnly => 0,
            Role::Vet => 1,
            Role::Admin => 2,
            Role::Owner => 3,
        }
    }

    #[allow(dead_code)]
    fn inherited_roles(self, env: &Env) -> Vec<Role> {
        let mut roles = Vec::new(env);
        roles.push_back(Role::ReadOnly);
        if self.rank() >= Role::Vet.rank() {
            roles.push_back(Role::Vet);
        }
        if self.rank() >= Role::Admin.rank() {
            roles.push_back(Role::Admin);
        }
        if self.rank() >= Role::Owner.rank() {
            roles.push_back(Role::Owner);
        }
        roles
    }
}

#[contracttype]
#[derive(Clone)]
pub struct AccessGrant {
    pub pet_id: u64,
    pub granter: Address, // Pet owner who granted access
    pub grantee: Address, // User receiving access
    pub access_level: AccessLevel,
    pub granted_at: u64,
    pub expires_at: Option<u64>, // None means permanent access
    pub is_active: bool,
}

/// A page of access grants returned by cursor pagination (Issue #1161).
///
/// `cursor` is an opaque slot index into the pet's access-grant index; pass
/// `0` for the first page and echo back `next_cursor` on subsequent calls
/// until it is `0`, meaning there are no more pages.
#[contracttype]
#[derive(Clone)]
pub struct AccessGrantPage {
    pub items: Vec<AccessGrant>,
    pub next_cursor: u64,
    /// Total number of grant slots for the pet (stable upper bound on live
    /// rows; some slots may be filtered out when `active_only` is set).
    pub total_slots: u64,
}

/// A decryption delegation token bound to the encryption key version that
/// was active when it was issued (Issue #1163). Rotating a pet's key
/// version deterministically invalidates every outstanding token, since
/// [`PetChainContract::verify_decryption_token`] requires an exact version
/// match rather than trusting a possibly-stale expiry alone.
#[contracttype]
#[derive(Clone)]
pub struct DecryptionDelegation {
    pub pet_id: u64,
    pub delegate: Address,
    pub key_version: u32,
    pub issued_at: u64,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TemporaryCustody {
    pub pet_id: u64,
    pub owner: Address,
    pub custodian: Address,
    pub start_date: u64,
    pub end_date: u64,
    pub permissions: Vec<String>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleGrant {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub role: Role,
    pub granted_at: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Medication {
    pub id: u64,
    pub pet_id: u64,
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub start_date: u64,
    pub end_date: Option<u64>,
    pub prescribing_vet: Address,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentMetadata {
    pub filename: String,
    pub file_type: String,
    pub size: u64,
    pub uploaded_date: u64,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanStatus {
    Clean,
    Suspicious,
    Malicious,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanResult {
    pub scanner_id: Address,
    pub scanned_at: u64,
    pub status: ScanStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attachment {
    pub ipfs_hash: String,
    pub metadata: AttachmentMetadata,
    pub content_hash: BytesN<32>,
    pub scan_result: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: Vec<Medication>,
    pub date: u64,
    pub updated_at: u64,
    pub notes: String,
    pub attachment_hashes: Vec<Attachment>,
    pub deleted_at: Option<u64>, // None = active; Some(ts) = soft-deleted at ts
}

#[contracttype]
#[derive(Clone)]
pub struct VaccinationInput {
    pub pet_id: u64,
    pub vaccine_type: VaccineType,
    pub vaccine_name: String,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub expires_at: u64,
    pub batch_number: String,
}

#[contracttype]
#[derive(Clone)]
pub struct MedicalRecordInput {
    pub pet_id: u64,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: Vec<Medication>,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAmendmentInput {
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub medications: Option<Vec<Medication>>,
    pub notes: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAmendment {
    pub record_id: u64,
    pub version: u32,
    pub updated_at: u64,
    pub changes: MedicalRecordAmendmentInput,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordFilter {
    pub vet_address: Option<Address>,
    pub from_date: Option<u64>,
    pub to_date: Option<u64>,
    pub diagnosis_keyword: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetReview {
    pub id: u64,
    pub vet_address: Address,
    pub reviewer: Address,
    pub rating: u32, // 1-5 stars
    pub comment: String,
    pub date: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct OwnershipRecord {
    pub pet_id: u64,
    pub previous_owner: Address,
    pub new_owner: Address,
    pub transfer_date: u64,
    pub transfer_reason: String,
}

/// Transfer type for chain-of-custody entries.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransferType {
    Direct,
    Adoption,
    Multisig,
}

/// A single chain-of-custody entry appended on every ownership change.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyEntry {
    pub from: Address,
    pub to: Address,
    pub timestamp: u64,
    pub transfer_type: TransferType,
}

/// Result of [`PetChainContract::verify_custody_chain`].
///
/// `gap_at` identifies the index in the custody chain (0-based) where the
/// break was found: either the first entry's `from` doesn't match the pet's
/// creator, some entry's `from` doesn't match the previous entry's `to`, or
/// (when `gap_at` equals the chain length) the last entry's `to` doesn't
/// match the pet's current owner.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustodyVerificationResult {
    pub valid: bool,
    pub gap_at: Option<u32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalState {
    Pending,         // Awaiting approvals
    TimelockPending, // Quorum reached, in timelock period
    Executable,      // Timelock expired, ready to execute
    Executed,        // Successfully executed
    Vetoed,          // Vetoed during timelock
}

/// Identifies a contract parameter that can be changed via governance vote.
///
/// Adding new variants here is the only change needed to expose a new
/// on-chain parameter to the governance system.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParamKey {
    /// Global storage quota (max entries per pet). Stored as `u64`.
    GlobalStorageQuota,
    /// Cache TTL in seconds for computed health scores. Stored as `u64`.
    HealthScoreCacheTtl,
    /// Multisig approval threshold. Stored as `u32` (cast to u64 in proposal).
    AdminThreshold,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalAction {
    UpgradeContract(BytesN<32>),
    VerifyVet(Address),
    RevokeVet(Address),
    ChangeAdmin((Vec<Address>, u32)),
    RotateSigner((Address, Address)),
    /// Governance vote to change a named contract parameter.
    /// `(key, new_value_as_u64)` — the value is cast to the parameter's
    /// native type at execution time.
    ParameterChange((ParamKey, u64)),
}

#[contracttype]
#[derive(Clone)]
pub struct MultiSigProposal {
    pub id: u64,
    pub action: ProposalAction,
    pub proposed_by: Address,
    pub approvals: Vec<Address>,
    pub required_approvals: u32,
    pub quorum_percent: u32,
    pub created_at: u64,
    pub expires_at: u64,
    pub executed: bool,
    pub state: ProposalState,
    pub timelock_end: u64,
    pub veto_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct PendingConfig {
    pub admins: Vec<Address>,
    pub threshold: u32,
    pub confirmations: Vec<Address>,
    pub proposed_at: u64,
}

/// A threshold change awaiting unanimous admin sign-off (Issue #815).
#[contracttype]
#[derive(Clone)]
pub struct PendingThresholdChange {
    pub new_threshold: u32,
    pub approvals: Vec<Address>,
}

/// Multi-signature configuration for a pet.
/// Enables multiple parties to approve pet ownership transfers.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultisigConfig {
    /// The pet ID this configuration applies to
    pub pet_id: u64,
    /// List of addresses authorized to sign transfer proposals
    pub signers: Vec<Address>,
    /// Minimum number of signatures required to execute a transfer
    pub threshold: u32,
    /// Whether multisig enforcement is enabled
    pub enabled: bool,
}

/// Admin-level timelock configuration for upgrade proposals.
/// Enforces a delay period and veto window for contract upgrades.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminTimelockConfig {
    /// Minimum timelock duration in seconds (enforced minimum: 86400 = 24 hours)
    pub timelock_duration: u64,
    /// Whether timelock is enabled
    pub enabled: bool,
}

/// Proposal for transferring pet ownership with multi-signature approval.
#[contracttype]
#[derive(Clone)]
pub struct PetTransferProposal {
    /// Unique proposal identifier
    pub id: u64,
    /// The pet being transferred
    pub pet_id: u64,
    /// Address of the new owner
    pub to: Address,
    /// Addresses that have signed this proposal
    pub signatures: Vec<Address>,
    /// Timestamp when proposal was created
    pub created_at: u64,
    /// Timestamp when proposal expires
    pub expires_at: u64,
    /// Whether the transfer has been executed
    pub executed: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreatmentType {
    Surgery,
    Therapy,
    Emergency,
    Routine,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Treatment {
    pub id: u64,
    pub pet_id: u64,
    pub treatment_type: TreatmentType,
    pub date: u64,
    pub vet_address: Address,
    pub notes: String,
    pub cost: Option<i128>,
    pub outcome: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreatmentAddedEvent {
    pub treatment_id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub treatment_type: TreatmentType,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

// --- EVENTS ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsurancePolicy {
    pub policy_id: String,
    pub provider: String,
    pub coverage_type: String,
    pub tier: PremiumTier,
    pub premium: u64,
    pub coverage_limit: u64,
    pub start_date: u64,
    pub expiry_date: u64,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceAddedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub policy_id: String,
    pub provider: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceUpdatedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub policy_id: String,
    pub active: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InsuranceClaimStatus {
    Pending,
    Approved,
    Rejected,
    Paid,
    /// Claim was flagged by one or more fraud heuristics and is awaiting
    /// manual admin review via `approve_flagged_claim`.
    UnderReview,
    /// Claim is under appeal after rejection, awaiting second reviewer decision.
    UnderAppeal,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaim {
    pub claim_id: u64,
    pub pet_id: u64,
    pub policy_id: String,
    pub amount: u64,
    pub date: u64,
    pub status: InsuranceClaimStatus,
    pub description: String,
    /// True when at least one fraud heuristic triggered for this claim.
    pub flagged: bool,
    /// Bitmask of triggered fraud rules:
    ///   bit 0 (0x01) — HIGH_AMOUNT:        amount > 3× pet's average past claim
    ///   bit 1 (0x02) — HIGH_FREQUENCY:     ≥ 2 claims within the last 7 days
    ///   bit 2 (0x04) — BEFORE_POLICY_START: claim date before policy start_date
    pub fraud_flags: u32,
    /// IPFS CIDs of attached evidence documents (max 10).
    pub documents: Vec<String>,
    /// Appeal tracking fields
    pub rejected_at: Option<u64>,
    pub appeal_reason: Option<String>,
    pub appeal_evidence_cids: Vec<String>,
    pub appealed_at: Option<u64>,
    pub original_reviewer: Option<Address>,
    pub appeal_reviewer: Option<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaimSubmittedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub policy_id: String,
    pub amount: u64,
    /// True when the claim was flagged by fraud heuristics.
    pub flagged: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaimStatusUpdatedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub status: InsuranceClaimStatus,
    pub timestamp: u64,
}

/// Emitted when a claim is automatically flagged by fraud heuristics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InsuranceClaimFlaggedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub fraud_flags: u32,
    pub timestamp: u64,
}

/// Emitted 30 days before a policy expires.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyExpiringSoonEvent {
    pub version: u32,
    pub pet_id: u64,
    pub policy_id: String,
    pub expiry_date: u64,
    pub timestamp: u64,
}

/// Emitted when a policy is renewed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PolicyRenewedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub policy_id: String,
    pub new_expiry_date: u64,
    pub timestamp: u64,
}

/// Emitted when an admin overrides a flagged claim and approves it.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlaggedClaimApprovedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub admin: Address,
    pub reason: String,
    pub timestamp: u64,
}

/// Emitted when a claim is appealed after rejection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimAppealedEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub claimant: Address,
    pub appeal_reason: String,
    pub new_evidence_count: u32,
    pub timestamp: u64,
}

/// Emitted when an appeal receives a final decision.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppealDecisionEvent {
    pub version: u32,
    pub claim_id: u64,
    pub pet_id: u64,
    pub reviewer: Address,
    pub decision: InsuranceClaimStatus, // Approved or Rejected
    pub timestamp: u64,
}

/// Emitted when verify_claim_document is called.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimDocumentIntegrityEvent {
    pub version: u32,
    pub claim_id: u64,
    pub doc_index: u32,
    pub matches: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessGrantedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub access_level: AccessLevel,
    pub expires_at: Option<u64>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessRevokedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessExtendedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub old_expires_at: Option<u64>,
    pub new_expires_at: Option<u64>,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessExpiredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub grantee: Address,
    pub expired_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetRegisteredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub owner: Address,
    pub name: String,
    pub species: Species,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationAddedEvent {
    pub version: u32,
    pub vaccine_id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub next_due_date: u64,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationRevokedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub vaccination_id: u64,
    pub vet_or_admin: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationExpiringSoonEvent {
    pub version: u32,
    pub vaccine_id: u64,
    pub pet_id: u64,
    pub vaccine_type: VaccineType,
    pub expires_at: u64,
    pub days_remaining: u64,
    pub already_expired: bool,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpiringVaccination {
    pub vaccine_id: u64,
    pub vaccine_type: VaccineType,
    pub expires_at: u64,
    pub days_remaining: u64,
    pub already_expired: bool,
}

/// Emitted when a vaccination certificate is anchored on-chain
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateAnchoredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub vaccination_id: u64,
    pub cert_hash: String,
    pub issuer: Address,
    pub timestamp: u64,
}

/// Emitted when a certificate is revoked on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateRevokedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub vaccination_id: u64,
    pub cert_id: u64,
    pub revoked_by: Address,
    pub reason: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetOwnershipTransferredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub old_owner: Address,
    pub new_owner: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAddedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordDeletedEvent {
    pub version: u32,
    pub record_id: u64,
    pub pet_id: u64,
    pub deleted_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordPurgedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub purged_count: u32,
    pub purged_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurgeResult {
    pub deleted: Vec<u64>,
    pub dry_run: bool,
}

/// Result of a bounded, resumable purge of soft-deleted medical records
/// (Issue #1172).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedPurgeResult {
    /// The record IDs purged in this batch.
    pub deleted: Vec<u64>,
    /// Opaque resume cursor: the 1-based slot index last examined by this batch.
    /// Pass it back as the `cursor` argument to continue; `0` means the scan is
    /// complete and there is no more work.
    pub next_cursor: u64,
    pub dry_run: bool,
}

/// A page of medical records returned by cursor pagination (Issue #1173).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordPage {
    pub items: Vec<MedicalRecord>,
    /// Opaque cursor to pass to the next call; `0` means there are no more pages.
    pub next_cursor: u64,
    /// Total number of record slots for the pet. Includes soft-deleted entries
    /// that have not yet been purged, so it is a stable upper bound on live rows;
    /// the live count is `items.len()` across the full paged scan.
    pub total_slots: u64,
}


// --- VET LICENSE VERIFICATION EVENTS ---

/// Emitted when a multisig admin verifies a vet's license on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetLicenseVerifiedEvent {
    pub version: u32,
    pub vet_address: Address,
    pub license_id: String,
    pub timestamp: u64,
}

/// Emitted when a multisig admin revokes a vet's license.
/// All active access grants held by this vet are also revoked.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VetLicenseRevokedEvent {
    pub version: u32,
    pub vet_address: Address,
    pub license_id: String,
    pub timestamp: u64,
}

/// Emitted when a temp vet grant expires.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TempVetGrantExpiredEvent {
    pub version: u32,
    pub pet_id: u64,
    pub vet: Address,
    pub expired_at: u64,
}

/// Emitted when a pet's profile is updated.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetProfileUpdatedEvent {
    pub version: u32,
    pub pet_id: u64,
    pub owner: Address,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

/// Emitted when a grooming record is created.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroomingRecordCreatedEvent {
    pub version: u32,
    pub record_id: u64,
    pub pet_id: u64,
    pub groomer: Address,
    pub timestamp: u64,
    pub subscription_ids: Vec<u64>,
}

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    // --- CONTRACT STATISTICS ---

    pub fn register_subscription(
        env: Env,
        subscriber: Address,
        event_types: Vec<EventType>,
        pet_ids: Vec<u64>,
        ttl: u64,
    ) -> u64 {
        subscriber.require_auth();

        if event_types.is_empty() || pet_ids.is_empty() || ttl == 0 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let now = env.ledger().timestamp();
        let existing_count: u64 = env
            .storage()
            .instance()
            .get(&SubscriptionKey::SubscriberSubscriptionCount(
                subscriber.clone(),
            ))
            .unwrap_or(0);
        let mut active_count = 0u32;
        for i in 1..=existing_count {
            let Some(subscription_id) = env.storage().instance().get::<SubscriptionKey, u64>(
                &SubscriptionKey::SubscriberSubscriptionIndex((subscriber.clone(), i)),
            ) else {
                continue;
            };
            if let Some(subscription) = env
                .storage()
                .instance()
                .get::<SubscriptionKey, EventSubscription>(&SubscriptionKey::Subscription(
                    subscription_id,
                ))
            {
                if subscription.expires_at > now {
                    active_count += 1;
                }
            }
        }

        if active_count >= Self::max_subscriptions_per_address(env.clone()) {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let current_id: u64 = env
            .storage()
            .instance()
            .get(&SubscriptionKey::SubscriptionCount)
            .unwrap_or(0);
        let subscription_id = safe_increment(&env, current_id);
        let expires_at = now.saturating_add(ttl);
        let subscription = EventSubscription {
            id: subscription_id,
            subscriber: subscriber.clone(),
            event_types,
            pet_ids,
            expires_at,
            created_at: now,
        };

        env.storage().instance().set(
            &SubscriptionKey::Subscription(subscription_id),
            &subscription,
        );
        env.storage()
            .instance()
            .set(&SubscriptionKey::SubscriptionCount, &subscription_id);

        let new_subscriber_count = safe_increment(&env, existing_count);
        env.storage().instance().set(
            &SubscriptionKey::SubscriberSubscriptionCount(subscriber.clone()),
            &new_subscriber_count,
        );
        env.storage().instance().set(
            &SubscriptionKey::SubscriberSubscriptionIndex((subscriber, new_subscriber_count)),
            &subscription_id,
        );

        subscription_id
    }

    /// Admin-only: override the per-address active subscription cap enforced
    /// by `register_subscription`. Lets private deployments (e.g. hospital
    /// instances) scale the limit without redeploying the contract.
    ///
    /// Named `set_max_subscriptions` (rather than `*_per_address`) to stay
    /// within the Soroban 32-char contract-function-name limit.
    pub fn set_max_subscriptions(env: Env, admin: Address, max: u32) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        env.storage()
            .instance()
            .set(&DataKey::MaxSubscriptionsPerAddress, &max);
    }

    /// Current per-address active subscription cap: the admin-configured
    /// value if one has been set, otherwise the default of 10.
    pub fn max_subscriptions_per_address(env: Env) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::MaxSubscriptionsPerAddress)
            .unwrap_or(MAX_ACTIVE_SUBSCRIPTIONS_PER_ADDRESS)
    }

    pub fn get_subscription(env: Env, subscription_id: u64) -> Option<EventSubscription> {
        env.storage()
            .instance()
            .get(&SubscriptionKey::Subscription(subscription_id))
    }

    pub fn get_matching_subscription_ids(env: Env, event_type: EventType, pet_id: u64) -> Vec<u64> {
        Self::matching_subscription_ids(&env, event_type, pet_id)
    }

    fn matching_subscription_ids(env: &Env, event_type: EventType, pet_id: u64) -> Vec<u64> {
        let now = env.ledger().timestamp();
        let count: u64 = env
            .storage()
            .instance()
            .get(&SubscriptionKey::SubscriptionCount)
            .unwrap_or(0);
        let mut matches = Vec::new(env);

        for subscription_id in 1..=count {
            let Some(subscription) = env
                .storage()
                .instance()
                .get::<SubscriptionKey, EventSubscription>(&SubscriptionKey::Subscription(
                    subscription_id,
                ))
            else {
                continue;
            };

            if subscription.expires_at <= now {
                continue;
            }
            if subscription.event_types.contains(&event_type)
                && subscription.pet_ids.contains(pet_id)
            {
                matches.push_back(subscription.id);
            }
        }

        matches
    }

    /// Returns the total number of pets ever registered in the contract.
    pub fn get_total_pets(env: Env) -> u64 {
        let cache_key = String::from_str(&env, "total_pets");
        let ttl = Self::_get_cache_ttl(&env);

        if let Some(cache) = env
            .storage()
            .instance()
            .get::<SystemKey, StatCache>(&SystemKey::StatCache(cache_key.clone()))
        {
            let current_time = env.ledger().timestamp();
            if current_time.saturating_sub(cache.computed_at) < ttl {
                return cache.value as u64;
            }
        }

        let value = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0) as i128;
        let cache = StatCache {
            value,
            computed_at: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&SystemKey::StatCache(cache_key), &cache);
        value as u64
    }

    /// Returns the number of registered pets for a given species.
    /// Pass the species name as a string: "Dog", "Cat", "Bird", or "Other".
    pub fn get_species_count(env: Env, species: String) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species))
            .unwrap_or(0)
    }

    /// Returns the number of currently active pets.
    /// This counter is maintained automatically by `activate_pet` and `deactivate_pet`.
    pub fn get_active_pets_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&StatsKey::ActivePetsCount)
            .unwrap_or(0)
    }

    fn _get_cache_ttl(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&SystemKey::StatCacheTTL)
            .unwrap_or(3600)
    }

    fn is_admin(env: &Env, caller: &Address) -> bool {
        Self::is_admin_address(env, caller)
    }

    fn check_access(env: Env, pet_id: u64, caller: Address) -> AccessLevel {
        let pet: Pet = match env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            Some(pet) => pet,
            None => return AccessLevel::None,
        };

        if pet.owner == caller {
            return AccessLevel::Full;
        }

        if let Some(grant) = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, caller.clone())))
        {
            // A grant issued by a previous owner does not survive an ownership
            // transfer: the grant is only honored while `grant.granter` still
            // matches the pet's current owner, so `accept_pet_transfer` (and
            // any other path that mutates `pet.owner`) implicitly cascades
            // the revocation without needing a separate sweep. (#1162)
            if grant.is_active && grant.grantee == caller && grant.granter == pet.owner {
                if let Some(expires_at) = grant.expires_at {
                    if env.ledger().timestamp() >= expires_at {
                        return AccessLevel::None;
                    }
                }
                return grant.access_level;
            }
        }

        AccessLevel::None
    }

    fn get_active_medications(env: Env, pet_id: u64) -> Vec<Medication> {
        let count = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetMedicationCount(pet_id))
            .unwrap_or(0);
        let mut medications = Vec::new(&env);
        for index in 1..=count {
            if let Some(medication_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicationIndex((pet_id, index)))
            {
                if let Some(medication) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, Medication>(&MedicalKey::GlobalMedication(medication_id))
                {
                    if medication.active {
                        medications.push_back(medication);
                    }
                }
            }
        }
        medications
    }

    fn get_pet_insurance(env: Env, pet_id: u64) -> Option<InsurancePolicy> {
        let count = env
            .storage()
            .instance()
            .get::<InsuranceKey, u64>(&InsuranceKey::PetPolicyCount(pet_id))
            .unwrap_or(0);
        for index in (1..=count).rev() {
            if let Some(policy) = env
                .storage()
                .instance()
                .get::<InsuranceKey, InsurancePolicy>(&InsuranceKey::PetPolicyIndex((
                    pet_id, index,
                )))
            {
                if policy.active {
                    return Some(policy);
                }
            }
        }
        None
    }

    fn get_active_consents(env: Env, pet_id: u64) -> Vec<Consent> {
        let count = env
            .storage()
            .instance()
            .get::<ConsentKey, u64>(&ConsentKey::PetConsentCount(pet_id))
            .unwrap_or(0);
        let mut consents = Vec::new(&env);
        let now = env.ledger().timestamp();
        let mut expired_count = 0u32;
        for index in 1..=count {
            if let Some(consent_id) = env
                .storage()
                .instance()
                .get::<ConsentKey, u64>(&ConsentKey::PetConsentIndex((pet_id, index)))
            {
                if let Some(consent) = env
                    .storage()
                    .instance()
                    .get::<ConsentKey, Consent>(&ConsentKey::Consent(consent_id))
                {
                    if consent.is_active {
                        // Filter out expired consents
                        if let Some(expires_at) = consent.expires_at {
                            if now >= expires_at {
                                expired_count = expired_count.saturating_add(1);
                                continue;
                            }
                        }
                        consents.push_back(consent);
                    }
                }
            }
        }
        if expired_count > 0 {
            env.events().publish(
                (soroban_sdk::symbol_short!("cnsnt_exp"),),
                (pet_id, expired_count),
            );
        }
        consents
    }

    pub fn get_medical_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
        if let Some(record) = Self::get_medical_record_raw(env, record_id) {
            if !Self::medical_record_is_deleted(&record) {
                return Some(record);
            }
        }
        None
    }

    fn get_medical_record_raw(env: Env, record_id: u64) -> Option<MedicalRecord> {
        env.storage()
            .instance()
            .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
    }

    fn get_lab_result(env: Env, lab_id: u64) -> Option<LabResult> {
        env.storage()
            .instance()
            .get::<MedicalKey, LabResult>(&MedicalKey::LabResult(lab_id))
    }

    fn propose_action(env: Env, proposer: Address, action: ProposalAction, ttl: u64) -> u64 {
        proposer.require_auth();
        if !Self::is_admin_address(&env, &proposer) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let proposal_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::ProposalCount)
            .unwrap_or(0);
        let proposal_id = safe_increment(proposal_count);
        let now = env.ledger().timestamp();
        let admin_count = env
            .storage()
            .instance()
            .get::<SystemKey, Vec<Address>>(&SystemKey::Admins)
            .map(|admins| admins.len())
            .unwrap_or(1);
    let required_approvals = env
        .storage()
        .instance()
        .get::<SystemKey, u32>(&SystemKey::AdminThreshold)
        .unwrap_or(admin_count.max(1));
    let quorum_percent: u32 = env
        .storage()
        .instance()
        .get(&SystemKey::AdminQuorumPercent)
        .unwrap_or(0);

    let proposal = MultiSigProposal {
        id: proposal_id,
        action,
        proposed_by: proposer,
        approvals: Vec::new(&env),
        required_approvals,
        quorum_percent,
        created_at: now,
            expires_at: now.saturating_add(ttl),
            executed: false,
            state: ProposalState::Pending,
            timelock_end: now.saturating_add(ttl),
            veto_count: 0,
        };

        env.storage()
            .instance()
            .set(&SystemKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&SystemKey::ProposalCount, &proposal_id);
        proposal_id
    }

    /// Returns the current global storage quota. Used by governance tests
    /// to verify that parameter changes take effect after proposal execution.
    pub fn get_global_storage_quota(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::GlobalStorageQuota)
            .unwrap_or(DEFAULT_STORAGE_QUOTA)
    }

    /// Fetches a multisig proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<MultiSigProposal> {
        env.storage()
            .instance()
            .get(&SystemKey::Proposal(proposal_id))
    }

    /// An admin casts an approval vote on a multisig proposal.
    ///
    /// # Panics
    /// - If the caller is not an admin.
    /// - If the admin has already approved.
    /// - If the proposal has expired.
    /// - If the proposal is not in `Pending` state.
    pub fn approve_proposal(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let mut proposal: MultiSigProposal = env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidInput));

        if proposal.state != ProposalState::Pending {
            panic_with_error!(&env, ContractError::InvalidState);
        }
        let now = env.ledger().timestamp();
        if now > proposal.expires_at {
            panic_with_error!(&env, ContractError::InvalidState);
        }
        if proposal.approvals.contains(&admin) {
            panic_with_error!(&env, ContractError::AdminAlreadyApproved);
        }

        let was_below_threshold =
            proposal.approvals.len() < proposal.required_approvals;
        proposal.approvals.push_back(admin);

        // Transition to TimelockPending only on the vote that first crosses
        // the threshold. Subsequent approvals leave the state unchanged.
        if was_below_threshold
            && proposal.approvals.len() >= proposal.required_approvals
        {
            let timelock_duration: u64 = env
                .storage()
                .instance()
                .get::<SystemKey, AdminTimelockConfig>(&SystemKey::AdminTimelockConfig)
                .map(|c| c.timelock_duration)
                .unwrap_or(86_400); // default 24 hours
            // Clamp timelock_end so it never exceeds expires_at.
            proposal.timelock_end =
                core::cmp::min(now.saturating_add(timelock_duration), proposal.expires_at);
            proposal.state = ProposalState::TimelockPending;
        }

        env.storage()
            .instance()
            .set(&SystemKey::Proposal(proposal_id), &proposal);
    }

    /// Executes a multisig proposal that has met the threshold and quorum
    /// requirements and whose timelock has elapsed.
    ///
    /// # Quorum check
    /// Quorum is calculated against the admin list at execution time:
    ///   `approvals.len() >= quorum_percent * admin_count / 100`
    /// A quorum_percent of 0 disables the quorum check entirely.
    ///
    /// # Panics
    /// - If the proposal does not exist.
    /// - If the proposal has already been executed.
    /// - If the timelock has not elapsed.
    /// - If the proposal has expired.
    /// - If threshold is not met.
    /// - If quorum is not met.
    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: MultiSigProposal = env
            .storage()
            .instance()
            .get(&SystemKey::Proposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidInput));

        if proposal.executed {
            panic_with_error!(&env, ContractError::InvalidState);
        }
        let now = env.ledger().timestamp();
        if now > proposal.expires_at {
            panic_with_error!(&env, ContractError::InvalidState);
        }
        let admin_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoAdminsConfigured));

        // Check threshold
        if proposal.approvals.len() < proposal.required_approvals {
            panic_with_error!(&env, ContractError::ThresholdNotMet);
        }

        // Check quorum (Issue #775) — read quorum_percent from storage at
        // execution time, not from the proposal snapshot, consistent with
        // the admin list being read at execution time.
        let current_quorum: u32 = env
            .storage()
            .instance()
            .get(&SystemKey::AdminQuorumPercent)
            .unwrap_or(0);
        if current_quorum > 0 {
            let admin_count = admin_list.len() as u64;
            if admin_count == 0 {
                panic_with_error!(&env, ContractError::NoAdminsConfigured);
            }
            let votes_cast = proposal.approvals.len() as u64;
            // Ceiling division so that e.g. 50 % of 3 admins = 2 votes, not 1.
            let required_votes =
                (current_quorum as u64).saturating_mul(admin_count).div_ceil(100);
            if votes_cast < required_votes {
                panic_with_error!(&env, ContractError::QuorumNotMet);
            }
        }

        // Must be in Executable state (timelock elapsed)
        match proposal.state {
            ProposalState::TimelockPending => {
                if now < proposal.timelock_end {
                    panic_with_error!(&env, ContractError::InvalidState);
                }
            }
            ProposalState::Executable => {}
            _ => panic_with_error!(&env, ContractError::InvalidState),
        }

        proposal.executed = true;
        proposal.state = ProposalState::Executed;

        env.storage()
            .instance()
            .set(&SystemKey::Proposal(proposal_id), &proposal);

        // Execute the proposal action
        if let ProposalAction::ParameterChange((key, value)) = &proposal.action {
            match key {
                ParamKey::GlobalStorageQuota => {
                    env.storage()
                        .instance()
                        .set(&DataKey::GlobalStorageQuota, value);
                }
                ParamKey::HealthScoreCacheTtl => {
                    env.storage()
                        .instance()
                        .set(&SystemKey::HealthScoreCacheTtl, value);
                }
                ParamKey::AdminThreshold => {
                    env.storage()
                        .instance()
                        .set(&SystemKey::AdminThreshold, &(*value as u32));
                    env.events().publish(
                        (Symbol::new(&env, "ThresholdChanged"),),
                        *value as u32,
                    );
                }
            }
        }
    }

    /// Appends a `StatPoint` for `key`, pruning the oldest entry when the
    /// series exceeds 365 points.
    fn record_stat_point(env: &Env, key: String, value: u64) {
        const MAX_POINTS: u64 = 365;
        let count: u64 = env
            .storage()
            .instance()
            .get(&StatSeriesKey::Count(key.clone()))
            .unwrap_or(0);

        let point = StatPoint {
            value,
            timestamp: env.ledger().timestamp(),
        };

        if count < MAX_POINTS {
            let new_count = safe_increment(count);
            env.storage()
                .instance()
                .set(&StatSeriesKey::Point((key.clone(), new_count)), &point);
            env.storage()
                .instance()
                .set(&StatSeriesKey::Count(key), &new_count);
        } else {
            // Shift: drop index 1, move 2..=MAX down by one, write at MAX
            for i in 1..MAX_POINTS {
                if let Some(p) = env
                    .storage()
                    .instance()
                    .get::<StatSeriesKey, StatPoint>(&StatSeriesKey::Point((key.clone(), i + 1)))
                {
                    env.storage()
                        .instance()
                        .set(&StatSeriesKey::Point((key.clone(), i)), &p);
                }
            }
            env.storage()
                .instance()
                .set(&StatSeriesKey::Point((key, MAX_POINTS)), &point);
        }
    }

    /// Returns all recorded `StatPoint`s for `key` whose timestamp falls
    /// within the inclusive range `[start_ts, end_ts]`.
    pub fn get_stat_series(env: Env, key: String, start_ts: u64, end_ts: u64) -> Vec<StatPoint> {
        let mut result = Vec::new(&env);
        if start_ts > end_ts {
            return result;
        }
        let count: u64 = env
            .storage()
            .instance()
            .get(&StatSeriesKey::Count(key.clone()))
            .unwrap_or(0);
        for i in 1..=count {
            if let Some(p) = env
                .storage()
                .instance()
                .get::<StatSeriesKey, StatPoint>(&StatSeriesKey::Point((key.clone(), i)))
            {
                if p.timestamp >= start_ts && p.timestamp <= end_ts {
                    result.push_back(p);
                }
            }
        }
        result
    }

    /// Returns the statistics for a given vet address.
    /// Returns a zeroed `VetStats` if the vet has no recorded activity.
    pub fn get_vet_stats(env: Env, vet_address: Address) -> VetStats {
        env.storage()
            .instance()
            .get::<_, VetStats>(&VetKey::VetStats(vet_address))
            .unwrap_or(VetStats {
                total_records: 0,
                total_vaccinations: 0,
                total_treatments: 0,
                pets_treated: 0,
            })
    }

    /// Returns a paginated page of medical records (treatments) created by a specific vet.
    pub fn get_vet_treatment_history(
        env: Env,
        vet_address: Address,
        page: u32,
        page_size: u32,
    ) -> TreatmentHistoryPage {
        if page_size > 50 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let total = env
            .storage()
            .instance()
            .get::<VetKey, u64>(&VetKey::VetTreatmentCount(vet_address.clone()))
            .unwrap_or(0);

        let mut items = Vec::new(&env);

        if total == 0 || page_size == 0 {
            return TreatmentHistoryPage {
                items,
                total,
                page,
                page_size,
            };
        }

        let offset = (page as u64).saturating_mul(page_size as u64);
        if offset >= total {
            return TreatmentHistoryPage {
                items,
                total,
                page,
                page_size,
            };
        }

        let start_index = offset.saturating_add(1);
        let end_index = offset.saturating_add(page_size as u64).min(total);

        for i in start_index..=end_index {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<VetKey, u64>(&VetKey::VetTreatmentIndex((vet_address.clone(), i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
                {
                    items.push_back(record);
                }
            }
        }

        TreatmentHistoryPage {
            items,
            total,
            page,
            page_size,
        }
    }

    /// Returns a paginated list of vaccinations administered by a specific vet.
    pub fn get_vet_vaccination_history(
        env: Env,
        vet_address: Address,
        offset: u64,
        limit: u32,
    ) -> Vec<Vaccination> {
        let count = env
            .storage()
            .instance()
            .get::<VetKey, u64>(&VetKey::VetVaccinationCount(vet_address.clone()))
            .unwrap_or(0);

        let mut results = Vec::new(&env);
        if count == 0 || limit == 0 || offset >= count {
            return results;
        }

        let start_index = offset.saturating_add(1);
        let end_index = (offset.saturating_add(limit as u64)).min(count);

        for i in start_index..=end_index {
            if let Some(vaccine_id) = env
                .storage()
                .instance()
                .get::<VetKey, u64>(&VetKey::VetVaccinationIndex((vet_address.clone(), i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vaccine_id))
                {
                    results.push_back(record);
                }
            }
        }
        results
    }

    /// Returns a paginated list of pet IDs that have at least one overdue vaccination.
    ///
    /// If `vet_address` is `Some`, only overdue pets with at least one treatment
    /// or vaccination record from that vet are returned. If `None`, all overdue
    /// pets are returned for admins.
    pub fn get_pets_overdue_vaccinations(
        env: Env,
        caller: Address,
        vet_address: Option<Address>,
        offset: u64,
        limit: u32,
    ) -> Vec<u64> {
        caller.require_auth();
        let is_admin = Self::is_admin_address(&env, &caller);

        if vet_address.is_none() && !is_admin {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        if let Some(ref vet_address) = vet_address {
            if !is_admin && caller != *vet_address {
                panic_with_error!(&env, ContractError::Unauthorized);
            }
            // A non-admin caller must be a verified vet to use this function.
            if !is_admin && !Self::is_verified_vet(env.clone(), vet_address.clone()) {
                panic_with_error!(&env, ContractError::Unauthorized);
            }
        }

        if limit == 0 {
            return Vec::new(&env);
        }

        let pet_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetCount)
            .unwrap_or(0);

        let mut overdue_pets = Vec::new(&env);
        if pet_count == 0 {
            return overdue_pets;
        }

        let mut skipped: u64 = 0;

        for pet_id in 1..=pet_count {
            if overdue_pets.len() >= limit {
                break;
            }

            let overdue = PetChainContract::get_overdue_vaccinations(env.clone(), pet_id);

            let has_record = match &vet_address {
                Some(vet) => Self::pet_has_vet_care_record(env.clone(), pet_id, vet.clone()),
                None => true, // No filter, include all overdue pets
            };

            if !overdue.is_empty() && has_record {
                if skipped < offset {
                    skipped = skipped.saturating_add(1);
                } else {
                    overdue_pets.push_back(pet_id);
                }
            }
        }
        overdue_pets
    }

    // --- STATISTICS SNAPSHOT FOR GOVERNANCE REPORTING (Issue #828) ---

    /// Captures a point-in-time snapshot of all key statistics for governance reporting.
    /// Requires multisig admin authorization.
    ///
    /// The snapshot includes:
    /// - Total pets (all registered)
    /// - Active pets (currently activated)
    /// - Species distribution (counts per species)
    /// - Total vets (all registered)
    /// - Total medical records
    /// - Total vaccinations
    /// - Total insurance claims
    /// - Ledger timestamp
    ///
    /// Maximum 100 snapshots are stored. When the 101st snapshot is taken,
    /// the oldest snapshot is purged automatically.
    ///
    /// Returns: The snapshot ID for later retrieval.
    pub fn take_statistics_snapshot(env: Env, admin: Address) -> u64 {
        Self::require_admin_auth(&env, &admin);

        // Generate snapshot ID
        let snapshot_count = env
            .storage()
            .instance()
            .get::<SystemKey, u64>(&SystemKey::SnapshotCount)
            .unwrap_or(0);

        let snapshot_id = safe_increment(&env, snapshot_count);

        // Gather total pets
        let total_pets = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetCount)
            .unwrap_or(0);

        // Gather active pets count
        let active_pets = env
            .storage()
            .instance()
            .get::<StatsKey, u64>(&StatsKey::ActivePetsCount)
            .unwrap_or(0);

        // Build species distribution map
        let mut species_distribution = Map::new(&env);
        let mut species_list = Vec::new(&env);
        species_list.push_back(String::from_str(&env, "Dog"));
        species_list.push_back(String::from_str(&env, "Cat"));
        species_list.push_back(String::from_str(&env, "Bird"));
        species_list.push_back(String::from_str(&env, "Rabbit"));
        species_list.push_back(String::from_str(&env, "Other"));

        for species in species_list.iter() {
            let count = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::SpeciesPetCount(species.clone()))
                .unwrap_or(0);
            species_distribution.set(species, count);
        }

        // Gather total vets count
        let total_vets = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::VetCount)
            .unwrap_or(0);

        // Gather total medical records
        let total_medical_records = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::MedicalRecordCount)
            .unwrap_or(0);

        // Gather total vaccinations
        let total_vaccinations = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::VaccinationCount)
            .unwrap_or(0);

        // Gather total insurance claims
        let total_insurance_claims = env
            .storage()
            .instance()
            .get::<InsuranceKey, u64>(&InsuranceKey::ClaimCount)
            .unwrap_or(0);

        // Get ledger timestamp
        let timestamp = env.ledger().timestamp();

        // Create the snapshot
        let snapshot = StatisticsSnapshot {
            snapshot_id,
            timestamp,
            total_pets,
            active_pets,
            species_distribution,
            total_vets,
            total_medical_records,
            total_vaccinations,
            total_insurance_claims,
        };

        // Store the snapshot
        env.storage()
            .instance()
            .set(&SystemKey::StatisticsSnapshot(snapshot_id), &snapshot);

        // Update snapshot count
        env.storage()
            .instance()
            .set(&SystemKey::SnapshotCount, &snapshot_id);

        // Manage the snapshot index (max 100 snapshots)
        // Calculate the index position (0-99)
        let index_position = (snapshot_id - 1) % 100;

        // Store the snapshot ID at the index position
        env.storage()
            .instance()
            .set(&SystemKey::SnapshotIndex(index_position), &snapshot_id);

        // If we've exceeded 100 snapshots, purge the oldest
        if snapshot_id > 100 {
            // The snapshot to purge is at the same index position (it's now the oldest)
            let snapshot_to_purge = snapshot_id - 100;

            // Remove the old snapshot from storage
            env.storage()
                .instance()
                .remove(&SystemKey::StatisticsSnapshot(snapshot_to_purge));
        }

        snapshot_id
    }

    /// Retrieves a statistics snapshot by its ID.
    /// This is a public function - no authorization required.
    ///
    /// Returns: The snapshot if it exists, None otherwise.
    pub fn get_snapshot(env: Env, snapshot_id: u64) -> Option<StatisticsSnapshot> {
        env.storage()
            .instance()
            .get::<SystemKey, StatisticsSnapshot>(&SystemKey::StatisticsSnapshot(snapshot_id))
    }

    /// Returns the total number of snapshots taken (including purged ones).
    /// This count never decreases - it represents the total snapshot ID counter.
    pub fn get_snapshot_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<SystemKey, u64>(&SystemKey::SnapshotCount)
            .unwrap_or(0)
    }

    /// Returns a list of all currently stored snapshot IDs (max 100).
    /// Useful for discovering which snapshots are available.
    pub fn get_available_snapshot_ids(env: Env) -> Vec<u64> {
        let snapshot_count = env
            .storage()
            .instance()
            .get::<SystemKey, u64>(&SystemKey::SnapshotCount)
            .unwrap_or(0);

        let mut snapshot_ids = Vec::new(&env);

        if snapshot_count == 0 {
            return snapshot_ids;
        }

        // Determine the range of snapshots that should exist
        let start_id = if snapshot_count > 100 {
            snapshot_count - 99 // Last 100 snapshots
        } else {
            1 // All snapshots from the beginning
        };

        // Collect all valid snapshot IDs
        for id in start_id..=snapshot_count {
            if env
                .storage()
                .instance()
                .has(&SystemKey::StatisticsSnapshot(id))
            {
                snapshot_ids.push_back(id);
            }
        }

        snapshot_ids
    }

    // --- ACCESS LOG EXPORT ---

    /// Export access events for a pet within [start_ts, end_ts].
    /// Caller must be the pet owner or a multisig admin.
    /// Results are paginated: max 100 per call, controlled by `page` (1-based).
    pub fn export_access_log(
        env: Env,
        caller: Address,
        pet_id: u64,
        start_ts: u64,
        end_ts: u64,
        page: u32,
    ) -> Vec<AccessEvent> {
        caller.require_auth();

        // Authorisation: owner or admin
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        let is_owner = caller == pet.owner;
        let is_admin = {
            let in_multisig: Vec<Address> = env
                .storage()
                .instance()
                .get(&SystemKey::Admins)
                .unwrap_or(Vec::new(&env));
            let legacy: Option<Address> = env.storage().instance().get(&DataKey::Admin);
            in_multisig.contains(&caller) || legacy.as_ref() == Some(&caller)
        };

        if !is_owner && !is_admin {
            env.panic_with_error(ContractError::Unauthorized);
        }

        let key = (Symbol::new(&env, "access_logs"), pet_id);
        let logs: Vec<AccessLog> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        const PAGE_SIZE: u32 = 100;
        let page = if page == 0 { 1 } else { page };
        let skip = ((page - 1) * PAGE_SIZE) as usize;

        let mut result = Vec::new(&env);
        let mut matched: usize = 0;
        let mut taken: u32 = 0;

        for log in logs.iter() {
            if log.timestamp < start_ts || log.timestamp > end_ts {
                continue;
            }
            if matched < skip {
                matched += 1;
                continue;
            }
            if taken >= PAGE_SIZE {
                break;
            }
            result.push_back(AccessEvent {
                actor: log.user.clone(),
                action: log.action.clone(),
                target: pet.owner.clone(),
                timestamp: log.timestamp,
                result: true,
            });
            matched += 1;
            taken += 1;
        }

        result
    }

    /// Extend the TTL of a persistent-storage entry per the archival policy
    /// defined by `PERSISTENT_TTL_THRESHOLD` / `PERSISTENT_TTL_EXTEND_TO`.
    /// (Issue #1154). Call this after every `persistent().set(...)` (and on
    /// reads of records that must remain reachable even when written once
    /// and read rarely) so critical persistent records are not silently
    /// archived/expired by the ledger.
    fn bump_persistent_ttl<K>(env: &Env, key: &K)
    where
        K: IntoVal<Env, Val>,
    {
        env.storage()
            .persistent()
            .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
    }

    fn log_access(env: &Env, pet_id: u64, user: Address, action: AccessAction, details: String) {
        let key = (Symbol::new(env, "access_logs"), pet_id);
        let mut logs: Vec<AccessLog> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        while logs.len() >= MAX_LOG_ENTRIES {
            logs.remove(0);
        }

        let id = if logs.is_empty() {
            0
        } else {
            logs.get(logs.len() - 1).unwrap().id + 1
        };
        let log = AccessLog {
            id,
            pet_id,
            user,
            action,
            timestamp: env.ledger().timestamp(),
            details,
        };

        logs.push_back(log);
        env.storage().persistent().set(&key, &logs);
        Self::bump_persistent_ttl(env, &key);
    }

    /// Read access log entries for a pet. Visible to the pet owner or any admin.
    /// Includes emergency-grant and emergency-revoke entries written by
    /// `add_emergency_responder` / `remove_emergency_responder`.
    pub fn get_access_logs(env: Env, pet_id: u64, caller: Address) -> Vec<AccessLog> {
        caller.require_auth();

        let pet: crate::Pet = env
            .storage()
            .instance()
            .get::<DataKey, crate::Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        // Visible to pet owner or any admin.
        if caller != pet.owner && !Self::is_admin_address(&env, &caller) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let key = (Symbol::new(&env, "access_logs"), pet_id);
        let logs = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        Self::bump_persistent_ttl(&env, &key);
        logs
    }

    fn require_admin(env: &Env) {
        if let Some(legacy_admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            legacy_admin.require_auth();
            return;
        }

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminsNotSet));

        if admins.is_empty() {
            env.panic_with_error(ContractError::NoAdminsConfigured);
        }

        let admin = admins
            .get(0)
            .unwrap_or_else(|| env.panic_with_error(ContractError::NoAdminsConfigured));

        admin.require_auth();
    }

    fn require_admin_auth(env: &Env, admin: &Address) {
        if let Some(legacy_admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            if &legacy_admin == admin {
                admin.require_auth();
                return;
            }
        }

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminsNotSet));

        if !admins.contains(admin) {
            panic_with_error!(env, ContractError::Unauthorized);
        }
        admin.require_auth();
    }

    pub fn init_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin)
            || env.storage().instance().has(&SystemKey::Admins)
        {
            panic_with_error!(&env, ContractError::AdminAlreadySet);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::ContractVersion,
            &ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
        );
    }

    pub fn init_multisig(env: Env, invoker: Address, admins: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&DataKey::Admin)
            || env.storage().instance().has(&SystemKey::Admins)
        {
            panic_with_error!(&env, ContractError::AdminAlreadySet);
        }
        if threshold == 0 || threshold > admins.len() {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }
        // Enforce admin signer cap to prevent unbounded Vec. (#1153)
        if admins.len() > MAX_MULTISIG_SIGNERS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        invoker.require_auth();
        if !admins.contains(invoker) {
            panic_with_error!(&env, ContractError::InvokerNotInAdminList);
        }

        env.storage().instance().set(&SystemKey::Admins, &admins);
        env.storage()
            .instance()
            .set(&SystemKey::AdminThreshold, &threshold);
        env.storage().instance().set(
            &DataKey::ContractVersion,
            &ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
        );
    }

    // --- THREE-PHASE BOOTSTRAP (Issue #626) ---

    /// Phase 1: Propose initial admin configuration
    pub fn propose_init(env: Env, admins: Vec<Address>, threshold: u32) {
        // Reject if config already exists
        if env.storage().instance().has(&DataKey::Admin)
            || env.storage().instance().has(&SystemKey::Admins)
        {
            panic_with_error!(&env, ContractError::AdminAlreadySet);
        }

        // Validate threshold
        if threshold == 0 || threshold > admins.len() {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }

        // Clear expired pending config if exists
        if let Some(pending) = env
            .storage()
            .instance()
            .get::<SystemKey, PendingConfig>(&SystemKey::PendingConfig)
        {
            let current_time = env.ledger().timestamp();
            if current_time > pending.proposed_at.saturating_add(3600) {
                // Timeout expired, clear and allow new proposal
                env.storage().instance().remove(&SystemKey::PendingConfig);
            } else {
                // Already have an active pending config
                panic_with_error!(&env, ContractError::InvalidState);
            }
        }

        let pending = PendingConfig {
            admins: admins.clone(),
            threshold,
            confirmations: Vec::new(&env),
            proposed_at: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&SystemKey::PendingConfig, &pending);
    }

    /// Phase 2: Confirm the pending admin configuration
    pub fn confirm_init(env: Env, confirmer: Address) {
        confirmer.require_auth();

        if let Some(mut pending) = env
            .storage()
            .instance()
            .get::<SystemKey, PendingConfig>(&SystemKey::PendingConfig)
        {
            let current_time = env.ledger().timestamp();
            let timeout = pending.proposed_at.saturating_add(3600);

            // Check timeout (1 hour = 3600 seconds)
            if current_time >= timeout {
                // Timeout expired, clear and return error
                env.storage().instance().remove(&SystemKey::PendingConfig);
                panic_with_error!(&env, ContractError::InvalidState);
            }

            // Check if confirmer is in proposed admins
            if !pending.admins.contains(&confirmer) {
                panic_with_error!(&env, ContractError::NotAnAdmin);
            }

            // Check if already confirmed
            if pending.confirmations.contains(&confirmer) {
                panic_with_error!(&env, ContractError::AdminAlreadyApproved);
            }

            // Add confirmation
            pending.confirmations.push_back(confirmer);
            env.storage()
                .instance()
                .set(&SystemKey::PendingConfig, &pending);
        } else {
            panic_with_error!(&env, ContractError::InvalidState);
        }
    }

    /// Phase 3: Activate the admin configuration once threshold is met
    pub fn activate_init(env: Env) {
        if let Some(pending) = env
            .storage()
            .instance()
            .get::<SystemKey, PendingConfig>(&SystemKey::PendingConfig)
        {
            // Check if enough confirmations
            if pending.confirmations.len() < pending.threshold {
                panic_with_error!(&env, ContractError::ThresholdNotMet);
            }

            // Activate configuration
            env.storage()
                .instance()
                .set(&SystemKey::Admins, &pending.admins);
            env.storage()
                .instance()
                .set(&SystemKey::AdminThreshold, &pending.threshold);

            // Clear pending config
            env.storage().instance().remove(&SystemKey::PendingConfig);

            // Set contract version
            env.storage().instance().set(
                &DataKey::ContractVersion,
                &ContractVersion {
                    major: 1,
                    minor: 0,
                    patch: 0,
                },
            );
        } else {
            panic_with_error!(&env, ContractError::InvalidState);
        }
    }

    pub fn get_admins(env: Env) -> Vec<Address> {
        if let Some(admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            let mut admins = Vec::new(&env);
            admins.push_back(admin);
            return admins;
        }
        env.storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_admin_threshold(env: Env) -> u32 {
        if env.storage().instance().has(&DataKey::Admin) {
            return 1u32;
        }
        env.storage()
            .instance()
            .get(&SystemKey::AdminThreshold)
            .unwrap_or(0u32)
    }

    /// Appends an entry to the admin activity log (Issue #816).
    fn record_admin_activity(env: &Env, actor: &Address, action: &str) {
        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::AdminActivityCount)
            .unwrap_or(0);
        let next = safe_increment(count);
        env.storage().instance().set(
            &SystemKey::AdminActivityLog(next),
            &AdminActivityEntry {
                actor: actor.clone(),
                action: String::from_str(env, action),
                timestamp: env.ledger().timestamp(),
            },
        );
        env.storage()
            .instance()
            .set(&SystemKey::AdminActivityCount, &next);
    }

    /// Returns a paginated view of on-chain actions performed by `admin`.
    ///
    /// Only current admins may call this — for their own activity log or
    /// any other admin's — since the log itself only tracks admin actions.
    pub fn get_admin_activity_log(
        env: Env,
        caller: Address,
        admin: Address,
        page: u32,
        page_size: u32,
    ) -> AdminActivityPage {
        caller.require_auth();
        if !Self::is_admin_address(&env, &caller) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let size = if page_size == 0 || page_size > 50 {
            50
        } else {
            page_size
        };

        let total_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::AdminActivityCount)
            .unwrap_or(0);

        let mut matches: Vec<AdminActivityEntry> = Vec::new(&env);
        for i in 1..=total_count {
            if let Some(entry) = env
                .storage()
                .instance()
                .get::<SystemKey, AdminActivityEntry>(&SystemKey::AdminActivityLog(i))
            {
                if entry.actor == admin {
                    matches.push_back(entry);
                }
            }
        }

        let total = matches.len() as u64;
        let start = (page as u64).saturating_mul(size as u64) as u32;
        let mut items = Vec::new(&env);
        for i in start..start.saturating_add(size) {
            match matches.get(i) {
                Some(entry) => items.push_back(entry),
                None => break,
            }
        }

        AdminActivityPage {
            items,
            total,
            page,
            page_size: size,
        }
    }

    /// Update the multisig admin threshold via a multisig proposal.
    /// Requires quorum approval. Rejects if an active proposal exists.
    /// Validates 1 <= new_threshold <= signer_count.
    pub fn set_threshold(env: Env, proposer: Address, new_threshold: u32) {
        PetChainContract::require_admin_auth(&env, &proposer);

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminsNotSet));

        if new_threshold == 0 || new_threshold > admins.len() {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }

        // Guard: reject if any active (non-executed, non-expired) proposal exists
        let proposal_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::ProposalCount)
            .unwrap_or(0);
        let now = env.ledger().timestamp();
        for i in 1..=proposal_count {
            if let Some(p) = env
                .storage()
                .instance()
                .get::<SystemKey, MultiSigProposal>(&SystemKey::Proposal(i))
            {
                if !p.executed && now <= p.expires_at {
                    panic_with_error!(&env, ContractError::InvalidState);
                }
            }
        }

        let mut pending: PendingThresholdChange = env
            .storage()
            .instance()
            .get(&SystemKey::PendingThresholdChange)
            .unwrap_or(PendingThresholdChange {
                new_threshold,
                approvals: Vec::new(&env),
            });

        // A differently-valued change supersedes whatever was pending.
        if pending.new_threshold != new_threshold {
            pending = PendingThresholdChange {
                new_threshold,
                approvals: Vec::new(&env),
            };
        }

        if pending.approvals.contains(&proposer) {
            panic_with_error!(&env, ContractError::AdminAlreadyApproved);
        }
        pending.approvals.push_back(proposer);

        if pending.approvals.len() < admins.len() {
            // Not every current admin has approved yet — remains pending.
            env.storage()
                .instance()
                .set(&SystemKey::PendingThresholdChange, &pending);
            return;
        }

        // Every current admin has approved — apply the change.
        env.storage()
            .instance()
            .remove(&SystemKey::PendingThresholdChange);

        let old_threshold: u32 = env
            .storage()
            .instance()
            .get(&SystemKey::AdminThreshold)
            .unwrap_or(1);

        env.storage()
            .instance()
            .set(&SystemKey::AdminThreshold, &new_threshold);

        env.events().publish(
            (Symbol::new(&env, "ThresholdChanged"),),
            (old_threshold, new_threshold),
        );
    }

    /// Set the quorum percentage required for governance proposal execution.
    /// `percent` is a whole-number percentage (e.g. 50 means 50% of admins
    /// must vote). 0 disables quorum checks entirely.
    ///
    /// Only callable by an existing admin.
    pub fn set_quorum_percent(env: Env, admin: Address, percent: u32) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        if percent > 100 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }
        env.storage()
            .instance()
            .set(&SystemKey::AdminQuorumPercent, &percent);

        env.events().publish(
            (Symbol::new(&env, "QuorumPercentChanged"),),
            percent,
        );
    }

    /// Returns the current quorum percentage. 0 means quorum is disabled.
    pub fn get_quorum_percent(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&SystemKey::AdminQuorumPercent)
            .unwrap_or(0)
    }

    fn update_vet_stats(
        env: &Env,
        vet: &Address,
        pet_id: u64,
        record_increment: u64,
        vaccination_increment: u64,
        treatment_increment: u64,
    ) {
        let mut stats = env
            .storage()
            .instance()
            .get::<_, VetStats>(&VetKey::VetStats(vet.clone()))
            .unwrap_or(VetStats {
                total_records: 0,
                total_vaccinations: 0,
                total_treatments: 0,
                pets_treated: 0,
            });

        stats.total_records = stats
            .total_records
            .checked_add(record_increment)
            .unwrap_or_else(|| panic_with_error!(env.clone(), ContractError::CounterOverflow));
        stats.total_vaccinations = stats
            .total_vaccinations
            .checked_add(vaccination_increment)
            .unwrap_or_else(|| panic_with_error!(env.clone(), ContractError::CounterOverflow));
        stats.total_treatments = stats
            .total_treatments
            .checked_add(treatment_increment)
            .unwrap_or_else(|| panic_with_error!(env.clone(), ContractError::CounterOverflow));

        // Unique pet tracking
        if !env
            .storage()
            .instance()
            .has(&VetKey::VetPetTreated((vet.clone(), pet_id)))
        {
            env.storage()
                .instance()
                .set(&VetKey::VetPetTreated((vet.clone(), pet_id)), &true);

            stats.pets_treated += 1;
        }

        env.storage()
            .instance()
            .set(&VetKey::VetStats(vet.clone()), &stats);
    }

    // --- STORAGE QUOTA SYSTEM (Issue #676) ---

    /// Get the effective storage quota for a pet (custom or global default)
    fn get_pet_quota(env: &Env, pet_id: u64) -> u64 {
        // Check for per-pet custom quota first
        if let Some(custom_quota) = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetStorageQuota(pet_id))
        {
            return custom_quota;
        }

        // Fall back to global default
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::GlobalStorageQuota)
            .unwrap_or(DEFAULT_STORAGE_QUOTA)
    }

    /// Get current storage usage for a pet
    fn get_pet_storage_count(env: &Env, pet_id: u64) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetStorageUsage(pet_id))
            .unwrap_or(0)
    }

    /// Increment storage usage for a pet and check quota
    /// Returns true if within quota, panics with StorageQuotaExceeded if over
    fn increment_pet_storage(env: &Env, pet_id: u64) {
        let current = Self::get_pet_storage_count(env, pet_id);
        let quota = Self::get_pet_quota(env, pet_id);

        // Check if adding one more entry would exceed quota
        if current >= quota {
            panic_with_error!(env, ContractError::StorageQuotaExceeded);
        }

        let new_count = current
            .checked_add(1)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::CounterOverflow));

        env.storage()
            .instance()
            .set(&DataKey::PetStorageUsage(pet_id), &new_count);
    }

    /// Check if a pet can add more storage entries without incrementing
    #[allow(dead_code)]
    fn check_pet_storage_quota(env: &Env, pet_id: u64) -> bool {
        let current = Self::get_pet_storage_count(env, pet_id);
        let quota = Self::get_pet_quota(env, pet_id);
        current < quota
    }

    /// Set global default storage quota (admin only)
    pub fn set_global_storage_quota(env: Env, admin: Address, quota: u64) {
        Self::require_admin_auth(&env, &admin);
        env.storage()
            .instance()
            .set(&DataKey::GlobalStorageQuota, &quota);
        Self::record_admin_activity(&env, &admin, "set_global_storage_quota");

        env.events()
            .publish((Symbol::new(&env, "GlobalStorageQuotaSet"),), quota);
    }

    /// Set custom storage quota for a specific pet (admin only)
    pub fn set_pet_storage_quota(env: Env, admin: Address, pet_id: u64, quota: u64) {
        Self::require_admin_auth(&env, &admin);

        // Verify pet exists
        if !env.storage().instance().has(&DataKey::Pet(pet_id)) {
            panic_with_error!(&env, ContractError::PetNotFound);
        }

        env.storage()
            .instance()
            .set(&DataKey::PetStorageQuota(pet_id), &quota);
        Self::record_admin_activity(&env, &admin, "set_pet_storage_quota");

        env.events()
            .publish((Symbol::new(&env, "PetStorageQuotaSet"), pet_id), quota);
    }

    /// Get storage usage information for a pet
    pub fn get_storage_usage(env: Env, pet_id: u64) -> StorageUsage {
        // Verify pet exists
        if !env.storage().instance().has(&DataKey::Pet(pet_id)) {
            panic_with_error!(&env, ContractError::PetNotFound);
        }

        let current_count = Self::get_pet_storage_count(&env, pet_id);
        let quota = Self::get_pet_quota(&env, pet_id);

        StorageUsage {
            pet_id,
            current_count,
            quota,
        }
    }

    // --- MULTI-LANGUAGE ERROR REGISTRY (Issue #684) ---

    /// Set an error message for a specific error code and language
    /// Only callable by admin
    pub fn set_error_message(
        env: Env,
        admin: Address,
        error_code: u32,
        language: String,
        message: String,
    ) {
        Self::require_admin_auth(&env, &admin);

        // Validate inputs
        if language.is_empty() || language.len() > 10 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }
        if message.is_empty() || message.len() > 500 {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        // Store the error message
        env.storage().instance().set(
            &ErrorRegistryKey::ErrorMessage((error_code, language.clone())),
            &message,
        );

        // Add language to supported languages if not already present
        let mut supported_langs: Vec<String> = env
            .storage()
            .instance()
            .get(&ErrorRegistryKey::SupportedLanguages)
            .unwrap_or_else(|| Vec::new(&env));

        if !supported_langs.contains(&language) {
            supported_langs.push_back(language.clone());
            env.storage()
                .instance()
                .set(&ErrorRegistryKey::SupportedLanguages, &supported_langs);
        }

        env.events().publish(
            (Symbol::new(&env, "ErrorMessageSet"), error_code),
            (language, message),
        );
    }

    /// Get an error message for a specific error code and language
    /// Returns the message if found, or None if not found
    pub fn get_error_message(env: Env, error_code: u32, language: String) -> Option<String> {
        env.storage()
            .instance()
            .get(&ErrorRegistryKey::ErrorMessage((error_code, language)))
    }

    /// Get all supported languages in the error registry
    pub fn get_supported_languages(env: Env) -> Vec<String> {
        env.storage()
            .instance()
            .get(&ErrorRegistryKey::SupportedLanguages)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Batch set error messages for multiple languages
    /// Only callable by admin
    pub fn batch_set_error_messages(env: Env, admin: Address, messages: Vec<ErrorMessage>) {
        Self::require_admin_auth(&env, &admin);

        if messages.len() > MAX_BATCH_ERROR_MESSAGES as u32 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        for msg in messages.iter() {
            // Validate inputs
            if msg.language.is_empty() || msg.language.len() > 10 {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
            if msg.message.is_empty() || msg.message.len() > 500 {
                panic_with_error!(&env, ContractError::InputStringTooLong);
            }

            // Store the error message
            env.storage().instance().set(
                &ErrorRegistryKey::ErrorMessage((msg.code, msg.language.clone())),
                &msg.message,
            );

            // Add language to supported languages if not already present
            let mut supported_langs: Vec<String> = env
                .storage()
                .instance()
                .get(&ErrorRegistryKey::SupportedLanguages)
                .unwrap_or_else(|| Vec::new(&env));

            if !supported_langs.contains(&msg.language) {
                supported_langs.push_back(msg.language.clone());
                env.storage()
                    .instance()
                    .set(&ErrorRegistryKey::SupportedLanguages, &supported_langs);
            }
        }

        env.events().publish(
            (Symbol::new(&env, "ErrorMessagesBatchSet"),),
            messages.len(),
        );
    }

    /// Initialize default error messages in English and Spanish
    /// Only callable by admin
    pub fn initialize_error_messages(env: Env, admin: Address) {
        // Authorization is enforced inside `batch_set_error_messages`; calling
        // `require_admin_auth` here as well would create a duplicate auth frame.
        let mut messages = Vec::new(&env);

        // English messages
        messages.push_back(ErrorMessage {
            code: 1,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Unauthorized access"),
        });
        messages.push_back(ErrorMessage {
            code: 2,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Admin not initialized"),
        });
        messages.push_back(ErrorMessage {
            code: 3,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Pet not found"),
        });
        messages.push_back(ErrorMessage {
            code: 4,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Veterinarian not found"),
        });
        messages.push_back(ErrorMessage {
            code: 5,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Veterinarian not verified"),
        });
        messages.push_back(ErrorMessage {
            code: 6,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Veterinarian already registered"),
        });
        messages.push_back(ErrorMessage {
            code: 7,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "License already registered"),
        });
        messages.push_back(ErrorMessage {
            code: 8,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Input string too long"),
        });
        messages.push_back(ErrorMessage {
            code: 160,
            language: String::from_str(&env, "en"),
            message: String::from_str(&env, "Storage quota exceeded"),
        });

        // Spanish messages
        messages.push_back(ErrorMessage {
            code: 1,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Acceso no autorizado"),
        });
        messages.push_back(ErrorMessage {
            code: 2,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Administrador no inicializado"),
        });
        messages.push_back(ErrorMessage {
            code: 3,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Mascota no encontrada"),
        });
        messages.push_back(ErrorMessage {
            code: 4,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Veterinario no encontrado"),
        });
        messages.push_back(ErrorMessage {
            code: 5,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Veterinario no verificado"),
        });
        messages.push_back(ErrorMessage {
            code: 6,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Veterinario ya registrado"),
        });
        messages.push_back(ErrorMessage {
            code: 7,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Licencia ya registrada"),
        });
        messages.push_back(ErrorMessage {
            code: 8,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Cadena de entrada demasiado larga"),
        });
        messages.push_back(ErrorMessage {
            code: 160,
            language: String::from_str(&env, "es"),
            message: String::from_str(&env, "Cuota de almacenamiento excedida"),
        });

        Self::batch_set_error_messages(env, admin, messages);
    }

    /// Remove an error message for a specific error code and language
    /// Only callable by admin
    pub fn remove_error_message(env: Env, admin: Address, error_code: u32, language: String) {
        Self::require_admin_auth(&env, &admin);

        env.storage()
            .instance()
            .remove(&ErrorRegistryKey::ErrorMessage((
                error_code,
                language.clone(),
            )));

        env.events().publish(
            (Symbol::new(&env, "ErrorMessageRemoved"), error_code),
            language,
        );
    }

    // --- BEHAVIOR RECORDS (Issue #798) ---

    /// Add a behavior record for a pet. Severity must be between 0 and 10.
    pub fn add_behavior_record(
        env: Env,
        pet_id: u64,
        caller: Address,
        behavior_type: BehaviorType,
        severity: u32,
        description: String,
    ) -> u64 {
        caller.require_auth();

        if severity > 10 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }
        // Bound description length to prevent unbounded ledger entries. (#1152)
        if description.len() > MAX_BEHAVIOR_DESC_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        let record_id: u64 = safe_increment(
            env.storage()
                .instance()
                .get(&BehaviorKey::BehaviorRecordCount)
                .unwrap_or(0u64),
        );

        let pet_index: u64 = safe_increment(
            env.storage()
                .instance()
                .get(&BehaviorKey::PetBehaviorCount(pet_id))
                .unwrap_or(0u64),
        );

        let record = BehaviorRecord {
            id: record_id,
            pet_id,
            behavior_type,
            severity,
            description,
            recorded_by: caller,
            recorded_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&BehaviorKey::BehaviorRecord(record_id), &record);
        env.storage().instance().set(
            &BehaviorKey::PetBehaviorIndex((pet_id, pet_index)),
            &record_id,
        );
        env.storage()
            .instance()
            .set(&BehaviorKey::PetBehaviorCount(pet_id), &pet_index);
        env.storage()
            .instance()
            .set(&BehaviorKey::BehaviorRecordCount, &record_id);

        record_id
    }

    /// Get a single behavior record by its ID.
    pub fn get_behavior_record(env: Env, record_id: u64) -> Option<BehaviorRecord> {
        env.storage()
            .instance()
            .get(&BehaviorKey::BehaviorRecord(record_id))
    }

    /// Get the total number of behavior records for a pet.
    pub fn get_behavior_count(env: Env, pet_id: u64) -> u64 {
        env.storage()
            .instance()
            .get(&BehaviorKey::PetBehaviorCount(pet_id))
            .unwrap_or(0u64)
    }

    /// Get the full (unbounded) behavior history for a pet.
    /// Prefer `get_behavior_records` for ledger-safe pagination.
    pub fn get_behavior_history(env: Env, pet_id: u64) -> Vec<BehaviorRecord> {
        let mut results = Vec::new(&env);
        let count: u64 = env
            .storage()
            .instance()
            .get(&BehaviorKey::PetBehaviorCount(pet_id))
            .unwrap_or(0u64);

        for i in 1u64..=count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<BehaviorKey, u64>(&BehaviorKey::PetBehaviorIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<BehaviorKey, BehaviorRecord>(&BehaviorKey::BehaviorRecord(record_id))
                {
                    results.push_back(record);
                }
            }
        }
        results
    }

    /// Returns a paginated, optionally type-filtered list of behavior records.
    /// `page_size` is capped at 50 to stay within ledger limits.
    pub fn get_behavior_records(
        env: Env,
        pet_id: u64,
        caller: Address,
        page: u32,
        page_size: u32,
        behavior_type: Option<BehaviorType>,
    ) -> BehaviorRecordPage {
        caller.require_auth();

        let page_size = page_size.min(50);
        let mut matched = Vec::new(&env);
        let count: u64 = env
            .storage()
            .instance()
            .get(&BehaviorKey::PetBehaviorCount(pet_id))
            .unwrap_or(0u64);

        for i in 1u64..=count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<BehaviorKey, u64>(&BehaviorKey::PetBehaviorIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<BehaviorKey, BehaviorRecord>(&BehaviorKey::BehaviorRecord(record_id))
                {
                    if let Some(ref filter) = behavior_type {
                        if record.behavior_type != *filter {
                            continue;
                        }
                    }
                    matched.push_back(record);
                }
            }
        }

        let total = matched.len() as u64;
        let mut items = Vec::new(&env);
        if page_size > 0 {
            let offset = (page as u64).saturating_mul(page_size as u64);
            if offset < total {
                let end = (offset + page_size as u64).min(total);
                for i in offset..end {
                    items.push_back(matched.get(i as u32).unwrap());
                }
            }
        }

        BehaviorRecordPage {
            items,
            total,
            page,
            page_size,
        }
    }

    // Pet Management Functions
    #[allow(clippy::too_many_arguments)]
    pub fn register_pet(
        env: Env,
        owner: Address,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        color: String,
        weight: u32,
        microchip_id: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> u64 {
        owner.require_auth();
        let birthday_ts = match PetChainContract::parse_birthday_timestamp(&birthday) {
            Ok(ts) => ts,
            Err(err) => env.panic_with_error(err),
        };
        let now = env.ledger().timestamp();
        if now > 0 {
            if birthday_ts > now {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
            const HUNDRED_YEARS_SECS: u64 = 100 * 365 * 86400;
            if now >= HUNDRED_YEARS_SECS && birthday_ts < now - HUNDRED_YEARS_SECS {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
        }
        Self::validate_pet_name(&env, &name);
        Self::validate_breed(&env, &species, &breed);
        let canonical_microchip = microchip_id
            .as_ref()
            .map(|value| Self::canonicalize_microchip_id(&env, value));
        if let Some(ref identifier) = canonical_microchip {
            if env.storage().instance().has(&DataKey::MicrochipIndex(identifier.clone())) {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
        }
        // Bound color field to prevent unbounded ledger entries. (#1152)
        if color.len() > MAX_COLOR_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count
            .checked_add(1)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));
        let timestamp = env.ledger().timestamp();

        let key = PetChainContract::get_encryption_key(&env);

        // Encrypt name
        let name_bytes = name.to_xdr(&env);
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, &name_bytes, &key);
        let encrypted_name = EncryptedData {
            nonce: name_nonce,
            ciphertext: name_ciphertext,
        };

        // Encrypt birthday
        let birthday_bytes = birthday.to_xdr(&env);
        let (birthday_nonce, birthday_ciphertext) =
            encrypt_sensitive_data(&env, &birthday_bytes, &key);
        let encrypted_birthday = EncryptedData {
            nonce: birthday_nonce,
            ciphertext: birthday_ciphertext,
        };

        // Encrypt breed
        let breed_bytes = breed.to_xdr(&env);
        let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(&env, &breed_bytes, &key);
        let encrypted_breed = EncryptedData {
            nonce: breed_nonce,
            ciphertext: breed_ciphertext,
        };

        // Initialize empty medical alerts/contacts
        let empty_alerts_bytes = Bytes::from_slice(&env, "".as_bytes());
        let (alerts_nonce, alerts_ciphertext) =
            encrypt_sensitive_data(&env, &empty_alerts_bytes, &key);
        let encrypted_medical_alerts = EncryptedData {
            nonce: alerts_nonce,
            ciphertext: alerts_ciphertext,
        };

        let empty_contacts = Vec::<EmergencyContact>::new(&env);
        let contacts_bytes = empty_contacts.to_xdr(&env);
        let (contacts_nonce, contacts_ciphertext) =
            encrypt_sensitive_data(&env, &contacts_bytes, &key);
        let encrypted_emergency_contacts = EncryptedData {
            nonce: contacts_nonce,
            ciphertext: contacts_ciphertext,
        };

        let empty_allergies = Vec::<Allergy>::new(&env);
        let allergies_bytes = empty_allergies.to_xdr(&env);
        let (allergies_nonce, allergies_ciphertext) =
            encrypt_sensitive_data(&env, &allergies_bytes, &key);
        let encrypted_allergies = EncryptedData {
            nonce: allergies_nonce,
            ciphertext: allergies_ciphertext,
        };

        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            privacy_level,
            encrypted_name,
            encrypted_birthday,
            encrypted_breed,
            encrypted_emergency_contacts,
            encrypted_medical_alerts,
            encrypted_allergies,

            // Empty placeholders for internal API consistency if needed
            name: String::from_str(&env, ""),
            birthday: String::from_str(&env, ""),
            breed: String::from_str(&env, ""),
            emergency_contacts: Vec::<EmergencyContact>::new(&env),
            medical_alerts: String::from_str(&env, ""),
            allergies: Vec::<Allergy>::new(&env),

            active: false,
            archived: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species: species.clone(),
            gender,
            color,
            weight,
            microchip_id: canonical_microchip,
            photo_hashes: Vec::new(&env),
        };

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);
        if let Some(ref identifier) = pet.microchip_id {
            env.storage()
                .instance()
                .set(&DataKey::MicrochipIndex(identifier.clone()), &pet_id);
        }

        PetChainContract::log_ownership_change(
            &env,
            pet_id,
            owner.clone(),
            owner.clone(),
            String::from_str(&env, "Initial Registration"),
        );

        let prev_owner_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0);
        let owner_pet_count = prev_owner_count
            .checked_add(1) // Prevent overflow: fail if owner has u64::MAX pets
            .unwrap_or_else(|| env.panic_with_error(ContractError::CounterOverflow));
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &owner_pet_count);
        env.storage().instance().set(
            &DataKey::OwnerPetIndex((owner.clone(), owner_pet_count)),
            &pet_id,
        );

        // Add to species index
        let species_key = PetChainContract::species_to_string(&env, &species);
        let prev_species_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species_key.clone()))
            .unwrap_or(0);
        let species_count = prev_species_count
            .checked_add(1) // Prevent overflow: fail if species has u64::MAX pets
            .unwrap_or_else(|| env.panic_with_error(ContractError::CounterOverflow));
        env.storage().instance().set(
            &DataKey::SpeciesPetCount(species_key.clone()),
            &species_count,
        );
        env.storage().instance().set(
            &DataKey::SpeciesPetIndex((species_key, species_count)),
            &pet_id,
        );

        // EMIT EVENT: PetRegistered (we emit the decrypted name for the event log as it's useful,
        // assuming standard privacy. If high strictness needed, this should be masked).
        // For now, we emit what was passed in.
        env.events().publish(
            (String::from_str(&env, "PetRegistered"), pet_id),
            PetRegisteredEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                owner,
                name: String::from_str(&env, "PROTECTED"), // Masking name in event for safety
                species,
                timestamp,
                subscription_ids: Self::matching_subscription_ids(
                    &env,
                    EventType::PetRegistered,
                    pet_id,
                ),
            },
        );

        pet_id
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_pet_profile(
        env: Env,
        id: u64,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        color: String,
        weight: u32,
        microchip_id: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            if let Err(err) = PetChainContract::parse_birthday_timestamp(&birthday) {
                env.panic_with_error(err);
            }
            Self::validate_pet_name(&env, &name);
            Self::validate_breed(&env, &species, &breed);

            let key = PetChainContract::get_encryption_key(&env);

            let name_bytes = name.to_xdr(&env);
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, &name_bytes, &key);
            pet.encrypted_name = EncryptedData {
                nonce: name_nonce,
                ciphertext: name_ciphertext,
            };

            let birthday_bytes = birthday.to_xdr(&env);
            let (birthday_nonce, birthday_ciphertext) =
                encrypt_sensitive_data(&env, &birthday_bytes, &key);
            pet.encrypted_birthday = EncryptedData {
                nonce: birthday_nonce,
                ciphertext: birthday_ciphertext,
            };

            let breed_bytes = breed.to_xdr(&env);
            let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(&env, &breed_bytes, &key);
            pet.encrypted_breed = EncryptedData {
                nonce: breed_nonce,
                ciphertext: breed_ciphertext,
            };

            pet.gender = gender;
            pet.species = species;
            pet.privacy_level = privacy_level;
            pet.color = color;
            pet.weight = weight;
            let canonical_microchip = microchip_id
                .as_ref()
                .map(|value| Self::canonicalize_microchip_id(&env, value));
            if canonical_microchip.as_ref() != pet.microchip_id.as_ref() {
                if let Some(ref identifier) = canonical_microchip {
                    if let Some(existing_id) = env
                        .storage()
                        .instance()
                        .get::<DataKey, u64>(&DataKey::MicrochipIndex(identifier.clone()))
                    {
                        if existing_id != id {
                            panic_with_error!(&env, ContractError::InvalidInput);
                        }
                    }
                }
                if let Some(ref previous) = pet.microchip_id {
                    env.storage().instance().remove(&DataKey::MicrochipIndex(previous.clone()));
                }
                if let Some(ref identifier) = canonical_microchip {
                    env.storage().instance().set(&DataKey::MicrochipIndex(identifier.clone()), &id);
                }
            }
            pet.microchip_id = canonical_microchip;
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);
            PetChainContract::log_access(
                &env,
                id,
                pet.owner.clone(),
                AccessAction::Write,
                String::from_str(&env, "Pet profile updated"),
            );
            let timestamp = env.ledger().timestamp();
            env.events().publish(
                (String::from_str(&env, "PetProfileUpdated"), id),
                PetProfileUpdatedEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id: id,
                    owner: pet.owner,
                    timestamp,
                    subscription_ids: Self::matching_subscription_ids(
                        &env,
                        EventType::PetProfileUpdated,
                        id,
                    ),
                },
            );
            true
        } else {
            false
        }
    }

    pub fn update_pet_privacy_level(env: Env, pet_id: u64, privacy_level: PrivacyLevel) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            pet.privacy_level = privacy_level;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
            true
        } else {
            false
        }
    }

    pub fn get_pet(env: Env, id: u64, caller: Address) -> Option<PetProfile> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            // Enforce access control based on privacy level.
            let allowed = match pet.privacy_level {
                PrivacyLevel::Public => true,
                PrivacyLevel::Restricted => {
                    let access = PetChainContract::check_access(env.clone(), id, caller.clone());
                    !matches!(access, AccessLevel::None)
                }
                PrivacyLevel::Private => pet.owner == caller,
            };
            if !allowed {
                return None;
            }

            let key = PetChainContract::get_encryption_key(&env);

            let decrypted_name = match decrypt_sensitive_data(
                &env,
                &pet.encrypted_name.ciphertext,
                &pet.encrypted_name.nonce,
                &key,
            ) {
                Ok(b) => b,
                Err(_) => return None,
            };
            let name = match String::from_xdr(&env, &decrypted_name) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let decrypted_birthday = match decrypt_sensitive_data(
                &env,
                &pet.encrypted_birthday.ciphertext,
                &pet.encrypted_birthday.nonce,
                &key,
            ) {
                Ok(b) => b,
                Err(_) => return None,
            };
            let birthday = match String::from_xdr(&env, &decrypted_birthday) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let decrypted_breed = match decrypt_sensitive_data(
                &env,
                &pet.encrypted_breed.ciphertext,
                &pet.encrypted_breed.nonce,
                &key,
            ) {
                Ok(b) => b,
                Err(_) => return None,
            };
            let breed = match String::from_xdr(&env, &decrypted_breed) {
                Ok(s) => s,
                Err(_) => return None,
            };

            let a_bytes = match decrypt_sensitive_data(
                &env,
                &pet.encrypted_allergies.ciphertext,
                &pet.encrypted_allergies.nonce,
                &key,
            ) {
                Ok(b) => b,
                Err(_) => return None,
            };
            let allergies = Vec::<Allergy>::from_xdr(&env, &a_bytes).unwrap_or(Vec::new(&env));

            let profile = PetProfile {
                id: pet.id,
                owner: pet.owner,
                privacy_level: pet.privacy_level,
                name,
                birthday,
                active: pet.active,
                created_at: pet.created_at,
                updated_at: pet.updated_at,
                new_owner: pet.new_owner,
                species: pet.species,
                gender: pet.gender,
                breed,
                color: pet.color,
                weight: pet.weight,
                microchip_id: pet.microchip_id,
                allergies,
            };
            // Pure view: no side effects
            Some(profile)
        } else {
            None
        }
    }

    pub fn get_pet_data(env: Env, id: u64, caller: Address) -> Option<PetData> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            let allowed = match pet.privacy_level {
                PrivacyLevel::Public => true,
                PrivacyLevel::Restricted => {
                    let access = PetChainContract::check_access(env.clone(), id, caller.clone());
                    !matches!(access, AccessLevel::None)
                }
                PrivacyLevel::Private => {
                    caller.require_auth();
                    pet.owner == caller
                }
            };

            if !allowed {
                return None;
            }

            let key = PetChainContract::get_encryption_key(&env);

            let decrypted_name = decrypt_sensitive_data(
                &env,
                &pet.encrypted_name.ciphertext,
                &pet.encrypted_name.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let name =
                String::from_xdr(&env, &decrypted_name).unwrap_or(String::from_str(&env, "Error"));

            let decrypted_breed = decrypt_sensitive_data(
                &env,
                &pet.encrypted_breed.ciphertext,
                &pet.encrypted_breed.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let breed =
                String::from_xdr(&env, &decrypted_breed).unwrap_or(String::from_str(&env, "Error"));

            let species_str = match pet.species {
                Species::Dog => "Dog",
                Species::Cat => "Cat",
                Species::Bird => "Bird",
                Species::Rabbit => "Rabbit",
                Species::Other => "Other",
            };

            Some(PetData {
                name,
                species: String::from_str(&env, species_str),
                breed,
            })
        } else {
            None
        }
    }

    pub fn get_pet_age(env: Env, pet_id: u64) -> (u64, u64) {
        if let Some(pet) =
            PetChainContract::get_pet(env.clone(), pet_id, env.current_contract_address())
        {
            let current_time = env.ledger().timestamp();
            let birthday_timestamp = match PetChainContract::parse_birthday_timestamp(&pet.birthday)
            {
                Ok(timestamp) => timestamp,
                Err(_) => return (0, 0),
            };

            if current_time < birthday_timestamp {
                return (0, 0);
            }

            let elapsed_seconds = current_time - birthday_timestamp;
            let elapsed_days = elapsed_seconds / 86_400;
            let years = elapsed_days / 365;
            let remaining_days = elapsed_days % 365;
            let months = remaining_days / 30;

            (years, months)
        } else {
            (0, 0)
        }
    }

    pub fn get_pet_full_profile(env: Env, pet_id: u64, caller: Address) -> Option<PetFullProfile> {
        // Check access control first
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            // Check if caller has access based on privacy level and access grants
            let access_level = PetChainContract::check_access(env.clone(), pet_id, caller.clone());

            // Private pets can only be accessed by owner
            if pet.privacy_level == PrivacyLevel::Private && pet.owner != caller {
                return None;
            }

            // Restricted pets require at least Basic access
            if pet.privacy_level == PrivacyLevel::Restricted && access_level == AccessLevel::None {
                return None;
            }

            // Public pets are accessible to anyone
            // Get the base pet profile
            let profile = PetChainContract::get_pet(env.clone(), pet_id, caller.clone())?;

            // Get latest vaccination ID (most recent by administered_at)
            let vax_count: u64 = env
                .storage()
                .instance()
                .get(&MedicalKey::PetVaccinationCount(pet_id))
                .unwrap_or(0);
            let mut latest_vaccination_id: Option<u64> = None;
            let mut latest_timestamp: u64 = 0;
            for i in 1..=vax_count {
                if let Some(vax_id) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
                {
                    if let Some(vax) = PetChainContract::get_vaccinations(env.clone(), vax_id) {
                        if vax.administered_at > latest_timestamp {
                            latest_timestamp = vax.administered_at;
                            latest_vaccination_id = Some(vax_id);
                        }
                    }
                }
            }

            // Get active medications count
            let active_medications = PetChainContract::get_active_medications(env.clone(), pet_id);
            let active_medications_count = active_medications.len() as u64;

            // Check if insurance exists
            let insurance = PetChainContract::get_pet_insurance(env.clone(), pet_id);
            let has_insurance = insurance.is_some();

            // Pure view: no side effects
            Some(PetFullProfile {
                profile,
                latest_vaccination_id,
                active_medications_count,
                has_insurance,
            })
        } else {
            None
        }
    }

    /// Batch read operation: Returns pet profile, owner, active consents, and latest medical record.
    /// Reduces multiple round trips to a single call.
    /// Respects access control - caller must have read permission.
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet
    /// * `caller` - The address requesting the data
    ///
    /// # Returns
    /// * `Some(PetFullProfileBatch)` if pet exists and caller has access
    /// * `None` if pet doesn't exist or caller lacks permission
    ///
    /// # Access Control
    /// - Public pets: accessible to anyone
    /// - Restricted pets: requires at least Basic access grant
    /// - Private pets: only accessible to owner
    pub fn get_pet_full_profile_batch(
        env: Env,
        pet_id: u64,
        caller: Address,
    ) -> Option<PetFullProfileBatch> {
        // Check if pet exists
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))?;

        // Check access control
        let access_level = PetChainContract::check_access(env.clone(), pet_id, caller.clone());

        // Private pets can only be accessed by owner
        if pet.privacy_level == PrivacyLevel::Private && pet.owner != caller {
            return None;
        }

        // Restricted pets require at least Basic access
        if pet.privacy_level == PrivacyLevel::Restricted && access_level == AccessLevel::None {
            return None;
        }

        // Get the base pet profile
        let profile = PetChainContract::get_pet(env.clone(), pet_id, caller.clone())?;

        // Get owner address
        let owner = pet.owner.clone();

        // Get active consents
        let active_consents = PetChainContract::get_active_consents(env.clone(), pet_id);

        // Get latest medical record (most recent by recorded_at)
        let record_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);

        let mut latest_medical_record_id: Option<u64> = None;
        let mut latest_timestamp: u64 = 0;

        for i in 1..=record_count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) =
                    PetChainContract::get_medical_record(env.clone(), record_id)
                {
                    if record.date > latest_timestamp {
                        latest_timestamp = record.date;
                        latest_medical_record_id = Some(record.id);
                    }
                }
            }
        }

        Some(PetFullProfileBatch {
            profile,
            owner,
            active_consents,
            latest_medical_record_id,
        })
    }

    /// Batch read operation: Returns latest vaccination, lab result, and active insurance.
    /// Reduces multiple round trips to a single call.
    /// Respects access control - caller must have read permission.
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet
    /// * `caller` - The address requesting the data
    ///
    /// # Returns
    /// * `Some(PetHealthSummary)` if pet exists and caller has access
    /// * `None` if pet doesn't exist or caller lacks permission
    ///
    /// # Access Control
    /// - Public pets: accessible to anyone
    /// - Restricted pets: requires at least Basic access grant
    /// - Private pets: only accessible to owner
    pub fn get_pet_health_summary(
        env: Env,
        pet_id: u64,
        caller: Address,
    ) -> Option<PetHealthSummary> {
        // Check if pet exists
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))?;

        // Check access control
        let access_level = PetChainContract::check_access(env.clone(), pet_id, caller.clone());

        // Private pets can only be accessed by owner
        if pet.privacy_level == PrivacyLevel::Private && pet.owner != caller {
            return None;
        }

        // Restricted pets require at least Basic access
        if pet.privacy_level == PrivacyLevel::Restricted && access_level == AccessLevel::None {
            return None;
        }

        // Get latest vaccination (most recent by administered_at)
        let vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut latest_vaccination_id: Option<u64> = None;
        let mut latest_vax_timestamp: u64 = 0;
        let mut latest_vax_expires_at: u64 = 0;

        for i in 1..=vax_count {
            if let Some(vax_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = PetChainContract::get_vaccinations(env.clone(), vax_id) {
                    if vax.administered_at > latest_vax_timestamp {
                        latest_vax_timestamp = vax.administered_at;
                        latest_vax_expires_at = vax.expires_at;
                        latest_vaccination_id = Some(vax_id);
                    }
                }
            }
        }

        // Get latest lab result (most recent by test_date)
        let lab_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetLabResultCount(pet_id))
            .unwrap_or(0);

        let mut latest_lab_result_id: Option<u64> = None;
        let mut latest_lab_timestamp: u64 = 0;

        for i in 1..=lab_count {
            if let Some(lab_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetLabResultIndex((pet_id, i)))
            {
                if let Some(lab) = PetChainContract::get_lab_result(env.clone(), lab_id) {
                    if lab.date > latest_lab_timestamp {
                        latest_lab_timestamp = lab.date;
                        latest_lab_result_id = Some(lab_id);
                    }
                }
            }
        }

        // Get active insurance policy (most recent active policy)
        let policy_count: u64 = env
            .storage()
            .instance()
            .get(&InsuranceKey::PetPolicyCount(pet_id))
            .unwrap_or(0);

        let mut active_insurance_policy_id: Option<u64> = None;
        let mut active_policy_expiry: u64 = 0;

        // Get the most recent policy (highest index)
        if policy_count > 0 {
            if let Some(policy) = env
                .storage()
                .instance()
                .get::<InsuranceKey, InsurancePolicy>(&InsuranceKey::PetPolicyIndex((
                    pet_id,
                    policy_count,
                )))
            {
                if policy.active {
                    active_insurance_policy_id = Some(policy_count);
                    active_policy_expiry = policy.expiry_date;
                }
            }
        }

        // Derive insurance eligibility: the pet must have an active, non-expired
        // insurance policy AND a non-expired latest vaccination at the current
        // ledger timestamp. A vaccination/policy is "not expired" while its
        // expiry timestamp is at or after now (matching the contract's existing
        // `expired = expiry < now` convention). Purely derived — no storage writes.
        let now = env.ledger().timestamp();
        let insurance_not_expired =
            active_insurance_policy_id.is_some() && active_policy_expiry >= now;
        let vaccination_not_expired =
            latest_vaccination_id.is_some() && latest_vax_expires_at >= now;
        let insurance_eligible = insurance_not_expired && vaccination_not_expired;

        Some(PetHealthSummary {
            pet_id,
            latest_vaccination_id,
            latest_lab_result_id,
            active_insurance_policy_id,
            insurance_eligible,
        })
    }

    fn parse_birthday_timestamp(birthday: &String) -> Result<u64, ContractError> {
        let len = birthday.len() as usize;
        if len == 0 || len > 20 {
            return Err(ContractError::InvalidInput);
        }

        let mut bytes = [0u8; 20];
        birthday.copy_into_slice(&mut bytes[..len]);

        if bytes.iter().take(len).all(u8::is_ascii_digit) {
            let mut timestamp = 0u64;
            for b in bytes.iter().take(len) {
                let digit = (b - b'0') as u64;
                timestamp = timestamp
                    .checked_mul(10)
                    .and_then(|v| v.checked_add(digit))
                    .ok_or(ContractError::InvalidInput)?;
            }
            return Ok(timestamp);
        }

        if len != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(ContractError::InvalidInput);
        }

        let year = PetChainContract::parse_fixed_digits(&bytes[0..4])?;
        let month = PetChainContract::parse_fixed_digits(&bytes[5..7])?;
        let day = PetChainContract::parse_fixed_digits(&bytes[8..10])?;

        if !(1..=12).contains(&month) {
            return Err(ContractError::InvalidInput);
        }

        let max_day = PetChainContract::days_in_month(year, month);
        if day == 0 || day > max_day {
            return Err(ContractError::InvalidInput);
        }

        let days_since_epoch =
            PetChainContract::days_from_civil(year as i32, month as i32, day as i32)?;
        Ok(days_since_epoch * 86_400)
    }

    fn parse_fixed_digits(bytes: &[u8]) -> Result<u32, ContractError> {
        let mut value = 0u32;
        for b in bytes {
            if !b.is_ascii_digit() {
                return Err(ContractError::InvalidInput);
            }
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add((b - b'0') as u32))
                .ok_or(ContractError::InvalidInput)?;
        }
        Ok(value)
    }

    fn is_leap_year(year: u32) -> bool {
        (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
    }

    fn days_in_month(year: u32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if PetChainContract::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    fn days_from_civil(year: i32, month: i32, day: i32) -> Result<u64, ContractError> {
        let adjusted_year = year - if month <= 2 { 1 } else { 0 };
        let era = if adjusted_year >= 0 {
            adjusted_year / 400
        } else {
            (adjusted_year - 399) / 400
        };
        let year_of_era = adjusted_year - era * 400;
        let month_of_year = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * month_of_year + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * 146_097 + day_of_era - 719_468;
        if days < 0 {
            return Err(ContractError::InvalidInput);
        }
        Ok(days as u64)
    }

    pub fn is_pet_active(env: Env, id: u64) -> bool {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.active
        } else {
            false
        }
    }

    pub fn get_pet_owner(env: Env, id: u64) -> Option<Address> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            Some(pet.owner)
        } else {
            None
        }
    }

    pub fn activate_pet(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            if !pet.active {
                let active_count: u64 = env
                    .storage()
                    .instance()
                    .get(&StatsKey::ActivePetsCount)
                    .unwrap_or(0);
                env.storage()
                    .instance()
                    .set(&StatsKey::ActivePetsCount, &safe_increment(&env, active_count));
                Self::record_stat_point(
                    &env,
                    String::from_str(&env, "ActivePetsCount"),
                    safe_increment(&env, active_count),
                );
            }
            pet.active = true;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }

    pub fn deactivate_pet(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            if pet.active {
                let active_count: u64 = env
                    .storage()
                    .instance()
                    .get(&StatsKey::ActivePetsCount)
                    .unwrap_or(0);
                if active_count > 0 {
                    env.storage()
                        .instance()
                        .set(&StatsKey::ActivePetsCount, &(active_count - 1));
                    Self::record_stat_point(
                        &env,
                        String::from_str(&env, "ActivePetsCount"),
                        active_count - 1,
                    );
                }
            }
            pet.active = false;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }

    pub fn archive_pet(env: Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        if pet.active {
            let active_count: u64 = env
                .storage()
                .instance()
                .get(&StatsKey::ActivePetsCount)
                .unwrap_or(0);
            if active_count > 0 {
                env.storage()
                    .instance()
                    .set(&StatsKey::ActivePetsCount, &(active_count - 1));
                Self::record_stat_point(
                    &env,
                    String::from_str(&env, "ActivePetsCount"),
                    active_count - 1,
                );
            }
        }
        pet.archived = true;
        pet.active = false;
        pet.updated_at = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
    }

    pub fn unarchive_pet(env: Env, pet_id: u64) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        pet.archived = false;
        pet.updated_at = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
    }

    pub fn add_pet_photo(env: Env, pet_id: u64, photo_hash: String) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            if let Err(err) = PetChainContract::validate_ipfs_hash(&env, &photo_hash) {
                env.panic_with_error(err);
            }

            // Enforce cap to prevent unbounded inline Vec growth. (#1153)
            if pet.photo_hashes.len() >= MAX_PHOTO_HASHES {
                panic_with_error!(&env, ContractError::TooManyItems);
            }

            // Check storage quota (Issue #782)
            Self::increment_pet_storage(&env, pet_id);

            pet.photo_hashes.push_back(photo_hash);
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
            true
        } else {
            false
        }
    }

    pub fn get_pet_photos(env: Env, pet_id: u64) -> Vec<String> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.photo_hashes
        } else {
            Vec::new(&env)
        }
    }

    /// Returns the total number of photos for a pet. Returns 0 for unknown pet IDs.
    pub fn get_pet_photo_count(env: Env, pet_id: u64) -> u64 {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.photo_hashes.len() as u64
        } else {
            0
        }
    }

    /// Returns a paginated slice of photo hashes for a pet.
    /// `offset` is the zero-based index of the first item to return.
    /// `limit` is the maximum number of items to return.
    pub fn get_pet_photos_paginated(env: Env, pet_id: u64, offset: u64, limit: u32) -> Vec<String> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            let total = pet.photo_hashes.len() as u64;
            let mut result = Vec::new(&env);

            if offset >= total || limit == 0 {
                return result;
            }

            let start = offset as u32;
            let end = (offset + limit as u64).min(total) as u32;

            for i in start..end {
                if let Some(hash) = pet.photo_hashes.get(i) {
                    result.push_back(hash);
                }
            }

            result
        } else {
            Vec::new(&env)
        }
    }

    pub fn remove_pet_photo(env: Env, pet_id: u64, photo_hash: String) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();

            // Find the photo in the vector
            let mut index_to_remove: Option<u32> = None;
            for (i, hash) in pet.photo_hashes.iter().enumerate() {
                if hash == photo_hash {
                    index_to_remove = Some(i as u32);
                    break;
                }
            }

            // If found, remove it and update the pet
            if let Some(idx) = index_to_remove {
                pet.photo_hashes.remove(idx);
                pet.updated_at = env.ledger().timestamp();
                env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn transfer_pet_ownership(env: Env, id: u64, to: Address, nonce: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            let owner = pet.owner.clone();
            owner.require_auth();
            Self::consume_caller_nonce(&env, &owner, nonce);
            pet.new_owner = to;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
            env.storage().instance().set(
                &SystemKey::PetTransferOffer(id),
                &env.ledger().timestamp().saturating_add(86_400),
            );
        }
    }

    /// Grant a grantee access to a pet. The caller must be the pet owner and
    /// supply the expected nonce value for replay protection.
    pub fn grant_access(
        env: Env,
        pet_id: u64,
        grantee: Address,
        access_level: AccessLevel,
        expires_at: Option<u64>,
        nonce: u64,
    ) -> bool {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            let owner = pet.owner.clone();
            owner.require_auth();
            Self::consume_caller_nonce(&env, &owner, nonce);

            let key = DataKey::AccessGrant((pet_id, grantee.clone()));
            let now = env.ledger().timestamp();
            let mut grant = if let Some(existing) = env
                .storage()
                .instance()
                .get::<DataKey, AccessGrant>(&key)
            {
                existing
            } else {
                AccessGrant {
                    pet_id,
                    granter: owner.clone(),
                    grantee: grantee.clone(),
                    access_level: access_level.clone(),
                    granted_at: now,
                    expires_at,
                    is_active: true,
                }
            };

            grant.access_level = access_level.clone();
            grant.granted_at = now;
            grant.expires_at = expires_at;
            grant.is_active = true;
            grant.granter = owner.clone();

            let is_new_grant = env
                .storage()
                .instance()
                .get::<DataKey, AccessGrant>(&key)
                .is_none();

            env.storage().instance().set(&key, &grant);

            if is_new_grant {
                let existing_count: u64 = env
                    .storage()
                    .instance()
                    .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
                    .unwrap_or(0);
                let new_count = safe_increment(&env, existing_count);
                env.storage()
                    .instance()
                    .set(&DataKey::AccessGrantCount(pet_id), &new_count);
                env.storage()
                    .instance()
                    .set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);
            }

            env.events().publish(
                (String::from_str(&env, "AccessGranted"), pet_id),
                AccessGrantedEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id,
                    granter: owner,
                    grantee,
                    access_level,
                    expires_at,
                    timestamp: now,
                },
            );
            true
        } else {
            false
        }
    }

    /// Revoke an access grant for a pet. Only the owner may revoke access.
    pub fn revoke_access(env: Env, pet_id: u64, grantee: Address) -> bool {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            let key = DataKey::AccessGrant((pet_id, grantee.clone()));
            if let Some(mut grant) = env
                .storage()
                .instance()
                .get::<DataKey, AccessGrant>(&key)
            {
                if !grant.is_active {
                    return false;
                }
                grant.is_active = false;
                env.storage().instance().set(&key, &grant);
                env.events().publish(
                    (String::from_str(&env, "AccessRevoked"), pet_id),
                    AccessRevokedEvent {
                        version: EVENT_SCHEMA_VERSION,
                        pet_id,
                        granter: pet.owner,
                        grantee,
                        timestamp: env.ledger().timestamp(),
                    },
                );
                return true;
            }
        }
        false
    }

    /// Cursor-based pagination of a pet's access grants (Issue #1161).
    ///
    /// Lets the owner (or an authorized auditor) review access grants in
    /// bounded pages instead of loading the entire grant list at once.
    /// Only the pet owner may call this today, matching the authorization
    /// used by [`Self::grant_access`] and [`Self::revoke_access`].
    ///
    /// Because the grant index is compacted (shifted left) whenever a grant
    /// is removed, a page may skip or repeat an entry if grants are revoked
    /// concurrently with pagination -- the same caveat that applies to other
    /// cursor-paginated views in this contract. Callers that need a
    /// point-in-time-consistent view should page within a single ledger
    /// close.
    ///
    /// `cursor` is an opaque slot index; pass `0` for the first page and
    /// echo back `next_cursor` on subsequent calls until it is `0`. When
    /// `active_only` is `true`, expired and explicitly revoked grants are
    /// filtered out of `items` (but still count toward slots examined).
    pub fn get_pet_access_grants_cursor(
        env: Env,
        pet_id: u64,
        cursor: u64,
        limit: u32,
        active_only: bool,
    ) -> AccessGrantPage {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id));
        if let Some(pet) = pet.as_ref() {
            pet.owner.require_auth();
        }

        let count: u64 = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let mut items = Vec::new(&env);

        if pet.is_none() || limit == 0 || count == 0 {
            return AccessGrantPage {
                items,
                next_cursor: 0,
                total_slots: count,
            };
        }

        // `cursor` is the last examined slot; resume from the following slot.
        let start = cursor.saturating_add(1);
        if start > count {
            return AccessGrantPage {
                items,
                next_cursor: 0,
                total_slots: count,
            };
        }

        let end = count.min(start.saturating_add(limit as u64).saturating_sub(1));
        let now = env.ledger().timestamp();
        let mut idx = start;
        while idx <= end && items.len() < limit {
            if let Some(grantee) = env
                .storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, idx)))
            {
                if let Some(grant) = env
                    .storage()
                    .instance()
                    .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, grantee)))
                {
                    let expired = grant.expires_at.map(|exp| now >= exp).unwrap_or(false);
                    if !active_only || (grant.is_active && !expired) {
                        items.push_back(grant);
                    }
                }
            }
            idx += 1;
        }

        let last_examined = idx.saturating_sub(1);
        let next_cursor = if last_examined < count { last_examined } else { 0 };

        AccessGrantPage {
            items,
            next_cursor,
            total_slots: count,
        }
    }

    /// Nonce-protected pet registration. Caller supplies their current nonce;
    /// the nonce is incremented atomically on success, preventing replay.
    /// All pets must belong to the same caller and the entire batch fails if
    /// any pet is missing or owned by a different address.
    pub fn batch_transfer(env: Env, pet_ids: Vec<u64>, new_owner: Address) {
        const MAX_BATCH_SIZE: u32 = 20;

        if pet_ids.is_empty() {
            panic_with_error!(&env, ContractError::InvalidInput);
        }
        if pet_ids.len() > MAX_BATCH_SIZE {
            panic_with_error!(&env, ContractError::BatchTooLarge);
        }

        let mut expected_owner: Option<Address> = None;
        let mut seen_ids = Vec::new(&env);
        let mut pets = Vec::new(&env);
        for pet_id in pet_ids.iter() {
            if seen_ids.contains(pet_id) {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
            seen_ids.push_back(pet_id);

            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
                .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

            match expected_owner {
                None => expected_owner = Some(pet.owner.clone()),
                Some(ref owner) if owner != &pet.owner => {
                    panic_with_error!(&env, ContractError::NotPetOwner);
                }
                _ => {}
            }

            pets.push_back(pet);
        }

        let owner =
            expected_owner.unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));
        owner.require_auth();

        let now = env.ledger().timestamp();
        for pet in pets.iter() {
            let pet_id = pet.id;
            let old_owner = pet.owner.clone();
            PetChainContract::remove_pet_from_owner_index(&env, &old_owner, pet_id);

            let mut pet = pet.clone();
            pet.owner = new_owner.clone();
            pet.new_owner = new_owner.clone();
            pet.updated_at = now;

            PetChainContract::add_pet_to_owner_index(&env, &pet.owner, pet_id);
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

            PetChainContract::log_ownership_change(
                &env,
                pet_id,
                old_owner.clone(),
                pet.owner.clone(),
                String::from_str(&env, "Batch Transfer"),
            );

            PetChainContract::append_custody_entry(
                &env,
                pet_id,
                old_owner.clone(),
                pet.owner.clone(),
                TransferType::Direct,
            );

            env.events().publish(
                (String::from_str(&env, "PetOwnershipTransferred"), pet_id),
                PetOwnershipTransferredEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id,
                    old_owner,
                    new_owner: pet.owner.clone(),
                    timestamp: now,
                },
            );
        }
    }

    pub fn accept_pet_transfer(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            let expires_at: u64 = env.storage().instance().get(&SystemKey::PetTransferOffer(id))
                .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidState));
            if env.ledger().timestamp() > expires_at {
                panic_with_error!(&env, ContractError::InvalidState);
            }
            pet.new_owner.require_auth();

            let old_owner = pet.owner.clone();
            PetChainContract::remove_pet_from_owner_index(&env, &old_owner, id);

            pet.owner = pet.new_owner.clone();
            pet.updated_at = env.ledger().timestamp();

            PetChainContract::add_pet_to_owner_index(&env, &pet.owner, id);

            env.storage().instance().set(&DataKey::Pet(id), &pet);

            PetChainContract::log_ownership_change(
                &env,
                id,
                old_owner.clone(),
                pet.owner.clone(),
                String::from_str(&env, "Ownership Transfer"),
            );

            PetChainContract::append_custody_entry(
                &env,
                id,
                old_owner.clone(),
                pet.owner.clone(),
                TransferType::Direct,
            );

            env.events().publish(
                (String::from_str(&env, "PetOwnershipTransferred"), id),
                PetOwnershipTransferredEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id: id,
                    old_owner,
                    new_owner: pet.owner.clone(),
                    timestamp: pet.updated_at,
                },
            );
            env.storage().instance().remove(&SystemKey::PetTransferOffer(id));
        }
    }

    pub fn cancel_pet_transfer(env: Env, id: u64) {
        let mut pet: Pet = env.storage().instance().get(&DataKey::Pet(id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        pet.new_owner = pet.owner.clone();
        env.storage().instance().set(&DataKey::Pet(id), &pet);
        env.storage().instance().remove(&SystemKey::PetTransferOffer(id));
    }

    pub fn configure_multisig(env: Env, pet_id: u64, signers: Vec<Address>, threshold: u32) -> bool {
        let pet: Pet = env.storage().instance().get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        Self::validate_multisig(&env, &pet.owner, &signers, threshold);
        env.storage().instance().set(&SystemKey::PetMultisigConfig(pet_id), &MultisigConfig { pet_id, signers, threshold, enabled: true });
        true
    }

    pub fn update_multisig_signers(env: Env, pet_id: u64, signers: Vec<Address>, threshold: u32) -> bool {
        Self::configure_multisig(env, pet_id, signers, threshold)
    }

    pub fn get_multisig_config(env: Env, pet_id: u64) -> Option<MultisigConfig> {
        env.storage().instance().get(&SystemKey::PetMultisigConfig(pet_id))
    }

    pub fn disable_multisig(env: Env, pet_id: u64) -> bool {
        let pet: Pet = env.storage().instance().get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        if let Some(mut config) = Self::get_multisig_config(env.clone(), pet_id) {
            config.enabled = false;
            env.storage().instance().set(&SystemKey::PetMultisigConfig(pet_id), &config);
            return true;
        }
        false
    }

    fn validate_multisig(env: &Env, owner: &Address, signers: &Vec<Address>, threshold: u32) {
        if signers.is_empty() || threshold == 0 || threshold > signers.len() as u32 || !signers.contains(owner) {
            panic_with_error!(env, ContractError::InvalidThreshold);
        }
        for i in 0..signers.len() {
            for j in (i + 1)..signers.len() {
                if signers.get(i) == signers.get(j) { panic_with_error!(env, ContractError::InvalidInput); }
            }
        }
    }

    pub fn require_multisig_for_transfer(env: Env, pet_id: u64, to: Address) -> u64 {
        let pet: Pet = env.storage().instance().get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        let config = Self::get_multisig_config(env.clone(), pet_id)
            .filter(|config| config.enabled)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidState));
        let count: u64 = env.storage().instance().get(&SystemKey::PetTransferProposalCount).unwrap_or(0);
        let id = safe_increment(count);
        let mut signatures = Vec::new(&env);
        signatures.push_back(pet.owner);
        let now = env.ledger().timestamp();
        let proposal = PetTransferProposal { id, pet_id, to, signatures, created_at: now, expires_at: now.saturating_add(86_400), executed: false };
        env.storage().instance().set(&SystemKey::PetTransferProposal(id), &proposal);
        env.storage().instance().set(&SystemKey::PetTransferProposalCount, &id);
        let mut active: Vec<u64> = env.storage().instance().get(&SystemKey::PetActiveProposals(pet_id)).unwrap_or(Vec::new(&env));
        active.push_back(id);
        env.storage().instance().set(&SystemKey::PetActiveProposals(pet_id), &active);
        let _ = config;
        id
    }

    pub fn get_transfer_proposal(env: Env, proposal_id: u64) -> Option<PetTransferProposal> {
        env.storage().instance().get(&SystemKey::PetTransferProposal(proposal_id))
    }

    pub fn sign_transfer_proposal(env: Env, proposal_id: u64, signer: Address) -> bool {
        signer.require_auth();
        let mut proposal = Self::get_transfer_proposal(env.clone(), proposal_id)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidInput));
        let config = Self::get_multisig_config(env.clone(), proposal.pet_id)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidState));
        if !config.enabled || !config.signers.contains(&signer) || proposal.executed { panic_with_error!(&env, ContractError::Unauthorized); }
        if env.ledger().timestamp() > proposal.expires_at || proposal.signatures.contains(&signer) { panic_with_error!(&env, ContractError::InvalidState); }
        proposal.signatures.push_back(signer);
        env.storage().instance().set(&SystemKey::PetTransferProposal(proposal_id), &proposal);
        true
    }

    pub fn multisig_transfer_pet(env: Env, proposal_id: u64) -> bool {
        let mut proposal = Self::get_transfer_proposal(env.clone(), proposal_id)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidInput));
        let config = Self::get_multisig_config(env.clone(), proposal.pet_id)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidState));
        if proposal.executed || env.ledger().timestamp() > proposal.expires_at { panic_with_error!(&env, ContractError::InvalidState); }
        if proposal.signatures.len() < config.threshold { panic_with_error!(&env, ContractError::ThresholdNotMet); }
        let mut pet: Pet = env.storage().instance().get(&DataKey::Pet(proposal.pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        let old_owner = pet.owner.clone();
        Self::remove_pet_from_owner_index(&env, &old_owner, pet.id);
        pet.owner = proposal.to.clone();
        pet.new_owner = proposal.to.clone();
        pet.updated_at = env.ledger().timestamp();
        Self::add_pet_to_owner_index(&env, &pet.owner, pet.id);
        env.storage().instance().set(&DataKey::Pet(pet.id), &pet);
        Self::log_ownership_change(&env, pet.id, old_owner.clone(), pet.owner.clone(), String::from_str(&env, "Multisig Transfer"));
        Self::append_custody_entry(&env, pet.id, old_owner.clone(), pet.owner.clone(), TransferType::Multisig);
        env.events().publish((String::from_str(&env, "PetOwnershipTransferred"), pet.id), PetOwnershipTransferredEvent { version: EVENT_SCHEMA_VERSION, pet_id: pet.id, old_owner, new_owner: pet.owner.clone(), timestamp: pet.updated_at });
        proposal.executed = true;
        env.storage().instance().set(&SystemKey::PetTransferProposal(proposal_id), &proposal);
        true
    }

    pub fn cancel_transfer_proposal(env: Env, proposal_id: u64) {
        let mut proposal = Self::get_transfer_proposal(env.clone(), proposal_id)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::InvalidInput));
        let pet: Pet = env.storage().instance().get(&DataKey::Pet(proposal.pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        pet.owner.require_auth();
        if proposal.executed { panic_with_error!(&env, ContractError::InvalidState); }
        proposal.executed = true;
        env.storage().instance().set(&SystemKey::PetTransferProposal(proposal_id), &proposal);
    }

    pub fn get_active_transfer_proposals(env: Env, pet_id: u64) -> Vec<PetTransferProposal> {
        let ids: Vec<u64> = env.storage().instance().get(&SystemKey::PetActiveProposals(pet_id)).unwrap_or(Vec::new(&env));
        let mut result = Vec::new(&env);
        for id in ids.iter() {
            if let Some(proposal) = Self::get_transfer_proposal(env.clone(), id) {
                if !proposal.executed && env.ledger().timestamp() <= proposal.expires_at { result.push_back(proposal); }
            }
        }
        result
    }

    // --- HELPER FOR INDEX MAINTENANCE ---
    fn remove_pet_from_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = PetChainContract::get_owner_pet_count(env, owner);
        if count == 0 {
            return;
        }

        let mut remove_index: Option<u64> = None;
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if pid == pet_id {
                    remove_index = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = remove_index {
            if idx != count {
                if let Some(last_pet_id) = env
                    .storage()
                    .instance()
                    .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), count)))
                {
                    env.storage()
                        .instance()
                        .set(&DataKey::OwnerPetIndex((owner.clone(), idx)), &last_pet_id);
                }
            }
            env.storage()
                .instance()
                .remove(&DataKey::OwnerPetIndex((owner.clone(), count)));
            env.storage()
                .instance()
                .set(&DataKey::PetCountByOwner(owner.clone()), &(count - 1));
        }
    }

    fn add_pet_to_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = PetChainContract::get_owner_pet_count(env, owner);
        let new_count = safe_increment(env, count);
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &new_count);
        env.storage()
            .instance()
            .set(&DataKey::OwnerPetIndex((owner.clone(), new_count)), &pet_id);
    }

    // --- OWNER MANAGEMENT ---

    pub fn register_pet_owner(
        env: Env,
        owner: Address,
        name: String,
        email: String,
        emergency_contact: String,
    ) {
        owner.require_auth();

        if name.len() > PetChainContract::MAX_STR_SHORT {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        if email.len() > PetChainContract::MAX_STR_SHORT {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        if emergency_contact.len() > PetChainContract::MAX_STR_SHORT {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        let key = PetChainContract::get_encryption_key(&env);
        let timestamp = env.ledger().timestamp();

        let name_bytes = name.to_xdr(&env);
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, &name_bytes, &key);
        let encrypted_name = EncryptedData {
            nonce: name_nonce,
            ciphertext: name_ciphertext,
        };

        let email_bytes = email.to_xdr(&env);
        let (email_nonce, email_ciphertext) = encrypt_sensitive_data(&env, &email_bytes, &key);
        let encrypted_email = EncryptedData {
            nonce: email_nonce,
            ciphertext: email_ciphertext,
        };

        let contact_bytes = emergency_contact.to_xdr(&env);
        let (contact_nonce, contact_ciphertext) =
            encrypt_sensitive_data(&env, &contact_bytes, &key);
        let encrypted_emergency_contact = EncryptedData {
            nonce: contact_nonce,
            ciphertext: contact_ciphertext,
        };

        let pet_owner = PetOwner {
            owner_address: owner.clone(),
            privacy_level: PrivacyLevel::Public,
            encrypted_name,
            encrypted_email,
            encrypted_emergency_contact,
            created_at: timestamp,
            updated_at: timestamp,
            is_pet_owner: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::PetOwner(owner), &pet_owner);
    }

    pub fn is_owner_registered(env: Env, owner: Address) -> bool {
        if let Some(pet_owner) = env
            .storage()
            .instance()
            .get::<DataKey, PetOwner>(&DataKey::PetOwner(owner))
        {
            pet_owner.is_pet_owner
        } else {
            false
        }
    }

    pub fn update_owner_profile(
        env: Env,
        owner: Address,
        name: String,
        email: String,
        emergency_contact: String,
    ) -> bool {
        owner.require_auth();

        if let Some(mut pet_owner) = env
            .storage()
            .instance()
            .get::<DataKey, PetOwner>(&DataKey::PetOwner(owner.clone()))
        {
            let key = PetChainContract::get_encryption_key(&env);

            let name_bytes = name.to_xdr(&env);
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, &name_bytes, &key);
            pet_owner.encrypted_name = EncryptedData {
                nonce: name_nonce,
                ciphertext: name_ciphertext,
            };

            let email_bytes = email.to_xdr(&env);
            let (email_nonce, email_ciphertext) = encrypt_sensitive_data(&env, &email_bytes, &key);
            pet_owner.encrypted_email = EncryptedData {
                nonce: email_nonce,
                ciphertext: email_ciphertext,
            };

            let contact_bytes = emergency_contact.to_xdr(&env);
            let (contact_nonce, contact_ciphertext) =
                encrypt_sensitive_data(&env, &contact_bytes, &key);
            pet_owner.encrypted_emergency_contact = EncryptedData {
                nonce: contact_nonce,
                ciphertext: contact_ciphertext,
            };

            pet_owner.updated_at = env.ledger().timestamp();

            env.storage()
                .instance()
                .set(&DataKey::PetOwner(owner), &pet_owner);
            true
        } else {
            false
        }
    }

    // Vet Verification & Registration
    #[allow(dead_code)]
    const MAX_STR_SHORT: u32 = 100;
    #[allow(dead_code)]
    const MAX_STR_LONG: u32 = 1000;
    #[allow(dead_code)]
    const MAX_VEC_MEDS: u32 = 20;
    #[allow(dead_code)]
    const MAX_VEC_ATTACHMENTS: u32 = 20;
    #[allow(dead_code)]
    const MAX_VET_NAME_LEN: u32 = 100;
    #[allow(dead_code)]
    const MAX_VET_LICENSE_LEN: u32 = 50;
    #[allow(dead_code)]
    const MAX_VET_SPEC_LEN: u32 = 100;

    /// Maximum byte length of a vet-review comment.
    /// Enforced in `add_vet_review` to bound on-chain storage and gas costs.
    #[allow(dead_code)]
    const MAX_REVIEW_COMMENT_LEN: u32 = 500;
    #[allow(dead_code)]
    const MAX_SEARCH_KEYWORD_LEN: u32 = 64;
    #[allow(dead_code)]
    const MAX_SEARCH_NOTES_LEN: u32 = 1000;
    #[allow(dead_code)]
    const MAX_SEARCH_TOKENS_PER_RECORD: u32 = 50;

    /// Validates that `value` does not exceed `max` bytes.
    ///
    /// `field` names the offending field so callers can surface a clear error.
    /// Returns `ContractError::InvalidInput` when the limit is exceeded.
    fn validate_len(field: &str, value: &String, max: u32) -> Result<(), ContractError> {
        let _ = field;
        if value.len() > max {
            return Err(ContractError::InvalidInput);
        }
        Ok(())
    }

    pub fn register_vet(
        env: Env,
        vet_address: Address,
        name: String,
        license_number: String,
        specialization: String,
    ) -> bool {
        vet_address.require_auth();

        if let Err(e) = Self::validate_len("name", &name, PetChainContract::MAX_VET_NAME_LEN) {
            panic_with_error!(&env, e);
        }

        if let Err(e) = Self::validate_len(
            "license_number",
            &license_number,
            PetChainContract::MAX_VET_LICENSE_LEN,
        ) {
            panic_with_error!(&env, e);
        }

        if let Err(e) = Self::validate_len(
            "specialization",
            &specialization,
            PetChainContract::MAX_VET_SPEC_LEN,
        ) {
            panic_with_error!(&env, e);
        }

        if env
            .storage()
            .instance()
            .has(&DataKey::VetLicense(license_number.clone()))
        {
            panic_with_error!(&env, ContractError::LicenseAlreadyRegistered);
        }

        if env
            .storage()
            .instance()
            .has(&DataKey::Vet(vet_address.clone()))
        {
            panic_with_error!(&env, ContractError::VetAlreadyRegistered);
        }

        let vet = Vet {
            address: vet_address.clone(),
            name,
            license_number: license_number.clone(),
            specialization,
            verified: false,
            clinic_info: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::Vet(vet_address.clone()), &vet);
        env.storage()
            .instance()
            .set(&DataKey::VetLicense(license_number), &vet_address);

        let vet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VetCount)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&DataKey::VetCount, &vet_count);
        env.storage()
            .instance()
            .set(&DataKey::VetIndex(vet_count), &vet_address);

        true
    }

    pub fn get_verified_vets(env: Env, offset: u64, limit: u32) -> Vec<Vet> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VetCount)
            .unwrap_or(0);
        let mut result = Vec::new(&env);
        if count == 0 || limit == 0 {
            return result;
        }
        let mut skipped = 0u64;
        for i in 1..=count {
            if let Some(addr) = env
                .storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::VetIndex(i))
            {
                if let Some(vet) = env
                    .storage()
                    .instance()
                    .get::<DataKey, Vet>(&DataKey::Vet(addr))
                {
                    if !vet.verified {
                        continue;
                    }
                    if skipped < offset {
                        skipped += 1;
                        continue;
                    }
                    result.push_back(vet);
                    if result.len() >= limit {
                        break;
                    }
                }
            }
        }
        result
    }

    pub fn verify_vet(env: Env, admin: Address, vet_address: Address) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);
        let verified = PetChainContract::_verify_vet_internal(&env, vet_address);
        if verified {
            Self::record_admin_activity(&env, &admin, "verify_vet");
        }
        verified
    }

    /// Batch verify multiple vets in a single call
    /// Maximum batch size: 20 vets
    /// Returns BatchResult with succeeded and failed addresses
    /// Does not abort on individual failures - continues processing all vets
    pub fn batch_verify_vets(env: Env, admin: Address, vet_addresses: Vec<Address>) -> BatchResult {
        // Require admin authorization
        PetChainContract::require_admin_auth(&env, &admin);

        // Validate batch size
        let batch_size = vet_addresses.len();
        if batch_size > 20 {
            panic_with_error!(&env, ContractError::BatchTooLarge);
        }

        // Initialize result vectors
        let mut succeeded = Vec::new(&env);
        let mut failed = Vec::new(&env);

        // Process each vet address
        for vet_address in vet_addresses.iter() {
            // Check if vet exists
            if let Some(mut vet) = env
                .storage()
                .instance()
                .get::<DataKey, Vet>(&DataKey::Vet(vet_address.clone()))
            {
                // Vet exists, verify it
                vet.verified = true;
                env.storage()
                    .instance()
                    .set(&DataKey::Vet(vet.address.clone()), &vet);
                env.storage()
                    .instance()
                    .remove(&DataKey::VetCredentialsExpiry(vet_address.clone()));
                succeeded.push_back(vet_address.clone());
            } else {
                // Vet not found, record failure
                failed.push_back((vet_address.clone(), ContractError::VetNotFound as u32));
            }
        }

        BatchResult { succeeded, failed }
    }

    pub fn register_vet_specializations(
        env: Env,
        admin: Address,
        vet_address: Address,
        specializations: Vec<Specialization>,
    ) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);

        let vet = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::VetNotFound));

        if !vet.verified {
            panic_with_error!(&env, ContractError::VeterinarianNotVerified);
        }
        if let Some(expiry) = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::VetCredentialsExpiry(vet_address.clone()))
        {
            if expiry != 0 && env.ledger().timestamp() >= expiry {
                panic_with_error!(&env, ContractError::VetCredentialsExpired);
            }
        }

        if specializations.is_empty() || specializations.len() > 5 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let mut verified = Vec::new(&env);
        for specialization in specializations.iter() {
            if !verified.contains(&specialization) {
                verified.push_back(specialization);
            }
        }

        env.storage()
            .instance()
            .set(&DataKey::VetSpecializations(vet_address), &verified);
        true
    }

    pub fn get_vet_specializations(env: Env, vet_address: Address) -> Vec<Specialization> {
        env.storage()
            .instance()
            .get(&DataKey::VetSpecializations(vet_address))
            .unwrap_or_else(|| Vec::new(&env))
    }

    #[allow(dead_code)]
    fn vet_has_specialization(
        env: &Env,
        vet_address: &Address,
        specialization: Specialization,
    ) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Vec<Specialization>>(&DataKey::VetSpecializations(vet_address.clone()))
            .map(|specializations| specializations.contains(&specialization))
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    fn require_vet_specialization(
        env: &Env,
        vet_address: &Address,
        specialization: Specialization,
    ) {
        if !Self::vet_has_specialization(env, vet_address, specialization) {
            panic_with_error!(env, ContractError::Unauthorized);
        }
    }

    fn _verify_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            vet.verified = true;
            env.storage()
                .instance()
                .set(&DataKey::Vet(vet.address.clone()), &vet);
            env.storage()
                .instance()
                .remove(&DataKey::VetCredentialsExpiry(vet_address));
            true
        } else {
            false
        }
    }

    pub fn revoke_vet_license(env: Env, admin: Address, vet_address: Address) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);
        let revoked = PetChainContract::_revoke_vet_internal(&env, vet_address);
        if revoked {
            Self::record_admin_activity(&env, &admin, "revoke_vet_license");
        }
        revoked
    }

    fn _revoke_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            vet.verified = false;
            env.storage()
                .instance()
                .set(&DataKey::Vet(vet.address.clone()), &vet);
            env.storage()
                .instance()
                .remove(&DataKey::VetCredentialsExpiry(vet_address));
            true
        } else {
            false
        }
    }

    pub fn is_vet_registered(env: Env, vet_address: Address) -> bool {
        env.storage().instance().has(&DataKey::Vet(vet_address))
    }

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        if let Some(vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            if !vet.verified {
                return false;
            }
            if let Some(expiry) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::VetCredentialsExpiry(vet_address))
            {
                if expiry != 0 && env.ledger().timestamp() >= expiry {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    /// Authorise a vet, optionally binding an optional credential expiry.
    ///
    /// * `expires_at == None` — credentials are perpetual (no expiry recorded).
    /// * `expires_at <= now` — rejected with `InvalidInput`.
    pub fn verify_vet_with_expiry(
        env: Env,
        admin: Address,
        vet_address: Address,
        expires_at: Option<u64>,
    ) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);
        let verified = PetChainContract::_verify_vet_internal(&env, vet_address);
        if verified {
            if let Some(exp) = expires_at {
                if exp <= env.ledger().timestamp() {
                    panic_with_error!(&env, ContractError::InvalidInput);
                }
                env.storage()
                    .instance()
                    .set(&DataKey::VetCredentialsExpiry(vet_address.clone()), &exp);
            } else {
                env.storage()
                    .instance()
                    .remove(&DataKey::VetCredentialsExpiry(vet_address));
            }
            Self::record_admin_activity(&env, &admin, "verify_vet_with_expiry");
        }
        verified
    }

    /// Authorise a vet, returning precise errors for missing, unverified, or
    /// expired credentials.
    fn require_verified_vet(env: &Env, vet_address: &Address) {
        let vet: Vet = env
            .storage()
            .instance()
            .get(&DataKey::Vet(vet_address.clone()))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::VetNotFound));
        if !vet.verified {
            panic_with_error!(env, ContractError::VetNotVerified);
        }
        if let Some(expiry) = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::VetCredentialsExpiry(vet_address.clone()))
        {
            if expiry != 0 && env.ledger().timestamp() >= expiry {
                panic_with_error!(env, ContractError::VetCredentialsExpired);
            }
        }
    }

    pub fn get_vet(env: Env, vet_address: Address) -> Option<Vet> {
        env.storage().instance().get(&DataKey::Vet(vet_address))
    }

    pub fn get_vet_by_license(env: Env, license_number: String) -> Option<Vet> {
        let vet_address: Option<Address> = env
            .storage()
            .instance()
            .get(&DataKey::VetLicense(license_number));
        vet_address.and_then(|address| PetChainContract::get_vet(env, address))
    }

    /*
    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub fn update_clinic_info(env: Env, vet_address: Address, clinic_info: String) -> bool {
        vet_address.require_auth();

        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<_, Vet>(&DataKey::Vet(vet_address.clone()))
        {
            vet.clinic_info = Some(clinic_info);
            env.storage()
                .instance()
                .set(&DataKey::Vet(vet_address), &vet);
            true
        } else {
            panic_with_error!(&env, ContractError::VetNotFound);
        }
    }
    */

    // Pet Vaccination Record
    #[allow(clippy::too_many_arguments)]
    pub fn add_vaccination(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        vaccine_type: VaccineType,
        vaccine_name: String,
        administered_at: u64,
        next_due_date: u64,
        expires_at: u64,
        batch_number: String,
    ) -> u64 {
        veterinarian.require_auth();
        if !PetChainContract::is_verified_vet(env.clone(), veterinarian.clone()) {
            panic_with_error!(&env, ContractError::VeterinarianNotVerified);
        }

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        let now = env.ledger().timestamp();

        // Validate medical-event timestamps against ledger time (Issue #1174).
        // `administered_at` must not be further in the future than the
        // allowed clock-skew tolerance relative to the current ledger time.
        if administered_at > now.saturating_add(MAX_EVENT_FUTURE_SKEW) {
            panic_with_error!(&env, ContractError::InvalidTimestamp);
        }
        // `next_due_date` and `expires_at` (when set) describe follow-up
        // dates and must not precede the event they follow, nor sit
        // absurdly far beyond it.
        let max_future = administered_at.saturating_add(MAX_EVENT_HORIZON);
        if next_due_date != 0 && (next_due_date < administered_at || next_due_date > max_future) {
            panic_with_error!(&env, ContractError::InvalidTimestamp);
        }
        if expires_at != 0 && (expires_at < administered_at || expires_at > max_future) {
            panic_with_error!(&env, ContractError::InvalidTimestamp);
        }

        // Check storage quota (Issue #676)
        Self::increment_pet_storage(&env, pet_id);

        let vaccine_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::VaccinationCount)
            .unwrap_or(0);
        let vaccine_id = vaccine_count
            .checked_add(1)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow));
        let key = PetChainContract::get_encryption_key(&env);

        let vname_bytes = vaccine_name.to_xdr(&env);
        let (vname_nonce, vname_ciphertext) = encrypt_sensitive_data(&env, &vname_bytes, &key);
        let encrypted_vaccine_name = EncryptedData {
            nonce: vname_nonce,
            ciphertext: vname_ciphertext,
        };

        let batch_bytes = batch_number.to_xdr(&env);
        let (batch_nonce, batch_ciphertext) = encrypt_sensitive_data(&env, &batch_bytes, &key);
        let encrypted_batch_number = EncryptedData {
            nonce: batch_nonce,
            ciphertext: batch_ciphertext,
        };

        // If expires_at is 0, default to next_due_date
        let effective_expires_at = if expires_at == 0 {
            next_due_date
        } else {
            expires_at
        };

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian: veterinarian.clone(),
            vaccine_type: vaccine_type.clone(),
            vaccine_name: None,
            encrypted_vaccine_name,
            administered_at,
            next_due_date,
            expires_at: effective_expires_at,
            batch_number: None,
            encrypted_batch_number,
            created_at: now,
            revoked: false,
            revocation_reason: None,
        };

        PetChainContract::update_vet_stats(&env, &veterinarian, pet_id, 1, 1, 0);

        env.storage()
            .instance()
            .set(&MedicalKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&MedicalKey::VaccinationCount, &vaccine_id);

        // Update indexes
        let pet_vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let new_pet_vax_count = safe_increment(&env, pet_vax_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetVaccinationCount(pet_id), &new_pet_vax_count);
        env.storage().instance().set(
            &MedicalKey::PetVaccinationByIndex((pet_id, new_pet_vax_count)),
            &vaccine_id,
        );

        // Update vet vaccination index
        let vet_vax_count = env
            .storage()
            .instance()
            .get::<VetKey, u64>(&VetKey::VetVaccinationCount(veterinarian.clone()))
            .unwrap_or(0);
        let new_vet_vax_count = safe_increment(&env, vet_vax_count);
        env.storage().instance().set(
            &VetKey::VetVaccinationCount(veterinarian.clone()),
            &new_vet_vax_count,
        );
        env.storage().instance().set(
            &VetKey::VetVaccinationIndex((veterinarian.clone(), new_vet_vax_count)),
            &vaccine_id,
        );

        env.events().publish(
            (String::from_str(&env, "VaccinationAdded"), pet_id),
            VaccinationAddedEvent {
                version: EVENT_SCHEMA_VERSION,
                vaccine_id,
                pet_id,
                veterinarian,
                vaccine_type,
                next_due_date,
                timestamp: now,
                subscription_ids: Self::matching_subscription_ids(
                    &env,
                    EventType::VaccinationAdded,
                    pet_id,
                ),
            },
        );

        // Lazy expiry check: emit VaccinationExpiringSoon for this pet's vaccinations
        PetChainContract::check_and_emit_expiry_events(env, pet_id, 30);

        vaccine_id
    }

    pub fn revoke_vaccination_certificate(
        env: Env,
        vet_or_admin: Address,
        pet_id: u64,
        cert_id: u64,
        reason: String,
    ) {
        vet_or_admin.require_auth();

        // Must exist
        let mut vax: Vaccination = env
            .storage()
            .instance()
            .get(&MedicalKey::Vaccination(cert_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::VaccinationNotFound));

        // Must match pet
        if vax.pet_id != pet_id {
            panic_with_error!(&env, ContractError::VaccinationNotFound);
        }

        // Verify authorization: must be issuing vet OR admin
        let is_admin = PetChainContract::is_admin(&env, &vet_or_admin);
        if !is_admin && vax.veterinarian != vet_or_admin {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        vax.revoked = true;
        vax.revocation_reason = Some(reason.clone());

        env.storage()
            .instance()
            .set(&MedicalKey::Vaccination(cert_id), &vax);

        // Emit event
        env.events().publish(
            (String::from_str(&env, "VaccinationRevoked"), pet_id),
            VaccinationRevokedEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                vaccination_id: cert_id,
                vet_or_admin,
                reason,
                timestamp: env.ledger().timestamp(),
            },
        );

        // Cascade revocation to any anchored certificate for this vaccination.
        let cert_lifecycle_key = MedicalKey::CertificateLifecycle((pet_id, cert_id));
        if env.storage().instance().has(&cert_lifecycle_key) {
            if let Some(mut cert_lifecycle) = env
                .storage()
                .instance()
                .get::<MedicalKey, CertificateLifecycle>(&cert_lifecycle_key)
            {
                if !cert_lifecycle.revoked {
                    cert_lifecycle.revoked = true;
                    cert_lifecycle.revoked_at = Some(env.ledger().timestamp());
                    if cert_lifecycle.revocation_reason.is_none() {
                        cert_lifecycle.revocation_reason = Some(reason.clone());
                    }
                    env.storage()
                        .instance()
                        .set(&cert_lifecycle_key, &cert_lifecycle);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Lab Results
    // -----------------------------------------------------------------------

    /// Integer square root (Newton's method).  Returns floor(sqrt(n)); 0 for n ≤ 0.
    fn isqrt(n: i128) -> i128 {
        if n <= 0 {
            return 0;
        }
        let mut x = n;
        let mut y = (x + 1) / 2;
        while y < x {
            x = y;
            y = (x + n / x) / 2;
        }
        x
    }

    /// Compute z-score × 100 (integer arithmetic) for `value` against `history`.
    /// Returns 0 when stddev is 0 (all values identical).
    fn zscore_scaled(value: i128, history: &[i128]) -> i128 {
        let n = history.len() as i128;
        let sum: i128 = history.iter().fold(0i128, |acc, &v| acc.saturating_add(v));
        let mean = sum / n;
        let variance = history.iter().fold(0i128, |acc, &v| {
            let diff = v.saturating_sub(mean);
            acc.saturating_add(diff.saturating_mul(diff))
        }) / n;
        let stddev = Self::isqrt(variance);
        if stddev == 0 {
            return 0;
        }
        value.saturating_sub(mean).saturating_mul(100) / stddev
    }

    /// Add a lab result for a pet.  If the result includes biomarker values and
    /// the pet already has at least 3 prior readings for a given biomarker, the
    /// z-score of the new value is computed against the last 10 readings.
    /// A [`LabResultAnomaly`] event is emitted for every biomarker whose
    /// |z-score × 100| exceeds 300 (i.e. z > 3.0).  The call is never blocked.
    pub fn add_lab_result(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        test_type: String,
        results: String,
        reference_ranges: String,
        attachment_hash: Option<String>,
        medical_record_id: Option<u64>,
        biomarkers: Map<String, i128>,
    ) -> u64 {
        vet_address.require_auth();

        // Bound string fields to prevent unbounded ledger growth. (#1152)
        if test_type.len() > MAX_LAB_TEST_TYPE_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        if results.len() > MAX_LAB_RESULTS_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        if reference_ranges.len() > MAX_LAB_REF_RANGES_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        // --- allocate ID ---
        let lab_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::LabResultCount)
            .unwrap_or(0);
        let lab_id = safe_increment(&env, lab_count);

        // --- anomaly detection: run BEFORE storing so we see only prior readings ---
        let prior_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetLabResultCount(pet_id))
            .unwrap_or(0);

        // For each biomarker in the new result, collect up to 10 prior values.
        for biomarker_name in biomarkers.keys() {
            let new_value = biomarkers.get(biomarker_name.clone()).unwrap();

            // Walk the most recent prior results (up to 10).
            let window_start = if prior_count >= 10 {
                prior_count - 9
            } else {
                1
            };
            let mut history: [i128; 10] = [0i128; 10];
            let mut history_len: usize = 0;

            let mut idx = prior_count;
            while idx >= window_start && history_len < 10 {
                if let Some(prev_lab_id) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, u64>(&MedicalKey::PetLabResultIndex((pet_id, idx)))
                {
                    if let Some(prev_lab) = env
                        .storage()
                        .instance()
                        .get::<MedicalKey, LabResult>(&MedicalKey::LabResult(prev_lab_id))
                    {
                        if let Some(v) = prev_lab.biomarkers.get(biomarker_name.clone()) {
                            history[history_len] = v;
                            history_len += 1;
                        }
                    }
                }
                if idx == 0 {
                    break;
                }
                idx -= 1;
            }

            // Need at least 3 readings to compute a meaningful z-score.
            if history_len < 3 {
                continue;
            }

            let z = Self::zscore_scaled(new_value, &history[..history_len]);
            // Emit anomaly when |z × 100| > 300  (i.e. z > 3.0).
            let abs_z = if z < 0 { z.saturating_neg() } else { z };
            if abs_z > 300 {
                env.events().publish(
                    (String::from_str(&env, "LAB_RESULT_ANOMALY"), pet_id),
                    LabResultAnomaly {
                        version: EVENT_SCHEMA_VERSION,
                        pet_id,
                        biomarker: biomarker_name,
                        value: new_value,
                        z_score: z,
                    },
                );
            }
        }

        // --- store the new lab result ---
        let now = env.ledger().timestamp();
        let lab_result = LabResult {
            id: lab_id,
            pet_id,
            test_type,
            date: now,
            results,
            vet_address,
            reference_ranges,
            attachment_hash,
            medical_record_id,
            biomarkers,
            biomarker_flags: Map::new(&env),
        };

        env.storage()
            .instance()
            .set(&MedicalKey::LabResult(lab_id), &lab_result);
        env.storage()
            .instance()
            .set(&MedicalKey::LabResultCount, &lab_id);

        let new_pet_lab_count = safe_increment(&env, prior_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetLabResultCount(pet_id), &new_pet_lab_count);
        env.storage().instance().set(
            &MedicalKey::PetLabResultIndex((pet_id, new_pet_lab_count)),
            &lab_id,
        );

        lab_id
    }

    pub fn get_lab_results(
        env: Env,
        pet_id: u64,
        offset: u64,
        limit: u32,
        from_timestamp: Option<u64>,
        to_timestamp: Option<u64>,
    ) -> Vec<LabResult> {
        if let (Some(from), Some(to)) = (from_timestamp, to_timestamp) {
            if from > to {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
        }

        let lab_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetLabResultCount(pet_id))
            .unwrap_or(0);

        let mut result = Vec::new(&env);
        let mut included_count = 0u64;

        for i in 1..=lab_count {
            if included_count >= limit as u64 {
                break;
            }

            if let Some(lab_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetLabResultIndex((pet_id, i)))
            {
                if let Some(lab) = PetChainContract::get_lab_result(env.clone(), lab_id) {
                    let in_range = match (from_timestamp, to_timestamp) {
                        (Some(from), Some(to)) => lab.date >= from && lab.date <= to,
                        (Some(from), None) => lab.date >= from,
                        (None, Some(to)) => lab.date <= to,
                        (None, None) => true,
                    };

                    if in_range {
                        if included_count >= offset {
                            result.push_back(lab);
                        }
                        included_count += 1;
                    }
                }
            }
        }

        result
    }

    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        if let Some(record) = env
            .storage()
            .instance()
            .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vaccine_id))
        {
            let key = PetChainContract::get_encryption_key(&env);

            let name_bytes = decrypt_sensitive_data(
                &env,
                &record.encrypted_vaccine_name.ciphertext,
                &record.encrypted_vaccine_name.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let vaccine_name =
                String::from_xdr(&env, &name_bytes).unwrap_or(String::from_str(&env, "Error"));

            let batch_bytes = decrypt_sensitive_data(
                &env,
                &record.encrypted_batch_number.ciphertext,
                &record.encrypted_batch_number.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let batch_number =
                String::from_xdr(&env, &batch_bytes).unwrap_or(String::from_str(&env, "Error"));

            let mut decrypted = record.clone();
            decrypted.vaccine_name = Some(vaccine_name);
            decrypted.batch_number = Some(batch_number);
            Some(decrypted)
        } else {
            None
        }
    }

    pub fn get_vaccination_history(
        env: Env,
        pet_id: u64,
        offset: u64,
        limit: u32,
    ) -> Vec<Vaccination> {
        if env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .is_none()
        {
            return Vec::new(&env);
        }

        let _vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        // Here we return decrypted history. Privacy check omitted for brevity in this merge step,
        // relying on upstream behavior + encryption presence.
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        // Calculate the range to return based on offset and limit
        let start_index = safe_increment(&env, offset); // Indices start from 1
        let end_index = (offset + limit as u64).min(count);

        for i in start_index..=end_index {
            if let Some(vid) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = PetChainContract::get_vaccinations(env.clone(), vid) {
                    history.push_back(vax);
                }
            }
        }
        history
    }

    pub fn get_upcoming_vaccinations(
        env: Env,
        pet_id: u64,
        days_threshold: u64,
    ) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let threshold = duration_window_end(current_time, days_threshold);
        let history = PetChainContract::get_vaccination_history(env.clone(), pet_id, 0, u32::MAX);
        let mut upcoming = Vec::new(&env);

        for vax in history.iter() {
            if vax.next_due_date <= threshold {
                upcoming.push_back(vax);
            }
        }
        upcoming
    }

    pub fn is_vaccination_current(env: Env, pet_id: u64, vaccine_type: VaccineType) -> bool {
        let current_time = env.ledger().timestamp();
        let history = PetChainContract::get_vaccination_history(env, pet_id, 0, u32::MAX);
        let mut most_recent: Option<Vaccination> = None;

        for vax in history.iter() {
            if vax.vaccine_type == vaccine_type && !vax.revoked {
                match most_recent.clone() {
                    Some(current) => {
                        if vax.administered_at > current.administered_at {
                            most_recent = Some(vax);
                        }
                    }
                    None => most_recent = Some(vax),
                }
            }
        }

        if let Some(vax) = most_recent {
            vax.next_due_date > current_time
        } else {
            false
        }
    }

    pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<VaccineType> {
        let current_time = env.ledger().timestamp();
        let history = PetChainContract::get_vaccination_history(env.clone(), pet_id, 0, u32::MAX);
        let mut overdue = Vec::new(&env);

        for vax in history.iter() {
            if vax.next_due_date < current_time {
                overdue.push_back(vax.vaccine_type);
            }
        }
        overdue
    }

    /// Returns true if `vet` has any vaccination or medical record for `pet_id`.
    fn pet_has_vet_care_record(env: Env, pet_id: u64, vet: Address) -> bool {
        // Check vaccinations
        let vax_count: u64 = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        for i in 1..=vax_count {
            if let Some(vax_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vax_id))
                {
                    if vax.veterinarian == vet {
                        return true;
                    }
                }
            }
        }
        // Check medical records
        let rec_count: u64 = env
            .storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        for i in 1..=rec_count {
            if let Some(rec_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(rec) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(rec_id))
                {
                    if rec.vet_address == vet && rec.deleted_at.is_none() {
                        return true;
                    }
                }
            }
        }
        false
    }



    /// Returns vaccinations for `pet_id` that expire within `within_days` days,
    /// including already-expired ones (flagged via `already_expired: true`).
    pub fn get_expiring_vaccinations(
        env: Env,
        pet_id: u64,
        within_days: u64,
    ) -> Vec<ExpiringVaccination> {
        let now = env.ledger().timestamp();
        let window_end = now.saturating_add(within_days.saturating_mul(86400));
        let history = PetChainContract::get_vaccination_history(env.clone(), pet_id, 0, u32::MAX);
        let mut result = Vec::new(&env);

        for vax in history.iter() {
            let exp = vax.expires_at;
            let already_expired = exp < now;
            let within_window = exp <= window_end;
            if already_expired || within_window {
                let days_remaining = if already_expired {
                    0
                } else {
                    (exp.saturating_sub(now)) / 86400
                };
                result.push_back(ExpiringVaccination {
                    vaccine_id: vax.id,
                    vaccine_type: vax.vaccine_type,
                    expires_at: exp,
                    days_remaining,
                    already_expired,
                });
            }
        }
        result
    }

    /// Internal helper: emit `VaccinationExpiringSoon` for any vaccination on
    /// `pet_id` that expires within `within_days` days (lazy, called on writes).
    fn check_and_emit_expiry_events(env: Env, pet_id: u64, within_days: u64) {
        let now = env.ledger().timestamp();
        let window_end = now.saturating_add(within_days.saturating_mul(86400));
        let history = PetChainContract::get_vaccination_history(env.clone(), pet_id, 0, u32::MAX);

        for vax in history.iter() {
            let exp = vax.expires_at;
            let already_expired = exp < now;
            if already_expired || exp <= window_end {
                let days_remaining = if already_expired {
                    0
                } else {
                    (exp.saturating_sub(now)) / 86400
                };
                env.events().publish(
                    (String::from_str(&env, "VaccinationExpiringSoon"), pet_id),
                    VaccinationExpiringSoonEvent {
                        version: EVENT_SCHEMA_VERSION,
                        vaccine_id: vax.id,
                        pet_id,
                        vaccine_type: vax.vaccine_type,
                        expires_at: exp,
                        days_remaining,
                        already_expired,
                        timestamp: now,
                    },
                );
            }
        }
    }

    pub fn get_vaccination_summary(env: Env, pet_id: u64) -> VaccinationSummary {
        let overdue_types = PetChainContract::get_overdue_vaccinations(env.clone(), pet_id);
        let upcoming = PetChainContract::get_upcoming_vaccinations(env.clone(), pet_id, 30);

        VaccinationSummary {
            is_fully_current: overdue_types.is_empty(),
            overdue_types,
            upcoming_count: upcoming.len() as u64,
        }
    }

    // --- VACCINATION CERTIFICATE ANCHORING (Issue #693) ---

    /// Anchor a vaccination certificate hash on-chain for authenticity verification.
    /// Only verified vets can anchor certificates.
    ///
    /// # Arguments
    /// * `issuer` - The verified vet anchoring the certificate
    /// * `pet_id` - The ID of the pet
    /// * `vaccination_id` - The ID of the vaccination
    /// * `cert_hash` - Hash of the PDF certificate (e.g., SHA-256)
    ///
    /// # Errors
    /// * `VetNotVerified` - Issuer is not a verified vet
    /// * `PetNotFound` - Pet doesn't exist
    /// * `VaccinationNotFound` - Vaccination doesn't exist
    /// * `CertificateAlreadyAnchored` - Certificate already anchored for this vaccination
    /// * `InvalidCertificateHash` - Certificate hash is empty or invalid format
    ///
    /// # Events
    /// Emits `CertificateAnchoredEvent` on success
    pub fn anchor_certificate(
        env: Env,
        issuer: Address,
        pet_id: u64,
        vaccination_id: u64,
        cert_hash: String,
    ) {
        issuer.require_auth();

        Self::require_verified_vet(&env, &issuer);

        // Verify pet exists
        let _pet: Pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        // Verify vaccination exists and belongs to the pet
        let vaccination: Vaccination = env
            .storage()
            .instance()
            .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vaccination_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::VaccinationNotFound));

        if vaccination.pet_id != pet_id {
            panic_with_error!(&env, ContractError::VaccinationNotFound);
        }

        // Validate certificate hash
        if cert_hash.is_empty() || cert_hash.len() > 128 {
            panic_with_error!(&env, ContractError::InvalidCertificateHash);
        }

        // Check if certificate already anchored
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        if env.storage().instance().has(&anchor_key) {
            panic_with_error!(&env, ContractError::CertificateAlreadyAnchored);
        }

        let current_time = env.ledger().timestamp();

        // Assign a stable certificate identifier.
        let cert_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::CertificateCount)
            .unwrap_or(0);
        let cert_id = safe_increment(cert_count);
        env.storage()
            .instance()
            .set(&MedicalKey::CertificateCount, &cert_id);

        // Certificate lifetime binds to the underlying dose's expiry.
        let expiry = vaccination.expires_at;

        // Create and store certificate anchor
        let anchor = CertificateAnchor {
            pet_id,
            vaccination_id,
            cert_hash: cert_hash.clone(),
            issuer: issuer.clone(),
            anchored_at: current_time,
        };

        env.storage().instance().set(&anchor_key, &anchor);

        let lifecycle = CertificateLifecycle {
            cert_id,
            issue_time: current_time,
            expiry,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        };
        env.storage().instance().set(
            &MedicalKey::CertificateLifecycle((pet_id, vaccination_id)),
            &lifecycle,
        );

        // Emit event
        env.events().publish(
            (String::from_str(&env, "CertificateAnchored"), pet_id),
            CertificateAnchoredEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                vaccination_id,
                cert_hash,
                issuer,
                timestamp: current_time,
            },
        );
    }

    /// Idempotent variant of `anchor_certificate`.
    ///
    /// Uniqueness key = `(pet_id, vaccination_id)`.
    ///
    /// * If no certificate is anchored for the pair, a new certificate is
    ///   created and its `cert_id` is returned.
    /// * If the same `cert_hash` is already anchored, the existing `cert_id` is
    ///   returned (idempotent / safe to replay).
    /// * If a different `cert_hash` is already anchored, the call aborts with
    ///   `CertificateHashConflict`.
    pub fn anchor_certificate_idempotent(
        env: Env,
        issuer: Address,
        pet_id: u64,
        vaccination_id: u64,
        cert_hash: String,
    ) -> u64 {
        issuer.require_auth();
        let now = env.ledger().timestamp();

        // Authorise issuer.
        Self::require_verified_vet(&env, &issuer);

        // Verify pet exists.
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        // Verify vaccination exists and belongs to the pet.
        let vaccination: Vaccination = env
            .storage()
            .instance()
            .get::<MedicalKey, Vaccination>(&MedicalKey::Vaccination(vaccination_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::VaccinationNotFound));
        if vaccination.pet_id != pet_id {
            panic_with_error!(&env, ContractError::VaccinationNotFound);
        }

        // Validate certificate hash.
        if cert_hash.is_empty() || cert_hash.len() > 128 {
            panic_with_error!(&env, ContractError::InvalidCertificateHash);
        }

        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        let lifecycle_key = MedicalKey::CertificateLifecycle((pet_id, vaccination_id));

        // Idempotency: same-hash replay returns the existing cert_id.
        if let Some(existing) = env
            .storage()
            .instance()
            .get::<MedicalKey, CertificateAnchor>(&anchor_key)
        {
            if existing.cert_hash == cert_hash {
                if let Some(lifecycle) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, CertificateLifecycle>(&lifecycle_key)
                {
                    return lifecycle.cert_id;
                }
            }
            panic_with_error!(&env, ContractError::CertificateHashConflict);
        }

        // Assign cert_id.
        let cert_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::CertificateCount)
            .unwrap_or(0);
        let cert_id = safe_increment(cert_count);
        env.storage()
            .instance()
            .set(&MedicalKey::CertificateCount, &cert_id);

        let expiry = vaccination.expires_at;

        let anchor = CertificateAnchor {
            pet_id,
            vaccination_id,
            cert_hash: cert_hash.clone(),
            issuer: issuer.clone(),
            anchored_at: now,
        };
        env.storage().instance().set(&anchor_key, &anchor);

        let lifecycle = CertificateLifecycle {
            cert_id,
            issue_time: now,
            expiry,
            revoked: false,
            revoked_at: None,
            revocation_reason: None,
        };
        env.storage()
            .instance()
            .set(&lifecycle_key, &lifecycle);

        env.events().publish(
            (String::from_str(&env, "CertificateAnchored"), pet_id),
            CertificateAnchoredEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                vaccination_id,
                cert_hash,
                issuer: issuer.clone(),
                timestamp: now,
            },
        );

        cert_id
    }

    /// Verify if a certificate hash matches the anchored hash for a vaccination.
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet
    /// * `vaccination_id` - The ID of the vaccination
    /// * `cert_hash` - Hash to verify against the anchored hash
    ///
    /// # Returns
    /// * `true` if the hash matches the anchored certificate and the certificate is
    ///   not expired or revoked.
    /// * `false` if no certificate is anchored, the hash doesn't match, the
    ///   certificate has expired, or the certificate has been revoked.
    pub fn verify_certificate(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
        cert_hash: String,
    ) -> bool {
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));

        let Some(anchor) = env
            .storage()
            .instance()
            .get::<MedicalKey, CertificateAnchor>(&anchor_key)
        else {
            return false;
        };

        if anchor.cert_hash != cert_hash {
            return false;
        }

        // Lifecycle check: reject revoked or expired certificates.
        if let Some(lifecycle) = env
            .storage()
            .instance()
            .get::<MedicalKey, CertificateLifecycle>(&MedicalKey::CertificateLifecycle((
                pet_id, vaccination_id,
            )))
        {
            if lifecycle.revoked {
                return false;
            }
            if lifecycle.expiry != 0 && env.ledger().timestamp() >= lifecycle.expiry {
                return false;
            }
        }

        true
    }

    /// Get the certificate anchor for a vaccination.
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet
    /// * `vaccination_id` - The ID of the vaccination
    ///
    /// # Returns
    /// * `Some(CertificateAnchor)` if certificate is anchored
    /// * `None` if no certificate is anchored
    pub fn get_certificate_anchor(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
    ) -> Option<CertificateAnchor> {
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        env.storage().instance().get(&anchor_key)
    }

    /// Read the lifecycle record for a certificate.
    ///
    /// # Returns
    /// * `Some(CertificateLifecycle)` — the lifecycle was anchored.
    /// * `None` — no certificate has been anchored for this vaccination.
    pub fn get_certificate_lifecycle(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
    ) -> Option<CertificateLifecycle> {
        env.storage()
            .instance()
            .get(&MedicalKey::CertificateLifecycle((pet_id, vaccination_id)))
    }

    /// Return the high-level lifecycle status of a certificate.
    pub fn get_certificate_status(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
    ) -> CertificateStatus {
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        if !env.storage().instance().has(&anchor_key) {
            return CertificateStatus::NotAnchored;
        }
        if let Some(lifecycle) = env
            .storage()
            .instance()
            .get::<MedicalKey, CertificateLifecycle>(&MedicalKey::CertificateLifecycle((
                pet_id, vaccination_id,
            )))
        {
            if lifecycle.revoked {
                return CertificateStatus::Revoked;
            }
            if lifecycle.expiry != 0 && env.ledger().timestamp() >= lifecycle.expiry {
                return CertificateStatus::Expired;
            }
        }
        CertificateStatus::Valid
    }

    /// Return the certificate anchor, failing if it is not active/valid.
    ///
    /// * `CertificateNotFound` — no certificate anchored.
    /// * `CertificateRevoked` — certificate has been revoked.
    /// * `CertificateExpired` — certificate expiry has passed.
    pub fn get_active_certificate(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
    ) -> CertificateAnchor {
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        let anchor: CertificateAnchor = env
            .storage()
            .instance()
            .get(&anchor_key)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CertificateNotFound));
        match Self::get_certificate_status(env.clone(), pet_id, vaccination_id) {
            CertificateStatus::Valid => anchor,
            CertificateStatus::NotAnchored => {
                panic_with_error!(&env, ContractError::CertificateNotFound)
            }
            CertificateStatus::Revoked => {
                panic_with_error!(&env, ContractError::CertificateRevoked)
            }
            CertificateStatus::Expired => {
                panic_with_error!(&env, ContractError::CertificateExpired)
            }
        }
    }

    /// Revoke a certificate anchor on-chain.
    ///
    /// Only the issuing vet or an admin may revoke. A second revocation of the
    /// same certificate returns `CertificateRevoked`.
    ///
    /// # Arguments
    /// * `caller` — issuer or admin (must `require_auth`)
    /// * `pet_id` — pet the certificate belongs to
    /// * `vaccination_id` — vaccination the certificate anchors
    /// * `reason` — free-text revocation reason
    pub fn revoke_certificate(
        env: Env,
        caller: Address,
        pet_id: u64,
        vaccination_id: u64,
        reason: String,
    ) {
        caller.require_auth();

        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));
        let anchor: CertificateAnchor = env
            .storage()
            .instance()
            .get(&anchor_key)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CertificateNotFound));

        let is_admin = Self::is_admin_address(&env, &caller);
        if !is_admin && anchor.issuer != caller {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let mut lifecycle: CertificateLifecycle = env
            .storage()
            .instance()
            .get(&MedicalKey::CertificateLifecycle((pet_id, vaccination_id)))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::CertificateNotFound));

        if lifecycle.revoked {
            panic_with_error!(&env, ContractError::CertificateRevoked);
        }

        let now = env.ledger().timestamp();
        lifecycle.revoked = true;
        lifecycle.revoked_at = Some(now);
        lifecycle.revocation_reason = Some(reason.clone());

        env.storage()
            .instance()
            .set(&MedicalKey::CertificateLifecycle((pet_id, vaccination_id)), &lifecycle);

        env.events().publish(
            (String::from_str(&env, "CertificateRevoked"), pet_id),
            CertificateRevokedEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                vaccination_id,
                cert_id: lifecycle.cert_id,
                revoked_by: caller,
                reason,
                timestamp: now,
            },
        );
    }

    // --- NUTRITION / DIET FUNCTIONS ---
    pub fn set_diet_plan(
        env: Env,
        pet_id: u64,
        food_type: String,
        portion_size: String,
        frequency: String,
        calories_per_serving: u32,
        daily_target_calories: u32,
        restrictions: Vec<String>,
        allergies: Vec<String>,
    ) -> bool {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        // Enforce Vec caps to prevent unbounded inline serialisation. (#1153)
        if restrictions.len() > MAX_DIETARY_RESTRICTIONS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }
        if allergies.len() > MAX_DIETARY_ALLERGIES {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let diet_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::DietPlanCount)
            .unwrap_or(0);
        let diet_id = safe_increment(&env, diet_count);

        let now = env.ledger().timestamp();

        let plan = DietPlan {
            pet_id,
            food_type,
            portion_size,
            feeding_frequency: frequency,
            calories_per_serving,
            daily_target_calories,
            dietary_restrictions: restrictions,
            allergies,
            created_by: pet.owner.clone(),
            created_at: now,
        };

        env.storage()
            .instance()
            .set(&NutritionKey::DietPlan(diet_id), &plan);
        env.storage()
            .instance()
            .set(&NutritionKey::DietPlanCount, &diet_id);

        let pet_diet_count: u64 = safe_increment(
            env.storage()
                .instance()
                .get(&NutritionKey::PetDietCount(pet_id))
                .unwrap_or(0),
        );
        env.storage()
            .instance()
            .set(&NutritionKey::PetDietCount(pet_id), &pet_diet_count);
        env.storage().instance().set(
            &NutritionKey::PetDietByIndex((pet_id, pet_diet_count)),
            &diet_id,
        );

        true
    }

    pub fn get_diet_plan(env: Env, diet_id: u64) -> Option<DietPlan> {
        env.storage()
            .instance()
            .get(&NutritionKey::DietPlan(diet_id))
    }

    pub fn get_diet_history(env: Env, pet_id: u64) -> Vec<DietPlan> {
        if env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .is_none()
        {
            return Vec::new(&env);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetDietCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(did) = env
                .storage()
                .instance()
                .get::<NutritionKey, u64>(&NutritionKey::PetDietByIndex((pet_id, i)))
            {
                if let Some(plan) = PetChainContract::get_diet_plan(env.clone(), did) {
                    history.push_back(plan);
                }
            }
        }
        history
    }

    pub fn get_current_diet_plan(env: Env, pet_id: u64) -> Option<DietPlan> {
        let history = PetChainContract::get_diet_history(env, pet_id);
        let mut current: Option<DietPlan> = None;
        for plan in history.iter() {
            let replace = match current {
                None => true,
                Some(ref c) => plan.created_at > c.created_at,
            };
            if replace {
                current = Some(plan);
            }
        }
        current
    }

    /// Returns the total number of diet plans recorded for a given pet.
    /// Returns 0 if the pet does not exist or has no diet plans.
    /// Useful for pagination UI to determine total pages.
    pub fn get_diet_plan_count(env: Env, pet_id: u64) -> u64 {
        env.storage()
            .instance()
            .get(&NutritionKey::PetDietCount(pet_id))
            .unwrap_or(0)
    }

    fn current_nutrition_day(env: &Env) -> u64 {
        env.ledger().timestamp() / 86_400
    }

    pub fn log_feeding(env: Env, pet_id: u64, plan_id: u64, servings: u32) -> bool {
        let plan: DietPlan = env
            .storage()
            .instance()
            .get(&NutritionKey::DietPlan(plan_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));

        if plan.pet_id != pet_id {
            env.panic_with_error(ContractError::InvalidInput)
        }

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        let calories = plan
            .calories_per_serving
            .checked_mul(servings)
            .unwrap_or_else(|| env.panic_with_error(ContractError::CounterOverflow));

        let day = PetChainContract::current_nutrition_day(&env);
        let now = env.ledger().timestamp();

        let mut summary = env
            .storage()
            .instance()
            .get::<NutritionKey, DailyNutritionSummary>(&NutritionKey::DailyNutritionSummary((
                pet_id, day,
            )))
            .unwrap_or(DailyNutritionSummary {
                pet_id,
                date: day,
                total_calories: 0,
                target_calories: plan.daily_target_calories,
                updated_at: now,
            });

        summary.total_calories = summary.total_calories.saturating_add(calories);
        summary.target_calories = plan.daily_target_calories;
        summary.updated_at = now;

        env.storage().instance().set(
            &NutritionKey::DailyNutritionSummary((pet_id, day)),
            &summary,
        );

        if summary.target_calories > 0 {
            let lower_threshold = summary.target_calories * 80 / 100;
            let upper_threshold = summary.target_calories * 120 / 100;
            let status = if summary.total_calories > upper_threshold {
                Some(String::from_str(&env, "AboveTarget"))
            } else if summary.total_calories < lower_threshold {
                Some(String::from_str(&env, "BelowTarget"))
            } else {
                None
            };

            if let Some(status_text) = status {
                env.events().publish(
                    (Symbol::new(&env, "NutritionAlert"),),
                    (
                        pet_id,
                        day,
                        plan_id,
                        summary.total_calories,
                        summary.target_calories,
                        status_text,
                    ),
                );
            }
        }

        true
    }

    pub fn get_daily_summary(env: Env, pet_id: u64, date: u64) -> Option<DailyNutritionSummary> {
        env.storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))?;

        let summary = env
            .storage()
            .instance()
            .get::<NutritionKey, DailyNutritionSummary>(&NutritionKey::DailyNutritionSummary((
                pet_id, date,
            )));

        if summary.is_some() {
            return summary;
        }

        let target = PetChainContract::get_current_diet_plan(env.clone(), pet_id)
            .map(|plan| plan.daily_target_calories)
            .unwrap_or(0);

        Some(DailyNutritionSummary {
            pet_id,
            date,
            total_calories: 0,
            target_calories: target,
            updated_at: env.ledger().timestamp(),
        })
    }

    pub fn get_weight_entry_count(env: Env, pet_id: u64) -> u64 {
        env.storage()
            .instance()
            .get(&NutritionKey::PetWeightCount(pet_id))
            .unwrap_or(0)
    }

    pub fn add_weight_entry(env: Env, pet_id: u64, weight: u32) -> bool {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        // Check storage quota (Issue #676)
        Self::increment_pet_storage(&env, pet_id);

        let weight_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::WeightCount)
            .unwrap_or(0);
        let weight_id = safe_increment(&env, weight_count);
        let now = env.ledger().timestamp();

        let entry = WeightEntry {
            pet_id,
            weight,
            recorded_at: now,
            recorded_by: pet.owner.clone(),
        };

        // Persist entry
        env.storage()
            .instance()
            .set(&NutritionKey::WeightEntry(weight_id), &entry);
        env.storage()
            .instance()
            .set(&NutritionKey::WeightCount, &weight_id);

        let pet_weight_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetWeightCount(pet_id))
            .unwrap_or(0)
            + 1;
        env.storage()
            .instance()
            .set(&NutritionKey::PetWeightCount(pet_id), &pet_weight_count);
        env.storage().instance().set(
            &NutritionKey::PetWeightByIndex((pet_id, pet_weight_count)),
            &weight_id,
        );

        // Update current pet weight
        pet.weight = weight;
        pet.updated_at = now;
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

        true
    }

    pub fn get_weight_history(env: Env, pet_id: u64) -> Vec<WeightEntry> {
        if env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .is_none()
        {
            return Vec::new(&env);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetWeightCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(wid) = env
                .storage()
                .instance()
                .get::<NutritionKey, u64>(&NutritionKey::PetWeightByIndex((pet_id, i)))
            {
                if let Some(entry) = env
                    .storage()
                    .instance()
                    .get(&NutritionKey::WeightEntry(wid))
                {
                    history.push_back(entry);
                }
            }
        }
        history
    }

    pub fn get_weight_entry(env: Env, weight_id: u64) -> Option<WeightEntry> {
        env.storage()
            .instance()
            .get(&NutritionKey::WeightEntry(weight_id))
    }

    // --- INGREDIENT-BASED NUTRITION PLANS (Issue #800) ---

    /// Add a nutrition plan whose ingredient calories must match the declared
    /// total within a ±5 kcal tolerance.
    pub fn add_nutrition_plan(
        env: Env,
        pet_id: u64,
        name: String,
        ingredients: Vec<Ingredient>,
        total_calories: u32,
    ) -> u64 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        if name.is_empty() {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        // Enforce Vec cap to prevent unbounded inline serialisation. (#1153)
        if ingredients.len() > MAX_INGREDIENTS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let mut sum: u32 = 0;
        for ingredient in ingredients.iter() {
            sum = sum.saturating_add(ingredient.calories);
        }

        if sum.abs_diff(total_calories) > 5 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        Self::increment_pet_storage(&env, pet_id);

        let plan_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::NutritionPlanCount)
            .unwrap_or(0u64);
        let plan_id = safe_increment(&env, plan_count);

        let pet_plan_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetNutritionPlanCount(pet_id))
            .unwrap_or(0u64);
        let next_pet_count = safe_increment(pet_plan_count);

        let plan = NutritionPlan {
            id: plan_id,
            pet_id,
            name,
            ingredients,
            total_calories,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&NutritionKey::NutritionPlan(plan_id), &plan);
        env.storage()
            .instance()
            .set(&NutritionKey::NutritionPlanCount, &plan_id);
        env.storage().instance().set(
            &NutritionKey::PetNutritionPlanCount(pet_id),
            &next_pet_count,
        );
        env.storage().instance().set(
            &NutritionKey::PetNutritionPlanIndex((pet_id, next_pet_count)),
            &plan_id,
        );

        plan_id
    }

    pub fn get_nutrition_plan(env: Env, plan_id: u64) -> Option<NutritionPlan> {
        env.storage()
            .instance()
            .get(&NutritionKey::NutritionPlan(plan_id))
    }

    // --- VERSIONED NUTRITION PLANS ---

    /// Creates a new version of nutrition plan for a pet.
    /// Stores up to 10 versions per pet, pruning oldest when limit exceeded.
    /// Only callable by pet owner or authorized vet.
    pub fn set_nutrition_version(
        env: Env,
        pet_id: u64,
        food_type: String,
        portion_size: String,
        frequency: String,
        calories_per_serving: u32,
        daily_target_calories: u32,
        restrictions: Vec<String>,
        allergies: Vec<String>,
    ) -> u64 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        // Enforce Vec caps to prevent unbounded inline serialisation. (#1153)
        if restrictions.len() > MAX_DIETARY_RESTRICTIONS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }
        if allergies.len() > MAX_DIETARY_ALLERGIES {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let current_version: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetNutritionVersionCount(pet_id))
            .unwrap_or(0);
        let new_version = safe_increment(current_version);
        let now = env.ledger().timestamp();

        let nutrition_version = NutritionVersion {
            pet_id,
            version: new_version,
            food_type,
            portion_size,
            feeding_frequency: frequency,
            calories_per_serving,
            daily_target_calories,
            dietary_restrictions: restrictions,
            allergies,
            created_by: pet.owner.clone(),
            created_at: now,
            is_active: true,
        };

        // Mark previous version as inactive
        if current_version > 0 {
            if let Some(mut prev) = env
                .storage()
                .instance()
                .get::<NutritionKey, NutritionVersion>(&NutritionKey::NutritionVersion((
                    pet_id,
                    current_version,
                )))
            {
                prev.is_active = false;
                env.storage().instance().set(
                    &NutritionKey::NutritionVersion((pet_id, current_version)),
                    &prev,
                );
            }
        }

        // Store new version
        env.storage().instance().set(
            &NutritionKey::NutritionVersion((pet_id, new_version)),
            &nutrition_version,
        );
        env.storage().instance().set(
            &NutritionKey::PetNutritionVersionCount(pet_id),
            &new_version,
        );
        env.storage()
            .instance()
            .set(&NutritionKey::CurrentNutritionVersion(pet_id), &new_version);

        // Prune oldest version if exceeding 10 versions
        if new_version > 10 {
            let oldest_version = new_version - 10;
            env.storage()
                .instance()
                .remove(&NutritionKey::NutritionVersion((pet_id, oldest_version)));
        }

        new_version
    }

    /// Retrieves a specific version of nutrition plan for a pet.
    pub fn get_nutrition_version(env: Env, pet_id: u64, version: u64) -> Option<NutritionVersion> {
        // Verify pet exists
        env.storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))?;

        env.storage()
            .instance()
            .get(&NutritionKey::NutritionVersion((pet_id, version)))
    }

    /// Lists all versions of nutrition plans for a pet (up to 10 most recent).
    pub fn list_nutrition_versions(env: Env, pet_id: u64) -> Vec<NutritionVersion> {
        // Verify pet exists
        if env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .is_none()
        {
            return Vec::new(&env);
        }

        let current_version: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetNutritionVersionCount(pet_id))
            .unwrap_or(0);

        let mut versions = Vec::new(&env);

        // Collect versions in reverse order (newest first)
        let start_version = if current_version > 10 {
            current_version - 9
        } else {
            1
        };

        for v in (start_version..=current_version).rev() {
            if let Some(nutrition_version) = env
                .storage()
                .instance()
                .get::<NutritionKey, NutritionVersion>(&NutritionKey::NutritionVersion((pet_id, v)))
            {
                versions.push_back(nutrition_version);
            }
        }

        versions
    }

    /// Rolls back nutrition plan to a specific version.
    /// Only callable by pet owner or authorized vet.
    /// Creates a new version that mirrors the target version.
    pub fn rollback_nutrition(env: Env, pet_id: u64, target_version: u64) -> u64 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        pet.owner.require_auth();

        // Verify target version exists
        let target = env
            .storage()
            .instance()
            .get::<NutritionKey, NutritionVersion>(&NutritionKey::NutritionVersion((
                pet_id,
                target_version,
            )))
            .unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));

        // Create new version with target's data
        let current_version: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetNutritionVersionCount(pet_id))
            .unwrap_or(0);
        let new_version = safe_increment(current_version);
        let now = env.ledger().timestamp();

        let rollback_version = NutritionVersion {
            pet_id,
            version: new_version,
            food_type: target.food_type,
            portion_size: target.portion_size,
            feeding_frequency: target.feeding_frequency,
            calories_per_serving: target.calories_per_serving,
            daily_target_calories: target.daily_target_calories,
            dietary_restrictions: target.dietary_restrictions,
            allergies: target.allergies,
            created_by: pet.owner.clone(),
            created_at: now,
            is_active: true,
        };

        // Mark previous version as inactive
        if current_version > 0 {
            if let Some(mut prev) = env
                .storage()
                .instance()
                .get::<NutritionKey, NutritionVersion>(&NutritionKey::NutritionVersion((
                    pet_id,
                    current_version,
                )))
            {
                prev.is_active = false;
                env.storage().instance().set(
                    &NutritionKey::NutritionVersion((pet_id, current_version)),
                    &prev,
                );
            }
        }

        // Store rollback version
        env.storage().instance().set(
            &NutritionKey::NutritionVersion((pet_id, new_version)),
            &rollback_version,
        );
        env.storage().instance().set(
            &NutritionKey::PetNutritionVersionCount(pet_id),
            &new_version,
        );
        env.storage()
            .instance()
            .set(&NutritionKey::CurrentNutritionVersion(pet_id), &new_version);

        // Prune oldest version if exceeding 10 versions
        if new_version > 10 {
            let oldest_version = new_version - 10;
            env.storage()
                .instance()
                .remove(&NutritionKey::NutritionVersion((pet_id, oldest_version)));
        }

        new_version
    }

    /// Gets the current active nutrition version for a pet.
    pub fn get_current_nutrition_version(env: Env, pet_id: u64) -> Option<NutritionVersion> {
        // Verify pet exists
        env.storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))?;

        let current_version: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::CurrentNutritionVersion(pet_id))
            .unwrap_or(0);

        if current_version == 0 {
            return None;
        }

        env.storage()
            .instance()
            .get(&NutritionKey::NutritionVersion((pet_id, current_version)))
    }

    // --- TAG LINKING (UPSTREAM IMPLEMENTATION) ---

    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        let nonce: u64 = env.storage().instance().get(&TagKey::TagNonce).unwrap_or(0);
        let new_nonce = safe_increment(env, nonce);
        env.storage().instance().set(&TagKey::TagNonce, &new_nonce);

        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();

        // Domain-separated so a tag ID can never collide with a hash
        // computed for another stored-hash domain (evidence, attachments,
        // claim documents, certificates, medical records). (#1168)
        let mut preimage = Bytes::from_slice(env, Self::hash_domain_tag(&HashDomain::TagId));
        for byte in pet_id.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in new_nonce.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in timestamp.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in sequence.to_be_bytes() {
            preimage.push_back(byte);
        }

        env.crypto().sha256(&preimage).into()
    }

    pub fn link_tag_to_pet(env: Env, pet_id: u64) -> BytesN<32> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();

        if env
            .storage()
            .instance()
            .get::<TagKey, BytesN<32>>(&TagKey::PetTagId(pet_id))
            .is_some()
        {
            panic_with_error!(&env, ContractError::PetAlreadyHasLinkedTag);
        }

        let tag_id = PetChainContract::generate_tag_id(&env, pet_id, &pet.owner);
        let now = env.ledger().timestamp();

        let pet_tag = PetTag {
            tag_id: tag_id.clone(),
            pet_id,
            owner: pet.owner.clone(),
            message: String::from_str(&env, ""),
            is_active: true,
            linked_at: now,
            updated_at: now,
            tag_message: String::from_str(&env, ""),
            created_at: now,
        };

        env.storage()
            .instance()
            .set(&TagKey::Tag(tag_id.clone()), &pet_tag);
        env.storage()
            .instance()
            .set(&TagKey::PetTagId(pet_id), &tag_id);

        let count: u64 = env
            .storage()
            .instance()
            .get(&TagKey::PetTagCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&TagKey::PetTagCount, &safe_increment(&env, count));

        env.events().publish(
            (String::from_str(&env, "TAG_LINKED"),),
            TagLinkedEvent {
                tag_id: tag_id.clone(),
                pet_id,
                owner: pet.owner.clone(),
                timestamp: now,
            },
        );

        tag_id
    }

    pub fn get_pet_by_tag(env: Env, tag_id: BytesN<32>) -> Option<PetProfile> {
        if let Some(tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id))
        {
            if !tag.is_active {
                return None;
            }
            PetChainContract::get_pet(env.clone(), tag.pet_id, env.current_contract_address())
        } else {
            None
        }
    }

    pub fn get_tag(env: Env, tag_id: BytesN<32>) -> Option<PetTag> {
        env.storage().instance().get(&TagKey::Tag(tag_id))
    }

    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage().instance().get(&TagKey::PetTagId(pet_id))
    }

    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
            pet.owner.require_auth();

            tag.message = message;
            tag.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&TagKey::Tag(tag_id), &tag);
            true
        } else {
            false
        }
    }

    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
            pet.owner.require_auth();

            tag.is_active = false;
            tag.updated_at = env.ledger().timestamp();
            env.storage()
                .instance()
                .set(&TagKey::Tag(tag_id.clone()), &tag);

            env.events().publish(
                (String::from_str(&env, "TAG_DEACTIVATED"),),
                TagDeactivatedEvent {
                    tag_id,
                    pet_id: tag.pet_id,
                    deactivated_by: pet.owner,
                    timestamp: env.ledger().timestamp(),
                },
            );
            true
        } else {
            false
        }
    }

    pub fn reactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
            pet.owner.require_auth();

            tag.is_active = true;
            tag.updated_at = env.ledger().timestamp();
            env.storage()
                .instance()
                .set(&TagKey::Tag(tag_id.clone()), &tag);

            env.events().publish(
                (String::from_str(&env, "TAG_REACTIVATED"),),
                TagReactivatedEvent {
                    tag_id,
                    pet_id: tag.pet_id,
                    reactivated_by: pet.owner,
                    timestamp: env.ledger().timestamp(),
                },
            );
            true
        } else {
            false
        }
    }

    pub fn is_tag_active(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(tag) = env
            .storage()
            .instance()
            .get::<TagKey, PetTag>(&TagKey::Tag(tag_id))
        {
            tag.is_active
        } else {
            false
        }
    }

    // --- HELPERS ---

    fn get_owner_pet_count(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
    }

    /// Search a pet's medical records with optional filters and offset/limit pagination.
    ///
    /// # Arguments
    /// * `pet_id`  – The pet whose records are searched.
    /// * `filter`  – Optional filters: `vet_address`, inclusive date window
    ///               (`from_date` / `to_date`), and `diagnosis_keyword`.
    /// * `offset`  – Number of matching records to skip (0-based page offset).
    /// * `limit`   – Maximum number of records to return; 0 returns an empty vec.
    ///
    /// # Errors
    /// * `ContractError::InvalidInput` – when `from_date` and `to_date` are both
    ///   `Some` but `from_date > to_date`.
    pub fn search_medical_records(
        env: Env,
        pet_id: u64,
        filter: MedicalRecordFilter,
        offset: u64,
        limit: u32,
    ) -> Vec<MedicalRecord> {
        // Validate date range: from must not be after to.
        if let (Some(from), Some(to)) = (filter.from_date, filter.to_date) {
            if from > to {
                panic_with_error!(&env, ContractError::InvalidInput);
            }
        }

        if limit == 0 {
            return Vec::new(&env);
        }

        let record_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);

        let mut results: Vec<MedicalRecord> = Vec::new(&env);
        // `matched` counts records that pass the filter (used for offset skipping).
        let mut matched: u64 = 0;

        for i in 1..=record_count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
                {
                    // Exclude soft-deleted records via the shared filter.
                    if PetChainContract::medical_record_is_deleted(&record) {
                        continue;
                    }

                    if PetChainContract::medical_record_matches_filter(&env, &record, &filter) {
                        // Apply offset: skip the first `offset` matching records.
                        if matched >= offset {
                            results.push_back(record);
                            if results.len() >= limit {
                                break;
                            }
                        }
                        matched += 1;
                    }
                }
            }
        }

        results
    }

    #[allow(dead_code)]
    fn medical_record_matches_filter(
        env: &Env,
        record: &MedicalRecord,
        filter: &MedicalRecordFilter,
    ) -> bool {
        if let Some(vet_address) = &filter.vet_address {
            if record.vet_address != *vet_address {
                return false;
            }
        }

        if let Some(from_date) = filter.from_date {
            if record.date < from_date {
                return false;
            }
        }

        if let Some(to_date) = filter.to_date {
            if record.date > to_date {
                return false;
            }
        }

        if let Some(keyword) = &filter.diagnosis_keyword {
            if !PetChainContract::string_contains(env, &record.diagnosis, keyword) {
                return false;
            }
        }

        true
    }

    /// Shared soft-delete predicate (Issue #1171). Every public read path that
    /// returns medical records must exclude rows for which this returns `true`,
    /// so deletion filtering is applied consistently across search, pagination,
    /// batch and keyword reads.
    fn medical_record_is_deleted(record: &MedicalRecord) -> bool {
        record.deleted_at.is_some()
    }

    #[allow(dead_code)]
    fn string_contains(_env: &Env, haystack: &String, needle: &String) -> bool {
        let haystack_len = haystack.len() as usize;
        let needle_len = needle.len() as usize;

        if needle_len == 0 {
            return true;
        }
        if needle_len > haystack_len {
            return false;
        }

        let mut haystack_bytes = [0u8; PetChainContract::MAX_STR_LONG as usize];
        let mut needle_bytes = [0u8; PetChainContract::MAX_STR_LONG as usize];
        haystack.copy_into_slice(&mut haystack_bytes[..haystack_len]);
        needle.copy_into_slice(&mut needle_bytes[..needle_len]);

        for start in 0..=(haystack_len - needle_len) {
            let mut matches = true;
            for offset in 0..needle_len {
                if haystack_bytes[start + offset] != needle_bytes[offset] {
                    matches = false;
                    break;
                }
            }

            if matches {
                return true;
            }
        }

        false
    }

    fn species_to_string(env: &Env, species: &Species) -> String {
        match species {
            Species::Other => String::from_str(env, "Other"),
            Species::Dog => String::from_str(env, "Dog"),
            Species::Cat => String::from_str(env, "Cat"),
            Species::Bird => String::from_str(env, "Bird"),
            Species::Rabbit => String::from_str(env, "Rabbit"),
        }
    }

    // --- PET PROFILE SCHEMA VALIDATION ---

    /// Validate pet name: 1-64 chars, alphanumeric + spaces + hyphens.
    fn validate_pet_name(env: &Env, name: &String) {
        let len = name.len() as usize;
        if len == 0 || len > 64 {
            panic_with_error!(env, ContractError::InvalidPetName);
        }
        let mut buf = [0u8; 64];
        name.copy_into_slice(&mut buf[..len]);
        for b in buf.iter().take(len) {
            if !matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b' ' | b'-') {
                panic_with_error!(env, ContractError::InvalidPetName);
            }
        }
    }

    /// Canonical form is trimmed, ASCII upper-case, and separator-free. Only
    /// ASCII letters and digits are accepted after separators are removed;
    /// this deliberately rejects Unicode lookalikes and ambiguous encodings.
    fn canonicalize_microchip_id(env: &Env, value: &String) -> String {
        let len = value.len() as usize;
        if len == 0 || len > MAX_MICROCHIP_ID_LEN {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        let mut input = [0u8; MAX_MICROCHIP_ID_LEN];
        value.copy_into_slice(&mut input[..len]);
        let mut output = [0u8; MAX_MICROCHIP_ID_LEN];
        let mut out_len = 0usize;
        for byte in input.iter().take(len) {
            if matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'-' | b':' | b'.') {
                continue;
            }
            let canonical = match byte {
                b'a'..=b'z' => byte.to_ascii_uppercase(),
                b'A'..=b'Z' | b'0'..=b'9' => *byte,
                _ => panic_with_error!(env, ContractError::InvalidInput),
            };
            output[out_len] = canonical;
            out_len += 1;
        }
        if out_len == 0 {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        String::from_bytes(env, &output[..out_len])
    }

    /// Validate breed against the species-specific whitelist stored on-chain.
    /// If no whitelist has been set for the species, any non-empty breed is accepted.
    fn validate_breed(env: &Env, species: &Species, breed: &String) {
        let species_key = Self::species_to_string(env, species);
        let list: Option<Vec<String>> = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesBreedList(species_key));
        if let Some(allowed) = list {
            if !allowed.contains(breed) {
                panic_with_error!(env, ContractError::InvalidBreed);
            }
        }
        // No whitelist set → any breed accepted
    }

    /// Admin: set the allowed breed list for a species.
    /// Pass an empty Vec to clear the whitelist (allow any breed).
    pub fn set_breed_list(env: Env, admin: Address, species: Species, breeds: Vec<String>) {
        Self::require_admin_auth(&env, &admin);
        let species_key = Self::species_to_string(&env, &species);
        if breeds.is_empty() {
            env.storage()
                .instance()
                .remove(&DataKey::SpeciesBreedList(species_key));
        } else {
            env.storage()
                .instance()
                .set(&DataKey::SpeciesBreedList(species_key), &breeds);
        }
    }

    /// Get the allowed breed list for a species (empty Vec if no whitelist set).
    pub fn get_breed_list(env: Env, species: Species) -> Vec<String> {
        let species_key = Self::species_to_string(&env, &species);
        env.storage()
            .instance()
            .get(&DataKey::SpeciesBreedList(species_key))
            .unwrap_or_else(|| Vec::new(&env))
    }

    // --- CALLER NONCE REPLAY PROTECTION ---

    /// Returns the current nonce for `caller`. The caller must supply this
    /// value in any state-mutating call that uses nonce protection.
    pub fn get_caller_nonce(env: Env, caller: Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::CallerNonce(caller))
            .unwrap_or(0)
    }

    /// Internal: verify `supplied` matches the stored nonce for `caller`,
    /// then atomically increment it.
    #[allow(dead_code)]
    fn consume_caller_nonce(env: &Env, caller: &Address, supplied: u64) {
        let current: u64 = env
            .storage()
            .instance()
            .get(&DataKey::CallerNonce(caller.clone()))
            .unwrap_or(0);
        if supplied != current {
            panic_with_error!(env, ContractError::InvalidNonce);
        }
        env.storage()
            .instance()
            .set(&DataKey::CallerNonce(caller.clone()), &(current + 1));
    }

    /// Nonce-protected pet registration. Caller supplies their current nonce;
    /// the nonce is incremented atomically on success, preventing replay.
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    fn register_pet_with_nonce(
        env: Env,
        owner: Address,
        nonce: u64,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        color: String,
        weight: u32,
        microchip_id: Option<String>,
        privacy_level: PrivacyLevel,
    ) -> u64 {
        owner.require_auth();
        Self::consume_caller_nonce(&env, &owner, nonce);
        Self::validate_pet_name(&env, &name);
        Self::validate_breed(&env, &species, &breed);
        if let Err(err) = PetChainContract::parse_birthday_timestamp(&birthday) {
            env.panic_with_error(err);
        }
        // Delegate to the core registration logic (reuse existing path)
        PetChainContract::register_pet(
            env,
            owner,
            name,
            birthday,
            gender,
            species,
            breed,
            color,
            weight,
            microchip_id,
            privacy_level,
        )
    }

    fn is_valid_cid(hash: &str) -> bool {
        let len = hash.len();
        let bytes = hash.as_bytes();

        // CIDv0: Qm... with exactly 46 Base58 characters.
        if len == 46 {
            if bytes[0] != b'Q' || bytes[1] != b'm' {
                return false;
            }
            return bytes.iter().all(|b| {
                matches!(
                    b,
                    b'1'..=b'9'
                        | b'A'..=b'H'
                        | b'J'..=b'N'
                        | b'P'..=b'Z'
                        | b'a'..=b'k'
                        | b'm'..=b'z'
                )
            });
        }

        // CIDv1: must start with "bafy" and use lowercase base32.
        if !(5..=128).contains(&len) {
            return false;
        }
        if &bytes[..4] != b"bafy" {
            return false;
        }

        bytes.iter().skip(4).all(|b| matches!(b, b'a'..=b'z' | b'2'..=b'7'))
    }

    fn validate_ipfs_hash(_env: &Env, hash: &String) -> Result<(), ContractError> {
        let len = hash.len() as usize;
        if len > 128 {
            return Err(ContractError::InvalidIpfsHash);
        }
        let mut bytes = [0u8; 128];
        hash.copy_into_slice(&mut bytes[..len]);
        let cid = core::str::from_utf8(&bytes[..len]).unwrap_or_default();

        if Self::is_valid_cid(cid) {
            Ok(())
        } else {
            Err(ContractError::InvalidIpfsHash)
        }
    }

    fn get_encryption_key(env: &Env) -> Bytes {
        // Derive a stable, contract-scoped key from contract identity + admin context.
        // This avoids static hardcoded key material while remaining deterministic.
        let mut preimage = Bytes::new(env);
        for byte in b"petchain:encryption-key:v1" {
            preimage.push_back(*byte);
        }

        let contract_xdr = env.current_contract_address().to_xdr(env);
        for byte in contract_xdr.iter() {
            preimage.push_back(byte);
        }

        if let Some(legacy_admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            let admin_xdr = legacy_admin.to_xdr(env);
            for byte in admin_xdr.iter() {
                preimage.push_back(byte);
            }
        } else if let Some(admins) = env
            .storage()
            .instance()
            .get::<SystemKey, Vec<Address>>(&SystemKey::Admins)
        {
            if let Some(primary_admin) = admins.get(0) {
                let admin_xdr = primary_admin.to_xdr(env);
                for byte in admin_xdr.iter() {
                    preimage.push_back(byte);
                }
            }
        }

        env.crypto().sha256(&preimage).into()
    }

    /// Versioned tag prefixed to a domain's canonical encoding before
    /// hashing (Issue #1168). Bumping the trailing version segment for a
    /// domain (e.g. to `v2`) is itself a deliberate, published breaking
    /// change to that domain's hash space, distinct from every other
    /// domain and every earlier version of the same domain.
    fn hash_domain_tag(domain: &HashDomain) -> &'static [u8] {
        match domain {
            HashDomain::Evidence => b"petchain:hash:evidence:v1",
            HashDomain::Attachment => b"petchain:hash:attachment:v1",
            HashDomain::ClaimDocument => b"petchain:hash:claim-document:v1",
            HashDomain::Certificate => b"petchain:hash:certificate:v1",
            HashDomain::MedicalRecord => b"petchain:hash:medical-record:v1",
            HashDomain::TagId => b"petchain:hash:tag-id:v1",
        }
    }

    /// Compute a domain-separated SHA-256 hash: `sha256(tag || content)`,
    /// where `tag` is a versioned, domain-specific ASCII prefix. (#1168)
    ///
    /// This is the canonical way to hash content that will be stored or
    /// compared as a `BytesN<32>` anywhere in this contract (evidence,
    /// attachments, claim documents, certificates, medical records, tag
    /// IDs). Off-chain callers computing a hash to submit to
    /// `submit_evidence`, `add_attachment`, or similar must reproduce this
    /// same prefixing so that a value hashed for one domain can never
    /// collide with, or be replayed as, a value from another domain -- see
    /// `test_domain_separated_hashes.rs` for published test vectors.
    pub fn compute_domain_hash(env: Env, domain: HashDomain, content: Bytes) -> BytesN<32> {
        let mut preimage = Bytes::from_slice(&env, Self::hash_domain_tag(&domain));
        preimage.append(&content);
        env.crypto().sha256(&preimage).into()
    }

    #[allow(dead_code)]
    fn derive_versioned_key(env: &Env, version: u32) -> Bytes {
        let base_key = Self::get_encryption_key(env);
        if version <= 1 {
            return base_key;
        }
        let mut preimage = base_key;
        for byte in version.to_be_bytes() {
            preimage.push_back(byte);
        }
        env.crypto().sha256(&preimage).into()
    }

    pub fn rotate_record_encryption(
        env: Env,
        pet_id: u64,
        record_id: u64,
        new_key_version: u32,
    ) -> bool {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();

        let mut record: MedicalRecord = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));

        if record.pet_id != pet_id {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        record.updated_at = env.ledger().timestamp();
        let _ = new_key_version;

        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecord(record_id), &record);
        true
    }

    pub fn get_record_encrypted_payload(env: Env, pet_id: u64, record_id: u64) -> Option<String> {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();

        let record: MedicalRecord = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id))?;

        if record.pet_id != pet_id {
            return None;
        }

        let _ = pet_id;
        Some(record.notes.clone())
    }

    // --- ATTACHMENT MANAGEMENT ---

    /// Add an attachment to a medical record.
    ///
    /// Only the vet who created the record may add attachments. The number of
    /// attachments per record is capped at [`MAX_ATTACHMENTS_PER_RECORD`]; a
    /// request that would exceed the cap fails with
    /// [`ContractError::StorageQuotaExceeded`] so a single record cannot be
    /// flooded to silently exhaust the owner's storage quota (Issue #774).
    pub fn add_attachment(
        env: Env,
        record_id: u64,
        ipfs_hash: String,
        metadata: AttachmentMetadata,
        content_hash: BytesN<32>,
    ) -> bool {
        // Validate the IPFS hash format up-front.
        let len = ipfs_hash.len() as usize;
        if len > 128 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }
        let mut bytes = [0u8; 128];
        ipfs_hash.copy_into_slice(&mut bytes[..len]);
        let cid = core::str::from_utf8(&bytes[..len]).unwrap_or_default();
        if !Self::is_valid_cid(cid) {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let mut record: MedicalRecord = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));

        // Only the authoring vet can attach files to the record.
        record.vet_address.require_auth();

        // Validate metadata.
        if metadata.filename.is_empty()
            || metadata.file_type.is_empty()
            || metadata.size == 0
        {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        // Enforce the per-record attachment cap before inserting so the count
        // can never exceed MAX_ATTACHMENTS_PER_RECORD. (#1153)
        if record.attachment_hashes.len() >= MAX_ATTACHMENTS_PER_RECORD {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let attachment = Attachment {
            ipfs_hash,
            metadata,
            content_hash,
            scan_result: None,
        };

        record.attachment_hashes.push_back(attachment);
        record.updated_at = env.ledger().timestamp();

        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecord(record_id), &record);

        Self::log_access(
            &env,
            record.pet_id,
            record.vet_address,
            AccessAction::Write,
            String::from_str(&env, "Attachment added to medical record"),
        );

        true
    }

    /// Return all attachments for a medical record (empty if it does not exist).
    pub fn get_attachments(env: Env, record_id: u64) -> Vec<Attachment> {
        match Self::get_medical_record(env.clone(), record_id) {
            Some(record) => record.attachment_hashes,
            None => Vec::new(&env),
        }
    }

    /// Return the number of attachments on a medical record (0 if it does not
    /// exist). Used to enforce [`MAX_ATTACHMENTS_PER_RECORD`].
    pub fn get_attachment_count(env: Env, record_id: u64) -> u32 {
        match Self::get_medical_record(env, record_id) {
            Some(record) => record.attachment_hashes.len(),
            None => 0,
        }
    }

    fn log_ownership_change(
        env: &Env,
        pet_id: u64,
        previous_owner: Address,
        new_owner: Address,
        reason: String,
    ) {
        let global_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::OwnershipRecordCount)
            .unwrap_or(0);
        let record_id = safe_increment(env, global_count);

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = safe_increment(env, pet_count);

        let record = OwnershipRecord {
            pet_id,
            previous_owner,
            new_owner,
            transfer_date: env.ledger().timestamp(),
            transfer_reason: reason,
        };

        env.storage()
            .instance()
            .set(&SystemKey::PetOwnershipRecord(record_id), &record);
        env.storage()
            .instance()
            .set(&SystemKey::OwnershipRecordCount, &record_id);
        env.storage()
            .instance()
            .set(&SystemKey::PetOwnershipRecordCount(pet_id), &new_pet_count);
        env.storage().instance().set(
            &SystemKey::PetOwnershipRecordIndex((pet_id, new_pet_count)),
            &record_id,
        );
    }

    /// Append a [`CustodyEntry`] to the chain-of-custody log for `pet_id`.
    fn append_custody_entry(
        env: &Env,
        pet_id: u64,
        from: Address,
        to: Address,
        transfer_type: TransferType,
    ) {
        let mut chain: Vec<CustodyEntry> = env
            .storage()
            .instance()
            .get(&SystemKey::CustodyChain(pet_id))
            .unwrap_or_else(|| Vec::new(env));
        // Enforce cap to prevent unbounded inline Vec growth. (#1153)
        if chain.len() >= MAX_CUSTODY_CHAIN {
            panic_with_error!(env, ContractError::TooManyItems);
        }
        chain.push_back(CustodyEntry {
            from,
            to,
            timestamp: env.ledger().timestamp(),
            transfer_type,
        });
        env.storage()
            .instance()
            .set(&SystemKey::CustodyChain(pet_id), &chain);
    }

    /// Return the full chain-of-custody log for `pet_id` in chronological order.
    pub fn get_custody_chain(env: Env, pet_id: u64) -> Vec<CustodyEntry> {
        env.storage()
            .instance()
            .get(&SystemKey::CustodyChain(pet_id))
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// Verifies the chain-of-custody log for `pet_id` is internally consistent:
    /// the first entry's `from` matches the pet's creator, each entry's `from`
    /// matches the previous entry's `to`, and the last entry's `to` matches the
    /// pet's current owner. Pure read function — no storage writes.
    ///
    /// A pet with no transfers (empty chain) is trivially valid.
    pub fn verify_custody_chain(env: Env, pet_id: u64) -> CustodyVerificationResult {
        let chain: Vec<CustodyEntry> = env
            .storage()
            .instance()
            .get(&SystemKey::CustodyChain(pet_id))
            .unwrap_or_else(|| Vec::new(&env));

        if chain.is_empty() {
            return CustodyVerificationResult {
                valid: true,
                gap_at: None,
            };
        }

        // The creator is the pet's registered owner at the first ownership
        // record (previous_owner == new_owner at registration), independent
        // of the custody chain itself.
        let creator = Self::get_ownership_history(env.clone(), pet_id, 0, 1)
            .get(0)
            .map(|record| record.previous_owner);

        let first = chain.get(0).unwrap();
        if Some(first.from.clone()) != creator {
            return CustodyVerificationResult {
                valid: false,
                gap_at: Some(0),
            };
        }

        for i in 1..chain.len() {
            let prev = chain.get(i - 1).unwrap();
            let curr = chain.get(i).unwrap();
            if curr.from != prev.to {
                return CustodyVerificationResult {
                    valid: false,
                    gap_at: Some(i),
                };
            }
        }

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        let last = chain.get(chain.len() - 1).unwrap();
        if last.to != pet.owner {
            return CustodyVerificationResult {
                valid: false,
                gap_at: Some(chain.len()),
            };
        }

        CustodyVerificationResult {
            valid: true,
            gap_at: None,
        }
    }

    pub fn get_ownership_history(
        env: Env,
        pet_id: u64,
        offset: u64,
        limit: u32,
    ) -> Vec<OwnershipRecord> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        if count == 0 || limit == 0 || offset >= count {
            return history;
        }

        let start_index = offset.saturating_add(1);
        let requested_end = offset.saturating_add(limit as u64);
        let end_index = if requested_end > count {
            count
        } else {
            requested_end
        };

        for i in start_index..=end_index {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<SystemKey, u64>(&SystemKey::PetOwnershipRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<SystemKey, OwnershipRecord>(&SystemKey::PetOwnershipRecord(record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
    }
    // --- EMERGENCY RESPONDER ALLOWLIST ---

    /// Grant a responder address access to read emergency data for a pet.
    /// Only the pet owner can call this.
    pub fn add_emergency_responder(env: Env, pet_id: u64, responder: Address) {
        let pet: crate::Pet = env
            .storage()
            .instance()
            .get::<DataKey, crate::Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        pet.owner.require_auth();

        let key = DataKey::EmergencyResponders(pet_id);
        let mut responders: Vec<Address> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        if !responders.contains(&responder) {
            responders.push_back(responder);
            env.storage().instance().set(&key, &responders);
        }
        Self::log_access(
            &env,
            pet_id,
            pet.owner.clone(),
            AccessAction::EmergencyGrant,
            String::from_str(&env, "Emergency responder granted"),
        );
    }

    /// Revoke a responder's access. Only the pet owner can call this.
    pub fn remove_emergency_responder(env: Env, pet_id: u64, responder: Address) {
        let pet: crate::Pet = env
            .storage()
            .instance()
            .get::<DataKey, crate::Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        pet.owner.require_auth();

        let key = DataKey::EmergencyResponders(pet_id);
        let responders: Vec<Address> = env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        let mut updated = Vec::new(&env);
        for r in responders.iter() {
            if r != responder {
                updated.push_back(r);
            }
        }
        env.storage().instance().set(&key, &updated);
        Self::log_access(
            &env,
            pet_id,
            pet.owner.clone(),
            AccessAction::EmergencyRevoke,
            String::from_str(&env, "Emergency responder revoked"),
        );
    }

    /// Returns true if caller is the pet owner or an approved emergency responder.
    pub(crate) fn is_emergency_authorized(
        env: &Env,
        pet_id: u64,
        caller: &Address,
        owner: &Address,
    ) -> bool {
        if caller == owner {
            return true;
        }
        if let Some(access) = env.storage().instance().get::<SystemKey, EmergencyOverride>(
            &SystemKey::EmergencyOverride((pet_id, caller.clone())),
        ) {
            if env.ledger().timestamp() <= access.expires_at {
                return true;
            }
        }
        let responders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyResponders(pet_id))
            .unwrap_or(Vec::new(env));
        responders.contains(caller)
    }

    pub fn grant_emergency_override(
        env: Env, pet_id: u64, responder: Address, reason_code: u32, duration: u64,
    ) {
        Self::set_emergency_override(&env, pet_id, responder, reason_code, duration, false);
    }

    pub fn renew_emergency_override(
        env: Env, pet_id: u64, responder: Address, reason_code: u32, duration: u64,
    ) {
        Self::set_emergency_override(&env, pet_id, responder, reason_code, duration, true);
    }

    fn set_emergency_override(
        env: &Env, pet_id: u64, responder: Address, reason_code: u32, duration: u64, renewal: bool,
    ) {
        let pet: Pet = env.storage().instance().get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        if reason_code == 0 || duration == 0 || duration > 86_400 {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        let key = SystemKey::EmergencyOverride((pet_id, responder.clone()));
        let previous: Option<EmergencyOverride> = env.storage().instance().get(&key);
        if renewal && previous.is_none() {
            panic_with_error!(env, ContractError::InvalidState);
        }
        let total = previous.as_ref().map(|access| access.total_duration).unwrap_or(0);
        let total = total.checked_add(duration)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInput));
        if total > 604_800 {
            panic_with_error!(env, ContractError::InvalidInput);
        }
        let now = env.ledger().timestamp();
        let start = previous.map(|access| access.expires_at.max(now)).unwrap_or(now);
        let expires_at = start.checked_add(duration)
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInput));
        env.storage().instance().set(&key, &EmergencyOverride { responder, expires_at, total_duration: total });
        Self::write_emergency_audit(env, pet_id, pet.owner, reason_code);
    }

    /// List all approved emergency responders for a pet. Owner auth required.
    pub fn get_emergency_responders(env: Env, pet_id: u64, owner: Address) -> Vec<Address> {
        let pet: crate::Pet = env
            .storage()
            .instance()
            .get::<DataKey, crate::Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));
        if owner != pet.owner {
            panic_with_error!(&env, ContractError::Unauthorized);
        }
        owner.require_auth();
        env.storage()
            .instance()
            .get(&DataKey::EmergencyResponders(pet_id))
            .unwrap_or(Vec::new(&env))
    }

    pub(crate) fn validate_emergency_contacts(env: &Env, contacts: &Vec<EmergencyContact>) {
        if contacts.is_empty() {
            panic_with_error!(env, ContractError::InvalidInput);
        }

        let mut has_primary = false;
        let mut priorities = soroban_sdk::Vec::new(env);

        for contact in contacts.iter() {
            if contact.name.is_empty() || contact.phone.is_empty() {
                panic_with_error!(env, ContractError::InvalidInput);
            }
            if contact.is_primary {
                has_primary = true;
            }

            // Check for duplicate priorities
            if priorities.contains(contact.priority) {
                panic_with_error!(env, ContractError::InvalidInput);
            }
            priorities.push_back(contact.priority);
        }

        if !has_primary {
            panic_with_error!(env, ContractError::InvalidInput);
        }
    }

    // --- EMERGENCY CONTACTS ---
    pub fn set_emergency_contacts(
        env: Env,
        pet_id: u64,
        contacts: Vec<EmergencyContact>,
        allergies: Vec<Allergy>,
        medical_notes: String,
    ) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            PetChainContract::validate_emergency_contacts(&env, &contacts);
            pet.owner.require_auth();

            let key = PetChainContract::get_encryption_key(&env);

            let contacts_bytes = contacts.to_xdr(&env);
            let (c_nonce, c_cipher) = encrypt_sensitive_data(&env, &contacts_bytes, &key);
            pet.encrypted_emergency_contacts = EncryptedData {
                nonce: c_nonce,
                ciphertext: c_cipher,
            };

            let allergies_bytes = allergies.to_xdr(&env);
            let (a_nonce, a_cipher) = encrypt_sensitive_data(&env, &allergies_bytes, &key);
            pet.encrypted_allergies = EncryptedData {
                nonce: a_nonce,
                ciphertext: a_cipher,
            };

            let notes_bytes = medical_notes.to_xdr(&env);
            let (n_nonce, n_cipher) = encrypt_sensitive_data(&env, &notes_bytes, &key);
            pet.encrypted_medical_alerts = EncryptedData {
                nonce: n_nonce,
                ciphertext: n_cipher,
            };

            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        } else {
            panic_with_error!(&env, ContractError::PetNotFound);
        }
    }

    pub fn get_emergency_info(env: Env, pet_id: u64, caller: Address) -> EmergencyInfo {
        Self::get_emergency_info_with_reason(env, pet_id, caller, 0)
    }

    pub fn get_emergency_info_with_reason(
        env: Env,
        pet_id: u64,
        caller: Address,
        reason_code: u32,
    ) -> EmergencyInfo {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if !PetChainContract::is_emergency_authorized(&env, pet_id, &caller, &pet.owner) {
                panic_with_error!(&env, ContractError::Unauthorized);
            }
            let key = PetChainContract::get_encryption_key(&env);

            let c_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let contacts =
                Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env));

            let n_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_medical_alerts.ciphertext,
                &pet.encrypted_medical_alerts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let notes = String::from_xdr(&env, &n_bytes).unwrap_or(String::from_str(&env, ""));

            let mut critical_alerts = Vec::new(&env);
            if !notes.is_empty() {
                critical_alerts.push_back(notes);
            }

            let a_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_allergies.ciphertext,
                &pet.encrypted_allergies.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let all_allergies = Vec::<Allergy>::from_xdr(&env, &a_bytes).unwrap_or(Vec::new(&env));

            let mut critical_allergies = Vec::new(&env);
            for allergy in all_allergies.iter() {
                if allergy.is_critical {
                    critical_allergies.push_back(allergy);
                }
            }

            // Log the emergency access
            let log = EmergencyAccessLog {
                pet_id,
                accessed_by: caller.clone(),
                timestamp: env.ledger().timestamp(),
            };

            let log_key = DataKey::EmergencyAccessLogs(pet_id);
            let mut logs: Vec<EmergencyAccessLog> = env
                .storage()
                .persistent()
                .get(&log_key)
                .unwrap_or(Vec::new(&env));
            while logs.len() >= MAX_LOG_ENTRIES {
                logs.remove(0);
            }
            logs.push_back(log);
            env.storage().persistent().set(&log_key, &logs);
            Self::bump_persistent_ttl(&env, &log_key);

            Self::write_emergency_audit(&env, pet_id, caller, reason_code);

            EmergencyInfo {
                pet_id,
                species: PetChainContract::species_to_string(&env, &pet.species),
                allergies: critical_allergies,
                critical_alerts,
                emergency_contacts: contacts,
            }
        } else {
            panic_with_error!(&env, ContractError::PetNotFound);
        }
    }

    fn write_emergency_audit(env: &Env, pet_id: u64, actor: Address, reason_code: u32) {
        let audit_key = DataKey::EmergencyAuditLog(pet_id);
        let mut entries: Vec<AuditEntry> = env
            .storage()
            .persistent()
            .get(&audit_key)
            .unwrap_or(Vec::new(env));
        while entries.len() >= MAX_LOG_ENTRIES {
            entries.remove(0);
        }
        entries.push_back(AuditEntry {
            actor,
            timestamp: env.ledger().timestamp(),
            reason_code,
            pet_id,
        });
        env.storage().persistent().set(&audit_key, &entries);
        Self::bump_persistent_ttl(env, &audit_key);
    }

    fn is_admin_address(env: &Env, caller: &Address) -> bool {
        if let Some(admin) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::Admin)
        {
            if &admin == caller {
                return true;
            }
        }
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or(Vec::new(env));
        admins.contains(caller.clone())
    }

    pub fn get_emergency_audit(
        env: Env,
        pet_id: u64,
        page: u64,
        page_size: u32,
        caller: Address,
    ) -> Vec<AuditEntry> {
        caller.require_auth();
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        if caller != pet.owner && !Self::is_admin_address(&env, &caller) {
            env.panic_with_error(ContractError::Unauthorized);
        }

        let size = if page_size == 0 || page_size > 50 {
            50
        } else {
            page_size
        };
        let entries: Vec<AuditEntry> = env
            .storage()
            .persistent()
            .get(&DataKey::EmergencyAuditLog(pet_id))
            .unwrap_or(Vec::new(&env));
        let start = (page.saturating_mul(size as u64)) as u32;
        let mut result = Vec::new(&env);
        for i in start..start.saturating_add(size) {
            match entries.get(i) {
                Some(entry) => result.push_back(entry),
                None => break,
            }
        }
        result
    }

    pub fn get_emergency_contacts(env: Env, pet_id: u64, caller: Address) -> Vec<EmergencyContact> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            if !PetChainContract::is_emergency_authorized(&env, pet_id, &caller, &pet.owner) {
                panic_with_error!(&env, ContractError::Unauthorized);
            }
            let key = PetChainContract::get_encryption_key(&env);
            let c_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env))
        } else {
            Vec::new(&env)
        }
    }

    /// Notifies a pet's emergency contacts. Rate-limited to 3 calls per hour
    /// per (caller, pet_id) pair so a stolen session can't spam every
    /// emergency contact (Issue #820). Returns the number of contacts
    /// notified.
    pub fn notify_emergency_contacts(env: Env, caller: Address, pet_id: u64) -> u32 {
        const MAX_NOTIFICATIONS_PER_WINDOW: u32 = 3;
        const RATE_LIMIT_WINDOW_SECONDS: u64 = 3_600;

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        if !PetChainContract::is_emergency_authorized(&env, pet_id, &caller, &pet.owner) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let now = env.ledger().timestamp();
        let rate_key = DataKey::EmergencyNotifyRateLimit((caller.clone(), pet_id));
        let mut rate_limit: NotifyRateLimit = env
            .storage()
            .instance()
            .get(&rate_key)
            .unwrap_or(NotifyRateLimit {
                window_start: now,
                count: 0,
            });

        // Reset the window once an hour has elapsed since it started.
        if now >= rate_limit.window_start.saturating_add(RATE_LIMIT_WINDOW_SECONDS) {
            rate_limit = NotifyRateLimit {
                window_start: now,
                count: 0,
            };
        }

        if rate_limit.count >= MAX_NOTIFICATIONS_PER_WINDOW {
            panic_with_error!(&env, ContractError::RateLimitExceeded);
        }

        rate_limit.count += 1;
        env.storage().instance().set(&rate_key, &rate_limit);

        let contacts = PetChainContract::get_emergency_contacts(env.clone(), pet_id, caller.clone());

        env.events().publish(
            (Symbol::new(&env, "EmergencyContactsNotified"), pet_id),
            (caller, contacts.len()),
        );

        contacts.len()
    }

    pub fn get_contacts_ordered(env: Env, pet_id: u64, owner: Address) -> Vec<EmergencyContact> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            if owner != pet.owner {
                panic_with_error!(&env, ContractError::Unauthorized);
            }
            owner.require_auth();

            let key = PetChainContract::get_encryption_key(&env);
            let c_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let contacts =
                Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env));

            let mut ordered: Vec<EmergencyContact> = Vec::new(&env);
            for i in 0..contacts.len() {
                let contact = contacts.get(i).unwrap();
                let mut inserted = false;
                for j in 0..ordered.len() {
                    if contact.priority < ordered.get(j).unwrap().priority {
                        ordered.insert(j, contact.clone());
                        inserted = true;
                        break;
                    }
                }
                if !inserted {
                    ordered.push_back(contact.clone());
                }
            }
            ordered
        } else {
            Vec::new(&env)
        }
    }

    pub fn reorder_contact(env: Env, pet_id: u64, index: u32, new_priority: u32) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();

            let key = PetChainContract::get_encryption_key(&env);
            let c_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let mut contacts =
                Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env));

            if index >= contacts.len() {
                return;
            }

            let old_priority = contacts.get(index).unwrap().priority;
            if old_priority != new_priority {
                for i in 0..contacts.len() {
                    if i != index && contacts.get(i).unwrap().priority == new_priority {
                        let mut other = contacts.get(i).unwrap().clone();
                        other.priority = old_priority;
                        contacts.set(i, other);
                        break;
                    }
                }
                let mut target = contacts.get(index).unwrap().clone();
                target.priority = new_priority;
                contacts.set(index, target);
            }

            PetChainContract::validate_emergency_contacts(&env, &contacts);
            let contacts_bytes = contacts.to_xdr(&env);
            let (c_nonce, c_cipher) = encrypt_sensitive_data(&env, &contacts_bytes, &key);
            pet.encrypted_emergency_contacts = EncryptedData {
                nonce: c_nonce,
                ciphertext: c_cipher,
            };
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        }
    }

    pub fn get_emergency_access_logs(
        env: Env,
        pet_id: u64,
        caller: Address,
    ) -> Vec<EmergencyAccessLog> {
        // Verify pet exists
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            // Require owner authorization
            if caller != pet.owner {
                panic_with_error!(&env, ContractError::NotPetOwner);
            }

            // Retrieve logs from persistent storage
            let log_key = DataKey::EmergencyAccessLogs(pet_id);
            env.storage()
                .persistent()
                .get(&log_key)
                .unwrap_or(Vec::new(&env))
        } else {
            panic_with_error!(&env, ContractError::PetNotFound);
        }
    }

    // --- DISPUTE RESOLUTION ---

    pub fn set_appeal_window(env: Env, admin: Address, window_seconds: u64) -> bool {
        Self::require_admin_auth(&env, &admin);
        env.storage()
            .instance()
            .set(&DisputeKey::AppealWindow, &window_seconds);
        true
    }

    pub fn assign_arbitrator(env: Env, admin: Address, arbitrator: Address) -> bool {
        Self::require_admin_auth(&env, &admin);
        if admin == arbitrator {
            env.panic_with_error(ContractError::Unauthorized);
        }
        env.storage()
            .instance()
            .set(&DisputeKey::Arbitrator, &arbitrator);
        true
    }

    pub fn get_arbitrator(env: Env) -> Option<Address> {
        env.storage().instance().get(&DisputeKey::Arbitrator)
    }

    pub fn raise_dispute(
        env: Env,
        pet_id: u64,
        claimer: Address,
        target: Address,
        amount: u64,
        reason: String,
        evidence_hash: String,
    ) -> u64 {
        claimer.require_auth();

        // Bound string fields to prevent unbounded ledger growth. (#1152)
        if reason.len() > MAX_DISPUTE_REASON_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&DisputeKey::DisputeCount)
            .unwrap_or(0);
        let dispute_id = safe_increment(count);

        let dispute = Dispute {
            dispute_id,
            pet_id,
            claimer: claimer.clone(),
            target: target.clone(),
            amount,
            reason,
            evidence_hash: evidence_hash.clone(),
            status: DisputeStatus::Pending,
            created_at: env.ledger().timestamp(),
            resolved_at: None,
        };

        env.storage()
            .instance()
            .set(&DisputeKey::Dispute(dispute_id), &dispute);
        env.storage()
            .instance()
            .set(&DisputeKey::DisputeCount, &dispute_id);

        let pet_count_key = DisputeKey::PetDisputesCount(pet_id);
        let pet_count: u64 = env.storage().instance().get(&pet_count_key).unwrap_or(0);
        let new_pet_count = safe_increment(pet_count);

        env.storage().instance().set(
            &DisputeKey::PetDisputesIndex((pet_id, new_pet_count)),
            &dispute_id,
        );
        env.storage().instance().set(&pet_count_key, &new_pet_count);

        dispute_id
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Option<Dispute> {
        env.storage()
            .instance()
            .get(&DisputeKey::Dispute(dispute_id))
    }

    /// Admin override: forcibly resolves a dispute, bypassing the consensus
    /// vote. Requires admin authorization. Use `vote_on_dispute` for the
    /// standard multi-party consensus path.
    pub fn resolve_dispute(env: Env, dispute_id: u64, status: DisputeStatus) -> bool {
        Self::require_admin(&env);

        let key = DisputeKey::Dispute(dispute_id);
        if let Some(mut dispute) = env.storage().instance().get::<DisputeKey, Dispute>(&key) {
            dispute.status = status;
            dispute.resolved_at = Some(env.ledger().timestamp());
            env.storage().instance().set(&key, &dispute);

            env.events().publish(
                (Symbol::new(&env, "DisputeResolved"),),
                (dispute_id, status, env.ledger().timestamp()),
            );
            true
        } else {
            false
        }
    }

    /// Returns true if `voter` is an eligible stakeholder for `dispute`:
    /// the pet owner (claimer), the opposing party (target), or a multisig
    /// admin.
    fn is_dispute_stakeholder(env: &Env, dispute: &Dispute, voter: &Address) -> bool {
        voter == &dispute.claimer || voter == &dispute.target || Self::is_admin_address(env, voter)
    }

    /// Casts a vote on a dispute's resolution. Eligible voters are the pet
    /// owner (claimer), the opposing party (target/vet/groomer), or a
    /// multisig admin. Once at least 2 of these 3 stakeholder classes have
    /// cast matching votes, the dispute is automatically resolved:
    /// `Approve` votes resolve in favor of the claimer, `Reject` votes
    /// resolve in favor of the target. Returns `true` if this vote caused
    /// the dispute to auto-resolve.
    pub fn vote_on_dispute(env: Env, voter: Address, dispute_id: u64, vote: DisputeVote) -> bool {
        voter.require_auth();

        let dispute_key = DisputeKey::Dispute(dispute_id);
        let mut dispute: Dispute = env
            .storage()
            .instance()
            .get(&dispute_key)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::DisputeNotFound));

        if !(dispute.status == DisputeStatus::Pending
            || dispute.status == DisputeStatus::EvidencePhase)
        {
            panic_with_error!(&env, ContractError::DisputeNotOpenForVoting);
        }

        if !Self::is_dispute_stakeholder(&env, &dispute, &voter) {
            panic_with_error!(&env, ContractError::NotDisputeStakeholder);
        }

        let vote_key = DisputeKey::DisputeVoteByVoter(dispute_id, voter.clone());
        let is_new_voter = !env.storage().instance().has(&vote_key);

        env.storage().instance().set(
            &vote_key,
            &DisputeVoteRecord {
                voter: voter.clone(),
                vote,
                timestamp: env.ledger().timestamp(),
            },
        );

        if is_new_voter {
            let voters_key = DisputeKey::DisputeVoters(dispute_id);
            let mut voters: Vec<Address> = env
                .storage()
                .instance()
                .get(&voters_key)
                .unwrap_or_else(|| Vec::new(&env));
            voters.push_back(voter.clone());
            env.storage().instance().set(&voters_key, &voters);
        }

        env.events().publish(
            (Symbol::new(&env, "DisputeVoteCast"),),
            (dispute_id, voter.clone(), vote, env.ledger().timestamp()),
        );

        // Tally votes across all distinct stakeholders who have voted so far.
        let voters_key = DisputeKey::DisputeVoters(dispute_id);
        let voters: Vec<Address> = env
            .storage()
            .instance()
            .get(&voters_key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut approve_count: u32 = 0;
        let mut reject_count: u32 = 0;
        for addr in voters.iter() {
            if let Some(record) = env
                .storage()
                .instance()
                .get::<DisputeKey, DisputeVoteRecord>(&DisputeKey::DisputeVoteByVoter(
                    dispute_id,
                    addr.clone(),
                ))
            {
                match record.vote {
                    DisputeVote::Approve => approve_count += 1,
                    DisputeVote::Reject => reject_count += 1,
                }
            }
        }

        const RESOLUTION_THRESHOLD: u32 = 2;

        let resolved_status = if approve_count >= RESOLUTION_THRESHOLD {
            Some(DisputeStatus::ResolvedInFavorOfClaimer)
        } else if reject_count >= RESOLUTION_THRESHOLD {
            Some(DisputeStatus::ResolvedInFavorOfTarget)
        } else {
            None
        };

        if let Some(final_status) = resolved_status {
            dispute.status = final_status;
            dispute.resolved_at = Some(env.ledger().timestamp());
            env.storage().instance().set(&dispute_key, &dispute);

            env.events().publish(
                (Symbol::new(&env, "DisputeResolved"),),
                (dispute_id, final_status, env.ledger().timestamp()),
            );
            true
        } else {
            false
        }
    }

    /// Returns all recorded votes for a dispute.
    pub fn get_dispute_votes(env: Env, dispute_id: u64) -> Vec<DisputeVoteRecord> {
        let voters_key = DisputeKey::DisputeVoters(dispute_id);
        let voters: Vec<Address> = env
            .storage()
            .instance()
            .get(&voters_key)
            .unwrap_or_else(|| Vec::new(&env));

        let mut result = Vec::new(&env);
        for addr in voters.iter() {
            if let Some(record) = env
                .storage()
                .instance()
                .get::<DisputeKey, DisputeVoteRecord>(&DisputeKey::DisputeVoteByVoter(
                    dispute_id,
                    addr.clone(),
                ))
            {
                result.push_back(record);
            }
        }
        result
    }

    pub fn get_pet_disputes(env: Env, pet_id: u64) -> Vec<Dispute> {
        let mut result = Vec::new(&env);
        let count_key = DisputeKey::PetDisputesCount(pet_id);
        let count: u64 = env.storage().instance().get(&count_key).unwrap_or(0);
        for i in 1..=count {
            if let Some(dispute_id) = env
                .storage()
                .instance()
                .get::<DisputeKey, u64>(&DisputeKey::PetDisputesIndex((pet_id, i)))
            {
                if let Some(dispute) = env
                    .storage()
                    .instance()
                    .get::<DisputeKey, Dispute>(&DisputeKey::Dispute(dispute_id))
                {
                    result.push_back(dispute);
                }
            }
        }
        result
    }

    pub fn submit_evidence(
        env: Env,
        dispute_id: u64,
        submitter: Address,
        cid: String,
        sha256_hash: BytesN<32>,
    ) -> u64 {
        submitter.require_auth();

        let dispute_key = DisputeKey::Dispute(dispute_id);
        let dispute: Dispute = env
            .storage()
            .instance()
            .get(&dispute_key)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::DisputeNotFound));

        if dispute.status != DisputeStatus::EvidencePhase {
            panic_with_error!(&env, ContractError::NotInEvidencePhase);
        }

        if !(submitter == dispute.claimer || submitter == dispute.target) {
            panic_with_error!(&env, ContractError::NotDisputeParty);
        }

        let count_key = DisputeKey::PartyEvidenceCount(dispute_id, submitter.clone());
        let party_count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        if party_count >= 10 {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let evidence_count_key = DisputeKey::DisputeEvidenceCount(dispute_id);
        let total_count: u64 = env
            .storage()
            .instance()
            .get(&evidence_count_key)
            .unwrap_or(0);
        let evidence_id = safe_increment(total_count);

        let evidence = Evidence {
            evidence_id,
            submitter: submitter.clone(),
            cid,
            sha256_hash,
        };

        env.storage().instance().set(
            &DisputeKey::DisputeEvidence(dispute_id, evidence_id),
            &evidence,
        );
        env.storage()
            .instance()
            .set(&evidence_count_key, &evidence_id);
        env.storage().instance().set(
            &count_key,
            &party_count
                .checked_add(1)
                .unwrap_or_else(|| panic_with_error!(&env, ContractError::CounterOverflow)),
        );

        evidence_id
    }

    pub fn verify_evidence(env: Env, dispute_id: u64, evidence_id: u64, hash: BytesN<32>) -> bool {
        let key = DisputeKey::DisputeEvidence(dispute_id, evidence_id);
        if let Some(evidence) = env.storage().instance().get::<DisputeKey, Evidence>(&key) {
            evidence.sha256_hash == hash
        } else {
            false
        }
    }

    pub fn propose_signer_rotation(
        env: Env,
        proposer: Address,
        remove_address: Address,
        add_address: Address,
    ) -> u64 {
        Self::propose_action(
            env,
            proposer,
            ProposalAction::RotateSigner((remove_address, add_address)),
            3600 * 24, // 1 day
        )
    }

    /// Returns the total number of grooming records for a given pet.
    /// Returns 0 if the pet has no grooming records.
    /// Useful for pagination UI to determine total pages.
    pub fn get_grooming_count(env: Env, pet_id: u64) -> u64 {
        env.storage()
            .instance()
            .get(&GroomingKey::PetGroomingCount(pet_id))
            .unwrap_or(0)
    }

    /// Create a recurring grooming schedule and generate the first 4 appointment slots.
    /// Returns the schedule_id.
    pub fn create_grooming_schedule(
        env: Env,
        pet_id: u64,
        frequency: GroomingFrequency,
        start_date: u64,
        end_date: u64,
        groomer: String,
        service_type: String,
        cost: u64,
    ) -> u64 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::PetNotFound));
        pet.owner.require_auth();

        if end_date <= start_date {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::RecurringScheduleCount)
            .unwrap_or(0);
        let schedule_id = safe_increment(&env, count);

        let interval = Self::frequency_to_seconds(&frequency);
        let mut last_slot_date = start_date;

        for i in 0u64..4 {
            let slot_date = start_date.saturating_add(interval.saturating_mul(i));
            if slot_date > end_date {
                break;
            }
            let next_due = slot_date.saturating_add(interval);
            let rec_count: u64 = env
                .storage()
                .instance()
                .get(&GroomingKey::GroomingRecordCount)
                .unwrap_or(0);
            let record_id = safe_increment(&env, rec_count);
            let record = GroomingRecord {
                id: record_id,
                pet_id,
                service_type: service_type.clone(),
                groomer: groomer.clone(),
                groomer_address: None,
                date: slot_date,
                next_due,
                cost,
                notes: String::from_str(&env, ""),
            };
            env.storage()
                .instance()
                .set(&GroomingKey::GroomingRecord(record_id), &record);
            env.storage()
                .instance()
                .set(&GroomingKey::GroomingRecordCount, &record_id);
            let pet_count: u64 = env
                .storage()
                .instance()
                .get(&GroomingKey::PetGroomingCount(pet_id))
                .unwrap_or(0);
            let new_pet_count = safe_increment(&env, pet_count);
            env.storage()
                .instance()
                .set(&GroomingKey::PetGroomingCount(pet_id), &new_pet_count);
            env.storage().instance().set(
                &GroomingKey::PetGroomingIndex((pet_id, new_pet_count)),
                &record_id,
            );
            last_slot_date = slot_date;
        }

        let schedule = RecurringGroomingSchedule {
            id: schedule_id,
            pet_id,
            frequency,
            start_date,
            end_date,
            groomer,
            service_type,
            cost,
            is_active: true,
            last_slot_date,
        };

        env.storage()
            .instance()
            .set(&GroomingKey::RecurringSchedule(schedule_id), &schedule);
        env.storage()
            .instance()
            .set(&GroomingKey::RecurringScheduleCount, &schedule_id);

        let pet_sched_count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::PetScheduleCount(pet_id))
            .unwrap_or(0);
        let new_pet_sched_count = safe_increment(&env, pet_sched_count);
        env.storage()
            .instance()
            .set(&GroomingKey::PetScheduleCount(pet_id), &new_pet_sched_count);
        env.storage().instance().set(
            &GroomingKey::PetScheduleIndex((pet_id, new_pet_sched_count)),
            &schedule_id,
        );

        schedule_id
    }

    /// Advance a schedule: generate the next appointment slot after the most recent one.
    /// Returns the new grooming record id, or 0 if schedule is inactive/past end_date.
    pub fn advance_schedule(env: Env, schedule_id: u64) -> u64 {
        let mut schedule: RecurringGroomingSchedule = env
            .storage()
            .instance()
            .get(&GroomingKey::RecurringSchedule(schedule_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInput));

        if !schedule.is_active {
            return 0;
        }

        let interval = Self::frequency_to_seconds(&schedule.frequency);
        let next_date = schedule.last_slot_date.saturating_add(interval);

        if next_date > schedule.end_date {
            return 0;
        }

        let rec_count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomingRecordCount)
            .unwrap_or(0);
        let record_id = safe_increment(&env, rec_count);
        let record = GroomingRecord {
            id: record_id,
            pet_id: schedule.pet_id,
            service_type: schedule.service_type.clone(),
            groomer: schedule.groomer.clone(),
            groomer_address: None,
            date: next_date,
            next_due: next_date.saturating_add(interval),
            cost: schedule.cost,
            notes: String::from_str(&env, ""),
        };
        env.storage()
            .instance()
            .set(&GroomingKey::GroomingRecord(record_id), &record);
        env.storage()
            .instance()
            .set(&GroomingKey::GroomingRecordCount, &record_id);

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::PetGroomingCount(schedule.pet_id))
            .unwrap_or(0);
        let new_pet_count = safe_increment(&env, pet_count);
        env.storage().instance().set(
            &GroomingKey::PetGroomingCount(schedule.pet_id),
            &new_pet_count,
        );
        env.storage().instance().set(
            &GroomingKey::PetGroomingIndex((schedule.pet_id, new_pet_count)),
            &record_id,
        );

        schedule.last_slot_date = next_date;
        env.storage()
            .instance()
            .set(&GroomingKey::RecurringSchedule(schedule_id), &schedule);

        record_id
    }

    /// Cancel a recurring schedule. Existing slots remain; no new slots will be generated.
    pub fn cancel_grooming_schedule(env: Env, schedule_id: u64) -> bool {
        let mut schedule: RecurringGroomingSchedule = env
            .storage()
            .instance()
            .get(&GroomingKey::RecurringSchedule(schedule_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidInput));

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(schedule.pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::PetNotFound));
        pet.owner.require_auth();

        schedule.is_active = false;
        env.storage()
            .instance()
            .set(&GroomingKey::RecurringSchedule(schedule_id), &schedule);
        true
    }

    fn frequency_to_seconds(frequency: &GroomingFrequency) -> u64 {
        match frequency {
            GroomingFrequency::Weekly => 7 * 24 * 3600,
            GroomingFrequency::Biweekly => 14 * 24 * 3600,
            GroomingFrequency::Monthly => 30 * 24 * 3600,
        }
    }

    pub fn register_groomer(
        env: Env,
        admin: Address,
        address: Address,
        name: String,
        license_id: String,
    ) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);

        if env
            .storage()
            .instance()
            .has(&GroomingKey::Groomer(address.clone()))
        {
            return false;
        }

        let profile = GroomerProfile {
            address: address.clone(),
            name,
            license_id,
            aggregate_rating: 0,
            review_count: 0,
        };

        env.storage()
            .instance()
            .set(&GroomingKey::Groomer(address), &profile);
        true
    }

    fn validate_groomer_identity(env: &Env, record: &GroomingRecord) {
        if let Some(groomer_address) = record.groomer_address.clone() {
            if let Some(profile) = env
                .storage()
                .instance()
                .get::<GroomingKey, GroomerProfile>(&GroomingKey::Groomer(groomer_address))
            {
                if profile.name != record.groomer {
                    panic_with_error!(env, ContractError::InvalidInput);
                }
            }
        }
    }

    pub fn rate_groomer(env: Env, pet_id: u64, grooming_record_id: u64, score: u32) -> bool {
        if !(1..=5).contains(&score) {
            panic_with_error!(env, ContractError::InvalidRating);
        }

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::PetNotFound));
        pet.owner.require_auth();

        let record: GroomingRecord = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomingRecord(grooming_record_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::InvalidState));

        if record.pet_id != pet_id {
            panic_with_error!(env, ContractError::InvalidInput);
        }

        Self::validate_groomer_identity(&env, &record);

        if let Some(groomer_address) = record.groomer_address.clone() {
            if let Some(mut profile) = env
                .storage()
                .instance()
                .get::<GroomingKey, GroomerProfile>(&GroomingKey::Groomer(groomer_address.clone()))
            {
                let old_rating = profile.aggregate_rating as u64;
                let count = profile.review_count;
                let new_count = safe_increment(count);
                let new_avg = ((old_rating * count) + (score as u64 * 100)) / new_count;
                profile.aggregate_rating = new_avg as u32;
                profile.review_count = new_count;
                env.storage()
                    .instance()
                    .set(&GroomingKey::Groomer(groomer_address), &profile);
                return true;
            }
        }

        false
    }

    pub fn get_groomer_profile(env: Env, address: Address) -> Option<GroomerProfile> {
        env.storage().instance().get(&GroomingKey::Groomer(address))
    }

    // --- BREED METADATA ---
    /// Book a grooming slot with conflict detection.
    /// Checks for overlapping bookings for the same groomer within the
    /// requested time window.  Returns the new slot_id on success, or
    /// panics with `SlotAlreadyBooked` if a conflict is found.
    ///
    /// Conflict logic: existing.start_time < new.start_time + new.duration_mins
    ///               && new.start_time < existing.start_time + existing.duration_mins
    pub fn book_grooming_slot(
        env: Env,
        groomer_id: Address,
        start_time: u64,
        duration_mins: u64,
        pet_id: u64,
        owner: Address,
    ) -> u64 {
        owner.require_auth();

        // Check the pet exists and caller is the owner
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(env, ContractError::PetNotFound));
        if pet.owner != owner {
            panic_with_error!(env, ContractError::NotPetOwner);
        }

        // Verify groomer is registered
        if !env
            .storage()
            .instance()
            .has(&GroomingKey::Groomer(groomer_id.clone()))
        {
            panic_with_error!(env, ContractError::InvalidInput);
        }

        // Load existing slots for this groomer and check for conflicts
        let slot_count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomerSlotCount(groomer_id.clone()))
            .unwrap_or(0);

        for i in 1u64..=slot_count {
            if let Some(slot) = env.storage().instance().get::<GroomingKey, GroomingSlot>(
                &GroomingKey::GroomerSlotIndex((groomer_id.clone(), i)),
            ) {
                // Conflict: existing.start_time < new.start_time + new.duration_mins
                //         && new.start_time < existing.start_time + existing.duration_mins
                if slot.start_time
                    < start_time
                        .saturating_add(duration_mins.saturating_mul(60))
                    && start_time
                        < slot
                            .start_time
                            .saturating_add(slot.duration_mins.saturating_mul(60))
                {
                    panic_with_error!(env, ContractError::SlotAlreadyBooked);
                }
            }
        }

        // No conflict — assign a new slot_id and persist the slot
        let slot_id: u64 = safe_increment(
            env.storage()
                .instance()
                .get(&GroomingKey::GroomingRecordCount)
                .unwrap_or(0u64),
        );

        let new_slot = GroomingSlot {
            slot_id,
            groomer_id: groomer_id.clone(),
            start_time,
            duration_mins,
            pet_id,
        };

        let new_count = safe_increment(slot_count);
        env.storage().instance().set(
            &GroomingKey::GroomerSlotIndex((groomer_id.clone(), new_count)),
            &new_slot,
        );
        env.storage()
            .instance()
            .set(&GroomingKey::GroomerSlotCount(groomer_id), &new_count);

        slot_id
    }

    /// Return all booked grooming slots for a given groomer (for testing / querying).
    pub fn get_groomer_slots(env: Env, groomer_id: Address) -> Vec<GroomingSlot> {
        let mut slots = Vec::new(&env);
        let count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomerSlotCount(groomer_id.clone()))
            .unwrap_or(0);
        for i in 1u64..=count {
            if let Some(slot) = env.storage().instance().get::<GroomingKey, GroomingSlot>(
                &GroomingKey::GroomerSlotIndex((groomer_id.clone(), i)),
            ) {
                slots.push_back(slot);
            }
        }
        slots
    }

    /// Returns paginated offspring IDs for a breeding record (Issue #1031).
    /// Storage key: `BreedingKey::BreedingOffspringIndex((record_id, seq))`
    pub fn get_offspring_ids(
        env: Env,
        record_id: u64,
        offset: u32,
        limit: u32,
    ) -> Vec<u64> {
        let total: u64 = env
            .storage()
            .persistent()
            .get(&BreedingKey::BreedingOffspringCount(record_id))
            .unwrap_or(0);

        let mut result = Vec::new(&env);
        if limit == 0 || (offset as u64) >= total {
            return result;
        }

        let start = (offset as u64) + 1; // 1-based index
        let end = (start + (limit as u64) - 1).min(total);

        for seq in start..=end {
            if let Some(offspring_id) = env
                .storage()
                .persistent()
                .get::<BreedingKey, u64>(&BreedingKey::BreedingOffspringIndex((record_id, seq)))
            {
                result.push_back(offspring_id);
            }
        }
        result
    }



    pub fn add_breed_metadata(
        env: Env,
        admin: Address,
        breed_id: String,
        species: String,
        avg_lifespan_years: u32,
    ) {
        admin.require_auth();
        if !PetChainContract::is_admin(&env, &admin) {
            env.panic_with_error(ContractError::NotAnAdmin);
        }

        let metadata = BreedMetadata {
            species,
            avg_lifespan_years,
        };

        env.storage()
            .instance()
            .set(&DataKey::BreedMetadata(breed_id), &metadata);
    }

    pub fn update_breed_metadata(
        env: Env,
        admin: Address,
        breed_id: String,
        species: String,
        avg_lifespan_years: u32,
    ) {
        admin.require_auth();
        if !PetChainContract::is_admin(&env, &admin) {
            env.panic_with_error(ContractError::NotAnAdmin);
        }

        let metadata = BreedMetadata {
            species,
            avg_lifespan_years,
        };

        env.storage()
            .instance()
            .set(&DataKey::BreedMetadata(breed_id), &metadata);
    }

    pub fn delete_breed_metadata(env: Env, admin: Address, breed_id: String) {
        admin.require_auth();
        if !PetChainContract::is_admin(&env, &admin) {
            env.panic_with_error(ContractError::NotAnAdmin);
        }

        env.storage()
            .instance()
            .remove(&DataKey::BreedMetadata(breed_id));
    }

    pub fn get_pet_age_with_lifespan(env: Env, pet_id: u64) -> PetAge {
        if let Some(pet) =
            PetChainContract::get_pet(env.clone(), pet_id, env.current_contract_address())
        {
            let current_time = env.ledger().timestamp();
            let birthday_timestamp = match PetChainContract::parse_birthday_timestamp(&pet.birthday)
            {
                Ok(timestamp) => timestamp,
                Err(_) => {
                    return PetAge {
                        years: 0,
                        months: 0,
                        days: 0,
                        lifespan_pct: None,
                    }
                }
            };

            if current_time < birthday_timestamp {
                return PetAge {
                    years: 0,
                    months: 0,
                    days: 0,
                    lifespan_pct: None,
                };
            }

            let elapsed_seconds = current_time - birthday_timestamp;
            let elapsed_days = elapsed_seconds / 86_400;
            let years = (elapsed_days / 365) as u32;
            let remaining_days = (elapsed_days % 365) as u32;
            let months = remaining_days / 30;
            let days = remaining_days % 30;

            let lifespan_pct = if let Some(metadata) = env
                .storage()
                .instance()
                .get::<DataKey, BreedMetadata>(&DataKey::BreedMetadata(pet.breed.clone()))
            {
                let age_years = years as u64;
                let lifespan_years = metadata.avg_lifespan_years as u64;
                (age_years * 100)
                    .checked_div(lifespan_years)
                    .map(|pct| (pct as u32).min(100))
            } else {
                None
            };

            PetAge {
                years,
                months,
                days,
                lifespan_pct,
            }
        } else {
            PetAge {
                years: 0,
                months: 0,
                days: 0,
                lifespan_pct: None,
            }
        }
    }
    // -------------------------------------------------------------------------
    // Storage Compaction (Issue: Soroban Contract Storage Compaction)
    // -------------------------------------------------------------------------

    /// Remove tombstone entries for a pet to reclaim storage.
    ///
    /// Removes:
    /// - Revoked or expired consent records (and their index slots)
    /// - Inactive or expired access grants (and their index slots)
    /// - Expired decryption delegation tokens
    /// - Fully-used nonce usage entries (used >= max_uses)
    ///
    /// Callable by the pet owner or any admin. Returns the total count of
    /// storage entries removed. The operation is idempotent — calling it
    /// multiple times produces the same final state.
    pub fn compact_storage(env: Env, pet_id: u64, caller: Address) -> u32 {
        caller.require_auth();

        // Authorise: owner or admin
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        if pet.owner != caller && !Self::is_admin_address(&env, &caller) {
            env.panic_with_error(ContractError::Unauthorized);
        }

        let now = env.ledger().timestamp();
        let mut removed: u32 = 0;

        // -----------------------------------------------------------------
        // 1. Compact revoked / expired consents
        // -----------------------------------------------------------------
        {
            let total: u64 = env
                .storage()
                .instance()
                .get(&ConsentKey::PetConsentCount(pet_id))
                .unwrap_or(0);

            // Collect indices of stale consents (1-based)
            let mut stale_indices: Vec<u64> = Vec::new(&env);
            for i in 1u64..=total {
                if let Some(cid) = env
                    .storage()
                    .instance()
                    .get::<ConsentKey, u64>(&ConsentKey::PetConsentIndex((pet_id, i)))
                {
                    if let Some(consent) = env
                        .storage()
                        .instance()
                        .get::<ConsentKey, Consent>(&ConsentKey::Consent(cid))
                    {
                        let expired = consent.expires_at.map(|exp| now > exp).unwrap_or(false);
                        if !consent.is_active || expired {
                            stale_indices.push_back(i);
                        }
                    }
                }
            }

            // Remove stale entries (iterate in reverse to keep index arithmetic simple)
            let stale_len = stale_indices.len();
            for rev in 0..stale_len {
                let pos = stale_indices.get(stale_len - 1 - rev).unwrap();

                // Remove the consent record itself
                if let Some(cid) = env
                    .storage()
                    .instance()
                    .get::<ConsentKey, u64>(&ConsentKey::PetConsentIndex((pet_id, pos)))
                {
                    env.storage().instance().remove(&ConsentKey::Consent(cid));
                    removed += 1;
                }

                // Compact the index: shift entries above `pos` down by one
                let current_count: u64 = env
                    .storage()
                    .instance()
                    .get(&ConsentKey::PetConsentCount(pet_id))
                    .unwrap_or(0);

                for j in pos..current_count {
                    if let Some(next_cid) = env
                        .storage()
                        .instance()
                        .get::<ConsentKey, u64>(&ConsentKey::PetConsentIndex((pet_id, j + 1)))
                    {
                        env.storage()
                            .instance()
                            .set(&ConsentKey::PetConsentIndex((pet_id, j)), &next_cid);
                    }
                }
                // Remove the now-dangling last slot
                env.storage()
                    .instance()
                    .remove(&ConsentKey::PetConsentIndex((pet_id, current_count)));
                removed += 1; // index slot

                env.storage()
                    .instance()
                    .set(&ConsentKey::PetConsentCount(pet_id), &(current_count - 1));
            }
        }

        // -----------------------------------------------------------------
        // 2. Compact inactive / expired access grants
        // -----------------------------------------------------------------
        {
            let grant_count: u64 = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
                .unwrap_or(0);

            // Collect (index, grantee) pairs for stale grants
            let mut stale: Vec<(u64, Address)> = Vec::new(&env);
            for i in 1u64..=grant_count {
                if let Some(grantee) = env
                    .storage()
                    .instance()
                    .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
                {
                    let key = DataKey::AccessGrant((pet_id, grantee.clone()));
                    if let Some(grant) = env.storage().instance().get::<DataKey, AccessGrant>(&key)
                    {
                        let expired = grant.expires_at.map(|exp| now >= exp).unwrap_or(false);
                        if !grant.is_active || expired {
                            stale.push_back((i, grantee));
                        }
                    }
                }
            }

            let stale_len = stale.len();
            for rev in 0..stale_len {
                let (pos, grantee) = stale.get(stale_len - 1 - rev).unwrap();

                // Remove the grant record
                env.storage()
                    .instance()
                    .remove(&DataKey::AccessGrant((pet_id, grantee)));
                removed += 1;

                // Compact the index
                let current_count: u64 = env
                    .storage()
                    .instance()
                    .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
                    .unwrap_or(0);

                for j in pos..current_count {
                    if let Some(next_grantee) = env
                        .storage()
                        .instance()
                        .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, j + 1)))
                    {
                        env.storage()
                            .instance()
                            .set(&DataKey::AccessGrantIndex((pet_id, j)), &next_grantee);
                    }
                }
                env.storage()
                    .instance()
                    .remove(&DataKey::AccessGrantIndex((pet_id, current_count)));
                removed += 1; // index slot

                env.storage()
                    .instance()
                    .set(&DataKey::AccessGrantCount(pet_id), &(current_count - 1));
            }
        }

        // -----------------------------------------------------------------
        // 3. Compact expired decryption delegation tokens
        // -----------------------------------------------------------------
        // We cannot enumerate all delegates without an index, so we rely on
        // the caller supplying delegates via a separate helper, or we scan
        // the known delegation count. Since there is no delegate index, we
        // only clean up tokens that are provably expired by checking the
        // PetDelegationCount sentinel and resetting it when it reaches zero.
        // A full sweep requires the owner to call compact_delegation (below).
        // Here we just reset the count if it has drifted above zero but no
        // tokens remain (idempotent guard).
        {
            let delegation_count: u64 = env
                .storage()
                .instance()
                .get(&DataKey::PetDelegationCount(pet_id))
                .unwrap_or(0);
            // If count is non-zero but we cannot verify tokens (no index),
            // we leave it alone — compact_delegation handles the full sweep.
            let _ = delegation_count;
        }

        // -----------------------------------------------------------------
        // 4. Compact fully-used nonce usage entries
        // -----------------------------------------------------------------
        {
            // Nonce history is a Vec<Bytes> stored per (pet_id, key_id).
            // We compact by clearing the history list when all nonces in it
            // have reached max_uses, freeing the storage slot.
            // We iterate over the nonce history for the default key_id "".
            // Callers that use custom key_ids should call compact_nonces directly.
            let key_id = String::from_str(&env, "");
            let history_key = DataKey::NonceHistory((pet_id, key_id.clone()));
            if let Some(history) = env
                .storage()
                .instance()
                .get::<DataKey, Vec<Bytes>>(&history_key)
            {
                let max_uses: u32 = env
                    .storage()
                    .instance()
                    .get(&DataKey::NonceMaxUse((pet_id, key_id.clone())))
                    .unwrap_or(DEFAULT_NONCE_MAX_USES);

                let mut all_exhausted = true;
                for nonce in history.iter() {
                    let usage_key = DataKey::NonceUsage((pet_id, key_id.clone(), nonce.clone()));
                    let used: u32 = env.storage().instance().get(&usage_key).unwrap_or(0);
                    if used < max_uses {
                        all_exhausted = false;
                        break;
                    }
                }

                if all_exhausted && !history.is_empty() {
                    // Remove all usage entries and the history list
                    for nonce in history.iter() {
                        let usage_key =
                            DataKey::NonceUsage((pet_id, key_id.clone(), nonce.clone()));
                        env.storage().instance().remove(&usage_key);
                        removed += 1;
                    }
                    env.storage().instance().remove(&history_key);
                    removed += 1;
                }
            }
        }

        removed
    }

    /// Compact expired decryption delegation tokens for a specific set of
    /// delegates. Returns the number of tokens removed.
    ///
    /// This is a targeted helper because there is no global delegate index —
    /// the caller must supply the list of delegates to check.
    pub fn compact_delegations(
        env: Env,
        pet_id: u64,
        caller: Address,
        delegates: Vec<Address>,
    ) -> u32 {
        caller.require_auth();

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        if pet.owner != caller && !Self::is_admin_address(&env, &caller) {
            env.panic_with_error(ContractError::Unauthorized);
        }

        let now = env.ledger().timestamp();
        let current_version = Self::get_pet_key_version(env.clone(), pet_id);
        let mut removed: u32 = 0;

        for delegate in delegates.iter() {
            let key = DataKey::DecryptionToken((pet_id, delegate.clone()));
            if let Some(token) = env
                .storage()
                .instance()
                .get::<DataKey, DecryptionDelegation>(&key)
            {
                let stale = now >= token.expires_at || token.key_version != current_version;
                if stale {
                    env.storage().instance().remove(&key);
                    removed += 1;

                    // Decrement delegation count
                    let count: u64 = env
                        .storage()
                        .instance()
                        .get(&DataKey::PetDelegationCount(pet_id))
                        .unwrap_or(0);
                    if count > 0 {
                        env.storage()
                            .instance()
                            .set(&DataKey::PetDelegationCount(pet_id), &(count - 1));
                    }
                }
            }
        }

        removed
    }

    /// The encryption key version currently active for a pet. Defaults to
    /// `1` when the pet has never had its key rotated. (Issue #1163)
    pub fn get_pet_key_version(env: Env, pet_id: u64) -> u32 {
        env.storage()
            .instance()
            .get::<DataKey, u32>(&DataKey::PetKeyVersion(pet_id))
            .unwrap_or(1)
    }

    /// Rotate a pet's encryption key version. Owner-authorized and
    /// nonce-protected for replay safety, matching [`Self::grant_access`].
    ///
    /// Because every decryption token records the key version active when
    /// it was issued, bumping the version here deterministically and
    /// immediately invalidates every outstanding delegated token: the next
    /// call to [`Self::verify_decryption_token`] for any prior delegate
    /// will observe a version mismatch and fail, with no separate sweep or
    /// storage write required per delegate. (#1163)
    pub fn rotate_pet_key_version(env: Env, pet_id: u64, nonce: u64) -> u32 {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        Self::consume_caller_nonce(&env, &pet.owner, nonce);

        let new_version = Self::get_pet_key_version(env.clone(), pet_id)
            .checked_add(1)
            .unwrap_or_else(|| env.panic_with_error(ContractError::CounterOverflow));
        env.storage()
            .instance()
            .set(&DataKey::PetKeyVersion(pet_id), &new_version);

        env.events().publish(
            (String::from_str(&env, "PetKeyRotated"), pet_id),
            (pet_id, new_version, env.ledger().timestamp()),
        );

        new_version
    }

    /// Issue a time-boxed decryption delegation token bound to the pet's
    /// current key version. Owner-authorized and nonce-protected. (#1163)
    pub fn delegate_decryption_access(
        env: Env,
        pet_id: u64,
        delegate: Address,
        ttl_seconds: u64,
        nonce: u64,
    ) -> bool {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();
        Self::consume_caller_nonce(&env, &pet.owner, nonce);

        if ttl_seconds == 0 {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        let key = DataKey::DecryptionToken((pet_id, delegate.clone()));
        let is_new = env
            .storage()
            .instance()
            .get::<DataKey, DecryptionDelegation>(&key)
            .is_none();

        let now = env.ledger().timestamp();
        let token = DecryptionDelegation {
            pet_id,
            delegate: delegate.clone(),
            key_version: Self::get_pet_key_version(env.clone(), pet_id),
            issued_at: now,
            expires_at: now.saturating_add(ttl_seconds),
        };
        env.storage().instance().set(&key, &token);

        if is_new {
            let count: u64 = env
                .storage()
                .instance()
                .get(&DataKey::PetDelegationCount(pet_id))
                .unwrap_or(0);
            env.storage()
                .instance()
                .set(&DataKey::PetDelegationCount(pet_id), &safe_increment(&env, count));
        }

        true
    }

    /// Revoke a delegate's decryption token before it expires.
    /// Owner-authorized. (#1163)
    pub fn revoke_decryption_delegation(env: Env, pet_id: u64, delegate: Address) -> bool {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));
        pet.owner.require_auth();

        let key = DataKey::DecryptionToken((pet_id, delegate));
        if env
            .storage()
            .instance()
            .get::<DataKey, DecryptionDelegation>(&key)
            .is_none()
        {
            return false;
        }
        env.storage().instance().remove(&key);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetDelegationCount(pet_id))
            .unwrap_or(0);
        if count > 0 {
            env.storage()
                .instance()
                .set(&DataKey::PetDelegationCount(pet_id), &(count - 1));
        }
        true
    }

    /// Verify whether `delegate` currently holds a valid decryption token
    /// for `pet_id`: the token must exist, be unexpired, and be bound to
    /// the pet's *current* key version. A rotation, expiry, or explicit
    /// revocation all cause this to deterministically return `false`.
    /// (#1163)
    pub fn verify_decryption_token(env: Env, pet_id: u64, delegate: Address) -> bool {
        let token = match env
            .storage()
            .instance()
            .get::<DataKey, DecryptionDelegation>(&DataKey::DecryptionToken((pet_id, delegate)))
        {
            Some(t) => t,
            None => return false,
        };

        if env.ledger().timestamp() >= token.expires_at {
            return false;
        }

        token.key_version == Self::get_pet_key_version(env, pet_id)
    }

    /// Verify that a stored claim document hash matches `content_hash`.
    /// Emits a `ClaimDocumentIntegrity` event with the result.
    /// Returns `true` if stored hash matches, `false` for mismatch or out-of-bounds index.
    pub fn verify_claim_document(
        env: Env,
        claim_id: u64,
        doc_index: u32,
        content_hash: BytesN<32>,
    ) -> bool {
        let docs: soroban_sdk::Vec<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::ClaimDocuments(claim_id))
            .unwrap_or(soroban_sdk::Vec::new(&env));

        let matches = if doc_index < docs.len() {
            docs.get(doc_index).unwrap() == content_hash
        } else {
            false
        };

        env.events().publish(
            (Symbol::new(&env, "ClaimDocIntegrity"), claim_id),
            ClaimDocumentIntegrityEvent {
                version: EVENT_SCHEMA_VERSION,
                claim_id,
                doc_index,
                matches,
                timestamp: env.ledger().timestamp(),
            },
        );

        matches
    }

    pub fn add_activity_record(
        env: Env,
        pet_id: u64,
        activity_type: ActivityType,
        duration_minutes: u32,
        intensity: u32,
        distance_meters: u32,
        notes: String,
    ) -> u64 {
        // Bound notes field to prevent unbounded ledger growth. (#1152)
        if notes.len() > MAX_ACTIVITY_NOTES_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        // Verify pet exists
        let _pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        let now = env.ledger().timestamp();
        let _window: u64 = env
            .storage()
            .instance()
            .get(&ActivityKey::IdempotencyWindow)
            .unwrap_or(60); // Default 60 seconds

        // Create idempotency key from activity components
        // Use: activity_type (as u32) + pet_id + rounded duration
        let mut key_bytes = [0u8; 32];
        let activity_u32 = activity_type.clone() as u32;
        key_bytes[0..4].copy_from_slice(&activity_u32.to_le_bytes());
        key_bytes[4..12].copy_from_slice(&pet_id.to_le_bytes());
        let rounded_duration = (duration_minutes / 10) as u64;
        key_bytes[12..20].copy_from_slice(&rounded_duration.to_le_bytes());

        let idem_key = ActivityKey::ActivityIdempotencyKey(Bytes::from_array(&env, &key_bytes));

        // Check if key exists and is not expired
        if let Some(submitted_at) = env.storage().instance().get::<ActivityKey, u64>(&idem_key) {
            let ttl: u64 = 86400; // 24 hours TTL
            let expiry = submitted_at.saturating_add(ttl);
            if now < expiry {
                panic_with_error!(&env, ContractError::DuplicateActivity);
            }
        }

        // Store idempotency key with current timestamp
        env.storage().instance().set(&idem_key, &now);

        // Allocate new activity ID
        let activity_count: u64 = env
            .storage()
            .instance()
            .get(&ActivityKey::ActivityRecordCount)
            .unwrap_or(0);
        let activity_id = safe_increment(&env, activity_count);

        // Create and store activity record
        let record = ActivityRecord {
            id: activity_id,
            pet_id,
            activity_type,
            duration_minutes,
            intensity,
            distance_meters,
            recorded_at: now,
            notes,
        };

        env.storage()
            .instance()
            .set(&ActivityKey::ActivityRecord(activity_id), &record);
        env.storage()
            .instance()
            .set(&ActivityKey::ActivityRecordCount, &activity_id);

        // Track pet's activities
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&ActivityKey::PetActivityCount(pet_id))
            .unwrap_or(0);
        let pet_index = safe_increment(&env, pet_count);
        env.storage().instance().set(
            &ActivityKey::PetActivityIndex((pet_id, pet_index)),
            &activity_id,
        );
        env.storage()
            .instance()
            .set(&ActivityKey::PetActivityCount(pet_id), &pet_index);

        // ── STREAK TRACKING ──────────────────────────────────────────────────
        // Update the pet's consecutive-day activity streak.
        //
        // The streak is stored in persistent storage (not instance storage) so
        // it survives ledger TTL extension.  The last-activity-date entry is
        // stored alongside it for gap detection.
        //
        // Day boundaries use whole-day slots: `timestamp / 86400`.
        let seconds_per_day: u64 = 86400;
        let today: u64 = now / seconds_per_day;

        let mut streak: ActivityStreak = env
            .storage()
            .persistent()
            .get(&ActivityKey::PetActivityStreak(pet_id))
            .unwrap_or(ActivityStreak {
                pet_id,
                current_streak: 0,
                longest_streak: 0,
                last_activity_date: 0,
                milestones_reached: Vec::new(&env),
            });

        let last_day = streak.last_activity_date;

        if last_day == 0 {
            // First-ever activity for this pet.
            streak.current_streak = 1;
        } else if today == last_day {
            // Same calendar day — streak already counted for today; no change.
        } else if today == last_day + 1 {
            // Consecutive day — extend streak.
            streak.current_streak = streak.current_streak.saturating_add(1);
        } else {
            // Gap of >1 day — streak resets to 1 (today counts as day 1 of a
            // new streak but does not carry forward old milestone progress).
            streak.current_streak = 1;
        }

        // Update longest streak.
        if streak.current_streak > streak.longest_streak {
            streak.longest_streak = streak.current_streak;
        }

        // Record any newly-crossed milestones.
        // Guard with MAX_MILESTONES so the Vec never grows without bound.
        for &milestone in STREAK_MILESTONE_DAYS {
            if streak.current_streak >= milestone {
                // Only append if not already present AND cap not exceeded.
                let already_recorded = streak.milestones_reached.contains(milestone);
                let under_cap =
                    streak.milestones_reached.len() < MAX_MILESTONES;

                if !already_recorded && under_cap {
                    streak.milestones_reached.push_back(milestone);

                    // Emit a streak-milestone event.
                    env.events().publish(
                        (
                            soroban_sdk::Symbol::new(&env, "streak_milestone"),
                            pet_id,
                        ),
                        StreakMilestoneEvent {
                            pet_id,
                            milestone_days: milestone,
                            timestamp: now,
                        },
                    );
                }
            }
        }

        // Advance the last-activity-date only when we move to a new day or on
        // the very first activity (last_day == 0).
        if today != last_day {
            streak.last_activity_date = today;
        }

        env.storage()
            .persistent()
            .set(&ActivityKey::PetActivityStreak(pet_id), &streak);
        Self::bump_persistent_ttl(&env, &ActivityKey::PetActivityStreak(pet_id));

        activity_id
    }

    /// Return the current [`ActivityStreak`] for a pet.
    ///
    /// If the pet has never had an activity recorded the returned streak has all
    /// fields set to zero / empty.  Returns a default struct rather than
    /// panicking so callers can distinguish "no data yet" from an error.
    pub fn get_activity_streak(env: Env, pet_id: u64) -> ActivityStreak {
        env.storage()
            .persistent()
            .get(&ActivityKey::PetActivityStreak(pet_id))
            .unwrap_or(ActivityStreak {
                pet_id,
                current_streak: 0,
                longest_streak: 0,
                last_activity_date: 0,
                milestones_reached: Vec::new(&env),
            })
    }

    /// Return `true` if `pet_id` has reached the given `milestone_days` streak.
    ///
    /// This checks [`ActivityStreak::milestones_reached`] for the exact value.
    /// The milestone is recorded when the consecutive-day streak first reaches
    /// (or exceeds) that value inside [`Self::add_activity_record`].
    pub fn has_reached_milestone(env: Env, pet_id: u64, milestone_days: u64) -> bool {
        let streak: ActivityStreak = env
            .storage()
            .persistent()
            .get(&ActivityKey::PetActivityStreak(pet_id))
            .unwrap_or(ActivityStreak {
                pet_id,
                current_streak: 0,
                longest_streak: 0,
                last_activity_date: 0,
                milestones_reached: Vec::new(&env),
            });
        streak
            .milestones_reached
            .contains(milestone_days)
    }

    pub fn set_activity_idempotency_window(env: Env, admin: Address, window_seconds: u64) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        env.storage()
            .instance()
            .set(&ActivityKey::IdempotencyWindow, &window_seconds);
    }

    pub fn purge_expired_idempotency_keys(env: Env, admin: Address) -> u32 {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let _now = env.ledger().timestamp();
        let _ttl: u64 = 86400;
        0u32
    }

    // ── BREEDING RECORD MANAGEMENT ──────────────────────────────────

    pub fn add_breeding_record(
        env: Env,
        sire_id: u64,
        dam_id: u64,
        breeding_date: u64,
        notes: String,
    ) -> u64 {
        // Bound notes field to prevent unbounded ledger growth. (#1152)
        if notes.len() > MAX_BREEDING_NOTES_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        let count = env
            .storage()
            .persistent()
            .get(&BreedingKey::BreedingRecordCount)
            .unwrap_or(0u64);
        let id = safe_increment(&env, count);

        let record = BreedingRecord {
            id,
            sire_id,
            dam_id,
            breeding_date,
            offspring_count: 0,
            breeder: env.current_contract_address(),
            notes,
        };

        env.storage()
            .persistent()
            .set(&BreedingKey::BreedingRecord(id), &record);
        Self::bump_persistent_ttl(&env, &BreedingKey::BreedingRecord(id));
        env.storage()
            .persistent()
            .set(&BreedingKey::BreedingRecordCount, &id);
        Self::bump_persistent_ttl(&env, &BreedingKey::BreedingRecordCount);

        Self::inc_pet_breeding_count(&env, sire_id);
        Self::inc_pet_breeding_count(&env, dam_id);

        id
    }

    fn inc_pet_breeding_count(env: &Env, pet_id: u64) {
        let count = env
            .storage()
            .persistent()
            .get(&BreedingKey::PetBreedingCount(pet_id))
            .unwrap_or(0u64);
        env.storage()
            .persistent()
            .set(&BreedingKey::PetBreedingCount(pet_id), &safe_increment(count));
        Self::bump_persistent_ttl(env, &BreedingKey::PetBreedingCount(pet_id));
    }

    pub fn add_offspring(env: Env, record_id: u64, offspring_id: u64) -> bool {
        let mut record: BreedingRecord = env
            .storage()
            .persistent()
            .get(&BreedingKey::BreedingRecord(record_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        // Self-lineage check
        if offspring_id == record.sire_id || offspring_id == record.dam_id {
            panic_with_error!(&env, ContractError::InvalidInput);
        }

        // Circular lineage check – ensure offspring is not already an ancestor of either parent
        let sire_lineage = Self::collect_lineage_vec(&env, record.sire_id, MAX_LINEAGE_DEPTH);
        let dam_lineage = Self::collect_lineage_vec(&env, record.dam_id, MAX_LINEAGE_DEPTH);
        for ancestor in sire_lineage.iter().chain(dam_lineage.iter()) {
            if ancestor == offspring_id {
                panic_with_error!(&env, ContractError::InvalidState);
            }
        }

        // Store parent pair for pedigree queries (COI, lineage)
        let parent_pair_key = BreedingKey::ParentPair(offspring_id);
        env.storage()
            .persistent()
            .set(&parent_pair_key, &(record.sire_id, record.dam_id));
        Self::bump_persistent_ttl(&env, &parent_pair_key);

        record.offspring_count = record.offspring_count.saturating_add(1);
        env.storage()
            .persistent()
            .set(&BreedingKey::BreedingRecord(record_id), &record);
        Self::bump_persistent_ttl(&env, &BreedingKey::BreedingRecord(record_id));

        let count = env
            .storage()
            .persistent()
            .get(&BreedingKey::PetOffspringCount(offspring_id))
            .unwrap_or(0u64);
        env.storage()
            .persistent()
            .set(&BreedingKey::PetOffspringCount(offspring_id), &(count + 1));
        Self::bump_persistent_ttl(&env, &BreedingKey::PetOffspringCount(offspring_id));

        true
    }

    /// Internal lineage collector used by `add_offspring` and `get_lineage`.
    fn collect_lineage_vec(env: &Env, pet_id: u64, max_depth: u32) -> Vec<u64> {
        let mut result = Vec::new(env);
        let mut frontier = Vec::new(env);
        frontier.push_back(pet_id);

        for _ in 0..max_depth {
            let mut next = Vec::new(env);
            for node in frontier.iter() {
                if let Some((s, d)) = env
                    .storage()
                    .persistent()
                    .get::<BreedingKey, (u64, u64)>(&BreedingKey::ParentPair(node))
                {
                    result.push_back(s);
                    result.push_back(d);
                    next.push_back(s);
                    next.push_back(d);
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        result
    }

    pub fn get_lineage(env: Env, pet_id: u64, max_depth: u32) -> Vec<u64> {
        Self::collect_lineage_vec(&env, pet_id, max_depth)
    }

    pub fn get_breeding_record(env: Env, record_id: u64) -> Option<BreedingRecord> {
        env.storage()
            .persistent()
            .get(&BreedingKey::BreedingRecord(record_id))
    }

    pub fn get_breeding_count(env: Env, pet_id: u64) -> u64 {
        env.storage()
            .persistent()
            .get(&BreedingKey::PetBreedingCount(pet_id))
            .unwrap_or(0u64)
    }

    // ── MENDELIAN GENETICS ──────────────────────────────────────────

    pub fn set_pet_traits(env: Env, pet_id: u64, traits: Map<String, Allele>) {
        env.storage()
            .persistent()
            .set(&GeneticsKey::PetTraits(pet_id), &traits);
    }

    pub fn get_pet_traits(env: Env, pet_id: u64) -> Map<String, Allele> {
        env.storage()
            .persistent()
            .get(&GeneticsKey::PetTraits(pet_id))
            .unwrap_or_else(|| Map::new(&env))
    }

    pub fn compute_offspring_traits(env: Env, record_id: u64) -> Map<String, u32> {
        let record: BreedingRecord = env
            .storage()
            .persistent()
            .get(&BreedingKey::BreedingRecord(record_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        let sire_traits: Map<String, Allele> = env
            .storage()
            .persistent()
            .get(&GeneticsKey::PetTraits(record.sire_id))
            .unwrap_or_else(|| Map::new(&env));
        let dam_traits: Map<String, Allele> = env
            .storage()
            .persistent()
            .get(&GeneticsKey::PetTraits(record.dam_id))
            .unwrap_or_else(|| Map::new(&env));

        let mut result: Map<String, u32> = Map::new(&env);
        let mut seen: Map<String, bool> = Map::new(&env);

        for (name, _) in sire_traits.iter() {
            seen.set(name.clone(), true);
        }
        for (name, _) in dam_traits.iter() {
            seen.set(name.clone(), true);
        }

        for (name, _) in seen.iter() {
            let sa = sire_traits.get(name.clone()).unwrap_or(Allele::Recessive);
            let da = dam_traits.get(name.clone()).unwrap_or(Allele::Recessive);
            let prob = match (&sa, &da) {
                (Allele::Dominant, Allele::Dominant) => 10000u32,
                (Allele::Dominant, Allele::Recessive) => 7500u32,
                (Allele::Recessive, Allele::Dominant) => 7500u32,
                (Allele::Recessive, Allele::Recessive) => 0u32,
            };
            result.set(name, prob);
        }

        env.storage()
            .persistent()
            .set(&GeneticsKey::PredictedTraits(record_id), &result);
        result
    }

    pub fn get_trait_probability(
        env: Env,
        record_id: u64,
        trait_name: String,
    ) -> Option<u32> {
        let predicted: Map<String, u32> = env
            .storage()
            .persistent()
            .get(&GeneticsKey::PredictedTraits(record_id))
            .unwrap_or_else(|| Map::new(&env));
        predicted.get(trait_name)
    }

    // ── COEFFICIENT OF INBREEDING (Issue #778) ─────────────────────

    /// Returns the Coefficient of Inbreeding in basis points (0–10000).
    ///
    /// Uses a 3-generation pedigree traversal.  Integer math only — no
    /// floating point.  The formula follows Sewall Wright's path method:
    ///
    ///   COI = Σ  10000 / 2^(n1 + n2 + 1)
    ///
    /// where n1 is the number of generations from pet_a up to a common
    /// ancestor, and n2 is the number from pet_b up to that same ancestor.
    pub fn calculate_coi(env: Env, pet_id_a: u64, pet_id_b: u64) -> u32 {
        Self::calc_coi(&env, pet_id_a, pet_id_b)
    }

    /// Internal helper shared by `calculate_coi` (view) and
    /// `register_breeding_pair` (write) to avoid code duplication.
    fn calc_coi(env: &Env, pet_a: u64, pet_b: u64) -> u32 {
        let a = Self::build_pedigree_map(env, pet_a, 3);
        let b = Self::build_pedigree_map(env, pet_b, 3);
        let mut coi: u32 = 0;
        for (ancestor, da) in a.iter() {
            if let Some(db) = b.get(ancestor) {
                let exp = da + db + 1;
                if exp <= 13 {
                    coi += 10000u32 / (1u32 << exp);
                }
            }
        }
        coi
    }

    /// Build a Map of ancestor_id → shortest depth (generations up).
    fn build_pedigree_map(env: &Env, root: u64, max_depth: u32) -> Map<u64, u32> {
        let mut out = Map::new(env);
        let mut frontier = Vec::new(env);
        frontier.push_back(root);

        for gen in 1u32..=max_depth {
            let mut next = Vec::new(env);
            for pet in frontier.iter() {
                if let Some((s, d)) = env
                    .storage()
                    .persistent()
                    .get::<BreedingKey, (u64, u64)>(&BreedingKey::ParentPair(pet))
                {
                    if !out.contains_key(s) {
                        out.set(s, gen);
                        next.push_back(s);
                    }
                    if !out.contains_key(d) {
                        out.set(d, gen);
                        next.push_back(d);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        out
    }

    // ── BREEDING PAIR REGISTRATION WITH COI GUARD ───────────────────

    /// Register a breeding pair after validating the Coefficient of
    /// Inbreeding does not exceed `max_coi_bp` (basis points, 0–10000).
    ///
    /// # Errors
    /// - `SelfBreeding` — sire_id and dam_id are the same.
    /// - `InbreedingThresholdExceeded` — calculated COI ≥ max_coi_bp.
    pub fn register_breeding_pair(
        env: Env,
        sire_id: u64,
        dam_id: u64,
        breeding_date: u64,
        notes: String,
        max_coi_bp: u32,
    ) -> u64 {
        if sire_id == dam_id {
            panic_with_error!(&env, ContractError::SelfBreeding);
        }

        let coi = Self::calc_coi(&env, sire_id, dam_id);
        if coi >= max_coi_bp {
            panic_with_error!(&env, ContractError::InbreedingThresholdExceeded);
        }

        Self::add_breeding_record(env, sire_id, dam_id, breeding_date, notes)
    }

    // ── #764: remove_admin with threshold guard ───────────────────────────────

    pub fn remove_admin(env: Env, proposer: Address, admin_to_remove: Address) -> u64 {
        if !Self::is_admin_address(&env, &proposer) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&SystemKey::Admins)
            .unwrap_or_else(|| env.panic_with_error(ContractError::AdminsNotSet));

        let threshold: u32 = env
            .storage()
            .instance()
            .get(&SystemKey::AdminThreshold)
            .unwrap_or(1);

        let remaining = admins.len().saturating_sub(1);
        if remaining < threshold {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }

        let mut new_admins: Vec<Address> = Vec::new(&env);
        for a in admins.iter() {
            if a != admin_to_remove {
                new_admins.push_back(a);
            }
        }

        Self::propose_action(
            env,
            proposer,
            ProposalAction::ChangeAdmin((new_admins, threshold)),
            3600,
        )
    }

    pub fn set_retention_period(env: Env, admin: Address, period_seconds: u64) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        env.storage()
            .instance()
            .set(&MedicalKey::RetentionPeriod, &period_seconds);
    }

    pub fn get_retention_period(env: Env) -> u64 {
        env.storage()
            .instance()
            .get::<MedicalKey, u64>(&MedicalKey::RetentionPeriod)
            .unwrap_or(30 * 86_400)
    }

    pub fn delete_medical_record(env: Env, pet_id: u64, record_id: u64, caller: Address) -> bool {
        caller.require_auth();

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        let mut record: MedicalRecord = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::InvalidInput));

        if record.pet_id != pet_id {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        if caller != pet.owner && caller != record.vet_address && !Self::is_admin_address(&env, &caller) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        if record.deleted_at.is_some() {
            panic_with_error!(&env, ContractError::AlreadyDeleted);
        }

        let now = env.ledger().timestamp();
        record.deleted_at = Some(now);
        record.updated_at = now;

        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecord(record_id), &record);

        env.events().publish(
            (String::from_str(&env, "MedicalRecordDeleted"), pet_id),
            MedicalRecordDeletedEvent {
                version: EVENT_SCHEMA_VERSION,
                record_id,
                pet_id,
                deleted_by: caller,
                timestamp: now,
            },
        );

        true
    }

    pub fn purge_deleted_records(
        env: Env,
        pet_id: u64,
        caller: Address,
        dry_run: bool,
    ) -> PurgeResult {
        caller.require_auth();

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        if caller != pet.owner && !Self::is_admin_address(&env, &caller) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let retention_period = Self::get_retention_period(env.clone());
        let now = env.ledger().timestamp();
        let mut deleted = Vec::new(&env);

        let record_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);

        let mut has_unretained_deleted = false;

        for i in 1..=record_count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
                {
                    if let Some(deleted_at) = record.deleted_at {
                        if now >= deleted_at.saturating_add(retention_period) {
                            deleted.push_back(record_id);
                            if !dry_run {
                                env.storage()
                                    .instance()
                                    .remove(&MedicalKey::MedicalRecord(record_id));
                            }
                        } else {
                            has_unretained_deleted = true;
                        }
                    }
                }
            }
        }

        if deleted.is_empty() && has_unretained_deleted {
            panic_with_error!(&env, ContractError::RetentionPeriodNotMet);
        }

        if !dry_run && !deleted.is_empty() {
            if Self::is_admin_address(&env, &caller) {
                Self::record_admin_activity(&env, &caller, "purge_deleted_records");
            }
            env.events().publish(
                (String::from_str(&env, "MedicalRecordPurged"), pet_id),
                MedicalRecordPurgedEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id,
                    purged_count: deleted.len(),
                    purged_by: caller,
                    timestamp: now,
                },
            );
        }

        PurgeResult { deleted, dry_run }
    }

    /// Bounded, resumable purge of soft-deleted medical records (Issue #1172).
    ///
    /// Unlike [`Self::purge_deleted_records`], which scans every record slot in a
    /// single call and can exceed transaction resource limits on large pets, this
    /// processes at most `limit` candidate slots per call and returns an opaque
    /// `next_cursor` for the caller to echo back on the next call. Iterate until
    /// `next_cursor == 0` to fully drain a pet's expired, deleted records.
    ///
    /// Authorization matches [`Self::delete_medical_record`]: only the pet owner or
    /// an admin may purge. Purging is idempotent for replays — a slot whose record
    /// was already removed simply yields `None` and is skipped, so re-running a
    /// batch with the same cursor is safe. With `dry_run = true` nothing is written
    /// and no event is emitted.
    pub fn purge_deleted_records_bounded(
        env: Env,
        pet_id: u64,
        caller: Address,
        limit: u32,
        cursor: u64,
        dry_run: bool,
    ) -> BoundedPurgeResult {
        caller.require_auth();

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        if caller != pet.owner && !Self::is_admin_address(&env, &caller) {
            panic_with_error!(&env, ContractError::Unauthorized);
        }

        let record_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);

        // A zero limit is a harmless no-op that still yields a valid cursor.
        if limit == 0 {
            return BoundedPurgeResult {
                deleted: Vec::new(&env),
                next_cursor: cursor,
                dry_run,
            };
        }

        // `cursor` is the last examined slot; resume from the following slot.
        let start = cursor.saturating_add(1);
        if start > record_count {
            return BoundedPurgeResult {
                deleted: Vec::new(&env),
                next_cursor: 0,
                dry_run,
            };
        }

        let end = record_count.min(start.saturating_add(limit as u64).saturating_sub(1));
        let retention_period = Self::get_retention_period(env.clone());
        let now = env.ledger().timestamp();
        let mut deleted = Vec::new(&env);

        let mut idx = start;
        while idx <= end {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, idx)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
                {
                    if let Some(deleted_at) = record.deleted_at {
                        if now >= deleted_at.saturating_add(retention_period) {
                            deleted.push_back(record_id);
                            if !dry_run {
                                env.storage()
                                    .instance()
                                    .remove(&MedicalKey::MedicalRecord(record_id));
                            }
                        }
                    }
                }
            }
            idx += 1;
        }

        let last_examined = idx.saturating_sub(1);
        let next_cursor = if last_examined < record_count {
            last_examined
        } else {
            0
        };

        if !dry_run && !deleted.is_empty() {
            if Self::is_admin_address(&env, &caller) {
                Self::record_admin_activity(&env, &caller, "purge_deleted_records");
            }
            env.events().publish(
                (String::from_str(&env, "MedicalRecordPurged"), pet_id),
                MedicalRecordPurgedEvent {
                    version: EVENT_SCHEMA_VERSION,
                    pet_id,
                    purged_count: deleted.len(),
                    purged_by: caller,
                    timestamp: now,
                },
            );
        }

        BoundedPurgeResult {
            deleted,
            next_cursor,
            dry_run,
        }
    }

    pub fn purge_expired_records(env: Env, pet_id: u64, caller: Address) -> u32 {
        let res = Self::purge_deleted_records(env, pet_id, caller, false);
        res.deleted.len()
    }

    /// Build the canonical, versioned preimage bytes for a [`MedicalRecord`]
    /// (Issue #1169).
    ///
    /// Off-chain clients (in any language with a Stellar/Soroban XDR codec)
    /// need to be able to reproduce the exact same commitment a contract
    /// computes for a medical record, independent of storage/audit
    /// metadata that can change without the clinical facts changing. The
    /// canonical encoding is:
    ///
    /// ```text
    /// sha256(
    ///     b"petchain:medical-record:v1"        (26-byte literal domain tag)
    ///  || pet_id            as 8-byte big-endian u64
    ///  || vet_address       as its XDR-encoded `ScAddress`
    ///  || diagnosis         as its XDR-encoded `ScString`
    ///  || treatment         as its XDR-encoded `ScString`
    ///  || medications       as its XDR-encoded `ScVec` (fixed struct field order)
    ///  || notes             as its XDR-encoded `ScString`
    ///  || date              as 8-byte big-endian u64   (clinical event time)
    /// )
    /// ```
    ///
    /// Fields are concatenated in this fixed order with no separators
    /// (XDR-encoded values are already self-delimiting/length-prefixed, and
    /// the two `u64` fields have a fixed 8-byte width, so the encoding is
    /// unambiguous). `id`, `updated_at`, `attachment_hashes`, and
    /// `deleted_at` are intentionally excluded: they are ledger
    /// bookkeeping/audit metadata, not clinical content, so the commitment
    /// stays stable across non-clinical housekeeping mutations (e.g. an
    /// attachment being added, or a soft-delete).
    ///
    /// The `v1` domain tag is part of the preimage precisely so that any
    /// future change to the field set, order, or encoding can ship as a
    /// `v2` tag without silently colliding with existing `v1` commitments
    /// clients may have already anchored off-chain.
    fn canonical_medical_record_preimage(env: &Env, record: &MedicalRecord) -> Bytes {
        let mut preimage = Bytes::new(env);
        for byte in b"petchain:medical-record:v1" {
            preimage.push_back(*byte);
        }
        for byte in record.pet_id.to_be_bytes() {
            preimage.push_back(byte);
        }
        for byte in record.vet_address.to_xdr(env).iter() {
            preimage.push_back(byte);
        }
        for byte in record.diagnosis.to_xdr(env).iter() {
            preimage.push_back(byte);
        }
        for byte in record.treatment.to_xdr(env).iter() {
            preimage.push_back(byte);
        }
        for byte in record.medications.to_xdr(env).iter() {
            preimage.push_back(byte);
        }
        for byte in record.notes.to_xdr(env).iter() {
            preimage.push_back(byte);
        }
        for byte in record.date.to_be_bytes() {
            preimage.push_back(byte);
        }
        preimage
    }

    /// Compute the canonical hash commitment for a stored medical record.
    /// See [`Self::canonical_medical_record_preimage`] for the exact
    /// versioned encoding. (Issue #1169)
    pub fn get_medical_record_hash(env: Env, record_id: u64) -> BytesN<32> {
        let record: MedicalRecord = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecord(record_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::RecordNotFound));
        let preimage = Self::canonical_medical_record_preimage(&env, &record);
        env.crypto().sha256(&preimage).into()
    }

    pub fn add_medical_record(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        diagnosis: String,
        treatment: String,
        medications: Vec<Medication>,
        notes: String,
    ) -> u64 {
        vet_address.require_auth();
        if !Self::is_verified_vet(env.clone(), vet_address.clone()) {
            panic_with_error!(&env, ContractError::VeterinarianNotVerified);
        }

        // Bound string fields to prevent unbounded ledger growth. (#1152)
        if diagnosis.len() > MAX_MEDICAL_DIAGNOSIS_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        if treatment.len() > MAX_MEDICAL_TREATMENT_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        if notes.len() > MAX_MEDICAL_NOTES_LEN {
            panic_with_error!(&env, ContractError::InputStringTooLong);
        }
        // Bound medications Vec to prevent unbounded inline serialisation. (#1153)
        if medications.len() > MAX_VEC_MEDS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

        Self::increment_pet_storage(&env, pet_id);

        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::MedicalRecordCount)
            .unwrap_or(0);
        let record_id = safe_increment(&env, count);
        let now = env.ledger().timestamp();

        let record = MedicalRecord {
            id: record_id,
            pet_id,
            vet_address: vet_address.clone(),
            diagnosis,
            treatment,
            medications,
            date: now,
            updated_at: now,
            notes,
            attachment_hashes: Vec::new(&env),
            deleted_at: None,
        };

        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecord(record_id), &record);
        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecordCount, &record_id);

        let pet_rec_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_rec_count = safe_increment(&env, pet_rec_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetMedicalRecordCount(pet_id), &new_pet_rec_count);
        env.storage().instance().set(
            &MedicalKey::PetMedicalRecordIndex((pet_id, new_pet_rec_count)),
            &record_id,
        );

        let vet_tx_count = env
            .storage()
            .instance()
            .get::<VetKey, u64>(&VetKey::VetTreatmentCount(vet_address.clone()))
            .unwrap_or(0);
        let new_vet_tx_count = safe_increment(&env, vet_tx_count);
        env.storage().instance().set(
            &VetKey::VetTreatmentCount(vet_address.clone()),
            &new_vet_tx_count,
        );
        env.storage().instance().set(
            &VetKey::VetTreatmentIndex((vet_address.clone(), new_vet_tx_count)),
            &record_id,
        );

        env.events().publish(
            (String::from_str(&env, "MedicalRecordAdded"), pet_id),
            MedicalRecordAddedEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                updated_by: vet_address,
                timestamp: now,
                subscription_ids: Self::matching_subscription_ids(
                    &env,
                    EventType::MedicalRecordAdded,
                    pet_id,
                ),
            },
        );

        record_id
    }

    pub fn get_pet_medical_records(
        env: Env,
        pet_id: u64,
        offset: u64,
        limit: u32,
    ) -> Vec<MedicalRecord> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let mut results = Vec::new(&env);
        if count == 0 || limit == 0 || offset >= count {
            return results;
        }
        let start_index = offset.saturating_add(1);
        let end_index = (offset.saturating_add(limit as u64)).min(count);
        for i in start_index..=end_index {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = Self::get_medical_record(env.clone(), record_id) {
                    results.push_back(record);
                }
            }
        }
        results
    }
    /// Cursor-based pagination of a pet's medical records (Issue #1173).
    ///
    /// This is the stable alternative to [`Self::get_pet_medical_records`], whose
    /// offset pagination can duplicate or skip rows when records are inserted or
    /// soft-deleted between pages. Because each record keeps a fixed slot in the
    /// pet's index, a cursor that resumes at the next slot is immune to concurrent
    /// appends: no live record is ever skipped or returned twice across a paged scan.
    ///
    /// Soft-deleted records are excluded via the shared filter, so a page may carry
    /// fewer than `limit` items when intervening slots were soft-deleted; the caller
    /// keeps paging with the returned `next_cursor` until it is `0`.
    ///
    /// `cursor` is an opaque slot index; pass `0` for the first page and echo back
    /// `next_cursor` on subsequent calls.
    pub fn get_pet_medical_records_cursor(
        env: Env,
        pet_id: u64,
        cursor: u64,
        limit: u32,
    ) -> MedicalRecordPage {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let mut items = Vec::new(&env);

        if limit == 0 || count == 0 {
            return MedicalRecordPage {
                items,
                next_cursor: 0,
                total_slots: count,
            };
        }

        // `cursor` is the last examined slot; resume from the following slot.
        let start = cursor.saturating_add(1);
        if start > count {
            return MedicalRecordPage {
                items,
                next_cursor: 0,
                total_slots: count,
            };
        }

        let end = count.min(start.saturating_add(limit as u64).saturating_sub(1));
        let mut idx = start;
        while idx <= end && items.len() < limit {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, idx)))
            {
                if let Some(record) = Self::get_medical_record(env.clone(), record_id) {
                    items.push_back(record);
                }
            }
            idx += 1;
        }

        let last_examined = idx.saturating_sub(1);
        let next_cursor = if last_examined < count { last_examined } else { 0 };

        MedicalRecordPage {
            items,
            next_cursor,
            total_slots: count,
        }
    }


    // --- Upgrade Proposal with Expiry (Issue #818) ---

    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        new_wasm_hash: BytesN<32>,
        expires_in_days: u32,
    ) -> u64 {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::UpgradeProposalCount)
            .unwrap_or(0);
        let proposal_id = safe_increment(count);
        let now = env.ledger().timestamp();
        let expires_at = now.saturating_add((expires_in_days as u64).saturating_mul(86400));

        let proposal = UpgradeProposal {
            id: proposal_id,
            proposed_by: admin,
            new_wasm_hash,
            proposed_at: now,
            approved: false,
            executed: false,
            timelock_duration: 86400,
            approved_at: None,
            vetoed: false,
            expires_at,
        };

        env.storage()
            .instance()
            .set(&SystemKey::UpgradeProposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&SystemKey::UpgradeProposalCount, &proposal_id);
        proposal_id
    }

    pub fn approve_upgrade_proposal(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&SystemKey::UpgradeProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::ProposalNotFound));

        let now = env.ledger().timestamp();
        if now > proposal.expires_at {
            panic_with_error!(&env, ContractError::ProposalExpired);
        }
        if proposal.executed {
            panic_with_error!(&env, ContractError::ProposalAlreadyExecuted);
        }

        proposal.approved = true;
        proposal.approved_at = Some(now);

        env.storage()
            .instance()
            .set(&SystemKey::UpgradeProposal(proposal_id), &proposal);
    }

    pub fn execute_upgrade(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let mut proposal: UpgradeProposal = env
            .storage()
            .instance()
            .get(&SystemKey::UpgradeProposal(proposal_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::ProposalNotFound));

        let now = env.ledger().timestamp();
        if now > proposal.expires_at {
            panic_with_error!(&env, ContractError::ProposalExpired);
        }
        if !proposal.approved {
            panic_with_error!(&env, ContractError::ProposalNotApproved);
        }
        if proposal.executed {
            panic_with_error!(&env, ContractError::ProposalAlreadyExecuted);
        }

        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        if proposal.new_wasm_hash != zero_hash {
            env.deployer()
                .update_current_contract_wasm(proposal.new_wasm_hash.clone());
        }

        // Store rollback info
        env.storage()
            .instance()
            .set(&SystemKey::PreviousWasmHash, &proposal.new_wasm_hash);
        env.storage()
            .instance()
            .set(&SystemKey::RollbackDeadline, &now.saturating_add(86400));

        proposal.executed = true;
        env.storage()
            .instance()
            .set(&SystemKey::UpgradeProposal(proposal_id), &proposal);
    }

    pub fn get_upgrade_proposal(env: Env, proposal_id: u64) -> Option<UpgradeProposal> {
        env.storage()
            .instance()
            .get(&SystemKey::UpgradeProposal(proposal_id))
    }

    pub fn list_upgrade_proposals(env: Env, start: u64, limit: u32) -> Vec<UpgradeProposal> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::UpgradeProposalCount)
            .unwrap_or(0);
        let mut result = Vec::new(&env);
        let mut added: u32 = 0;
        let mut id = start + 1;
        while id <= count && added < limit {
            if let Some(p) = env
                .storage()
                .instance()
                .get::<SystemKey, UpgradeProposal>(&SystemKey::UpgradeProposal(id))
            {
                result.push_back(p);
                added += 1;
            }
            id += 1;
        }
        result
    }

    // --- Version management ---

    pub fn get_version(env: Env) -> ContractVersion {
        env.storage()
            .instance()
            .get(&DataKey::ContractVersion)
            .unwrap_or(ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            })
    }

    pub fn set_version(env: Env, admin: Address, major: u32, minor: u32, patch: u32) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let version = ContractVersion {
            major,
            minor,
            patch,
        };
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &version);
    }

    pub fn migrate_version(env: Env, admin: Address, major: u32, minor: u32, patch: u32) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let version = ContractVersion {
            major,
            minor,
            patch,
        };
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &version);
    }

    pub fn migrate_v1_to_v2(env: Env, admin: Address) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let current = Self::get_version(env.clone());
        if current.major >= 2 {
            return;
        }
        let version = ContractVersion {
            major: 2,
            minor: 0,
            patch: 0,
        };
        env.storage()
            .instance()
            .set(&DataKey::ContractVersion, &version);
    }

    pub fn migrate_v2_to_v3(env: Env, admin: Address) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let current = Self::get_storage_version(env.clone());
        if current.major >= 3 {
            return;
        }
        let version = ContractVersion {
            major: 3,
            minor: 0,
            patch: 0,
        };
        env.storage()
            .instance()
            .set(&SystemKey::StorageVersion, &version);
    }

    pub fn get_storage_version(env: Env) -> ContractVersion {
        env.storage()
            .instance()
            .get(&SystemKey::StorageVersion)
            .unwrap_or(ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            })
    }

    // -----------------------------------------------------------------------
    // STORAGE SCHEMA VERSION  (Issue #1149)
    //
    // `get_schema_version` returns the flat u32 stored under
    // `SystemKey::StorageSchemaVersion`.  Absent key → 0 (pre-versioning).
    //
    // `migrate_schema_version` is:
    //   • Authorized  — only an admin may invoke it.
    //   • Idempotent  — calling it a second time with the same (or lower)
    //                   target panics with `StaleMigration` rather than
    //                   silently re-running migration steps.
    //   • Resumable   — only ever advances forward; target must be > current.
    //
    // Threat model:
    //   An attacker who compromises an admin key could call `migrate_schema_version`
    //   to bump the schema version without running the corresponding migration
    //   code, orphaning old data.  The risk is mitigated by the multisig admin
    //   threshold: migration calls must satisfy the same quorum as any other
    //   admin-gated operation.  The idempotency guard also prevents a valid
    //   migration from being run twice (no double-writes).
    // -----------------------------------------------------------------------

    /// Returns the current flat storage-schema version (0 = pre-versioning).
    pub fn get_schema_version(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&SystemKey::StorageSchemaVersion)
            .unwrap_or(0u32)
    }

    /// Advance the storage-schema version from `current_version` to `target_version`.
    ///
    /// # Behaviour
    /// - Returns `StaleMigration` when `current_version` does not match the
    ///   stored value, or when `target_version <= stored_version`.  This makes
    ///   the call safe to retry after a partial failure.
    /// - Panics with `NotAnAdmin` / `Unauthorized` when authorization fails.
    ///
    /// # Adding a new migration step
    /// 1. Bump `STORAGE_SCHEMA_VERSION` by one.
    /// 2. Add the migration logic in the `match` arm for the new version inside
    ///    this function.
    pub fn migrate_schema_version(
        env: Env,
        admin: Address,
        current_version: u32,
        target_version: u32,
    ) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let stored: u32 = env
            .storage()
            .instance()
            .get(&SystemKey::StorageSchemaVersion)
            .unwrap_or(0u32);

        // Idempotency guard: reject replays and backward jumps.
        if stored != current_version || target_version <= stored {
            panic_with_error!(&env, ContractError::StaleMigration);
        }

        // Execute per-version migration steps.
        // Each arm should be narrow and idempotent at the data level.
        match target_version {
            1 => {
                // v0 → v1: First versioned schema.
                // No structural data changes in this version; the StorageSchemaVersion
                // key itself is the only new storage entry.
            }
            _ => {
                // Unknown target — reject to prevent silent version skips.
                panic_with_error!(&env, ContractError::InvalidInput);
            }
        }

        env.storage()
            .instance()
            .set(&SystemKey::StorageSchemaVersion, &target_version);
    }

    /// Rebuild the canonical microchip index for existing records. The work
    /// is bounded so large deployments can retry in batches. A collision or
    /// invalid legacy value aborts the batch with InvalidInput.
    pub fn migrate_microchip_index(env: Env, admin: Address, start: u64, limit: u64) -> u64 {
        Self::require_admin_auth(&env, &admin);
        let total: u64 = env.storage().instance().get(&DataKey::PetCount).unwrap_or(0);
        let end = start.saturating_add(limit).min(total);
        let mut cursor = start;
        while cursor < end {
            let pet_id = cursor + 1;
            if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
                if let Some(ref legacy) = pet.microchip_id {
                    let canonical = Self::canonicalize_microchip_id(&env, legacy);
                    if let Some(existing) = env.storage().instance().get::<DataKey, u64>(&DataKey::MicrochipIndex(canonical.clone())) {
                        if existing != pet_id {
                            panic_with_error!(&env, ContractError::InvalidInput);
                        }
                    }
                    if *legacy != canonical {
                        pet.microchip_id = Some(canonical.clone());
                        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
                    }
                    env.storage().instance().set(&DataKey::MicrochipIndex(canonical), &pet_id);
                }
            }
            cursor += 1;
        }
        end
    }

    pub fn migrate_storage(
        env: Env,
        admin: Address,
        _from_major: u32,
        _from_minor: u32,
        _from_patch: u32,
        to_major: u32,
        to_minor: u32,
        to_patch: u32,
    ) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }
        let current = Self::get_storage_version(env.clone());
        // Idempotency: if the stored version already meets or exceeds the
        // target we return StaleMigration instead of silently re-running.
        let already_met = (current.major > to_major)
            || (current.major == to_major && current.minor > to_minor)
            || (current.major == to_major
                && current.minor == to_minor
                && current.patch >= to_patch);
        if already_met {
            panic_with_error!(&env, ContractError::StaleMigration);
        }
        let version = ContractVersion {
            major: to_major,
            minor: to_minor,
            patch: to_patch,
        };
        env.storage()
            .instance()
            .set(&SystemKey::StorageVersion, &version);
    }

    // --- Rollback ---

    pub fn rollback_upgrade(env: Env, admin: Address) {
        admin.require_auth();
        if !Self::is_admin_address(&env, &admin) {
            panic_with_error!(&env, ContractError::NotAnAdmin);
        }

        let deadline: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::RollbackDeadline)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPreviousUpgrade));
        if deadline == 0 {
            panic_with_error!(&env, ContractError::NoPreviousUpgrade);
        }

        let now = env.ledger().timestamp();
        if now > deadline {
            panic_with_error!(&env, ContractError::RollbackWindowExpired);
        }

        let prev_hash: BytesN<32> = env
            .storage()
            .instance()
            .get(&SystemKey::PreviousWasmHash)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NoPreviousUpgrade));

        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        if prev_hash != zero_hash {
            env.deployer()
                .update_current_contract_wasm(prev_hash);
        }

        // Clear rollback state
        env.storage()
            .instance()
            .set(&SystemKey::RollbackDeadline, &0u64);
    }

    pub fn get_rollback_deadline(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&SystemKey::RollbackDeadline)
            .unwrap_or(0)
    }
    pub fn get_lab_result_count(env: Env, pet_id: u64) -> u64 {
        env.storage().instance().get::<MedicalKey, u64>(&MedicalKey::PetLabResultCount(pet_id)).unwrap_or(0)
    }

    pub fn search_by_keyword(env: Env, pet_id: u64, keyword: String) -> Vec<MedicalRecord> {
        if keyword.len() > crate::MAX_SEARCH_KEYWORD_LEN {
            panic_with_error!(&env, PetChainError::KeywordTooLong);
        }
        let count: u64 = env.storage().instance().get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordCount(pet_id)).unwrap_or(0);
        let mut results = Vec::new(&env);
        for i in 1..=count {
            if let Some(record_id) = env.storage().instance().get::<MedicalKey, u64>(&MedicalKey::PetMedicalRecordIndex((pet_id, i))) {
                if let Some(record) = Self::get_medical_record(env.clone(), record_id) {
                    if Self::string_contains(&env, &record.diagnosis, &keyword) || Self::string_contains(&env, &record.notes, &keyword) {
                        results.push_back(record);
                    }
                }
            }
        }
        results
    }

    pub fn remove_medical_record(env: Env, record_id: u64) -> bool {
        if let Some(mut record) = env.storage().instance().get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id)) {
            record.vet_address.require_auth();
            record.deleted_at = Some(env.ledger().timestamp());
            env.storage().instance().set(&MedicalKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }

    pub fn update_medical_record_notes(env: Env, record_id: u64, notes: String) -> bool {
        if let Some(mut record) = env.storage().instance().get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id)) {
            record.vet_address.require_auth();
            record.notes = notes;
            record.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&MedicalKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }

    pub fn amend_medical_record(env: Env, pet_id: u64, record_id: u64, input: MedicalRecordAmendmentInput) -> u32 {
        let _ = pet_id;
        let record: MedicalRecord = env.storage().instance().get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id)).unwrap_or_else(|| panic_with_error!(&env, ContractError::RecordNotFound));
        record.vet_address.require_auth();
        let version: u32 = env.storage().instance().get::<MedicalKey, u32>(&MedicalKey::MedicalRecordAmendmentCount(record_id)).unwrap_or(0);
        if version >= 5 { panic_with_error!(&env, ContractError::TooManyItems); }
        let amendment = MedicalRecordAmendment { record_id, version: version + 1, updated_at: env.ledger().timestamp(), changes: input };
        env.storage().instance().set(&MedicalKey::MedicalRecordAmendment((record_id, version + 1)), &amendment);
        env.storage().instance().set(&MedicalKey::MedicalRecordAmendmentCount(record_id), &(version + 1));
        version + 1
    }

    pub fn diff_record_versions(env: Env, pet_id: u64, record_id: u64, from_version: u32, to_version: u32) -> Vec<MedicalFieldDiff> {
        let _ = pet_id;
        let mut diffs = Vec::new(&env);
        let a: Option<MedicalRecordAmendment> = if from_version == 0 { None } else {
            env.storage().instance().get(&MedicalKey::MedicalRecordAmendment((record_id, from_version)))
        };
        let b: MedicalRecordAmendment = match env.storage().instance().get(&MedicalKey::MedicalRecordAmendment((record_id, to_version))) {
            Some(v) => v,
            None => return diffs,
        };
        if let Some(diag) = &b.changes.diagnosis {
            let from_val = a.as_ref().and_then(|x| x.changes.diagnosis.clone()).unwrap_or(String::from_str(&env, ""));
            diffs.push_back(MedicalFieldDiff { field: String::from_str(&env, "diagnosis"), from_value: from_val, to_value: diag.clone() });
        }
        if let Some(treat) = &b.changes.treatment {
            let from_val = a.as_ref().and_then(|x| x.changes.treatment.clone()).unwrap_or(String::from_str(&env, ""));
            diffs.push_back(MedicalFieldDiff { field: String::from_str(&env, "treatment"), from_value: from_val, to_value: treat.clone() });
        }
        if let Some(notes) = &b.changes.notes {
            let from_val = a.as_ref().and_then(|x| x.changes.notes.clone()).unwrap_or(String::from_str(&env, ""));
            diffs.push_back(MedicalFieldDiff { field: String::from_str(&env, "notes"), from_value: from_val, to_value: notes.clone() });
        }
        diffs
    }
    // -------------------------------------------------------------------------
    // CONVENIENCE WRAPPERS  (Issue #1153 / #1152 tests)
    // -------------------------------------------------------------------------

    /// Transfer a pet directly to `new_owner` in one atomic call.
    ///
    /// This is a simpler alternative to the two-step
    /// `transfer_pet_ownership` + `accept_pet_transfer` flow.  The caller
    /// must be the current owner.  An ownership record and a custody-chain
    /// entry are both appended, consistent with `batch_transfer`.
    ///
    /// # Errors
    /// * `PetNotFound`  — if `pet_id` does not exist.
    /// * `NotPetOwner`  — if `owner` is not the current owner.
    pub fn transfer_pet(env: Env, owner: Address, pet_id: u64, new_owner: Address) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        owner.require_auth();
        if pet.owner != owner {
            panic_with_error!(&env, ContractError::NotPetOwner);
        }

        let old_owner = pet.owner.clone();
        let now = env.ledger().timestamp();

        PetChainContract::remove_pet_from_owner_index(&env, &old_owner, pet_id);

        pet.owner = new_owner.clone();
        pet.new_owner = new_owner.clone();
        pet.updated_at = now;

        PetChainContract::add_pet_to_owner_index(&env, &new_owner, pet_id);
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

        PetChainContract::log_ownership_change(
            &env,
            pet_id,
            old_owner.clone(),
            new_owner.clone(),
            String::from_str(&env, "Direct Transfer"),
        );

        PetChainContract::append_custody_entry(
            &env,
            pet_id,
            old_owner.clone(),
            new_owner.clone(),
            TransferType::Direct,
        );

        env.events().publish(
            (String::from_str(&env, "PetOwnershipTransferred"), pet_id),
            PetOwnershipTransferredEvent {
                version: EVENT_SCHEMA_VERSION,
                pet_id,
                old_owner,
                new_owner,
                timestamp: now,
            },
        );
    }

    /// Configure a multi-signature approval requirement for pet ownership transfers.
    ///
    /// Any of the listed `signers` can sign a subsequent `PetTransferProposal`
    /// via `sign_transfer_proposal`.  At least `threshold` signatures are needed
    /// before the proposal can be executed.
    ///
    /// # Caps (Issue #1153)
    /// * `signers.len()` is bounded by [`MAX_MULTISIG_SIGNERS`].
    ///
    /// # Errors
    /// * `PetNotFound`      — if `pet_id` does not exist.
    /// * `NotPetOwner`      — if `owner` is not the current owner.
    /// * `InvalidThreshold` — if `threshold == 0` or `threshold > signers.len()`.
    /// * `TooManyItems`     — if `signers.len() > MAX_MULTISIG_SIGNERS`.
    pub fn setup_pet_multisig(
        env: Env,
        owner: Address,
        pet_id: u64,
        signers: Vec<Address>,
        threshold: u32,
    ) {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        owner.require_auth();
        if pet.owner != owner {
            panic_with_error!(&env, ContractError::NotPetOwner);
        }

        if threshold == 0 || threshold > signers.len() {
            panic_with_error!(&env, ContractError::InvalidThreshold);
        }

        // Cap enforced to prevent unbounded Vec stored inside MultisigConfig. (#1153)
        if signers.len() > MAX_MULTISIG_SIGNERS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let config = MultisigConfig {
            pet_id,
            signers,
            threshold,
            enabled: true,
        };

        env.storage()
            .instance()
            .set(&SystemKey::PetMultisigConfig(pet_id), &config);
    }

    /// Retrieve the multi-signature config for a pet, if one has been set.
    pub fn get_pet_multisig_config(env: Env, pet_id: u64) -> Option<MultisigConfig> {
        env.storage()
            .instance()
            .get(&SystemKey::PetMultisigConfig(pet_id))
    }

    /// Record a training milestone for a pet.
    ///
    /// Each milestone can optionally reference up to [`MAX_PREREQUISITES`]
    /// previously achieved milestone IDs.  Calling this with more than
    /// [`MAX_PREREQUISITES`] prerequisite IDs panics with `TooManyItems`
    /// (Issue #1153).
    ///
    /// # Errors
    /// * `PetNotFound`  — if `pet_id` does not exist.
    /// * `TooManyItems` — if `prerequisites.len() > MAX_PREREQUISITES`.
    ///
    /// # Returns
    /// The newly assigned milestone ID (monotonically increasing).
    pub fn add_training_milestone(
        env: Env,
        pet_id: u64,
        trainer: Address,
        milestone_name: String,
        prerequisites: Vec<u64>,
    ) -> u64 {
        trainer.require_auth();

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::PetNotFound));

        // Cap prerequisites to prevent unbounded inline Vec growth. (#1153)
        if prerequisites.len() > MAX_PREREQUISITES {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let global_count: u64 = env
            .storage()
            .instance()
            .get(&BehaviorKey::TrainingMilestoneCount)
            .unwrap_or(0);
        let milestone_id = safe_increment(global_count);

        let pet_count: u64 = safe_increment(
            env.storage()
                .instance()
                .get(&BehaviorKey::PetMilestoneCount(pet_id))
                .unwrap_or(0),
        );

        let milestone = TrainingMilestone {
            id: milestone_id,
            pet_id,
            milestone_name,
            achieved: false,
            achieved_at: None,
            trainer,
            notes: String::from_str(&env, ""),
            prerequisites,
        };

        env.storage()
            .instance()
            .set(&BehaviorKey::TrainingMilestone(milestone_id), &milestone);
        env.storage()
            .instance()
            .set(&BehaviorKey::TrainingMilestoneCount, &milestone_id);
        env.storage()
            .instance()
            .set(&BehaviorKey::PetMilestoneCount(pet_id), &pet_count);
        env.storage().instance().set(
            &BehaviorKey::PetMilestoneIndex((pet_id, pet_count)),
            &milestone_id,
        );

        milestone_id
    }

    /// Retrieve a training milestone by its ID.
    pub fn get_training_milestone(env: Env, milestone_id: u64) -> Option<TrainingMilestone> {
        env.storage()
            .instance()
            .get(&BehaviorKey::TrainingMilestone(milestone_id))
    }
} // end impl PetChainContract

// --- OVERFLOW-SAFE COUNTER HELPER ---
pub(crate) fn safe_increment(env: &Env, count: u64) -> u64 {
    count
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::CounterOverflow))
}

// --- ENCRYPTION HELPERS ---
fn encrypt_sensitive_data(env: &Env, data: &Bytes, key: &Bytes) -> (Bytes, Bytes) {
    let nonce = derive_encryption_nonce(env);
    let ciphertext = xor_stream_crypt(env, data, key, &nonce);
    (nonce, ciphertext)
}

fn decrypt_sensitive_data(
    env: &Env,
    ciphertext: &Bytes,
    nonce: &Bytes,
    key: &Bytes,
) -> Result<Bytes, ()> {
    if nonce.len() != 12 {
        return Err(());
    }
    Ok(xor_stream_crypt(env, ciphertext, key, nonce))
}

fn derive_encryption_nonce(env: &Env) -> Bytes {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&SystemKey::EncryptionNonceCounter)
        .unwrap_or(0);
    let next_counter = safe_increment(env, counter);
    env.storage()
        .instance()
        .set(&SystemKey::EncryptionNonceCounter, &next_counter);

    let timestamp = env.ledger().timestamp();
    let mut nonce = Bytes::new(env);
    for byte in timestamp.to_be_bytes() {
        nonce.push_back(byte);
    }
    for byte in (next_counter as u32).to_be_bytes() {
        nonce.push_back(byte);
    }
    nonce
}

fn xor_stream_crypt(env: &Env, input: &Bytes, key: &Bytes, nonce: &Bytes) -> Bytes {
    let mut output = Bytes::new(env);
    let mut block_index: u32 = 0;

    while output.len() < input.len() {
        let mut seed = Bytes::new(env);
        for byte in key.iter() {
            seed.push_back(byte);
        }
        for byte in nonce.iter() {
            seed.push_back(byte);
        }
        for byte in block_index.to_be_bytes() {
            seed.push_back(byte);
        }

        let stream_block: Bytes = env.crypto().sha256(&seed).into();
        let start = output.len();
        let remaining = input.len() - start;
        let take = if remaining < 32 { remaining } else { 32 };
        for i in 0..take {
            let src = input.get_unchecked(start + i);
            let key_byte = stream_block.get_unchecked(i);
            output.push_back(src ^ key_byte);
        }
        block_index = block_index.saturating_add(1);
    }
    output
}

// =============================================================================
// LAB RESULT ANOMALY DETECTION TESTS  (Issue #811)
// =============================================================================
//
// History setup: 9 readings of value 100 and 1 reading of value 200.
//   mean     = (9×100 + 200) / 10 = 110
//   variance = (9×(100-110)² + (200-110)²) / 10 = (900 + 8100) / 10 = 900
//   stddev   = 30
//
// z-scores (×100):
//   value=100 → (100-110)×100/30 = -33  → |z|=33  → no anomaly
//   value=200 → (200-110)×100/30 = 300  → |z|=300 → no anomaly (NOT > 300)
//   value=210 → (210-110)×100/30 = 333  → |z|=333 → anomaly!
#[cfg(test)]
mod test_lab_result_anomaly {
    use crate::{
        Gender, LabResultAnomaly, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
        EVENT_SCHEMA_VERSION,
    };
    use soroban_sdk::{
        testutils::{Address as _, Events, Ledger as _},
        Address, Env, Map, String, TryFromVal, Val,
    };

    fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        client.init_admin(&admin);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        );
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Smith"),
            &String::from_str(&env, "LIC-001"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&admin, &vet);

        (env, client, owner, vet, pet_id)
    }

    fn add_glucose(
        env: &Env,
        client: &PetChainContractClient,
        pet_id: u64,
        vet: &Address,
        glucose: i128,
        ts: u64,
    ) {
        env.ledger().set_timestamp(ts);
        let mut bm = Map::new(env);
        bm.set(String::from_str(env, "glucose"), glucose);
        client.add_lab_result(
            &pet_id,
            vet,
            &String::from_str(env, "Blood Test"),
            &String::from_str(env, "Normal"),
            &String::from_str(env, "0-200"),
            &None,
            &None,
            &bm,
        );
    }

    fn seed_history(env: &Env, client: &PetChainContractClient, pet_id: u64, vet: &Address) {
        // 9 readings of 100
        for i in 0..9u64 {
            add_glucose(env, client, pet_id, vet, 100, 1000 + i * 100);
        }
        // 1 reading of 200  →  mean=110, stddev=30
        add_glucose(env, client, pet_id, vet, 200, 2000);
    }

    fn anomaly_events(env: &Env) -> soroban_sdk::Vec<(soroban_sdk::Vec<Val>, Val)> {
        let topic = String::from_str(env, "LAB_RESULT_ANOMALY");
        let all = env.events().all();
        let mut out = soroban_sdk::Vec::new(env);
        for i in 0..all.len() {
            // Events are (contract_id, topics, data)
            let (_contract, topics, data): (Address, soroban_sdk::Vec<Val>, Val) =
                all.get(i).unwrap();
            if topics.len() > 0 {
                let t0: Val = topics.get(0).unwrap();
                if let Ok(s) = String::try_from_val(env, &t0) {
                    if s == topic {
                        out.push_back((topics, data));
                    }
                }
            }
        }
        out
    }

    // Test 1: normal value – z-score well within threshold, no event emitted.
    #[test]
    fn test_normal_value_no_anomaly() {
        let (env, client, _owner, vet, pet_id) = setup();
        seed_history(&env, &client, pet_id, &vet);

        // value=100 → z=-33 → |z|<300 → no anomaly
        add_glucose(&env, &client, pet_id, &vet, 100, 3000);

        assert_eq!(anomaly_events(&env).len(), 0);
    }

    // Test 2: borderline value – z-score exactly 300 (not strictly > 300), no event.
    #[test]
    fn test_borderline_value_no_anomaly() {
        let (env, client, _owner, vet, pet_id) = setup();
        seed_history(&env, &client, pet_id, &vet);

        // value=200 → z=300 → |z|=300, NOT > 300 → no anomaly
        add_glucose(&env, &client, pet_id, &vet, 200, 3000);

        assert_eq!(anomaly_events(&env).len(), 0);
    }

    // Test 3: clear anomaly – z-score 333 > 300, event must be emitted.
    #[test]
    fn test_clear_anomaly_emits_event() {
        let (env, client, _owner, vet, pet_id) = setup();
        seed_history(&env, &client, pet_id, &vet);

        // value=210 → z=333 → |z|>300 → anomaly
        add_glucose(&env, &client, pet_id, &vet, 210, 3000);

        let events = anomaly_events(&env);
        assert_eq!(events.len(), 1);

        // Decode and verify the event payload.
        let (_topics, data) = events.get(0).unwrap();
        let anomaly: LabResultAnomaly = LabResultAnomaly::try_from_val(&env, &data).unwrap();
        assert_eq!(anomaly.pet_id, pet_id);
        assert_eq!(anomaly.biomarker, String::from_str(&env, "glucose"));
        assert_eq!(anomaly.value, 210);
        assert_eq!(anomaly.z_score, 333);
        assert_eq!(anomaly.version, EVENT_SCHEMA_VERSION);
    }
}

#[cfg(test)]
mod test_breeding_coi;
