use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let arbitrator = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-06-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden"),
        &String::from_str(&env, "Golden Retriever"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.set_appeal_window(&admin, &100);
    client.assign_arbitrator(&admin, &arbitrator);

    (env, client, admin, owner, target, arbitrator, pet_id)
}

#[test]
fn test_raise_dispute_and_list_pet_disputes() {
    let (env, client, _admin, owner, target, _arbitrator, pet_id) = setup();

    let dispute_id = client.raise_dispute(
        &pet_id,
        &owner,
        &target,
        &1_000u64,
        &String::from_str(&env, "Pet not delivered"),
        &String::from_str(&env, "ipfs://initial"),
    );

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::Open);
    assert_eq!(client.get_pet_disputes(&pet_id).len(), 1);
}

#[test]
fn test_submit_evidence_and_review_flow() {
    let (env, client, _admin, owner, target, arbitrator, pet_id) = setup();

    let dispute_id = client.raise_dispute(
        &pet_id,
        &owner,
        &target,
        &1_000u64,
        &String::from_str(&env, "Pet not delivered"),
        &String::from_str(&env, "ipfs://initial"),
    );

    assert!(client.submit_evidence(
        &dispute_id,
        &target,
        &String::from_str(&env, "ipfs://target-evidence"),
    ));
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::EvidencePhase
    );

    env.ledger().set_timestamp(1_000);
    assert!(client.start_review(&dispute_id, &arbitrator));
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::UnderReview
    );

    assert!(client.rule(
        &dispute_id,
        &arbitrator,
        &DisputeOutcome::InFavorOfClaimer,
    ));
    let resolved = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(resolved.status, DisputeStatus::Resolved);
    assert_eq!(resolved.resolved_at, Some(1_000));
}
