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

// --- Reputation-based arbitrator tests ---

fn full_dispute(
    client: &PetChainContractClient,
    env: &Env,
    pet_id: u64,
    owner: &Address,
    target: &Address,
    arbitrator: &Address,
) -> u64 {
    let dispute_id = raise(client, env, pet_id, owner, target);
    client.submit_evidence(
        &dispute_id,
        target,
        &String::from_str(env, "ipfs://evidence"),
    );
    client.start_review(&dispute_id, arbitrator);
    client.rule(&dispute_id, arbitrator, &DisputeOutcome::InFavorOfClaimer);
    dispute_id
}

#[test]
fn test_register_arbitrator_and_auto_assign_selects_highest_reputation() {
    let (env, client, admin, owner, target, arb1, pet_id) = setup();
    let arb2 = Address::generate(&env);

    client.register_arbitrator(&admin, &arb1);
    client.register_arbitrator(&admin, &arb2);

    // Give arb2 two rulings (higher reputation) by assigning it manually first
    client.assign_arbitrator(&admin, &arb2);
    let d1 = raise(&client, &env, pet_id, &owner, &target);
    client.submit_evidence(&d1, &target, &String::from_str(&env, "ipfs://e1"));
    client.start_review(&d1, &arb2);
    client.rule(&d1, &arb2, &DisputeOutcome::InFavorOfClaimer);

    let d2 = raise(&client, &env, pet_id, &owner, &target);
    client.submit_evidence(&d2, &target, &String::from_str(&env, "ipfs://e2"));
    client.start_review(&d2, &arb2);
    client.rule(&d2, &arb2, &DisputeOutcome::InFavorOfClaimer);

    // arb1 has 0 reputation, arb2 has 2 — auto_assign should pick arb2
    let d3 = raise(&client, &env, pet_id, &owner, &target);
    let assigned = client.auto_assign_arbitrator(&d3);
    assert_eq!(assigned, arb2);
}

#[test]
fn test_auto_assign_excludes_dispute_parties() {
    let (env, client, admin, owner, target, arb1, pet_id) = setup();

    // Register owner and target as arbitrators (should be excluded)
    client.register_arbitrator(&admin, &owner);
    client.register_arbitrator(&admin, &target);
    client.register_arbitrator(&admin, &arb1);

    let dispute_id = raise(&client, &env, pet_id, &owner, &target);
    let assigned = client.auto_assign_arbitrator(&dispute_id);
    assert_ne!(assigned, owner);
    assert_ne!(assigned, target);
    assert_eq!(assigned, arb1);
}

#[test]
fn test_get_arbitrator_stats_returns_reputation() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    client.register_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&admin, &arbitrator);

    full_dispute(&client, &env, pet_id, &owner, &target, &arbitrator);

    let stats = client.get_arbitrator_stats(&arbitrator);
    assert_eq!(stats.reputation, 1);
}

#[test]
fn test_penalise_arbitrator_decreases_reputation() {
    let (env, client, admin, owner, target, arbitrator, pet_id) = setup();
    client.register_arbitrator(&admin, &arbitrator);
    client.assign_arbitrator(&admin, &arbitrator);

    full_dispute(&client, &env, pet_id, &owner, &target, &arbitrator);
    // reputation is now 1; penalise brings it back to 0
    client.penalise_arbitrator(&admin, &arbitrator);

    let stats = client.get_arbitrator_stats(&arbitrator);
    assert_eq!(stats.reputation, 0);
}
