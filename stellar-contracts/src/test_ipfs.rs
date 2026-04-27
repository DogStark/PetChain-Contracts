use crate::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_validate_ipfs_hash_v0_success() {
    let env = Env::default();
    let valid_v0 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &valid_v0),
        Ok(())
    );
}

#[test]
fn test_validate_ipfs_hash_v1_success() {
    let env = Env::default();
    // CIDv1 base32
    let valid_v1 = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &valid_v1),
        Ok(())
    );
}

#[test]
fn test_validate_ipfs_hash_too_short() {
    let env = Env::default();
    let invalid = String::from_str(&env, "QmTooShort");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_invalid_prefix() {
    let env = Env::default();
    // 46 chars but starts with Am
    let invalid = String::from_str(&env, "AmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_invalid_chars() {
    let env = Env::default();
    // 46 chars, starts with Qm, but contains '0' (invalid Base58)
    let invalid = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbd0");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v1_invalid_chars() {
    let env = Env::default();
    // Starts with b, but contains '1' (invalid Base32)
    let invalid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzd1",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v1_too_short() {
    let env = Env::default();
    let invalid = String::from_str(&env, "b");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_boundary_length() {
    let env = Env::default();
    let invalid = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbd");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_sightings_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "LostPet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Color"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    let alert_id = client.report_lost(&pet_id, &String::from_str(&env, "Park"), &None);

    // Add 5 sightings
    for _i in 0..5 {
        client.report_sighting(
            &alert_id,
            &String::from_str(&env, "Location"),
            &String::from_str(&env, "Sighting "),
        );
    }

    assert_eq!(client.get_sighting_count(&alert_id), 5);

    let page1 = client.get_sightings_paginated(&alert_id, &0u64, &2u32);
    assert_eq!(page1.len(), 2);

    let page2 = client.get_sightings_paginated(&alert_id, &2u64, &2u32);
    assert_eq!(page2.len(), 2);

    let page3 = client.get_sightings_paginated(&alert_id, &4u64, &2u32);
    assert_eq!(page3.len(), 1);

    let empty = client.get_sightings_paginated(&alert_id, &10u64, &2u32);
    assert_eq!(empty.len(), 0);
}

#[test]
fn test_update_lost_alert() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    client.init_admin(&owner);

    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "LostPet"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Breed"),
        &String::from_str(&env, "Color"),
        &10u32,
        &None,
        &PrivacyLevel::Public,
    );

    let alert_id = client.report_lost(&pet_id, &String::from_str(&env, "Park"), &Some(100u64));

    let new_location = String::from_str(&env, "Downtown");
    let new_reward = Some(200u64);

    client.update_lost_alert(&alert_id, &new_location, &new_reward);

    let alert = client.get_alert(&alert_id).unwrap();
    assert_eq!(alert.last_seen_location, new_location);
    assert_eq!(alert.reward_amount, new_reward);
}
