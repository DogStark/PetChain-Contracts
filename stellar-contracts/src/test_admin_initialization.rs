#![cfg(test)]

use soroban_sdk::{testutils::Address as _, vec, Address, Env};

use crate::PetChainContract;
use crate::PetChainContractClient;

#[test]
fn test_get_admins_after_init_multisig() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone(), admin3.clone()];
    client.init_multisig(&admin1, &admins, &2u32);

    let result = client.get_admins();
    assert_eq!(result.len(), 3);
    assert!(result.contains(admin1));
    assert!(result.contains(admin2));
    assert!(result.contains(admin3));
}

#[test]
fn test_get_admin_threshold_after_init_multisig() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &2u32);

    let threshold = client.get_admin_threshold();
    assert_eq!(threshold, 2u32);
}

#[test]
fn test_get_admins_empty_before_init() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let result = client.get_admins();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_get_admin_threshold_zero_before_init() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let threshold = client.get_admin_threshold();
    assert_eq!(threshold, 0u32);
}

#[test]
fn test_get_admins_reflects_change_admin_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    // Propose and execute a ChangeAdmin action
    use crate::ProposalAction;
    let new_admins = vec![&env, new_admin.clone()];
    let action = ProposalAction::ChangeAdmin((new_admins, 1u32));
    let proposal_id = client.propose_action(&admin1, &action, &3600u64);
    client.execute_proposal(&proposal_id);

    let result = client.get_admins();
    assert_eq!(result.len(), 1);
    assert!(result.contains(new_admin));

    let threshold = client.get_admin_threshold();
    assert_eq!(threshold, 1u32);
}
