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

    let admins = soroban_sdk::vec![&env, admin1.clone(), admin2.clone()];
    client.init_multisig(&admin1, &admins, &1u32);

    use crate::ProposalAction;
    let new_admins = soroban_sdk::vec![&env, new_admin.clone()];
    let action = ProposalAction::ChangeAdmin((new_admins, 1u32));
    let proposal_id = client.propose_action(&admin1, &action, &3600u64);
    client.execute_proposal(&proposal_id);

    let result = client.get_admins();
    assert_eq!(result.len(), 1);
    assert!(result.contains(new_admin));
}

use crate::*;

fn setup_client() -> (Env, PetChainContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_single_admin_initialization_succeeds() {
    let (env, client) = setup_client();
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
    let (env, client) = setup_client();
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
    let (env, client) = setup_client();
    let admin = Address::generate(&env);
    let other_admin = Address::generate(&env);

    client.init_admin(&admin);
    client.init_admin(&other_admin);
}

#[test]
#[should_panic(expected = "Admin already set")]
fn test_multisig_reinitialization_rejected_after_single_admin() {
    let (env, client) = setup_client();
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
    let (env, client) = setup_client();
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
    let (env, client) = setup_client();
    let admin = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());

    client.init_multisig(&admin, &admins, &0u32);
}

#[test]
#[should_panic(expected = "Invalid threshold")]
fn test_multisig_initialization_rejects_threshold_above_admin_count() {
    let (env, client) = setup_client();
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
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let new_wasm_hash = BytesN::from_array(&env, &[1u8; 32]);

    // Try to upgrade without initializing admin - should panic with typed error
    client.upgrade_contract(&new_wasm_hash);
}

#[test]
#[should_panic]

fn test_propose_upgrade_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);
    let new_wasm_hash = BytesN::from_array(&env, &[2u8; 32]);

    // Try to propose upgrade without initializing admin - should panic with typed error
    client.propose_upgrade(&proposer, &new_wasm_hash);
}

#[test]
#[should_panic]

fn test_approve_upgrade_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // Try to approve upgrade without initializing admin - should panic with typed error
    client.approve_upgrade(&1u64);
}

#[test]
#[should_panic]

fn test_migrate_version_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // Try to migrate version without initializing admin - should panic with typed error
    client.migrate_version(&1u32, &0u32, &0u32);
}

#[test]
#[should_panic]

fn test_verify_vet_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet first
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );

    // Try to verify vet without initializing admin - should panic with typed error
    client.verify_vet(&admin, &vet);
}

#[test]
#[should_panic]

fn test_revoke_vet_license_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    // Register vet first
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Jones"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "Dentistry"),
    );

    // Try to revoke license without initializing admin - should panic with typed error
    client.revoke_vet_license(&admin, &vet);
}

#[test]
#[should_panic]

fn test_propose_action_without_admin_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let proposer = Address::generate(&env);

    // Create a simple proposal action
    let action = ProposalAction::VerifyVet(Address::generate(&env));

    // Try to propose action without initializing admin - should panic with typed error
    client.propose_action(&proposer, &action, &3600u64);
}

#[test]
fn test_admin_methods_work_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let vet = Address::generate(&env);

    // Initialize admin
    client.init_admin(&admin);

    // Register vet
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Wilson"),
        &String::from_str(&env, "LIC-003"),
        &String::from_str(&env, "General"),
    );

    // Now verifying vet should work
    let result = client.verify_vet(&admin, &vet);
    assert!(result);

    // Vet should be verified
    assert!(client.is_verified_vet(&vet));
}

#[test]
fn test_multisig_admin_methods_work_after_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    admins.push_back(admin2.clone());

    // Initialize multisig admin
    client.init_multisig(&admin, &admins, &1u32);

    // Now proposing action should work
    let action = ProposalAction::VerifyVet(Address::generate(&env));

    let proposal_id = client.propose_action(&admin, &action, &3600u64);
    assert_eq!(proposal_id, 1u64);

    // Approving with the other admin should work
    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
fn test_get_admins_single_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

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
    let mut admins_vec = soroban_sdk::Vec::new(&env);
    admins_vec.push_back(admin1.clone());
    admins_vec.push_back(admin2.clone());

    client.init_multisig(&admin1, &admins_vec, &2u32);

    let admins = client.get_admins();
    assert_eq!(admins.len(), 2);
    assert_eq!(client.get_admin_threshold(), 2u32);
}

#[test]
fn test_get_admins_no_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admins = client.get_admins();
    assert_eq!(admins.len(), 0);
}
