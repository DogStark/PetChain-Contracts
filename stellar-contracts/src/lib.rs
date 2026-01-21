#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

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

    pub fn get_owner_profile(env: Env, owner_addr: Address) -> Option<PetOwner> {
        env.storage().instance().get(&DataKey::PetOwner(owner_addr))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        Env,
    };

    // ============================================================================
    // HELPER FUNCTIONS FOR TESTS
    // ============================================================================

    /// Creates a test environment with a registered contract
    fn setup_test_env() -> (Env, PetChainContractClient<'static>) {
        let env = Env::default();
        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);
        (env, client)
    }

    /// Registers a test pet owner and returns the address
    fn register_test_owner(env: &Env, client: &PetChainContractClient) -> Address {
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "John Doe");
        let email = String::from_str(&env, "john@example.com");
        let emergency = String::from_str(&env, "555-1234");
        client.register_pet_owner(&owner, &name, &email, &emergency);
        owner
    }

    /// Registers a test pet and returns the pet ID
    fn register_test_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
        env.mock_all_auths();
        let name = String::from_str(&env, "Buddy");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Golden Retriever");
        client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        )
    }

    // ============================================================================
    // PET REGISTRATION TESTS
    // ============================================================================

    #[test]
    fn test_register_pet_creates_pet_with_correct_data() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
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
        assert_eq!(pet.owner, owner);
        assert_eq!(pet.name, name);
        assert_eq!(pet.birthday, birthday);
        assert_eq!(pet.gender, Gender::Male);
        assert_eq!(pet.species, Species::Dog);
        assert_eq!(pet.breed, breed);
        assert_eq!(pet.active, false);
        assert_eq!(pet.new_owner, owner);
    }

    #[test]
    fn test_register_multiple_pets_increments_id() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Pet");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Breed");

        let pet_id_1 = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        let pet_id_2 = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Female,
            &Species::Cat,
            &breed,
        );
        let pet_id_3 = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::NotSpecified,
            &Species::Bird,
            &breed,
        );

        assert_eq!(pet_id_1, 1);
        assert_eq!(pet_id_2, 2);
        assert_eq!(pet_id_3, 3);
    }

    #[test]
    fn test_register_pet_with_all_species() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Pet");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Breed");

        let dog_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        let cat_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Female,
            &Species::Cat,
            &breed,
        );
        let bird_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::NotSpecified,
            &Species::Bird,
            &breed,
        );
        let other_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Other,
            &breed,
        );

        assert_eq!(client.get_pet(&dog_id).unwrap().species, Species::Dog);
        assert_eq!(client.get_pet(&cat_id).unwrap().species, Species::Cat);
        assert_eq!(client.get_pet(&bird_id).unwrap().species, Species::Bird);
        assert_eq!(client.get_pet(&other_id).unwrap().species, Species::Other);
    }

    #[test]
    fn test_register_pet_with_all_genders() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let name = String::from_str(&env, "Pet");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Breed");

        let male_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        let female_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::Female,
            &Species::Dog,
            &breed,
        );
        let not_specified_id = client.register_pet(
            &owner,
            &name,
            &birthday,
            &Gender::NotSpecified,
            &Species::Dog,
            &breed,
        );

        assert_eq!(client.get_pet(&male_id).unwrap().gender, Gender::Male);
        assert_eq!(client.get_pet(&female_id).unwrap().gender, Gender::Female);
        assert_eq!(
            client.get_pet(&not_specified_id).unwrap().gender,
            Gender::NotSpecified
        );
    }

    // ============================================================================
    // PET UPDATE TESTS
    // ============================================================================

    #[test]
    fn test_update_pet_profile_updates_all_fields() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let new_name = String::from_str(&env, "Max");
        let new_birthday = String::from_str(&env, "2021-06-15");
        let new_breed = String::from_str(&env, "Labrador");

        let result = client.update_pet_profile(
            &pet_id,
            &new_name,
            &new_birthday,
            &Gender::Female,
            &Species::Cat,
            &new_breed,
        );
        assert_eq!(result, true);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.name, new_name);
        assert_eq!(pet.birthday, new_birthday);
        assert_eq!(pet.gender, Gender::Female);
        assert_eq!(pet.species, Species::Cat);
        assert_eq!(pet.breed, new_breed);
    }

    #[test]
    fn test_update_pet_profile_updates_timestamp() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let pet_before = client.get_pet(&pet_id).unwrap();
        let created_at = pet_before.created_at;

        // Advance ledger time
        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);

        let new_name = String::from_str(&env, "Max");
        let new_birthday = String::from_str(&env, "2021-06-15");
        let new_breed = String::from_str(&env, "Labrador");

        client.update_pet_profile(
            &pet_id,
            &new_name,
            &new_birthday,
            &Gender::Male,
            &Species::Dog,
            &new_breed,
        );

        let pet_after = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet_after.created_at, created_at);
        assert!(pet_after.updated_at > created_at);
    }

    #[test]
    fn test_update_nonexistent_pet_returns_false() {
        let (_env, client) = setup_test_env();
        let name = String::from_str(&_env, "Max");
        let birthday = String::from_str(&_env, "2021-06-15");
        let breed = String::from_str(&_env, "Labrador");

        let result =
            client.update_pet_profile(&999, &name, &birthday, &Gender::Male, &Species::Dog, &breed);
        assert_eq!(result, false);
    }

    // ============================================================================
    // PET QUERY TESTS
    // ============================================================================

    #[test]
    fn test_get_pet_returns_none_for_nonexistent_pet() {
        let (_env, client) = setup_test_env();
        let pet = client.get_pet(&999);
        assert_eq!(pet.is_none(), true);
    }

    #[test]
    fn test_is_pet_active_returns_false_for_inactive_pet() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        assert_eq!(client.is_pet_active(&pet_id), false);
    }

    #[test]
    fn test_is_pet_active_returns_false_for_nonexistent_pet() {
        let (_env, client) = setup_test_env();
        assert_eq!(client.is_pet_active(&999), false);
    }

    #[test]
    fn test_get_pet_owner_returns_correct_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let retrieved_owner = client.get_pet_owner(&pet_id).unwrap();
        assert_eq!(retrieved_owner, owner);
    }

    #[test]
    fn test_get_pet_owner_returns_none_for_nonexistent_pet() {
        let (_env, client) = setup_test_env();
        let owner = client.get_pet_owner(&999);
        assert_eq!(owner.is_none(), true);
    }

    // ============================================================================
    // PET ACTIVATION/DEACTIVATION TESTS
    // ============================================================================

    #[test]
    fn test_activate_pet_sets_active_to_true() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.activate_pet(&pet_id);

        assert_eq!(client.is_pet_active(&pet_id), true);
        assert_eq!(client.get_pet(&pet_id).unwrap().active, true);
    }

    #[test]
    fn test_deactivate_pet_sets_active_to_false() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.activate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), true);

        client.deactivate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), false);
    }

    #[test]
    fn test_activate_deactivate_updates_timestamp() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let pet_before = client.get_pet(&pet_id).unwrap();
        let created_at = pet_before.created_at;

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);
        client.activate_pet(&pet_id);

        let pet_after_activate = client.get_pet(&pet_id).unwrap();
        assert!(pet_after_activate.updated_at > created_at);

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);
        client.deactivate_pet(&pet_id);

        let pet_after_deactivate = client.get_pet(&pet_id).unwrap();
        assert!(pet_after_deactivate.updated_at > pet_after_activate.updated_at);
    }

    #[test]
    fn test_activate_nonexistent_pet_does_not_panic() {
        let (_env, client) = setup_test_env();
        // Should not panic
        client.activate_pet(&999);
    }

    #[test]
    fn test_deactivate_nonexistent_pet_does_not_panic() {
        let (_env, client) = setup_test_env();
        // Should not panic
        client.deactivate_pet(&999);
    }

    // ============================================================================
    // PET OWNERSHIP TRANSFER TESTS
    // ============================================================================

    #[test]
    fn test_transfer_pet_ownership_updates_new_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.transfer_pet_ownership(&pet_id, &new_owner);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.new_owner, new_owner);
        assert_eq!(pet.owner, owner); // Owner should not change yet
    }

    #[test]
    fn test_accept_pet_transfer_changes_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.transfer_pet_ownership(&pet_id, &new_owner);
        client.accept_pet_transfer(&pet_id);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.owner, new_owner);
        assert_eq!(pet.new_owner, new_owner);
    }

    #[test]
    fn test_transfer_ownership_updates_timestamp() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let pet_before = client.get_pet(&pet_id).unwrap();

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);
        client.transfer_pet_ownership(&pet_id, &new_owner);

        let pet_after = client.get_pet(&pet_id).unwrap();
        assert!(pet_after.updated_at > pet_before.updated_at);
    }

    #[test]
    fn test_accept_transfer_updates_timestamp() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let new_owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.transfer_pet_ownership(&pet_id, &new_owner);
        let pet_before_accept = client.get_pet(&pet_id).unwrap();

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);
        client.accept_pet_transfer(&pet_id);

        let pet_after_accept = client.get_pet(&pet_id).unwrap();
        assert!(pet_after_accept.updated_at > pet_before_accept.updated_at);
    }

    #[test]
    fn test_transfer_nonexistent_pet_does_not_panic() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let new_owner = Address::generate(&env);
        // Should not panic
        client.transfer_pet_ownership(&999, &new_owner);
    }

    #[test]
    fn test_accept_transfer_nonexistent_pet_does_not_panic() {
        let (_env, client) = setup_test_env();
        // Should not panic
        client.accept_pet_transfer(&999);
    }

    // ============================================================================
    // PET OWNER REGISTRATION TESTS
    // ============================================================================

    #[test]
    fn test_register_pet_owner_creates_owner_with_correct_data() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = Address::generate(&env);
        let name = String::from_str(&env, "John Doe");
        let email = String::from_str(&env, "john@example.com");
        let emergency = String::from_str(&env, "555-1234");

        client.register_pet_owner(&owner_addr, &name, &email, &emergency);

        let owner = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner.owner_address, owner_addr);
        assert_eq!(owner.name, name);
        assert_eq!(owner.email, email);
        assert_eq!(owner.emergency_contact, emergency);
        assert_eq!(owner.is_pet_owner, true);
    }

    #[test]
    fn test_is_owner_registered_returns_true_for_registered_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = register_test_owner(&env, &client);

        assert_eq!(client.is_owner_registered(&owner_addr), true);
    }

    #[test]
    fn test_is_owner_registered_returns_false_for_unregistered_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let unregistered = Address::generate(&env);

        assert_eq!(client.is_owner_registered(&unregistered), false);
    }

    #[test]
    fn test_register_multiple_owners() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let owner3 = Address::generate(&env);

        let name = String::from_str(&env, "Owner");
        let email = String::from_str(&env, "owner@example.com");
        let emergency = String::from_str(&env, "555-1234");

        client.register_pet_owner(&owner1, &name, &email, &emergency);
        client.register_pet_owner(&owner2, &name, &email, &emergency);
        client.register_pet_owner(&owner3, &name, &email, &emergency);

        assert_eq!(client.is_owner_registered(&owner1), true);
        assert_eq!(client.is_owner_registered(&owner2), true);
        assert_eq!(client.is_owner_registered(&owner3), true);
    }

    // ============================================================================
    // PET OWNER UPDATE TESTS
    // ============================================================================

    #[test]
    fn test_update_owner_profile_updates_all_fields() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = register_test_owner(&env, &client);

        let new_name = String::from_str(&env, "Jane Smith");
        let new_email = String::from_str(&env, "jane@example.com");
        let new_emergency = String::from_str(&env, "555-5678");

        let result =
            client.update_owner_profile(&owner_addr, &new_name, &new_email, &new_emergency);
        assert_eq!(result, true);

        let owner = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner.name, new_name);
        assert_eq!(owner.email, new_email);
        assert_eq!(owner.emergency_contact, new_emergency);
    }

    #[test]
    fn test_update_owner_profile_updates_timestamp() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = register_test_owner(&env, &client);

        let owner_before = client.get_owner_profile(&owner_addr).unwrap();
        let created_at = owner_before.created_at;

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);

        let new_name = String::from_str(&env, "Jane Smith");
        let new_email = String::from_str(&env, "jane@example.com");
        let new_emergency = String::from_str(&env, "555-5678");

        client.update_owner_profile(&owner_addr, &new_name, &new_email, &new_emergency);

        let owner_after = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner_after.created_at, created_at);
        assert!(owner_after.updated_at > created_at);
    }

    #[test]
    fn test_update_unregistered_owner_returns_false() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let unregistered = Address::generate(&env);

        let name = String::from_str(&env, "Jane Smith");
        let email = String::from_str(&env, "jane@example.com");
        let emergency = String::from_str(&env, "555-5678");

        let result = client.update_owner_profile(&unregistered, &name, &email, &emergency);
        assert_eq!(result, false);
    }

    // ============================================================================
    // PET OWNER QUERY TESTS
    // ============================================================================

    #[test]
    fn test_get_owner_profile_returns_none_for_unregistered_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let unregistered = Address::generate(&env);

        let owner = client.get_owner_profile(&unregistered);
        assert_eq!(owner.is_none(), true);
    }

    // ============================================================================
    // INTEGRATION TESTS - COMPLETE WORKFLOWS
    // ============================================================================

    #[test]
    fn test_complete_pet_lifecycle() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();

        // 1. Register owner
        let owner_addr = Address::generate(&env);
        let owner_name = String::from_str(&env, "Alice Johnson");
        let email = String::from_str(&env, "alice@example.com");
        let emergency = String::from_str(&env, "555-9999");
        client.register_pet_owner(&owner_addr, &owner_name, &email, &emergency);
        assert_eq!(client.is_owner_registered(&owner_addr), true);

        // 2. Register pet
        let pet_name = String::from_str(&env, "Charlie");
        let birthday = String::from_str(&env, "2019-03-15");
        let breed = String::from_str(&env, "Beagle");
        let pet_id = client.register_pet(
            &owner_addr,
            &pet_name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        assert_eq!(pet_id, 1);

        // 3. Activate pet
        client.activate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), true);

        // 4. Update pet profile
        let new_name = String::from_str(&env, "Charlie Brown");
        let new_birthday = String::from_str(&env, "2019-04-15");
        let new_breed = String::from_str(&env, "Mixed Beagle");
        client.update_pet_profile(
            &pet_id,
            &new_name,
            &new_birthday,
            &Gender::Male,
            &Species::Dog,
            &new_breed,
        );
        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.name, new_name);

        // 5. Update owner profile
        let new_email = String::from_str(&env, "alice.j@example.com");
        let new_emergency = String::from_str(&env, "555-8888");
        client.update_owner_profile(&owner_addr, &owner_name, &new_email, &new_emergency);
        let owner = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner.email, new_email);

        // 6. Deactivate pet
        client.deactivate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), false);
    }

    #[test]
    fn test_ownership_transfer_workflow() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();

        // 1. Register first owner and pet
        let owner1 = register_test_owner(&env, &client);
        let pet_id = register_test_pet(&env, &client, &owner1);

        // 2. Register second owner
        let owner2 = Address::generate(&env);
        let name2 = String::from_str(&env, "Bob Smith");
        let email2 = String::from_str(&env, "bob@example.com");
        let emergency2 = String::from_str(&env, "555-7777");
        client.register_pet_owner(&owner2, &name2, &email2, &emergency2);

        // 3. Verify initial ownership
        assert_eq!(client.get_pet(&pet_id).unwrap().owner, owner1);

        // 4. Initiate transfer
        client.transfer_pet_ownership(&pet_id, &owner2);
        let pet_after_transfer = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet_after_transfer.owner, owner1); // Still owned by owner1
        assert_eq!(pet_after_transfer.new_owner, owner2); // Transfer initiated

        // 5. Accept transfer
        client.accept_pet_transfer(&pet_id);
        let pet_after_accept = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet_after_accept.owner, owner2); // Now owned by owner2
        assert_eq!(pet_after_accept.new_owner, owner2);
    }

    #[test]
    fn test_multiple_pets_per_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = register_test_owner(&env, &client);

        // Register multiple pets for the same owner
        let pet1_id = register_test_pet(&env, &client, &owner);

        let name2 = String::from_str(&env, "Luna");
        let birthday2 = String::from_str(&env, "2021-07-20");
        let breed2 = String::from_str(&env, "Siamese");
        let pet2_id = client.register_pet(
            &owner,
            &name2,
            &birthday2,
            &Gender::Female,
            &Species::Cat,
            &breed2,
        );

        let name3 = String::from_str(&env, "Tweety");
        let birthday3 = String::from_str(&env, "2022-01-10");
        let breed3 = String::from_str(&env, "Canary");
        let pet3_id = client.register_pet(
            &owner,
            &name3,
            &birthday3,
            &Gender::NotSpecified,
            &Species::Bird,
            &breed3,
        );

        // Verify all pets exist and have correct owners
        assert_eq!(client.get_pet(&pet1_id).unwrap().owner, owner);
        assert_eq!(client.get_pet(&pet2_id).unwrap().owner, owner);
        assert_eq!(client.get_pet(&pet3_id).unwrap().owner, owner);

        // Verify pets have correct species
        assert_eq!(client.get_pet(&pet1_id).unwrap().species, Species::Dog);
        assert_eq!(client.get_pet(&pet2_id).unwrap().species, Species::Cat);
        assert_eq!(client.get_pet(&pet3_id).unwrap().species, Species::Bird);
    }

    #[test]
    fn test_multiple_transfers_same_pet() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();

        // Register three owners
        let owner1 = register_test_owner(&env, &client);
        let owner2 = Address::generate(&env);
        let owner3 = Address::generate(&env);

        let name = String::from_str(&env, "Owner");
        let email = String::from_str(&env, "owner@example.com");
        let emergency = String::from_str(&env, "555-0000");

        client.register_pet_owner(&owner2, &name, &email, &emergency);
        client.register_pet_owner(&owner3, &name, &email, &emergency);

        // Register pet with owner1
        let pet_id = register_test_pet(&env, &client, &owner1);

        // Transfer from owner1 to owner2
        client.transfer_pet_ownership(&pet_id, &owner2);
        client.accept_pet_transfer(&pet_id);
        assert_eq!(client.get_pet(&pet_id).unwrap().owner, owner2);

        // Transfer from owner2 to owner3
        client.transfer_pet_ownership(&pet_id, &owner3);
        client.accept_pet_transfer(&pet_id);
        assert_eq!(client.get_pet(&pet_id).unwrap().owner, owner3);
    }

    #[test]
    fn test_concurrent_pet_registrations() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();

        // Simulate multiple users registering pets concurrently
        let owner1 = Address::generate(&env);
        let owner2 = Address::generate(&env);
        let owner3 = Address::generate(&env);

        let name = String::from_str(&env, "Pet");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Breed");

        let pet1 = client.register_pet(
            &owner1,
            &name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );
        let pet2 = client.register_pet(
            &owner2,
            &name,
            &birthday,
            &Gender::Female,
            &Species::Cat,
            &breed,
        );
        let pet3 = client.register_pet(
            &owner3,
            &name,
            &birthday,
            &Gender::NotSpecified,
            &Species::Bird,
            &breed,
        );

        // Verify unique IDs
        assert_eq!(pet1, 1);
        assert_eq!(pet2, 2);
        assert_eq!(pet3, 3);

        // Verify correct ownership
        assert_eq!(client.get_pet(&pet1).unwrap().owner, owner1);
        assert_eq!(client.get_pet(&pet2).unwrap().owner, owner2);
        assert_eq!(client.get_pet(&pet3).unwrap().owner, owner3);
    }

    // ============================================================================
    // EDGE CASE TESTS
    // ============================================================================

    #[test]
    fn test_pet_with_empty_strings() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let empty_name = String::from_str(&env, "");
        let empty_birthday = String::from_str(&env, "");
        let empty_breed = String::from_str(&env, "");

        let pet_id = client.register_pet(
            &owner,
            &empty_name,
            &empty_birthday,
            &Gender::NotSpecified,
            &Species::Other,
            &empty_breed,
        );

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.name, empty_name);
        assert_eq!(pet.birthday, empty_birthday);
        assert_eq!(pet.breed, empty_breed);
    }

    #[test]
    fn test_owner_with_empty_strings() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = Address::generate(&env);
        let empty_name = String::from_str(&env, "");
        let empty_email = String::from_str(&env, "");
        let empty_emergency = String::from_str(&env, "");

        client.register_pet_owner(&owner_addr, &empty_name, &empty_email, &empty_emergency);

        let owner = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner.name, empty_name);
        assert_eq!(owner.email, empty_email);
        assert_eq!(owner.emergency_contact, empty_emergency);
    }

    #[test]
    fn test_activate_already_active_pet() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        client.activate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), true);

        // Activate again
        client.activate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), true);
    }

    #[test]
    fn test_deactivate_already_inactive_pet() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        assert_eq!(client.is_pet_active(&pet_id), false);

        // Deactivate again
        client.deactivate_pet(&pet_id);
        assert_eq!(client.is_pet_active(&pet_id), false);
    }

    #[test]
    fn test_transfer_to_same_owner() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        // Transfer to self
        client.transfer_pet_ownership(&pet_id, &owner);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.owner, owner);
        assert_eq!(pet.new_owner, owner);
    }

    #[test]
    fn test_register_same_owner_twice() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner_addr = Address::generate(&env);
        let name1 = String::from_str(&env, "John Doe");
        let email1 = String::from_str(&env, "john@example.com");
        let emergency1 = String::from_str(&env, "555-1234");

        client.register_pet_owner(&owner_addr, &name1, &email1, &emergency1);

        // Register again with different data
        let name2 = String::from_str(&env, "Jane Doe");
        let email2 = String::from_str(&env, "jane@example.com");
        let emergency2 = String::from_str(&env, "555-5678");

        client.register_pet_owner(&owner_addr, &name2, &email2, &emergency2);

        // Should have the latest data
        let owner = client.get_owner_profile(&owner_addr).unwrap();
        assert_eq!(owner.name, name2);
        assert_eq!(owner.email, email2);
        assert_eq!(owner.emergency_contact, emergency2);
    }

    #[test]
    fn test_timestamp_ordering() {
        let (env, client) = setup_test_env();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        let pet_id = register_test_pet(&env, &client, &owner);

        let pet = client.get_pet(&pet_id).unwrap();
        assert_eq!(pet.created_at, pet.updated_at);

        env.ledger()
            .with_mut(|li| li.timestamp = li.timestamp + 1000);

        let new_name = String::from_str(&env, "Updated");
        let birthday = String::from_str(&env, "2020-01-01");
        let breed = String::from_str(&env, "Breed");
        client.update_pet_profile(
            &pet_id,
            &new_name,
            &birthday,
            &Gender::Male,
            &Species::Dog,
            &breed,
        );

        let updated_pet = client.get_pet(&pet_id).unwrap();
        assert!(updated_pet.updated_at > updated_pet.created_at);
    }
}
