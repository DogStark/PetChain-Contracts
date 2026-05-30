use crate::{
    Gender, InsuranceClaimStatus, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

fn setup_pet_with_policy(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    let pet_id = client.register_pet(
        owner,
        &String::from_str(env, "TestPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Labrador"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(env, "POL-APPEAL-01"),
        &String::from_str(env, "AppealTest Insurance"),
        &String::from_str(env, "Standard"),
        &100,
        &10000,
        &expiry,
    );

    pet_id
}

#[test]
fn test_appeal_rejected_claim_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal the claim
    let mut new_evidence = Vec::new(&env);
    new_evidence.push_back(String::from_str(&env, "QmNewEvidence1234567890123456789012345678901234"));
    
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Additional medical records show necessity"),
        &new_evidence,
    );

    // Verify claim is now under appeal
    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::UnderAppeal);
    assert!(claim.appealed_at.is_some());
    assert_eq!(claim.appeal_evidence_cids.len(), 1);
    assert_eq!(
        claim.appeal_reason.unwrap(),
        String::from_str(&env, "Additional medical records show necessity")
    );
}

#[test]
#[should_panic(expected = "ClaimNotRejected")]
fn test_appeal_non_rejected_claim_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit a claim but don't reject it
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Try to appeal a pending claim - should fail
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Appeal reason"),
        &new_evidence,
    );
}

#[test]
#[should_panic(expected = "AppealWindowExpired")]
fn test_appeal_after_14_days_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Fast forward 15 days (14 days + 1 day)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + (15 * 24 * 60 * 60);
    });

    // Try to appeal - should fail
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Too late"),
        &new_evidence,
    );
}

#[test]
fn test_appeal_within_14_days_succeeds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Fast forward 13 days (within window)
    env.ledger().with_mut(|li| {
        li.timestamp = li.timestamp + (13 * 24 * 60 * 60);
    });

    // Appeal should succeed
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Within window"),
        &new_evidence,
    );

    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::UnderAppeal);
}

#[test]
#[should_panic(expected = "ClaimAlreadyAppealed")]
fn test_cannot_appeal_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // First appeal
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "First appeal"),
        &new_evidence,
    );

    // Try to appeal again - should fail
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Second appeal"),
        &new_evidence,
    );
}

#[test]
#[should_panic(expected = "InvalidIpfsHash")]
fn test_appeal_with_invalid_cid_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Try to appeal with invalid CID
    let mut invalid_evidence = Vec::new(&env);
    invalid_evidence.push_back(String::from_str(&env, "invalid-cid"));
    
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Appeal with bad CID"),
        &invalid_evidence,
    );
}

#[test]
#[should_panic(expected = "ClaimDocumentLimitReached")]
fn test_appeal_exceeding_document_limit_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Attach 10 documents to the claim (max limit)
    for i in 0..10 {
        let cid = String::from_str(&env, &format!("QmDoc{:044}", i));
        client.attach_claim_document(&owner, &claim_id, &cid);
    }

    // Reject the claim
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Try to appeal with additional evidence - should fail due to limit
    let mut new_evidence = Vec::new(&env);
    new_evidence.push_back(String::from_str(&env, "QmNewEvidence1234567890123456789012345678901234"));
    
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Too many docs"),
        &new_evidence,
    );
}

#[test]
fn test_review_appeal_approve() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    // Initialize admin
    client.init_admin(&admin1);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Set original reviewer
    client.set_claim_reviewer(&admin1, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal the claim
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "New evidence"),
        &new_evidence,
    );

    // Change admin to admin2 for second review
    let proposal_id = client.propose_change_admin(&admin1, &admin2);
    client.execute_proposal(&proposal_id);

    // Second reviewer approves the appeal
    client.review_appeal(&admin2, &claim_id, &InsuranceClaimStatus::Approved);

    // Verify final status
    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::Approved);
    assert!(claim.appeal_reviewer.is_some());
    assert_eq!(claim.appeal_reviewer.unwrap(), admin2);
}

#[test]
fn test_review_appeal_reject() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    // Initialize admin
    client.init_admin(&admin1);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Set original reviewer
    client.set_claim_reviewer(&admin1, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal the claim
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "New evidence"),
        &new_evidence,
    );

    // Change admin to admin2 for second review
    let proposal_id = client.propose_change_admin(&admin1, &admin2);
    client.execute_proposal(&proposal_id);

    // Second reviewer rejects the appeal (final decision)
    client.review_appeal(&admin2, &claim_id, &InsuranceClaimStatus::Rejected);

    // Verify final status
    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::Rejected);
    assert!(claim.appeal_reviewer.is_some());
}

#[test]
#[should_panic(expected = "ClaimNotUnderAppeal")]
fn test_review_appeal_non_appealed_claim_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim but don't appeal
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Try to review appeal without appeal - should fail
    client.review_appeal(&admin, &claim_id, &InsuranceClaimStatus::Approved);
}

#[test]
#[should_panic(expected = "ReviewerCannotBeOriginal")]
fn test_original_reviewer_cannot_review_appeal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Set original reviewer
    client.set_claim_reviewer(&admin, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal the claim
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "New evidence"),
        &new_evidence,
    );

    // Try to have same admin review appeal - should fail
    client.review_appeal(&admin, &claim_id, &InsuranceClaimStatus::Approved);
}

#[test]
#[should_panic(expected = "Unauthorized")]
fn test_non_admin_cannot_review_appeal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.set_claim_reviewer(&admin, &claim_id);
    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal the claim
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "New evidence"),
        &new_evidence,
    );

    // Try to have non-admin review appeal - should fail
    client.review_appeal(&non_admin, &claim_id, &InsuranceClaimStatus::Approved);
}

#[test]
fn test_appeal_with_multiple_evidence_documents() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit and reject a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Rejected);

    // Appeal with multiple evidence documents
    let mut new_evidence = Vec::new(&env);
    new_evidence.push_back(String::from_str(&env, "QmEvidence11234567890123456789012345678901234567"));
    new_evidence.push_back(String::from_str(&env, "QmEvidence21234567890123456789012345678901234567"));
    new_evidence.push_back(String::from_str(&env, "QmEvidence31234567890123456789012345678901234567"));
    
    client.appeal_claim(
        &owner,
        &claim_id,
        &String::from_str(&env, "Multiple documents"),
        &new_evidence,
    );

    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(claim.status, InsuranceClaimStatus::UnderAppeal);
    assert_eq!(claim.appeal_evidence_cids.len(), 3);
}

#[test]
fn test_set_claim_reviewer() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);

    client.init_admin(&admin);

    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit a claim
    let claim_id = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"))
        .unwrap();

    // Set reviewer
    client.set_claim_reviewer(&admin, &claim_id);

    // Verify reviewer is set
    let claim = client.get_insurance_claim(&claim_id).unwrap();
    assert!(claim.original_reviewer.is_some());
    assert_eq!(claim.original_reviewer.unwrap(), admin);
}

#[test]
fn test_get_claims_by_status_under_appeal() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = setup_pet_with_policy(&env, &client, &owner);

    // Submit multiple claims
    let claim_id1 = client
        .submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Claim 1"))
        .unwrap();
    let claim_id2 = client
        .submit_insurance_claim(&pet_id, &600, &String::from_str(&env, "Claim 2"))
        .unwrap();

    // Reject both
    client.update_insurance_claim_status(&claim_id1, &InsuranceClaimStatus::Rejected);
    client.update_insurance_claim_status(&claim_id2, &InsuranceClaimStatus::Rejected);

    // Appeal only the first one
    let new_evidence = Vec::new(&env);
    client.appeal_claim(
        &owner,
        &claim_id1,
        &String::from_str(&env, "Appeal claim 1"),
        &new_evidence,
    );

    // Check status filtering
    let under_appeal = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::UnderAppeal);
    assert_eq!(under_appeal.len(), 1);
    assert_eq!(under_appeal.get(0).unwrap().claim_id, claim_id1);

    let rejected = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Rejected);
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected.get(0).unwrap().claim_id, claim_id2);
}
