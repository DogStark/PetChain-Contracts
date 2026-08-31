// Tests for cursor-based access-grant enumeration (Issue #1161).
use crate::{AccessLevel, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup(env: &Env) -> (PetChainContractClient<'_>, Address, u64) {
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(env, &contract_id);

    let owner = Address::generate(env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(env, "GrantedPet"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(env, "Persian"),
        &String::from_str(env, "White"),
        &5,
        &None,
        &PrivacyLevel::Public,
    );

    (client, owner, pet_id)
}

fn grant(
    client: &PetChainContractClient<'_>,
    env: &Env,
    owner: &Address,
    pet_id: u64,
    grantee: &Address,
    expires_at: Option<u64>,
) {
    let nonce = client.get_caller_nonce(owner);
    client.grant_access(
        &pet_id,
        grantee,
        &AccessLevel::Basic,
        &expires_at,
        &nonce,
    );
    let _ = env;
}

#[test]
fn test_first_page() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);

    for _ in 0..10 {
        let grantee = Address::generate(&env);
        grant(&client, &env, &owner, pet_id, &grantee, None);
    }

    let page = client.get_pet_access_grants_cursor(&pet_id, &0, &5, &false);
    assert_eq!(page.items.len(), 5);
    assert_eq!(page.total_slots, 10);
    assert_ne!(page.next_cursor, 0);
}

#[test]
fn test_pages_cover_all_without_duplicates() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);

    for _ in 0..7 {
        let grantee = Address::generate(&env);
        grant(&client, &env, &owner, pet_id, &grantee, None);
    }

    let mut seen = 0u32;
    let mut cursor = 0u64;
    loop {
        let page = client.get_pet_access_grants_cursor(&pet_id, &cursor, &3, &false);
        seen += page.items.len() as u32;
        cursor = page.next_cursor;
        if cursor == 0 {
            break;
        }
    }
    assert_eq!(seen, 7);
}

#[test]
fn test_active_only_filters_revoked_and_expired() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);

    let active_grantee = Address::generate(&env);
    let revoked_grantee = Address::generate(&env);
    let expired_grantee = Address::generate(&env);

    grant(&client, &env, &owner, pet_id, &active_grantee, None);
    grant(&client, &env, &owner, pet_id, &revoked_grantee, None);
    grant(&client, &env, &owner, pet_id, &expired_grantee, Some(1));

    client.revoke_access(&pet_id, &revoked_grantee);
    env.ledger().with_mut(|l| l.timestamp = 1000);

    let page = client.get_pet_access_grants_cursor(&pet_id, &0, &10, &true);
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items.get(0).unwrap().grantee, active_grantee);
    // Slot count still reflects all grants issued, active or not.
    assert_eq!(page.total_slots, 3);
}

#[test]
fn test_empty_when_no_grants() {
    let env = Env::default();
    let (client, _owner, pet_id) = setup(&env);

    let page = client.get_pet_access_grants_cursor(&pet_id, &0, &10, &false);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.next_cursor, 0);
    assert_eq!(page.total_slots, 0);
}

#[test]
fn test_cursor_past_end_returns_empty() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);

    let grantee = Address::generate(&env);
    grant(&client, &env, &owner, pet_id, &grantee, None);

    let page = client.get_pet_access_grants_cursor(&pet_id, &1, &10, &false);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.next_cursor, 0);
}

#[test]
fn test_zero_limit_returns_empty_page() {
    let env = Env::default();
    let (client, owner, pet_id) = setup(&env);

    let grantee = Address::generate(&env);
    grant(&client, &env, &owner, pet_id, &grantee, None);

    let page = client.get_pet_access_grants_cursor(&pet_id, &0, &0, &false);
    assert_eq!(page.items.len(), 0);
    assert_eq!(page.next_cursor, 0);
    assert_eq!(page.total_slots, 1);
}
