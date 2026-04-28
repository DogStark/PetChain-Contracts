use crate::{PetChainContract, PetChainContractClient, ProposalAction};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

fn setup(env: &Env) -> (PetChainContractClient, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let admin1 = Address::generate(env);
    let admin2 = Address::generate(env);
    let mut admins = Vec::new(env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());

    client.init_multisig(&admin1, &admins, &2);

    (client, admin1, admin2)
}

#[test]
fn test_upgrade_contract_proposal_lifecycle() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    // Using [0u8; 32] as a mock hash that the contract will skip in tests
    let new_wasm_hash = BytesN::from_array(&env, &[0u8; 32]);
    let action = ProposalAction::UpgradeContract(new_wasm_hash.clone());

    // Propose
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    assert_eq!(proposal_id, 1);

    // Approve
    client.approve_proposal(&admin2, &proposal_id);

    // Execute — calls env.deployer().update_current_contract_wasm internally.
    // In our tests, we skip the actual update if the hash is 0.
    client.execute_proposal(&proposal_id);

    // Verify status
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
#[should_panic]
fn test_upgrade_proposal_cannot_execute_twice() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);
    client.execute_proposal(&proposal_id);
    client.execute_proposal(&proposal_id); // must panic
}

#[test]
#[should_panic]
fn test_upgrade_proposal_threshold_not_met() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    // Only 1 of 2 required approvals — must panic
    client.execute_proposal(&proposal_id);
}

// --- Tests verifying admins[1] can perform upgrade/migration ---

#[test]
fn test_admin2_can_propose_upgrade() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));

    // admin2 (index 1) proposes
    let proposal_id = client.propose_action(&admin2, &action, &3600);
    assert_eq!(proposal_id, 1);

    // admin1 approves to meet threshold of 2
    client.approve_proposal(&admin1, &proposal_id);
    client.execute_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_admin2_can_migrate_version() {
    let env = Env::default();
    let (client, _admin1, admin2) = setup(&env);

    // admin2 (index 1) calls migrate_version directly
    client.migrate_version(&admin2, &2, &0, &0);

    let version = client.get_version();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_admin2_can_approve_upgrade_proposal() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    // admin2 (index 1) approves
    client.approve_proposal(&admin2, &proposal_id);
    client.execute_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_migrate_version() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let non_admin = Address::generate(&env);
    client.migrate_version(&non_admin, &2, &0, &0);
}

#[test]
fn test_get_upgrade_proposal_returns_correct_data() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash);

    let proposal = client.get_upgrade_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.new_wasm_hash, hash);
    assert!(!proposal.approved);
    assert!(!proposal.executed);
}

#[test]
fn test_get_upgrade_proposal_nonexistent_returns_none() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    assert!(client.get_upgrade_proposal(&999u64).is_none());
}

#[test]
fn test_list_upgrade_proposals_returns_all() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[1u8; 32]));
    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[2u8; 32]));
    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[3u8; 32]));

    let list = client.list_upgrade_proposals(&0u64, &10u32);
    assert_eq!(list.len(), 3);
    assert_eq!(list.get(0).unwrap().id, 1);
    assert_eq!(list.get(2).unwrap().id, 3);
}

#[test]
fn test_list_upgrade_proposals_pagination() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    for i in 1u8..=5 {
        client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[i; 32]));
    }

    let page1 = client.list_upgrade_proposals(&0u64, &2u32);
    assert_eq!(page1.len(), 2);
    assert_eq!(page1.get(0).unwrap().id, 1);

    let page2 = client.list_upgrade_proposals(&2u64, &2u32);
    assert_eq!(page2.len(), 2);
    assert_eq!(page2.get(0).unwrap().id, 3);

    let page3 = client.list_upgrade_proposals(&4u64, &2u32);
    assert_eq!(page3.len(), 1);
    assert_eq!(page3.get(0).unwrap().id, 5);
}

#[test]
fn test_list_upgrade_proposals_empty() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let list = client.list_upgrade_proposals(&0u64, &10u32);
    assert_eq!(list.len(), 0);
}

#[test]
fn test_get_version_default() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let version = client.get_version();
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_set_version_by_admin() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    client.set_version(&admin1, &2, &3, &4);

    let version = client.get_version();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 3);
    assert_eq!(version.patch, 4);
}

#[test]
#[should_panic]
fn test_set_version_non_admin_fails() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let non_admin = Address::generate(&env);
    client.set_version(&non_admin, &2, &0, &0);
}

#[test]
fn test_version_readable_publicly() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    client.set_version(&admin1, &3, &1, &5);

    // Any address can read version (no auth required)
    let version = client.get_version();
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 1);
    assert_eq!(version.patch, 5);
}
