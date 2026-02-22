use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn register_test_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
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
    )
}

#[test]
fn test_register_pet_and_get_profile() {
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
        &Some(String::from_str(&env, "982000123456789")),
        &PrivacyLevel::Public,
    );

    let pet = client.get_pet(&pet_id).unwrap();
    assert_eq!(pet.id, pet_id);
    assert_eq!(pet.owner, owner);
    assert_eq!(pet.name, String::from_str(&env, "Buddy"));
    assert_eq!(pet.breed, String::from_str(&env, "Golden Retriever"));
    assert_eq!(pet.color, String::from_str(&env, "Golden"));
    assert_eq!(pet.weight, 25u32);
}

#[test]
fn test_grant_custody() {
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
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "feed"));
    permissions.push_back(String::from_str(&env, "walk"));

    let custody =
        client.grant_temporary_custody(&pet_id, &custodian, &100u64, &200u64, &permissions);

    assert!(custody.is_active);
    assert_eq!(custody.pet_id, pet_id);
    assert_eq!(custody.owner, owner);
    assert_eq!(custody.custodian, custodian);
    assert_eq!(custody.permissions.len(), 2);
}

#[test]
fn test_auto_expiry() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    env.ledger().with_mut(|l| l.timestamp = 1000);

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

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "feed"));

    client.grant_temporary_custody(&pet_id, &custodian, &900u64, &1100u64, &permissions);
    assert!(client.is_custody_valid(&pet_id));

    env.ledger().with_mut(|l| l.timestamp = 1200);
    assert!(!client.is_custody_valid(&pet_id));
}

#[test]
fn test_limited_permissions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2022-06-10"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Tricolor"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let custodian = Address::generate(&env);
    let mut permissions = Vec::new(&env);
    permissions.push_back(String::from_str(&env, "medicate"));

    let custody = client.grant_temporary_custody(&pet_id, &custodian, &10u64, &20u64, &permissions);

    assert_eq!(custody.permissions.len(), 1);
    assert_eq!(
        custody.permissions.get(0).unwrap(),
        String::from_str(&env, "medicate")
    );
}

#[test]
fn test_log_all_actions() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let grantee = Address::generate(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let meds = Vec::new(&env);
    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Healthy"),
        &String::from_str(&env, "No treatment"),
        &meds,
    );
    client.get_medical_record(&record_id);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);
    client.revoke_access(&pet_id, &grantee);

    let logs = client.get_access_logs(&pet_id);
    assert!(logs.len() >= 4);

    let mut has_read = false;
    let mut has_write = false;
    let mut has_grant = false;
    let mut has_revoke = false;
    for log in logs.iter() {
        match log.action {
            AccessAction::Read => has_read = true,
            AccessAction::Write => has_write = true,
            AccessAction::Grant => has_grant = true,
            AccessAction::Revoke => has_revoke = true,
        }
    }

    assert!(has_read);
    assert!(has_write);
    assert!(has_grant);
    assert!(has_revoke);
}

#[test]
fn test_log_immutability() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let meds = Vec::new(&env);
    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Checkup"),
        &String::from_str(&env, "Stable"),
        &String::from_str(&env, "Observe"),
        &meds,
    );
    let logs_before = client.get_access_logs(&pet_id);
    let initial_count = logs_before.len();

    client.get_medical_record(&record_id);
    let logs_after = client.get_access_logs(&pet_id);
    assert!(logs_after.len() > initial_count);

    assert_eq!(
        logs_before.get(0).unwrap().id,
        logs_after.get(0).unwrap().id
    );
    assert_eq!(
        logs_before.get(0).unwrap().details,
        logs_after.get(0).unwrap().details
    );
}

#[test]
fn test_log_retrieval_by_pet_id() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let pet_1 = register_test_pet(&client, &env, &owner);
    let pet_2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &6u32,
        &None,
        &PrivacyLevel::Public,
    );

    let meds = Vec::new(&env);
    client.add_medical_record(
        &pet_1,
        &vet,
        &String::from_str(&env, "Exam"),
        &String::from_str(&env, "Good"),
        &String::from_str(&env, "None"),
        &meds,
    );

    let logs_pet_1 = client.get_access_logs(&pet_1);
    let logs_pet_2 = client.get_access_logs(&pet_2);

    assert!(!logs_pet_1.is_empty());
    assert_eq!(logs_pet_2.len(), 0);
}

// --- TREATMENT HISTORY TESTS ---

#[test]
fn test_add_treatment_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Alice"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &Species::Dog,
        &Gender::Male,
        &0u64,
        &String::from_str(&env, "Labrador"),
    );

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
    );
    client.verify_vet(&admin, &vet);

    let treatment_id = client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Surgery,
        &1_000_000u64,
        &String::from_str(&env, "Splenectomy performed successfully"),
        &Some(500_00i128),
        &String::from_str(&env, "Full recovery expected"),
    );

    assert_eq!(treatment_id, 1);

    let treatment = client.get_treatment(&treatment_id).unwrap();
    assert_eq!(treatment.pet_id, pet_id);
    assert_eq!(treatment.treatment_type, TreatmentType::Surgery);
    assert_eq!(treatment.vet_address, vet);
    assert_eq!(treatment.cost, Some(500_00i128));
    assert_eq!(
        treatment.outcome,
        String::from_str(&env, "Full recovery expected")
    );
}

#[test]
fn test_get_treatment_history_multiple() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Alice"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &Species::Cat,
        &Gender::Female,
        &0u64,
        &String::from_str(&env, "Siamese"),
    );

    client.register_vet(&vet, &String::from_str(&env, "Dr. Lee"), &String::from_str(&env, "LIC-002"));
    client.verify_vet(&admin, &vet);

    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Routine,
        &1_000_000u64,
        &String::from_str(&env, "Annual checkup"),
        &Some(100_00i128),
        &String::from_str(&env, "Healthy"),
    );

    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Emergency,
        &2_000_000u64,
        &String::from_str(&env, "Swallowed foreign object"),
        &Some(800_00i128),
        &String::from_str(&env, "Recovered after observation"),
    );

    client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Therapy,
        &3_000_000u64,
        &String::from_str(&env, "Post-op physiotherapy"),
        &None,
        &String::from_str(&env, "Ongoing"),
    );

    let history = client.get_treatment_history(&pet_id);
    assert_eq!(history.len(), 3);
    assert_eq!(history.get(0).unwrap().treatment_type, TreatmentType::Routine);
    assert_eq!(history.get(1).unwrap().treatment_type, TreatmentType::Emergency);
    assert_eq!(history.get(2).unwrap().treatment_type, TreatmentType::Therapy);
}

#[test]
fn test_filter_treatments_by_type() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Bob"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &Species::Dog,
        &Gender::Female,
        &0u64,
        &String::from_str(&env, "Beagle"),
    );

    client.register_vet(&vet, &String::from_str(&env, "Dr. Park"), &String::from_str(&env, "LIC-003"));
    client.verify_vet(&admin, &vet);

    // Add 2 surgeries and 1 routine
    client.add_treatment(
        &pet_id, &vet, &TreatmentType::Surgery, &1_000_000u64,
        &String::from_str(&env, "Leg surgery"), &Some(1200_00i128),
        &String::from_str(&env, "Success"),
    );
    client.add_treatment(
        &pet_id, &vet, &TreatmentType::Routine, &2_000_000u64,
        &String::from_str(&env, "Checkup"), &Some(80_00i128),
        &String::from_str(&env, "Healthy"),
    );
    client.add_treatment(
        &pet_id, &vet, &TreatmentType::Surgery, &3_000_000u64,
        &String::from_str(&env, "Tooth extraction"), &Some(400_00i128),
        &String::from_str(&env, "Healed"),
    );

    let surgeries = client.get_treatments_by_type(&pet_id, &TreatmentType::Surgery);
    assert_eq!(surgeries.len(), 2);

    let routines = client.get_treatments_by_type(&pet_id, &TreatmentType::Routine);
    assert_eq!(routines.len(), 1);

    let therapies = client.get_treatments_by_type(&pet_id, &TreatmentType::Therapy);
    assert_eq!(therapies.len(), 0);
}

#[test]
fn test_treatment_outcome_tracking() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Carol"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &Species::Dog,
        &Gender::Male,
        &0u64,
        &String::from_str(&env, "German Shepherd"),
    );

    client.register_vet(&vet, &String::from_str(&env, "Dr. Jones"), &String::from_str(&env, "LIC-004"));
    client.verify_vet(&admin, &vet);

    let treatment_id = client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Emergency,
        &1_000_000u64,
        &String::from_str(&env, "Trauma from accident"),
        &Some(2000_00i128),
        &String::from_str(&env, "Stable after surgery"),
    );

    let treatment = client.get_treatment(&treatment_id).unwrap();
    assert_eq!(
        treatment.outcome,
        String::from_str(&env, "Stable after surgery")
    );
    assert_eq!(treatment.treatment_type, TreatmentType::Emergency);
}

#[test]
fn test_treatment_history_empty_for_unknown_pet() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let history = client.get_treatment_history(&999u64);
    assert_eq!(history.len(), 0);
}

#[test]
fn test_add_treatment_without_cost() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Dave"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &Species::Cat,
        &Gender::Male,
        &0u64,
        &String::from_str(&env, "Persian"),
    );

    client.register_vet(&vet, &String::from_str(&env, "Dr. Kim"), &String::from_str(&env, "LIC-005"));
    client.verify_vet(&admin, &vet);

    let treatment_id = client.add_treatment(
        &pet_id,
        &vet,
        &TreatmentType::Other,
        &1_000_000u64,
        &String::from_str(&env, "Behavioral assessment"),
        &None,
        &String::from_str(&env, "Pending follow-up"),
    );

    let treatment = client.get_treatment(&treatment_id).unwrap();
    assert_eq!(treatment.cost, None);
    assert_eq!(treatment.treatment_type, TreatmentType::Other);
}

#[test]
#[should_panic(expected = "Veterinarian not verified")]
fn test_add_treatment_unverified_vet_panics() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let unverified_vet = Address::generate(&env);

    client.initialize(&admin);
    client.register_pet_owner(&owner, &String::from_str(&env, "Eve"));

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Nala"),
        &Species::Dog,
        &Gender::Female,
        &0u64,
        &String::from_str(&env, "Poodle"),
    );

    // Vet registered but NOT verified
    client.register_vet(
        &unverified_vet,
        &String::from_str(&env, "Fake Vet"),
        &String::from_str(&env, "FAKE-001"),
    );

    client.add_treatment(
        &pet_id,
        &unverified_vet,
        &TreatmentType::Surgery,
        &1_000_000u64,
        &String::from_str(&env, "Unauthorised surgery"),
        &None,
        &String::from_str(&env, "N/A"),
    );
}