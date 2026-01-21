#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec, Symbol};

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
    PetVaccinationIndex((Address, u64))
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
        
        let pet_count: u64 = env.storage().instance().get(&DataKey::PetCount).unwrap_or(0);
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
        
        let owner_pet_count: u64 = env.storage().instance()
            .get(&DataKey::PetCountByOwner(owner.clone())).unwrap_or(0) + 1;
        env.storage().instance().set(&DataKey::PetCountByOwner(owner.clone()), &owner_pet_count);
        env.storage().instance().set(&DataKey::OwnerPetIndex((owner.clone(), owner_pet_count)), &pet_id);
        
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
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id)) {
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
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id)) {
            pet.active = true;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    
    pub fn deactivate_pet(env: Env, id: u64) {
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id)) {
            pet.owner.require_auth();
            pet.active = false;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    
    pub fn transfer_pet_ownership(env: Env, id: u64, to: Address) {
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id)) {
            pet.owner.require_auth();
            pet.new_owner = to;
            pet.updated_at = env.ledger().timestamp();
            env.storage().instance().set(&DataKey::Pet(id), &pet);
        }
    }
    
    pub fn accept_pet_transfer(env: Env, id: u64) {
        if let Some(mut pet) = env.storage().instance().get::<DataKey, Pet>(&DataKey::Pet(id)) {
            pet.new_owner.require_auth();
            
            let old_owner = pet.owner.clone();
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
        
        env.storage().instance().set(&DataKey::PetOwner(owner), &pet_owner);
    }
    
    pub fn is_owner_registered(env: Env, owner: Address) -> bool {
        if let Some(pet_owner) = env.storage().instance().get::<DataKey, PetOwner>(&DataKey::PetOwner(owner)) {
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
        
        if let Some(mut pet_owner) = env.storage().instance().get::<DataKey, PetOwner>(&DataKey::PetOwner(owner.clone())) {
            pet_owner.name = name;
            pet_owner.email = email;
            pet_owner.emergency_contact = emergency_contact;
            pet_owner.updated_at = env.ledger().timestamp();
            
            env.storage().instance().set(&DataKey::PetOwner(owner), &pet_owner);
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

        let pet: Pet = env.storage().instance().get(&DataKey::Pet(pet_id)).expect("Pet not found");
        let vaccine_count: u64 = env.storage().instance().get(&DataKey::VaccinationCount).unwrap_or(0);
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

        env.storage().instance().set(&DataKey::Vaccination(vaccine_id), &record);
        env.storage().instance().set(&DataKey::VaccinationCount, &vaccine_id);

        let pet_vaccine_count_key = DataKey::PetVaccinations(pet.owner.clone());
        let mut pet_vaccine_count: u64 = env.storage().instance().get(&pet_vaccine_count_key).unwrap_or(0);
        pet_vaccine_count += 1;

        env.storage().instance().set(&pet_vaccine_count_key, &pet_vaccine_count);
        env.storage().instance().set(&DataKey::PetVaccinationIndex((pet.owner.clone(), pet_vaccine_count)),&vaccine_id,);

        vaccine_id
    }

    pub fn get_vaccinations(env: Env, vaccine_id: u64) -> Option<Vaccination> {
        env.storage().instance().get(&DataKey::Vaccination(vaccine_id))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};
    use crate::{PetChainContract, PetChainContractClient};

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
        
        let pet_id = client.register_pet(&owner, &name, &birthday, &Gender::Male, &Species::Dog, &breed);
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
        let pet_id = client.register_pet(&owner, &name, &birthday, &Gender::Male, &Species::Dog, &breed);

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
        let pet_id = client.register_pet(&owner, &name, &birthday, &Gender::Male, &Species::Dog, &breed);

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
        let now = env.ledger().timestamp();
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
