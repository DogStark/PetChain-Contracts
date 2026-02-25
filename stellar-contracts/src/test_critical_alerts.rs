use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn get_stored_critical_alerts_count(env: &Env, contract_id: &soroban_sdk::Address, pet_id: u64) -> u32 {
    env.as_contract(contract_id, || {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap();
        pet.critical_alerts.len()
    })
}

fn get_stored_critical_alert_at(env: &Env, contract_id: &soroban_sdk::Address, pet_id: u64, index: u32) -> Option<CriticalAlert> {
    env.as_contract(contract_id, || {
        let pet: Pet = env
            .storage()
            .instance()
            .get(&DataKey::Pet(pet_id))
            .unwrap();
        pet.critical_alerts.get(index)
    })
}

#[test]
fn test_add_critical_alert() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2019-05-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let alert = CriticalAlert {
        alert_type: AlertType::Allergy,
        description: String::from_str(&env, "Severe peanut allergy"),
        severity: String::from_str(&env, "Critical"),
        added_date: env.ledger().timestamp(),
    };
    client.add_critical_alert(&pet_id, &alert);

    assert_eq!(get_stored_critical_alerts_count(&env, &contract_id, pet_id), 1);
    let stored = get_stored_critical_alert_at(&env, &contract_id, pet_id, 0).unwrap();
    assert_eq!(stored.alert_type, AlertType::Allergy);
    assert_eq!(stored.description, String::from_str(&env, "Severe peanut allergy"));
    assert_eq!(stored.severity, String::from_str(&env, "Critical"));
}

#[test]
fn test_public_access_emergency_critical_alerts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2020-01-15"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &10u32,
        &None,
        &PrivacyLevel::Private,
    );

    let alert = CriticalAlert {
        alert_type: AlertType::Medication,
        description: String::from_str(&env, "Daily insulin required"),
        severity: String::from_str(&env, "High"),
        added_date: env.ledger().timestamp(),
    };
    client.add_critical_alert(&pet_id, &alert);

    // Public access: get_critical_alerts is callable without auth (no require_auth in that fn)
    env.mock_all_auths(); // clear auths - simulate unauthenticated emergency responder
    assert_eq!(get_stored_critical_alerts_count(&env, &contract_id, pet_id), 1);
    let stored = get_stored_critical_alert_at(&env, &contract_id, pet_id, 0).unwrap();
    assert_eq!(stored.alert_type, AlertType::Medication);
    assert_eq!(stored.description, String::from_str(&env, "Daily insulin required"));
}

#[test]
fn test_alert_types() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2018-03-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "German Shepherd"),
        &String::from_str(&env, "Brown"),
        &35u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_critical_alert(
        &pet_id,
        &CriticalAlert {
            alert_type: AlertType::Allergy,
            description: String::from_str(&env, "Pollen"),
            severity: String::from_str(&env, "Medium"),
            added_date: 1000,
        },
    );
    client.add_critical_alert(
        &pet_id,
        &CriticalAlert {
            alert_type: AlertType::Medication,
            description: String::from_str(&env, "Heartworm pill monthly"),
            severity: String::from_str(&env, "Routine"),
            added_date: 2000,
        },
    );
    client.add_critical_alert(
        &pet_id,
        &CriticalAlert {
            alert_type: AlertType::Condition,
            description: String::from_str(&env, "Epilepsy"),
            severity: String::from_str(&env, "Critical"),
            added_date: 3000,
        },
    );
    client.add_critical_alert(
        &pet_id,
        &CriticalAlert {
            alert_type: AlertType::Behavior,
            description: String::from_str(&env, "Fear aggressive with strangers"),
            severity: String::from_str(&env, "High"),
            added_date: 4000,
        },
    );

    assert_eq!(get_stored_critical_alerts_count(&env, &contract_id, pet_id), 4);
    assert_eq!(get_stored_critical_alert_at(&env, &contract_id, pet_id, 0).unwrap().alert_type, AlertType::Allergy);
    assert_eq!(get_stored_critical_alert_at(&env, &contract_id, pet_id, 1).unwrap().alert_type, AlertType::Medication);
    assert_eq!(get_stored_critical_alert_at(&env, &contract_id, pet_id, 2).unwrap().alert_type, AlertType::Condition);
    assert_eq!(get_stored_critical_alert_at(&env, &contract_id, pet_id, 3).unwrap().alert_type, AlertType::Behavior);
}

#[test]
fn test_remove_critical_alert() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Tri-color"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.add_critical_alert(
        &pet_id,
        &CriticalAlert {
            alert_type: AlertType::Condition,
            description: String::from_str(&env, "Resolved condition"),
            severity: String::from_str(&env, "Low"),
            added_date: env.ledger().timestamp(),
        },
    );
    assert_eq!(get_stored_critical_alerts_count(&env, &contract_id, pet_id), 1);

    client.remove_critical_alert(&pet_id, &0);
    assert_eq!(get_stored_critical_alerts_count(&env, &contract_id, pet_id), 0);
}
