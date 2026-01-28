#![no_std]
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec};

// ==================== CORE DATA STRUCTURES ====================

#[contracttype]
#[derive(Clone)]
pub struct Pet {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub species: String,
    pub gender: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub encrypted_emergency_contacts: EncryptedData,
    pub encrypted_med_info: EncryptedData,
    pub encrypted_medical_alerts: EncryptedData,
}

#[contracttype]
#[derive(Clone)]
pub struct EncryptedData {
    pub nonce: Bytes,
    pub ciphertext: Bytes,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyContactInfo {
    pub name: String,
    pub phone: String,
    pub relationship: String,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyMedicalInfo {
    pub allergies: Vec<AllergyInfo>,
    pub medical_notes: String,
    pub critical_alerts: Vec<String>,
    pub last_updated: u64,
    pub updated_by: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct AllergyInfo {
    pub allergen: String,
    pub severity: String,
    pub notes: String,
}

#[contracttype]
#[derive(Clone)]
pub struct EmergencyInfoResponse {
    pub pet_id: u64,
    pub species: String,
    pub gender: String,
    pub emergency_contacts: Vec<EmergencyContactInfo>,
    pub emergency_medical_info: Option<EmergencyMedicalInfo>,
    pub last_updated: u64,
}

// ==================== TREATMENT STRUCTURES ====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreatmentType {
    Surgery,
    Medication,
    Therapy,
    Checkup,
    Diagnostic,
    Dental,
    Other,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreatmentStatus {
    Scheduled,
    Ongoing,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreatmentOutcome {
    Pending,
    Successful,
    Complications,
    Failed,
    NotApplicable,
}

#[contracttype]
#[derive(Clone)]
pub struct Treatment {
    pub id: u64,
    pub pet_id: u64,
    pub vet_address: Address,
    pub treatment_type: TreatmentType,
    pub description: String,
    pub notes: String,
    pub date: u64,
    pub cost: i64,
    pub status: TreatmentStatus,
    pub outcome: TreatmentOutcome,
}

// ==================== VACCINATION STRUCTURES ====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VaccineType {
    Rabies,
    Distemper,
    Parvovirus,
    Bordetella,
    FeLV,
    Other,
}

#[contracttype]
#[derive(Clone)]
pub struct Vaccination {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub vaccine_name: Option<String>,
    pub encrypted_vaccine_name: EncryptedData,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: Option<String>,
    pub encrypted_batch_number: EncryptedData,
    pub created_at: u64,
    pub side_effects: Option<String>,
}

#[contracttype]
#[derive(Clone)]
pub struct VaccinationAddedEvent {
    pub vaccine_id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub vaccine_type: VaccineType,
    pub next_due_date: u64,
    pub timestamp: u64,
}

// ==================== TAG STRUCTURES ====================

#[contracttype]
#[derive(Clone)]
pub struct PetTag {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub message: String,
    pub is_active: bool,
    pub linked_at: u64,
    pub created_at: u64,
    pub updated_at: u64,
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
#[derive(Clone)]
pub struct TagReactivatedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub reactivated_by: Address,
    pub timestamp: u64,
}

// ==================== ACCESS CONTROL ====================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessLevel {
    None,
    Basic,
    Full,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessGrant {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub access_level: AccessLevel,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessGrantedEvent {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub access_level: AccessLevel,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct AccessRevokedEvent {
    pub pet_id: u64,
    pub granter: Address,
    pub grantee: Address,
    pub timestamp: u64,
}

// ==================== MEDICAL RECORDS ====================

#[contracttype]
#[derive(Clone)]
pub struct MedicalRecord {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub record_type: String,
    pub diagnosis: String,
    pub treatment: String,
    pub medications: Vec<Medication>,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct Medication {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
}

#[contracttype]
#[derive(Clone)]
pub struct MedicalRecordAddedEvent {
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct LabResult {
    pub id: u64,
    pub pet_id: u64,
    pub veterinarian: Address,
    pub test_type: String,
    pub result_summary: String,
    pub medical_record_id: Option<u64>,
    pub created_at: u64,
}

// ==================== VET STRUCTURES ====================

#[contracttype]
#[derive(Clone)]
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
}

// ==================== PET PROFILE ====================

#[contracttype]
#[derive(Clone)]
pub struct PetProfile {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub species: String,
    pub gender: String,
    pub active: bool,
}

// ==================== DATA KEY ENUM ====================

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetCountByOwner(Address),
    OwnerPetIndex((Address, u64)),

    // Treatment
    Treatment(u64),
    TreatmentCount,
    PetTreatmentCount(u64),
    PetTreatmentByIndex((u64, u64)),

    // Vaccination
    Vaccination(u64),
    VaccinationCount,
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),

    // Tags
    Tag(BytesN<32>),
    PetTagId(u64),
    PetTagCount,

    // Access Control
    AccessGrant((u64, Address)),
    AccessGrantCount(u64),
    AccessGrantIndex((u64, u64)),
    UserAccessCount(Address),

    // Medical Records
    MedicalRecord(u64),
    MedicalRecordCount,
    PetMedicalRecordCount(u64),
    PetMedicalRecordIndex((u64, u64)),
    EmergencyMedicalInfo(u64),
    EmergencyContacts(u64),

    // Lab Results
    LabResult(u64),
    LabResultCount,
    PetLabResultCount(u64),
    PetLabResultIndex((u64, u64)),

    // Vet
    Vet(Address),
    VetLicense(String),
}

// ==================== CONTRACT ====================

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    // ==================== PET MANAGEMENT ====================

    pub fn register_pet(
        env: Env,
        owner: Address,
        name: String,
        species: String,
        gender: String,
    ) -> u64 {
        owner.require_auth();

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count + 1;
        let now = env.ledger().timestamp();

        let empty_encrypted = EncryptedData {
            nonce: Bytes::new(&env),
            ciphertext: Bytes::new(&env),
        };

        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            name,
            species,
            gender,
            active: true,
            created_at: now,
            updated_at: now,
            encrypted_emergency_contacts: empty_encrypted.clone(),
            encrypted_med_info: empty_encrypted.clone(),
            encrypted_medical_alerts: empty_encrypted,
        };

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);

        // Update owner index
        let owner_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0);
        let new_owner_count = owner_count + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &new_owner_count);
        env.storage()
            .instance()
            .set(&DataKey::OwnerPetIndex((owner, new_owner_count)), &pet_id);

        pet_id
    }

    pub fn get_pet(env: Env, pet_id: u64) -> Option<PetProfile> {
        env.storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .map(|pet| PetProfile {
                id: pet.id,
                owner: pet.owner,
                name: pet.name,
                species: pet.species,
                gender: pet.gender,
                active: pet.active,
            })
    }

    pub fn update_pet_status(env: Env, pet_id: u64, active: bool) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();
            pet.active = active;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        }
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

    // ==================== TREATMENT MANAGEMENT ====================

    pub fn add_treatment(
        env: Env,
        pet_id: u64,
        vet_address: Address,
        treatment_type: TreatmentType,
        description: String,
        notes: String,
        date: u64,
        cost: i64,
        status: TreatmentStatus,
        outcome: TreatmentOutcome,
    ) -> u64 {
        vet_address.require_auth();

        if env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .is_none()
        {
            panic!("Pet not found");
        }

        let treatment_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TreatmentCount)
            .unwrap_or(0);
        let treatment_id = treatment_count + 1;

        let treatment = Treatment {
            id: treatment_id,
            pet_id,
            vet_address,
            treatment_type,
            description,
            notes,
            date,
            cost,
            status,
            outcome,
        };

        env.storage()
            .instance()
            .set(&DataKey::Treatment(treatment_id), &treatment);
        env.storage()
            .instance()
            .set(&DataKey::TreatmentCount, &treatment_id);

        let pet_treatment_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetTreatmentCount(pet_id))
            .unwrap_or(0);
        let new_count = pet_treatment_count + 1;

        env.storage()
            .instance()
            .set(&DataKey::PetTreatmentCount(pet_id), &new_count);
        env.storage().instance().set(
            &DataKey::PetTreatmentByIndex((pet_id, new_count)),
            &treatment_id,
        );

        treatment_id
    }

    pub fn get_treatment_history(
        env: Env,
        pet_id: u64,
        filter_type: Option<TreatmentType>,
        filter_vet: Option<Address>,
        min_date: Option<u64>,
    ) -> Vec<Treatment> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetTreatmentCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(t_id) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::PetTreatmentByIndex((pet_id, i)))
            {
                if let Some(treatment) = env
                    .storage()
                    .instance()
                    .get::<DataKey, Treatment>(&DataKey::Treatment(t_id))
                {
                    let type_match = match &filter_type {
                        Some(t) => *t == treatment.treatment_type,
                        None => true,
                    };

                    let vet_match = match &filter_vet {
                        Some(v) => *v == treatment.vet_address,
                        None => true,
                    };

                    let date_match = match min_date {
                        Some(d) => treatment.date >= d,
                        None => true,
                    };

                    if type_match && vet_match && date_match {
                        history.push_back(treatment);
                    }
                }
            }
        }
        history
    }

    // ==================== VACCINATION MANAGEMENT ====================

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
            .get(&DataKey::VaccinationCount)
            .unwrap_or(0);
        let vaccine_id = vaccine_count + 1;
        let now = env.ledger().timestamp();

        // For MVP, store directly without encryption
        // In production, implement proper encryption
        let empty_encrypted = EncryptedData {
            nonce: Bytes::new(&env),
            ciphertext: Bytes::new(&env),
        };

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian: veterinarian.clone(),
            vaccine_type: vaccine_type.clone(),
            vaccine_name: Some(vaccine_name),
            encrypted_vaccine_name: empty_encrypted.clone(),
            administered_at,
            next_due_date,
            batch_number: Some(batch_number),
            encrypted_batch_number: empty_encrypted,
            created_at: now,
            side_effects: None,
        };

        env.storage()
            .instance()
            .set(&DataKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&DataKey::VaccinationCount, &vaccine_id);

        let pet_vax_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let new_pet_vax_count = pet_vax_count + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetVaccinationCount(pet_id), &new_pet_vax_count);
        env.storage().instance().set(
            &DataKey::PetVaccinationByIndex((pet_id, new_pet_vax_count)),
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
        env.storage()
            .instance()
            .get::<DataKey, Vaccination>(&DataKey::Vaccination(vaccine_id))
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

        let storage = env.storage().instance();
        let vax_count: u64 = storage
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut history = Vec::new(&env);

        for i in 1..=vax_count {
            if let Some(vax_id) =
                storage.get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) =
                    storage.get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    history.push_back(vaccination);
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
        let threshold_time = current_time + (days_threshold * 86400);

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let storage = env.storage().instance();
        let vax_count: u64 = storage
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut upcoming = Vec::new(&env);

        for i in 1..=vax_count {
            if let Some(vax_id) =
                storage.get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) =
                    storage.get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    if vaccination.next_due_date <= threshold_time {
                        upcoming.push_back(vaccination);
                    }
                }
            }
        }
        upcoming
    }

    pub fn is_vaccination_current(env: Env, pet_id: u64, vaccine_type: VaccineType) -> bool {
        let current_time = env.ledger().timestamp();

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let storage = env.storage().instance();
        let vax_count: u64 = storage
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut most_recent: Option<Vaccination> = None;

        for i in 1..=vax_count {
            if let Some(vax_id) =
                storage.get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) =
                    storage.get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    if vaccination.vaccine_type == vaccine_type {
                        if let Some(ref current) = most_recent {
                            if vaccination.administered_at > current.administered_at {
                                most_recent = Some(vaccination);
                            }
                        } else {
                            most_recent = Some(vaccination);
                        }
                    }
                }
            }
        }

        if let Some(vax) = most_recent {
            vax.next_due_date > current_time
        } else {
            false
        }
    }

    pub fn update_vaccination_record(
        env: Env,
        vaccination_id: u64,
        new_side_effects: Option<String>,
        new_next_due_date: Option<u64>,
    ) {
        let mut record: Vaccination = env
            .storage()
            .instance()
            .get(&DataKey::Vaccination(vaccination_id))
            .expect("Vaccination record not found");

        record.veterinarian.require_auth();

        if let Some(side_effects) = new_side_effects {
            record.side_effects = Some(side_effects);
        }

        if let Some(next_due) = new_next_due_date {
            record.next_due_date = next_due;
        }

        env.storage()
            .instance()
            .set(&DataKey::Vaccination(vaccination_id), &record);
    }

    pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();

        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let storage = env.storage().instance();
        let vax_count: u64 = storage
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut overdue = Vec::new(&env);

        for i in 1..=vax_count {
            if let Some(vax_id) =
                storage.get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) =
                    storage.get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    if vaccination.next_due_date < current_time {
                        overdue.push_back(vaccination);
                    }
                }
            }
        }
        overdue
    }

    // ==================== TAG MANAGEMENT ====================

    fn generate_tag_id(env: &Env, pet_id: u64, owner: &Address) -> BytesN<32> {
        let mut data = Bytes::new(env);
        data.append(&pet_id.to_xdr(env));
        data.append(&owner.to_xdr(env));
        data.append(&env.ledger().timestamp().to_xdr(env));

        env.crypto().sha256(&data).into()
    }

    pub fn link_pet_tag(env: Env, pet_id: u64) -> BytesN<32> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        if env
            .storage()
            .instance()
            .get::<DataKey, BytesN<32>>(&DataKey::PetTagId(pet_id))
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
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .instance()
            .set(&DataKey::Tag(tag_id.clone()), &pet_tag);
        env.storage()
            .instance()
            .set(&DataKey::PetTagId(pet_id), &tag_id);

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetTagCount)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::PetTagCount, &(count + 1));

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
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id))
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
        env.storage().instance().get(&DataKey::Tag(tag_id))
    }

    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage().instance().get(&DataKey::PetTagId(pet_id))
    }

    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id.clone()))
        {
            let pet = env
                .storage()
                .instance()
                .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                .expect("Pet not found");
            pet.owner.require_auth();

            tag.message = message;
            tag.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Tag(tag_id), &tag);
            true
        } else {
            false
        }
    }

    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id.clone()))
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
                .set(&DataKey::Tag(tag_id.clone()), &tag);

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
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id.clone()))
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
                .set(&DataKey::Tag(tag_id.clone()), &tag);

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
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id))
        {
            tag.is_active
        } else {
            false
        }
    }

    // ==================== EMERGENCY INFO ====================

    pub fn set_emergency_contacts(
        env: Env,
        pet_id: u64,
        contacts: Vec<EmergencyContactInfo>,
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();

            // Store contacts directly in a separate key
            env.storage()
                .instance()
                .set(&DataKey::EmergencyContacts(pet_id), &contacts);

            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

            env.events().publish(
                (String::from_str(&env, "EmergencyContactsUpdated"), pet_id),
                (pet_id, pet.owner.clone(), pet.updated_at),
            );

            true
        } else {
            false
        }
    }

    pub fn set_emergency_medical_info(
        env: Env,
        pet_id: u64,
        allergies: Vec<AllergyInfo>,
        medical_notes: String,
        critical_alerts: Vec<String>,
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            pet.owner.require_auth();

            let timestamp = env.ledger().timestamp();
            let caller = pet.owner.clone();

            let emergency_medical_info = EmergencyMedicalInfo {
                allergies: allergies.clone(),
                medical_notes: medical_notes.clone(),
                critical_alerts: critical_alerts.clone(),
                last_updated: timestamp,
                updated_by: caller,
            };

            // Store directly without complex encryption for now
            env.storage().instance().set(
                &DataKey::EmergencyMedicalInfo(pet_id),
                &emergency_medical_info,
            );

            pet.updated_at = timestamp;
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

            env.events().publish(
                (
                    String::from_str(&env, "EmergencyMedicalInfoUpdated"),
                    pet_id,
                ),
                (pet_id, pet.owner.clone(), timestamp),
            );

            true
        } else {
            false
        }
    }

    pub fn get_emergency_info(env: Env, pet_id: u64) -> Option<EmergencyInfoResponse> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if !pet.active {
                return None;
            }

            let emergency_contacts = env
                .storage()
                .instance()
                .get::<DataKey, Vec<EmergencyContactInfo>>(&DataKey::EmergencyContacts(pet_id))
                .unwrap_or(Vec::new(&env));

            let emergency_medical_info = env
                .storage()
                .instance()
                .get::<DataKey, EmergencyMedicalInfo>(&DataKey::EmergencyMedicalInfo(pet_id));

            Some(EmergencyInfoResponse {
                pet_id,
                species: pet.species,
                gender: pet.gender,
                emergency_contacts,
                emergency_medical_info,
                last_updated: pet.updated_at,
            })
        } else {
            None
        }
    }

    pub fn get_emergency_contacts(env: Env, pet_id: u64) -> Option<Vec<EmergencyContactInfo>> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if !pet.active {
                return None;
            }

            env.storage()
                .instance()
                .get::<DataKey, Vec<EmergencyContactInfo>>(&DataKey::EmergencyContacts(pet_id))
        } else {
            None
        }
    }

    pub fn get_medical_alerts(env: Env, pet_id: u64) -> Option<EmergencyMedicalInfo> {
        if let Some(pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
        {
            if !pet.active {
                return None;
            }

            env.storage()
                .instance()
                .get::<DataKey, EmergencyMedicalInfo>(&DataKey::EmergencyMedicalInfo(pet_id))
        } else {
            None
        }
    }

    // ==================== VET MANAGEMENT ====================

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
            .has(&DataKey::VetLicense(license_number.clone()))
        {
            panic!("License already registered");
        }

        if env
            .storage()
            .instance()
            .has(&DataKey::Vet(vet_address.clone()))
        {
            panic!("Vet already registered");
        }

        let vet = Vet {
            address: vet_address.clone(),
            name,
            license_number: license_number.clone(),
            specialization,
            verified: false,
        };

        env.storage()
            .instance()
            .set(&DataKey::Vet(vet_address.clone()), &vet);
        env.storage()
            .instance()
            .set(&DataKey::VetLicense(license_number), &vet_address);

        true
    }

    pub fn verify_vet(env: Env, vet_address: Address) -> bool {
        Self::require_admin(&env);

        if let Some(mut vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address.clone()))
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

    pub fn is_verified_vet(env: Env, vet_address: Address) -> bool {
        if let Some(vet) = env
            .storage()
            .instance()
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {
            vet.verified
        } else {
            false
        }
    }

    // ==================== ACCESS CONTROL ====================

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

        let now = env.ledger().timestamp();

        let grant = AccessGrant {
            pet_id,
            granter: pet.owner.clone(),
            grantee: grantee.clone(),
            access_level: access_level.clone(),
            granted_at: now,
            expires_at,
            is_active: true,
        };

        let storage = env.storage().instance();
        let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
        let is_new_grant = storage.get::<DataKey, AccessGrant>(&grant_key).is_none();

        storage.set(&grant_key, &grant);

        if is_new_grant {
            let grant_count: u64 = storage.get(&DataKey::AccessGrantCount(pet_id)).unwrap_or(0);
            let new_count = grant_count + 1;
            storage.set(&DataKey::AccessGrantCount(pet_id), &new_count);
            storage.set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

            let user_access_count: u64 = storage
                .get(&DataKey::UserAccessCount(grantee.clone()))
                .unwrap_or(0);
            storage.set(
                &DataKey::UserAccessCount(grantee.clone()),
                &(user_access_count + 1),
            );
        }

        env.events().publish(
            (String::from_str(&env, "AccessGranted"), pet_id),
            AccessGrantedEvent {
                pet_id,
                granter: pet.owner,
                grantee,
                access_level,
                granted_at: now,
                expires_at,
                is_active: true,
            },
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

        let key = DataKey::AccessGrant((pet_id, grantee.clone()));
        if let Some(mut grant) = env.storage().instance().get::<DataKey, AccessGrant>(&key) {
            grant.is_active = false;
            grant.access_level = AccessLevel::None;
            env.storage().instance().set(&key, &grant);
            env.events().publish(
                (String::from_str(&env, "AccessRevoked"), pet_id),
                AccessRevokedEvent {
                    pet_id,
                    granter: pet.owner,
                    grantee,
                    timestamp: env.ledger().timestamp(),
                },
            );
            true
        } else {
            false
        }
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
                .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, user)))
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
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let storage = env.storage().instance();
        let grant_count: u64 = storage.get(&DataKey::AccessGrantCount(pet_id)).unwrap_or(0);

        let mut authorized_users = Vec::new(&env);

        for i in 1..=grant_count {
            if let Some(grantee) =
                storage.get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
            {
                let access_level = if pet.owner == grantee {
                    AccessLevel::Full
                } else {
                    let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
                    if let Some(grant) = storage.get::<DataKey, AccessGrant>(&grant_key) {
                        if !grant.is_active {
                            AccessLevel::None
                        } else {
                            if let Some(exp_time) = grant.expires_at {
                                let now = env.ledger().timestamp();
                                if now >= exp_time {
                                    AccessLevel::None
                                } else {
                                    grant.access_level
                                }
                            } else {
                                grant.access_level
                            }
                        }
                    } else {
                        AccessLevel::None
                    }
                };

                if access_level != AccessLevel::None {
                    authorized_users.push_back(grantee);
                }
            }
        }
        authorized_users
    }

    pub fn get_access_grant(env: Env, pet_id: u64, grantee: Address) -> Option<AccessGrant> {
        env.storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, grantee)))
    }

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

    // ==================== ADDITIONAL ACCESS CONTROL HELPERS ====================

    /// Get all pets that a user has been granted access to (not owned)
    pub fn get_granted_access_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();

        let user_access_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::UserAccessCount(user.clone()))
            .unwrap_or(0);

        let mut granted_pets = Vec::new(&env);

        // This is a simplified approach - in production, you'd maintain a proper index
        // For now, we'll iterate through potential pet IDs
        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);

        for pet_id in 1..=pet_count {
            if let Some(grant) = env
                .storage()
                .instance()
                .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, user.clone())))
            {
                if grant.is_active {
                    // Check if not expired
                    if let Some(exp) = grant.expires_at {
                        if env.ledger().timestamp() < exp {
                            granted_pets.push_back(pet_id);
                        }
                    } else {
                        granted_pets.push_back(pet_id);
                    }
                }
            }
        }

        granted_pets
    }

    /// Get detailed access information for a specific grant
    pub fn get_access_details(env: Env, pet_id: u64, user: Address) -> Option<AccessGrant> {
        env.storage()
            .instance()
            .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, user)))
    }

    /// Check if an access grant has expired
    pub fn is_access_expired(env: Env, pet_id: u64, user: Address) -> bool {
        if let Some(grant) = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, user)))
        {
            if let Some(exp) = grant.expires_at {
                return env.ledger().timestamp() >= exp;
            }
            false
        } else {
            true // No grant means effectively expired
        }
    }

    /// Extend access expiration time
    pub fn extend_access(
        env: Env,
        pet_id: u64,
        grantee: Address,
        new_expires_at: Option<u64>,
    ) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let key = DataKey::AccessGrant((pet_id, grantee.clone()));
        if let Some(mut grant) = env.storage().instance().get::<DataKey, AccessGrant>(&key) {
            grant.expires_at = new_expires_at;
            env.storage().instance().set(&key, &grant);

            env.events().publish(
                (String::from_str(&env, "AccessExtended"), pet_id),
                (pet_id, grantee, new_expires_at),
            );
            true
        } else {
            false
        }
    }

    /// Update access level for existing grant
    pub fn update_access_level(
        env: Env,
        pet_id: u64,
        grantee: Address,
        new_level: AccessLevel,
    ) -> bool {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let key = DataKey::AccessGrant((pet_id, grantee.clone()));
        if let Some(mut grant) = env.storage().instance().get::<DataKey, AccessGrant>(&key) {
            grant.access_level = new_level.clone();
            env.storage().instance().set(&key, &grant);

            env.events().publish(
                (String::from_str(&env, "AccessLevelUpdated"), pet_id),
                (pet_id, grantee, new_level),
            );
            true
        } else {
            false
        }
    }

    /// Batch grant access to multiple users
    pub fn batch_grant_access(
        env: Env,
        pet_id: u64,
        grantees: Vec<Address>,
        access_level: AccessLevel,
        expires_at: Option<u64>,
    ) -> u64 {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let mut successful_grants = 0u64;

        for grantee in grantees.iter() {
            let now = env.ledger().timestamp();

            let grant = AccessGrant {
                pet_id,
                granter: pet.owner.clone(),
                grantee: grantee.clone(),
                access_level: access_level.clone(),
                granted_at: now,
                expires_at,
                is_active: true,
            };

            let storage = env.storage().instance();
            let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
            let is_new_grant = storage.get::<DataKey, AccessGrant>(&grant_key).is_none();

            storage.set(&grant_key, &grant);

            if is_new_grant {
                let grant_count: u64 = storage.get(&DataKey::AccessGrantCount(pet_id)).unwrap_or(0);
                let new_count = grant_count + 1;
                storage.set(&DataKey::AccessGrantCount(pet_id), &new_count);
                storage.set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

                let user_access_count: u64 = storage
                    .get(&DataKey::UserAccessCount(grantee.clone()))
                    .unwrap_or(0);
                storage.set(
                    &DataKey::UserAccessCount(grantee.clone()),
                    &(user_access_count + 1),
                );
            }

            successful_grants += 1;
        }

        env.events().publish(
            (String::from_str(&env, "BatchAccessGranted"), pet_id),
            (pet_id, successful_grants),
        );

        successful_grants
    }

    /// Get all active grants for a pet (for owner to review)
    pub fn get_all_active_grants(env: Env, pet_id: u64) -> Vec<AccessGrant> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");
        pet.owner.require_auth();

        let storage = env.storage().instance();
        let grant_count: u64 = storage.get(&DataKey::AccessGrantCount(pet_id)).unwrap_or(0);

        let mut grants = Vec::new(&env);
        let now = env.ledger().timestamp();

        for i in 1..=grant_count {
            if let Some(grantee) =
                storage.get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
            {
                if let Some(grant) =
                    storage.get::<DataKey, AccessGrant>(&DataKey::AccessGrant((pet_id, grantee)))
                {
                    if grant.is_active {
                        // Check if not expired
                        let is_valid = if let Some(exp) = grant.expires_at {
                            now < exp
                        } else {
                            true
                        };

                        if is_valid {
                            grants.push_back(grant);
                        }
                    }
                }
            }
        }

        grants
    }

    // ==================== MEDICAL RECORDS ====================

    pub fn add_medical_record(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        record_type: String,
        diagnosis: String,
        treatment: String,
        medications: Vec<Medication>,
    ) -> u64 {
        veterinarian.require_auth();

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        // Check if veterinarian has Full access (owner or granted Full access)
        let access_level = Self::check_access(env.clone(), pet_id, veterinarian.clone());

        if access_level != AccessLevel::Full {
            panic!("Full access required to add medical records");
        }

        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::MedicalRecordCount)
            .unwrap_or(0);
        let id = count + 1;
        let now = env.ledger().timestamp();

        let record = MedicalRecord {
            id,
            pet_id,
            veterinarian: veterinarian.clone(),
            record_type,
            diagnosis,
            treatment,
            medications,
            created_at: now,
            updated_at: now,
        };

        env.storage()
            .instance()
            .set(&DataKey::MedicalRecordCount, &id);
        env.storage()
            .instance()
            .set(&DataKey::MedicalRecord(id), &record);

        let pet_record_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let new_pet_record_count = pet_record_count + 1;
        env.storage().instance().set(
            &DataKey::PetMedicalRecordCount(pet_id),
            &new_pet_record_count,
        );
        env.storage().instance().set(
            &DataKey::PetMedicalRecordIndex((pet_id, new_pet_record_count)),
            &id,
        );

        env.events().publish(
            (String::from_str(&env, "MedicalRecordAdded"), pet_id),
            MedicalRecordAddedEvent {
                pet_id,
                updated_by: veterinarian,
                timestamp: now,
            },
        );

        id
    }

    pub fn update_medical_record(
        env: Env,
        record_id: u64,
        diagnosis: String,
        treatment: String,
        medications: Vec<Medication>,
    ) -> bool {
        if let Some(mut record) = env
            .storage()
            .instance()
            .get::<DataKey, MedicalRecord>(&DataKey::MedicalRecord(record_id))
        {
            record.veterinarian.require_auth();

            record.diagnosis = diagnosis;
            record.treatment = treatment;
            record.medications = medications;
            record.updated_at = env.ledger().timestamp();

            env.storage()
                .instance()
                .set(&DataKey::MedicalRecord(record_id), &record);
            true
        } else {
            false
        }
    }

    pub fn get_medical_record(
        env: Env,
        record_id: u64,
        requester: Address,
    ) -> Option<MedicalRecord> {
        requester.require_auth();

        if let Some(record) = env
            .storage()
            .instance()
            .get::<DataKey, MedicalRecord>(&DataKey::MedicalRecord(record_id))
        {
            // Check if requester has access to this pet's records
            let access_level = Self::check_access(env.clone(), record.pet_id, requester);

            if access_level == AccessLevel::None {
                panic!("Access denied");
            }

            Some(record)
        } else {
            None
        }
    }

    pub fn get_pet_medical_records(
        env: Env,
        pet_id: u64,
        requester: Address,
    ) -> Vec<MedicalRecord> {
        requester.require_auth();

        // Check if requester has access to this pet's records
        let access_level = Self::check_access(env.clone(), pet_id, requester);

        if access_level == AccessLevel::None {
            panic!("Access denied");
        }

        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetMedicalRecordCount(pet_id))
            .unwrap_or(0);
        let mut records = Vec::new(&env);
        for i in 1..=count {
            if let Some(rid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::PetMedicalRecordIndex((pet_id, i)))
            {
                if let Some(record) = env
                    .storage()
                    .instance()
                    .get::<DataKey, MedicalRecord>(&DataKey::MedicalRecord(rid))
                {
                    records.push_back(record);
                }
            }
        }
        records
    }

    // ==================== LAB RESULTS ====================

    pub fn add_lab_result(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        test_type: String,
        result_summary: String,
        medical_record_id: Option<u64>,
    ) -> u64 {
        veterinarian.require_auth();
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::LabResultCount)
            .unwrap_or(0);
        let id = count + 1;
        env.storage().instance().set(&DataKey::LabResultCount, &id);

        let result = LabResult {
            id,
            pet_id,
            veterinarian,
            test_type,
            result_summary,
            medical_record_id,
            created_at: env.ledger().timestamp(),
        };
        env.storage()
            .instance()
            .set(&DataKey::LabResult(id), &result);

        let p_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetLabResultCount(pet_id))
            .unwrap_or(0);
        let new_p = p_count + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetLabResultCount(pet_id), &new_p);
        env.storage()
            .instance()
            .set(&DataKey::PetLabResultIndex((pet_id, new_p)), &id);

        id
    }

    pub fn get_lab_result(env: Env, lab_result_id: u64) -> Option<LabResult> {
        env.storage()
            .instance()
            .get(&DataKey::LabResult(lab_result_id))
    }

    pub fn get_pet_lab_results(env: Env, pet_id: u64) -> Vec<LabResult> {
        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetLabResultCount(pet_id))
            .unwrap_or(0);
        let mut res = Vec::new(&env);
        for i in 1..=count {
            if let Some(lid) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::PetLabResultIndex((pet_id, i)))
            {
                if let Some(r) = Self::get_lab_result(env.clone(), lid) {
                    res.push_back(r);
                }
            }
        }
        res
    }

    // ==================== HELPER FUNCTIONS ====================

    fn get_encryption_key(env: &Env) -> Bytes {
        Bytes::from_array(env, &[0u8; 32])
    }

    fn get_owner_pet_count(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
    }

    fn require_admin(_env: &Env) {
        // Placeholder for admin check
        // In production, verify caller is admin
    }
}

// ==================== ENCRYPTION HELPERS ====================

fn encrypt_sensitive_data(env: &Env, data: &Bytes, _key: &Bytes) -> (Bytes, Bytes) {
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

// ==================== TESTS ====================

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_pet() {
        let env = Env::default();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Buddy");
        let species = String::from_str(&env, "Dog");
        let gender = String::from_str(&env, "Male");

        let pet_id = client.register_pet(&owner, &name, &species, &gender);
        assert_eq!(pet_id, 1);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.id, 1);
        assert_eq!(pet.active, true);
    }

    #[test]
    fn test_treatment_history() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "Cat"),
            &String::from_str(&env, "Female"),
        );

        client.add_treatment(
            &pet_id,
            &vet,
            &TreatmentType::Surgery,
            &String::from_str(&env, "Spay"),
            &String::from_str(&env, "Routine"),
            &1000,
            &20000,
            &TreatmentStatus::Completed,
            &TreatmentOutcome::Successful,
        );

        client.add_treatment(
            &pet_id,
            &vet,
            &TreatmentType::Checkup,
            &String::from_str(&env, "Annual"),
            &String::from_str(&env, "Healthy"),
            &2000,
            &5000,
            &TreatmentStatus::Completed,
            &TreatmentOutcome::Successful,
        );

        let all_history = client.get_treatment_history(&pet_id, &None, &None, &None);
        assert_eq!(all_history.len(), 2);

        let surgery_history =
            client.get_treatment_history(&pet_id, &Some(TreatmentType::Surgery), &None, &None);
        assert_eq!(surgery_history.len(), 1);
        assert_eq!(
            surgery_history.get(0).unwrap().treatment_type,
            TreatmentType::Surgery
        );
    }

    #[test]
    fn test_access_control_grant_and_check() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);
        let unauthorized_user = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        // Owner should have Full access
        let owner_access = client.check_access(&pet_id, &owner);
        assert_eq!(owner_access, AccessLevel::Full);

        // Unauthorized user should have None access
        let no_access = client.check_access(&pet_id, &unauthorized_user);
        assert_eq!(no_access, AccessLevel::None);

        // Grant Basic access to vet
        let success = client.grant_access(&pet_id, &vet, &AccessLevel::Basic, &None);
        assert!(success);

        // Vet should now have Basic access
        let vet_access = client.check_access(&pet_id, &vet);
        assert_eq!(vet_access, AccessLevel::Basic);

        // Check authorized users list
        let authorized = client.get_authorized_users(&pet_id);
        assert_eq!(authorized.len(), 1);
        assert_eq!(authorized.get(0).unwrap(), vet);
    }

    #[test]
    fn test_access_control_revoke() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Bella"),
            &String::from_str(&env, "Cat"),
            &String::from_str(&env, "Female"),
        );

        // Grant Full access
        client.grant_access(&pet_id, &vet, &AccessLevel::Full, &None);
        let access = client.check_access(&pet_id, &vet);
        assert_eq!(access, AccessLevel::Full);

        // Revoke access
        let revoked = client.revoke_access(&pet_id, &vet);
        assert!(revoked);

        // Access should now be None
        let no_access = client.check_access(&pet_id, &vet);
        assert_eq!(no_access, AccessLevel::None);
    }

    #[test]
    fn test_access_control_with_expiry() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let temp_vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Rocky"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        // Grant access with short expiry (1000 seconds from now)
        let current_time = env.ledger().timestamp();
        let expires_at = current_time + 1000;

        client.grant_access(&pet_id, &temp_vet, &AccessLevel::Basic, &Some(expires_at));

        // Should have access now
        let access = client.check_access(&pet_id, &temp_vet);
        assert_eq!(access, AccessLevel::Basic);

        // Check if expired (should be false)
        let expired = client.is_access_expired(&pet_id, &temp_vet);
        assert!(!expired);
    }

    #[test]
    fn test_access_level_hierarchy() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let basic_user = Address::generate(&env);
        let full_user = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Charlie"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        // Grant different levels
        client.grant_access(&pet_id, &basic_user, &AccessLevel::Basic, &None);
        client.grant_access(&pet_id, &full_user, &AccessLevel::Full, &None);

        // Check access levels
        let basic_access = client.check_access(&pet_id, &basic_user);
        let full_access = client.check_access(&pet_id, &full_user);

        assert_eq!(basic_access, AccessLevel::Basic);
        assert_eq!(full_access, AccessLevel::Full);

        // Both should be in authorized users
        let authorized = client.get_authorized_users(&pet_id);
        assert_eq!(authorized.len(), 2);
    }

    #[test]
    fn test_update_access_level() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let user = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Daisy"),
            &String::from_str(&env, "Cat"),
            &String::from_str(&env, "Female"),
        );

        // Grant Basic access initially
        client.grant_access(&pet_id, &user, &AccessLevel::Basic, &None);
        assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Basic);

        // Update to Full access
        let updated = client.update_access_level(&pet_id, &user, &AccessLevel::Full);
        assert!(updated);
        assert_eq!(client.check_access(&pet_id, &user), AccessLevel::Full);
    }

    #[test]
    fn test_extend_access() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let user = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "Cat"),
            &String::from_str(&env, "Female"),
        );

        let current_time = env.ledger().timestamp();
        let initial_expiry = current_time + 1000;

        // Grant access with expiry
        client.grant_access(&pet_id, &user, &AccessLevel::Basic, &Some(initial_expiry));

        // Extend expiry
        let new_expiry = current_time + 5000;
        let extended = client.extend_access(&pet_id, &user, &Some(new_expiry));
        assert!(extended);

        // Verify the extension
        let grant = client.get_access_details(&pet_id, &user).unwrap();
        assert_eq!(grant.expires_at, Some(new_expiry));
    }

    #[test]
    fn test_batch_grant_access() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        // Create vector of grantees
        let mut grantees = Vec::new(&env);
        grantees.push_back(user1.clone());
        grantees.push_back(user2.clone());
        grantees.push_back(user3.clone());

        // Batch grant
        let granted = client.batch_grant_access(&pet_id, &grantees, &AccessLevel::Basic, &None);
        assert_eq!(granted, 3);

        // Verify all have access
        assert_eq!(client.check_access(&pet_id, &user1), AccessLevel::Basic);
        assert_eq!(client.check_access(&pet_id, &user2), AccessLevel::Basic);
        assert_eq!(client.check_access(&pet_id, &user3), AccessLevel::Basic);

        // Check authorized users
        let authorized = client.get_authorized_users(&pet_id);
        assert_eq!(authorized.len(), 3);
    }

    #[test]
    fn test_get_all_active_grants() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet1 = Address::generate(&env);
        let vet2 = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        // Grant access to multiple vets
        client.grant_access(&pet_id, &vet1, &AccessLevel::Full, &None);
        client.grant_access(&pet_id, &vet2, &AccessLevel::Basic, &None);

        // Get all active grants
        let active_grants = client.get_all_active_grants(&pet_id);
        assert_eq!(active_grants.len(), 2);

        // Revoke one
        client.revoke_access(&pet_id, &vet1);

        // Should now have only one active grant
        let active_grants = client.get_all_active_grants(&pet_id);
        assert_eq!(active_grants.len(), 1);
    }

    #[test]
    fn test_get_granted_access_pets() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let vet = Address::generate(&env);

        // Create two pets with different owners
        let pet_id1 = client.register_pet(
            &owner1,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "Dog"),
            &String::from_str(&env, "Male"),
        );

        let pet_id2 = client.register_pet(
            &owner2,
            &String::from_str(&env, "Luna"),
            &String::from_str(&env, "Cat"),
            &String::from_str(&env, "Female"),
        );

        // Grant access to same vet for both pets
        client.grant_access(&pet_id1, &vet, &AccessLevel::Full, &None);
        client.grant_access(&pet_id2, &vet, &AccessLevel::Basic, &None);

        // Vet should have access to both pets
        let granted_pets = client.get_granted_access_pets(&vet);
        assert_eq!(granted_pets.len(), 2);
    }
}
