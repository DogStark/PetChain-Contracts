use crate::{DataKey, PetChainContract, PetChainContractClient};
use soroban_sdk::{BytesN, Env};

fn make_hash(env: &Env, val: u8) -> BytesN<32> {
    BytesN::from_array(env, &[val; 32])
}

fn store_doc_hashes(env: &Env, contract_id: &soroban_sdk::Address, claim_id: u64, hashes: soroban_sdk::Vec<BytesN<32>>) {
    env.as_contract(contract_id, || {
        env.storage()
            .instance()
            .set(&DataKey::ClaimDocuments(claim_id), &hashes);
    });
}

#[test]
fn test_verify_claim_document_matching_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let claim_id: u64 = 1;
    let hash = make_hash(&env, 0xAB);
    let mut hashes: soroban_sdk::Vec<BytesN<32>> = soroban_sdk::Vec::new(&env);
    hashes.push_back(hash.clone());
    store_doc_hashes(&env, &contract_id, claim_id, hashes);

    let result = client.verify_claim_document(&claim_id, &0u32, &hash);
    assert!(result, "matching hash should return true");
}

#[test]
fn test_verify_claim_document_non_matching_hash() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let claim_id: u64 = 1;
    let stored_hash = make_hash(&env, 0xAB);
    let wrong_hash = make_hash(&env, 0xCD);
    let mut hashes: soroban_sdk::Vec<BytesN<32>> = soroban_sdk::Vec::new(&env);
    hashes.push_back(stored_hash);
    store_doc_hashes(&env, &contract_id, claim_id, hashes);

    let result = client.verify_claim_document(&claim_id, &0u32, &wrong_hash);
    assert!(!result, "non-matching hash should return false");
}

#[test]
fn test_verify_claim_document_out_of_bounds_index() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    // No documents stored for claim_id 99
    let hash = make_hash(&env, 0x01);
    let result = client.verify_claim_document(&99u64, &0u32, &hash);
    assert!(!result, "out-of-bounds index should return false");
}
