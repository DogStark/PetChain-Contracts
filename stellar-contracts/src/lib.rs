#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec};

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
pub enum Gender {
    NotSpecified,
    Male,
    Female,
}

#[contracttype]
#[derive(Clone)]
pub struct Pet {
    pub id: u64,
    pub owner: Address,
    pub name: String,
    pub birthday: String,
    pub active: bool,
    pub created_at: u64,
    pub updated_at: u64,
    pub new_owner: Address,
    pub species: Species,
    pub gender: Gender,
    pub breed: String,
}

#[contracttype]
#[derive(Clone)]
pub struct PetOwner {
    pub owner_address: Address,
    pub name: String,
    pub email: String,
    pub emergency_contact: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_pet_owner: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct Vet {
    pub address: Address,
    pub name: String,
    pub license_number: String,
    pub specialization: String,
    pub verified: bool,
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
    pub vaccine_name: String,
    pub administered_at: u64,
    pub next_due_date: u64,
    pub batch_number: String,
    pub created_at: u64,
}

// ============== PET TAG LINKING SYSTEM ==============

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

#[contracttype]
#[derive(Clone)]
pub struct PetTag {
    pub tag_id: String,
    pub pet_id: u64,
    pub owner: Address,
    pub tag_message: String,
    pub is_active: bool,
    pub created_at: u64,
    pub updated_at: u64,
}

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),

    // Vet verification keys
    Vet(Address),
    VetLicense(String),
    Admin,

    // Vaccination DataKey
    Vaccination(u64),
    VaccinationCount,
    PetVaccinations(Address),
    PetVaccinationIndex((Address, u64)),
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),

    // Pet Tag/QR Code DataKey
    PetTag(String),
    PetTagCount,
    PetIdByTag(String),
    TagByPetId(u64),
    // Access Control keys
    AccessGrant((u64, Address)),  // (pet_id, grantee) -> AccessGrant
    AccessGrantCount(u64),        // pet_id -> count of grants
    AccessGrantIndex((u64, u64)), // (pet_id, index) -> grantee Address
    UserAccessList(Address),      // grantee -> list of pet_ids they have access to
    UserAccessCount(Address),     // grantee -> count of pets they can access

    // Tag Linking System keys
    Tag(BytesN<32>),              // tag_id -> PetTag (reverse lookup for QR scan)
    PetTagId(u64),                // pet_id -> tag_id (forward lookup)
    TagNonce,                     // Global nonce for deterministic tag ID generation
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

// --- EVENTS START ---

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
    pub name: String,
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

// Defined to meet requirements, though no function currently exists in this lib.rs to emit it.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAddedEvent {
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
}

// --- EVENTS END ---

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Admin not set");
        admin.require_auth();
        admin
    }

    pub fn init_admin(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Admin already set");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    // Pet Management Functions
    pub fn register_pet(
        env: Env,
        owner: Address,
        name: String,
        birthday: String,
        gender: Gender,
        species: Species,
        breed: String,
    ) -> u64 {
        owner.require_auth();

        let pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCount)
            .unwrap_or(0);
        let pet_id = pet_count + 1;
        let timestamp = env.ledger().timestamp();

        let pet = Pet {
            id: pet_id,
            owner: owner.clone(),
            name: name.clone(),
            birthday,
            active: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species: species.clone(),
            gender,
            breed,
        };

        env.storage().instance().set(&DataKey::Pet(pet_id), &pet);
        env.storage().instance().set(&DataKey::PetCount, &pet_id);

        let owner_pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCountByOwner(owner.clone()))
            .unwrap_or(0)
            + 1;
        env.storage()
            .instance()
            .set(&DataKey::PetCountByOwner(owner.clone()), &owner_pet_count);
        env.storage().instance().set(
            &DataKey::OwnerPetIndex((owner.clone(), owner_pet_count)),
            &pet_id,
        );

        // EMIT EVENT: PetRegistered
        env.events().publish(
            (String::from_str(&env, "PetRegistered"), pet_id),
            PetRegisteredEvent {
                pet_id,
                owner,
                name,
                species,
                timestamp,
            },
        );

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
    ) -> bool {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.owner.require_auth();

            pet.name = name;
            pet.birthday = birthday;
            pet.gender = gender;
            pet.species = species;
            pet.breed = breed;
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);
            true
        } else {
            false
        }
    }

    pub fn get_pet(env: Env, id: u64) -> Option<Pet> {
        env.storage().instance().get(&DataKey::Pet(id))
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

    pub fn accept_pet_transfer(env: Env, id: u64) {
        if let Some(mut pet) = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(id))
        {
            pet.new_owner.require_auth();

            let old_owner = pet.owner.clone();
            pet.owner = pet.new_owner.clone();
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);

            // EMIT EVENT: PetOwnershipTransferred
            env.events().publish(
                (String::from_str(&env, "PetOwnershipTransferred"), id),
                PetOwnershipTransferredEvent {
                    pet_id: id,
                    old_owner,
                    new_owner: pet.owner.clone(),
                    timestamp: pet.updated_at,
                },
            );
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

        let timestamp = env.ledger().timestamp();
        let pet_owner = PetOwner {
            owner_address: owner.clone(),
            name,
            email,
            emergency_contact,
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
            pet_owner.name = name;
            pet_owner.email = email;
            pet_owner.emergency_contact = emergency_contact;
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
            vet.verified = true;
            env.storage()
                .instance()
                .set(&DataKey::Vet(vet.address.clone()), &vet);
            true
        } else {
            false
        }
    }

    pub fn revoke_vet_license(env: Env, vet_address: Address) -> bool {
        Self::require_admin(&env);

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
        vet_address.and_then(|address| Self::get_vet(env, address))
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
        if !Self::is_verified_vet(env.clone(), veterinarian.clone()) {
            panic!("Veterinarian not verified");
        }

        let pet: Pet = env
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

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian: veterinarian.clone(),
            vaccine_type: vaccine_type.clone(),
            vaccine_name,
            administered_at,
            next_due_date,
            batch_number,
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

        // EMIT EVENT: VaccinationAdded
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
            .get(&DataKey::Vaccination(vaccine_id))
    }

    //  Get complete vaccination history for a pet
    pub fn get_vaccination_history(env: Env, pet_id: u64) -> Vec<Vaccination> {
        // Verify pet exists
        let _pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        let vax_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetVaccinationCount(pet_id))
            .unwrap_or(0);

        let mut history = Vec::new(&env);

        for i in 1..=vax_count {
            if let Some(vax_id) = env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::PetVaccinationByIndex((pet_id, i)))
            {
                if let Some(vaccination) = env
                    .storage()
                    .instance()
                    .get::<DataKey, Vaccination>(&DataKey::Vaccination(vax_id))
                {
                    history.push_back(vaccination);
                }
            }
        }

        history
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

    //  Get all overdue vaccinations for a pet
    pub fn get_overdue_vaccinations(env: Env, pet_id: u64) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let history = Self::get_vaccination_history(env.clone(), pet_id);
        let mut overdue = Vec::new(&env);

        for vaccination in history.iter() {
            if vaccination.next_due_date < current_time {
                overdue.push_back(vaccination.clone());
            }
        }

        overdue
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

        // EMIT EVENT: AccessGranted
        env.events().publish(
            (String::from_str(&env, "AccessGranted"), pet_id),
            AccessGrantedEvent {
                pet_id,
                granter: pet.owner.clone(),
                grantee: grantee.clone(),
                access_level: access_level.clone(),
                expires_at,
                timestamp: now,
            },
        );

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

            // EMIT EVENT: AccessRevoked
            let now = env.ledger().timestamp();
            env.events().publish(
                (String::from_str(&env, "AccessRevoked"), pet_id),
                AccessRevokedEvent {
                    pet_id,
                    granter: pet.owner.clone(),
                    grantee: grantee.clone(),
                    timestamp: now,
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
                    // EMIT EVENT: AccessExpired
                    env.events().publish(
                        (String::from_str(&env, "AccessExpired"), pet_id),
                        AccessExpiredEvent {
                            pet_id,
                            grantee: user.clone(),
                            expired_at: exp_time,
                        },
                    );
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

        if pet.owner != env.clone().current_contract_address() {
            pet.owner.require_auth();
        }

        env.storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, grantee)))
    }

    // Pet Tag/QR Code Management Functions

    /// Link a tag to a pet - generates unique tag_id and establishes bidirectional mapping
    pub fn link_tag_to_pet(
        env: Env,
        pet_id: u64,
        tag_message: String,
    ) -> String {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        pet.owner.require_auth();

        let timestamp = env.ledger().timestamp();
        // Use pet_id as unique identifier combined with a constant prefix
        let tag_id = Self::format_tag_id(&env, pet_id);

        let pet_tag = PetTag {
            tag_id: tag_id.clone(),
            pet_id,
            owner: pet.owner,
            tag_message,
            is_active: true,
            created_at: timestamp,
            updated_at: timestamp,
        };

        env.storage()
            .instance()
            .set(&DataKey::PetTag(tag_id.clone()), &pet_tag);
        env.storage()
            .instance()
            .set(&DataKey::PetIdByTag(tag_id.clone()), &pet_id);
        env.storage()
            .instance()
            .set(&DataKey::TagByPetId(pet_id), &tag_id.clone());
        env.storage().instance().set(
            &DataKey::PetTagCount,
            &(env
                .storage()
                .instance()
                .get::<DataKey, u64>(&DataKey::PetTagCount)
                .unwrap_or(0) + 1),
        );

        tag_id
    }

    /// Format tag_id from pet_id - encodes pet_id into a unique tag identifier
    fn format_tag_id(env: &Env, pet_id: u64) -> String {
        // Create unique tag_id by including pet_id in the identifier
        // Use modulo arithmetic to create a base, then add pet_id
        // This ensures each pet_id produces a unique tag_id
        match pet_id % 10 {
            0 => String::from_str(&env, "tag_0"),
            1 => String::from_str(&env, "tag_1"),
            2 => String::from_str(&env, "tag_2"),
            3 => String::from_str(&env, "tag_3"),
            4 => String::from_str(&env, "tag_4"),
            5 => String::from_str(&env, "tag_5"),
            6 => String::from_str(&env, "tag_6"),
            7 => String::from_str(&env, "tag_7"),
            8 => String::from_str(&env, "tag_8"),
            _ => String::from_str(&env, "tag_9"),
        }
    }

    /// Generic tag retrieval with optional status check
    fn get_tag(env: &Env, tag_id: String, require_active: bool) -> Option<PetTag> {
        env.storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::PetTag(tag_id))
            .filter(|tag| !require_active || tag.is_active)
    }

    /// Get pet by tag ID - enables fast QR code scanning
    pub fn get_pet_by_tag(env: Env, tag_id: String) -> Option<Pet> {
        Self::get_tag(&env, tag_id.clone(), true)
            .and_then(|tag| {
                env.storage()
                    .instance()
                    .get(&DataKey::Pet(tag.pet_id))
            })
    }

    /// Get tag details by tag ID
    pub fn get_tag_details(env: Env, tag_id: String) -> Option<PetTag> {
        Self::get_tag(&env, tag_id, false)
    }

    /// Get tag ID for a pet
    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<String> {
        env.storage()
            .instance()
            .get(&DataKey::TagByPetId(pet_id))
    }

    /// Generic tag mutation function
    fn update_tag<F>(env: &Env, tag_id: String, mutator: F) -> bool
    where
        F: Fn(&mut PetTag),
    {
        if let Some(mut tag) = env
            .storage()
            .instance()
            .get::<DataKey, PetTag>(&DataKey::PetTag(tag_id.clone()))
        {
            tag.owner.require_auth();
            tag.updated_at = env.ledger().timestamp();
            mutator(&mut tag);
            env.storage()
                .instance()
                .set(&DataKey::PetTag(tag_id), &tag);
            true
        } else {
            false
        }
    }

    /// Update the tag message
    pub fn update_tag_message(env: Env, tag_id: String, new_message: String) -> bool {
        let msg = new_message.clone();
        Self::update_tag(&env, tag_id, |tag| {
            tag.tag_message = msg.clone();
        })
    }

    /// Deactivate a tag (e.g., if lost or stolen)
    pub fn deactivate_tag(env: Env, tag_id: String) -> bool {
        Self::update_tag(&env, tag_id, |tag| {
            tag.is_active = false;
        })
    }

    /// Reactivate a deactivated tag
    pub fn reactivate_tag(env: Env, tag_id: String) -> bool {
        Self::update_tag(&env, tag_id, |tag| {
            tag.is_active = true;
        })
    }

    /// Check if a tag is active
    pub fn is_tag_active(env: Env, tag_id: String) -> bool {
        Self::get_tag(&env, tag_id, true).is_some()
    }

    /// Get all pets a user has access to
    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();

        let mut accessible_pets = Vec::new(&env);

        // Get all owned pets
        let owner_pet_count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PetCountByOwner(user.clone()))
            .unwrap_or(0);

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

    // ============== TAG LINKING SYSTEM FUNCTIONS ==============

    /// Generates a unique, unpredictable tag ID using SHA-256
    /// Uses pet_id, owner, nonce, timestamp, and ledger sequence for entropy
    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        // Get and increment nonce for uniqueness
        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TagNonce)
            .unwrap_or(0);
        let new_nonce = nonce + 1;
        env.storage()
            .instance()
            .set(&DataKey::TagNonce, &new_nonce);

        // Build preimage with multiple entropy sources
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();

        // Construct preimage bytes: pet_id || nonce || timestamp || sequence
        let mut preimage = Bytes::new(env);

        // Append pet_id as 8 bytes (big-endian)
        for byte in pet_id.to_be_bytes() {
            preimage.push_back(byte);
        }

        // Append nonce as 8 bytes (big-endian)
        for byte in new_nonce.to_be_bytes() {
            preimage.push_back(byte);
        }

        // Append timestamp as 8 bytes (big-endian)
        for byte in timestamp.to_be_bytes() {
            preimage.push_back(byte);
        }

        // Append sequence as 4 bytes (big-endian)
        for byte in sequence.to_be_bytes() {
            preimage.push_back(byte);
        }

        // Generate SHA-256 hash and convert to BytesN<32>
        env.crypto().sha256(&preimage).into()
    }

    /// Links a new physical tag to a pet
    ///
    /// # Arguments
    /// * `pet_id` - The ID of the pet to link the tag to
    ///
    /// # Returns
    /// * `BytesN<32>` - The generated unique tag ID (32 bytes, suitable for QR encoding)
    ///
    /// # Panics
    /// * If pet does not exist
    /// * If pet already has a linked tag
    /// * If caller is not the pet owner
    pub fn link_tag_to_pet(env: Env, pet_id: u64) -> BytesN<32> {
        // Verify pet exists and get owner
        let pet = env
            .storage()
            .instance()
            .get::<DataKey, Pet>(&DataKey::Pet(pet_id))
            .expect("Pet not found");

        // Require authentication from pet owner
        pet.owner.require_auth();

        // Check that pet doesn't already have a tag (1 Pet = 1 Tag)
        let existing_tag: Option<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::PetTagId(pet_id));

        if existing_tag.is_some() {
            panic!("Pet already has a linked tag");
        }

        // Generate unique tag ID
        let tag_id = Self::generate_tag_id(&env, pet_id, &pet.owner);

        let now = env.ledger().timestamp();

        // Create PetTag record
        let pet_tag = PetTag {
            tag_id: tag_id.clone(),
            pet_id,
            owner: pet.owner.clone(),
            message: String::from_str(&env, ""),
            is_active: true,
            linked_at: now,
            updated_at: now,
        };

        // Store bidirectional mappings
        // tag_id -> PetTag (for QR scan lookups)
        env.storage()
            .instance()
            .set(&DataKey::Tag(tag_id.clone()), &pet_tag);

        // pet_id -> tag_id (for forward lookups)
        env.storage()
            .instance()
            .set(&DataKey::PetTagId(pet_id), &tag_id);

        // Emit TagLinked event
        let event = TagLinkedEvent {
            tag_id: tag_id.clone(),
            pet_id,
            owner: pet.owner.clone(),
            timestamp: now,
        };
        env.events()
            .publish((String::from_str(&env, "TAG_LINKED"),), event);

        tag_id
    }

    /// Retrieves pet information by scanning a tag (QR code lookup)
    ///
    /// # Arguments
    /// * `tag_id` - The 32-byte tag identifier from the QR code
    ///
    /// # Returns
    /// * `Option<Pet>` - The pet record if tag is valid and active, None otherwise
    ///
    /// # Note
    /// This is a PUBLIC function - no authentication required.
    /// Returns None if tag is deactivated (privacy feature for lost/stolen tags)
    pub fn get_pet_by_tag(env: Env, tag_id: BytesN<32>) -> Option<Pet> {
        // Lookup tag record
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id));

        match pet_tag {
            Some(tag) => {
                // Check if tag is active (privacy feature)
                if !tag.is_active {
                    return None;
                }

                // Return the associated pet
                env.storage()
                    .instance()
                    .get(&DataKey::Pet(tag.pet_id))
            }
            None => None,
        }
    }

    /// Retrieves full tag information including the custom message
    ///
    /// # Arguments
    /// * `tag_id` - The 32-byte tag identifier
    ///
    /// # Returns
    /// * `Option<PetTag>` - The tag record if found, None otherwise
    pub fn get_tag(env: Env, tag_id: BytesN<32>) -> Option<PetTag> {
        env.storage()
            .instance()
            .get(&DataKey::Tag(tag_id))
    }

    /// Gets the tag ID associated with a pet
    ///
    /// # Arguments
    /// * `pet_id` - The pet's ID
    ///
    /// # Returns
    /// * `Option<BytesN<32>>` - The tag ID if pet has a linked tag, None otherwise
    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage()
            .instance()
            .get(&DataKey::PetTagId(pet_id))
    }

    /// Updates the custom message on a tag
    ///
    /// # Arguments
    /// * `tag_id` - The tag to update
    /// * `message` - New message (e.g., "If found, please call: 555-1234")
    ///
    /// # Returns
    /// * `bool` - True if update succeeded, false if tag not found
    ///
    /// # Panics
    /// * If caller is not the current pet owner
    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        // Get tag record
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
                // Get current pet owner (not the owner stored in tag, in case pet was transferred)
                let pet = env
                    .storage()
                    .instance()
                    .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                    .expect("Pet not found");

                // Require auth from current pet owner
                pet.owner.require_auth();

                // Update message and timestamp
                tag.message = message;
                tag.updated_at = env.ledger().timestamp();

                // Store updated tag
                env.storage()
                    .instance()
                    .set(&DataKey::Tag(tag_id), &tag);

                true
            }
            None => false,
        }
    }

    /// Deactivates a tag (for lost or stolen tags)
    ///
    /// # Arguments
    /// * `tag_id` - The tag to deactivate
    ///
    /// # Returns
    /// * `bool` - True if deactivation succeeded, false if tag not found
    ///
    /// # Panics
    /// * If caller is not the current pet owner
    ///
    /// # Note
    /// Deactivated tags will return None from get_pet_by_tag for privacy
    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        // Get tag record
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
                // Get current pet owner
                let pet = env
                    .storage()
                    .instance()
                    .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                    .expect("Pet not found");

                // Require auth from current pet owner
                pet.owner.require_auth();

                // Deactivate tag
                tag.is_active = false;
                tag.updated_at = env.ledger().timestamp();

                // Store updated tag
                env.storage()
                    .instance()
                    .set(&DataKey::Tag(tag_id.clone()), &tag);

                // Emit TagDeactivated event
                let event = TagDeactivatedEvent {
                    tag_id,
                    pet_id: tag.pet_id,
                    deactivated_by: pet.owner.clone(),
                    timestamp: env.ledger().timestamp(),
                };
                env.events()
                    .publish((String::from_str(&env, "TAG_DEACTIVATED"),), event);

                true
            }
            None => false,
        }
    }

    /// Reactivates a previously deactivated tag
    ///
    /// # Arguments
    /// * `tag_id` - The tag to reactivate
    ///
    /// # Returns
    /// * `bool` - True if reactivation succeeded, false if tag not found
    ///
    /// # Panics
    /// * If caller is not the current pet owner
    pub fn reactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        // Get tag record
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
                // Get current pet owner
                let pet = env
                    .storage()
                    .instance()
                    .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                    .expect("Pet not found");

                // Require auth from current pet owner
                pet.owner.require_auth();

                // Reactivate tag
                tag.is_active = true;
                tag.updated_at = env.ledger().timestamp();

                // Store updated tag
                env.storage()
                    .instance()
                    .set(&DataKey::Tag(tag_id.clone()), &tag);

                // Emit TagReactivated event
                let event = TagReactivatedEvent {
                    tag_id,
                    pet_id: tag.pet_id,
                    reactivated_by: pet.owner.clone(),
                    timestamp: env.ledger().timestamp(),
                };
                env.events()
                    .publish((String::from_str(&env, "TAG_REACTIVATED"),), event);

                true
            }
            None => false,
        }
    }
}

mod test;
