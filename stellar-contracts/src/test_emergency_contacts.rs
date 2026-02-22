use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_emergency_contacts_add() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Jane Doe"),
        phone: String::from_str(&env, "555-0100"),
        email: String::from_str(&env, "jane@example.com"),
        relationship: String::from_str(&env, "Vet"),
        is_primary: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );
    let retrieved = client.get_emergency_contacts(&pet_id);
    assert_eq!(retrieved.len(), 1);
    assert_eq!(
        retrieved.get(0).unwrap().name,
        String::from_str(&env, "Jane Doe")
    );
    assert_eq!(
        retrieved.get(0).unwrap().phone,
        String::from_str(&env, "555-0100")
    );
    assert_eq!(
        retrieved.get(0).unwrap().email,
        String::from_str(&env, "jane@example.com")
    );
}

#[test]
fn test_emergency_contacts_multiple() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Primary Contact"),
        phone: String::from_str(&env, "555-1000"),
        email: String::from_str(&env, "primary@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Backup Contact"),
        phone: String::from_str(&env, "555-2000"),
        email: String::from_str(&env, "backup@example.com"),
        relationship: String::from_str(&env, "Spouse"),
        is_primary: false,
    });
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Vet Clinic"),
        phone: String::from_str(&env, "555-3000"),
        email: String::from_str(&env, "vet@clinic.com"),
        relationship: String::from_str(&env, "Veterinarian"),
        is_primary: false,
    });

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );
    let retrieved = client.get_emergency_contacts(&pet_id);
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved.get(0).unwrap().is_primary, true);
    assert_eq!(
        retrieved.get(1).unwrap().relationship,
        String::from_str(&env, "Spouse")
    );
    assert_eq!(
        retrieved.get(2).unwrap().name,
        String::from_str(&env, "Vet Clinic")
    );
}

#[test]
fn test_emergency_contacts_public_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Emergency Contact"),
        phone: String::from_str(&env, "555-9999"),
        email: String::from_str(&env, "emergency@example.com"),
        relationship: String::from_str(&env, "Owner"),
        is_primary: true,
    });
    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );

    // get_emergency_contacts is publicly accessible - no auth required for emergency responders
    let retrieved = client.get_emergency_contacts(&pet_id);
    assert_eq!(retrieved.len(), 1);
    assert_eq!(
        retrieved.get(0).unwrap().phone,
        String::from_str(&env, "555-9999")
    );
}
