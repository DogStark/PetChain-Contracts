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
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),

    Vaccination(u64),
    VaccinationCount,
    PetVaccinations(Address),
    PetVaccinationIndex((Address, u64)),
    PetVaccinationCount(u64),
    PetVaccinationByIndex((u64, u64)),

    AccessGrant((u64, Address)),
    AccessGrantCount(u64),
    AccessGrantIndex((u64, u64)),
    UserAccessList(Address),
    UserAccessCount(Address),

    Tag(BytesN<32>),
    PetTagId(u64),
    TagNonce,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MedicalRecordAddedEvent {
    pub pet_id: u64,
    pub updated_by: Address,
    pub timestamp: u64,
}

#[contract]
pub struct PetChainContract;

#[contractimpl]
impl PetChainContract {
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

    pub fn get_vaccination_history(env: Env, pet_id: u64) -> Vec<Vaccination> {
        let pet_exists: Option<Pet> = env.storage().instance().get(&DataKey::Pet(pet_id));
        if pet_exists.is_none() {
            return Vec::new(&env);
        }

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

    pub fn get_upcoming_vaccinations(env: Env, pet_id: u64, days_threshold: u64) -> Vec<Vaccination> {
        let current_time = env.ledger().timestamp();
        let threshold_time = current_time + (days_threshold * 86400);
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

    pub fn revoke_access(env: Env, pet_id: u64, grantee: Address) -> bool {
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

            if let Some(exp_time) = grant.expires_at {
                let now = env.ledger().timestamp();
                if now >= exp_time {
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

    pub fn get_accessible_pets(env: Env, user: Address) -> Vec<u64> {
        user.require_auth();

        let mut accessible_pets = Vec::new(&env);

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

    fn generate_tag_id(env: &Env, pet_id: u64, _owner: &Address) -> BytesN<32> {
        let nonce: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TagNonce)
            .unwrap_or(0);
        let new_nonce = nonce + 1;
        env.storage()
            .instance()
            .set(&DataKey::TagNonce, &new_nonce);

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
            .expect("Pet not found");

        pet.owner.require_auth();

        let existing_tag: Option<BytesN<32>> = env
            .storage()
            .instance()
            .get(&DataKey::PetTagId(pet_id));

        if existing_tag.is_some() {
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
        };

        env.storage()
            .instance()
            .set(&DataKey::Tag(tag_id.clone()), &pet_tag);

        env.storage()
            .instance()
            .set(&DataKey::PetTagId(pet_id), &tag_id);

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

    pub fn get_pet_by_tag(env: Env, tag_id: BytesN<32>) -> Option<Pet> {
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id));

        match pet_tag {
            Some(tag) => {
                if !tag.is_active {
                    return None;
                }
                env.storage().instance().get(&DataKey::Pet(tag.pet_id))
            }
            None => None,
        }
    }

    pub fn get_tag(env: Env, tag_id: BytesN<32>) -> Option<PetTag> {
        env.storage().instance().get(&DataKey::Tag(tag_id))
    }

    pub fn get_tag_by_pet(env: Env, pet_id: u64) -> Option<BytesN<32>> {
        env.storage().instance().get(&DataKey::PetTagId(pet_id))
    }

    pub fn update_tag_message(env: Env, tag_id: BytesN<32>, message: String) -> bool {
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
                let pet = env
                    .storage()
                    .instance()
                    .get::<DataKey, Pet>(&DataKey::Pet(tag.pet_id))
                    .expect("Pet not found");

                pet.owner.require_auth();
                tag.message = message;
                tag.updated_at = env.ledger().timestamp();

                env.storage()
                    .instance()
                    .set(&DataKey::Tag(tag_id), &tag);
                true
            }
            None => false,
        }
    }

    pub fn deactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
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

    pub fn reactivate_tag(env: Env, tag_id: BytesN<32>) -> bool {
        let pet_tag: Option<PetTag> = env
            .storage()
            .instance()
            .get(&DataKey::Tag(tag_id.clone()));

        match pet_tag {
            Some(mut tag) => {
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
