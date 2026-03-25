use crate::{PetChainContract, PetChainContractClient, ProposalAction};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec};

fn setup<'a>(env: &'a Env) -> (PetChainContractClient<'a>, Address, Address) {
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

    let new_wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    let action = ProposalAction::UpgradeContract(new_wasm_hash.clone());

    // Propose
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    assert_eq!(proposal_id, 1);

    // Verify proposal stored correctly
    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(!proposal.executed);
    assert_eq!(proposal.approvals.len(), 1);

    // Second admin approves
    client.approve_proposal(&admin2, &proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.approvals.len(), 2);

    // Execute — calls env.deployer().update_current_contract_wasm internally.
    // In the test environment this is a no-op but must not panic.
    client.execute_proposal(&proposal_id);

    let proposal = client.get_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
#[should_panic(expected = "Proposal already executed")]
fn test_upgrade_proposal_cannot_execute_twice() {
    let env = Env::default();
    let (client, admin1, admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[2u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    client.approve_proposal(&admin2, &proposal_id);
    client.execute_proposal(&proposal_id);
    client.execute_proposal(&proposal_id); // must panic
}

#[test]
#[should_panic(expected = "Threshold not met")]
fn test_upgrade_proposal_threshold_not_met() {
    let env = Env::default();
    let (client, admin1, _admin2) = setup(&env);

    let action = ProposalAction::UpgradeContract(BytesN::from_array(&env, &[3u8; 32]));
    let proposal_id = client.propose_action(&admin1, &action, &3600);
    // Only 1 of 2 required approvals — must panic
    client.execute_proposal(&proposal_id);
}
