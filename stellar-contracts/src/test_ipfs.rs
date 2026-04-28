use crate::*;
use soroban_sdk::{testutils::{Address as _, Ledger as _}, Address, Env, String};

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
fn test_validate_ipfs_hash_empty_string() {
    let env = Env::default();
    let invalid = String::from_str(&env, "");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_too_long() {
    let env = Env::default();
    // 129 chars starting with 'b' — exceeds CIDv1 max of 128
    let invalid = String::from_str(
        &env,
        "baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_excluded_base58_char_l() {
    let env = Env::default();
    // 'l' (lowercase L) is excluded from Base58
    let invalid = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdl");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_excluded_base58_char_O() {
    let env = Env::default();
    // 'O' (uppercase O) is excluded from Base58
    let invalid = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdO");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v0_excluded_base58_char_I() {
    let env = Env::default();
    // 'I' (uppercase I) is excluded from Base58
    let invalid = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdI");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v1_invalid_char_8() {
    let env = Env::default();
    // '8' is not in Base32 alphabet (a-z, 2-7)
    let invalid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzd8",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v1_uppercase_rejected() {
    let env = Env::default();
    // CIDv1 base32 must be lowercase; uppercase 'B' in body is invalid
    let invalid = String::from_str(
        &env,
        "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdB",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
fn test_validate_ipfs_hash_v1_max_length_valid() {
    let env = Env::default();
    // 128 chars, starts with 'b', all valid Base32 chars
    let hash = String::from_str(
        &env,
        "baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &hash),
        Ok(())
    );
}

#[test]
fn test_validate_ipfs_hash_v1_min_length_valid() {
    let env = Env::default();
    // 2 chars — minimum valid CIDv1 (starts with 'b', one valid body char)
    let hash = String::from_str(&env, "ba");
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &hash),
        Ok(())
    );
}

#[test]
fn test_validate_ipfs_hash_wrong_prefix_not_qm_or_b() {
    let env = Env::default();
    // Starts with 'z' — neither CIDv0 (Qm) nor CIDv1 (b)
    let invalid = String::from_str(
        &env,
        "zafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );
    assert_eq!(
        PetChainContract::validate_ipfs_hash(&env, &invalid),
        Err(ContractError::InvalidIpfsHash)
    );
}

#[test]
#[should_panic]
fn test_add_pet_photo_panics_on_invalid_hash() {
    let (env, client, _owner, pet_id) = setup_pet_test_env();
    let bad_hash = String::from_str(&env, "not-a-valid-ipfs-hash");
    client.add_pet_photo(&pet_id, &bad_hash);
}



fn setup_pet_test_env() -> (Env, PetChainContractClient<'static>, Address, u64) {
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

    (env, client, owner, pet_id)
}

#[test]
fn test_add_pet_photo_success() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo_hash = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let result = client.add_pet_photo(&pet_id, &photo_hash);
    assert!(result);

    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 1);
    assert_eq!(photos.get(0).unwrap(), photo_hash);
}

#[test]
fn test_add_multiple_pet_photos() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo1 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let photo2 = String::from_str(&env, "QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");
    let photo3 = String::from_str(&env, "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB");

    assert!(client.add_pet_photo(&pet_id, &photo1));
    assert!(client.add_pet_photo(&pet_id, &photo2));
    assert!(client.add_pet_photo(&pet_id, &photo3));

    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 3);
    assert_eq!(photos.get(0).unwrap(), photo1);
    assert_eq!(photos.get(1).unwrap(), photo2);
    assert_eq!(photos.get(2).unwrap(), photo3);
}

#[test]
fn test_remove_pet_photo_success() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo1 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let photo2 = String::from_str(&env, "QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");

    client.add_pet_photo(&pet_id, &photo1);
    client.add_pet_photo(&pet_id, &photo2);

    // Verify both photos exist
    let photos_before = client.get_pet_photos(&pet_id);
    assert_eq!(photos_before.len(), 2);

    // Remove first photo
    let result = client.remove_pet_photo(&pet_id, &photo1);
    assert!(result);

    // Verify only second photo remains
    let photos_after = client.get_pet_photos(&pet_id);
    assert_eq!(photos_after.len(), 1);
    assert_eq!(photos_after.get(0).unwrap(), photo2);
}

#[test]
fn test_remove_pet_photo_nonexistent_hash() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let nonexistent = String::from_str(&env, "QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");

    client.add_pet_photo(&pet_id, &photo);

    // Try to remove non-existent photo
    let result = client.remove_pet_photo(&pet_id, &nonexistent);
    assert!(!result);

    // Verify original photo still exists
    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 1);
    assert_eq!(photos.get(0).unwrap(), photo);
}

#[test]
fn test_remove_pet_photo_nonexistent_pet() {
    let (env, client, _owner, _pet_id) = setup_pet_test_env();

    let photo = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let result = client.remove_pet_photo(&9999u64, &photo);
    assert!(!result);
}

#[test]
fn test_remove_pet_photo_from_multiple() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo1 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let photo2 = String::from_str(&env, "QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");
    let photo3 = String::from_str(&env, "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB");

    client.add_pet_photo(&pet_id, &photo1);
    client.add_pet_photo(&pet_id, &photo2);
    client.add_pet_photo(&pet_id, &photo3);

    // Remove middle photo
    let result = client.remove_pet_photo(&pet_id, &photo2);
    assert!(result);

    // Verify order is preserved
    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 2);
    assert_eq!(photos.get(0).unwrap(), photo1);
    assert_eq!(photos.get(1).unwrap(), photo3);
}

#[test]
fn test_remove_pet_photo_all_photos() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo1 = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");
    let photo2 = String::from_str(&env, "QmT5NvUtoM5nWFfrQdVrFtvGfKFmG7AHE8P34isapyhCxX");

    client.add_pet_photo(&pet_id, &photo1);
    client.add_pet_photo(&pet_id, &photo2);

    // Remove both photos
    assert!(client.remove_pet_photo(&pet_id, &photo1));
    assert!(client.remove_pet_photo(&pet_id, &photo2));

    // Verify no photos remain
    let photos = client.get_pet_photos(&pet_id);
    assert_eq!(photos.len(), 0);
}

#[test]
fn test_remove_pet_photo_updates_timestamp() {
    let (env, client, owner, pet_id) = setup_pet_test_env();

    let photo = String::from_str(&env, "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG");

    // Add photo and get initial timestamp
    client.add_pet_photo(&pet_id, &photo);
    let pet_after_add = client.get_pet(&pet_id, &owner).unwrap();
    let timestamp_after_add = pet_after_add.updated_at;

    // Advance time
    env.ledger().set_timestamp(env.ledger().timestamp() + 1000);

    // Remove photo
    client.remove_pet_photo(&pet_id, &photo);
    let pet_after_remove = client.get_pet(&pet_id, &owner).unwrap();
    let timestamp_after_remove = pet_after_remove.updated_at;

    // Verify timestamp was updated
    assert!(timestamp_after_remove > timestamp_after_add);
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
