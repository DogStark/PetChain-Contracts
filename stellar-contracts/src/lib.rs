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

#[contracttype]
pub enum DataKey {
    Pet(u64),
    PetCount,
    OwnerPets(Address),
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
}
