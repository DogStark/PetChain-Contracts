// ============================================================
// PERSISTENT STORAGE TTL EXTENSION POLICY TESTS (Issue #1154)
//
// Persistent-storage entries (breeding records, access logs, ...) are not
// kept alive automatically by contract invocations the way instance storage
// is: their TTL must be extended explicitly on write, or the ledger can
// archive/expire them out from under the contract. `PetChainContract`
// bumps the TTL of every persistent entry it writes via
// `bump_persistent_ttl`, using the `PERSISTENT_TTL_THRESHOLD` /
// `PERSISTENT_TTL_EXTEND_TO` policy constants. These tests assert that a
// freshly written persistent entry's live-until horizon is actually
// extended out to (at least) the policy's `extend_to` value.
// ============================================================

use crate::{BreedingKey, PetChainContract, PetChainContractClient};
use soroban_sdk::{testutils::storage::Persistent as _, Env, String};

fn make_client(env: &Env) -> PetChainContractClient<'static> {
    let contract_id = env.register_contract(None, PetChainContract);
    PetChainContractClient::new(env, &contract_id)
}

/// Writing a breeding record extends its persistent TTL to at least
/// `PERSISTENT_TTL_EXTEND_TO` ledgers, not just whatever the network's
/// default minimum persistent TTL happens to be.
#[test]
fn test_add_breeding_record_extends_persistent_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    let record_id = client.add_breeding_record(
        &1u64,
        &2u64,
        &1_000_000u64,
        &String::from_str(&env, "TTL policy check"),
    );

    let ttl = env
        .as_contract(&client.address, || {
            env.storage()
                .persistent()
                .get_ttl(&BreedingKey::BreedingRecord(record_id))
        });

    assert!(
        ttl as u64 >= crate::PERSISTENT_TTL_EXTEND_TO as u64,
        "expected breeding record TTL >= {} ledgers, got {}",
        crate::PERSISTENT_TTL_EXTEND_TO,
        ttl
    );
}

/// The `BreedingRecordCount` sentinel is written on every call and must
/// also have its TTL extended, so the counter itself cannot expire while
/// individual records remain live.
#[test]
fn test_breeding_record_count_extends_persistent_ttl() {
    let env = Env::default();
    env.mock_all_auths();
    let client = make_client(&env);

    client.add_breeding_record(
        &1u64,
        &2u64,
        &1_000_000u64,
        &String::from_str(&env, "TTL policy check"),
    );

    let ttl = env.as_contract(&client.address, || {
        env.storage()
            .persistent()
            .get_ttl(&BreedingKey::BreedingRecordCount)
    });

    assert!(
        ttl as u64 >= crate::PERSISTENT_TTL_EXTEND_TO as u64,
        "expected BreedingRecordCount TTL >= {} ledgers, got {}",
        crate::PERSISTENT_TTL_EXTEND_TO,
        ttl
    );
}
