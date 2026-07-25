use crate::{ContractError, PetChainContract, PetChainContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Error, Vec,
};

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

// ======================================================
// Upgrade proposal expiry tests (Issue #818)
// ======================================================

#[test]
fn test_execute_upgrade_within_expiry_window() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    // Approve the proposal
    client.approve_upgrade_proposal(&admin1, &proposal_id);

    // Execute within the 7-day window (advance 3 days)
    env.ledger().with_mut(|l| l.timestamp = 3 * 86400);

    client.execute_upgrade(&admin1, &proposal_id);

    let proposal = client.get_upgrade_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_execute_upgrade_past_expiry_window_fails() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    // Approve the proposal
    client.approve_upgrade_proposal(&admin1, &proposal_id);

    // Advance past the 7-day expiry window
    env.ledger().with_mut(|l| l.timestamp = 8 * 86400);

    // Attempt to execute — must fail with ProposalExpired
    let result = client.try_execute_upgrade(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalExpired as u32,
        )))
    );
}

#[test]
fn test_create_new_proposal_after_old_one_expires() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash1 = BytesN::from_array(&env, &[1u8; 32]);
    let proposal_id_1 = client.propose_upgrade(&admin1, &hash1, &7);
    assert_eq!(proposal_id_1, 1);

    // Advance past expiry
    env.ledger().with_mut(|l| l.timestamp = 8 * 86400);

    // Create a new proposal after old one expired (use zero hash to skip actual WASM update)
    let hash2 = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id_2 = client.propose_upgrade(&admin1, &hash2, &7);
    assert_eq!(proposal_id_2, 2);

    // New proposal has correct expiry
    let proposal = client.get_upgrade_proposal(&proposal_id_2).unwrap();
    assert_eq!(proposal.expires_at, 8 * 86400 + 7 * 86400);
    assert!(!proposal.approved);
    assert!(!proposal.executed);

    // Approve and execute the new proposal within window
    client.approve_upgrade_proposal(&admin1, &proposal_id_2);
    client.execute_upgrade(&admin1, &proposal_id_2);

    let proposal = client.get_upgrade_proposal(&proposal_id_2).unwrap();
    assert!(proposal.executed);
}

#[test]
fn test_propose_upgrade_stores_correct_data() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[1u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    let proposal = client.get_upgrade_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.id, proposal_id);
    assert_eq!(proposal.new_wasm_hash, hash);
    assert!(!proposal.approved);
    assert!(!proposal.executed);
    assert_eq!(proposal.expires_at, 7 * 86400); // 7 days from timestamp 0
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

    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[1u8; 32]), &7);
    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[2u8; 32]), &7);
    client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[3u8; 32]), &7);

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
        client.propose_upgrade(&admin1, &BytesN::from_array(&env, &[i; 32]), &7);
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
fn test_execute_unapproved_proposal_fails() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    // Try to execute without approving
    let result = client.try_execute_upgrade(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalNotApproved as u32,
        )))
    );
}

#[test]
fn test_execute_upgrade_cannot_execute_twice() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    client.approve_upgrade_proposal(&admin1, &proposal_id);
    client.execute_upgrade(&admin1, &proposal_id);

    // Second execution must fail
    let result = client.try_execute_upgrade(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalAlreadyExecuted as u32,
        )))
    );
}

#[test]
fn test_approve_expired_upgrade_proposal_fails() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let hash = BytesN::from_array(&env, &[0u8; 32]);
    let proposal_id = client.propose_upgrade(&admin1, &hash, &7);

    // Advance past expiry
    env.ledger().with_mut(|l| l.timestamp = 8 * 86400);

    let result = client.try_approve_upgrade_proposal(&admin1, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::from_contract_error(
            ContractError::ProposalExpired as u32,
        )))
    );
}
