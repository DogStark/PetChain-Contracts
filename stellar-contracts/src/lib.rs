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
}