use crate::*;
use soroban_sdk::{testutils::Address as _, Env};

fn valid_contact(
    env: &Env,
    name: &str,
    phone: &str,
    email: &str,
    relationship: &str,
    is_primary: bool,
) -> EmergencyContact {
    EmergencyContact {
        name: String::from_str(env, name),
        phone: String::from_str(env, phone),
        email: String::from_str(env, email),
        relationship: String::from_str(env, relationship),
        is_primary,
    }
}

fn setup_pet_with_contacts(
    env: &Env,
    client: &PetChainContractClient,
    owner: &Address,
) -> (u64, soroban_sdk::Vec<EmergencyContact>) {
    let pet_id = client.register_pet(
        owner,
        &String::from_str(env, "Buddy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Golden Retriever"),
        &String::from_str(env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let mut contacts = soroban_sdk::Vec::new(env);
    contacts.push_back(valid_contact(
        env,
        "Jane Doe",
        "555-0100",
        "jane@example.com",
        "Vet",
        true,
    ));

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(env),
        &String::from_str(env, ""),
    );

    (pet_id, contacts)
}

#[test]
fn test_emergency_contacts_add() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    // Owner can read their own emergency contacts
    let retrieved = client.get_emergency_contacts(&pet_id, &owner);
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

    let mut contacts = soroban_sdk::Vec::new(&env);
    contacts.push_back(valid_contact(
        &env,
        "Primary Contact",
        "555-1000",
        "primary@example.com",
        "Owner",
        true,
    ));
    contacts.push_back(valid_contact(
        &env,
        "Backup Contact",
        "555-2000",
        "backup@example.com",
        "Spouse",
        false,
    ));
    contacts.push_back(valid_contact(
        &env,
        "Vet Clinic",
        "555-3000",
        "vet@clinic.com",
        "Veterinarian",
        false,
    ));

    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );

    let retrieved = client.get_emergency_contacts(&pet_id, &owner);
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
fn test_approved_responder_can_read_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let responder = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    // Owner approves the responder
    client.add_emergency_responder(&pet_id, &responder);

    // Approved responder can now read contacts
    let retrieved = client.get_emergency_contacts(&pet_id, &responder);
    assert_eq!(retrieved.len(), 1);
    assert_eq!(
        retrieved.get(0).unwrap().phone,
        String::from_str(&env, "555-0100")
    );
}

#[test]
#[should_panic]
fn test_unauthorized_address_cannot_read_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    // Stranger has NOT been added as a responder — must panic
    client.get_emergency_contacts(&pet_id, &stranger);
}

#[test]
#[should_panic]
fn test_revoked_responder_cannot_read_contacts() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let responder = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    client.add_emergency_responder(&pet_id, &responder);
    client.remove_emergency_responder(&pet_id, &responder);

    // Revoked responder must no longer have access
    client.get_emergency_contacts(&pet_id, &responder);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_empty_emergency_contacts_rejected() {
    let env = Env::default();
    PetChainContract::validate_emergency_contacts(&env, &soroban_sdk::Vec::new(&env));
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_contact_without_primary_rejected() {
    let env = Env::default();
    let mut contacts = soroban_sdk::Vec::new(&env);
    contacts.push_back(valid_contact(
        &env,
        "Backup Contact",
        "555-2200",
        "backup@example.com",
        "Neighbor",
        false,
    ));

    PetChainContract::validate_emergency_contacts(&env, &contacts);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_contact_with_empty_name_rejected() {
    let env = Env::default();
    let mut contacts = soroban_sdk::Vec::new(&env);
    contacts.push_back(valid_contact(
        &env,
        "",
        "555-3300",
        "primary@example.com",
        "Owner",
        true,
    ));

    PetChainContract::validate_emergency_contacts(&env, &contacts);
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_contact_with_empty_phone_rejected() {
    let env = Env::default();
    let mut contacts = soroban_sdk::Vec::new(&env);
    contacts.push_back(valid_contact(
        &env,
        "Primary Contact",
        "",
        "primary@example.com",
        "Owner",
        true,
    ));

    PetChainContract::validate_emergency_contacts(&env, &contacts);
}

#[test]
fn test_emergency_contacts_update() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    let mut new_contacts = soroban_sdk::Vec::new(&env);
    new_contacts.push_back(valid_contact(
        &env,
        "John Smith",
        "555-9999",
        "john@example.com",
        "Brother",
        true,
    ));

    client.set_emergency_contacts(
        &pet_id,
        &new_contacts,
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, "Updated notes"),
    );

    let retrieved = client.get_emergency_contacts(&pet_id, &owner);
    assert_eq!(retrieved.len(), 1);
    assert_eq!(
        retrieved.get(0).unwrap().name,
        String::from_str(&env, "John Smith")
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")]
fn test_emergency_contacts_empty_rejection() {
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

    client.set_emergency_contacts(
        &pet_id,
        &soroban_sdk::Vec::new(&env),
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );
}

#[test]
#[should_panic]
fn test_set_emergency_contacts_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let _stranger = Address::generate(&env);
    let (pet_id, contacts) = setup_pet_with_contacts(&env, &client, &owner);
    env.as_contract(&contract_id, || {
        client.set_emergency_contacts(
            &pet_id,
            &contacts,
            &soroban_sdk::Vec::new(&env),
            &String::from_str(&env, "Hacked!"),
        );
    });
}

#[test]
fn test_get_emergency_info_authorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let (pet_id, _) = setup_pet_with_contacts(&env, &client, &owner);

    let mut allergies = soroban_sdk::Vec::new(&env);
    allergies.push_back(Allergy {
        name: String::from_str(&env, "Peanuts"),
        severity: String::from_str(&env, "High"),
        is_critical: true,
    });

    client.set_emergency_contacts(
        &pet_id,
        &client.get_emergency_contacts(&pet_id, &owner),
        &allergies,
        &String::from_str(&env, "Critical medical history"),
    );

    let info = client.get_emergency_info(&pet_id, &owner);
    assert_eq!(info.pet_id, pet_id);
    assert_eq!(info.allergies.len(), 1);
    assert_eq!(info.critical_alerts.len(), 1);
    assert_eq!(
        info.critical_alerts.get(0).unwrap(),
        String::from_str(&env, "Critical medical history")
    );
}

#[test]
#[should_panic]
fn test_set_emergency_contacts_pet_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    client.set_emergency_contacts(
        &999u64,
        &soroban_sdk::Vec::new(&env),
        &soroban_sdk::Vec::new(&env),
        &String::from_str(&env, ""),
    );
}
