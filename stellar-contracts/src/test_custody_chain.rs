use crate::{CustodyEntry, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species, SystemKey, TransferType};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup(env: &Env) -> (PetChainContractClient, Address, Address) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let owner = Address::generate(env);
    let new_owner = Address::generate(env);
    client.init_admin(&owner);
    (client, owner, new_owner)
}

fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Buddy"),
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

// -------------------------------------------------------
// Direct transfer appends a Direct custody entry
// -------------------------------------------------------

#[test]
fn direct_transfer_appends_custody_entry() {
    let env = Env::default();
    let (client, owner, new_owner) = setup(&env);
    let pet_id = register_pet(&client, &env, &owner);

    client.transfer_pet_ownership(&pet_id, &new_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 1);

    let entry = chain.get(0).unwrap();
    assert_eq!(entry.from, owner);
    assert_eq!(entry.to, new_owner);
    assert_eq!(entry.transfer_type, TransferType::Direct);
}

// -------------------------------------------------------
// Multiple transfers produce chronologically ordered entries
// -------------------------------------------------------

#[test]
fn multiple_transfers_produce_ordered_chain() {
    let env = Env::default();
    let (client, owner, new_owner) = setup(&env);
    let third_owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    // First transfer: owner -> new_owner
    client.transfer_pet_ownership(&pet_id, &new_owner, &0);
    client.accept_pet_transfer(&pet_id);

    // Second transfer: new_owner -> third_owner
    client.transfer_pet_ownership(&pet_id, &third_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 2);

    let first = chain.get(0).unwrap();
    assert_eq!(first.from, owner);
    assert_eq!(first.to, new_owner);
    assert_eq!(first.transfer_type, TransferType::Direct);

    let second = chain.get(1).unwrap();
    assert_eq!(second.from, new_owner);
    assert_eq!(second.to, third_owner);
    assert_eq!(second.transfer_type, TransferType::Direct);

    // Timestamps must be non-decreasing (chronological order)
    assert!(first.timestamp <= second.timestamp);
}

// -------------------------------------------------------
// No delete path: custody chain is append-only
// -------------------------------------------------------

#[test]
fn custody_chain_is_append_only_no_delete_path_exists() {
    // The contract exposes no function to remove entries from the custody chain.
    // We verify this by confirming the chain length only ever grows.
    let env = Env::default();
    let (client, owner, new_owner) = setup(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let chain_before = client.get_custody_chain(&pet_id);
    assert_eq!(chain_before.len(), 0);

    client.transfer_pet_ownership(&pet_id, &new_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let chain_after = client.get_custody_chain(&pet_id);
    assert_eq!(chain_after.len(), 1);

    // Attempting a second transfer does not shrink the chain
    let third_owner = Address::generate(&env);
    client.transfer_pet_ownership(&pet_id, &third_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let chain_final = client.get_custody_chain(&pet_id);
    assert_eq!(chain_final.len(), 2);
}

// -------------------------------------------------------
// get_custody_chain returns empty vec for pet with no transfers
// -------------------------------------------------------

#[test]
fn get_custody_chain_returns_empty_for_new_pet() {
    let env = Env::default();
    let (client, owner, _) = setup(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 0);
}

// -------------------------------------------------------
// verify_custody_chain (Issue #822)
// -------------------------------------------------------

fn set_custody_chain(env: &Env, contract_id: &Address, pet_id: &u64, chain: &Vec<CustodyEntry>) {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .set(&SystemKey::CustodyChain(*pet_id), chain);
    });
}

#[test]
fn verify_custody_chain_valid_for_new_pet_with_no_transfers() {
    let env = Env::default();
    let (client, owner, _) = setup(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let result = client.verify_custody_chain(&pet_id);
    assert!(result.valid);
    assert_eq!(result.gap_at, None);
}

#[test]
fn verify_custody_chain_valid_for_consistent_chain() {
    let env = Env::default();
    let (client, owner, new_owner) = setup(&env);
    let third_owner = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    client.transfer_pet_ownership(&pet_id, &new_owner, &0);
    client.accept_pet_transfer(&pet_id);
    client.transfer_pet_ownership(&pet_id, &third_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let result = client.verify_custody_chain(&pet_id);
    assert!(result.valid);
    assert_eq!(result.gap_at, None);
}

#[test]
fn verify_custody_chain_detects_gap_at_index() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let stranger = Address::generate(&env);
    env.mock_all_auths();
    client.init_admin(&owner);
    let pet_id = register_pet(&client, &env, &owner);

    client.transfer_pet_ownership(&pet_id, &a, &0);
    client.accept_pet_transfer(&pet_id);
    client.transfer_pet_ownership(&pet_id, &b, &0);
    client.accept_pet_transfer(&pet_id);
    client.transfer_pet_ownership(&pet_id, &c, &0);
    client.accept_pet_transfer(&pet_id);

    let mut chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 3);

    // Break the link between entry 1 (A -> B) and entry 2 (should start at B):
    // rewrite entry 2's `from` to an unrelated address, so the record no
    // longer picks up where the previous one left off.
    let mut broken = chain.get(2).unwrap();
    broken.from = stranger;
    chain.set(2, broken);
    set_custody_chain(&env, &contract_id, &pet_id, &chain);

    let result = client.verify_custody_chain(&pet_id);
    assert!(!result.valid);
    assert_eq!(result.gap_at, Some(2));
}

#[test]
fn verify_custody_chain_detects_forked_chain() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    env.mock_all_auths();
    client.init_admin(&owner);
    let pet_id = register_pet(&client, &env, &owner);

    client.transfer_pet_ownership(&pet_id, &a, &0);
    client.accept_pet_transfer(&pet_id);
    client.transfer_pet_ownership(&pet_id, &b, &0);
    client.accept_pet_transfer(&pet_id);
    client.transfer_pet_ownership(&pet_id, &c, &0);
    client.accept_pet_transfer(&pet_id);

    let mut chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 3);

    // Fork the chain: entry 1 claims to originate from the same custodian as
    // entry 0 (`owner`) instead of continuing from entry 0's `to` (`a`), i.e.
    // two branches both claim custody handed off directly from `owner`.
    let entry_zero = chain.get(0).unwrap();
    let mut forked = chain.get(1).unwrap();
    forked.from = entry_zero.from;
    chain.set(1, forked);
    set_custody_chain(&env, &contract_id, &pet_id, &chain);

    let result = client.verify_custody_chain(&pet_id);
    assert!(!result.valid);
    assert_eq!(result.gap_at, Some(1));
}

#[test]
fn verify_custody_chain_detects_current_owner_mismatch() {
    let env = Env::default();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let stranger = Address::generate(&env);
    env.mock_all_auths();
    client.init_admin(&owner);
    let pet_id = register_pet(&client, &env, &owner);

    client.transfer_pet_ownership(&pet_id, &new_owner, &0);
    client.accept_pet_transfer(&pet_id);

    let mut chain = client.get_custody_chain(&pet_id);
    let mut last = chain.get(0).unwrap();
    last.to = stranger;
    chain.set(0, last);
    set_custody_chain(&env, &contract_id, &pet_id, &chain);

    let result = client.verify_custody_chain(&pet_id);
    assert!(!result.valid);
    assert_eq!(result.gap_at, Some(1));
}
