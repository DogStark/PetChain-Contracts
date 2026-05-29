use crate::{PetChainContract, PetChainContractClient, ProposalAction};
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, BytesN, Env, Vec};

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

// --- Missing coverage: expired proposals, duplicate approval, non-admin ---

#[test]
#[should_panic]
fn test_approve_expired_proposal_panics() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    // expires_in = 100 seconds
    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &100);

    // Advance time past expiry
    env.ledger().with_mut(|l| l.timestamp = 200);

    // admin2 tries to approve an expired proposal — must panic
    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
#[should_panic]
fn test_execute_expired_proposal_panics() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &100);

    // Approve before expiry
    client.approve_proposal(&admin2, &proposal_id);

    // Advance time past expiry before executing
    env.ledger().with_mut(|l| l.timestamp = 200);

    // Execute after expiry — must panic
    client.execute_proposal(&proposal_id);
}

#[test]
#[should_panic]
fn test_duplicate_approval_panics() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    // admin2 approves once
    client.approve_proposal(&admin2, &proposal_id);
    // admin2 approves again — must panic
    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_propose_action() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let non_admin = Address::generate(&env);
    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    client.propose_action(&non_admin, &action, &3600);
}

#[test]
#[should_panic]
fn test_non_admin_cannot_approve_proposal() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    let non_admin = Address::generate(&env);
    client.approve_proposal(&non_admin, &proposal_id);
}

// --- migrate_v1_to_v2 tests ---

#[test]
fn test_migrate_v1_to_v2_bumps_version() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let before = client.get_version();
    assert_eq!(before.major, 1);

    client.migrate_v1_to_v2(&admin1);

    let after = client.get_version();
    assert_eq!(after.major, 2);
    assert_eq!(after.minor, 0);
    assert_eq!(after.patch, 0);
}

#[test]
fn test_migrate_v1_to_v2_idempotent() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    client.migrate_v1_to_v2(&admin1);
    client.migrate_v1_to_v2(&admin1); // second call must be a no-op

    let version = client.get_version();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);
    assert_eq!(version.patch, 0);
}

#[test]
fn test_migrate_v1_to_v2_does_not_downgrade() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    // Manually set version to 3.x
    client.set_version(&admin1, &3, &0, &0);
    client.migrate_v1_to_v2(&admin1); // must not downgrade

    let version = client.get_version();
    assert_eq!(version.major, 3);
}

#[test]
fn test_migrate_storage_bumps_version() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let before = client.get_storage_version();
    assert_eq!(before.major, 1);

    client.migrate_storage(&admin1, &1, &0, &0, &2, &0, &0);

    let after = client.get_storage_version();
    assert_eq!(after.major, 2);
}

#[test]
fn test_migrate_storage_idempotent() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    client.migrate_storage(&admin1, &1, &0, &0, &2, &0, &0);
    // second call must be no-op
    client.migrate_storage(&admin1, &1, &0, &0, &2, &0, &0);

    let version = client.get_storage_version();
    assert_eq!(version.major, 2);
}

#[test]
#[should_panic]
fn test_migrate_storage_non_admin_panics() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let non_admin = Address::generate(&env);
    client.migrate_storage(&non_admin, &1, &0, &0, &2, &0, &0);
}

#[test]
#[should_panic]
fn test_migrate_v1_to_v2_non_admin_panics() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    let non_admin = Address::generate(&env);
    client.migrate_v1_to_v2(&non_admin);
}

// --- TIMELOCK AND VETO TESTS ---

#[test]
fn test_proposal_enters_timelock_after_quorum() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    // Proposal should be in Pending state initially
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::Pending);

    // After second approval (quorum reached), should enter TimelockPending
    client.approve_proposal(&admin2, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::TimelockPending);
    assert!(proposal.timelock_end > 0);
}

#[test]
#[should_panic]
fn test_execution_rejected_before_timelock_expiry() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Try to execute immediately (should fail - timelock not expired)
    client.execute_proposal(&proposal_id);
}

#[test]
fn test_execution_allowed_after_timelock_expiry() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Get timelock end time
    let proposal = client.get_proposal(&proposal_id).unwrap();
    let timelock_end = proposal.timelock_end;

    // Advance time past timelock
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    // Now execution should succeed
    client.execute_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
    assert_eq!(proposal.state, crate::ProposalState::Executed);
}

#[test]
fn test_veto_during_timelock_cancels_proposal() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Verify proposal is in timelock
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::TimelockPending);

    // Admin1 vetoes during timelock
    client.veto_proposal(&admin1, &proposal_id);

    // Proposal should now be vetoed
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::Vetoed);
    assert_eq!(proposal.veto_count, 1);
}

#[test]
#[should_panic]
fn test_veto_after_timelock_rejected() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Get timelock end time
    let proposal = client.get_proposal(&proposal_id).unwrap();
    let timelock_end = proposal.timelock_end;

    // Advance time past timelock
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    // Try to veto after timelock expired (should fail)
    client.veto_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic]
fn test_execution_rejected_if_vetoed() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Veto the proposal
    client.veto_proposal(&admin1, &proposal_id);

    // Get timelock end time
    let proposal = client.get_proposal(&proposal_id).unwrap();
    let timelock_end = proposal.timelock_end;

    // Advance time past timelock
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    // Try to execute vetoed proposal (should fail)
    client.execute_proposal(&proposal_id);
}

#[test]
#[should_panic]
fn test_timelock_config_enforces_minimum_24_hours() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    // Try to set timelock less than 24 hours (should fail)
    client.set_timelock_config(&admin1, &3600, &true); // 1 hour
}

#[test]
fn test_timelock_config_accepts_24_hours() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    // Set timelock to exactly 24 hours (should succeed)
    client.set_timelock_config(&admin1, &86400, &true);

    let config = client.get_timelock_config();
    assert_eq!(config.timelock_duration, 86400);
    assert!(config.enabled);
}

#[test]
fn test_timelock_config_can_be_disabled() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    // Disable timelock
    client.set_timelock_config(&admin1, &86400, &false);

    let config = client.get_timelock_config();
    assert!(!config.enabled);
}

#[test]
fn test_proposal_skips_timelock_when_disabled() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    // Disable timelock
    client.set_timelock_config(&admin1, &86400, &false);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Proposal should be directly Executable (not TimelockPending)
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::Executable);

    // Should be able to execute immediately
    client.execute_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_get_proposal_veto_count() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Initially no vetoes
    let veto_count = client.get_proposal_veto_count(&proposal_id);
    assert_eq!(veto_count, 0);

    // After veto
    client.veto_proposal(&admin1, &proposal_id);
    let veto_count = client.get_proposal_veto_count(&proposal_id);
    assert_eq!(veto_count, 1);
}

#[test]
fn test_has_admin_vetoed() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Initially admin1 has not vetoed
    assert!(!client.has_admin_vetoed(&proposal_id, &admin1));

    // After veto
    client.veto_proposal(&admin1, &proposal_id);
    assert!(client.has_admin_vetoed(&proposal_id, &admin1));

    // admin2 still has not vetoed
    assert!(!client.has_admin_vetoed(&proposal_id, &admin2));
}

#[test]
#[should_panic]
fn test_veto_prevents_duplicate_veto_from_same_admin() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // First veto succeeds
    client.veto_proposal(&admin1, &proposal_id);

    // Second veto from same admin should fail
    client.veto_proposal(&admin1, &proposal_id);
}

#[test]
fn test_multiple_admins_can_veto() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Both admins veto
    client.veto_proposal(&admin1, &proposal_id);
    client.veto_proposal(&admin2, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.veto_count, 2);
    assert_eq!(proposal.state, crate::ProposalState::Vetoed);
}

#[test]
fn test_timelock_duration_applies_correctly() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    // Set custom timelock duration (48 hours)
    let custom_duration = 172800u64;
    client.set_timelock_config(&admin1, &custom_duration, &true);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    let now = env.ledger().timestamp();
    client.approve_proposal(&admin2, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.timelock_end, now + custom_duration);
}

#[test]
#[should_panic]
fn test_veto_cannot_happen_on_pending_proposal() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    // Try to veto while still pending (should fail)
    client.veto_proposal(&admin1, &proposal_id);
}

#[test]
#[should_panic]
fn test_approval_rejected_on_vetoed_proposal() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Veto the proposal
    client.veto_proposal(&admin1, &proposal_id);

    // Try to approve vetoed proposal (should fail)
    client.approve_proposal(&admin2, &proposal_id);
}

#[test]
fn test_proposal_state_transitions() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[0u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);

    // State 1: Pending
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::Pending);

    // State 2: TimelockPending (after quorum)
    client.approve_proposal(&admin2, &proposal_id);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::TimelockPending);

    // State 3: Executable (after timelock expires)
    let timelock_end = proposal.timelock_end;
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    // State 4: Executed
    client.execute_proposal(&proposal_id);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::Executed);
}

#[test]
fn test_coverage_get_timelock_config_default() {
    let env = Env::default();
    let (client, _admin1, _admin2) = setup(&env);

    // Get config without setting (should return default)
    let config = client.get_timelock_config();
    assert_eq!(config.timelock_duration, 86400); // 24 hours
    assert!(config.enabled);
}

#[test]
fn test_coverage_verify_vet_proposal_with_timelock() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let vet_address = Address::generate(&env);
    let action = ProposalAction::VerifyVet(vet_address);
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Verify enters timelock
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::TimelockPending);

    // Advance time and execute
    let timelock_end = proposal.timelock_end;
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    client.execute_proposal(&proposal_id);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_coverage_change_admin_proposal_with_timelock() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let mut new_admins = Vec::new(&env);
    new_admins.push_back(admin1.clone());
    new_admins.push_back(admin2.clone());

    let action = ProposalAction::ChangeAdmin((new_admins, 2));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);

    // Verify enters timelock
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.state, crate::ProposalState::TimelockPending);

    // Advance time and execute
    let timelock_end = proposal.timelock_end;
    env.ledger().with_mut(|ledger| {
        ledger.set_timestamp(timelock_end + 1);
    });

    client.execute_proposal(&proposal_id);
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}
