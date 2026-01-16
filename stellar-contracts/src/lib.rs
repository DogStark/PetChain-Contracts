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
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
    PetOwner(Address),
    OwnerPetIndex((Address, u64)),
    PetCountByOwner(Address),
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
    
    pub fn get_pet_owner(env: Env, owner_addr: Address) -> Option<PetOwner> {
        env.storage().instance().get(&DataKey::PetOwner(owner_addr))
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
}
