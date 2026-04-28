use crate::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Env, Symbol, Vec,
};

#[test]
fn test_remove_pet_from_owner_index_missing_last_entry_does_not_panic() {
    // Simulates index inconsistency: PetCountByOwner says 2 but the last
    // index slot (index 2) is absent. remove_pet_from_owner_index must
    // return early instead of panicking.
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    // Register two pets so the owner index has two entries.
    let pet1 = client.register_pet(
        &owner,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "1000000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    let _pet2 = client.register_pet(
        &owner,
        &String::from_str(&env, "Beta"),
        &String::from_str(&env, "1000000"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Corrupt the index: remove the last slot entry (index 2) directly from
    // storage so the count says 2 but slot 2 is missing.
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .remove(&DataKey::OwnerPetIndex((owner.clone(), 2u64)));
    });

    // Initiate a transfer of pet1 — this calls remove_pet_from_owner_index
    // internally. With the fix it must complete without panicking.
    client.transfer_pet_ownership(&pet1, &new_owner);
    client.accept_pet_transfer(&pet1);

    // pet1 now belongs to new_owner; the call did not panic.
    assert_eq!(client.get_pet_owner(&pet1), Some(new_owner));
}

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
fn test_extend_access_grant_updates_expiry() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

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
        &PrivacyLevel::Private,
    );

    let now = env.ledger().timestamp();
    let expires_at = now + 3600;
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));

    let extended_expires_at = now + 7200;
    let result = client.extend_access_grant(&pet_id, &grantee, &Some(extended_expires_at));
    assert!(result);

    env.ledger().with_mut(|l| l.timestamp = expires_at + 1);
    assert_eq!(client.check_access(&pet_id, &grantee), AccessLevel::Full);

    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert_eq!(grant.expires_at, Some(extended_expires_at));
}

#[test]
fn test_extend_access_grant_cannot_extend_revoked_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let grantee = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Private,
    );

    let now = env.ledger().timestamp();
    let expires_at = now + 3600;
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));
    assert!(client.revoke_access(&pet_id, &grantee));

    let result = client.extend_access_grant(&pet_id, &grantee, &Some(expires_at + 3600));
    assert!(!result);

    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert_eq!(grant.is_active, false);
    assert_eq!(grant.expires_at, Some(expires_at));
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
fn test_revoke_all_access() {
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
        &String::from_str(&env, "BulkRevoke"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Color"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    client.grant_access(&pet_id, &grantee1, &AccessLevel::Basic, &None);
    client.grant_access(&pet_id, &grantee2, &AccessLevel::Full, &None);
    client.grant_access(&pet_id, &grantee3, &AccessLevel::Basic, &None);

    assert_eq!(client.get_authorized_users(&pet_id).len(), 3);

    client.revoke_all_access(&pet_id);

    assert_eq!(client.get_authorized_users(&pet_id).len(), 0);
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
fn test_get_pets_by_owner_single_owner_returns_only_owned_pets() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let other_owner = Address::generate(&env);

    client.register_pet(
        &owner,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &other_owner,
        &String::from_str(&env, "Other"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );

    let pets = client.get_pets_by_owner(&owner, &0u64, &10u32);
    assert_eq!(pets.len(), 1);
    assert_eq!(pets.get(0).unwrap().name, String::from_str(&env, "Alpha"));
}

#[test]
fn test_get_pets_by_owner_multiple_pets_returns_in_index_order() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.register_pet(
        &owner,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Beta"),
        &String::from_str(&env, "2020-02-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Gamma"),
        &String::from_str(&env, "2020-03-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Brown"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let pets = client.get_pets_by_owner(&owner, &0u64, &10u32);
    assert_eq!(pets.len(), 3);
    assert_eq!(pets.get(0).unwrap().name, String::from_str(&env, "Alpha"));
    assert_eq!(pets.get(1).unwrap().name, String::from_str(&env, "Beta"));
    assert_eq!(pets.get(2).unwrap().name, String::from_str(&env, "Gamma"));
}

#[test]
fn test_get_pets_by_owner_supports_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);

    client.register_pet(
        &owner,
        &String::from_str(&env, "Alpha"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Beta"),
        &String::from_str(&env, "2020-02-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "White"),
        &5u32,
        &None,
        &PrivacyLevel::Public,
    );
    client.register_pet(
        &owner,
        &String::from_str(&env, "Gamma"),
        &String::from_str(&env, "2020-03-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Beagle"),
        &String::from_str(&env, "Brown"),
        &12u32,
        &None,
        &PrivacyLevel::Public,
    );

    let page = client.get_pets_by_owner(&owner, &1u64, &2u32);
    assert_eq!(page.len(), 2);
    assert_eq!(page.get(0).unwrap().name, String::from_str(&env, "Beta"));
    assert_eq!(page.get(1).unwrap().name, String::from_str(&env, "Gamma"));

    assert_eq!(client.get_pets_by_owner(&owner, &5u64, &2u32).len(), 0);
    assert_eq!(client.get_pets_by_owner(&owner, &0u64, &0u32).len(), 0);
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

#[test]
fn test_access_logs_are_capped() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Bounded"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Retriever"),
        &String::from_str(&env, "Gold"),
        &20u32,
        &None,
        &PrivacyLevel::Private,
    );

    let log_owner = Address::generate(&env);
    let mut logs = Vec::new(&env);
    for id in 0..MAX_LOG_ENTRIES {
        logs.push_back(AccessLog {
            id: id as u64,
            pet_id,
            user: log_owner.clone(),
            action: AccessAction::Read,
            timestamp: id as u64,
            details: String::from_str(&env, "seed"),
        });
    }
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, "access_logs"), pet_id), &logs);
    });

    let grantee = Address::generate(&env);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Basic, &None);

    let logs = client.get_access_logs(&pet_id, &owner);
    assert_eq!(logs.len(), MAX_LOG_ENTRIES);
}

#[test]
fn test_access_logs_retain_newest_entries() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Recent"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Shorthair"),
        &String::from_str(&env, "Gray"),
        &6u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    let log_owner = Address::generate(&env);
    let mut logs = Vec::new(&env);
    for id in 0..MAX_LOG_ENTRIES {
        logs.push_back(AccessLog {
            id: id as u64,
            pet_id,
            user: log_owner.clone(),
            action: AccessAction::Read,
            timestamp: id as u64,
            details: String::from_str(&env, "seed"),
        });
    }
    env.as_contract(&contract_id, || {
        env.storage()
            .persistent()
            .set(&(Symbol::new(&env, "access_logs"), pet_id), &logs);
    });

    let grantee = Address::generate(&env);
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &None);

    let logs = client.get_access_logs(&pet_id, &owner);
    assert_eq!(logs.get(0).unwrap().id, 1);
    assert_eq!(logs.get(logs.len() - 1).unwrap().id, MAX_LOG_ENTRIES as u64);
    assert_eq!(logs.get(0).unwrap().action, AccessAction::Read);
    assert_eq!(
        logs.get(logs.len() - 1).unwrap().action,
        AccessAction::Grant
    );
}

#[test]
#[should_panic]
fn test_get_access_logs_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);

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

    client.get_access_logs(&pet_id, &stranger);
}

#[test]
fn test_get_vaccination_history_pagination_first_page() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Set up a verified vet
    let admin = Address::generate(&env);
    let vet = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1u32);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Test"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Add multiple vaccinations
    client.add_vaccination(
        &pet_id,
        &vet,
        &crate::VaccineType::Rabies,
        &String::from_str(&env, "RabiesVax"),
        &1000u64,
        &2000u64,
        &String::from_str(&env, "BATCH-001"),
    );
    client.add_vaccination(
        &pet_id,
        &vet,
        &crate::VaccineType::Parvovirus,
        &String::from_str(&env, "ParvoVax"),
        &1000u64,
        &2000u64,
        &String::from_str(&env, "BATCH-002"),
    );
    client.add_vaccination(
        &pet_id,
        &vet,
        &crate::VaccineType::Bordetella,
        &String::from_str(&env, "BordetellaVax"),
        &1000u64,
        &2000u64,
        &String::from_str(&env, "BATCH-003"),
    );

    // Test first page with limit 2
    let history = client.get_vaccination_history(&pet_id, &0u64, &2u32);
    assert_eq!(history.len(), 2);
    assert_eq!(
        history.get(0).unwrap().vaccine_type,
        crate::VaccineType::Rabies
    );
    assert_eq!(
        history.get(1).unwrap().vaccine_type,
        crate::VaccineType::Parvovirus
    );
}

#[test]
fn test_get_vaccination_history_pagination_out_of_bounds_offset() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Set up a verified vet
    let admin = Address::generate(&env);
    let vet = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1u32);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Test"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Add one vaccination
    client.add_vaccination(
        &pet_id,
        &vet,
        &crate::VaccineType::Rabies,
        &String::from_str(&env, "RabiesVax"),
        &1000u64,
        &2000u64,
        &String::from_str(&env, "BATCH-001"),
    );

    // Test out-of-bounds offset
    let history = client.get_vaccination_history(&pet_id, &10u64, &5u32);
    assert_eq!(history.len(), 0);
}

#[test]
fn test_get_vaccination_history_pagination_limit_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    );

    // Set up a verified vet
    let admin = Address::generate(&env);
    let vet = Address::generate(&env);
    let mut admins = soroban_sdk::Vec::new(&env);
    admins.push_back(admin.clone());
    client.init_multisig(&admin, &admins, &1u32);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Test"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // Add vaccinations
    client.add_vaccination(
        &pet_id,
        &vet,
        &crate::VaccineType::Rabies,
        &String::from_str(&env, "RabiesVax"),
        &1000u64,
        &2000u64,
        &String::from_str(&env, "BATCH-001"),
    );

    // Test limit of 0
    let history = client.get_vaccination_history(&pet_id, &0u64, &0u32);
    assert_eq!(history.len(), 0);
}

#[test]
fn test_get_verified_vets_only_returns_verified() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let vet1 = Address::generate(&env);
    let vet2 = Address::generate(&env);

    client.register_vet(
        &vet1,
        &String::from_str(&env, "Dr. Alice"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "Surgery"),
    );
    client.register_vet(
        &vet2,
        &String::from_str(&env, "Dr. Bob"),
        &String::from_str(&env, "LIC-002"),
        &String::from_str(&env, "Dentistry"),
    );

    // Only verify vet1
    client.verify_vet(&admin, &vet1);

    let vets = client.get_verified_vets(&0u64, &10u32);
    assert_eq!(vets.len(), 1);
    assert_eq!(vets.get(0).unwrap().address, vet1);
}

#[test]
fn test_check_and_expire_access_marks_expired_grant_inactive() {
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

    // Before expiry, grant is active
    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert!(grant.is_active);

    // Move time past expiry
    env.ledger().with_mut(|l| l.timestamp = expires_at + 1);

    // Call check_and_expire_access
    client.check_and_expire_access(&pet_id, &grantee);

    // Grant should now be inactive
    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert!(!grant.is_active);
}

#[test]
fn test_get_verified_vets_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let licenses = ["LIC-P1", "LIC-P2", "LIC-P3"];
    for lic in licenses.iter() {
        let vet = Address::generate(&env);
        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Vet"),
            &String::from_str(&env, lic),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&admin, &vet);
    }

    // Page 1: offset=0, limit=2
    let page1 = client.get_verified_vets(&0u64, &2u32);
    assert_eq!(page1.len(), 2);

    // Page 2: offset=2, limit=2
    let page2 = client.get_verified_vets(&2u64, &2u32);
    assert_eq!(page2.len(), 1);
}

#[test]
fn test_check_and_expire_access_does_not_affect_active_grant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

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
    let expires_at = now + 1000; // Far in the future
    client.grant_access(&pet_id, &grantee, &AccessLevel::Full, &Some(expires_at));

    // Call check_and_expire_access before expiry
    client.check_and_expire_access(&pet_id, &grantee);

    // Grant should still be active
    let grant = client.get_access_grant(&pet_id, &grantee).unwrap();
    assert!(grant.is_active);
    assert_eq!(grant.access_level, AccessLevel::Full);
}

#[test]
fn test_custody_history_multiple_updates() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Unverified"),
        &String::from_str(&env, "LIC-UNVER"),
        &String::from_str(&env, "General"),
    );

    let vets = client.get_verified_vets(&0u64, &10u32);
    assert_eq!(vets.len(), 0);
    let owner = Address::generate(&env);
    let custodian1 = Address::generate(&env);
    let custodian2 = Address::generate(&env);

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

    let mut perms1 = Vec::new(&env);
    perms1.push_back(String::from_str(&env, "feed"));

    let mut perms2 = Vec::new(&env);
    perms2.push_back(String::from_str(&env, "walk"));
    perms2.push_back(String::from_str(&env, "groom"));

    // Grant custody to custodian1
    client.grant_temporary_custody(&pet_id, &custodian1, &1000u64, &2000u64, &perms1);

    // Grant custody to custodian2 (overwrites current but history preserves both)
    client.grant_temporary_custody(&pet_id, &custodian2, &2500u64, &3500u64, &perms2);

    let history = client.get_custody_history(&pet_id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().custodian, custodian1);
    assert!(history.get(0).unwrap().is_active);
    assert_eq!(history.get(1).unwrap().custodian, custodian2);
    assert!(history.get(1).unwrap().is_active);
}

#[test]
fn test_custody_history_appended_correctly() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let custodian = Address::generate(&env);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Luna"),
        &String::from_str(&env, "2021-03-20"),
        &Gender::Female,
        &Species::Cat,
        &String::from_str(&env, "Siamese"),
        &String::from_str(&env, "Cream"),
        &8u32,
        &None,
        &PrivacyLevel::Restricted,
    );

    let mut perms = Vec::new(&env);
    perms.push_back(String::from_str(&env, "medical"));

    // First custody grant
    let custody1 = client.grant_temporary_custody(&pet_id, &custodian, &1000u64, &2000u64, &perms);
    assert!(custody1.is_active);

    // Second custody grant to same custodian with different dates
    let custody2 = client.grant_temporary_custody(&pet_id, &custodian, &3000u64, &4000u64, &perms);
    assert!(custody2.is_active);

    let history = client.get_custody_history(&pet_id);
    assert_eq!(history.len(), 2);

    // Verify order and data
    let first = history.get(0).unwrap();
    let second = history.get(1).unwrap();

    assert_eq!(first.start_date, 1000u64);
    assert_eq!(first.end_date, 2000u64);
    assert_eq!(second.start_date, 3000u64);
    assert_eq!(second.end_date, 4000u64);
}

#[test]
fn test_get_custody_history_returns_complete_history() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let custodian = Address::generate(&env);

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

    let mut perms = Vec::new(&env);
    perms.push_back(String::from_str(&env, "exercise"));

    // Grant custody
    client.grant_temporary_custody(&pet_id, &custodian, &1000u64, &2000u64, &perms);

    // Revoke custody - this should append the revoked snapshot to history
    client.revoke_temporary_custody(&pet_id);

    // History should contain both the grant and the revocation snapshot
    let history = client.get_custody_history(&pet_id);
    assert_eq!(history.len(), 2);

    let first = history.get(0).unwrap();
    let second = history.get(1).unwrap();

    // First entry: active grant
    assert!(first.is_active);
    assert_eq!(first.custodian, custodian);

    // Second entry: revoked snapshot (inactive)
    assert!(!second.is_active);
    assert_eq!(second.custodian, custodian);

    // Verify current custody is inactive
    assert!(!client.is_custody_valid(&pet_id));
}

#[test]
fn test_is_vet_registered_distinguishes_from_unregistered() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let vet = Address::generate(&env);
    let unregistered_vet = Address::generate(&env);

    client.register_vet(
        &vet,
        &String::from_str(&env, "Vet Name"),
        &String::from_str(&env, "LIC123"),
        &String::from_str(&env, "General"),
    );

    assert!(client.is_vet_registered(&vet));
    assert!(!client.is_verified_vet(&vet));

    assert!(!client.is_vet_registered(&unregistered_vet));
}

#[test]
fn test_get_vaccination_summary() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let admin = Address::generate(&env);
    client.init_admin(&admin);

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

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Smith"),
        &String::from_str(&env, "LIC123"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    // 1. No vaccinations - should be fully current
    let summary = client.get_vaccination_summary(&pet_id);
    assert!(summary.is_fully_current);
    assert_eq!(summary.overdue_types.len(), 0);
    assert_eq!(summary.upcoming_count, 0);

    // 2. Add an upcoming vaccination (due in 15 days)
    let now = 1000u64;
    env.ledger().with_mut(|l| l.timestamp = now);
    
    client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies v1"),
        &now,
        &(now + 15 * 86400), // Due in 15 days
        &String::from_str(&env, "BATCH001"),
    );

    let summary2 = client.get_vaccination_summary(&pet_id);
    assert!(summary2.is_fully_current);
    assert_eq!(summary2.upcoming_count, 1);

    // 3. Move time forward so it's overdue
    env.ledger().with_mut(|l| l.timestamp = now + 20 * 86400);
    
    let summary3 = client.get_vaccination_summary(&pet_id);
    assert!(!summary3.is_fully_current);
    assert_eq!(summary3.overdue_types.len(), 1);
    assert_eq!(summary3.overdue_types.get(0).unwrap(), VaccineType::Rabies);
}
