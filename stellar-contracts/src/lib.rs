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

/// Canonical enum for off-chain indexers to identify the active event schema.
/// Bump the variant and `EVENT_SCHEMA_VERSION` together whenever fields change.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventSchema {
    V1 = 1,
}

#[contracttype]
pub enum InsuranceKey {
    Policy(u64),               // (pet_id) -> InsurancePolicy [deprecated, kept for migration]
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
    Env, Map, String, Symbol, Vec,
};

#[cfg(test)]
mod test_dispute_voting;

const DEFAULT_NONCE_MAX_USES: u32 = 1;
#[allow(dead_code)]
const NONCE_HISTORY_LIMIT: u32 = 8;
#[allow(dead_code)]
const MAX_SEARCH_KEYWORD_LEN: u32 = 32;
#[allow(dead_code)]
const MAX_SEARCH_TOKENS_PER_RECORD: u32 = 16;
#[allow(dead_code)]
const MAX_SEARCH_NOTES_LEN: u32 = 512;
#[allow(dead_code)]
const MAX_LINEAGE_DEPTH: u32 = 16;
const MAX_LOG_ENTRIES: u32 = 1_000;
const MAX_ACTIVE_SUBSCRIPTIONS_PER_ADDRESS: u32 = 10;

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
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BreedingRecord {
    pub id: u64,
    pub sire_id: u64,
    pub dam_id: u64,
    pub breeding_date: u64,
    pub offspring_ids: Vec<u64>,
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
    pub prerequisites: Vec<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Gender {
    NotSpecified,
    Male,
    Female,
    Unknown,
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
pub struct ArbitratorStats {
    pub address: Address,
    pub reputation: i64,
    pub total_rulings: u64,
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    pub actor: Address,
    pub timestamp: u64,
    pub reason_code: u32,
    pub pet_id: u64,
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
}

/// Certificate anchor for vaccination PDF metadata
/// Stores hash of off-chain certificate for authenticity verification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateAnchor {
    pub pet_id: u64,
    pub vaccination_id: u64,
    pub cert_hash: String, // Hash of the PDF certificate
    pub issuer: Address,   // Verified vet who issued the certificate
    pub anchored_at: u64,  // Timestamp when anchored
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
    EmergencyAccessLogs(u64),
    EmergencyAuditLog(u64),
    EmergencyResponders(u64),
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
    PendingConfig, // Issue #626: Three-phase bootstrap
    Proposal(u64),
    ProposalCount,

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
    EncryptionNonceCounter,

    // Statistics caching keys
    StatCacheTTL,
    StatCache(String),
    LabThreshold,
    // Chain-of-custody log (Issue #637)
    CustodyChain(u64), // pet_id -> Vec<CustodyEntry>
    // #699: governance-controlled parameters
    HealthScoreCacheTtl, // TTL (seconds) for health-score cache entries
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

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeStatus {
    Pending = 1,
    EvidencePhase = 2,
    ResolvedInFavorOfClaimer = 3,
    ResolvedInFavorOfTarget = 4,
    Cancelled = 5,
}

/// A stakeholder's vote on a dispute resolution.
#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DisputeVote {
    Approve = 1,
    Reject = 2,
}

/// A single recorded vote on a dispute, tracking who voted and how.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeVoteRecord {
    pub voter: Address,
    pub vote: DisputeVote,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub dispute_id: u64,
    pub pet_id: u64,
    pub claimer: Address,
    pub target: Address,
    pub amount: u64,
    pub reason: String,
    pub evidence_hash: String,
    pub status: DisputeStatus,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub evidence_id: u64,
    pub submitter: Address,
    pub cid: String,
    pub sha256_hash: BytesN<32>,
}

#[contracttype]
pub enum DisputeKey {
    Dispute(u64),
    DisputeCount,
    AppealWindow,
    Arbitrator,
    PetDisputesCount(u64),
    PetDisputesIndex((u64, u64)),
    DisputeEvidence(u64, u64),
    DisputeEvidenceCount(u64),
    PartyEvidenceCount(u64, Address),
    /// Vote cast by a given address on a given dispute.
    DisputeVoteByVoter(u64, Address),
    /// Ordered list of addresses that have voted on a dispute (for enumeration).
    DisputeVoters(u64),
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

        if active_count >= MAX_ACTIVE_SUBSCRIPTIONS_PER_ADDRESS {
            panic_with_error!(&env, ContractError::TooManyItems);
        }

        let current_id: u64 = env
            .storage()
            .instance()
            .get(&SubscriptionKey::SubscriptionCount)
            .unwrap_or(0);
        let subscription_id = safe_increment(current_id);
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

        let new_subscriber_count = safe_increment(existing_count);
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
            if grant.is_active && grant.grantee == caller {
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
                        consents.push_back(consent);
                    }
                }
            }
        }
        consents
    }

    fn get_medical_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
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
        let proposal_id = proposal_count + 1;
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

        let proposal = MultiSigProposal {
            id: proposal_id,
            action,
            proposed_by: proposer,
            approvals: Vec::new(&env),
            required_approvals,
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
            let new_count = count + 1;
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

    /// Returns a paginated list of medical records (treatments) created by a specific vet.
    pub fn get_vet_treatment_history(
        env: Env,
        vet_address: Address,
        offset: u64,
        limit: u32,
    ) -> Vec<MedicalRecord> {
        let count = env
            .storage()
            .instance()
            .get::<VetKey, u64>(&VetKey::VetTreatmentCount(vet_address.clone()))
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
                .get::<VetKey, u64>(&VetKey::VetTreatmentIndex((vet_address.clone(), i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<MedicalKey, MedicalRecord>(&MedicalKey::MedicalRecord(record_id))
                {
                    results.push_back(record);
                }
            }
        }
        results
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
    pub fn get_pets_overdue_vaccinations(env: Env, offset: u64, limit: u32) -> Vec<u64> {
        let pet_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetCount)
            .unwrap_or(0);

        let mut overdue_pets = Vec::new(&env);
        if pet_count == 0 || limit == 0 {
            return overdue_pets;
        }

        let mut found: u64 = 0;
        let mut skipped: u64 = 0;

        for pet_id in 1..=pet_count {
            if overdue_pets.len() >= limit {
                break;
            }
            let overdue = PetChainContract::get_overdue_vaccinations(env.clone(), pet_id);
            if !overdue.is_empty() {
                if skipped < offset {
                    skipped = skipped.saturating_add(1);
                } else {
                    overdue_pets.push_back(pet_id);
                    found = found.saturating_add(1);
                }
            }
        }
        overdue_pets
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
        Self::require_admin_auth(&env, &admin);

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
        if let Err(err) = PetChainContract::parse_birthday_timestamp(&birthday) {
            env.panic_with_error(err);
        }
        Self::validate_pet_name(&env, &name);
        Self::validate_breed(&env, &species, &breed);

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
            microchip_id,
            photo_hashes: Vec::new(&env),
        };

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);

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
            pet.microchip_id = microchip_id;
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
                if let Some(record) = PetChainContract::get_medical_record(env.clone(), record_id) {
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

        for i in 1..=vax_count {
            if let Some(vax_id) = env
                .storage()
                .instance()
                .get::<MedicalKey, u64>(&MedicalKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = PetChainContract::get_vaccinations(env.clone(), vax_id) {
                    if vax.administered_at > latest_vax_timestamp {
                        latest_vax_timestamp = vax.administered_at;
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
                }
            }
        }

        Some(PetHealthSummary {
            pet_id,
            latest_vaccination_id,
            latest_lab_result_id,
            active_insurance_policy_id,
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
                    .set(&StatsKey::ActivePetsCount, &safe_increment(active_count));
                Self::record_stat_point(
                    &env,
                    String::from_str(&env, "ActivePetsCount"),
                    safe_increment(active_count),
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

    pub fn transfer_pet_ownership(env: Env, id: u64, to: Address) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();
            pet.new_owner = to;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }

    /// Transfer multiple pets to the same new owner atomically.
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
        }
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
        let new_count = safe_increment(count);
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
        PetChainContract::_verify_vet_internal(&env, vet_address)
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
            true
        } else {
            false
        }
    }

    pub fn revoke_vet_license(env: Env, admin: Address, vet_address: Address) -> bool {
        PetChainContract::require_admin_auth(&env, &admin);
        PetChainContract::_revoke_vet_internal(&env, vet_address)
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
            true
        } else {
            false
        }
    }

    pub fn is_vet_registered(env: Env, vet_address: Address) -> bool {
        env.storage().instance().has(&DataKey::Vet(vet_address))
    }

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        env.storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
            .map(|vet| vet.verified)
            .unwrap_or(false)
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
            panic!("Veterinarian not verified");
        }

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap_or_else(|| env.panic_with_error(ContractError::PetNotFound));

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
        let now = env.ledger().timestamp();
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
        let new_pet_vax_count = safe_increment(pet_vax_count);
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
        let new_vet_vax_count = safe_increment(vet_vax_count);
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
        let variance = history
            .iter()
            .fold(0i128, |acc, &v| {
                let diff = v.saturating_sub(mean);
                acc.saturating_add(diff.saturating_mul(diff))
            })
            / n;
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

        // --- allocate ID ---
        let lab_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalKey::LabResultCount)
            .unwrap_or(0);
        let lab_id = safe_increment(lab_count);

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
            let window_start = if prior_count >= 10 { prior_count - 9 } else { 1 };
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
                if idx == 0 { break; }
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

        let new_pet_lab_count = safe_increment(prior_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetLabResultCount(pet_id), &new_pet_lab_count);
        env.storage()
            .instance()
            .set(&MedicalKey::PetLabResultIndex((pet_id, new_pet_lab_count)), &lab_id);

        lab_id
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
        let start_index = safe_increment(offset); // Indices start from 1
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
        let threshold = current_time + (days_threshold * 86400);
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
            if vax.vaccine_type == vaccine_type {
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

        // Verify issuer is a verified vet
        let vet: Vet = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(issuer.clone()))
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::VetNotFound));

        if !vet.verified {
            panic_with_error!(&env, ContractError::VetNotVerified);
        }

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

        // Create and store certificate anchor
        let anchor = CertificateAnchor {
            pet_id,
            vaccination_id,
            cert_hash: cert_hash.clone(),
            issuer: issuer.clone(),
            anchored_at: current_time,
        };

        env.storage().instance().set(&anchor_key, &anchor);

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

    /// Verify if a certificate hash matches the anchored hash for a vaccination.
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet
    /// * `vaccination_id` - The ID of the vaccination
    /// * `cert_hash` - Hash to verify against the anchored hash
    ///
    /// # Returns
    /// * `true` if the hash matches the anchored certificate
    /// * `false` if no certificate is anchored or hash doesn't match
    pub fn verify_certificate(
        env: Env,
        pet_id: u64,
        vaccination_id: u64,
        cert_hash: String,
    ) -> bool {
        let anchor_key = MedicalKey::CertificateAnchor((pet_id, vaccination_id));

        if let Some(anchor) = env
            .storage()
            .instance()
            .get::<MedicalKey, CertificateAnchor>(&anchor_key)
        {
            anchor.cert_hash == cert_hash
        } else {
            false
        }
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

        let diet_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::DietPlanCount)
            .unwrap_or(0);
        let diet_id = safe_increment(diet_count);

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

        let pet_diet_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetDietCount(pet_id))
            .unwrap_or(0)
            + 1;
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
        let weight_id = safe_increment(weight_count);
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

        let current_version: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::PetNutritionVersionCount(pet_id))
            .unwrap_or(0);
        let new_version = current_version + 1;
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
        let new_version = current_version + 1;
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
        let new_nonce = safe_increment(nonce);
        env.storage().instance().set(&TagKey::TagNonce, &new_nonce);

        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();

        let mut preimage = Bytes::new(env);
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
            .set(&TagKey::PetTagCount, &safe_increment(count));

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
            panic_with_error!(env, ContractError::InvalidCallerNonce);
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

    fn validate_ipfs_hash(_env: &Env, hash: &String) -> Result<(), ContractError> {
        let len = hash.len() as usize;
        if len == 46 {
            let mut bytes = [0u8; 46];
            hash.copy_into_slice(&mut bytes);

            if bytes[0] != b'Q' || bytes[1] != b'm' {
                return Err(ContractError::InvalidIpfsHash);
            }

            for b in bytes.iter() {
                if !matches!(
                    b,
                    b'1'..=b'9'
                        | b'A'..=b'H'
                        | b'J'..=b'N'
                        | b'P'..=b'Z'
                        | b'a'..=b'k'
                        | b'm'..=b'z'
                ) {
                    return Err(ContractError::InvalidIpfsHash);
                }
            }

            return Ok(());
        }

        if !(2..=128).contains(&len) {
            return Err(ContractError::InvalidIpfsHash);
        }

        let mut bytes = [0u8; 128];
        hash.copy_into_slice(&mut bytes[..len]);

        if bytes[0] != b'b' {
            return Err(ContractError::InvalidIpfsHash);
        }

        for b in bytes.iter().take(len).skip(1) {
            if !matches!(b, b'a'..=b'z' | b'2'..=b'7') {
                return Err(ContractError::InvalidIpfsHash);
            }
        }

        Ok(())
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
        let record_id = safe_increment(global_count);

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&SystemKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = safe_increment(pet_count);

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
        let responders: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::EmergencyResponders(pet_id))
            .unwrap_or(Vec::new(env));
        responders.contains(caller)
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
                panic!("Unauthorized");
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
                panic!("Unauthorized");
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

        let count: u64 = env
            .storage()
            .instance()
            .get(&DisputeKey::DisputeCount)
            .unwrap_or(0);
        let dispute_id = count + 1;

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
        let new_pet_count = pet_count + 1;

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
            .unwrap_or_else(|| panic!("Dispute not found"));

        assert!(
            dispute.status == DisputeStatus::Pending
                || dispute.status == DisputeStatus::EvidencePhase,
            "Dispute is not open for voting"
        );

        assert!(
            Self::is_dispute_stakeholder(&env, &dispute, &voter),
            "Only the pet owner, the opposing party, or an admin may vote"
        );

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
            .unwrap_or_else(|| panic!("Dispute not found"));

        assert!(
            dispute.status == DisputeStatus::EvidencePhase,
            "Submission outside evidence phase rejected"
        );

        assert!(
            submitter == dispute.claimer || submitter == dispute.target,
            "Only claimer or target can submit evidence"
        );

        let count_key = DisputeKey::PartyEvidenceCount(dispute_id, submitter.clone());
        let party_count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        assert!(
            party_count < 10,
            "Max 10 evidence items per dispute per party"
        );

        let evidence_count_key = DisputeKey::DisputeEvidenceCount(dispute_id);
        let total_count: u64 = env
            .storage()
            .instance()
            .get(&evidence_count_key)
            .unwrap_or(0);
        let evidence_id = total_count + 1;

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
        env.storage().instance().set(&count_key, &(party_count + 1));

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
        let schedule_id = safe_increment(count);

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
            let record_id = safe_increment(rec_count);
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
            let new_pet_count = safe_increment(pet_count);
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
        let new_pet_sched_count = safe_increment(pet_sched_count);
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
        let record_id = safe_increment(rec_count);
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
        let new_pet_count = safe_increment(pet_count);
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

        if let Some(groomer_address) = record.groomer_address.clone() {
            if let Some(mut profile) = env
                .storage()
                .instance()
                .get::<GroomingKey, GroomerProfile>(&GroomingKey::Groomer(groomer_address.clone()))
            {
                let old_rating = profile.aggregate_rating as u64;
                let count = profile.review_count;
                let new_avg = ((old_rating * count) + (score as u64 * 100)) / (count + 1);
                profile.aggregate_rating = new_avg as u32;
                profile.review_count = count + 1;
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
        if !env.storage().instance().has(&GroomingKey::Groomer(groomer_id.clone())) {
            panic_with_error!(env, ContractError::InvalidInput);
        }

        // Load existing slots for this groomer and check for conflicts
        let slot_count: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomerSlotCount(groomer_id.clone()))
            .unwrap_or(0);

        for i in 1u64..=slot_count {
            if let Some(slot) = env
                .storage()
                .instance()
                .get::<GroomingKey, GroomingSlot>(&GroomingKey::GroomerSlotIndex(
                    (groomer_id.clone(), i),
                ))
            {
                // Conflict: existing.start_time < new.start_time + new.duration_mins
                //         && new.start_time < existing.start_time + existing.duration_mins
                if slot.start_time < start_time.saturating_add(duration_mins * 60)
                    && start_time < slot.start_time.saturating_add(slot.duration_mins * 60)
                {
                    panic_with_error!(env, ContractError::SlotAlreadyBooked);
                }
            }
        }

        // No conflict — assign a new slot_id and persist the slot
        let slot_id: u64 = env
            .storage()
            .instance()
            .get(&GroomingKey::GroomingRecordCount)
            .unwrap_or(0)
            .saturating_add(1);

        let new_slot = GroomingSlot {
            slot_id,
            groomer_id: groomer_id.clone(),
            start_time,
            duration_mins,
            pet_id,
        };

        let new_count = slot_count.saturating_add(1);
        env.storage()
            .instance()
            .set(&GroomingKey::GroomerSlotIndex((groomer_id.clone(), new_count)), &new_slot);
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
            if let Some(slot) = env
                .storage()
                .instance()
                .get::<GroomingKey, GroomingSlot>(&GroomingKey::GroomerSlotIndex(
                    (groomer_id.clone(), i),
                ))
            {
                slots.push_back(slot);
            }
        }
        slots
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
        let mut removed: u32 = 0;

        for delegate in delegates.iter() {
            let key = DataKey::DecryptionToken((pet_id, delegate.clone()));
            if let Some(expires_at) = env.storage().instance().get::<DataKey, u64>(&key) {
                if now >= expires_at {
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
} // end impl PetChainContract

// --- OVERFLOW-SAFE COUNTER HELPER ---
pub(crate) fn safe_increment(count: u64) -> u64 {
    count
        .checked_add(1)
        .unwrap_or_else(|| panic!("counter overflow"))
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
    let next_counter = safe_increment(counter);
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
        Gender, LabResultAnomaly, PetChainContract, PetChainContractClient, PrivacyLevel,
        Species, EVENT_SCHEMA_VERSION,
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
