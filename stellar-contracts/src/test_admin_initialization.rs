use soroban_sdk::{testutils::Address as _, vec, Address, BytesN, Env, String};

use crate::{PetChainContract, PetChainContractClient, ProposalAction};

fn setup_client(env: &Env) -> PetChainContractClient<'static> {
    let contract_id = env.register_contract(None, PetChainContract);
    PetChainContractClient::new(env, &contract_id)
}

#[test]
fn test_get_admins_after_init_multisig() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

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
    let client = setup_client(&env);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &2u32);

    assert_eq!(client.get_admin_threshold(), 2u32);
}

#[test]
fn test_get_admins_empty_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    assert_eq!(client.get_admins().len(), 0);
    let result = client.get_admins();
    assert_eq!(result.len(), 0);
}

#[test]
fn test_get_admin_threshold_zero_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let threshold = client.get_admin_threshold();
    assert_eq!(threshold, 0u32);
}

#[test]
fn test_get_admins_reflects_change_admin_proposal() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let new_admin = Address::generate(&env);


    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    let new_admins = vec![&env, new_admin.clone()];
    let action = ProposalAction::ChangeAdmin((new_admins, 1u32));
    let proposal_id = client.propose_action(&admin1, &action, &3600u64);
    client.execute_proposal(&proposal_id);

    let result = client.get_admins();
    assert_eq!(result.len(), 1);
    assert!(result.contains(new_admin));
}

#[test]
fn test_single_admin_initialization_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Single Admin"),
        &String::from_str(&env, "LIC-ADMIN-001"),
        &String::from_str(&env, "General"),
    );

    assert!(client.verify_vet(&admin, &vet));
}

#[test]
fn test_multisig_initialization_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2);

    client.init_multisig(&admin, &admins, &2u32);

    let action = ProposalAction::VerifyVet(Address::generate(&env));
    let proposal_id = client.propose_action(&admin, &action, &3600u64);

    assert_eq!(proposal_id, 1u64);
}

#[test]
#[should_panic(expected = "Admin already set")]
fn test_single_admin_reinitialization_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let other_admin = Address::generate(&env);

    client.init_admin(&admin);
    client.init_admin(&other_admin);
}

#[test]
#[should_panic(expected = "Admin already set")]
fn test_multisig_reinitialization_rejected_after_single_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2);

    client.init_admin(&admin);
    client.init_multisig(&admin, &admins, &1u32);
}

#[test]
#[should_panic(expected = "Admin already set")]
fn test_single_admin_reinitialization_rejected_after_multisig() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2.clone());

    client.init_multisig(&admin, &admins, &1u32);
    client.init_admin(&admin2);
}

#[test]
#[should_panic(expected = "Invalid threshold")]
fn test_multisig_initialization_rejects_zero_threshold() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());

    client.init_multisig(&admin, &admins, &0u32);
}

#[test]
#[should_panic(expected = "Invalid threshold")]
fn test_multisig_initialization_rejects_threshold_above_admin_count() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);
    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2);

    client.init_multisig(&admin, &admins, &3u32);
}

#[test]
#[should_panic]
fn test_upgrade_contract_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let new_wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.upgrade_contract(&new_wasm_hash);
}

#[test]
#[should_panic]
fn test_propose_upgrade_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let proposer = Address::generate(&env);
    let new_wasm_hash = BytesN::from_array(&env, &[2u8; 32]);
    client.propose_upgrade(&proposer, &new_wasm_hash);
}

#[test]
#[should_panic]
fn test_approve_upgrade_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    client.approve_upgrade(&1u64);
}

#[test]
#[should_panic]
fn test_migrate_version_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let caller = Address::generate(&env);
    client.migrate_version(&caller, &1u32, &0u32, &0u32);
}

#[test]
#[should_panic]
fn test_verify_vet_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );

    client.verify_vet(&admin, &vet);
}

#[test]
#[should_panic]
fn test_revoke_vet_license_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "Dentistry"),
    );

    client.revoke_vet_license(&admin, &vet);
}

#[test]
#[should_panic]
fn test_propose_action_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let proposer = Address::generate(&env);
    let action = ProposalAction::VerifyVet(Address::generate(&env));
    client.propose_action(&proposer, &action, &3600u64);
}

#[test]
fn test_admin_methods_work_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Wilson"),
        &String::from_str(&env, "LIC-003"),
        &String::from_str(&env, "General"),
    );

    assert!(client.verify_vet(&admin, &vet));
    assert!(client.is_verified_vet(&vet));
}

#[test]
fn test_multisig_admin_methods_work_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2.clone());

    client.init_multisig(&admin, &admins, &1u32);

    let action = ProposalAction::VerifyVet(Address::generate(&env));
    let proposal_id = client.propose_action(&admin, &action, &3600u64);
    assert_eq!(proposal_id, 1u64);

    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
fn test_get_admins_single_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let admins = client.get_admins();
    assert_eq!(admins.len(), 1);
    assert_eq!(admins.get(0).unwrap(), admin);
    assert_eq!(client.get_admin_threshold(), 1u32);
}

#[test]
fn test_get_admins_multisig() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let admins = vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

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

#[test]
fn test_get_admins_no_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let client = setup_client(&env);

    assert_eq!(client.get_admins().len(), 0);
}
