use crate::{
    Gender, InsuranceClaimStatus, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_submit_and_get_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Mixed"),
        &20,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-12345"),
        &String::from_str(&env, "Happy Pets Inc"),
        &String::from_str(&env, "Comprehensive"),
        &100,
        &10000,
        &expiry,
    );

    let claim_amount = 500;
    let claim_desc = String::from_str(&env, "Vet visit for broken leg");

    let claim_id = client
        .submit_insurance_claim(&pet_id, &claim_amount, &claim_desc)
        .unwrap();

    let claim = client.get_insurance_claim(&claim_id).unwrap();

    assert_eq!(claim.claim_id, claim_id);
    assert_eq!(claim.pet_id, pet_id);
    assert_eq!(claim.policy_id, String::from_str(&env, "POL-12345"));
    assert_eq!(claim.amount, claim_amount);
    assert_eq!(claim.status, InsuranceClaimStatus::Pending);
    assert_eq!(claim.description, claim_desc);
}

#[test]
fn test_submit_claim_inactive_policy() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Whiskers"),
        &String::from_str(&env, "2021-02-02"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Domestic Shorthair"),
        &4,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-INACTIVE"),
        &String::from_str(&env, "Sad Pets Inc"),
        &String::from_str(&env, "Accident Only"),
        &50,
        &5000,
        &expiry,
    );

    client.update_insurance_status(
        &owner,
        &pet_id,
        &String::from_str(&env, "POL-INACTIVE"),
        &false,
    );

    let result = client.submit_insurance_claim(
        &pet_id,
        &100,
        &String::from_str(&env, "Attempted claim on inactive policy"),
    );

    assert!(result.is_none());
}

#[test]
fn test_update_claim_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2019-05-05"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &String::from_str(&env, "Golden Retriever"),
        &30,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-987"),
        &String::from_str(&env, "Best Pets"),
        &String::from_str(&env, "Premium"),
        &200,
        &20000,
        &expiry,
    );

    let claim_id = client
        .submit_insurance_claim(
            &pet_id,
            &1500,
            &String::from_str(&env, "Surgery for hip dysplasia"),
        )
        .unwrap();

    let _ = client.update_insurance_claim_status(&claim_id, &InsuranceClaimStatus::Approved);

    let updated_claim = client.get_insurance_claim(&claim_id).unwrap();

    assert_eq!(updated_claim.status, InsuranceClaimStatus::Approved);
}

#[test]
fn test_get_all_pet_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-10-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "White"),
        &String::from_str(&env, "Persian"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-CAT-01"),
        &String::from_str(&env, "Feline Care"),
        &String::from_str(&env, "Standard"),
        &80,
        &8000,
        &expiry,
    );

    client.submit_insurance_claim(&pet_id, &200, &String::from_str(&env, "Routine checkup"));

    client.submit_insurance_claim(&pet_id, &150, &String::from_str(&env, "Vaccinations"));

    let claims = client.get_pet_insurance_claims(&pet_id);

    assert_eq!(claims.len(), 2);
    assert_eq!(claims.get(0).unwrap().amount, 200);
    assert_eq!(claims.get(1).unwrap().amount, 150);
}

#[test]
fn test_claim_recalculates_policy_premium() {
fn test_get_insurance_claim_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Risky"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Mixed"),
        &20,

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-06-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Black"),
        &String::from_str(&env, "Labrador"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "RISK-1"),
        &String::from_str(&env, "Happy Pets Inc"),
        &String::from_str(&env, "Standard"),
        &1000,
        &10000,
        &expiry,
    );

    let before = client.get_pet_insurance(&pet_id).unwrap().premium;
    client.submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Vet visit"));
    let after = client.get_pet_insurance(&pet_id).unwrap().premium;
    assert!(after > before);
    // Count should be 0 before any claims
    assert_eq!(client.get_insurance_claim_count(&pet_id), 0);

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(&env, "POL-COUNT-01"),
        &String::from_str(&env, "PetSure"),
        &String::from_str(&env, "Standard"),
        &120,
        &12000,
        &expiry,
    );

    client.submit_insurance_claim(&pet_id, &300, &String::from_str(&env, "Dental cleaning"));
    assert_eq!(client.get_insurance_claim_count(&pet_id), 1);

    client.submit_insurance_claim(
        &pet_id,
        &750,
        &String::from_str(&env, "X-ray and treatment"),
    );
    assert_eq!(client.get_insurance_claim_count(&pet_id), 2);

    client.submit_insurance_claim(&pet_id, &1200, &String::from_str(&env, "Emergency surgery"));
    assert_eq!(client.get_insurance_claim_count(&pet_id), 3);
}

#[test]
fn test_get_insurance_claim_count_no_claims() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2023-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Orange"),
        &String::from_str(&env, "Tabby"),
        &4,
        &None,
        &PrivacyLevel::Public,
    );

    // Pet with no claims should return 0
    assert_eq!(client.get_insurance_claim_count(&pet_id), 0);
}

fn setup_pet_with_claims(
    env: &Env,
    client: &PetChainContractClient,
) -> u64 {
    let owner = Address::generate(env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "Cleo"),
        &String::from_str(env, "2020-03-15"),
        &crate::Gender::Female,
        &crate::Species::Dog,
        &String::from_str(env, "Spotted"),
        &String::from_str(env, "Dalmatian"),
        &22,
        &None,
        &crate::PrivacyLevel::Public,
    );

    let expiry = env.ledger().timestamp() + 31536000;
    client.add_insurance_policy(
        &pet_id,
        &String::from_str(env, "POL-STATUS-01"),
        &String::from_str(env, "StatusCare"),
        &String::from_str(env, "Full"),
        &150,
        &15000,
        &expiry,
    );

    pet_id
}

#[test]
fn test_get_claims_by_status_pending() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = setup_pet_with_claims(&env, &client);

    let id1 = client.submit_insurance_claim(&pet_id, &100, &String::from_str(&env, "Checkup")).unwrap();
    let id2 = client.submit_insurance_claim(&pet_id, &200, &String::from_str(&env, "X-ray")).unwrap();
    client.update_insurance_claim_status(&id2, &InsuranceClaimStatus::Approved);

    let pending = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Pending);
    assert_eq!(pending.len(), 1);
    assert_eq!(pending.get(0).unwrap().claim_id, id1);
}

#[test]
fn test_get_claims_by_status_approved() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = setup_pet_with_claims(&env, &client);

    let id1 = client.submit_insurance_claim(&pet_id, &300, &String::from_str(&env, "Surgery")).unwrap();
    let id2 = client.submit_insurance_claim(&pet_id, &400, &String::from_str(&env, "Meds")).unwrap();
    client.update_insurance_claim_status(&id1, &InsuranceClaimStatus::Approved);
    client.update_insurance_claim_status(&id2, &InsuranceClaimStatus::Approved);

    let approved = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Approved);
    assert_eq!(approved.len(), 2);
}

#[test]
fn test_get_claims_by_status_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = setup_pet_with_claims(&env, &client);

    let id1 = client.submit_insurance_claim(&pet_id, &500, &String::from_str(&env, "Cosmetic")).unwrap();
    client.update_insurance_claim_status(&id1, &InsuranceClaimStatus::Rejected);
    client.submit_insurance_claim(&pet_id, &600, &String::from_str(&env, "Dental"));

    let rejected = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Rejected);
    assert_eq!(rejected.len(), 1);
    assert_eq!(rejected.get(0).unwrap().claim_id, id1);
}

#[test]
fn test_get_claims_by_status_paid() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = setup_pet_with_claims(&env, &client);

    let id1 = client.submit_insurance_claim(&pet_id, &700, &String::from_str(&env, "Hip surgery")).unwrap();
    client.update_insurance_claim_status(&id1, &InsuranceClaimStatus::Paid);

    let paid = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Paid);
    assert_eq!(paid.len(), 1);
    assert_eq!(paid.get(0).unwrap().claim_id, id1);
}

#[test]
fn test_get_claims_by_status_empty() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let pet_id = setup_pet_with_claims(&env, &client);

    client.submit_insurance_claim(&pet_id, &100, &String::from_str(&env, "Checkup"));

    // No claims have been approved, so result should be empty
    let approved = client.get_claims_by_status(&pet_id, &InsuranceClaimStatus::Approved);
    assert_eq!(approved.len(), 0);
}
