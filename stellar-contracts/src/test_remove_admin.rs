use crate::{ContractError, PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::Address as _, vec, Address, Env};

fn setup_multisig(env: &Env) -> (PetChainContractClient, Address, Address, Address) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let admin3 = Address::generate(env);

    let admins = vec![env, admin1.clone(), admin2.clone(), admin3.clone()];
    client.init_multisig(&admin1, &admins, &2u32);

    (client, admin1, admin2, admin3)
}

// Boundary: with 3 admins and threshold=2, removing one admin leaves 2 == threshold → ok.
// But removing a second would leave 1 < 2 → invalid.
#[test]
fn test_remove_admin_boundary_violation_with_2_of_3() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin1, admin2, _admin3) = setup_multisig(&env);

    // 3 admins, threshold=2, removing one leaves 2 which equals threshold — should succeed.
    let proposal_id = client.remove_admin(&admin1, &admin2);
    assert!(proposal_id > 0);
}

// Threshold violation: 2 admins threshold=2, remove one → 1 < 2 → should panic.
#[test]
fn test_remove_admin_threshold_violated() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &2u32);

    let result = client.try_remove_admin(&admin1, &admin2);
    assert!(result.is_err());
    let err = result.unwrap_err().unwrap();
    assert_eq!(err, ContractError::InvalidThreshold.into());
}

// Non-boundary: 4 admins, threshold=2, remove one → 3 >= 2 → ok.
#[test]
fn test_remove_admin_safe_case() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let admin4 = Address::generate(&env);

    let admins = vec![
        &env,
        admin1.clone(),
        admin2.clone(),
        admin3.clone(),
        admin4.clone(),
    ];
    client.init_multisig(&admin1, &admins, &2u32);

    let proposal_id = client.remove_admin(&admin1, &admin4);
    assert!(proposal_id > 0);
}

// With threshold=1 and 2 admins, removing one is safe (1 >= 1).
#[test]
fn test_remove_admin_threshold_one() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    let proposal_id = client.remove_admin(&admin1, &admin2);
    assert!(proposal_id > 0);
}
