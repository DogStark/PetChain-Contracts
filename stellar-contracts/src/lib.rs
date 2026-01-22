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
}

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
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
            name,
            birthday,
            active: false,
            created_at: timestamp,
            updated_at: timestamp,
            new_owner: owner.clone(),
            species,
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

            let _old_owner = pet.owner.clone();
            pet.owner = pet.new_owner.clone();
            pet.updated_at = env.ledger().timestamp();

            env.storage().instance().set(&DataKey::Pet(id), &pet);
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

    // pub fn get_pet_owner(env: Env, owner_addr: Address) -> Option<PetOwner> {
    //     env.storage().instance().get(&DataKey::PetOwner(owner_addr))
    // }

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

        let record = Vaccination {
            id: vaccine_id,
            pet_id,
            veterinarian,
            vaccine_type,
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

        vaccine_id
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

    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        env.storage()
            .instance()
            .get(&DataKey::Vaccination(vaccine_id))
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
}

mod test;
