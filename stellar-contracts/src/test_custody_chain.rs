use crate::{CustodyEntry, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species, TransferType};
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

    client.transfer_pet_ownership(&pet_id, &new_owner);
    client.accept_pet_transfer(&pet_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 1);

    let entry = chain.get(0).unwrap();
    assert_eq!(entry.from, owner);
    assert_eq!(entry.to, new_owner);
    assert_eq!(entry.transfer_type, TransferType::Direct);
}

// -------------------------------------------------------
// Multisig transfer appends a Multisig custody entry
// -------------------------------------------------------

#[test]
fn multisig_transfer_appends_custody_entry() {
    let env = Env::default();
    let (client, owner, new_owner) = setup(&env);
    let signer1 = Address::generate(&env);
    let pet_id = register_pet(&client, &env, &owner);

    let mut signers = Vec::new(&env);
    signers.push_back(owner.clone());
    signers.push_back(signer1.clone());
    client.configure_multisig(&pet_id, &signers, &2);

    let proposal_id = client.require_multisig_for_transfer(&pet_id, &new_owner);
    client.sign_transfer_proposal(&proposal_id, &signer1);
    client.multisig_transfer_pet(&proposal_id);

    let chain = client.get_custody_chain(&pet_id);
    assert_eq!(chain.len(), 1);

    let entry = chain.get(0).unwrap();
    assert_eq!(entry.from, owner);
    assert_eq!(entry.to, new_owner);
    assert_eq!(entry.transfer_type, TransferType::Multisig);
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
    client.transfer_pet_ownership(&pet_id, &new_owner);
    client.accept_pet_transfer(&pet_id);

    // Second transfer: new_owner -> third_owner
    client.transfer_pet_ownership(&pet_id, &third_owner);
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

    client.transfer_pet_ownership(&pet_id, &new_owner);
    client.accept_pet_transfer(&pet_id);

    let chain_after = client.get_custody_chain(&pet_id);
    assert_eq!(chain_after.len(), 1);

    // Attempting a second transfer does not shrink the chain
    let third_owner = Address::generate(&env);
    client.transfer_pet_ownership(&pet_id, &third_owner);
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
