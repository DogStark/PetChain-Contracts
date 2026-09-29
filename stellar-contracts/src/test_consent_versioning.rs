use crate::*;
use soroban_sdk::{testutils::Address as _, Env};

fn setup() -> (Env, PetChainContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    (env, client, admin)
}

fn seed_consent(env: &Env, client: &PetChainContractClient, owner: &Address) {
    env.as_contract(&client.address, || {
        env.storage().instance().set(
            &ConsentKey::Consent(1),
            &Consent {
                id: 1,
                pet_id: 1,
                owner: owner.clone(),
                consent_type: ConsentType::Research,
                granted_to: Address::generate(env),
                granted_at: 0,
                expires_at: None,
                revoked_at: None,
                is_active: true,
                scope: ConsentScope::ReadMedical,
                parent_consent_id: None,
                max_depth: 0,
            },
        );
    });
}

#[test]
fn bump_makes_consent_stale_until_renewed() {
    let (env, client, admin) = setup();
    let owner = Address::generate(&env);
    seed_consent(&env, &client, &owner);
    assert!(client.is_consent_current(&1));
    assert_eq!(client.bump_consent_policy_version(&admin), 2);
    assert!(!client.is_consent_current(&1));
    assert_eq!(client.renew_consent_version(&1, &owner), 2);
    assert!(client.is_consent_current(&1));
}

#[test]
#[should_panic]
fn bump_requires_admin() {
    let (env, client, _) = setup();
    client.bump_consent_policy_version(&Address::generate(&env));
}

#[test]
#[should_panic]
fn renew_requires_owner() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    seed_consent(&env, &client, &owner);
    client.renew_consent_version(&1, &Address::generate(&env));
}

#[test]
fn unknown_consent_is_not_current() {
    let (_, client, _) = setup();
    assert!(!client.is_consent_current(&99));
}
