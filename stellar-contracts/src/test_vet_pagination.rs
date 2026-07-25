use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

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
        &String::from_str(env, "Dr. Page"),
        &String::from_str(env, "LIC-PAGE"),
        &String::from_str(env, "General"),
    );
    client.verify_vet(&admin, &vet);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "PaginatedPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(env, "Persian"),
        &String::from_str(env, "White"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    (client, admin, vet, pet_id)
}

fn add_record(client: &PetChainContractClient<'_>, env: &Env, vet: &Address, pet_id: u64) {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(env, "Diagnosis"),
        &String::from_str(env, "Treatment"),
        &Vec::new(env),
        &String::from_str(env, "notes"),
    );
}

#[test]
fn test_first_page() {
    let env = Env::default();
    let (client, _admin, vet, pet_id) = setup(&env);

    for _ in 0..10 {
        add_record(&client, &env, &vet, pet_id);
    }

    let page = client.get_vet_treatment_history(&vet, &0, &5);
    assert_eq!(page.items.len(), 5);
    assert_eq!(page.total, 10);
    assert_eq!(page.page, 0);
    assert_eq!(page.page_size, 5);
}

#[test]
fn test_last_page_partial() {
    let env = Env::default();
    let (client, _admin, vet, pet_id) = setup(&env);

    for _ in 0..7 {
        add_record(&client, &env, &vet, pet_id);
    }

    let page = client.get_vet_treatment_history(&vet, &1, &5);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.total, 7);
    assert_eq!(page.page, 1);
}

#[test]
fn test_empty_page_out_of_bounds() {
    let env = Env::default();
    let (client, _admin, vet, pet_id) = setup(&env);

    add_record(&client, &env, &vet, pet_id);

    let page = client.get_vet_treatment_history(&vet, &5, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total, 1);
}

#[test]
fn test_no_records() {
    let env = Env::default();
    let (client, _admin, vet, _pet_id) = setup(&env);

    let page = client.get_vet_treatment_history(&vet, &0, &10);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.total, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #12)")]
fn test_page_size_exceeds_cap() {
    let env = Env::default();
    let (client, _admin, vet, _pet_id) = setup(&env);

    client.get_vet_treatment_history(&vet, &0, &51);
}
