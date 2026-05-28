use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

fn setup() -> (
    Env,
    PetChainContractClient<'static>,
    Address,
    Address,
    Address,
    Address,
    u64,
) {
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

    (env, client, admin, owner, target, arbitrator, pet_id)
}

fn raise(client: &PetChainContractClient, env: &Env, pet_id: u64, owner: &Address, target: &Address) -> u64 {
    client.raise_dispute(
        &pet_id,
        owner,
        target,
        &1_000u64,
        &String::from_str(env, "Pet not delivered"),
        &String::from_str(env, "ipfs://initial"),
    )
}

#[test]
fn test_dispute_state_machine_happy_path_and_appeal() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    client.set_appeal_window(&admin, &100);
    client.assign_arbitrator(&admin, &arbitrator);

    let dispute_id = raise(&client, &env, pet_id, &owner, &target);
    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::Open);

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

    env.ledger().set_timestamp(1_050);
    assert!(client.appeal(&dispute_id, &target));
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::Appealed
    );
}

#[test]
#[should_panic]
fn test_non_arbitrator_cannot_rule() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    let stranger = Address::generate(&env);
    client.assign_arbitrator(&admin, &arbitrator);
    let dispute_id = raise(&client, &env, pet_id, &owner, &target);
    client.submit_evidence(
        &dispute_id,
fn test_resolve_dispute_no_admin_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let target = Address::generate(&env);

    let pet_id = 1; // Dummy ID for this test

    let dispute_id = client.raise_dispute(
        &pet_id,
        &owner,
        &String::from_str(&env, "ipfs://owner-evidence"),
    );
    client.start_review(&dispute_id, &arbitrator);

    client.rule(
        &dispute_id,
        &stranger,
        &DisputeOutcome::InFavorOfTarget,
    );
}

#[test]
#[should_panic]
fn test_invalid_transition_rule_before_evidence_rejected() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    client.assign_arbitrator(&admin, &arbitrator);
    let dispute_id = raise(&client, &env, pet_id, &owner, &target);

    client.rule(
        &dispute_id,
        &arbitrator,
        &DisputeOutcome::Split,
    );
}

#[test]
#[should_panic]
fn test_unauthorized_evidence_rejected() {
    let (env, client, _admin, owner, target, _arbitrator, pet_id) = setup();
    let stranger = Address::generate(&env);
    let dispute_id = raise(&client, &env, pet_id, &owner, &target);

    client.submit_evidence(
        &dispute_id,
        &stranger,
        &String::from_str(&env, "ipfs://stranger"),
    );
}

#[test]
#[should_panic]
fn test_late_appeal_rejected() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    client.set_appeal_window(&admin, &10);
    client.assign_arbitrator(&admin, &arbitrator);
    let dispute_id = raise(&client, &env, pet_id, &owner, &target);
    client.submit_evidence(
        &dispute_id,
        &target,
        &String::from_str(&env, "ipfs://target-evidence"),
    );

    env.ledger().set_timestamp(100);
    client.start_review(&dispute_id, &arbitrator);
    client.rule(
        &dispute_id,
        &arbitrator,
        &DisputeOutcome::InFavorOfTarget,
    );
    env.ledger().set_timestamp(111);

    client.appeal(&dispute_id, &owner);
}

#[test]
fn test_get_pet_disputes_indexes_all_disputes() {
    let (env, client, _admin, owner, target, _arbitrator, pet_id) = setup();
    raise(&client, &env, pet_id, &owner, &target);
    raise(&client, &env, pet_id, &owner, &target);

    let disputes = client.get_pet_disputes(&pet_id);
    assert_eq!(disputes.len(), 2);
    assert_eq!(disputes.get(0).unwrap().amount, 1_000);
    assert_eq!(disputes.get(1).unwrap().status, DisputeStatus::Open);
}
