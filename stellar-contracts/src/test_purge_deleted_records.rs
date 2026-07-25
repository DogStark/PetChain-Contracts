use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String, Vec,
};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let vet = Address::generate(env);
    let owner = Address::generate(env);

    client.init_admin(&admin);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Purge"),
        &String::from_str(env, "LIC-PURGE"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    (client, admin, vet, pet_id)
}

fn add_record(
    client: &PetChainContractClient<'_>,
    env: &Env,
    vet: &Address,
    pet_id: u64,
    diagnosis: &str,
) -> u64 {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(env, diagnosis),
        &String::from_str(env, "Treatment"),
        &Vec::new(env),
        &String::from_str(env, "test notes"),
    )
}

#[test]
fn test_purge_nothing_when_no_deleted_records() {
    let env = Env::default();
    let (client, admin, vet, pet_id) = setup(&env);

    add_record(&client, &env, &vet, pet_id, "Healthy");

    let purged = client.purge_deleted_records(&admin, &pet_id, &30);
    assert_eq!(purged, 0);
}

#[test]
fn test_purge_partial_old_and_new() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let (client, admin, vet, pet_id) = setup(&env);

    let r1 = add_record(&client, &env, &vet, pet_id, "Old1");
    let r2 = add_record(&client, &env, &vet, pet_id, "Old2");
    let r3 = add_record(&client, &env, &vet, pet_id, "Recent");

    client.delete_medical_record(&pet_id, &r1, &vet);
    client.delete_medical_record(&pet_id, &r2, &vet);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_700_000_000 + 20 * 86_400);
    client.delete_medical_record(&pet_id, &r3, &vet);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_700_000_000 + 31 * 86_400);

    let purged = client.purge_deleted_records(&admin, &pet_id, &25);
    assert_eq!(purged, 2);
}

#[test]
fn test_purge_all_old_records() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let (client, admin, vet, pet_id) = setup(&env);

    let r1 = add_record(&client, &env, &vet, pet_id, "Rec1");
    let r2 = add_record(&client, &env, &vet, pet_id, "Rec2");
    let r3 = add_record(&client, &env, &vet, pet_id, "Rec3");

    client.delete_medical_record(&pet_id, &r1, &vet);
    client.delete_medical_record(&pet_id, &r2, &vet);
    client.delete_medical_record(&pet_id, &r3, &vet);

    env.ledger()
        .with_mut(|l| l.timestamp = 1_700_000_000 + 60 * 86_400);

    let purged = client.purge_deleted_records(&admin, &pet_id, &30);
    assert_eq!(purged, 3);
}

#[test]
#[should_panic]
fn test_purge_requires_admin() {
    let env = Env::default();
    let (client, _admin, vet, pet_id) = setup(&env);

    let stranger = Address::generate(&env);
    client.purge_deleted_records(&stranger, &pet_id, &30);
}
