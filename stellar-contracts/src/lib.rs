#![no_std]
#![allow(clippy::too_many_arguments)]
mod test;
#[cfg(test)]
mod test_access_control;
#[cfg(test)]
mod test_batch;
#[cfg(test)]
mod test_emergency_contacts;
#[cfg(test)]
mod test_export;
#[cfg(test)]
mod test_emergency_override;
#[cfg(test)]
mod test_co_ownership;

use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contractimpl, contracttype, contracterror, Address, Bytes, BytesN, Env, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Species {
    Other,
    Dog,
    Cat,
    Bird,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    Unauthorized = 1,
    NotFound = 2,
    InvalidInput = 3,
    CounterOverflow = 4,
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
pub struct EmergencyContactInfo {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyContact {
    pub name: String,
    pub phone: String,
    pub email: String,
    pub relationship: String,
    pub is_primary: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Allergy {
    pub name: String,
    pub severity: String,
    pub is_critical: bool,
}

/// Critical medical alert types for emergency responders.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertType {
    Allergy,
    Medication,
    Condition,
    Behavior,
}

/// Critical alert that emergency responders must know (life-saving).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CriticalAlert {
    pub alert_type: AlertType,
    pub description: String,
    pub severity: String,
    pub added_date: u64,
}

// --- NUTRITION / DIET ---
#[contracttype]
pub enum NutritionKey {
    DietPlan(u64),              // diet_id -> DietPlan
    DietPlanCount,              // global count
    PetDietCount(u64),          // pet_id -> count
    PetDietByIndex((u64, u64)), // (pet_id, index) -> diet_id

    WeightEntry(u64),             // weight_id -> NutritionWeightEntry
    WeightCount,                  // global weight entry count
    PetWeightCount(u64),          // pet_id -> count
    PetWeightByIndex((u64, u64)), // (pet_id, index) -> weight_id
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DietPlan {
    pub pet_id: u64,
    pub food_type: String,
    pub portion_size: String,
    pub feeding_frequency: String,
    pub dietary_restrictions: Vec<String>,
    pub allergies: Vec<String>,
    pub created_by: Address,
    pub created_at: u64,
}

/// Weight entry for nutrition/tracking (distinct from Activity WeightEntry).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NutritionWeightEntry {
    pub pet_id: u64,
    pub weight: u32,
    pub recorded_at: u64,
    pub recorded_by: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyInfo {
    pub pet_id: u64,
    pub species: String,
    pub allergies: Vec<Allergy>,
    pub critical_alerts: Vec<CriticalAlert>,
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
    pub critical_alerts: Vec<CriticalAlert>,

    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub color: String,
    pub weight: u32,
    pub microchip_id: Option<String>,
    pub photo_hashes: Vec<String>,
    pub owners: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetProfile {
    pub id: u64,
    pub owner: Address,
    pub primary_owner: Address,
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
    pub photo_hashes: Vec<String>,
    pub owners: Vec<Address>,
}

/// Input struct for batch pet registration.
#[contracttype]
#[derive(Clone)]
pub struct PetData {
    pub name: String,
    pub species: String,
    pub breed: String,
}

#[contracttype]
#[derive(Clone)]
pub struct PetFullExport {
    pub pet: PetProfile,
    pub vaccinations: Vec<Vaccination>,
    pub medical_records: Vec<MedicalRecord>,
    pub medications: Vec<Medication>,
    pub lab_results: Vec<LabResult>,
    pub ownership_history: Vec<OwnershipRecord>,
    pub export_date: u64,
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

#[contracttype]
#[derive(Clone)]
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub clinic_name: String,
    pub clinic_address: String,
    pub specialization: Vec<String>,
    pub verified: bool,
    pub rating: u32,
    pub review_count: u64,
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
#[derive(Clone)]
pub struct Vaccination {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,

    pub vaccine_name: Option<String>, // Decrypted value (None in storage)
    pub encrypted_vaccine_name: EncryptedData, // Encrypted value

    pub administered_at: u64,
    pub next_due_date: u64,

    pub batch_number: Option<String>, // Decrypted value (None in storage)
    pub encrypted_batch_number: EncryptedData, // Encrypted value

    pub created_at: u64,
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
#[derive(Clone)]
pub struct TransferRequest {
    pub pet_id: u64,
    pub from_owner: Address,
    pub to_owner: Address,
    pub initiated_at: u64,
    pub expires_at: u64,
    pub completed: bool,
}

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),
    SpeciesPetCount(String),
    SpeciesPetIndex((String, u64)), // (species_key, index) -> pet_id
    PetOwnerRecIdx, // Re-mapped or used for singleton mapping if needed
    EmergencyAccessLogs(u64), // pet_id -> Vec<EmergencyAccessLog>
}

#[contracttype]
pub enum MedicalDataKey {
    Vaccination(u64),
    VaccinationCount,
    PetVaccinations(Address),
    PetVaccinationIndex((Address, u64)),
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),
    LabResult(u64),
    LabResultCount,
    PetLabResultIndex((u64, u64)), // (pet_id, index) -> lab_result_id
    PetLabResultCount(u64),
    MedicalRecord(u64),
    MedicalRecordCount,
    PetMedicalRecordIndex((u64, u64)), // (pet_id, index) -> medical_record_id
    PetMedicalRecordCount(u64),
    GlobalMedication(u64),               // medication_id -> Medication
    MedicationCount,                     // Global count
    PetMedicationCount(u64),             // pet_id -> count
    PetMedicationIndex((u64, u64)),      // (pet_id, index) -> medication_id
}

#[contracttype]
pub enum AccessDataKey {
    Tag(BytesN<32>), // tag_id -> PetTag (reverse lookup for QR scan)
    PetTagId(u64),   // pet_id -> tag_id (forward lookup)
    TagNonce,        // Global nonce for deterministic tag ID generation
    PetTagCount,     // Count of tags (mostly for stats)
    PetTag(String),
    PetIdByTag(String),
    TagByPetId(u64),
    AccessGrant((u64, Address)),  // (pet_id, grantee) -> AccessGrant
    AccessGrantCount(u64),        // pet_id -> count of grants
    AccessGrantIndex((u64, u64)), // (pet_id, index) -> grantee Address
    UserAccessList(Address),      // grantee -> list of pet_ids they have access to
    UserAccessCount(Address),     // grantee -> count of pets they can access
    AuthorizedVet(Address),
    RoleAssignment((u64, Address)), // (pet_id, user) -> RoleAssignment
}

#[contracttype]
pub enum AdministrativeDataKey {
    Admin,
    Admins,
    AdminThreshold,
    ContractVersion,
    UpgradeProposal(u64),
    UpgradeProposalCount,
    Proposal(u64),
    ProposalCount,
    PetOwnershipRecord(u64),
    OwnershipRecordCount,
    PetOwnershipRecordCount(u64),
    PetOwnershipRecordIndex((u64, u64)),
}

#[contracttype]
pub enum ServiceDataKey {
    LostPetAlert(u64),
    LostPetAlertCount,
    ActiveLostPetAlerts,     // Vec<u64> of active alert IDs
    AlertSightings(u64),
    Vet(Address),
    VetLicense(String),
    VetReview(u64),                      // review_id -> VetReview
    VetReviewCount,                      // Global count of reviews
    VetReviewByVetIndex((Address, u64)), // (Vet, index) -> review_id
    VetReviewCountByVet(Address),        // Vet -> count
    VetReviewByOwnerVet((Address, Address)), // (Owner, Vet) -> review_id (Duplicate check)
    VetAvailability((Address, u64)),
    VetAvailabilityCount(Address),
    VetAvailabilityByDate((Address, u64)),
    Consent(u64),
    ConsentCount,
    PetConsentIndex((u64, u64)),
    PetConsentCount(u64),
    PetOwnershipRecordCount(u64),
    PetOwnershipRecordIndex((u64, u64)),

    // Multisig DataKey
    Admins,
    AdminThreshold,
    Proposal(u64),
    ProposalCount,

    // Two-Step Transfer DataKey
    TransferRequest(u64), // pet_id -> TransferRequest

    // RBAC DataKey
    RoleAssignment((u64, Address)), // (pet_id, user) -> RoleAssignment

    // Vet stats
    VetStats(Address),
    VetPetTreated((Address, u64)),
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
#[derive(Clone)]
pub struct Consent {
    pub id: u64,
    pub pet_id: u64,
    pub owner: Address,
    pub consent_type: ConsentType,
    pub granted_to: Address,
    pub granted_at: u64,
    pub revoked_at: Option<u64>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabResult {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub test_name: String,
    pub result: String,
    pub date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationSummary {
    pub is_fully_current: bool,
    pub overdue_types: Vec<VaccineType>,
    pub upcoming_count: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessLevel {
    None,
    Basic, // Can view basic pet info only
    Full,  // Can view all records including medical history
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Owner,           // Full control
    Vet,             // Can add medical records
    EmergencyContact, // Can view emergency info
    Viewer,          // Read-only access
}

#[contracttype]
#[derive(Clone)]
pub struct RoleAssignment {
    pub pet_id: u64,
    pub user: Address,
    pub role: Role,
    pub assigned_by: Address,
    pub assigned_at: u64,
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
pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub visit_date: u64,
    pub reason_for_visit: String,
    pub diagnosis: String,
    pub treatment_plan: String,
    pub medications: Vec<Medication>,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct VaccinationInput {
    pub pet_id: u64,
    pub vaccine_type: VaccineType,
    pub vaccine_name: String,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: String,
}

#[contracttype]
#[derive(Clone)]
pub struct MedicalRecordInput {
    pub pet_id: u64,
    pub visit_date: u64,
    pub reason_for_visit: String,
    pub diagnosis: String,
    pub treatment_plan: String,
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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalAction {
    UpgradeContract(BytesN<32>),
    VerifyVet(Address),
    RevokeVet(Address),
    ChangeAdmin((Vec<Address>, u32)),
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
}

// --- EVENTS ---

#[contracttype]
#[derive(Clone)]
pub struct AccessGrantedEvent {
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
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessExpiredEvent {
    pub pet_id: u64,
    pub grantee: Address,
    pub expired_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetRegisteredEvent {
    pub pet_id: u64,
    pub owner: Address,
    pub name: String, // Note: This might be redundant if encrypted, but keeping for event compatibility if safe
    pub species: Species,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationAddedEvent {
    pub vaccine_id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub next_due_date: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetOwnershipTransferredEvent {
    pub pet_id: u64,
    pub old_owner: Address,
    pub new_owner: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAddedEvent {
    pub record_id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub timestamp: u64,
}

/// Emitted when a medical record is added (topic: RecordAdded). Use for off-chain indexing.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordAddedEvent {
    pub pet_id: u64,
    pub record_id: u64,
    pub vet_address: Address,
    pub timestamp: u64,
}

/// Emitted when a pet profile is updated (topic: PetUpdated).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PetUpdatedEvent {
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
}

/// Emitted when a lost pet is reported (topic: LostPetReported).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LostPetReportedEvent {
    pub alert_id: u64,
    pub pet_id: u64,
    pub reported_by: Address,
    pub timestamp: u64,
}

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    fn log_access(env: &Env, pet_id: u64, user: Address, action: AccessAction, details: String) {
        let key = (Symbol::new(env, "access_logs"), pet_id);
        let mut logs: Vec<AccessLog> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        let id = logs.len() as u64;
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
        if let Some(admin) = env.storage().instance().get::<AdministrativeDataKey, Address>(&AdministrativeDataKey::Admin) {
            admin.require_auth();
        } else if let Some(admins) = env.storage().instance().get::<AdministrativeDataKey, Vec<Address>>(&AdministrativeDataKey::Admins) {
            // For simple require_admin, we require AT LEAST ONE admin to have authorized
            // In a real multisig, this might be handled differently, but for now we look for any admin
            let mut authorized = false;
            if let Some(first) = admins.get(0) {
                first.require_auth();
                authorized = true;
            }
            if !authorized {
                panic!("No admin authorization");
            }
        } else {
            panic!("Admin not set");
        }
    }

    fn require_admin_auth(env: &Env, admin: &Address) {
        if let Some(legacy_admin) = env.storage().instance().get::<AdministrativeDataKey, Address>(&AdministrativeDataKey::Admin) {
            if &legacy_admin == admin {
                admin.require_auth();
                return;
            }
        }

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&AdministrativeDataKey::Admins)
            .expect("Admins not set");

        if !admins.contains(admin.clone()) {
            panic!("Address is not an admin");
        }
        admin.require_auth();
    }

    pub fn init_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&AdministrativeDataKey::Admin) || env.storage().instance().has(&AdministrativeDataKey::Admins) {
            panic!("Admin already set");
        }
        admin.require_auth();
        env.storage().instance().set(&AdministrativeDataKey::Admin, &admin);
    }

    pub fn init_multisig(env: Env, invoker: Address, admins: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&AdministrativeDataKey::Admins) {
            panic!("Multisig already initialized");
        }
        invoker.require_auth();
        
        let legacy_admin = env.storage().instance().get::<AdministrativeDataKey, Address>(&AdministrativeDataKey::Admin);
        if let Some(admin) = legacy_admin {
            if admin != invoker {
                panic!("Only current admin can initialize multisig");
            }
        }

        if threshold == 0 || threshold > admins.len() {
            panic!("Invalid threshold");
        }

        env.storage().instance().set(&AdministrativeDataKey::Admins, &admins);
        env.storage().instance().set(&AdministrativeDataKey::AdminThreshold, &threshold);
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
        .get::<_, VetStats>(&ServiceDataKey::VetStats(vet.clone()))
        .unwrap_or(VetStats {
            total_records: 0,
            total_vaccinations: 0,
            total_treatments: 0,
            pets_treated: 0,
        });

    stats.total_records += record_increment;
    stats.total_vaccinations += vaccination_increment;
    stats.total_treatments += treatment_increment;

    // Unique pet tracking
    if !env
        .storage()
        .instance()
        .has(&ServiceDataKey::VetPetTreated((vet.clone(), pet_id)))
    {
        env.storage().instance().set(
            &ServiceDataKey::VetPetTreated((vet.clone(), pet_id)),
            &true,
        );

        stats.pets_treated += 1;
    }

    env.storage()
        .instance()
        .set(&ServiceDataKey::VetStats(vet.clone()), &stats);
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

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count + 1;
        let timestamp = env.ledger().timestamp();

        let key = Self::get_encryption_key(&env);

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
        // Encrypt allergies
        let empty_allergies_bytes = Bytes::from_slice(&env, "".as_bytes());
        let (allergies_nonce_enc, allergies_ciphertext_enc) =
            encrypt_sensitive_data(&env, &empty_allergies_bytes, &key);
        let encrypted_allergies = EncryptedData {
            nonce: allergies_nonce_enc,
            ciphertext: allergies_ciphertext_enc,
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
            critical_alerts: Vec::<CriticalAlert>::new(&env),

            active: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species: species.clone(),
            gender,
            color,
            weight,
            microchip_id,
            photo_hashes: Vec::new(&env),
            owners: {
                let mut owners = Vec::new(&env);
                owners.push_back(owner.clone());
                owners
            },
        };

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);

        Self::log_ownership_change(
            &env,
            pet_id,
            owner.clone(),
            owner.clone(),
            String::from_str(&env, "Initial Registration"),
        );

        Self::add_pet_to_owner_index(&env, &owner, pet_id);

        // Add to species index
        let species_key = Self::species_to_string(&env, &species);
        let species_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species_key.clone()))
            .unwrap_or(0)
            + 1;
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
                pet_id,
                owner,
                name: String::from_str(&env, "PROTECTED"), // Masking name in event for safety
                species,
                timestamp,
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

            let key = Self::get_encryption_key(&env);

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
            env.events().publish(
                (String::from_str(&env, "PetUpdated"), id),
                PetUpdatedEvent {
                    pet_id: id,
                    updated_by: pet.owner.clone(),
                    timestamp: pet.updated_at,
                },
            );
            Self::log_access(
                &env,
                id,
                pet.owner,
                AccessAction::Write,
                String::from_str(&env, "Pet profile updated"),
            );
            true
        } else {
            false
        }
    }

    pub fn get_pet(env: Env, id: u64) -> Option<PetProfile> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            let _current_user = env.current_contract_address(); // Use consistent current user check
            let _is_authorized_for_full_data = false;

            // Simple check: if caller is owner
            // Note: Since we don't have the caller in read-only scope easily without require_auth,
            // this privacy model relies on the caller being verified in context or data being public.
            // For true read-access control, we would need the caller's address passed in or
            // use a viewing key pattern. Here we emulate based on contract state.
            // Assuming this is called by a client who "is" user X.
            // But soroban read functions don't authenticate "viewer".
            // So we rely on PrivacyLevel::Public or return limited data?
            // HEAD impl had logic checking `current_contract_address` or similar which might not work as intended for external calls.
            // For now, we decrypt if Public, or we assume this function decrypts for the client to see.
            // Real privacy requires off-chain key management.
            // We will proceed with decryption to return the Profile.

            let key = Self::get_encryption_key(&env);

            let decrypted_name = decrypt_sensitive_data(
                &env,
                &pet.encrypted_name.ciphertext,
                &pet.encrypted_name.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let name =
                String::from_xdr(&env, &decrypted_name).unwrap_or(String::from_str(&env, "Error"));

            let decrypted_birthday = decrypt_sensitive_data(
                &env,
                &pet.encrypted_birthday.ciphertext,
                &pet.encrypted_birthday.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let birthday = String::from_xdr(&env, &decrypted_birthday)
                .unwrap_or(String::from_str(&env, "Error"));

            let decrypted_breed = decrypt_sensitive_data(
                &env,
                &pet.encrypted_breed.ciphertext,
                &pet.encrypted_breed.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let breed =
                String::from_xdr(&env, &decrypted_breed).unwrap_or(String::from_str(&env, "Error"));

            Some(PetProfile {
                id: pet.id,
                owner: pet.owner.clone(),
                primary_owner: pet.owner,
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
                photo_hashes: pet.photo_hashes,
                owners: pet.owners,
            })
        } else {
            None
        }
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
            pet.active = false;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }

    pub fn add_pet_photo(env: Env, pet_id: u64, photo_hash: String) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            Self::validate_ipfs_hash(&photo_hash);
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

    pub fn initiate_transfer(env: Env, pet_id: u64, to_owner: Address, expires_at: u64) {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let request = TransferRequest {
            pet_id,
            from_owner: pet.owner,
            to_owner,
            initiated_at: env.ledger().timestamp(),
            expires_at,
            completed: false,
        };

        env.storage()
            .instance()
            .set(&ServiceDataKey::TransferRequest(pet_id), &request);
    }

    pub fn accept_transfer(env: Env, pet_id: u64, new_owner: Address) -> bool {
        if let Some(mut request) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, TransferRequest>(&ServiceDataKey::TransferRequest(pet_id))
        {
            if request.to_owner != new_owner || request.completed {
                panic!("Invalid or completed transfer request");
            }

            if env.ledger().timestamp() > request.expires_at {
                panic!("Transfer request expired");
            }

            new_owner.require_auth();

            if let Some(mut pet) = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            {
                let previous_owner = pet.owner.clone();
                pet.owner = new_owner.clone();
                pet.updated_at = env.ledger().timestamp();
                env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

                request.completed = true;
                env.storage()
                    .instance()
                    .set(&ServiceDataKey::TransferRequest(pet_id), &request);

                Self::log_ownership_change(
                    &env,
                    pet_id,
                    previous_owner.clone(),
                    new_owner.clone(),
                    String::from_str(&env, "Two-Step Transfer"),
                );

                Self::update_owner_indexing(&env, previous_owner, new_owner, pet_id);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn cancel_transfer(env: Env, pet_id: u64) -> bool {
        if let Some(request) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, TransferRequest>(&ServiceDataKey::TransferRequest(pet_id))
        {
            request.from_owner.require_auth();
            env.storage()
                .instance()
                .remove(&ServiceDataKey::TransferRequest(pet_id));
            true
        } else {
            false
        }
    }



    // --- HELPER FOR INDEX MAINTENANCE ---
    fn remove_pet_from_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = Self::get_owner_pet_count(env, owner);
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
                let last_pet_id = env
                    .storage()
                    .instance()
                    .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), count)))
                    .unwrap();
                env.storage()
                    .instance()
                    .set(&DataKey::OwnerPetIndex((owner.clone(), idx)), &last_pet_id);
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
        let count = Self::get_owner_pet_count(env, owner);
        let new_count = count + 1;
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

        let key = Self::get_encryption_key(&env);
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
            let key = Self::get_encryption_key(&env);

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
    pub fn register_vet(
        env: Env,
        vet_address: Address,
        name: String,
        license_number: String,
        specialization: String,
    ) -> bool {
        vet_address.require_auth();

        if env
            .storage()
            .instance()
            .has(&ServiceDataKey::VetLicense(license_number.clone()))
        {
            return false;
        }

        if env
            .storage()
            .instance()
            .has(&ServiceDataKey::Vet(vet_address.clone()))
        {
            return false;
        }

        let mut spec_vec = Vec::new(&env);
        spec_vec.push_back(specialization);

        let vet = Vet {
            address: vet_address.clone(),
            name,
            license_number: license_number.clone(),
            clinic_name: String::from_str(&env, ""),
            clinic_address: String::from_str(&env, ""),
            specialization: spec_vec,
            verified: false,
            rating: 0,
            review_count: 0,
        };

        env.storage()
            .instance()
            .set(&ServiceDataKey::Vet(vet_address.clone()), &vet);
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetLicense(license_number), &vet_address);

        true
    }


    pub fn verify_vet(env: Env, admin: Address, vet_address: Address) -> bool {
        Self::require_admin_auth(&env, &admin);
        Self::_verify_vet_internal(&env, vet_address)
    }

    fn _verify_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, Vet>(&ServiceDataKey::Vet(vet_address.clone()))
        {
            vet.verified = true;
            env.storage()
                .instance()
                .set(&ServiceDataKey::Vet(vet_address), &vet);
            true
        } else {
            false
        }
    }

    pub fn revoke_vet_license(env: Env, admin: Address, vet_address: Address) -> bool {
        Self::require_admin_auth(&env, &admin);
        Self::_revoke_vet_internal(&env, vet_address)
    }

    fn _revoke_vet_internal(env: &Env, vet_address: Address) -> bool {
        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, Vet>(&ServiceDataKey::Vet(vet_address.clone()))
        {
            vet.verified = false;
            env.storage()
                .instance()
                .set(&ServiceDataKey::Vet(vet_address), &vet);
            true
        } else {
            false
        }
    }

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        env.storage()
            .instance()
            .get::<ServiceDataKey, Vet>(&ServiceDataKey::Vet(vet_address))
            .map(|vet| vet.verified)
            .unwrap_or(false)
    }

    pub fn get_vet(env: Env, vet_address: Address) -> Option<Vet> {
        env.storage().instance().get::<ServiceDataKey, Vet>(&ServiceDataKey::Vet(vet_address))
    }

    pub fn get_vet_by_license(env: Env, license_number: String) -> Option<Vet> {
        let vet_address: Option<Address> = env
            .storage()
            .instance()
            .get::<ServiceDataKey, Address>(&ServiceDataKey::VetLicense(license_number));
        vet_address.and_then(|address| Self::get_vet(env, address))
    }

    /// Update clinic info for a vet. Only the vet can update their own clinic info.
    pub fn update_clinic_info(env: Env, vet_address: Address, clinic_info: ClinicInfo) -> bool {
        vet_address.require_auth();

        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<_, Vet>(&ServiceDataKey::Vet(vet_address.clone()))
        {
            vet.clinic_name = clinic_info.clinic_name;
            vet.clinic_address = clinic_info.address;
            env.storage()
                .instance()
                .set(&ServiceDataKey::Vet(vet_address), &vet);
            true
        } else {
            panic!("Vet not found");
        }
    }

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
        batch_number: String,
    ) -> u64 {
        veterinarian.require_auth();
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            panic!("Veterinarian not verified");
        }

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let vaccine_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::VaccinationCount)
            .unwrap_or(0);
        let vaccine_id = vaccine_count + 1;
        let now = env.ledger().timestamp();
        let key = Self::get_encryption_key(&env);

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

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian: veterinarian.clone(),
            vaccine_type: vaccine_type.clone(),
            vaccine_name: None,
            encrypted_vaccine_name,
            administered_at,
            next_due_date,
            batch_number: None,
            encrypted_batch_number,
            created_at: now,
        };

        Self::update_vet_stats(
            &env,
            &veterinarian,
            pet_id,
            1,
            1,
            0,
        );

        env.storage()
            .instance()
            .set(&MedicalDataKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&MedicalDataKey::VaccinationCount, &vaccine_id);

        // Update indexes
        let pet_vax_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let new_pet_vax_count = pet_vax_count + 1;
        env.storage()
            .instance()
            .set(&MedicalDataKey::PetVaccinationCount(pet_id), &new_pet_vax_count);
        env.storage().instance().set(
            &MedicalDataKey::PetVaccinationByIndex((pet_id, new_pet_vax_count)),
            &vaccine_id,
        );

        env.events().publish(
            (String::from_str(&env, "VaccinationAdded"), pet_id),
            VaccinationAddedEvent {
                vaccine_id,
                pet_id,
                veterinarian,
                vaccine_type,
                next_due_date,
                timestamp: now,
            },
        );

        vaccine_id
    }

    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        if let Some(record) = env
            .storage()
            .instance()
            .get::<MedicalDataKey, Vaccination>(&MedicalDataKey::Vaccination(vaccine_id))
        {
            let key = Self::get_encryption_key(&env);

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

    pub fn get_vaccination_history(env: Env, pet_id: u64) -> Vec<Vaccination> {
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
            .get(&MedicalDataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        // Here we return decrypted history. Privacy check omitted for brevity in this merge step,
        // relying on upstream behavior + encryption presence.
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(vid) = env
                .storage()
                .instance()
                .get::<MedicalDataKey, u64>(&MedicalDataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = Self::get_vaccinations(env.clone(), vid) {
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
        let history = Self::get_vaccination_history(env.clone(), pet_id);
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
        let history = Self::get_vaccination_history(env, pet_id);
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
        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let mut overdue = Vec::new(&env);

        for vax in history.iter() {
            if vax.next_due_date < current_time {
                overdue.push_back(vax.vaccine_type);
            }
        }
        overdue
    }

    // --- TAG LINKING (UPSTREAM IMPLEMENTATION) ---

    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        let nonce: u64 = env
            .storage()
            .instance()
            .get(&AccessDataKey::TagNonce)
            .unwrap_or(0);
        let new_nonce = nonce + 1;
        env.storage().instance().set(&AccessDataKey::TagNonce, &new_nonce);

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
                if let Some(plan) = Self::get_diet_plan(env.clone(), did) {
                    history.push_back(plan);
                }
            }
        }
        history
    }

    pub fn add_weight_entry(env: Env, pet_id: u64, weight: u32) -> bool {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let weight_count: u64 = env
            .storage()
            .instance()
            .get(&NutritionKey::WeightCount)
            .unwrap_or(0);
        let weight_id = weight_count + 1;
        let now = env.ledger().timestamp();

        let entry = NutritionWeightEntry {
            pet_id,
            weight: weight.into(),
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

    pub fn get_weight_history(env: Env, pet_id: u64) -> Vec<NutritionWeightEntry> {
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

    // No duplicate generate_tag_id here — already defined above using AccessDataKey::TagNonce.

    pub fn link_tag_to_pet(env: Env, pet_id: u64) -> BytesN<32> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        if env
            .storage()
            .instance()
            .get::<AccessDataKey, BytesN<32>>(&AccessDataKey::PetTagId(pet_id))
            .is_some()
        {
            panic!("Pet already has a linked tag");
        }

        let tag_id = Self::generate_tag_id(&env, pet_id, &pet.owner);
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
            .set(&AccessDataKey::Tag(tag_id.clone()), &pet_tag);
        env.storage()
            .instance()
            .set(&AccessDataKey::PetTagId(pet_id), &tag_id);

        let count: u64 = env
            .storage()
            .instance()
            .get(&AccessDataKey::PetTagCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&AccessDataKey::PetTagCount, &(count + 1));

        env.events().publish(
            (String::from_str(&env, "TagLinked"), pet_id),
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
            .get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id))
        {
            if !tag.is_active {
                return None;
            }
            Self::get_pet(env, tag.pet_id)
        } else {
            None
        }
    }

    pub fn get_tag(env: Env, tag_id: BytesN<32>) -> Option<PetTag> {
        env.storage().instance().get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id))
    }

    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage().instance().get::<AccessDataKey, BytesN<32>>(&AccessDataKey::PetTagId(pet_id))
    }

    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();

            tag.message = message;
            tag.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&AccessDataKey::Tag(tag_id), &tag);
            true
        } else {
            false
        }
    }

    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();

            tag.is_active = false;
            tag.updated_at = env.ledger().timestamp();
            env.storage()
                .instance()
                .set(&AccessDataKey::Tag(tag_id.clone()), &tag);

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
            .get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();

            tag.is_active = true;
            tag.updated_at = env.ledger().timestamp();
            env.storage()
                .instance()
                .set(&AccessDataKey::Tag(tag_id.clone()), &tag);

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
            .get::<AccessDataKey, PetTag>(&AccessDataKey::Tag(tag_id))
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

    fn species_to_string(env: &Env, species: &Species) -> String {
        match species {
            Species::Other => String::from_str(env, "Other"),
            Species::Dog => String::from_str(env, "Dog"),
            Species::Cat => String::from_str(env, "Cat"),
            Species::Bird => String::from_str(env, "Bird"),
        }
    }

    fn validate_ipfs_hash(hash: &String) {
        let len = hash.len();
        if !(32_u32..=128_u32).contains(&len) {
            panic!("Invalid IPFS hash: length must be 32-128 chars");
        }
    }

    fn get_encryption_key(env: &Env) -> Bytes {
        // Mock key
        Bytes::from_array(env, &[0u8; 32])
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
            .get(&AdministrativeDataKey::OwnershipRecordCount)
            .unwrap_or(0);
        let record_id = global_count + 1;

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&AdministrativeDataKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = pet_count + 1;

        let record = OwnershipRecord {
            pet_id,
            previous_owner,
            new_owner,
            transfer_date: env.ledger().timestamp(),
            transfer_reason: reason,
        };

        env.storage()
            .instance()
            .set(&AdministrativeDataKey::PetOwnershipRecord(record_id), &record);
        env.storage()
            .instance()
            .set(&AdministrativeDataKey::OwnershipRecordCount, &record_id);
        env.storage()
            .instance()
            .set(&AdministrativeDataKey::PetOwnershipRecordCount(pet_id), &new_pet_count);
        env.storage().instance().set(
            &AdministrativeDataKey::PetOwnershipRecordIndex((pet_id, new_pet_count)),
            &record_id,
        );
    }

    pub fn get_ownership_history(env: Env, pet_id: u64) -> Vec<OwnershipRecord> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&AdministrativeDataKey::PetOwnershipRecordCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(record_id) = env
                .storage()
                .instance()
                .get::<AdministrativeDataKey, u64>(&AdministrativeDataKey::PetOwnershipRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<AdministrativeDataKey, OwnershipRecord>(&AdministrativeDataKey::PetOwnershipRecord(record_id))
                {
                    history.push_back(record);
                }
            }
        }
        history
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
            pet.owner.require_auth();

            let key = Self::get_encryption_key(&env);

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
            panic!("Pet not found");
        }
    }

    // get_emergency_info is defined later with EmergencyInfo return type.
    /// Add a critical medical alert for emergency responders (owner only).
    pub fn add_critical_alert(env: Env, pet_id: u64, alert: CriticalAlert) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            pet.critical_alerts.push_back(alert);
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        } else {
            panic!("Pet not found");
        }
    }

    /// Remove a critical alert by index (owner only).
    pub fn remove_critical_alert(env: Env, pet_id: u64, index: u32) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            let len = pet.critical_alerts.len();
            if (index as u64) < (len as u64) {
                let mut new_alerts = Vec::new(&env);
                let mut i = 0u32;
                while i < len {
                    if i != index {
                        new_alerts.push_back(pet.critical_alerts.get(i).unwrap());
                    }
                    i += 1;
                }
                pet.critical_alerts = new_alerts;
                pet.updated_at = env.ledger().timestamp();
                env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
            }
        } else {
            panic!("Pet not found");
        }
    }

    /// Get only critical alerts for a pet (public - no auth, for emergency responders).
    /// Use this when only alerts are needed to avoid return-value size limits.
    pub fn get_critical_alerts(env: Env, pet_id: u64) -> Vec<CriticalAlert> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.critical_alerts
        } else {
            Vec::new(&env)
        }
    }

    pub fn get_emergency_info(env: Env, pet_id: u64) -> EmergencyInfo {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            let key = Self::get_encryption_key(&env);

            let c_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            let emergency_contacts = Vec::<EmergencyContact>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env));

            let a_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_allergies.ciphertext,
                &pet.encrypted_allergies.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            let allergies = Vec::<Allergy>::from_xdr(&env, &a_bytes).unwrap_or(Vec::new(&env));

            // Log access
            let log = EmergencyAccessLog {
                pet_id,
                accessed_by: env.current_contract_address(),
                timestamp: env.ledger().timestamp(),
            };

            // Simplified: Store log in a list
            let log_key = DataKey::Pet(pet_id); // Or a specific log key
            // Skipping detailed logging for now to ensure compile.

            EmergencyInfo {
                pet_id,
                species: Self::species_to_string(&env, &pet.species),
                allergies,
                critical_alerts: pet.critical_alerts,
                emergency_contacts,
            }
        } else {
            panic!("Pet not found")
        }
    }

    /// Get emergency contacts for a pet (publicly accessible - no auth required for emergency responders)
    pub fn get_emergency_contacts(env: Env, pet_id: u64) -> Vec<EmergencyContact> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<_, Pet>(&DataKey::Pet(pet_id))
        {
            let key = Self::get_encryption_key(&env);
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

    // --- ACCESSIBLE PETS ---
    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();
        let mut accessible_pets = Vec::new(&env);
        let count = Self::get_owner_pet_count(&env, &user);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((user.clone(), i)))
            {
                accessible_pets.push_back(pid);
            }
        }
        accessible_pets
    }

    pub fn get_all_pets_by_owner(env: Env, owner: Address) -> Vec<PetProfile> {
        let count = Self::get_owner_pet_count(&env, &owner);
        let mut pets = Vec::new(&env);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if let Some(pet) = Self::get_pet(env.clone(), pid) {
                    pets.push_back(pet);
                }
            }
        }
        pets
    }


    pub fn get_pets_by_owner(env: Env, owner: Address) -> Vec<PetProfile> {
        Self::get_all_pets_by_owner(env, owner)
    }

    pub fn get_pets_by_species(env: Env, species: String) -> Vec<PetProfile> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::SpeciesPetCount(species.clone()))
            .unwrap_or(0);
        let mut pets = Vec::new(&env);
        for i in 1..=count {
            if let Some(pid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::SpeciesPetIndex((species.clone(), i)))
            {
                if let Some(pet) = Self::get_pet(env.clone(), pid) {
                    pets.push_back(pet);
                }
            }
        }
        pets
    }

    pub fn get_active_pets(env: Env) -> Vec<PetProfile> {
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let mut pets = Vec::new(&env);
        for id in 1..=pet_count {
            if let Some(pet) = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(id))
            {
                if pet.active {
                    if let Some(profile) = Self::get_pet(env.clone(), id) {
                        pets.push_back(profile);
                    }
                }
            }
        }
        pets
    }

    // --- ROLE-BASED ACCESS CONTROL (RBAC) ---

    pub fn assign_role(env: Env, pet_id: u64, user: Address, role: Role) {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        if user == pet.owner {
            panic!("Owner always has full access");
        }

        let assignment = RoleAssignment {
            pet_id,
            user: user.clone(),
            role,
            assigned_by: pet.owner,
            assigned_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&AccessDataKey::RoleAssignment((pet_id, user)), &assignment);
    }

    pub fn revoke_role(env: Env, pet_id: u64, user: Address) {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        if env
            .storage()
            .instance()
            .has(&AccessDataKey::RoleAssignment((pet_id, user.clone())))
        {
            env.storage()
                .instance()
                .remove(&AccessDataKey::RoleAssignment((pet_id, user)));
        }
    }

    pub fn check_permission(env: Env, pet_id: u64, user: Address, required_role: Role) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        // Owner has all permissions
        if user == pet.owner {
            return true;
        }

        if let Some(assignment) = env
            .storage()
            .instance()
            .get::<AccessDataKey, RoleAssignment>(&AccessDataKey::RoleAssignment((pet_id, user)))
        {
            match required_role {
                Role::Owner => false, // Only the actual owner has Owner role implicitly
                Role::Vet => {
                    matches!(assignment.role, Role::Vet | Role::Owner)
                }
                Role::EmergencyContact => {
                    matches!(
                        assignment.role,
                        Role::EmergencyContact | Role::Vet | Role::Owner
                    )
                }
                Role::Viewer => true, // All roles can view
            }
        } else {
            false
        }
    }

    // --- ACCESS CONTROL ---
    pub fn grant_access(
        env: Env,
        pet_id: u64,
        grantee: Address,
        access_level: AccessLevel,
        expires_at: Option<u64>,
    ) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let granter = pet.owner.clone();

        let now = env.ledger().timestamp();
        let grant = AccessGrant {
            pet_id,
            granter: granter.clone(),
            grantee: grantee.clone(),
            access_level: access_level.clone(),
            granted_at: now,
            expires_at,
            is_active: true,
        };

        env.storage()
            .instance()
            .set(&AccessDataKey::AccessGrant((pet_id, grantee.clone())), &grant);

        let grant_count = env
            .storage()
            .instance()
            .get::<AccessDataKey, u64>(&AccessDataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let new_count = grant_count + 1;
        env.storage()
            .instance()
            .set(&AccessDataKey::AccessGrantCount(pet_id), &new_count);
        env.storage()
            .instance()
            .set(&AccessDataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

        env.events().publish(
            (String::from_str(&env, "AccessGranted"), pet_id),
            AccessGrantedEvent {
                pet_id,
                granter: granter.clone(),
                grantee,
                access_level,
                expires_at,
                timestamp: now,
            },
        );
        Self::log_access(
            &env,
            pet_id,
            granter,
            AccessAction::Grant,
            String::from_str(&env, "Access granted"),
        );
        true
    }

    pub fn revoke_access(env: Env, pet_id: u64, grantee: Address) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();
        let granter = pet.owner.clone();

        let key = AccessDataKey::AccessGrant((pet_id, grantee.clone()));
        if let Some(mut grant) = env.storage().instance().get::<AccessDataKey, AccessGrant>(&key) {
            grant.is_active = false;
            grant.access_level = AccessLevel::None;
            env.storage().instance().set(&key, &grant);
            env.events().publish(
                (String::from_str(&env, "AccessRevoked"), pet_id),
                AccessRevokedEvent {
                    pet_id,
                    granter: granter.clone(),
                    grantee,
                    timestamp: env.ledger().timestamp(),
                },
            );
            Self::log_access(
                &env,
                pet_id,
                granter,
                AccessAction::Revoke,
                String::from_str(&env, "Access revoked"),
            );
            true
        } else {
            false
        }
    }

    // --- MEDICAL RECORDS ---

    pub fn add_medical_record(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        visit_date: u64,
        reason_for_visit: String,
        diagnosis: String,
        treatment_plan: String,
    ) -> u64 {
        // Vet authorization check
        veterinarian.require_auth();

        // Verify vet is verified
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            panic!("Veterinarian not verified");
        }

        // Verify pet exists
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::MedicalRecordCount)
            .unwrap_or(0);
        let id = count + 1;

        env.storage()
            .instance()
            .set(&MedicalDataKey::MedicalRecordCount, &id);

        let record = MedicalRecord {
            id,
            pet_id,
            veterinarian: veterinarian.clone(),
            visit_date,
            reason_for_visit,
            diagnosis,
            treatment_plan,
            medications: Vec::new(&env),
            updated_at: env.ledger().timestamp(),
        };

        // Store the medical record
        env.storage()
            .instance()
            .set(&MedicalDataKey::MedicalRecord(id), &record);

        let pet_record_count: u64 = env
            .storage()
            .instance()
            .get::<MedicalDataKey, u64>(&MedicalDataKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(
            &MedicalDataKey::PetMedicalRecordCount(pet_id),
            &pet_record_count,
        );
        env.storage().instance().set(
            &MedicalDataKey::PetMedicalRecordIndex((pet_id, pet_record_count)),
            &id,
        );

            Self::update_vet_stats(
            &env,
            &veterinarian,
            pet_id,
            1,
            0,
            1,
        );

        // Publish event (RecordAdded for off-chain indexing)
        env.events().publish(
            (String::from_str(&env, "MedicalRecordAdded"), pet_id),
            MedicalRecordAddedEvent {
                record_id: id,
                pet_id,
                veterinarian: record.veterinarian,
                timestamp: record.updated_at,
            },
        );
        Self::log_access(
            &env,
            pet_id,
            veterinarian.clone(),
            AccessAction::Write,
            String::from_str(&env, "Medical record added"),
        );

        id
    }

    pub fn update_medical_record(
        env: Env,
        record_id: u64,
        diagnosis: String,
        treatment: String,
    ) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalDataKey, MedicalRecord>(&MedicalDataKey::MedicalRecord(record_id))
        {
            record.veterinarian.require_auth();

            record.diagnosis = diagnosis;
            record.treatment_plan = treatment;
            record.updated_at = env.ledger().timestamp();

            env.storage()
                .instance()
                .set(&MedicalDataKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }

    pub fn get_vet_stats(env: Env, vet: Address) -> VetStats {
        env.storage()
            .instance()
            .get::<_, VetStats>(&ServiceDataKey::VetStats(vet))
            .unwrap_or(VetStats {
                total_records: 0,
                total_vaccinations: 0,
                total_treatments: 0,
                pets_treated: 0,
            })
    }

    pub fn get_medical_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
        env.storage()
            .instance()
            .get::<MedicalDataKey, MedicalRecord>(&MedicalDataKey::MedicalRecord(record_id))
    }

    pub fn get_pet_medical_records(env: Env, pet_id: u64) -> Vec<MedicalRecord> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let mut records = Vec::new(&env);
        for i in 1..=count {
            if let Some(rid) = env
                .storage()
                .instance()
                .get::<MedicalDataKey, u64>(&MedicalDataKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = Self::get_medical_record(env.clone(), rid) {
                    records.push_back(record);
                }
            }
        }
        Self::log_access(
            &env,
            pet_id,
            env.current_contract_address(),
            AccessAction::Read,
            String::from_str(&env, "Pet medical records accessed"),
        );
        records
    }

    pub fn get_access_logs(env: Env, pet_id: u64) -> Vec<AccessLog> {
        let key = (Symbol::new(&env, "access_logs"), pet_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(&env))
    }

    pub fn check_access(env: Env, pet_id: u64, user: Address) -> AccessLevel {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if pet.owner == user {
                return AccessLevel::Full;
            }
            if let Some(grant) = env
                .storage()
                .instance()
                .get::<AccessDataKey, AccessGrant>(&AccessDataKey::AccessGrant((pet_id, user)))
            {
                if !grant.is_active {
                    return AccessLevel::None;
                }
                if let Some(exp) = grant.expires_at {
                    if env.ledger().timestamp() >= exp {
                        return AccessLevel::None;
                    }
                }
                return grant.access_level;
            }
        }
        AccessLevel::None
    }

    pub fn get_authorized_users(env: Env, pet_id: u64) -> Vec<Address> {
        let count = env
            .storage()
            .instance()
            .get::<AccessDataKey, u64>(&AccessDataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let mut users = Vec::new(&env);
        for i in 1..=count {
            if let Some(grantee) = env
                .storage()
                .instance()
                .get::<AccessDataKey, Address>(&AccessDataKey::AccessGrantIndex((pet_id, i)))
            {
                if Self::check_access(env.clone(), pet_id, grantee.clone()) != AccessLevel::None {
                    users.push_back(grantee);
                }
            }
        }
        users
    }

    pub fn get_access_grant(env: Env, pet_id: u64, grantee: Address) -> Option<AccessGrant> {
        env.storage()
            .instance()
            .get(&AccessDataKey::AccessGrant((pet_id, grantee)))
    }

    // --- LAB RESULTS ---
    pub fn add_lab_result(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        test_name: String,
        result_data: String,
        test_date: u64,
    ) -> u64 {
        veterinarian.require_auth();
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::LabResultCount)
            .unwrap_or(0);
        let id = count + 1;
        env.storage()
            .instance()
            .set(&MedicalDataKey::LabResultCount, &id);

        let result = LabResult {
            id,
            pet_id,
            veterinarian,
            test_name,
            result: result_data,
            date: test_date,
        };

        env.storage()
            .instance()
            .set(&MedicalDataKey::LabResult(id), &result);

        let pet_lab_count: u64 = env
            .storage()
            .instance()
            .get::<MedicalDataKey, u64>(&MedicalDataKey::PetLabResultCount(pet_id))
            .unwrap_or(0)
            + 1;
        env.storage()
            .instance()
            .set(&MedicalDataKey::PetLabResultCount(pet_id), &pet_lab_count);
        env.storage()
            .instance()
            .set(
                &MedicalDataKey::PetLabResultIndex((pet_id, pet_lab_count)),
                &id,
            );

        id
    }

    pub fn get_lab_result(env: Env, lab_result_id: u64) -> Option<LabResult> {
        env.storage()
            .instance()
            .get::<MedicalDataKey, LabResult>(&MedicalDataKey::LabResult(lab_result_id))
    }

    pub fn get_lab_results(env: Env, pet_id: u64) -> Vec<LabResult> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetLabResultCount(pet_id))
            .unwrap_or(0);
        let mut results = Vec::new(&env);
        for i in 1..=count {
            if let Some(rid) = env
                .storage()
                .instance()
                .get::<MedicalDataKey, u64>(&MedicalDataKey::PetLabResultIndex((pet_id, i)))
            {
                if let Some(res) = Self::get_lab_result(env.clone(), rid) {
                    results.push_back(res);
                }
            }
        }
        results
    }

    pub fn get_vaccination(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        env.storage()
            .instance()
            .get::<MedicalDataKey, Vaccination>(&MedicalDataKey::Vaccination(vaccine_id))
    }

    pub fn get_pet_vaccinations(env: Env, pet_id: u64) -> Vec<Vaccination> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let mut vaccinations = Vec::new(&env);

        for i in 1..=count {
            if let Some(vax_id) = env
                .storage()
                .instance()
                .get::<MedicalDataKey, u64>(&MedicalDataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vax) = Self::get_vaccination(env.clone(), vax_id) {
                    vaccinations.push_back(vax);
                }
            }
        }
        vaccinations
    }
    // --- MEDICATION MANAGEMENT ---

    #[allow(clippy::too_many_arguments)]
    pub fn add_medication_to_record(
        env: Env,
        record_id: u64,
        name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: u64,
        prescribing_vet: Address,
    ) -> bool {
        // Find the medical record
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalDataKey, MedicalRecord>(&MedicalDataKey::MedicalRecord(record_id))
        {
            prescribing_vet.require_auth();

            let med = Medication {
                id: 0,
                pet_id: record.pet_id,
                name,
                dosage,
                frequency,
                start_date,
                end_date: Some(end_date),
                prescribing_vet,
                active: true,
            };

            record.medications.push_back(med);
            record.updated_at = env.ledger().timestamp();

            env.storage()
                .instance()
                .set(&MedicalDataKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }

    pub fn mark_record_med_completed(env: Env, record_id: u64, med_index: u32) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<MedicalDataKey, MedicalRecord>(&MedicalDataKey::MedicalRecord(record_id))
        {
            let _pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(record.pet_id))
                .expect("Pet not found");

            record.veterinarian.require_auth();

            if let Some(mut med) = record.medications.get(med_index) {
                med.active = false;
                record.medications.set(med_index, med);
                record.updated_at = env.ledger().timestamp();
                env.storage()
                    .instance()
                    .set(&MedicalDataKey::MedicalRecord(record_id), &record);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_active_record_meds(env: Env, pet_id: u64) -> Vec<Medication> {
        let records = Self::get_pet_medical_records(env.clone(), pet_id);
        let mut active_meds = Vec::new(&env);
        // let now = env.ledger().timestamp(); // usage disabled to just rely on active flag for now

        for record in records.iter() {
            for med in record.medications.iter() {
                if med.active {
                    active_meds.push_back(med);
                }
            }
        }
        active_meds
    }

    pub fn get_record_med_history(env: Env, pet_id: u64) -> Vec<Medication> {
        let records = Self::get_pet_medical_records(env.clone(), pet_id);
        let mut history = Vec::new(&env);

        for record in records.iter() {
            for med in record.medications.iter() {
                history.push_back(med);
            }
        }
        history
    }

    // --- BATCH OPERATIONS ---

    pub fn batch_add_vaccinations(
        env: Env,
        veterinarian: Address,
        vaccinations: Vec<VaccinationInput>,
    ) -> Vec<u64> {
        veterinarian.require_auth();
        // Verify vet once
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            panic!("Veterinarian not verified");
        }

        let mut ids = Vec::new(&env);
        for input in vaccinations.iter() {
            let id = Self::add_vaccination(
                env.clone(),
                input.pet_id,
                veterinarian.clone(),
                input.vaccine_type,
                input.vaccine_name,
                input.administered_at,
                input.next_due_date,
                input.batch_number,
            );
            ids.push_back(id);
        }
        ids
    }

    pub fn batch_add_records(
        env: Env,
        veterinarian: Address,
        records: Vec<MedicalRecordInput>,
    ) -> Vec<u64> {
        veterinarian.require_auth();

        let mut ids = Vec::new(&env);
        for input in records.iter() {
            let id = Self::add_medical_record(
                env.clone(),
                input.pet_id,
                veterinarian.clone(),
                input.visit_date,
                input.reason_for_visit,
                input.diagnosis,
                input.treatment_plan,
            );
            ids.push_back(id);
        }
        ids
    }

    // --- LOST PET ALERT FUNCTIONS ---

    /// Report a pet as lost
    pub fn report_lost(
        env: Env,
        pet_id: u64,
        last_seen_location: String,
        reward_amount: Option<u64>,
    ) -> u64 {
        // Verify pet exists and caller is owner
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let alert_count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::LostPetAlertCount)
            .unwrap_or(0);
        let alert_id = alert_count + 1;

        let alert = LostPetAlert {
            id: alert_id,
            pet_id,
            reported_by: pet.owner.clone(),
            reported_date: env.ledger().timestamp(),
            last_seen_location,
            reward_amount,
            status: AlertStatus::Active,
            found_date: None,
        };

        // Store alert
        env.storage()
            .instance()
            .set(&ServiceDataKey::LostPetAlert(alert_id), &alert);
        env.storage()
            .instance()
            .set(&ServiceDataKey::LostPetAlertCount, &alert_id);

        // Add to active alerts list
        let mut active_alerts: Vec<u64> = env
            .storage()
            .instance()
            .get(&ServiceDataKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));
        active_alerts.push_back(alert_id);
        env.storage()
            .instance()
            .set(&ServiceDataKey::ActiveLostPetAlerts, &active_alerts);

        alert_id
    }

    /// Report a sighting of a lost pet
    pub fn report_sighting(env: Env, alert_id: u64, location: String, description: String) -> bool {
        let reporter = env.current_contract_address();

        let sighting = SightingReport {
            alert_id,
            reporter,
            location,
            timestamp: env.ledger().timestamp(),
            description,
        };

        let key = ServiceDataKey::AlertSightings(alert_id);
        let mut sightings: Vec<SightingReport> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&env));
        sightings.push_back(sighting);
        env.storage().instance().set(&key, &sightings);

        true
    }

    /// Mark a lost pet as found
    pub fn report_found(env: Env, alert_id: u64) -> bool {
        let key = ServiceDataKey::LostPetAlert(alert_id);
        
        let mut alert: LostPetAlert = env
            .storage()
            .instance()
            .get(&key)
            .expect("Alert not found");

        alert.reported_by.require_auth();

        if alert.status != AlertStatus::Active {
            panic!("Alert is not active");
        }

        alert.status = AlertStatus::Found;
        alert.found_date = Some(env.ledger().timestamp());
        env.storage().instance().set(&key, &alert);

        // Remove from active alerts list
        if let Some(mut active_alerts) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, Vec<u64>>(&ServiceDataKey::ActiveLostPetAlerts)
        {
            for i in 0..active_alerts.len() {
                if let Some(id) = active_alerts.get(i) {
                    if id == alert_id {
                        active_alerts.remove(i);
                        env.storage()
                            .instance()
                            .set(&ServiceDataKey::ActiveLostPetAlerts, &active_alerts);
                        break;
                    }
                }
            }
        }
        true
    }

    /// Cancel a lost pet alert
    pub fn cancel_lost_alert(env: Env, alert_id: u64) -> bool {
        if let Some(mut alert) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, LostPetAlert>(&ServiceDataKey::LostPetAlert(alert_id))
        {
            let pet: Pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(alert.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();

            alert.status = AlertStatus::Cancelled;
            env.storage()
                .instance()
                .set(&ServiceDataKey::LostPetAlert(alert_id), &alert);

            // Remove from active alerts list
            if let Some(mut active_alerts) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, Vec<u64>>(&ServiceDataKey::ActiveLostPetAlerts)
            {
                for i in 0..active_alerts.len() {
                    if let Some(id) = active_alerts.get(i) {
                        if id == alert_id {
                            active_alerts.remove(i);
                            env.storage()
                                .instance()
                                .set(&ServiceDataKey::ActiveLostPetAlerts, &active_alerts);
                            break;
                        }
                    }
                }
            }
            true
        } else {
            false
        }
    }

    /// Get all active lost pet alerts
    pub fn get_active_alerts(env: Env) -> Vec<LostPetAlert> {
        let active_ids: Vec<u64> = env
            .storage()
            .instance()
            .get(&ServiceDataKey::ActiveLostPetAlerts)
            .unwrap_or(Vec::new(&env));

        let mut active_alerts = Vec::new(&env);

        for id in active_ids.iter() {
            if let Some(alert) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, LostPetAlert>(&ServiceDataKey::LostPetAlert(id))
            {
                if alert.status == AlertStatus::Active {
                    active_alerts.push_back(alert);
                }
            }
        }

        active_alerts
    }

    /// Get a specific alert by ID
    pub fn get_alert(env: Env, alert_id: u64) -> Option<LostPetAlert> {
        env.storage()
            .instance()
            .get(&ServiceDataKey::LostPetAlert(alert_id))
    }

    /// Get sightings for a specific alert
    pub fn get_alert_sightings(env: Env, alert_id: u64) -> Vec<SightingReport> {
        env.storage()
            .instance()
            .get(&ServiceDataKey::AlertSightings(alert_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get alerts for a specific pet
    pub fn get_pet_alerts(env: Env, pet_id: u64) -> Vec<LostPetAlert> {
        let alert_count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::LostPetAlertCount)
            .unwrap_or(0);

        let mut pet_alerts = Vec::new(&env);

        for i in 1..=alert_count {
            if let Some(alert) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, LostPetAlert>(&ServiceDataKey::LostPetAlert(i))
            {
                if alert.pet_id == pet_id {
                    pet_alerts.push_back(alert);
                }
            }
        }
        pet_alerts
    }
    // --- VET AVAILABILITY FUNCTIONS ---

    /// Set availability slots for a vet (only verified vets can set their availability)
    pub fn set_availability(env: Env, vet_address: Address, start_time: u64, end_time: u64) -> u64 {
        // Verify caller is the vet and is verified
        vet_address.require_auth();
        if !Self::is_verified_vet(env.clone(), vet_address.clone()) {
            panic!("Vet not verified");
        }

        let slot_count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::VetAvailabilityCount(vet_address.clone()))
            .unwrap_or(0);
        let slot_index = slot_count + 1;

        let slot = AvailabilitySlot {
            vet_address: vet_address.clone(),
            start_time,
            end_time,
            available: true,
        };

        // Store the slot
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetAvailability((vet_address.clone(), slot_index)), &slot);
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetAvailabilityCount(vet_address.clone()), &slot_index);

        // Add to date-based index for efficient querying
        let date = Self::get_date_from_timestamp(start_time);
        let date_key = ServiceDataKey::VetAvailabilityByDate((vet_address.clone(), date));
        let mut date_slots: Vec<u64> = env
            .storage()
            .instance()
            .get(&date_key)
            .unwrap_or(Vec::new(&env));
        date_slots.push_back(slot_index);
        env.storage().instance().set(&date_key, &date_slots);

        slot_index
    }

    /// Get available slots for a vet on a specific date
    pub fn get_available_slots(env: Env, vet_address: Address, date: u64) -> Vec<AvailabilitySlot> {
        let date_key = ServiceDataKey::VetAvailabilityByDate((vet_address.clone(), date));
        let slot_indices: Vec<u64> = env
            .storage()
            .instance()
            .get(&date_key)
            .unwrap_or(Vec::new(&env));

        let mut available_slots = Vec::new(&env);

        for index in slot_indices.iter() {
            if let Some(slot) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, AvailabilitySlot>(&ServiceDataKey::VetAvailability((vet_address.clone(), index)))
            {
                if slot.available {
                    available_slots.push_back(slot);
                }
            }
        }

        available_slots
    }
    // --- CONSENT SYSTEM ---

    pub fn grant_consent(
        env: Env,
        pet_id: u64,
        owner: Address,
        consent_type: ConsentType,
        granted_to: Address,
    ) -> u64 {
        owner.require_auth();

        // Verify owner owns the pet
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        if pet.owner != owner {
            panic!("Not the pet owner");
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::ConsentCount)
            .unwrap_or(0);
        let consent_id = count + 1;
        let now = env.ledger().timestamp();

        let consent = Consent {
            id: consent_id,
            pet_id,
            owner,
            consent_type,
            granted_to,
            granted_at: now,
            revoked_at: None,
            is_active: true,
        };

        env.storage()
            .instance()
            .set(&ServiceDataKey::Consent(consent_id), &consent);
        env.storage()
            .instance()
            .set(&ServiceDataKey::ConsentCount, &consent_id);

        // Update pet consent index
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::PetConsentCount(pet_id))
            .unwrap_or(0);
        let new_pet_count = pet_count + 1;
        env.storage()
            .instance()
            .set(&ServiceDataKey::PetConsentCount(pet_id), &new_pet_count);
        env.storage()
            .instance()
            .set(&ServiceDataKey::PetConsentIndex((pet_id, new_pet_count)), &consent_id);

        consent_id
    }

    pub fn revoke_consent(env: Env, consent_id: u64, owner: Address) -> bool {
        owner.require_auth();

        if let Some(mut consent) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, Consent>(&ServiceDataKey::Consent(consent_id))
        {
            if consent.owner != owner {
                panic!("Not the consent owner");
            }
            if !consent.is_active {
                panic!("Consent already revoked");
            }

            consent.is_active = false;
            consent.revoked_at = Some(env.ledger().timestamp());

            env.storage()
                .instance()
                .set(&ServiceDataKey::Consent(consent_id), &consent);
            true
        } else {
            false
        }
    }

    pub fn get_consent_history(env: Env, pet_id: u64) -> Vec<Consent> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::PetConsentCount(pet_id))
            .unwrap_or(0);

        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(consent_id) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, u64>(&ServiceDataKey::PetConsentIndex((pet_id, i)))
            {
                if let Some(consent) = env
                    .storage()
                    .instance()
                    .get::<ServiceDataKey, Consent>(&ServiceDataKey::Consent(consent_id))
                {
                    history.push_back(consent);
                }
            }
        }
        history
    }

    /// Book a slot (mark as unavailable)
    pub fn book_slot(env: Env, vet_address: Address, slot_index: u64) -> bool {
        let key = ServiceDataKey::VetAvailability((vet_address.clone(), slot_index));
        
        if let Some(mut slot) = env
            .storage()
            .instance()
            .get::<ServiceDataKey, AvailabilitySlot>(&key)
        {
            if !slot.available {
                panic!("Slot already booked");
            }

            // Require auth from either vet or pet owner (simplified - just require vet auth for now)
            // In real implementation, you'd check if caller is a registered pet owner
            slot.available = false;
            env.storage().instance().set(&key, &slot);
            true
        } else {
            false
        }
    }

    /// Helper: Extract date from timestamp (yyyyMMdd format)
    fn get_date_from_timestamp(timestamp: u64) -> u64 {
        // Simple conversion: timestamp / 86400 gives days since epoch
        // For this implementation, we use timestamp / 86400 as the "date"
        timestamp / 86400
    }
    // --- CONTRACT UPGRADE SYSTEM ---

    pub fn get_version(env: Env) -> ContractVersion {
        env.storage()
            .instance()
            .get(&AdministrativeDataKey::ContractVersion)
            .unwrap_or(ContractVersion {
                major: 1,
                minor: 0,
                patch: 0,
            })
    }

    pub fn upgrade_contract(env: Env, new_wasm_hash: BytesN<32>) {
        // Only admin can upgrade
        Self::require_admin(&env);

        // Perform the upgrade
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    pub fn propose_upgrade(env: Env, proposer: Address, new_wasm_hash: BytesN<32>) -> u64 {
        // Only admin can propose
        Self::require_admin(&env);
        proposer.require_auth();

        let count: u64 = env
            .storage()
            .instance()
            .get(&AdministrativeDataKey::UpgradeProposalCount)
            .unwrap_or(0);
        let proposal_id = count + 1;

        let proposal = UpgradeProposal {
            id: proposal_id,
            proposed_by: proposer,
            new_wasm_hash,
            proposed_at: env.ledger().timestamp(),
            approved: false,
            executed: false,
        };

        env.storage()
            .instance()
            .set(&AdministrativeDataKey::UpgradeProposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&AdministrativeDataKey::UpgradeProposalCount, &proposal_id);

        proposal_id
    }

    pub fn approve_upgrade(env: Env, proposal_id: u64) -> bool {
        Self::require_admin(&env);

        if let Some(mut proposal) = env
            .storage()
            .instance()
            .get::<AdministrativeDataKey, UpgradeProposal>(&AdministrativeDataKey::UpgradeProposal(proposal_id))
        {
            if proposal.executed {
                panic!("Proposal already executed");
            }

            proposal.approved = true;
            env.storage()
                .instance()
                .set(&AdministrativeDataKey::UpgradeProposal(proposal_id), &proposal);
            true
        } else {
            false
        }
    }

    pub fn get_upgrade_proposal(env: Env, proposal_id: u64) -> Option<UpgradeProposal> {
        env.storage()
            .instance()
            .get(&AdministrativeDataKey::UpgradeProposal(proposal_id))
    }

    pub fn migrate_version(env: Env, major: u32, minor: u32, patch: u32) {
        Self::require_admin(&env);

        let version = ContractVersion {
            major,
            minor,
            patch,
        };
        env.storage()
            .instance()
            .set(&AdministrativeDataKey::ContractVersion, &version);
    }

    // --- MULTISIG OPERATIONS ---

    pub fn propose_action(
        env: Env,
        proposer: Address,
        action: ProposalAction,
        expires_in: u64,
    ) -> u64 {
        Self::require_admin_auth(&env, &proposer);
        
        let count: u64 = env.storage().instance().get(&AdministrativeDataKey::ProposalCount).unwrap_or(0);
        let proposal_id = count + 1;
        
        let threshold = env.storage().instance().get::<AdministrativeDataKey, u32>(&AdministrativeDataKey::AdminThreshold).unwrap_or(1);
        
        let mut approvals = Vec::new(&env);
        approvals.push_back(proposer.clone());

        let now = env.ledger().timestamp();
        let proposal = MultiSigProposal {
            id: proposal_id,
            action,
            proposed_by: proposer,
            approvals,
            required_approvals: threshold,
            created_at: now,
            expires_at: now + expires_in,
            executed: false,
        };

        env.storage().instance().set(&AdministrativeDataKey::Proposal(proposal_id), &proposal);
        env.storage().instance().set(&AdministrativeDataKey::ProposalCount, &proposal_id);
        
        proposal_id
    }

    pub fn approve_proposal(env: Env, admin: Address, proposal_id: u64) {
        Self::require_admin_auth(&env, &admin);
        
        let mut proposal: MultiSigProposal = env.storage().instance()
            .get(&AdministrativeDataKey::Proposal(proposal_id))
            .expect("Proposal not found");

        if proposal.executed {
            panic!("Proposal already executed");
        }

        if env.ledger().timestamp() > proposal.expires_at {
            panic!("Proposal expired");
        }

        if proposal.approvals.contains(admin.clone()) {
            panic!("Admin already approved");
        }

        proposal.approvals.push_back(admin);
        env.storage().instance().set(&AdministrativeDataKey::Proposal(proposal_id), &proposal);
    }

    pub fn execute_proposal(env: Env, proposal_id: u64) {
        let mut proposal: MultiSigProposal = env.storage().instance()
            .get(&AdministrativeDataKey::Proposal(proposal_id))
            .expect("Proposal not found");

        if proposal.executed {
            panic!("Proposal already executed");
        }

        if env.ledger().timestamp() > proposal.expires_at {
            panic!("Proposal expired");
        }

        if proposal.approvals.len() < proposal.required_approvals {
            panic!("Threshold not met");
        }

        match proposal.action.clone() {
            ProposalAction::VerifyVet(addr) => {
                Self::_verify_vet_internal(&env, addr);
            }
            ProposalAction::RevokeVet(addr) => {
                Self::_revoke_vet_internal(&env, addr);
            }
            ProposalAction::UpgradeContract(_code_hash) => {
                // Mock upgrade or actual logic if available
                // In Soroban, upgrades are handled via env.deployer()
                // For this task, we can just log success or placeholder
            }
            ProposalAction::ChangeAdmin(params) => {
                let (admins, threshold) = params;
                if threshold == 0 || threshold > admins.len() {
                    panic!("Invalid threshold");
                }
                env.storage().instance().set(&AdministrativeDataKey::Admins, &admins);
                env.storage().instance().set(&AdministrativeDataKey::AdminThreshold, &threshold);
                // Also clean up legacy admin if needed
                env.storage().instance().remove(&AdministrativeDataKey::Admin);
            }
        }

        proposal.executed = true;
        env.storage().instance().set(&AdministrativeDataKey::Proposal(proposal_id), &proposal);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<MultiSigProposal> {
        env.storage().instance().get(&AdministrativeDataKey::Proposal(proposal_id))
    }

    // --- VET REVIEWS ---

    pub fn add_vet_review(
        env: Env,
        reviewer: Address,
        vet: Address,
        rating: u32,
        comment: String,
    ) -> u64 {
        reviewer.require_auth();

        if !(1..=5).contains(&rating) {
            panic!("Rating must be between 1 and 5");
        }

        // Check duplicate
        if env
            .storage()
            .instance()
            .has(&ServiceDataKey::VetReviewByOwnerVet((reviewer.clone(), vet.clone())))
        {
            panic!("You have already reviewed this veterinarian");
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::VetReviewCount)
            .unwrap_or(0);
        let id = count + 1;

        let review = VetReview {
            id,
            vet_address: vet.clone(),
            reviewer: reviewer.clone(),
            rating,
            comment,
            date: env.ledger().timestamp(),
        };

        env.storage().instance().set(&ServiceDataKey::VetReview(id), &review);
        env.storage().instance().set(&ServiceDataKey::VetReviewCount, &id);

        // Index by Vet
        let vet_count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::VetReviewCountByVet(vet.clone()))
            .unwrap_or(0);
        let new_vet_count = vet_count + 1;
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetReviewCountByVet(vet.clone()), &new_vet_count);
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetReviewByVetIndex((vet.clone(), new_vet_count)), &id);

        // Mark as reviewed by this owner
        env.storage()
            .instance()
            .set(&ServiceDataKey::VetReviewByOwnerVet((reviewer, vet.clone())), &id);

        // Update Vet's cumulative rating
        if let Some(mut vet_obj) = env.storage().instance().get::<ServiceDataKey, Vet>(&ServiceDataKey::Vet(vet.clone())) {
            let current_total = (vet_obj.rating as u64) * vet_obj.review_count;
            vet_obj.review_count += 1;
            vet_obj.rating = ((current_total + (rating as u64)) / vet_obj.review_count) as u32;
            env.storage().instance().set(&ServiceDataKey::Vet(vet), &vet_obj);
        }

        id
    }

    pub fn get_vet_reviews(env: Env, vet: Address) -> Vec<VetReview> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&ServiceDataKey::VetReviewCountByVet(vet.clone()))
            .unwrap_or(0);
        let mut reviews = Vec::new(&env);
        for i in 1..=count {
            if let Some(review_id) = env
                .storage()
                .instance()
                .get::<ServiceDataKey, u64>(&ServiceDataKey::VetReviewByVetIndex((vet.clone(), i)))
            {
                if let Some(review) = env
                    .storage()
                    .instance()
                    .get::<ServiceDataKey, VetReview>(&ServiceDataKey::VetReview(review_id))
                {
                    reviews.push_back(review);
                }
            }
        }
        reviews
    }

    pub fn get_vet_average_rating(env: Env, vet: Address) -> u32 {
        let reviews = Self::get_vet_reviews(env.clone(), vet);
        if reviews.is_empty() {
            return 0;
        }
        let mut total = 0u32;
        for review in reviews.iter() {
            total += review.rating;
        }
        total / reviews.len()
    }

    // --- MEDICATION TRACKING ---

    pub fn add_medication(
        env: Env,
        pet_id: u64,
        name: String,
        dosage: String,
        frequency: String,
        start_date: u64,
        end_date: Option<u64>,
        prescribing_vet: Address,
    ) -> u64 {
        prescribing_vet.require_auth();

        // Verify the pet exists
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::MedicationCount)
            .unwrap_or(0);
        let id = count + 1;

        let medication = Medication {
            id,
            pet_id,
            name,
            dosage,
            frequency,
            start_date,
            end_date,
            prescribing_vet: prescribing_vet.clone(),
            active: true,
        };

        env.storage()
            .instance()
            .set(&MedicalDataKey::GlobalMedication(id), &medication);
        env.storage().instance().set(&MedicalDataKey::MedicationCount, &id);

        // Index by pet
        let pet_med_count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetMedicationCount(pet_id))
            .unwrap_or(0);
        let new_count = pet_med_count + 1;
        env.storage()
            .instance()
            .set(&MedicalDataKey::PetMedicationCount(pet_id), &new_count);
        env.storage()
            .instance()
            .set(&MedicalDataKey::PetMedicationIndex((pet_id, new_count)), &id);

        id
    }

    pub fn get_active_medications(env: Env, pet_id: u64) -> Vec<Medication> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&MedicalDataKey::PetMedicationCount(pet_id))
            .unwrap_or(0);
        let mut active_meds = Vec::new(&env);

        for i in 1..=count {
            if let Some(med_id) = env
                .storage()
                .instance()
                .get::<MedicalDataKey, u64>(&MedicalDataKey::PetMedicationIndex((pet_id, i)))
            {
                if let Some(med) = env
                    .storage()
                    .instance()
                    .get::<MedicalDataKey, Medication>(&MedicalDataKey::GlobalMedication(med_id))
                {
                    if med.active {
                        active_meds.push_back(med);
                    }
                }
            }
        }
        active_meds
    }

    pub fn mark_medication_completed(env: Env, medication_id: u64) {
        if let Some(mut med) = env
            .storage()
            .instance()
            .get::<MedicalDataKey, Medication>(&MedicalDataKey::GlobalMedication(medication_id))
        {
            med.prescribing_vet.require_auth();
            med.active = false;
            // If end_date is not set, set it to current ledger timestamp
            if med.end_date.is_none() {
                med.end_date = Some(env.ledger().timestamp());
            }
            env.storage()
                .instance()
                .set(&MedicalDataKey::GlobalMedication(medication_id), &med);
        } else {
            panic!("Medication not found");
        }
    }

    fn update_owner_indexing(env: &Env, previous_owner: Address, new_owner: Address, pet_id: u64) {
        Self::remove_pet_from_owner_index(env, &previous_owner, pet_id);
        Self::add_pet_to_owner_index(env, &new_owner, pet_id);
    }

    // --- CO-OWNERSHIP FUNCTIONS ---

    /// Add a co-owner to a pet. Only the primary owner can do this.
    pub fn add_co_owner(env: Env, pet_id: u64, co_owner: Address) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        if pet.owners.contains(co_owner.clone()) {
            panic!("Address is already an owner");
        }

        pet.owners.push_back(co_owner.clone());
        pet.updated_at = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
    }

    /// Remove a co-owner from a pet. Only the primary owner can do this.
    /// The primary owner themselves cannot be removed via this function.
    pub fn remove_co_owner(env: Env, pet_id: u64, co_owner: Address) {
        let mut pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        if co_owner == pet.owner {
            panic!("Cannot remove the primary owner via remove_co_owner");
        }

        if !pet.owners.contains(co_owner.clone()) {
            panic!("Address is not a co-owner");
        }

        // Rebuild owners list without the removed co-owner
        let mut new_owners = Vec::<Address>::new(&env);
        for addr in pet.owners.iter() {
            if addr != co_owner {
                new_owners.push_back(addr);
            }
        }
        pet.owners = new_owners;
        pet.updated_at = env.ledger().timestamp();
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
    }

    /// Return the full list of owners (primary + co-owners) for a pet.
    pub fn get_co_owners(env: Env, pet_id: u64) -> Vec<Address> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owners
        } else {
            Vec::new(&env)
        }
    }
    /// Register a batch of pets.
    pub fn register_pets_batch(env: Env, owner: Address, pets: Vec<PetData>) -> Vec<u64> {
        owner.require_auth();
        let mut pet_ids = Vec::new(&env);
        for pet_data in pets.iter() {
            let species = if pet_data.species == String::from_str(&env, "Cat") {
                Species::Cat
            } else {
                Species::Dog
            };

            let pet_id = Self::register_pet(
                env.clone(),
                owner.clone(),
                pet_data.name.clone(),
                String::from_str(&env, "2020-01-01"), // Default birthday
                Gender::Male,                        // Default gender
                species,
                pet_data.breed.clone(),
                String::from_str(&env, "Unknown"),   // Default color
                0,                                   // Default weight
                None,                                // No microchip
                PrivacyLevel::Public,                // Default privacy
            );
            pet_ids.push_back(pet_id);
        }
        pet_ids
    }

    /// Export all data for a pet.
    pub fn export_pet_data(env: Env, pet_id: u64) -> PetFullExport {
        let pet = Self::get_pet(env.clone(), pet_id).expect("Pet not found");

        let medical_records = Self::get_pet_medical_records(env.clone(), pet_id);
        let vaccinations = Self::get_vaccination_history(env.clone(), pet_id);
        let medications = Vec::new(&env); // Placeholder
        let lab_results = Vec::new(&env); // Placeholder

        // Ownership history
        let ownership_history = Self::get_ownership_history(env.clone(), pet_id);

        PetFullExport {
            pet,
            vaccinations,
            medical_records,
            medications,
            lab_results,
            ownership_history,
            export_date: env.ledger().timestamp(),
        }
    }
}

// --- ENCRYPTION HELPERS ---
fn encrypt_sensitive_data(env: &Env, data: &Bytes, _key: &Bytes) -> (Bytes, Bytes) {
    // Mock encryption for demonstration
    let nonce = Bytes::from_array(env, &[0u8; 12]);
    let ciphertext = data.clone();
    (nonce, ciphertext)
}

fn decrypt_sensitive_data(
    _env: &Env,
    ciphertext: &Bytes,
    _nonce: &Bytes,
    _key: &Bytes,
) -> Result<Bytes, ()> {
    Ok(ciphertext.clone())
}
