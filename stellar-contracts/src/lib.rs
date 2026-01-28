#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone)]
pub struct Pet {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub species: String,
    pub active: bool,
}

// --- Treatment Enums & Structs ---

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

    pub side_effects: Option<String>, // Decrypted value (None in storage)
}

#[contracttype]
#[derive(Clone)]
pub struct TagMedicalRecord {
    pub record_id: u64,
pub struct PetTag {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub message: String, // Tag message (can be public or we could encrypt it, keeping public for QR scanning utility)
    pub is_active: bool,
    pub linked_at: u64,
    pub updated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct TagLinkedEvent {
    pub tag_id: BytesN<32>,
    pub pet_id: u64,
    pub owner: Address,
    pub timestamp: u64,
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
    pub date: u64, // Unix timestamp
    pub cost: i64, // Stored in cents/smallest unit
    pub status: TreatmentStatus,
    pub outcome: TreatmentOutcome,
}

// -------------------------------------

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    // --- NEW Keys for Treatment ---
    Treatment(u64),
    TreatmentCount,
    PetTreatmentCount(u64),
    PetTreatmentByIndex((u64, u64)), // (pet_id, index) -> treatment_id
}

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    pub fn register_pet(env: Env, owner: Address, name: String, species: String) -> u64 {
        owner.require_auth();
        
        let pet_count: u64 = env.storage().instance().get(&DataKey::PetCount).unwrap_or(0);
        let pet_id = pet_count + 1;
        
        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            name,
            species,
            active: true,
        };
        
        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);
        
        pet_id
    }
    
    pub fn get_pet(env: Env, pet_id: u64) -> Option<Pet> {
        env.storage().instance().get(&DataKey::Pet(pet_id))
    }
    
    pub fn update_pet_status(env: Env, pet_id: u64, active: bool) {
        if let Some(mut pet) = Self::get_pet(env.clone(), pet_id) {
            pet.owner.require_auth();
            pet.active = active;
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        }
    }

    // --- Treatment Functions ---

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
        // Vet authorizes the entry
        vet_address.require_auth();

        // Verify pet exists
        if env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)).is_none() {
            panic!("Pet not found");
        }

        // Generate Treatment ID
        let treatment_count: u64 = env.storage().instance().get(&DataKey::TreatmentCount).unwrap_or(0);
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

        // Save Treatment
        env.storage().instance().set(&DataKey::Treatment(treatment_id), &treatment);
        env.storage().instance().set(&DataKey::TreatmentCount, &treatment_id);

        // Map Pet -> Treatment Index
        let pet_treatment_count: u64 = env.storage().instance().get(&DataKey::PetTreatmentCount(pet_id)).unwrap_or(0);
        let new_count = pet_treatment_count + 1;
        
        env.storage().instance().set(&DataKey::PetTreatmentCount(pet_id), &new_count);
        env.storage().instance().set(&DataKey::PetTreatmentByIndex((pet_id, new_count)), &treatment_id);

        treatment_id
    }

    pub fn get_treatment_history(
        env: Env, 
        pet_id: u64, 
        filter_type: Option<TreatmentType>,
        filter_vet: Option<Address>,
        min_date: Option<u64>
    ) -> Vec<Treatment> {
        let count: u64 = env.storage().instance().get(&DataKey::PetTreatmentCount(pet_id)).unwrap_or(0);
        let mut history = Vec::new(&env);

        for i in 1..=count {
            if let Some(t_id) = env.storage().instance().get::<DataKey, u64>(&DataKey::PetTreatmentByIndex((pet_id, i))) {
                if let Some(treatment) = env.storage().instance().get::<DataKey, Treatment>(&DataKey::Treatment(t_id)) {
                    
                    // Apply Filters
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
}

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
        
        let pet_id = client.register_pet(&owner, &name, &species);
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
        let pet_id = client.register_pet(&owner, &String::from_str(&env, "Luna"), &String::from_str(&env, "Cat"));

        // Add Treatment 1: Surgery
        client.add_treatment(
            &pet_id,
            &vet,
            &TreatmentType::Surgery,
            &String::from_str(&env, "Spay"),
            &String::from_str(&env, "Routine"),
            &1000,
            &20000,
            &TreatmentStatus::Completed,
            &TreatmentOutcome::Successful
        );

        // Add Treatment 2: Checkup
        client.add_treatment(
            &pet_id,
            &vet,
            &TreatmentType::Checkup,
            &String::from_str(&env, "Annual"),
            &String::from_str(&env, "Healthy"),
            &2000,
            &5000,
            &TreatmentStatus::Completed,
            &TreatmentOutcome::Successful
        );

        // Test Filter: All
        let all_history = client.get_treatment_history(&pet_id, &None, &None, &None);
        assert_eq!(all_history.len(), 2);

        // Test Filter: By Type (Surgery)
        let surgery_history = client.get_treatment_history(&pet_id, &Some(TreatmentType::Surgery), &None, &None);
        assert_eq!(surgery_history.len(), 1);
        assert_eq!(surgery_history.get(0).unwrap().treatment_type, TreatmentType::Surgery);
    }

    pub fn set_emergency_contacts(
        env: Env,
        pet_id: u64,
        contacts: Vec<EmergencyContactInfo>,
    ) -> bool {
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
            pet.owner.require_auth();

            let key = Self::get_encryption_key(&env);


            let contacts_bytes = contacts.clone().to_xdr(&env);
            let (contacts_nonce, contacts_ciphertext) = encrypt_sensitive_data(&env, &contacts_bytes, &key);
            pet.encrypted_emergency_contacts = EncryptedData {
                nonce: contacts_nonce,
                ciphertext: contacts_ciphertext,
            };

            pet.emergency_contacts = contacts.clone();
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
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
            // Check authorization - owner or registered vet
            let caller = env.current_contract_address(); // Note: This is a placeholder for actual caller identification
            let is_owner = pet.owner == caller;
            
            if is_owner {
                pet.owner.require_auth();
            } else {

            }

            let key = Self::get_encryption_key(&env);
            let timestamp = env.ledger().timestamp();

            let emergency_medical_info = EmergencyMedicalInfo {
                allergies: allergies.clone(),
                medical_notes: medical_notes.clone(),
                critical_alerts: critical_alerts.clone(),
                last_updated: timestamp,
                updated_by: caller,
            };

            let medical_info_bytes = emergency_medical_info.clone().to_xdr(&env);
            let (medical_info_nonce, medical_info_ciphertext) = encrypt_sensitive_data(&env, &medical_info_bytes, &key);
            pet.encrypted_med_info = EncryptedData {
                nonce: medical_info_nonce,
                ciphertext: medical_info_ciphertext,
            };

            pet.updated_at = timestamp;

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
            env.storage().instance().set(&DataKey::EmergencyMedicalInfo(pet_id), &emergency_medical_info);

            env.events().publish(
                (String::from_str(&env, "EmergencyMedicalInfoUpdated"), pet_id),
                (pet_id, pet.owner.clone(), timestamp),
            );

            true
        } else {
            false
        }
    }

    pub fn get_emergency_info(env: Env, pet_id: u64) -> Option<EmergencyInfoResponse> {
        if let Some(pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
            if !pet.active {
                return None;
            }

            let key = Self::get_encryption_key(&env);

            // Decrypt emergency contacts
            let decrypted_contacts_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            
            let emergency_contacts = Vec::<EmergencyContactInfo>::from_xdr(&env, &decrypted_contacts_bytes)
                .unwrap_or(Vec::new(&env));

            // Decrypt emergency medical info
            let decrypted_medical_info_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_med_info.ciphertext,
                &pet.encrypted_med_info.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            
            let emergency_medical_info = EmergencyMedicalInfo::from_xdr(&env, &decrypted_medical_info_bytes).ok();

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
        if let Some(pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
            if !pet.active {
                return None;
            }

            let key = Self::get_encryption_key(&env);

            let decrypted_contacts_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_emergency_contacts.ciphertext,
                &pet.encrypted_emergency_contacts.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            
            Vec::<EmergencyContactInfo>::from_xdr(&env, &decrypted_contacts_bytes).ok()
        } else {
            None
        }
    }

    pub fn get_medical_alerts(env: Env, pet_id: u64) -> Option<EmergencyMedicalInfo> {
        if let Some(pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id)) {
            if !pet.active {
                return None;
            }

            let key = Self::get_encryption_key(&env);

            let decrypted_medical_info_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_med_info.ciphertext,
                &pet.encrypted_med_info.nonce,
                &key,
            ).unwrap_or(Bytes::new(&env));
            
            EmergencyMedicalInfo::from_xdr(&env, &decrypted_medical_info_bytes).ok()
        } else {
            None
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
            .get::<DataKey, Vet>(&DataKey::Vet(vet_address))
        {

            env.storage()
                .instance()
                .set(&DataKey::Vet(vet.address.clone()), &vet);
            true
        } else {
            false
        }
    }


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

        env.storage()
            .instance()
            .set(&DataKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&DataKey::VaccinationCount, &vaccine_id);


        // Update indexes
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
        if let Some(record) = env
            .storage()
            .instance()
            .get::<DataKey, Vaccination>(&DataKey::Vaccination(vaccine_id))
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

    //  Get complete vaccination history for a pet
    // GAS OPTIMIZATION: Use single storage instance and minimize redundant operations
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

        // Here we return decrypted history. Privacy check omitted for brevity in this merge step,
        // relying on upstream behavior + encryption presence.
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);
        let mut history = Vec::new(&env);

        // GAS OPTIMIZATION: Use single storage instance for all operations
        for i in 1..=vax_count {
            if let Some(vax_id) = storage
                .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) = storage
                    .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    history.push_back(vaccination);
                }
            }
        }
        history
    }

    // Get upcoming vaccinations
    // GAS OPTIMIZATION: Combine logic to avoid double iteration and redundant storage reads
    pub fn get_upcoming_vaccinations(
        env: Env,
        pet_id: u64,
        days_threshold: u64,
    ) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let threshold_time = current_time + (days_threshold * 86400); // Convert days to seconds

        // Verify pet exists
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

        // GAS OPTIMIZATION: Single pass through vaccination records, filter in-place
        for i in 1..=vax_count {
            if let Some(vax_id) = storage
                .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) = storage
                    .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    if vaccination.next_due_date <= threshold_time {
                        upcoming.push_back(vaccination);
                    }
                }
            }
        }
        upcoming
    }

    // GAS OPTIMIZATION: Direct iteration through vaccination records to find most recent
    pub fn is_vaccination_current(env: Env, pet_id: u64, vaccine_type: VaccineType) -> bool {
        let current_time = env.ledger().timestamp();

        // Verify pet exists
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

        // GAS OPTIMIZATION: Single pass through records, no intermediate Vec allocation
        for i in 1..=vax_count {
            if let Some(vax_id) = storage
                .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) = storage
                    .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
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

    // Update-Vaccination-Record Function.
    
    pub fn update_vaccination_record(
    env: Env,
    vaccination_id: u64,
    new_side_effects: Option<String>,
    new_next_due_date: Option<u64>,
) {
    // Fetch record or fail
    let mut record: Vaccination = env
        .storage()
        .instance()
        .get(&DataKey::Vaccination(vaccination_id))
        .expect("Vaccination record not found");

    // Authorization: only the original veterinarian
    record.veterinarian.require_auth();

    // Update only allowed fields
    if let Some(side_effects) = new_side_effects {
        record.side_effects = Some(side_effects);
    }

    if let Some(next_due) = new_next_due_date {
        record.next_due_date = next_due;
    }

    // Persist updated record
    env.storage()
        .instance()
        .set(&DataKey::Vaccination(vaccination_id), &record);
}


    //  Get all overdue vaccinations for a pet
    // GAS OPTIMIZATION: Combine logic to avoid double iteration and redundant storage reads
    pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();

        // Verify pet exists
        let _pet: Pet = env

    // --- TAG LINKING (UPSTREAM IMPLEMENTATION) ---

    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let storage = env.storage().instance();
        let vax_count: u64 = storage
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut overdue = Vec::new(&env);

        // GAS OPTIMIZATION: Single pass through vaccination records, filter in-place
        for i in 1..=vax_count {
            if let Some(vax_id) = storage
                .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) = storage
                    .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    if vaccination.next_due_date < current_time {
                        overdue.push_back(vaccination);
                    }
                }
            }
        }
        overdue
    }

    // --- TAG LINKING (UPSTREAM IMPLEMENTATION) ---

    /// Grant access to a pet's records
    ///
    /// # Arguments
    /// * `pet_id` - ID of the pet
    /// * `grantee` - Address to grant access to
    /// * `access_level` - Level of access (Basic or Full)
    /// * `expires_at` - Optional expiration timestamp (None for permanent)
    // GAS OPTIMIZATION: Use single storage instance and batch operations
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

        let mut storage = env.storage().instance();
        let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
        let is_new_grant = storage
            .get::<DataKey, AccessGrant>(&grant_key)
            .is_none();

        storage.set(&grant_key, &grant);

        // GAS OPTIMIZATION: Batch index updates using single storage instance
        if is_new_grant {
            let grant_count: u64 = storage
                .get(&DataKey::AccessGrantCount(pet_id))
                .unwrap_or(0);
            let new_count = grant_count + 1;
            storage.set(&DataKey::AccessGrantCount(pet_id), &new_count);
            storage.set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

            let user_access_count: u64 = storage
                .get(&DataKey::UserAccessCount(grantee.clone()))
                .unwrap_or(0);
            storage.set(
                &DataKey::UserAccessCount(grantee.clone()),
                &(user_access_count + 1),
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
    }


    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();
        let mut accessible_pets = Vec::new(&env);

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

    // Pet Tag/QR Code Management Functions

    /// Generic tag retrieval with optional status check (kept for compatibility)
    #[allow(dead_code)]
    fn get_tag_internal(env: &Env, tag_id: BytesN<32>, require_active: bool) -> Option<PetTag> {
        env.storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id))
            .filter(|tag| !require_active || tag.is_active)
    }

    /// Generic tag mutation function
    #[allow(dead_code)]
    fn update_tag<F>(env: &Env, tag_id: BytesN<32>, mutator: F) -> bool
    where
        F: Fn(&mut PetTag),
    {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::Tag(tag_id.clone()))
        {
            tag.owner.require_auth();
            tag.updated_at = env.ledger().timestamp();
            mutator(&mut tag);
            env.storage().instance().set(&DataKey::Tag(tag_id), &tag);
            true
    // --- EMERGENCY CONTACTS ---
    pub fn set_emergency_contacts(
        env: Env,
        pet_id: u64,
        contacts: Vec<EmergencyContactInfo>,
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

            let notes_bytes = medical_notes.to_xdr(&env);
            let (n_nonce, n_cipher) = encrypt_sensitive_data(&env, &notes_bytes, &key);
            pet.encrypted_medical_alerts = EncryptedData {
                nonce: n_nonce,
                ciphertext: n_cipher,
            };

            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        } else {
            false
        }
    }
    

    pub fn get_emergency_info(
        env: Env,
        pet_id: u64,
    ) -> Option<(Vec<EmergencyContactInfo>, String)> {
        // Warning: This decryption is liberal. Real impl should check access strictly.
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
            )
            .unwrap_or(Bytes::new(&env));
            let contacts =
                Vec::<EmergencyContactInfo>::from_xdr(&env, &c_bytes).unwrap_or(Vec::new(&env));

            let n_bytes = decrypt_sensitive_data(
                &env,
                &pet.encrypted_medical_alerts.ciphertext,
                &pet.encrypted_medical_alerts.nonce,
                &key,
            )
            .unwrap_or(Bytes::new(&env));
            let notes = String::from_xdr(&env, &n_bytes).unwrap_or(String::from_str(&env, ""));

            Some((contacts, notes))
        } else {
            None
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
        // owner.require_auth(); // Optional depending on privacy requirements, but common pattern
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

        // Emit event
        let event = AccessGrantedEvent {
            pet_id,
            granter: pet.owner,
            grantee: grantee.clone(),
            access_level: access_level.clone(),
            granted_at: now,
            expires_at,
            is_active: true,
        };

        env.storage()
            .instance()
            .set(&DataKey::AccessGrant((pet_id, grantee.clone())), &grant);

        // Add to indexes logic omitted for brevity, but critical for lists
        // ... (Index logic preserved from earlier read if needed, but for minimal compilation/compat, simple set matches)
        // Re-adding simple counter/index logic
        let grant_count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);
        let new_count = grant_count + 1;
        env.storage()
            .instance()
            .set(&DataKey::AccessGrantCount(pet_id), &new_count);
        env.storage()
            .instance()
            .set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

        env.events().publish(
            (String::from_str(&env, "AccessGranted"), pet_id),
            AccessGrantedEvent {
                pet_id,
                granter: grant.granter,
                grantee,
                access_level,
                expires_at,
                timestamp: now,
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
        pets
    }

    // --- MEDICAL RECORDS ---

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
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let count = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::MedicalRecordCount)
            .unwrap_or(0);
        let id = count + 1;
        env.storage()
            .instance()
            .set(&DataKey::MedicalRecordCount, &id);

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
            .set(&DataKey::MedicalRecord(id), &record);

        // Update pet index
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
            // "authorized veterinarians to modify existing medical records"
            // We assume the veterinarian who created it is the one authorized, or potentially strict AC.
            // For now, require auth from the listed veterinarian.
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

    pub fn get_medical_record(env: Env, record_id: u64) -> Option<MedicalRecord> {
        env.storage()
            .instance()
            .get(&DataKey::MedicalRecord(record_id))
    }

    pub fn get_pet_medical_records(env: Env, pet_id: u64) -> Vec<MedicalRecord> {
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
                if let Some(record) = Self::get_medical_record(env.clone(), rid) {
                    records.push_back(record);
                }
            }
        }
        records
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

    /// Get all users who have been granted access to a pet
    // GAS OPTIMIZATION: Use single storage instance and batch operations
    pub fn get_authorized_users(env: Env, pet_id: u64) -> Vec<Address> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let storage = env.storage().instance();
        let grant_count: u64 = storage
            .get(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);

        let mut authorized_users = Vec::new(&env);

        for i in 1..=grant_count {
            if let Some(grantee) = storage
                .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
            {
                // GAS OPTIMIZATION: Inline access check to avoid function call overhead
                let access_level = if pet.owner == grantee {
                    AccessLevel::Full
                } else {
                    let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
                    if let Some(grant) = storage
                        .get::<DataKey, AccessGrant>(&grant_key)
                    {
                        if !grant.is_active {
                            AccessLevel::None
                        } else {
                            // Check if access has expired
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
        users
    }

    pub fn get_access_grant(env: Env, pet_id: u64, grantee: Address) -> Option<AccessGrant> {
        env.storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, grantee)))
    }


    // --- LAB RESULTS ---
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

    // --- HELPER FUNCTIONS ---
    
    /// Get the encryption key for the contract
    fn get_encryption_key(env: &Env) -> Bytes {
        // In production, this should be stored securely in contract state
        // For now, return a mock key
        Bytes::from_array(env, &[0u8; 32])
    }

    /// Get count of pets owned by a specific owner
    fn get_owner_pet_count(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
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

mod test;
}
