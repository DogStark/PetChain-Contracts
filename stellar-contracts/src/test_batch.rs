use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec, String};

#[test]
fn test_register_pets_batch() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let mut pets_data = Vec::new(&env);
    pets_data.push_back(PetData {
        name: String::from_str(&env, "Buddy"),
        species: String::from_str(&env, "Dog"),
        breed: String::from_str(&env, "Golden Retriever"),
    });
    pets_data.push_back(PetData {
        name: String::from_str(&env, "Misty"),
        species: String::from_str(&env, "Cat"),
        breed: String::from_str(&env, "Siamese"),
    });

    let pet_ids = client.register_pets_batch(&owner, &pets_data);

    assert_eq!(pet_ids.len(), 2);
    assert_eq!(pet_ids.get(0).unwrap(), 1);
    assert_eq!(pet_ids.get(1).unwrap(), 2);

    let pet1 = client.get_pet(&1).unwrap();
    assert_eq!(pet1.id, 1);
    // Name is encrypted, but get_pet returns it as "PROTECTED" or similar in metadata if we follow encryption helpers
    // Actually, get_pet (PetProfile) uses decrypt_sensitive_data which we mocked to return ciphertext directly
    assert_eq!(pet1.species, Species::Dog);

    let pet2 = client.get_pet(&2).unwrap();
    assert_eq!(pet2.id, 2);
    assert_eq!(pet2.species, Species::Cat);

    // Verify ownership
    let owner_pets = client.get_all_pets_by_owner(&owner);
    assert_eq!(owner_pets.len(), 2);
}

#[test]
#[should_panic]
fn test_register_pets_batch_unauthorized() {
    let env = Env::default();
    // No mock_all_auths for the wrong owner call

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let malicious = Address::generate(&env);

    let mut pets_data = Vec::new(&env);
    pets_data.push_back(PetData {
        name: String::from_str(&env, "Buddy"),
        species: String::from_str(&env, "Dog"),
        breed: String::from_str(&env, "Golden Retriever"),
    });

    // This should panic because malicious is calling on behalf of owner without auth
    env.as_contract(&contract_id, || {
        client.register_pets_batch(&owner, &pets_data);
    });
}
