//! Storage-rent and cleanup observability (Issue #1258).

use crate::{
    ContractError, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
    StorageMetrics, MAX_STORAGE_METRICS_SCAN,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Error, String, Vec,
};

const RETENTION: u64 = 100;

struct Ctx<'a> {
    env: Env,
    client: PetChainContractClient<'a>,
    admin: Address,
    owner: Address,
    vet: Address,
    pet_id: u64,
}

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let client = PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
    let (admin, owner, vet) = (
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    );

    client.init_admin(&admin);
    client.set_retention_period(&admin, &RETENTION);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Metrics"),
        &String::from_str(&env, "LIC-METRICS"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Rex"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Brown"),
        &25,
        &None,
        &PrivacyLevel::Public,
    );

    Ctx {
        env,
        client,
        admin,
        owner,
        vet,
        pet_id,
    }
}

impl Ctx<'_> {
    fn add_record(&self) -> u64 {
        self.client.add_medical_record(
            &self.pet_id,
            &self.vet,
            &String::from_str(&self.env, "Private diagnosis"),
            &String::from_str(&self.env, "Private treatment"),
            &Vec::new(&self.env),
            &String::from_str(&self.env, "Private notes"),
        )
    }

    /// Adds `total` records and soft-deletes the first `deleted` of them.
    fn add_records(&self, total: u32, deleted: u32) {
        for i in 0..total {
            let id = self.add_record();
            if i < deleted {
                self.client
                    .delete_medical_record(&self.pet_id, &id, &self.owner);
            }
        }
    }

    fn metrics(&self, cursor: u64, limit: u32) -> StorageMetrics {
        self.client
            .get_storage_metrics(&self.pet_id, &cursor, &limit)
    }

    /// Sums the backlog across every page of a bounded scan.
    fn total_backlog(&self, limit: u32) -> (u64, u64) {
        let (mut cursor, mut backlog, mut pending) = (0, 0, 0);
        loop {
            let page = self.metrics(cursor, limit);
            backlog += page.cleanup_backlog;
            pending += page.pending_retention;
            if page.next_cursor == 0 {
                return (backlog, pending);
            }
            cursor = page.next_cursor;
        }
    }

    fn pass_retention(&self) {
        self.env
            .ledger()
            .set_timestamp(self.env.ledger().timestamp() + RETENTION + 1);
    }
}

#[test]
fn metrics_report_usage_against_cap_thresholds() {
    let c = setup();
    c.client.set_pet_storage_quota(&c.admin, &c.pet_id, &3);

    c.client.add_weight_entry(&c.pet_id, &20);
    c.client.add_weight_entry(&c.pet_id, &21);
    let below = c.metrics(0, 10);
    assert_eq!((below.used, below.quota, below.remaining), (2, 3, 1));

    c.client.add_weight_entry(&c.pet_id, &22);
    let at_cap = c.metrics(0, 10);
    assert_eq!((at_cap.used, at_cap.remaining), (3, 0));

    assert_eq!(
        c.client.try_add_weight_entry(&c.pet_id, &23),
        Err(Ok(Error::from_contract_error(
            ContractError::StorageQuotaExceeded as u32
        )))
    );
    assert_eq!(c.metrics(0, 10), at_cap);

    // Lowering the cap below usage saturates instead of underflowing.
    c.client.set_pet_storage_quota(&c.admin, &c.pet_id, &1);
    assert_eq!(c.metrics(0, 10).remaining, 0);
}

#[test]
fn metrics_and_cleanup_are_empty_without_deleted_records() {
    let c = setup();
    let empty = c.metrics(0, 10);
    assert_eq!(
        (
            empty.medical_record_slots,
            empty.cleanup_backlog,
            empty.next_cursor
        ),
        (0, 0, 0)
    );

    c.add_records(3, 0);
    let live = c.metrics(0, 10);
    assert_eq!((live.medical_record_slots, live.used), (3, 3));
    assert_eq!((live.cleanup_backlog, live.pending_retention), (0, 0));

    let purge = c
        .client
        .purge_deleted_records_bounded(&c.pet_id, &c.owner, &10, &0, &false);
    assert!(purge.deleted.is_empty());
    assert_eq!(purge.next_cursor, 0);
}

#[test]
fn metrics_separate_purgeable_backlog_from_retained_deletes() {
    let c = setup();
    c.add_records(4, 3);
    assert_eq!(c.total_backlog(10), (0, 3));

    c.pass_retention();
    assert_eq!(c.total_backlog(10), (3, 0));
}

#[test]
fn partial_cleanup_is_bounded_and_resumable() {
    let c = setup();
    c.add_records(6, 5);
    c.pass_retention();

    // Bounded metrics pages add up to the full backlog.
    let first = c.metrics(0, 2);
    assert_eq!((first.cleanup_backlog, first.next_cursor), (2, 2));
    assert_eq!(c.total_backlog(2), (5, 0));

    // Purge one bounded batch, then observe the remaining backlog.
    let batch = c
        .client
        .purge_deleted_records_bounded(&c.pet_id, &c.owner, &2, &0, &false);
    assert_eq!(batch.deleted.len(), 2);
    assert_eq!(batch.next_cursor, 2);
    assert_eq!(c.total_backlog(10), (3, 0));

    // Resume from the returned cursor until done.
    let rest = c.client.purge_deleted_records_bounded(
        &c.pet_id,
        &c.admin,
        &10,
        &batch.next_cursor,
        &false,
    );
    assert_eq!(rest.deleted.len(), 3);
    assert_eq!(rest.next_cursor, 0);
    assert_eq!(c.total_backlog(10), (0, 0));
    assert_eq!(c.metrics(0, 10).medical_record_slots, 6);
}

#[test]
fn metrics_scan_is_capped_per_call() {
    let c = setup();
    c.env.budget().reset_unlimited();
    c.add_records(MAX_STORAGE_METRICS_SCAN + 1, 0);

    // A full-size page must fit in the default per-invocation budget.
    c.env.budget().reset_default();
    let page = c.metrics(0, u32::MAX);
    assert_eq!(page.next_cursor, MAX_STORAGE_METRICS_SCAN as u64);
    assert_eq!(c.metrics(page.next_cursor, u32::MAX).next_cursor, 0);

    // A zero limit scans nothing and hands back the same cursor.
    assert_eq!(c.metrics(1, 0).next_cursor, 1);
}

#[test]
fn failed_continuation_leaves_backlog_intact() {
    let c = setup();
    c.add_records(4, 4);
    c.pass_retention();
    let batch = c
        .client
        .purge_deleted_records_bounded(&c.pet_id, &c.owner, &1, &0, &false);
    assert_eq!(batch.next_cursor, 1);

    // An unauthorized caller cannot continue the cleanup.
    let stranger = Address::generate(&c.env);
    assert_eq!(
        c.client.try_purge_deleted_records_bounded(
            &c.pet_id,
            &stranger,
            &10,
            &batch.next_cursor,
            &false
        ),
        Err(Ok(Error::from_contract_error(
            ContractError::Unauthorized as u32
        )))
    );
    assert_eq!(c.total_backlog(10), (3, 0));

    // A stale cursor past the end does no work and reports completion.
    let stale = c
        .client
        .purge_deleted_records_bounded(&c.pet_id, &c.owner, &10, &99, &false);
    assert!(stale.deleted.is_empty());
    assert_eq!(stale.next_cursor, 0);
    assert_eq!(c.metrics(99, 10).next_cursor, 0);
    assert_eq!(c.total_backlog(10), (3, 0));

    // Metrics for an unknown pet fail cleanly.
    assert_eq!(
        c.client.try_get_storage_metrics(&999, &0, &10),
        Err(Ok(Error::from_contract_error(
            ContractError::PetNotFound as u32
        )))
    );
}
