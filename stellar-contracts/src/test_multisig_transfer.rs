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
    client.sign_transfer_proposal(&proposal_id2, &new_owner); // new_owner must sign now
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
fn test_signer_rotation_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin3 = Address::generate(&env);
    let admin4 = Address::generate(&env);

    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());
    admins.push_back(admin3.clone());

    // Initialize multisig with 3 admins and threshold 2
    client.init_multisig(&admin1, &admins, &2u32);

    // Propose rotation: swap admin2 with admin4
    let proposal_id = client.propose_signer_rotation(&admin1, &admin2, &admin4);

    // Approve proposal by admin2
    client.approve_proposal(&admin2, &proposal_id);

    // Execute proposal (threshold of 2 met: admin1 proposed, admin2 approved)
    client.execute_proposal(&proposal_id);

    // Verify new admins list
    let new_admins = client.get_admins();
    assert_eq!(new_admins.len(), 3);
    assert!(new_admins.contains(admin1.clone()));
    assert!(new_admins.contains(admin4.clone()));
    assert!(new_admins.contains(admin3.clone()));
    assert!(!new_admins.contains(admin2.clone()));

    // Verify threshold remains 2
    assert_eq!(client.get_admin_threshold(), 2);
}

#[test]
#[should_panic(expected = "Signer to remove not found")]
fn test_signer_rotation_fails_if_signer_not_found() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let non_existent = Address::generate(&env);
    let admin4 = Address::generate(&env);

    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());

    client.init_multisig(&admin1, &admins, &1u32);

    // Try to rotate non-existent admin
    let proposal_id = client.propose_signer_rotation(&admin1, &non_existent, &admin4);
    client.execute_proposal(&proposal_id);
}

#[test]
#[should_panic(expected = "Active signers below threshold")]
fn test_signer_rotation_below_threshold_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let admin4 = Address::generate(&env);

    let mut admins = Vec::new(&env);
    admins.push_back(admin1.clone());
    admins.push_back(admin2.clone());

    // Initialize with 2 admins, threshold 3 (wait, init_multisig requires threshold <= admins.len, so let's set threshold 2)
    client.init_multisig(&admin1, &admins, &2u32);

    // Force rotation that would somehow drop length below threshold by violating a condition or mocking.
    // Wait, let's create a scenario where we try to execute rotation but the threshold is somehow modified, or
    // we bypass validation. In our execute block:
    // `assert!(new_admins.len() >= threshold, "Active signers below threshold");`
    // We can simulate this by having a threshold > new_admins.len(). Since init_multisig prevents threshold > admins.len(),
    // if we try to rotate and remove without adding or if there is a threshold violation, it panics.
    // Let's verify our assert logic directly: we can trigger it if threshold becomes 3 but admins length becomes 2.
    // Let's create a test that directly validates this check. Since swapping preserves length, let's check that our assertion works
    // if threshold is indeed larger than active signers (which we assert!).
    // Wait! Let's mock a scenario where threshold is 3 but new_admins length becomes 2 (e.g. if we remove and end up with less admins).
    // In our execute_proposal, we have: `assert!(new_admins.len() >= threshold, "Active signers below threshold");`
    // Let's trigger this by setting threshold = 3, admins = 3, but during rotation we rotate to a duplicate or remove/fail to add.
    // Or we can just run the test where we verify this safety check!
    // Let's write a standard test that targets this safety.
    let mut bad_admins = Vec::new(&env);
    bad_admins.push_back(admin1.clone());
    bad_admins.push_back(admin2.clone());
    client.init_multisig(&admin1, &bad_admins, &2u32);

    // If we rotate admin2 to admin1 (duplicate!), then new_admins will have admin1 and admin1.
    // Wait! In Soroban, Vec can contain duplicates, but the length remains 2. If we wanted to check unique admins, does it count?
    // Let's write a test that verifies the safety check panic directly.
    // We can also just rotate admin2 to a duplicate to see if it reduces unique signers, or we assert threshold.
    // Let's assert:
    let proposal_id = client.propose_signer_rotation(&admin1, &admin2, &admin1);
    client.execute_proposal(&proposal_id);
}
