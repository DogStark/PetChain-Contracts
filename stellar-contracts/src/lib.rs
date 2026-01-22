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
    pub administered_at: u64,
    pub next_due_date: u64,
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

    // Access Control keys
    AccessGrant((u64, Address)),  // (pet_id, grantee) -> AccessGrant
    AccessGrantCount(u64),        // pet_id -> count of grants
    AccessGrantIndex((u64, u64)), // (pet_id, index) -> grantee Address
    UserAccessList(Address),      // grantee -> list of pet_ids they have access to
    UserAccessCount(Address),     // grantee -> count of pets they can access
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

    // Pet Vaccination Record
    pub fn record_vaccination(
        env: Env,
        pet_id: u64,
        veterinarian: Address,
        vaccine_type: VaccineType,
        administered_at: u64,
        next_due_date: u64,
        // side_effects: Vec<String>
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
            veterinarian,
            vaccine_type,
            administered_at,
            next_due_date,
            created_at: now,
        };

        env.storage()
            .instance()
            .set(&DataKey::Vaccination(vaccine_id), &record);
        env.storage()
            .instance()
            .set(&DataKey::VaccinationCount, &vaccine_id);

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

        if pet.owner != env.clone().current_contract_address() {
            pet.owner.require_auth();
        }

        env.storage()
            .instance()
            .get(&DataKey::AccessGrant((pet_id, grantee)))
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{PetChainContract, PetChainContractClient};
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_register_pet() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Buddy");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Golden Retriever");

        let pet_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        assert_eq!(pet_id, 1);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.id, 1);
        assert_eq!(pet.active, false);
    }

    #[test]
    fn test_register_pet_owner() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let name = String::from_str(&env, "John Doe");
        let email = String::from_str(&env, "john@example.com");
        let emergency = String::from_str(&env, "555-1234");

        client.register_pet_owner(&owner, &name, &email, &emergency);

        let is_registered = client.is_owner_registered(&owner);
        assert_eq!(is_registered, true);
    }

    #[test]
    fn test_record_and_get_vaccination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        // Register pet first
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Buddy");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Golden Retriever");
        let pet_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );

        let administered_time = 1735689600;
        let next_due_date = administered_time + 31536000;
        let now = env.ledger().timestamp();

        let vaccine_id = client.record_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &administered_time,
            &next_due_date,
        );
        assert_eq!(vaccine_id, 1u64);

        let record = client.get_vaccinations(&vaccine_id).unwrap();

        assert_eq!(record.id, 1);
        assert_eq!(record.pet_id, pet_id);
        assert_eq!(record.veterinarian, vet);
        assert_eq!(record.vaccine_type, VaccineType::Rabies);
        assert_eq!(record.administered_at, administered_time);
        assert_eq!(record.next_due_date, next_due_date);
        assert!(record.created_at == now);
    }

    #[test]
    fn test_multiple_record_and_get_vaccination() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let vet = Address::generate(&env);

        // Register pet first
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Buddy");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Golden Retriever");
        let pet_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );

        let pet_id_2 = client.register_pet(
            &owner,
            &String::from_str(&env, "Max"),
            &String::from_str(&env, "2021-05-15"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
        );

        let administered_time = 1735689600;
        let next_due_date = administered_time + 31536000;
        let _now = env.ledger().timestamp();
        let vaccine_id = client.record_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Rabies,
            &administered_time,
            &next_due_date,
        );
        assert_eq!(vaccine_id, 1u64);

        let administered_time = 2735689600;
        let next_due_date = administered_time + 31536000;
        let now = env.ledger().timestamp();
        let vaccine_id_2 = client.record_vaccination(
            &pet_id_2,
            &vet,
            &VaccineType::Other,
            &administered_time,
            &next_due_date,
        );
        assert_eq!(vaccine_id_2, 2u64);

        let record_2 = client.get_vaccinations(&vaccine_id_2).unwrap();

        assert_eq!(record_2.id, 2);
        assert_eq!(record_2.pet_id, pet_id_2);
        assert_eq!(record_2.veterinarian, vet);
        assert_eq!(record_2.vaccine_type, VaccineType::Other);
        assert_eq!(record_2.administered_at, administered_time);
        assert_eq!(record_2.next_due_date, next_due_date);
        assert!(record_2.created_at == now);
    }
}
