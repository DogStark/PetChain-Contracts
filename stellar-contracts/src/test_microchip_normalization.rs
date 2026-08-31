use crate::{DataKey, Gender, Pet, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let owner = Address::generate(env);
    (client, owner)
}

fn register(client: &PetChainContractClient, env: &Env, owner: &Address, chip: &str) -> u64 {
    client.register_pet(owner, &String::from_str(env, "Buddy"), &String::from_str(env, "2020-01-01"), &Gender::Male, &Species::Dog, &String::from_str(env, "Labrador"), &String::from_str(env, "Brown"), &10, &Some(String::from_str(env, chip)), &PrivacyLevel::Public)
}

#[test]
fn canonicalizes_case_whitespace_and_separators() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = setup(&env);
    let id = register(&client, &env, &owner, "  ab-12:cd.34 ");
    let pet: Pet = env.storage().instance().get(&DataKey::Pet(id)).unwrap();
    assert_eq!(pet.microchip_id, Some(String::from_str(&env, "AB12CD34")));
}

#[test]
#[should_panic]
fn canonical_collisions_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = setup(&env);
    register(&client, &env, &owner, "AB-12");
    register(&client, &env, &owner, " ab12 ");
}

#[test]
#[should_panic]
fn unicode_lookalikes_are_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, owner) = setup(&env);
    register(&client, &env, &owner, "ＡＢ１２");
}
