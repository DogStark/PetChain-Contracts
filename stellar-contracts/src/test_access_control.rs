use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env,
};

#[test]
fn test_grant_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    let result = client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None);
    assert!(result);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::Basic);
}

#[test]
fn test_grant_access_with_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Max"),
        &String::from_str(&env, "2021-05-15"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Yellow"),
        &30u32,
        &None,
        &PrivacyLevel::Private,
    );

    let now = env.ledger().timestamp();
    let expires_at = now + 3600; // Expires in 1 hour

    let result = client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));
    assert!(result);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::Full);
}

#[test]
fn test_revoke_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2022-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);
    assert_eq!(client.check_access(&pet_id, &grantee), AccessLevel::Full);

    let result = client.revoke_access(&pet_id, &grantee);
    assert!(result);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::None);
}

#[test]
fn test_access_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2019-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Boxer"),
        &String::from_str(&env, "Brindle"),
        &28u32,
        &None,
        &PrivacyLevel::Private,
    );

    let now = 1000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let expires_at = now + 100;
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));

    assert_eq!(client.check_access(&pet_id, &grantee), AccessLevel::Full);

    env.ledger().with_mut(|l| l.timestamp = expires_at + 1);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::None);
}

#[test]
fn test_access_level_enforcement_basic() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Whiskers"),
        &String::from_str(&env, "2020-06-10"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Persian"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::Basic);
}

#[test]
fn test_access_level_enforcement_full() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Charlie"),
        &String::from_str(&env, "2018-11-22"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Tricolor"),
        &12u32,
        &None,
        &PrivacyLevel::Private,
    );

    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::Full);
}

#[test]
fn test_owner_has_full_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bella"),
        &String::from_str(&env, "2021-02-14"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Poodle"),
        &String::from_str(&env, "Black"),
        &10u32,
        &None,
        &PrivacyLevel::Private,
    );

    let access_level = client.check_access(&pet_id, &owner);
    assert_eq!(access_level, AccessLevel::Full);
}

#[test]
fn test_get_authorized_users() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee1 = Address::generate(&env);
    let grantee2 = Address::generate(&env);
    let grantee3 = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rocky"),
        &String::from_str(&env, "2020-08-05"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Bulldog"),
        &String::from_str(&env, "Brown"),
        &22u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    client.grant_access(&pet_id, &grantee1, &AccessLevel::Basic, &None);
    client.grant_access(&pet_id, &grantee2, &AccessLevel::Full, &None);
    client.grant_access(&pet_id, &grantee3, &AccessLevel::Basic, &None);

    let authorized_users = client.get_authorized_users(&pet_id);
    assert_eq!(authorized_users.len(), 3);
    assert!(authorized_users.contains(grantee1));
    assert!(authorized_users.contains(grantee2));
    assert!(authorized_users.contains(grantee3));
}

#[test]
fn test_get_authorized_users_excludes_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee1 = Address::generate(&env);
    let grantee2 = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Daisy"),
        &String::from_str(&env, "2019-12-25"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Maine Coon"),
        &String::from_str(&env, "Gray"),
        &7u32,
        &None,
        &PrivacyLevel::Private,
    );

    client.grant_access(&pet_id, &grantee1, &AccessLevel::Full, &None);
    client.grant_access(&pet_id, &grantee2, &AccessLevel::Basic, &None);

    client.revoke_access(&pet_id, &grantee1);

    let authorized_users = client.get_authorized_users(&pet_id);
    assert_eq!(authorized_users.len(), 1);
    assert!(!authorized_users.contains(grantee1));
    assert!(authorized_users.contains(grantee2));
}

#[test]
fn test_get_authorized_users_excludes_expired() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee1 = Address::generate(&env);
    let grantee2 = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2020-04-18"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Corgi"),
        &String::from_str(&env, "Orange"),
        &11u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    let now = 1000;
    env.ledger().with_mut(|l| l.timestamp = now);

    let expires_at = now + 100;
    client.grant_access(&pet_id, &grantee1, &AccessLevel::Full, &Some(expires_at));
    client.grant_access(&pet_id, &grantee2, &AccessLevel::Basic, &None);

    env.ledger().with_mut(|l| l.timestamp = expires_at + 1);

    let authorized_users = client.get_authorized_users(&pet_id);
    assert_eq!(authorized_users.len(), 1);
    assert!(!authorized_users.contains(grantee1));
    assert!(authorized_users.contains(grantee2));
}

#[test]
fn test_get_access_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Shadow"),
        &String::from_str(&env, "2021-07-30"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Black Cat"),
        &String::from_str(&env, "Black"),
        &6u32,
        &None,
        &PrivacyLevel::Private,
    );

    let now = env.ledger().timestamp();
    let expires_at = now + 7200;

    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));

    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert_eq!(grant.pet_id, pet_id);
    assert_eq!(grant.granter, owner);
    assert_eq!(grant.grantee, grantee);
    assert_eq!(grant.access_level, AccessLevel::Full);
    assert_eq!(grant.expires_at, Some(expires_at));
    assert!(grant.is_active);
}

#[test]
fn test_multiple_access_levels() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let family_member = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Cooper"),
        &String::from_str(&env, "2020-09-12"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "German Shepherd"),
        &String::from_str(&env, "Black and Tan"),
        &35u32,
        &None,
        &PrivacyLevel::Private,
    );

    client.grant_access(&pet_id, &vet, &AccessLevel::Full, &None);
    client.grant_access(&pet_id, &family_member, &AccessLevel::Basic, &None);

    assert_eq!(client.check_access(&pet_id, &vet), AccessLevel::Full);
    assert_eq!(
        client.check_access(&pet_id, &family_member),
        AccessLevel::Basic
    );
    assert_eq!(client.check_access(&pet_id, &owner), AccessLevel::Full);
}

#[test]
fn test_no_access_by_default() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Zoe"),
        &String::from_str(&env, "2022-01-10"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(&env, "Husky"),
        &String::from_str(&env, "Gray and White"),
        &24u32,
        &None,
        &PrivacyLevel::Private,
    );

    let access_level = client.check_access(&pet_id, &stranger);
    assert_eq!(access_level, AccessLevel::None);
}

#[test]
fn test_permanent_access() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Oscar"),
        &String::from_str(&env, "2019-05-20"),
        &Gender::Male,
        &Species::Cat,
        &String::from_str(&env, "Tabby"),
        &String::from_str(&env, "Orange"),
        &9u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert!(grant.expires_at.is_none());

    let now = env.ledger().timestamp();
    env.ledger().with_mut(|l| l.timestamp = now + 1_000_000);

    let access_level = client.check_access(&pet_id, &grantee);
    assert_eq!(access_level, AccessLevel::Full);
}
