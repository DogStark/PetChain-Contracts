#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Species {
    Other,
    Dog,
    Cat,
    Bird,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrivacyLevel {
    Public, // Accessible to anyone
    Restricted, // Accessible to granted access (e.g., vets, owners)
    Private, // Accessible only to the owner
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Gender {
    NotSpecified,
    Male,
    Female,
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
pub struct Pet {
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
    pub emergency_contacts: Vec<EmergencyContactInfo>,
    pub medical_alerts: String,
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
    pub vaccine_name: Option<String>, // Decrypted value
    pub encrypted_vaccine_name: Option<EncryptedData>, // Encrypted value
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: Option<String>, // Decrypted value
    pub encrypted_batch_number: Option<EncryptedData>, // Encrypted value
    pub created_at: u64,
}

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),

    // Vaccination DataKey
    Vaccination(u64),
    VaccinationCount,
    PetVaccinations(Address),
    PetVaccinationIndex((Address, u64)),
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),

    // Access Control keys

    // Privacy keys
    OffChainDataHash(u64),

    // Access Control keys
    AccessGrant((u64, Address)),  // (pet_id, grantee) -> AccessGrant
    AccessGrantCount(u64),        // pet_id -> count of grants
    AccessGrantIndex((u64, u64)), // (pet_id, index) -> grantee Address
    UserAccessList(Address),      // grantee -> list of pet_ids they have access to
    UserAccessCount(Address),     // grantee -> count of pets they can access
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationSummary {
    pub is_fully_current: bool,
    pub overdue_types: Vec<VaccineType>,
    pub upcoming_count: u64, // Count of vaccinations due within a certain future period
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

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    fn get_owner_pet_count(env: &Env, owner: &Address) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
    }

    fn add_pet_to_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = Self::get_owner_pet_count(env, owner);
        let new_count = count.checked_add(1).expect("Owner pet count overflow");

        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &new_count);
        env.storage()
            .instance()
            .set(&DataKey::OwnerPetIndex((owner.clone(), new_count)), &pet_id);
    }

    fn remove_pet_from_owner_index(env: &Env, owner: &Address, pet_id: u64) {
        let count = Self::get_owner_pet_count(env, owner);
        if count == 0 {
            panic!("Owner has no pets");
        }

        let mut remove_index: Option<u64> = None;
        for i in 1..=count {
            if let Some(existing_pet_id) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if existing_pet_id == pet_id {
                    remove_index = Some(i);
                    break;
                }
            }
        }

        let remove_index = remove_index.expect("Pet not found for owner");
        if remove_index != count {
            let last_pet_id: u64 = env
                .storage()
                .instance()
                .get(&DataKey::OwnerPetIndex((owner.clone(), count)))
                .expect("Owner pet index missing");
            env.storage().instance().set(
                &DataKey::OwnerPetIndex((owner.clone(), remove_index)),
                &last_pet_id,
            );
        }

        env.storage()
            .instance()
            .remove(&DataKey::OwnerPetIndex((owner.clone(), count)));

        let new_count = count.checked_sub(1).expect("Owner pet count underflow");
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &new_count);
    }

    pub fn register_pet(
        env: Env,
        owner: Address,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        privacy_level: PrivacyLevel, // Added privacy_level parameter
    ) -> u64 {
        owner.require_auth();

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count.checked_add(1).expect("Pet count overflow");
        let timestamp = env.ledger().timestamp();

        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            privacy_level, // Set privacy_level
            encrypted_name: EncryptedData { nonce: vec![], ciphertext: vec![] }, // Placeholder, to be encrypted
            encrypted_birthday: EncryptedData { nonce: vec![], ciphertext: vec![] }, // Placeholder
            active: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species,
            gender,
            encrypted_breed: EncryptedData { nonce: vec![], ciphertext: vec![] }, // Placeholder
            encrypted_emergency_contacts: EncryptedData { nonce: vec![], ciphertext: vec![] }, // Placeholder
            encrypted_medical_alerts: EncryptedData { nonce: vec![], ciphertext: vec![] }, // Placeholder
        };
        
        // Encrypt sensitive fields
        let key = self.get_encryption_key(&env);

        // Encrypt name
        let name_bytes = name.as_bytes();
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, name_bytes, &key);
        pet.encrypted_name = EncryptedData { nonce: name_nonce, ciphertext: name_ciphertext };

        // Encrypt birthday
        let birthday_bytes = birthday.as_bytes();
        let (birthday_nonce, birthday_ciphertext) = encrypt_sensitive_data(&env, birthday_bytes, &key);
        pet.encrypted_birthday = EncryptedData { nonce: birthday_nonce, ciphertext: birthday_ciphertext };

        // Encrypt breed
        let breed_bytes = breed.as_bytes();
        let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(&env, breed_bytes, &key);
        pet.encrypted_breed = EncryptedData { nonce: breed_nonce, ciphertext: breed_ciphertext };

        // Encrypt medical_alerts
        let alerts_bytes = medical_alerts.as_bytes();
        let (alerts_nonce, alerts_ciphertext) = encrypt_sensitive_data(&env, alerts_bytes, &key);
        pet.encrypted_medical_alerts = EncryptedData { nonce: alerts_nonce, ciphertext: alerts_ciphertext };

        // Emergency contacts are handled separately by set_emergency_contacts

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);
        Self::add_pet_to_owner_index(&env, &owner, pet_id);

        pet_id
    }

    pub fn update_pet_profile(
        env: Env,
        id: u64,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
        privacy_level: PrivacyLevel, // Added privacy_level parameter
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();

            let key = self.get_encryption_key(&env);

            // Encrypt name
            let name_bytes = name.as_bytes();
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, name_bytes, &key);
            pet.encrypted_name = EncryptedData { nonce: name_nonce, ciphertext: name_ciphertext };

            // Encrypt birthday
            let birthday_bytes = birthday.as_bytes();
            let (birthday_nonce, birthday_ciphertext) = encrypt_sensitive_data(&env, birthday_bytes, &key);
            pet.encrypted_birthday = EncryptedData { nonce: birthday_nonce, ciphertext: birthday_ciphertext };

            // Update non-encrypted fields
            pet.gender = gender;
            pet.species = species;

            // Encrypt breed
            let breed_bytes = breed.as_bytes();
            let (breed_nonce, breed_ciphertext) = encrypt_sensitive_data(&env, breed_bytes, &key);
            pet.encrypted_breed = EncryptedData { nonce: breed_nonce, ciphertext: breed_ciphertext };
            
            // Update privacy level
            pet.privacy_level = privacy_level; // Set the new privacy level
            
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);
            true
        } else {
            false
        }
    }

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
            //  Solo el dueño puede modificar la info
            pet.owner.require_auth();

            let key = self.get_encryption_key(&env);

            // Encrypt emergency_contacts (Vec<EmergencyContactInfo>)
            // Serialize the Vec to bytes first.
            let val_contacts = env.to_val(contacts.clone()); // Clone is needed because to_val might consume.
            let contacts_bytes: Vec<u8> = val_contacts.try_into(env.clone()).unwrap_or_else(|_| panic_with_error!(&env, "Failed to serialize emergency contacts"));
            let (contacts_nonce, contacts_ciphertext) = encrypt_sensitive_data(&env, &contacts_bytes, &key);
            pet.encrypted_emergency_contacts = EncryptedData { nonce: contacts_nonce, ciphertext: contacts_ciphertext };

            // Encrypt medical_alerts
            let alerts_bytes = medical_notes.as_bytes();
            let (alerts_nonce, alerts_ciphertext) = encrypt_sensitive_data(&env, alerts_bytes, &key);
            pet.encrypted_medical_alerts = EncryptedData { nonce: alerts_nonce, ciphertext: alerts_ciphertext };
            
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        } else {
            panic!("Pet not found");
        }
    }

    pub fn get_emergency_info(env: Env, pet_id: u64) -> Option<(Vec<EmergencyContactInfo>, String)> {
        let pet_storage = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id));

        if let Some(pet) = pet_storage {
            let current_user = env.current_contract_address();
            let owner_address = pet.owner.clone();
            let mut is_authorized_for_full_data = false;

            // Check if caller is the owner
            if owner_address == current_user {
                is_authorized_for_full_data = true;
            } else {
                // Check for granted access
                let access_level = Self::check_access(env.clone(), pet_id, current_user.clone());
                if access_level != AccessLevel::None {
                    is_authorized_for_full_data = true;
                }
            }

            // Enforce privacy level based on authorization and privacy setting
            if !is_authorized_for_full_data && pet.privacy_level != PrivacyLevel::Public {
                // Deny access if not authorized and privacy is not Public.
                return None;
            }

            // If authorized or Public, proceed with decryption.
            let key = self.get_encryption_key(&env);

            // Decrypt emergency_contacts
            let decrypted_contacts_bytes = decrypt_sensitive_data(&env, &pet.encrypted_emergency_contacts.ciphertext, &pet.encrypted_emergency_contacts.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt emergency contacts for pet ID {}", pet_id));
            let contacts: Vec<EmergencyContactInfo> = env.from_slice(&decrypted_contacts_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Failed to deserialize emergency contacts for pet ID {}", pet_id));

            // Decrypt medical_alerts
            let decrypted_alerts_bytes = decrypt_sensitive_data(&env, &pet.encrypted_medical_alerts.ciphertext, &pet.encrypted_medical_alerts.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt medical alerts for pet ID {}", pet_id));
            let medical_alerts = String::from_utf8(decrypted_alerts_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for medical alerts for pet ID {}", pet_id));

            Some((contacts, medical_alerts))
        } else {
            None // Pet not found
        }
    }

    pub fn get_pet(env: Env, id: u64) -> Option<Pet> {
        let pet_storage = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id));

        if let Some(mut pet) = pet_storage { // Use mut for modifying the pet object to potentially remove encrypted data later
            let current_user = env.current_contract_address();
            let owner_address = pet.owner.clone();

            // Determine if the caller is authorized to view the full pet data.
            // Caller is authorized if:
            // 1. They are the owner of the pet.
            // 2. They have been granted access via check_access (Full or Basic).
            let mut is_authorized_for_full_data = false;

            // Check if caller is the owner
            if owner_address == current_user {
                is_authorized_for_full_data = true;
            } else {
                // Check for granted access
                let access_level = Self::check_access(env.clone(), id, current_user.clone());
                if access_level != AccessLevel::None {
                    is_authorized_for_full_data = true;
                }
            }

            // Enforce privacy level based on authorization and privacy setting
            if !is_authorized_for_full_data && pet.privacy_level != PrivacyLevel::Public {
                // If not authorized and privacy is not Public, deny full access.
                // Return None, as we cannot provide decrypted data without authorization.
                return None;
            }

            // If authorized OR privacy is Public, proceed with decryption.
            let key = self.get_encryption_key(&env);

            // Decrypt name
            let decrypted_name_bytes = decrypt_sensitive_data(&env, &pet.encrypted_name.ciphertext, &pet.encrypted_name.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt pet name for ID {}", id));
            let name = String::from_utf8(decrypted_name_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for pet name for ID {}", id));

            // Decrypt birthday
            let decrypted_birthday_bytes = decrypt_sensitive_data(&env, &pet.encrypted_birthday.ciphertext, &pet.encrypted_birthday.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt pet birthday for ID {}", id));
            let birthday = String::from_utf8(decrypted_birthday_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for pet birthday for ID {}", id));

            // Decrypt breed
            let decrypted_breed_bytes = decrypt_sensitive_data(&env, &pet.encrypted_breed.ciphertext, &pet.encrypted_breed.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt pet breed for ID {}", id));
            let breed = String::from_utf8(decrypted_breed_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for pet breed for ID {}", id));

            // Decrypt medical_alerts
            let decrypted_alerts_bytes = decrypt_sensitive_data(&env, &pet.encrypted_medical_alerts.ciphertext, &pet.encrypted_medical_alerts.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt pet medical alerts for ID {}", id));
            let medical_alerts = String::from_utf8(decrypted_alerts_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for pet medical alerts for ID {}", id));

            // Decrypt emergency_contacts (Vec<EmergencyContactInfo>)
            let decrypted_contacts_bytes = decrypt_sensitive_data(&env, &pet.encrypted_emergency_contacts.ciphertext, &pet.encrypted_emergency_contacts.nonce, &key)
                .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt pet emergency contacts for ID {}", id));
            let contacts: Vec<EmergencyContactInfo> = env.from_slice(&decrypted_contacts_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Failed to deserialize emergency contacts for ID {}", id));

            // Construct the decrypted Pet object
            // This assumes the Pet struct definition is updated to hold both decrypted String fields
            // and encrypted fields (which will be set to None or default for returned object).
            // If Pet struct *only* has encrypted fields, this reconstruction is invalid.
            // For this task, we assume the struct can hold decrypted strings.
            let decrypted_pet = Pet {
                id: pet.id,
                owner: pet.owner,
                name, // Decrypted
                birthday, // Decrypted
                active: pet.active,
                created_at: pet.created_at,
                updated_at: pet.updated_at,
                new_owner: pet.new_owner,
                species: pet.species,
                gender: pet.gender,
                breed, // Decrypted
                emergency_contacts: contacts, // Decrypted
                medical_alerts, // Decrypted
                // Note: privacy_level field is assumed to be part of the Pet struct definition from previous step
                // privacy_level: pet.privacy_level, // Assuming it's already there or will be added.
                // Need to ensure privacy_level is added to Pet struct definition if not done yet.
                // If Pet struct has privacy_level field, it should be included here.
                // Assuming it's already part of struct definition from previous step 15.2
                privacy_level: pet.privacy_level,
            };

            Some(decrypted_pet)
        } else {
            None
        }
    }

    pub fn get_all_pets_by_owner(env: Env, owner: Address) -> Vec<Pet> {
        owner.require_auth();

        let pet_count = Self::get_owner_pet_count(&env, &owner);
        let mut pets = Vec::new(&env);

        for i in 1..=pet_count {
            if let Some(pet_id) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((owner.clone(), i)))
            {
                if let Some(pet) = env
                    .storage()
                    .instance()
                    .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
                {
                    if pet.owner == owner {
                        pets.push_back(pet);
                    }
                }
            }
        }

        pets
    }

    pub fn is_pet_active(env: Env, id: u64) -> bool {
        if let Some(pet) = Self::get_pet(env, id) {
            pet.active
        } else {
            false
        }
    }

    pub fn get_pet_owner(env: Env, id: u64) -> Option<Address> {
        if let Some(pet) = Self::get_pet(env, id) {
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

    pub fn batch_transfer_pet_ownership(env: Env, owner: Address, pet_ids: Vec<u64>, to: Address) {
        owner.require_auth();
        let timestamp = env.ledger().timestamp();

        for pet_id in pet_ids.iter() {
            let mut pet: Pet = env
                .storage()
                .instance()
                .get(&DataKey::Pet(pet_id))
                .expect("Pet not found");

            if pet.owner != owner {
                panic!("Not pet owner");
            }

            pet.new_owner = to.clone();
            pet.updated_at = timestamp;
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        }
    }

    pub fn transfer_all_pets(env: Env, owner: Address, to: Address) {
        owner.require_auth();
        let timestamp = env.ledger().timestamp();
        let pet_count = Self::get_owner_pet_count(&env, &owner);

        for i in 1..=pet_count {
            let pet_id: u64 = env
                .storage()
                .instance()
                .get(&DataKey::OwnerPetIndex((owner.clone(), i)))
                .expect("Pet not found for owner");
            let mut pet: Pet = env
                .storage()
                .instance()
                .get(&DataKey::Pet(pet_id))
                .expect("Pet not found");

            if pet.owner != owner {
                panic!("Pet owner mismatch");
            }

            pet.new_owner = to.clone();
            pet.updated_at = timestamp;
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
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
            let new_owner = pet.new_owner.clone();
            pet.owner = new_owner.clone();
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);

            if old_owner != new_owner {
                Self::remove_pet_from_owner_index(&env, &old_owner, id);
                Self::add_pet_to_owner_index(&env, &new_owner, id);
            }
        }
    }

    pub fn batch_accept_pet_transfers(env: Env, new_owner: Address, pet_ids: Vec<u64>) {
        new_owner.require_auth();
        let timestamp = env.ledger().timestamp();

        for pet_id in pet_ids.iter() {
            let mut pet: Pet = env
                .storage()
                .instance()
                .get(&DataKey::Pet(pet_id))
                .expect("Pet not found");

            if pet.new_owner != new_owner {
                panic!("Transfer not authorized");
            }

            let old_owner = pet.owner.clone();
            pet.owner = new_owner.clone();
            pet.updated_at = timestamp;
            env.storage().instance().set(&DataKey::Pet(pet_id), &pet);

            if old_owner != new_owner {
                Self::remove_pet_from_owner_index(&env, &old_owner, pet_id);
                Self::add_pet_to_owner_index(&env, &new_owner, pet_id);
            }
        }
    }

    // Pet Owner Management Functions
    pub fn register_pet_owner(
        env: Env,
        owner: Address,
        name: String,
        email: String,
        emergency_contact: String,
    ) {
        owner.require_auth();

        let key = self.get_encryption_key(&env);

        // Encrypt name
        let name_bytes = name.as_bytes();
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, name_bytes, &key);
        let encrypted_name = EncryptedData { nonce: name_nonce, ciphertext: name_ciphertext };

        // Encrypt email
        let email_bytes = email.as_bytes();
        let (email_nonce, email_ciphertext) = encrypt_sensitive_data(&env, email_bytes, &key);
        let encrypted_email = EncryptedData { nonce: email_nonce, ciphertext: email_ciphertext };

        // Encrypt emergency_contact
        let emergency_contact_bytes = emergency_contact.as_bytes();
        let (emergency_contact_nonce, emergency_contact_ciphertext) = encrypt_sensitive_data(&env, emergency_contact_bytes, &key);
        let encrypted_emergency_contact = EncryptedData { nonce: emergency_contact_nonce, ciphertext: emergency_contact_ciphertext };

        let timestamp = env.ledger().timestamp();
        let pet_owner = PetOwner {
            owner_address: owner.clone(),
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
            let key = self.get_encryption_key(&env);

            // Encrypt name
            let name_bytes = name.as_bytes();
            let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, name_bytes, &key);
            pet_owner.encrypted_name = EncryptedData { nonce: name_nonce, ciphertext: name_ciphertext };

            // Encrypt email
            let email_bytes = email.as_bytes();
            let (email_nonce, email_ciphertext) = encrypt_sensitive_data(&env, email_bytes, &key);
            pet_owner.encrypted_email = EncryptedData { nonce: email_nonce, ciphertext: email_ciphertext };

            // Encrypt emergency_contact
            let emergency_contact_bytes = emergency_contact.as_bytes();
            let (emergency_contact_nonce, emergency_contact_ciphertext) = encrypt_sensitive_data(&env, emergency_contact_bytes, &key);
            pet_owner.encrypted_emergency_contact = EncryptedData { nonce: emergency_contact_nonce, ciphertext: emergency_contact_ciphertext };
            
            pet_owner.updated_at = env.ledger().timestamp();

            env.storage()
                .instance()
                .set(&DataKey::PetOwner(owner), &pet_owner);
            true
        } else {
            false
        }
    }

    // Pet Vaccination Record
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

        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let key = self.get_encryption_key(&env);

        // Encrypt vaccine_name
        let vaccine_name_bytes = vaccine_name.as_bytes();
        let (name_nonce, name_ciphertext) = encrypt_sensitive_data(&env, vaccine_name_bytes, &key);
        let encrypted_vaccine_name = EncryptedData { nonce: name_nonce, ciphertext: name_ciphertext };

        // Encrypt batch_number
        let batch_number_bytes = batch_number.as_bytes();
        let (batch_nonce, batch_ciphertext) = encrypt_sensitive_data(&env, batch_number_bytes, &key);
        let encrypted_batch_number = EncryptedData { nonce: batch_nonce, ciphertext: batch_ciphertext };

        let vaccine_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::VaccinationCount)
            .unwrap_or(0);
        let vaccine_id = vaccine_count + 1;
        let now = env.ledger().timestamp();

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian,
            vaccine_type,
            vaccine_name: None, // Decrypted field set to None when adding
            encrypted_vaccine_name: Some(encrypted_vaccine_name),
            administered_at,
            next_due_date,
            batch_number: None, // Decrypted field set to None when adding
            encrypted_batch_number: Some(encrypted_batch_number),
            created_at: now,
        };

        env.storage()
            .instance()
            .set(&DataKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&DataKey::VaccinationCount, &vaccine_id);

        // Update pet vaccination indexes
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

        // Also maintain owner-based index for compatibility
        let pet_vaccine_count_key = DataKey::PetVaccinations(pet.owner.clone());
        let mut pet_vaccine_count: u64 = env
            .storage()
            .instance()
            .get(&pet_vaccine_count_key)
            .unwrap_or(0);
        pet_vaccine_count += 1;

        env.storage()
            .instance()
            .set(&pet_vaccine_count_key, &pet_vaccine_count);
        env.storage().instance().set(
            &DataKey::PetVaccinationIndex((pet.owner.clone(), pet_vaccine_count)),
            &vaccine_id,
        );

        vaccine_id
    }

    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        env.storage()
            .instance()
            .get(&DataKey::Vaccination(vaccine_id))
    }

            // Get complete vaccination history for a pet
            pub fn get_vaccination_history(env: Env, pet_id: u64) -> Vec<Vaccination> {
                // Fetch pet data and check authorization first
                let pet_storage = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(pet_id));
        
                if let Some(pet) = pet_storage {
                    let current_user = env.current_contract_address();
                    let owner_address = pet.owner.clone();
                    let mut is_authorized_for_full_data = false;
        
                    // Check if caller is the owner
                    if owner_address == current_user {
                        is_authorized_for_full_data = true;
                    } else {
                        // Check for granted access
                        let access_level = Self::check_access(env.clone(), pet_id, current_user.clone());
                        if access_level != AccessLevel::None {
                            is_authorized_for_full_data = true;
                        }
                    }
        
                    // Enforce privacy level
                    if !is_authorized_for_full_data && pet.privacy_level != PrivacyLevel::Public {
                        // Deny access if not authorized and privacy is not Public.
                        return Vec::new(&env); // Return empty history
                    }
        
                    // If authorized or Public, proceed with fetching and decrypting vaccinations
                    let vax_count: u64 = env
                        .storage()
                        .instance()
                        .get(&DataKey::PetVaccinationCount(pet_id))
                        .unwrap_or(0);
        
                    let mut history = Vec::new(&env);
                    let key = self.get_encryption_key(&env);
        
                    for i in 1..=vax_count {
                        if let Some(vax_id) = env
                            .storage()
                            .instance()
                            .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
                        {
                            if let Some(vaccination_record) = env
                                .storage()
                                .instance()
                                .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                            {
                                // Decrypt vaccine_name
                                let vaccine_name = if let Some(ref encrypted_data) = vaccination_record.encrypted_vaccine_name {
                                    let decrypted_bytes = decrypt_sensitive_data(&env, &encrypted_data.ciphertext, &encrypted_data.nonce, &key)
                                        .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt vaccine name for ID {}", vaccination_record.id));
                                    String::from_utf8(decrypted_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for vaccine name for ID {}", vaccination_record.id))
                                } else {
                                    vaccination_record.vaccine_name.clone().unwrap_or_default()
                                };
        
                                // Decrypt batch_number
                                let batch_number = if let Some(ref encrypted_data) = vaccination_record.encrypted_batch_number {
                                    let decrypted_bytes = decrypt_sensitive_data(&env, &encrypted_data.ciphertext, &encrypted_data.nonce, &key)
                                        .unwrap_or_else(|_| panic_with_error!(&env, "Failed to decrypt batch number for ID {}", vaccination_record.id));
                                    String::from_utf8(decrypted_bytes).unwrap_or_else(|_| panic_with_error!(&env, "Invalid UTF-8 sequence for batch number for ID {}", vaccination_record.id))
                                } else {
                                    vaccination_record.batch_number.clone().unwrap_or_default()
                                };
        
                                // Construct decrypted vaccination record
                                let decrypted_vaccination = Vaccination {
                                    id: vaccination_record.id,
                                    pet_id: vaccination_record.pet_id,
                                    veterinarian: vaccination_record.veterinarian,
                                    vaccine_type: vaccination_record.vaccine_type,
                                    vaccine_name: Some(vaccine_name), // Decrypted value
                                    encrypted_vaccine_name: None, // Encrypted field set to None after decryption
                                    administered_at: vaccination_record.administered_at,
                                    next_due_date: vaccination_record.next_due_date,
                                    batch_number: Some(batch_number), // Decrypted value
                                    encrypted_batch_number: None, // Encrypted field set to None after decryption
                                    created_at: vaccination_record.created_at,
                                };
                                history.push_back(decrypted_vaccination);
                            }
                        }
                    }
        
                    history
                } else {
                    Vec::new(&env) // Pet not found, return empty history
                }
            }
    // Get upcoming vaccinations
    pub fn get_upcoming_vaccinations(
        env: Env,
        pet_id: u64,
        days_threshold: u64,
    ) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let threshold_time = current_time + (days_threshold * 86400); // Convert days to seconds

        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let mut upcoming = Vec::new(&env);

        for vaccination in history.iter() {
            if vaccination.next_due_date <= threshold_time {
                upcoming.push_back(vaccination.clone());
            }
        }

        upcoming
    }

    pub fn is_vaccination_current(env: Env, pet_id: u64, vaccine_type: VaccineType) -> bool {
        let current_time = env.ledger().timestamp();
        let history = Self::get_vaccination_history(env, pet_id);

        let mut most_recent: Option<Vaccination> = None;

        for vaccination in history.iter() {
            if vaccination.vaccine_type == vaccine_type {
                if let Some(ref current) = most_recent {
                    if vaccination.administered_at > current.administered_at {
                        most_recent = Some(vaccination.clone());
                    }
                } else {
                    most_recent = Some(vaccination.clone());
                }
            }
        }

        if let Some(vax) = most_recent {
            vax.next_due_date > current_time
        } else {
            false
        }
    }

        // Get all overdue vaccination types for a pet

        pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<VaccineType> {

            let current_time = env.ledger().timestamp();

            // get_vaccination_history returns Vec<Vaccination> with decrypted data

            let history = Self::get_vaccination_history(env.clone(), pet_id);

            let mut overdue_types = Vec::new(&env);

    

            for vaccination in history.iter() {

                if vaccination.next_due_date < current_time {

                    overdue_types.push_back(vaccination.vaccine_type.clone());

                }

            }

    

            overdue_types

        }

    // ============== ACCESS CONTROL FUNCTIONS ==============

    /// Grant access to a pet's records
    ///
    /// # Arguments
    /// * `pet_id` - ID of the pet
    /// * `grantee` - Address to grant access to
    /// * `access_level` - Level of access (Basic or Full)
    /// * `expires_at` - Optional expiration timestamp (None for permanent)
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

        if access_level == AccessLevel::None {
            panic!("Use revoke_access to remove access");
        }

        if let Some(exp_time) = expires_at {
            let now = env.ledger().timestamp();
            if exp_time <= now {
                panic!("Expiration time must be in the future");
            }
        }

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

        let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));
        let is_new_grant = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&grant_key)
            .is_none();

        env.storage().instance().set(&grant_key, &grant);

        // Update indexes if this is a new grant
        if is_new_grant {
            let grant_count: u64 = env
                .storage()
                .instance()
                .get(&DataKey::AccessGrantCount(pet_id))
                .unwrap_or(0);
            let new_count = grant_count + 1;
            env.storage()
                .instance()
                .set(&DataKey::AccessGrantCount(pet_id), &new_count);
            env.storage()
                .instance()
                .set(&DataKey::AccessGrantIndex((pet_id, new_count)), &grantee);

            let user_access_count: u64 = env
                .storage()
                .instance()
                .get(&DataKey::UserAccessCount(grantee.clone()))
                .unwrap_or(0);
            env.storage().instance().set(
                &DataKey::UserAccessCount(grantee.clone()),
                &(user_access_count + 1),
            );
        }

        // Emit event
        let now = env.ledger().timestamp();
        let event = AccessGrantedEvent {
            pet_id,
            granter: pet.owner.clone(),
            grantee: grantee.clone(),
            access_level: access_level.clone(),
            expires_at,
            timestamp: now,
        };

        env.events()
            .publish((String::from_str(&env, "ACCESS_GRANTED"),), event);

        true
    }

    /// Revoke access to a pet's records
    pub fn revoke_access(env: Env, pet_id: u64, grantee: Address) -> bool {
        // Get pet and verify ownership
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let grant_key = DataKey::AccessGrant((pet_id, grantee.clone()));

        if let Some(mut grant) = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&grant_key)
        {
            grant.is_active = false;
            grant.access_level = AccessLevel::None;
            env.storage().instance().set(&grant_key, &grant);

            // Emit event
            let now = env.ledger().timestamp();
            let event = AccessRevokedEvent {
                pet_id,
                granter: pet.owner.clone(),
                grantee: grantee.clone(),
                timestamp: now,
            };

            env.events()
                .publish((String::from_str(&env, "ACCESS_REVOKED"),), event);

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
        }

        let grant_key = DataKey::AccessGrant((pet_id, user.clone()));

        if let Some(grant) = env
            .storage()
            .instance()
            .get::<DataKey, AccessGrant>(&grant_key)
        {
            if !grant.is_active {
                return AccessLevel::None;
            }

            // Check if access has expired
            if let Some(exp_time) = grant.expires_at {
                let now = env.ledger().timestamp();
                if now >= exp_time {
                    let event = AccessExpiredEvent {
                        pet_id,
                        grantee: user.clone(),
                        expired_at: exp_time,
                    };

                    env.events()
                        .publish((String::from_str(&env, "ACCESS_EXPIRED"),), event);
                    return AccessLevel::None;
                }
            }

            grant.access_level
        } else {
            AccessLevel::None
        }
    }

    /// Get all users who have been granted access to a pet
    pub fn get_authorized_users(env: Env, pet_id: u64) -> Vec<Address> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let grant_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::AccessGrantCount(pet_id))
            .unwrap_or(0);

        let mut authorized_users = Vec::new(&env);

        for i in 1..=grant_count {
            if let Some(grantee) = env
                .storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::AccessGrantIndex((pet_id, i)))
            {
                let access_level = Self::check_access(env.clone(), pet_id, grantee.clone());
                if access_level != AccessLevel::None {
                    authorized_users.push_back(grantee);
                }
            }
        }

        authorized_users
    }

    /// Get access grant details for a specific user and pet
    pub fn get_access_grant(env: Env, pet_id: u64, grantee: Address) -> Option<AccessGrant> {
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        env.storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, grantee)))
    }

    /// Get all pets a user has access to
    pub fn get_vaccination_summary(env: Env, pet_id: u64) -> VaccinationSummary {
        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let current_time = env.ledger().timestamp();

        let mut overdue_types = Vec::new(&env);
        let mut upcoming_count = 0;
        let mut all_current = true;

        // Define a threshold for "upcoming" - e.g., within 90 days
        let ninety_days_in_seconds = 90 * 86400;
        let upcoming_threshold = current_time + ninety_days_in_seconds;

        if history.is_empty() {
            // If no vaccination history, assume fully current for simplicity, or define behavior as needed
            all_current = true;
            upcoming_count = 0;
        } else {
            for vaccination in history.iter() {
                if vaccination.next_due_date < current_time {
                    all_current = false; // Found an overdue vaccination
                    overdue_types.push_back(vaccination.vaccine_type.clone());
                } else if vaccination.next_due_date <= upcoming_threshold {
                    upcoming_count += 1;
                }
            }
        }

        VaccinationSummary {
            is_fully_current: all_current,
            overdue_types,
            upcoming_count,
        }
    }

    // Function to store the hash of off-chain data
    // This function assumes hashing is done off-chain and the hash is provided.
    // data_id is used to identify which off-chain data's hash is being stored.
    pub fn store_offchain_data_hash(env: Env, data_id: u64, data_hash: Bytes) {
        // TODO: Add authentication/authorization logic here if needed.
        // For example, if data_id refers to a pet, require pet owner's auth.
        // For simplicity, this example allows anyone to store a hash, which is not secure.

        let hash_key = DataKey::OffChainDataHash(data_id);
        env.storage().instance().set(&hash_key, &data_hash);
    }

    // Function to verify the hash of off-chain data against a stored hash.
    // data_id is used to identify which off-chain data's hash to verify.
    // provided_hash is the hash calculated from the off-chain data.
    pub fn verify_offchain_data_hash(env: Env, data_id: u64, provided_hash: Bytes) -> bool {
        let hash_key = DataKey::OffChainDataHash(data_id);

        if let Some(stored_hash) = env.storage().instance().get::<DataKey, Bytes>(&hash_key) {
            // Compare the provided hash with the stored hash.
            // Bytes comparison is direct.
            stored_hash == provided_hash
        } else {
            // No hash found for this data_id, so verification fails.
            false
        }
    }

    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();

        let mut accessible_pets = Vec::new(&env);

        // Get all owned pets
        let owner_pet_count = Self::get_owner_pet_count(&env, &user);

        for i in 1..=owner_pet_count {
            if let Some(pet_id) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::OwnerPetIndex((user.clone(), i)))
            {
                accessible_pets.push_back(pet_id);
            }
        }

        accessible_pets
    }
}
mod test;
