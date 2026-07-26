use crate::{ContractError, PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient, Address, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let admins = vec![env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    (client, admin1, admin2)
}

#[test]
fn admin_activity_log_populated_after_multiple_actions() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, _admin2) = setup(&env);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-1"),
        &String::from_str(&env, "Surgery"),
    );

    client.set_global_storage_quota(&admin1, &500u64);
    client.verify_vet(&admin1, &vet);

    let page = client.get_admin_activity_log(&admin1, &admin1, &0u32, &10u32);
    assert_eq!(page.total, 2);
    assert_eq!(page.items.len(), 2);
    assert_eq!(page.items.get(0).unwrap().action, String::from_str(&env, "set_global_storage_quota"));
    assert_eq!(page.items.get(1).unwrap().action, String::from_str(&env, "verify_vet"));
}

#[test]
fn admin_activity_log_paginates_matching_entries() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, _admin2) = setup(&env);

    for quota in [100u64, 200, 300] {
        client.set_global_storage_quota(&admin1, &quota);
    }

    let first_page = client.get_admin_activity_log(&admin1, &admin1, &0u32, &2u32);
    assert_eq!(first_page.total, 3);
    assert_eq!(first_page.items.len(), 2);

    let second_page = client.get_admin_activity_log(&admin1, &admin1, &1u32, &2u32);
    assert_eq!(second_page.total, 3);
    assert_eq!(second_page.items.len(), 1);
}

#[test]
fn admin_activity_log_only_returns_entries_for_the_requested_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, admin2) = setup(&env);

    client.set_global_storage_quota(&admin1, &100u64);
    client.set_global_storage_quota(&admin2, &200u64);

    let admin1_page = client.get_admin_activity_log(&admin1, &admin1, &0u32, &10u32);
    assert_eq!(admin1_page.total, 1);

    let admin2_page = client.get_admin_activity_log(&admin2, &admin2, &0u32, &10u32);
    assert_eq!(admin2_page.total, 1);
}

#[test]
fn admin_activity_log_blocks_non_admin_callers() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, _admin2) = setup(&env);

    client.set_global_storage_quota(&admin1, &100u64);

    let outsider = Address::generate(&env);
    let result = client.try_get_admin_activity_log(&outsider, &admin1, &0u32, &10u32);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), ContractError::Unauthorized.into());
}
