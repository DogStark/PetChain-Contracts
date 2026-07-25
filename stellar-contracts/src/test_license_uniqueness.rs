use crate::{ContractError, PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.init_admin(&admin);
    (client, admin)
}

#[test]
fn test_first_registration_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let vet = Address::generate(&env);
    let result = client.register_vet(
        &vet,
        &String::from_str(&env, "Dr Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );
    assert!(result);
}

#[test]
#[should_panic]
fn test_duplicate_license_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    client.register_vet(
        &vet1,
        &String::from_str(&env, "Dr Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );
    // Same license number, different address — should panic with LicenseAlreadyRegistered
    client.register_vet(
        &vet2,
        &String::from_str(&env, "Dr Jones"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Dermatology"),
    );
}

#[test]
fn test_different_license_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _) = setup(&env);
    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);
    let r1 = client.register_vet(
        &vet1,
        &String::from_str(&env, "Dr Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );
    let r2 = client.register_vet(
        &vet2,
        &String::from_str(&env, "Dr Jones"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "Dermatology"),
    );
    assert!(r1 && r2);
}
