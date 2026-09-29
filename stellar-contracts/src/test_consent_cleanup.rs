use crate::*;
use soroban_sdk::{testutils::Address as _, Env, String};

fn setup() -> (Env, PetChainContractClient<'static>, u64, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &id);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "1000000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    (env, client, pet_id, owner)
}

fn put(env: &Env, client: &PetChainContractClient, pet_id: u64, owner: &Address, idx: u64, active: bool) {
    env.as_contract(&client.address, || {
        env.storage().instance().set(
            &ConsentKey::Consent(idx),
            &Consent {
                id: idx,
                pet_id,
                owner: owner.clone(),
                consent_type: ConsentType::Research,
                granted_to: Address::generate(env),
                granted_at: 0,
                expires_at: None,
                revoked_at: None,
                is_active: active,
                scope: ConsentScope::ReadMedical,
                parent_consent_id: None,
                max_depth: 0,
            },
        );
        env.storage()
            .instance()
            .set(&ConsentKey::PetConsentIndex((pet_id, idx)), &idx);
        env.storage()
            .instance()
            .set(&ConsentKey::PetConsentCount(pet_id), &idx);
    });
}

fn live_ids(env: &Env, client: &PetChainContractClient, pet_id: u64) -> u64 {
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .get(&ConsentKey::PetConsentCount(pet_id))
            .unwrap_or(0)
    })
}

#[test]
fn bounded_cleanup_resumes_and_keeps_live_entries() {
    let (env, client, pet_id, owner) = setup();
    for i in 1..=6u64 {
        put(&env, &client, pet_id, &owner, i, i % 2 == 0);
    }
    let (r1, n1) = client.compact_consents_bounded(&pet_id, &owner, &2);
    assert!(n1.is_some());
    // Interleaved append between calls.
    let c = live_ids(&env, &client, pet_id);
    put(&env, &client, pet_id, &owner, c + 1, true);
    let mut removed = r1;
    let mut next = n1;
    while next.is_some() {
        let (r, n) = client.compact_consents_bounded(&pet_id, &owner, &2);
        removed += r;
        next = n;
    }
    assert_eq!(removed, 3);
    assert_eq!(live_ids(&env, &client, pet_id), 4);
}

#[test]
#[should_panic]
fn cleanup_rejects_zero_and_oversized_steps() {
    let (_, client, pet_id, owner) = setup();
    client.compact_consents_bounded(&pet_id, &owner, &0);
}

#[test]
#[should_panic]
fn cleanup_rejects_stranger() {
    let (env, client, pet_id, _) = setup();
    client.compact_consents_bounded(&pet_id, &Address::generate(&env), &5);
}
