// Tests for binding decryption tokens to encryption key versions
// (Issue #1163): tokens issued under one key version must not verify
// once the pet's key has been rotated to a new version.
use crate::{AccessLevel, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let owner = Address::generate(env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "TokenPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &10,
        &None,
        &PrivacyLevel::Public,
    );

    (client, owner, pet_id)
}

#[test]
fn test_new_pet_starts_at_key_version_one() {
    let env = Env::default();
    let (client, _owner, pet_id) = setup(&env);
    assert_eq!(client.get_pet_key_version(&pet_id), 1);
}

#[test]
fn test_freshly_issued_token_verifies() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &3600, &nonce);

    assert!(client.verify_decryption_token(&pet_id, &delegate));
}

#[test]
fn test_rotation_invalidates_prior_token() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &3600, &nonce);
    assert!(client.verify_decryption_token(&pet_id, &delegate));

    let rotate_nonce = client.get_caller_nonce(&owner);
    let new_version = client.rotate_pet_key_version(&pet_id, &rotate_nonce);
    assert_eq!(new_version, 2);
    assert_eq!(client.get_pet_key_version(&pet_id), 2);

    // Token was issued under version 1; it must not verify against
    // the rotated version, deterministically, without any further
    // per-delegate cleanup call.
    assert!(!client.verify_decryption_token(&pet_id, &delegate));
}

#[test]
fn test_reissued_token_after_rotation_verifies_again() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &3600, &nonce);

    let rotate_nonce = client.get_caller_nonce(&owner);
    client.rotate_pet_key_version(&pet_id, &rotate_nonce);
    assert!(!client.verify_decryption_token(&pet_id, &delegate));

    // Re-delegating after the rotation binds a fresh token to the new
    // version, restoring access deterministically.
    let redelegate_nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &3600, &redelegate_nonce);
    assert!(client.verify_decryption_token(&pet_id, &delegate));
}

#[test]
fn test_expired_token_does_not_verify() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &100, &nonce);
    assert!(client.verify_decryption_token(&pet_id, &delegate));

    env.ledger().with_mut(|l| l.timestamp = 1000);
    assert!(!client.verify_decryption_token(&pet_id, &delegate));
}

#[test]
fn test_explicit_revocation_invalidates_token() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &3600, &nonce);
    assert!(client.verify_decryption_token(&pet_id, &delegate));

    assert!(client.revoke_decryption_delegation(&pet_id, &delegate));
    assert!(!client.verify_decryption_token(&pet_id, &delegate));
    // Revoking again is a no-op, not a panic.
    assert!(!client.revoke_decryption_delegation(&pet_id, &delegate));
}

#[test]
fn test_unknown_delegate_never_verifies() {
    let env = Env::default();
    let (client, _owner, pet_id) = setup(&env);
    let stranger = Address::generate(&env);
    assert!(!client.verify_decryption_token(&pet_id, &stranger));
}

#[test]
#[should_panic]
fn test_zero_ttl_rejected() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);
    let delegate = Address::generate(&env);

    let nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &delegate, &0, &nonce);
}

/// Combined lifecycle cascade (Issue #1162 + #1163): after an ownership
/// transfer and a subsequent key rotation by the new owner, a delegate
/// that held both view access and a decryption token loses both.
#[test]
fn test_combined_transfer_then_rotation_leaves_delegate_fully_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    // Restricted so that view access actually depends on an AccessGrant
    // rather than being open to everyone as with PrivacyLevel::Public.
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "CascadePet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &7,
        &None,
        &PrivacyLevel::Restricted,
    );
    let grantee = Address::generate(&env);
    let new_owner = Address::generate(&env);

    let grant_nonce = client.get_caller_nonce(&owner);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None, &grant_nonce);
    let delegate_nonce = client.get_caller_nonce(&owner);
    client.delegate_decryption_access(&pet_id, &grantee, &3600, &delegate_nonce);

    assert!(client.get_pet(&pet_id, &grantee).is_some());
    assert!(client.verify_decryption_token(&pet_id, &grantee));

    let transfer_nonce = client.get_caller_nonce(&owner);
    client.transfer_pet_ownership(&pet_id, &new_owner, &transfer_nonce);
    client.accept_pet_transfer(&pet_id);

    let rotate_nonce = client.get_caller_nonce(&new_owner);
    client.rotate_pet_key_version(&pet_id, &rotate_nonce);

    assert!(
        client.get_pet(&pet_id, &grantee).is_none(),
        "view access must be gone after ownership transfer"
    );
    assert!(
        !client.verify_decryption_token(&pet_id, &grantee),
        "decryption capability must be gone after the new owner rotates the key"
    );
}
