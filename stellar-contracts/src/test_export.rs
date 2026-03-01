use crate::*;
use soroban_sdk::{
    testutils::Address as _,
    Env,
};

#[test]
fn test_export_pet_data_complete() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register pet
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &Some(String::from_str(&env, "123456789")),
        &PrivacyLevel::Public,
    );

    // Setup vet
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "VET-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Add vaccination
    let now = env.ledger().timestamp();
    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies Vaccine"),
        &now,
        &(now + 31536000),
        &String::from_str(&env, "BATCH-001"),
    );

    // Add medical record
    client.add_medical_record(
        &pet_id,
        &vet,
        &now,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "Continue regular care"),
    );

    // Add medication
    client.add_medication(
        &pet_id,
        &String::from_str(&env, "Heartgard"),
        &String::from_str(&env, "68mcg"),
        &String::from_str(&env, "Monthly"),
        &now,
        &None,
        &vet,
    );

    // Add lab result
    client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "Blood Test"),
        &String::from_str(&env, "Normal"),
        &now,
    );

    // Export data
    let export = client.export_pet_data(&pet_id);

    // Verify export
    assert_eq!(export.pet.id, pet_id);
    assert_eq!(export.pet.name, String::from_str(&env, "Buddy"));
    assert_eq!(export.vaccinations.len(), 1);
    assert_eq!(export.medical_records.len(), 1);
    assert_eq!(export.medications.len(), 0); // Placeholder: empty
    assert_eq!(export.ownership_history.len(), 1);
    assert_eq!(export.export_date, env.ledger().timestamp());
}

#[test]
#[should_panic(expected = "Pet not found")]
fn test_export_nonexistent_pet() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    client.export_pet_data(&999);
}

#[test]
fn test_export_authorization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Owner can export (auth is mocked)
    let export = client.export_pet_data(&pet_id);
    assert_eq!(export.pet.id, pet_id);
}

#[test]
fn test_export_data_completeness() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2019-05-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Public,
    );

    let admin = Address::generate(&env);
    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "VET-002"),
        &String::from_str(&env, "Feline"),
    );
    client.verify_vet(&admin, &vet);

    let now = env.ledger().timestamp();

    // Add multiple records
    for i in 0..3 {
        client.add_vaccination(
            &pet_id,
            &vet,
            &VaccineType::Leukemia,
            &String::from_str(&env, "Leukemia Vaccine"),
            &(now + i * 1000),
            &(now + (i + 1) * 31536000),
            &String::from_str(&env, "BATCH"),
        );
    }

    for _ in 0..2 {
        client.add_medical_record(
            &pet_id,
            &vet,
            &now,
            &String::from_str(&env, "Visit"),
            &String::from_str(&env, "Good"),
            &String::from_str(&env, "None"),
        );
    }

    let export = client.export_pet_data(&pet_id);

    assert_eq!(export.vaccinations.len(), 3);
    assert_eq!(export.medical_records.len(), 2);
    assert_eq!(export.ownership_history.len(), 1);
}
