// Confirmed purge: role, replay, scope, hold and audit-privacy tests
// (Issue #1344).
use crate::{
    ContractError, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, PurgeAuditEvent,
    PurgeRole, Species, MAX_PURGE_BATCH, PURGE_RECORDS_DIGEST_DOMAIN,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _},
    vec,
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, Error, String, Symbol, TryFromVal, Vec,
};

const RETENTION: u64 = 100;
const DIAGNOSIS: &str = "SECRET-DIAGNOSIS-TEXT";

struct Ctx<'a> {
    env: Env,
    client: PetChainContractClient<'a>,
    admin: Address,
    owner: Address,
    vet: Address,
    pet_id: u64,
}

fn register_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Rex"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Mix"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    )
}

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env.ledger().with_mut(|l| l.timestamp = 1_700_000_000);
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    client.init_admin(&admin);
    client.set_retention_period(&admin, &RETENTION);
    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Purge"),
        &String::from_str(&env, "PURGE-1"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);
    let pet_id = register_pet(&env, &client, &owner);
    Ctx {
        env,
        client,
        admin,
        owner,
        vet,
        pet_id,
    }
}

fn add_record(c: &Ctx, pet_id: u64) -> u64 {
    c.client.add_medical_record(
        &pet_id,
        &c.vet,
        &String::from_str(&c.env, DIAGNOSIS),
        &String::from_str(&c.env, "Treatment"),
        &Vec::new(&c.env),
        &String::from_str(&c.env, "Notes"),
    )
}

/// Adds and soft-deletes records for `pet_id`, then moves past retention.
fn expired_records(c: &Ctx, pet_id: u64, n: u32) -> Vec<u64> {
    let owner = c.client.get_pet_owner(&pet_id).unwrap();
    let mut ids = Vec::new(&c.env);
    for _ in 0..n {
        let id = add_record(c, pet_id);
        c.client.delete_medical_record(&pet_id, &id, &owner);
        ids.push_back(id);
    }
    let now = c.env.ledger().timestamp();
    c.env.ledger().with_mut(|l| l.timestamp = now + RETENTION);
    ids
}

fn confirm(c: &Ctx, caller: &Address, pet_id: u64, ids: &Vec<u64>) -> (u64, BytesN<32>) {
    let nonce = c.client.get_purge_nonce(caller);
    let conf = c
        .client
        .compute_purge_confirmation(caller, &pet_id, ids, &nonce);
    (nonce, conf)
}

fn purge_err(c: &Ctx, caller: &Address, pet_id: u64, ids: &Vec<u64>) -> Error {
    let (nonce, conf) = confirm(c, caller, pet_id, ids);
    c.client
        .try_purge_records_confirmed(caller, &pet_id, ids, &nonce, &conf)
        .unwrap_err()
        .unwrap()
}

fn err(e: ContractError) -> Error {
    Error::from(e)
}

/// Raw storage presence (`get_medical_record` hides soft-deleted records).
fn present(c: &Ctx, id: u64) -> bool {
    c.client.try_get_medical_record_hash(&id).is_ok()
}

fn all_present(c: &Ctx, ids: &Vec<u64>) -> bool {
    ids.iter().all(|id| present(c, id))
}

fn contains(haystack: &Bytes, needle: &[u8]) -> bool {
    let n = needle.len() as u32;
    if n == 0 || haystack.len() < n {
        return false;
    }
    (0..=haystack.len() - n)
        .any(|i| (0..n).all(|j| haystack.get(i + j) == Some(needle[j as usize])))
}

// ---- roles ---------------------------------------------------------------

#[test]
fn owner_purge_removes_records_and_writes_audit() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 2);
    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    let entry = c
        .client
        .purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf);

    assert!(ids.iter().all(|id| !present(&c, id)));
    assert_eq!(entry.audit_id, 1);
    assert_eq!(entry.role, PurgeRole::Owner);
    assert_eq!(entry.purged_by, c.owner);
    assert_eq!(entry.record_count, 2);
    assert_eq!(entry.nonce, 0);
    assert_eq!(c.client.get_purge_nonce(&c.owner), 1);
    assert_eq!(c.client.get_purge_audit(&1), Some(entry));
    assert_eq!(c.client.get_purge_audit_count(), 1);
}

#[test]
fn admin_can_purge_any_pet_with_admin_role() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 1);
    let (nonce, conf) = confirm(&c, &c.admin, c.pet_id, &ids);
    let entry = c
        .client
        .purge_records_confirmed(&c.admin, &c.pet_id, &ids, &nonce, &conf);
    assert_eq!(entry.role, PurgeRole::Admin);
}

#[test]
fn unauthorized_purge_always_fails() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 1);
    // Strangers and even the authoring vet cannot purge.
    for caller in [Address::generate(&c.env), c.vet.clone()] {
        assert_eq!(
            purge_err(&c, &caller, c.pet_id, &ids),
            err(ContractError::Unauthorized)
        );
    }
    // Without the caller's signature the owner path fails too.
    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    c.env.set_auths(&[]);
    assert!(c
        .client
        .try_purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf)
        .is_err());
    assert!(all_present(&c, &ids));
    assert_eq!(c.client.get_purge_audit_count(), 0);
}

// ---- replay / nonce ------------------------------------------------------

#[test]
fn replayed_purge_is_rejected() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 1);
    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    c.client
        .purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf);
    assert_eq!(
        c.client
            .try_purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf)
            .unwrap_err()
            .unwrap(),
        err(ContractError::InvalidNonce)
    );
    assert_eq!(c.client.get_purge_audit_count(), 1);
}

#[test]
fn nonce_must_be_exact_and_is_per_account() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 2);
    let first = vec![&c.env, ids.get(0).unwrap()];
    let second = vec![&c.env, ids.get(1).unwrap()];

    // A skipped-ahead nonce is rejected even with a matching confirmation.
    let ahead = c
        .client
        .compute_purge_confirmation(&c.owner, &c.pet_id, &first, &1);
    assert_eq!(
        c.client
            .try_purge_records_confirmed(&c.owner, &c.pet_id, &first, &1, &ahead)
            .unwrap_err()
            .unwrap(),
        err(ContractError::InvalidNonce)
    );

    let (n, conf) = confirm(&c, &c.owner, c.pet_id, &first);
    c.client
        .purge_records_confirmed(&c.owner, &c.pet_id, &first, &n, &conf);
    // The owner's purge did not consume the admin's nonce.
    assert_eq!(c.client.get_purge_nonce(&c.admin), 0);
    let (n, conf) = confirm(&c, &c.admin, c.pet_id, &second);
    assert_eq!(n, 0);
    c.client
        .purge_records_confirmed(&c.admin, &c.pet_id, &second, &n, &conf);
}

// ---- scope ---------------------------------------------------------------

#[test]
fn confirmation_is_bound_to_account_pet_records_and_nonce() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 2);
    let subset = vec![&c.env, ids.get(0).unwrap()];
    let other_pet = register_pet(&c.env, &c.client, &c.owner);
    let (nonce, owner_conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    let mismatch = err(ContractError::PurgeConfirmationMismatch);

    // Different record set.
    assert_eq!(
        c.client
            .try_purge_records_confirmed(&c.owner, &c.pet_id, &subset, &nonce, &owner_conf)
            .unwrap_err()
            .unwrap(),
        mismatch
    );
    // Different pet.
    let other_conf = c
        .client
        .compute_purge_confirmation(&c.owner, &other_pet, &ids, &nonce);
    assert_eq!(
        c.client
            .try_purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &other_conf)
            .unwrap_err()
            .unwrap(),
        mismatch
    );
    // Different account presenting the owner's confirmation (same nonce 0).
    assert_eq!(
        c.client
            .try_purge_records_confirmed(&c.admin, &c.pet_id, &ids, &nonce, &owner_conf)
            .unwrap_err()
            .unwrap(),
        mismatch
    );
    assert!(all_present(&c, &ids));
}

#[test]
fn purge_cannot_target_another_pet_or_account() {
    let c = setup();
    let other_owner = Address::generate(&c.env);
    let other_pet = register_pet(&c.env, &c.client, &other_owner);
    let mine = add_record(&c, c.pet_id);
    let theirs = add_record(&c, other_pet);
    c.client.delete_medical_record(&c.pet_id, &mine, &c.owner);
    c.client
        .delete_medical_record(&other_pet, &theirs, &other_owner);
    c.env.ledger().with_mut(|l| l.timestamp += RETENTION);

    // Owner cannot purge a pet they do not own.
    let theirs_only = vec![&c.env, theirs];
    assert_eq!(
        purge_err(&c, &c.owner, other_pet, &theirs_only),
        err(ContractError::Unauthorized)
    );
    // Nor smuggle another pet's record into their own pet's scope; the
    // whole batch is rejected, including the in-scope record.
    let mixed = vec![&c.env, mine, theirs];
    assert_eq!(
        purge_err(&c, &c.owner, c.pet_id, &mixed),
        err(ContractError::PetScopeViolation)
    );
    assert!(all_present(&c, &mixed));
}

#[test]
fn only_expired_soft_deleted_records_are_purgeable() {
    let c = setup();
    let live = add_record(&c, c.pet_id);
    assert_eq!(
        purge_err(&c, &c.owner, c.pet_id, &vec![&c.env, live]),
        err(ContractError::InvalidState)
    );
    c.client.delete_medical_record(&c.pet_id, &live, &c.owner);
    c.env.ledger().with_mut(|l| l.timestamp += RETENTION - 1);
    assert_eq!(
        purge_err(&c, &c.owner, c.pet_id, &vec![&c.env, live]),
        err(ContractError::RetentionPeriodNotMet)
    );
    assert_eq!(
        purge_err(&c, &c.owner, c.pet_id, &vec![&c.env, 999u64]),
        err(ContractError::RecordNotFound)
    );
    // Exactly at the retention boundary it becomes purgeable.
    c.env.ledger().with_mut(|l| l.timestamp += 1);
    let ids = vec![&c.env, live];
    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    c.client
        .purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf);
}

#[test]
fn record_list_must_be_canonical_and_bounded() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 2);
    let (a, b) = (ids.get(0).unwrap(), ids.get(1).unwrap());
    let invalid = err(ContractError::InvalidInput);
    for list in [vec![&c.env], vec![&c.env, b, a], vec![&c.env, a, a]] {
        assert_eq!(
            c.client
                .try_compute_purge_confirmation(&c.owner, &c.pet_id, &list, &0)
                .unwrap_err()
                .unwrap(),
            invalid
        );
    }
    let mut big = Vec::new(&c.env);
    for id in 1..=(MAX_PURGE_BATCH as u64 + 1) {
        big.push_back(id);
    }
    assert_eq!(
        c.client
            .try_compute_purge_confirmation(&c.owner, &c.pet_id, &big, &0)
            .unwrap_err()
            .unwrap(),
        err(ContractError::BatchTooLarge)
    );
}

// ---- protected records ---------------------------------------------------

#[test]
fn held_records_cannot_be_purged_by_any_path() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 1);
    let id = ids.get(0).unwrap();
    c.client.set_purge_hold(&c.admin, &id, &true);
    assert!(c.client.is_record_on_purge_hold(&id));

    assert_eq!(
        purge_err(&c, &c.owner, c.pet_id, &ids),
        err(ContractError::RecordOnPurgeHold)
    );
    assert_eq!(
        purge_err(&c, &c.admin, c.pet_id, &ids),
        err(ContractError::RecordOnPurgeHold)
    );
    // Legacy purge paths skip held records too.
    let res = c.client.purge_deleted_records(&c.pet_id, &c.owner, &false);
    assert!(res.deleted.is_empty());
    let res = c
        .client
        .purge_deleted_records_bounded(&c.pet_id, &c.owner, &10, &0, &false);
    assert!(res.deleted.is_empty());
    assert!(all_present(&c, &ids));

    // Lifting the hold makes it purgeable again.
    c.client.set_purge_hold(&c.admin, &id, &false);
    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    c.client
        .purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf);
}

#[test]
fn only_admin_can_place_or_lift_holds() {
    let c = setup();
    let id = add_record(&c, c.pet_id);
    assert_eq!(
        c.client
            .try_set_purge_hold(&c.owner, &id, &true)
            .unwrap_err()
            .unwrap(),
        err(ContractError::Unauthorized)
    );
    c.client.set_purge_hold(&c.admin, &id, &true);
    assert_eq!(
        c.client
            .try_set_purge_hold(&c.owner, &id, &false)
            .unwrap_err()
            .unwrap(),
        err(ContractError::Unauthorized)
    );
    assert!(c.client.is_record_on_purge_hold(&id));
    assert_eq!(
        c.client
            .try_set_purge_hold(&c.admin, &999, &true)
            .unwrap_err()
            .unwrap(),
        err(ContractError::RecordNotFound)
    );
}

// ---- audit privacy -------------------------------------------------------

#[test]
fn audit_event_contains_digest_and_count_not_raw_data() {
    let c = setup();
    let ids = expired_records(&c, c.pet_id, 2);

    // Independently recompute the expected records digest from the
    // canonical record hashes before the records disappear.
    let mut pre = Bytes::from_slice(&c.env, PURGE_RECORDS_DIGEST_DOMAIN);
    pre.extend_from_array(&c.pet_id.to_be_bytes());
    pre.extend_from_array(&ids.len().to_be_bytes());
    for id in ids.iter() {
        pre.extend_from_array(&id.to_be_bytes());
        pre.append(&Bytes::from(c.client.get_medical_record_hash(&id)));
    }
    let expected_digest: BytesN<32> = c.env.crypto().sha256(&pre).into();

    let (nonce, conf) = confirm(&c, &c.owner, c.pet_id, &ids);
    let entry = c
        .client
        .purge_records_confirmed(&c.owner, &c.pet_id, &ids, &nonce, &conf);
    assert_eq!(entry.records_digest, expected_digest);

    let events = c.env.events().all();
    let topic = Symbol::new(&c.env, "RecordsPurged");
    let (_, _, data) = events
        .iter()
        .find(|(_, topics, _)| {
            topics
                .get(0)
                .and_then(|t| Symbol::try_from_val(&c.env, &t).ok())
                .map(|t| t == topic)
                .unwrap_or(false)
        })
        .expect("RecordsPurged event");
    let event = PurgeAuditEvent::try_from_val(&c.env, &data).unwrap();
    assert_eq!(event.record_count, 2);
    assert_eq!(event.records_digest, expected_digest);
    assert_eq!(event.audit_id, entry.audit_id);
    assert_eq!(event.role, PurgeRole::Owner);

    // No record content reaches the event or the stored audit entry.
    let event_xdr = data.to_xdr(&c.env);
    let entry_xdr = entry.to_xdr(&c.env);
    for field in [DIAGNOSIS, "Treatment", "Notes"] {
        assert!(!contains(&event_xdr, field.as_bytes()));
        assert!(!contains(&entry_xdr, field.as_bytes()));
    }
}
