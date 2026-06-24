use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

/// Sets up a contract with a single admin, an owner (claimer), and an
/// opposing party (target), plus a pet and an open dispute between them.
fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let owner = Address::generate(&env);
    let target = Address::generate(&env);

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

    let dispute_id = client.raise_dispute(
        &pet_id,
        &owner,
        &target,
        &1_000u64,
        &String::from_str(&env, "Pet not delivered"),
        &String::from_str(&env, "ipfs://initial"),
    );

    (env, client, admin, owner, target, dispute_id)
}

#[test]
fn test_unanimous_approval_auto_resolves_in_favor_of_claimer() {
    let (_env, client, admin, owner, target, dispute_id) = setup();

    // Owner (claimer) and admin both approve -> 2 of 3 stakeholders agree.
    let resolved_by_owner = client.vote_on_dispute(&owner, &dispute_id, &DisputeVote::Approve);
    assert!(!resolved_by_owner, "single vote must not resolve the dispute");
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::Pending
    );

    let resolved_by_admin = client.vote_on_dispute(&admin, &dispute_id, &DisputeVote::Approve);
    assert!(resolved_by_admin, "second matching vote must auto-resolve");

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::ResolvedInFavorOfClaimer);
    assert!(dispute.resolved_at.is_some());

    // The third stakeholder's vote is irrelevant once resolved, but the
    // vote ledger should reflect both recorded votes.
    let votes = client.get_dispute_votes(&dispute_id);
    assert_eq!(votes.len(), 2);

    // Sanity: target was never asked to vote here.
    let _ = target;
}

#[test]
fn test_split_vote_does_not_resolve_dispute() {
    let (_env, client, admin, owner, target, dispute_id) = setup();

    // Owner approves (favors claimer), target rejects (favors target).
    let r1 = client.vote_on_dispute(&owner, &dispute_id, &DisputeVote::Approve);
    let r2 = client.vote_on_dispute(&target, &dispute_id, &DisputeVote::Reject);

    assert!(!r1);
    assert!(!r2);

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::Pending);
    assert!(dispute.resolved_at.is_none());

    let votes = client.get_dispute_votes(&dispute_id);
    assert_eq!(votes.len(), 2);

    // Admin breaks the tie in favor of the target.
    let r3 = client.vote_on_dispute(&admin, &dispute_id, &DisputeVote::Reject);
    assert!(r3, "admin's matching vote should reach the 2-of-3 threshold");

    let resolved = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(resolved.status, DisputeStatus::ResolvedInFavorOfTarget);
}

#[test]
fn test_admin_override_resolves_without_votes() {
    let (_env, client, admin, owner, _target, dispute_id) = setup();

    // Owner casts a single vote; threshold is not met.
    client.vote_on_dispute(&owner, &dispute_id, &DisputeVote::Approve);
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::Pending
    );

    // Admin override bypasses the consensus mechanism entirely.
    let overridden = client.resolve_dispute(&dispute_id, &DisputeStatus::ResolvedInFavorOfTarget);
    assert!(overridden);

    let dispute = client.get_dispute(&dispute_id).unwrap();
    assert_eq!(dispute.status, DisputeStatus::ResolvedInFavorOfTarget);
    assert!(dispute.resolved_at.is_some());
}

#[test]
#[should_panic(expected = "Only the pet owner, the opposing party, or an admin may vote")]
fn test_non_stakeholder_cannot_vote() {
    let (env, client, _admin, _owner, _target, dispute_id) = setup();
    let stranger = Address::generate(&env);
    client.vote_on_dispute(&stranger, &dispute_id, &DisputeVote::Approve);
}

#[test]
fn test_revoting_updates_existing_vote_instead_of_double_counting() {
    let (_env, client, admin, owner, target, dispute_id) = setup();

    client.vote_on_dispute(&owner, &dispute_id, &DisputeVote::Approve);
    // Owner changes their mind before the threshold is reached.
    client.vote_on_dispute(&owner, &dispute_id, &DisputeVote::Reject);

    let votes = client.get_dispute_votes(&dispute_id);
    assert_eq!(votes.len(), 1, "changing a vote must not create a duplicate entry");
    assert_eq!(votes.get(0).unwrap().vote, DisputeVote::Reject);

    // Target agrees with the (updated) reject vote -> resolves in favor of target.
    let resolved = client.vote_on_dispute(&target, &dispute_id, &DisputeVote::Reject);
    assert!(resolved);
    assert_eq!(
        client.get_dispute(&dispute_id).unwrap().status,
        DisputeStatus::ResolvedInFavorOfTarget
    );

    let _ = admin;
}
