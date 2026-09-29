//! Issue #1216: evidence paths enforce roles, dispute state and limits.

use crate::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    let owner = Address::generate(&env);
    let target = Address::generate(&env);
    let d = client.raise_dispute(
        &1u64,
        &owner,
        &target,
        &100u64,
        &String::from_str(&env, "reason"),
        &String::from_str(&env, "ipfs://x"),
    );
    (env, client, admin, owner, target, d)
}

fn hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[7u8; 32])
}

#[test]
fn self_dispute_rejected() {
    let (env, client, _a, owner, _t, _d) = setup();
    let r = client.try_raise_dispute(
        &1u64,
        &owner,
        &owner,
        &1u64,
        &String::from_str(&env, "r"),
        &String::from_str(&env, "h"),
    );
    assert!(r.is_err());
}

#[test]
fn evidence_requires_evidence_phase() {
    let (env, client, _a, owner, _t, d) = setup();
    let cid = String::from_str(&env, "cid");
    assert!(client.try_submit_evidence(&d, &owner, &cid, &hash(&env)).is_err());
    client.open_evidence_phase(&owner, &d);
    client.submit_evidence(&d, &owner, &cid, &hash(&env));
}

#[test]
fn outsider_cannot_submit_open_phase_or_read() {
    let (env, client, _a, owner, _t, d) = setup();
    let outsider = Address::generate(&env);
    assert!(client.try_open_evidence_phase(&outsider, &d).is_err());
    client.open_evidence_phase(&owner, &d);
    let cid = String::from_str(&env, "cid");
    assert!(client.try_submit_evidence(&d, &outsider, &cid, &hash(&env)).is_err());
    assert!(client.try_get_dispute_evidence(&outsider, &d).is_err());
}

#[test]
fn open_phase_is_not_repeatable() {
    let (_env, client, _a, owner, _t, d) = setup();
    client.open_evidence_phase(&owner, &d);
    assert!(client.try_open_evidence_phase(&owner, &d).is_err());
}

#[test]
fn invalid_cid_and_per_party_limit() {
    let (env, client, _a, owner, _t, d) = setup();
    client.open_evidence_phase(&owner, &d);
    assert!(client
        .try_submit_evidence(&d, &owner, &String::from_str(&env, ""), &hash(&env))
        .is_err());
    let long = String::from_str(&env, &"a".repeat(129));
    assert!(client.try_submit_evidence(&d, &owner, &long, &hash(&env)).is_err());
    let cid = String::from_str(&env, "cid");
    for _ in 0..10 {
        client.submit_evidence(&d, &owner, &cid, &hash(&env));
    }
    assert!(client.try_submit_evidence(&d, &owner, &cid, &hash(&env)).is_err());
}

#[test]
fn parties_and_admin_can_read_evidence() {
    let (env, client, admin, owner, target, d) = setup();
    client.open_evidence_phase(&target, &d);
    client.submit_evidence(&d, &owner, &String::from_str(&env, "cid"), &hash(&env));
    assert_eq!(client.get_dispute_evidence(&owner, &d).len(), 1);
    assert_eq!(client.get_dispute_evidence(&target, &d).len(), 1);
    assert_eq!(client.get_dispute_evidence(&admin, &d).len(), 1);
}
