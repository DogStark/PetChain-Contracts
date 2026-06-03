use crate::{
    Gender, InsuranceClaimStatus, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// A valid CIDv0 (46 chars, starts with Qm, base58)
const VALID_CID: &str = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";
const VALID_CID2: &str = "QmSoLPppuBtQSGwKDZT2M73ULpjvfd3aZ6ha4oFGL1KrGM";

fn setup(env: &Env) -> (PetChainContractClient, Address, Address, u64) {
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let owner = Address::generate(env);
    client.init_admin(&admin);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Rex"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(env, "POL-001"),
        &String::from_str(env, "PetCare"),
        &String::from_str(env, "Standard"),
        &100,
        &10000,
        &expiry,
    );

    (client, admin, owner, pet_id)
}

#[test]
fn test_attach_document_in_pending_state() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, owner, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));

    let docs = client.get_claim_documents(&claim_id);
    assert_eq!(docs.len(), 1);
    assert_eq!(docs.get(0).unwrap(), String::from_str(&env, VALID_CID));
}

#[test]
fn test_attach_multiple_documents() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, owner, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Surgery"))
        .unwrap();

    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));
    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID2));

    let docs = client.get_claim_documents(&claim_id);
    assert_eq!(docs.len(), 2);
}

#[test]
#[should_panic]
fn test_attach_document_after_under_review_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, owner, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::UnderReview);

    // Should panic — claim is immutable
    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));
}

#[test]
#[should_panic]
fn test_attach_document_after_approved_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, owner, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Approved);
    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));
}

#[test]
#[should_panic]
fn test_document_limit_enforced() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, owner, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Attach 10 documents (max)
    for _ in 0..10 {
        client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));
    }
    // 11th should panic
    client.attach_claim_document(&owner, &claim_id, &String::from_str(&env, VALID_CID));
}

#[test]
fn test_get_claim_documents_empty_for_new_claim() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    let docs = client.get_claim_documents(&claim_id);
    assert_eq!(docs.len(), 0);
}

#[test]
#[should_panic]
fn test_non_owner_cannot_attach_document() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, _, _, pet_id) = setup(&env);

    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    let stranger = Address::generate(&env);
    client.attach_claim_document(&stranger, &claim_id, &String::from_str(&env, VALID_CID));
}
