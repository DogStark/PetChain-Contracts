use crate::*;
use soroban_sdk::{
    testutils::Address as _,
    Env, String, Vec, Address,
};

#[test]
fn test_public_emergency_access() {
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
        &PrivacyLevel::Private, // Even if private, emergency access should work
    );

    let mut contacts = Vec::new(&env);
    contacts.push_back(EmergencyContact {
        name: String::from_str(&env, "Emergency Name"),
        phone: String::from_str(&env, "555-1234"),
        email: String::from_str(&env, "emergency@test.com"),
        relationship: String::from_str(&env, "Friend"),
        is_primary: true,
    });

    let mut allergies = Vec::new(&env);
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Peanuts"),
        severity: String::from_str(&env, "High"),
        is_critical: true,
    });
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Pollen"),
        severity: String::from_str(&env, "Low"),
        is_critical: false,
    });

    client.set_emergency_contacts(&pet_id, &contacts, &allergies, &String::from_str(&env, "Critical medical condition!"));

    // Set responder (no owner auth needed)
    env.mock_all_auths(); // Clear auths
    let info = client.get_emergency_info(&pet_id);

    assert_eq!(info.pet_id, pet_id);
    assert_eq!(info.species, String::from_str(&env, "Dog"));
    assert_eq!(info.emergency_contacts.len(), 1);
    assert_eq!(info.emergency_contacts.get(0).unwrap().phone, String::from_str(&env, "555-1234"));
}

#[test]
fn test_emergency_data_filtering() {
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
        &PrivacyLevel::Private,
    );

    let mut allergies = Vec::new(&env);
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Penicillin"),
        severity: String::from_str(&env, "Critical"),
        is_critical: true,
    });
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Dust"),
        severity: String::from_str(&env, "Mild"),
        is_critical: false,
    });

    client.set_emergency_contacts(&pet_id, &Vec::new(&env), &allergies, &String::from_str(&env, "Needs daily medication"));

    let info = client.get_emergency_info(&pet_id);

    // Should only have the critical allergy
    assert_eq!(info.allergies.len(), 1);
    assert_eq!(info.allergies.get(0).unwrap().name, String::from_str(&env, "Penicillin"));
    assert!(info.allergies.get(0).unwrap().is_critical);
    
    // Critical alert should be present
    assert_eq!(info.critical_alerts.len(), 1);
}

#[test]
fn test_emergency_logging() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Initial log state (could check storage directly)
    // Actually, I'll use the client to call get_emergency_info multiple times
    client.get_emergency_info(&pet_id);
    client.get_emergency_info(&pet_id);
    client.get_emergency_info(&pet_id);

    // Verify logs in storage
    let log_key = DataKey::EmergencyAccessLogs(pet_id);
    let logs: Vec<EmergencyAccessLog> = env.as_contract(&contract_id, || {
        env.storage().persistent().get(&log_key).unwrap_or(Vec::new(&env))
    });

    assert_eq!(logs.len(), 3);
    assert_eq!(logs.get(0).unwrap().pet_id, pet_id);
    assert!(logs.get(0).unwrap().timestamp > 0);
}
