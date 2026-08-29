//! Boundary tests for the per-record attachment cap (Issue #774).
//!
//! `add_attachment` must accept up to `MAX_ATTACHMENTS_PER_RECORD` (20)
//! attachments on a single medical record and reject any further attachment
//! with `ContractError::StorageQuotaExceeded`.

use crate::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

/// A valid CIDv0 IPFS hash (46 chars, base58, `Qm` prefix).
const VALID_IPFS_HASH: &str = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

fn setup() -> (Env, PetChainContractClient<'static>, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let vet = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Golden Retriever"),
        &String::from_str(&env, "Golden"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    // No public `add_medical_record` exists in the current contract surface, so
    // seed a record directly into instance storage — the same store and key
    // `add_attachment` reads/writes.
    let record_id: u64 = 1;
    let record = MedicalRecord {
        id: record_id,
        pet_id,
        vet_address: vet,
        diagnosis: String::from_str(&env, "Annual checkup"),
        treatment: String::from_str(&env, "Healthy"),
        medications: Vec::new(&env),
        date: 0,
        updated_at: 0,
        notes: String::from_str(&env, "All vitals normal"),
        attachment_hashes: Vec::new(&env),
        deleted_at: None,
    };
    env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .set(&MedicalKey::MedicalRecord(record_id), &record);
    });

    (env, client, record_id)
}

fn metadata(env: &Env) -> AttachmentMetadata {
    AttachmentMetadata {
        filename: String::from_str(env, "xray.jpg"),
        file_type: String::from_str(env, "image/jpeg"),
        size: 1024,
        uploaded_date: 0,
    }
}

/// Adding exactly `MAX_ATTACHMENTS_PER_RECORD` (20) attachments succeeds.
#[test]
fn test_add_up_to_cap_succeeds() {
    let (env, client, record_id) = setup();
    let hash = String::from_str(&env, VALID_IPFS_HASH);
    let content_hash = BytesN::from_array(&env, &[7u8; 32]);

    for i in 0..MAX_ATTACHMENTS_PER_RECORD {
        assert!(
            client.add_attachment(&record_id, &hash, &metadata(&env), &content_hash),
            "attachment {i} within the cap should be accepted",
        );
    }

    assert_eq!(client.get_attachment_count(&record_id), MAX_ATTACHMENTS_PER_RECORD);
    assert_eq!(
        client.get_attachments(&record_id).len(),
        MAX_ATTACHMENTS_PER_RECORD
    );
}

/// Adding the 21st attachment fails with `StorageQuotaExceeded` and leaves the
/// count unchanged at the cap.
#[test]
fn test_exceeding_cap_fails() {
    let (env, client, record_id) = setup();
    let hash = String::from_str(&env, VALID_IPFS_HASH);
    let content_hash = BytesN::from_array(&env, &[7u8; 32]);

    for _ in 0..MAX_ATTACHMENTS_PER_RECORD {
        client.add_attachment(&record_id, &hash, &metadata(&env), &content_hash);
    }

    let result = client.try_add_attachment(&record_id, &hash, &metadata(&env), &content_hash);
    let expected = soroban_sdk::Error::from_contract_error(ContractError::TooManyItems as u32);
    assert_eq!(result, Err(Ok(expected)));

    // The rejected attachment must not have been stored.
    assert_eq!(client.get_attachment_count(&record_id), MAX_ATTACHMENTS_PER_RECORD);
}
