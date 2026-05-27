use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup_test_env<'a>(
    env: &'a Env,
) -> (
    PetChainContractClient<'a>,
    Address,
    Address,
    Address,
    Address,
) {
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let owner = Address::generate(env);
    let signer1 = Address::generate(env);
    let signer2 = Address::generate(env);
    let new_owner = Address::generate(env);

    client.init_admin(&owner);

    (client, owner, signer1, signer2, new_owner)
}

fn register_test_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Golden"),
        &30,
        &None,
        &PrivacyLevel::Public,
    )
}

#[test]
fn test_configure_multisig() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    let result = client.configure_multisig(&pet_id, &signers, &2);
    assert!(result);

    let config = client.get_multisig_config(&pet_id);
    assert!(config.is_some());

    let config = config.unwrap();
    assert_eq!(config.pet_id, pet_id);
    assert_eq!(config.threshold, 2);
    assert_eq!(config.signers.len(), 3);
    assert!(config.enabled);
}

#[test]
#[should_panic]
fn test_configure_multisig_invalid_threshold_zero() {
    let env = Env::default();
    let (client, owner, signer1, _, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());

    client.configure_multisig(&pet_id, &signers, &0);
}

#[test]
#[should_panic]
fn test_configure_multisig_invalid_threshold_exceeds() {
    let env = Env::default();
    let (client, owner, signer1, _, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());

    client.configure_multisig(&pet_id, &signers, &3);
}

#[test]
#[should_panic]
fn test_configure_multisig_owner_not_in_signers() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
}

#[test]
fn test_update_multisig_signers_success() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut initial_signers = Vec::new(&env);
    initial_signers.push_back(owner.clone());
    initial_signers.push_back(signer1.clone());
    initial_signers.push_back(signer2.clone());
    client.configure_multisig(&pet_id, &initial_signers, &2);

    let mut updated_signers = Vec::new(&env);
    updated_signers.push_back(owner.clone());
    updated_signers.push_back(new_owner.clone());

    let result = client.update_multisig_signers(&pet_id, &updated_signers, &1);
    assert!(result);

    let config = client.get_multisig_config(&pet_id).unwrap();
    assert_eq!(config.threshold, 1);
    assert_eq!(config.signers.len(), 2);
    assert!(config.signers.contains(owner.clone()));
    assert!(config.signers.contains(new_owner.clone()));
}

#[test]
#[should_panic]
fn test_update_multisig_signers_requires_owner_auth() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());
    client.configure_multisig(&pet_id, &signers, &2);

    env.set_auths(&[]);

    let mut updated_signers = Vec::new(&env);
    updated_signers.push_back(owner.clone());
    updated_signers.push_back(signer1.clone());

    client.update_multisig_signers(&pet_id, &updated_signers, &2);
}

#[test]
#[should_panic]
fn test_update_multisig_signers_owner_must_be_in_signers() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    client.configure_multisig(&pet_id, &signers, &2);

    let mut updated_signers = Vec::new(&env);
    updated_signers.push_back(signer1.clone());
    updated_signers.push_back(signer2.clone());

    client.update_multisig_signers(&pet_id, &updated_signers, &2);
}

#[test]
#[should_panic]
fn test_update_multisig_signers_invalid_threshold() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    client.configure_multisig(&pet_id, &signers, &2);

    let mut updated_signers = Vec::new(&env);
    updated_signers.push_back(owner.clone());
    updated_signers.push_back(signer1.clone());

    client.update_multisig_signers(&pet_id, &updated_signers, &3);
}

#[test]
fn test_disable_multisig() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);

    let result = client.disable_multisig(&pet_id);
    assert!(result);

    let config = client.get_multisig_config(&pet_id).unwrap();
    assert!(!config.enabled);
}

#[test]
fn test_require_multisig_for_transfer() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);

    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);
    assert_eq!(proposal_id, 1);

    let proposal = client.get_transfer_proposal(&proposal_id);
    assert!(proposal.is_some());

    let proposal = proposal.unwrap();
    assert_eq!(proposal.pet_id, pet_id);
    assert_eq!(proposal.to, new_owner);
    assert_eq!(proposal.signatures.len(), 1);
    assert!(!proposal.executed);
}

#[test]
#[should_panic]
fn test_require_multisig_for_transfer_not_configured() {
    let env = Env::default();
    let (client, owner, _, _, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    client.require_multisig_for_transfer(&pet_id, &new_owner);
}

#[test]
#[should_panic]
fn test_require_multisig_for_transfer_disabled() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    client.disable_multisig(&pet_id);

    client.require_multisig_for_transfer(&pet_id, &new_owner);
}

#[test]
fn test_sign_transfer_proposal() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    let result = client.sign_transfer_proposal(&proposal_id, &signer1);
    assert!(result);

    let proposal = client.get_transfer_proposal(&proposal_id).unwrap();
    assert_eq!(proposal.signatures.len(), 2);
}

#[test]
#[should_panic]
fn test_sign_transfer_proposal_unauthorized() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer2);
}

#[test]
#[should_panic]
fn test_sign_transfer_proposal_duplicate() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &owner);
}

#[test]
fn test_multisig_transfer_pet_success() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer1);

    let result = client.multisig_transfer_pet(&proposal_id);
    assert!(result);

    let pet_owner = client.get_pet_owner(&pet_id).unwrap();
    assert_eq!(pet_owner, new_owner);

    let proposal = client.get_transfer_proposal(&proposal_id).unwrap();
    assert!(proposal.executed);
}

#[test]
#[should_panic]
fn test_multisig_transfer_pet_threshold_not_met() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &3);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer1);

    client.multisig_transfer_pet(&proposal_id);
}

#[test]
#[should_panic]
fn test_multisig_transfer_pet_already_executed() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer1);
    client.multisig_transfer_pet(&proposal_id);

    client.multisig_transfer_pet(&proposal_id);
}

#[test]
fn test_multisig_transfer_with_all_signers() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &3);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer1);
    client.sign_transfer_proposal(&proposal_id, &signer2);

    let result = client.multisig_transfer_pet(&proposal_id);
    assert!(result);

    let pet_owner = client.get_pet_owner(&pet_id).unwrap();
    assert_eq!(pet_owner, new_owner);
}

#[test]
fn test_multisig_config_per_pet() {
    let env = Env::default();
    let (client, owner, signer1, signer2, _) = setup_test_env(&env);
    let pet_id1 = register_test_pet(&client, &env, &owner);
    let pet_id2 = register_test_pet(&client, &env, &owner);

    let mut signers1 = Vec::new(&env);
    signers1.push_back(owner.clone());
    signers1.push_back(signer1.clone());

    let mut signers2 = Vec::new(&env);
    signers2.push_back(owner.clone());
    signers2.push_back(signer1.clone());
    signers2.push_back(signer2.clone());

    client.configure_multisig(&pet_id1, &signers1, &2);
    client.configure_multisig(&pet_id2, &signers2, &3);

    let config1 = client.get_multisig_config(&pet_id1).unwrap();
    let config2 = client.get_multisig_config(&pet_id2).unwrap();

    assert_eq!(config1.threshold, 2);
    assert_eq!(config1.signers.len(), 2);

    assert_eq!(config2.threshold, 3);
    assert_eq!(config2.signers.len(), 3);
}

#[test]
fn test_ownership_history_after_multisig_transfer() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.sign_transfer_proposal(&proposal_id, &signer1);
    client.multisig_transfer_pet(&proposal_id);

    let history = client.get_ownership_history(&pet_id, &0u64, &10u32);
    assert_eq!(history.len(), 2);

    let last_record = history.get(1).unwrap();
    assert_eq!(last_record.previous_owner, owner);
    assert_eq!(last_record.new_owner, new_owner);
}

#[test]
fn test_ownership_history_pagination() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);

    // Transfer 1
    let proposal_id1 = client.require_multisig_for_transfer(&pet_id, &new_owner);
    client.sign_transfer_proposal(&proposal_id1, &signer1);
    client.multisig_transfer_pet(&proposal_id1);

    // Transfer 2 (back to owner)
    let proposal_id2 = client.require_multisig_for_transfer(&pet_id, &owner);
    client.sign_transfer_proposal(&proposal_id2, &signer1); // signer1 is a configured signer
    client.multisig_transfer_pet(&proposal_id2);

    // Total 3 records (initial registration + 2 transfers)
    let history_all = client.get_ownership_history(&pet_id, &0u64, &10u32);
    assert_eq!(history_all.len(), 3);

    let history_paged = client.get_ownership_history(&pet_id, &1u64, &1u32);
    assert_eq!(history_paged.len(), 1);
    assert_eq!(history_paged.get(0).unwrap().new_owner, new_owner);

    let history_empty = client.get_ownership_history(&pet_id, &5u64, &1u32);
    assert_eq!(history_empty.len(), 0);
}

#[test]
fn test_cancel_transfer_proposal() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.cancel_transfer_proposal(&proposal_id);

    let proposal = client.get_transfer_proposal(&proposal_id).unwrap();
    assert!(proposal.executed); // Executed is used for cancelled too
}

#[test]
#[should_panic]
fn test_sign_cancelled_proposal() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);

    client.cancel_transfer_proposal(&proposal_id);

    client.sign_transfer_proposal(&proposal_id, &signer1);
}

#[test]
fn test_get_active_transfer_proposals_empty() {
    let env = Env::default();
    let (client, owner, _, _, _) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    // No proposals yet, should return empty
    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_get_active_transfer_proposals_single() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    client.require_multisig_for_transfer(&pet_id, &new_owner);

    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(0).unwrap().pet_id, pet_id);
    assert!(!active.get(0).unwrap().executed);
}

#[test]
fn test_get_active_transfer_proposals_multiple() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);

    // Create multiple proposals
    client.require_multisig_for_transfer(&pet_id, &new_owner);
    client.require_multisig_for_transfer(&pet_id, &signer1);
    client.require_multisig_for_transfer(&pet_id, &signer2);

    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 3);
}

#[test]
fn test_get_active_transfer_proposals_excludes_executed() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);
    client.sign_transfer_proposal(&proposal_id, &signer1);
    client.multisig_transfer_pet(&proposal_id);

    // After execution, should return empty
    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_get_active_transfer_proposals_excludes_cancelled() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);
    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);
    client.cancel_transfer_proposal(&proposal_id);

    // After cancellation, should return empty
    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_get_active_transfer_proposals_mixed_states() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id, &signers, &2);

    // Proposal 1: active
    let proposal_id1 = client.require_multisig_for_transfer(&pet_id, &new_owner);

    // Proposal 2: cancelled
    let proposal_id2 = client.require_multisig_for_transfer(&pet_id, &signer1);
    client.cancel_transfer_proposal(&proposal_id2);

    // Proposal 3: active
    let proposal_id3 = client.require_multisig_for_transfer(&pet_id, &signer2);

    let active = client.get_active_transfer_proposals(&pet_id);
    assert_eq!(active.len(), 2);

    // Verify the active proposals are the correct ones
    let mut found_proposal1 = false;
    let mut found_proposal3 = false;
    for i in 0..active.len() {
        let proposal = active.get(i);
        if proposal.as_ref().map(|p| p.id) == Some(proposal_id1) {
            found_proposal1 = true;
        }
        if proposal.as_ref().map(|p| p.id) == Some(proposal_id3) {
            found_proposal3 = true;
        }
    }
    assert!(found_proposal1);
    assert!(found_proposal3);
}

#[test]
fn test_get_active_transfer_proposals_per_pet_isolation() {
    let env = Env::default();
    let (client, owner, signer1, signer2, new_owner) = setup_test_env(&env);
    let pet_id1 = register_test_pet(&client, &env, &owner);
    let pet_id2 = register_test_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    signers.push_back(signer2.clone());

    client.configure_multisig(&pet_id1, &signers, &2);
    client.configure_multisig(&pet_id2, &signers, &2);

    // Create proposal for pet 1
    client.require_multisig_for_transfer(&pet_id1, &new_owner);

    // Create proposal for pet 2
    client.require_multisig_for_transfer(&pet_id2, &signer1);

    // Each pet should only see its own proposals
    let active_pet1 = client.get_active_transfer_proposals(&pet_id1);
    let active_pet2 = client.get_active_transfer_proposals(&pet_id2);

    assert_eq!(active_pet1.len(), 1);
    assert_eq!(active_pet1.get(0).unwrap().pet_id, pet_id1);

    assert_eq!(active_pet2.len(), 1);
    assert_eq!(active_pet2.get(0).unwrap().pet_id, pet_id2);
}
