// Health-score input provenance: score vectors, digest vectors, determinism,
// version isolation and authorization (Issue #1343).
use crate::{
    ContractError, Gender, HealthInputKind, HealthInputRef, HealthScoreInputs, PetChainContract,
    PetChainContractClient, PrivacyLevel, Species, VaccineType, HEALTH_SCORE_ALGORITHM_V1,
    HEALTH_SCORE_ALGORITHM_V2, MAX_HEALTH_SCORE_SOURCES,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Error, Map, String, Vec,
};

const NOW: u64 = 1_700_000_000;
const V1: u32 = HEALTH_SCORE_ALGORITHM_V1;
const V2: u32 = HEALTH_SCORE_ALGORITHM_V2;

struct Ctx<'a> {
    env: Env,
    client: PetChainContractClient<'a>,
    admin: Address,
    owner: Address,
    vet: Address,
    pet_id: u64,
    record_id: u64,
    vaccination_id: u64,
    lab_id: u64,
}

fn register_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Milo"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Mix"),
        &String::from_str(env, "Brown"),
        &24u32,
        &None,
        &PrivacyLevel::Public,
    )
}

fn register_verified_vet(
    env: &Env,
    client: &PetChainContractClient,
    admin: &Address,
    license: &str,
) -> Address {
    let vet = Address::generate(env);
    client.register_vet(
        &vet,
        &String::from_str(env, "Dr. Score"),
        &String::from_str(env, license),
        &String::from_str(env, "General"),
    );
    client.verify_vet(admin, &vet);
    vet
}

fn add_medical_record(
    c_env: &Env,
    client: &PetChainContractClient,
    pet_id: u64,
    vet: &Address,
) -> u64 {
    client.add_medical_record(
        &pet_id,
        vet,
        &String::from_str(c_env, "Checkup"),
        &String::from_str(c_env, "None"),
        &Vec::new(c_env),
        &String::from_str(c_env, "Healthy"),
    )
}

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    env.ledger().with_mut(|l| l.timestamp = NOW);
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    client.init_admin(&admin);
    let vet = register_verified_vet(&env, &client, &admin, "HEALTH-1");
    let pet_id = register_pet(&env, &client, &owner);

    let record_id = add_medical_record(&env, &client, pet_id, &vet);
    let vaccination_id = client.add_vaccination(
        &pet_id,
        &vet,
        &VaccineType::Rabies,
        &String::from_str(&env, "Rabies"),
        &NOW,
        &(NOW + 90 * 86_400),
        &(NOW + 90 * 86_400),
        &String::from_str(&env, "BATCH-1"),
    );
    let lab_id = client.add_lab_result(
        &pet_id,
        &vet,
        &String::from_str(&env, "CBC"),
        &String::from_str(&env, "Normal"),
        &String::from_str(&env, "{}"),
        &None,
        &None,
        &Map::new(&env),
    );

    Ctx {
        env,
        client,
        admin,
        owner,
        vet,
        pet_id,
        record_id,
        vaccination_id,
        lab_id,
    }
}

fn inputs(
    env: &Env,
    as_of: u64,
    comps: [u32; 4],
    sources: &[(HealthInputKind, u64)],
) -> HealthScoreInputs {
    let mut v = Vec::new(env);
    for (kind, id) in sources {
        v.push_back(HealthInputRef {
            kind: *kind,
            id: *id,
        });
    }
    HealthScoreInputs {
        as_of,
        vaccination: comps[0],
        lab_results: comps[1],
        activity: comps[2],
        insurance: comps[3],
        sources: v,
    }
}

fn full_inputs(c: &Ctx) -> HealthScoreInputs {
    inputs(
        &c.env,
        NOW,
        [100, 100, 10, 100],
        &[
            (HealthInputKind::MedicalRecord, c.record_id),
            (HealthInputKind::Vaccination, c.vaccination_id),
            (HealthInputKind::LabResult, c.lab_id),
        ],
    )
}

fn hex32(env: &Env, hex: &str) -> BytesN<32> {
    let b = hex.as_bytes();
    let mut out = [0u8; 32];
    let nib = |c: u8| match c {
        b'0'..=b'9' => c - b'0',
        _ => c - b'a' + 10,
    };
    for i in 0..32 {
        out[i] = (nib(b[2 * i]) << 4) | nib(b[2 * i + 1]);
    }
    BytesN::from_array(env, &out)
}

fn err(e: ContractError) -> Error {
    Error::from(e)
}

// ---- score vectors -------------------------------------------------------

#[test]
fn score_vectors_per_algorithm_version() {
    let c = setup();
    // (components, v1, v2)
    let vectors: [([u32; 4], u32, u32); 5] = [
        ([0, 0, 0, 0], 0, 0),
        ([100, 100, 100, 100], 100, 100),
        ([100, 100, 10, 100], 77, 86),
        ([99, 1, 50, 0], 37, 42),
        ([0, 0, 100, 0], 25, 15),
    ];
    for (comps, v1, v2) in vectors {
        let i = inputs(&c.env, NOW, comps, &[]);
        assert_eq!(c.client.compute_health_score_value(&V1, &i), v1);
        assert_eq!(c.client.compute_health_score_value(&V2, &i), v2);
    }
}

#[test]
fn input_digest_matches_published_vectors() {
    let c = setup();
    let i = inputs(
        &c.env,
        1_000,
        [100, 100, 10, 100],
        &[
            (HealthInputKind::MedicalRecord, 1),
            (HealthInputKind::Vaccination, 1),
            (HealthInputKind::LabResult, 1),
        ],
    );
    assert_eq!(
        c.client.compute_health_input_digest(&1, &i),
        hex32(
            &c.env,
            "1194ed9c56db1d681284bd126e1899a63ce1883e1cd2ff98ca4f297839d18244"
        )
    );
    let empty = inputs(&c.env, 0, [0, 0, 0, 0], &[]);
    assert_eq!(
        c.client.compute_health_input_digest(&1, &empty),
        hex32(
            &c.env,
            "64ce5abd8edda021c1bc891b3e8ebe03ca2b7669fe31e64d882d2e3b152362fd"
        )
    );
}

#[test]
fn digest_binds_pet_snapshot_time_components_and_sources() {
    let c = setup();
    let base = full_inputs(&c);
    let d = c.client.compute_health_input_digest(&c.pet_id, &base);

    assert_ne!(
        c.client.compute_health_input_digest(&(c.pet_id + 1), &base),
        d
    );
    let mut t = base.clone();
    t.as_of -= 1;
    assert_ne!(c.client.compute_health_input_digest(&c.pet_id, &t), d);
    let mut comp = base.clone();
    comp.activity += 1;
    assert_ne!(c.client.compute_health_input_digest(&c.pet_id, &comp), d);
    let mut fewer = base.clone();
    fewer.sources.pop_back();
    assert_ne!(c.client.compute_health_input_digest(&c.pet_id, &fewer), d);
}

// ---- provenance binding and determinism ----------------------------------

#[test]
fn recorded_score_is_bound_to_digest_version_scorer_and_time() {
    let c = setup();
    let i = full_inputs(&c);
    let rec = c.client.record_health_score(&c.vet, &c.pet_id, &V1, &i);
    assert_eq!(rec.seq, 1);
    assert_eq!(rec.score, 77);
    assert_eq!(rec.algorithm_version, V1);
    assert_eq!(
        rec.input_digest,
        c.client.compute_health_input_digest(&c.pet_id, &i)
    );
    assert_eq!(rec.inputs, i);
    assert_eq!(rec.scorer, c.vet);
    assert_eq!(rec.scored_at, NOW);
    assert_eq!(
        c.client.get_health_score_record(&c.pet_id, &1),
        Some(rec.clone())
    );
    assert_eq!(c.client.get_latest_health_score(&c.pet_id, &V1), Some(rec));
    assert!(c.client.verify_health_score_provenance(&c.pet_id, &1, &i));
}

#[test]
fn recalculation_with_same_inputs_is_deterministic() {
    let c = setup();
    let i = full_inputs(&c);
    let first = c.client.record_health_score(&c.vet, &c.pet_id, &V1, &i);

    // Later, by another authorized scorer: identical result, no new entry.
    c.env.ledger().with_mut(|l| l.timestamp = NOW + 500);
    let again = c.client.record_health_score(&c.admin, &c.pet_id, &V1, &i);
    assert_eq!(again, first);
    assert_eq!(c.client.get_health_score_record_count(&c.pet_id), 1);
}

#[test]
fn different_algorithm_versions_cannot_overwrite_history() {
    let c = setup();
    let i = full_inputs(&c);
    let v1 = c.client.record_health_score(&c.vet, &c.pet_id, &V1, &i);
    let v2 = c.client.record_health_score(&c.vet, &c.pet_id, &V2, &i);

    assert_eq!((v1.seq, v2.seq), (1, 2));
    assert_eq!(v1.input_digest, v2.input_digest);
    assert_eq!((v1.score, v2.score), (77, 86));
    assert_eq!(
        c.client.get_health_score_record(&c.pet_id, &1),
        Some(v1.clone())
    );
    assert_eq!(
        c.client.get_latest_health_score(&c.pet_id, &V1),
        Some(v1.clone())
    );
    assert_eq!(
        c.client.get_latest_health_score(&c.pet_id, &V2),
        Some(v2.clone())
    );

    // Replaying v1 afterwards does not displace v2 or rewrite v1.
    assert_eq!(c.client.record_health_score(&c.vet, &c.pet_id, &V1, &i), v1);
    assert_eq!(c.client.get_health_score_record_count(&c.pet_id), 2);
    assert_eq!(c.client.get_latest_health_score(&c.pet_id, &V2), Some(v2));
}

#[test]
fn new_inputs_append_without_mutating_prior_records() {
    let c = setup();
    let first = c
        .client
        .record_health_score(&c.vet, &c.pet_id, &V1, &full_inputs(&c));
    let mut changed = full_inputs(&c);
    changed.insurance = 0;
    let second = c
        .client
        .record_health_score(&c.vet, &c.pet_id, &V1, &changed);
    assert_eq!(second.seq, 2);
    assert_eq!(second.score, 52);
    assert_eq!(c.client.get_health_score_record(&c.pet_id, &1), Some(first));
    assert_eq!(
        c.client.get_latest_health_score(&c.pet_id, &V1),
        Some(second)
    );
}

#[test]
fn provenance_check_rejects_altered_inputs() {
    let c = setup();
    let i = full_inputs(&c);
    c.client.record_health_score(&c.vet, &c.pet_id, &V1, &i);
    let mut altered = i.clone();
    altered.lab_results = 99;
    assert!(!c
        .client
        .verify_health_score_provenance(&c.pet_id, &1, &altered));
    assert!(!c.client.verify_health_score_provenance(&c.pet_id, &2, &i));
}

// ---- authorization -------------------------------------------------------

#[test]
fn unauthorized_scorers_are_rejected() {
    let c = setup();
    let i = full_inputs(&c);
    let unverified = Address::generate(&c.env);
    c.client.register_vet(
        &unverified,
        &String::from_str(&c.env, "Dr. New"),
        &String::from_str(&c.env, "HEALTH-2"),
        &String::from_str(&c.env, "General"),
    );
    let revoked = register_verified_vet(&c.env, &c.client, &c.admin, "HEALTH-3");
    c.client.revoke_vet_license(&c.admin, &revoked);

    for scorer in [
        c.owner.clone(),
        Address::generate(&c.env),
        unverified,
        revoked,
    ] {
        assert_eq!(
            c.client
                .try_record_health_score(&scorer, &c.pet_id, &V1, &i)
                .unwrap_err()
                .unwrap(),
            err(ContractError::Unauthorized)
        );
    }
    assert_eq!(c.client.get_health_score_record_count(&c.pet_id), 0);
}

#[test]
fn score_requires_scorer_signature() {
    let c = setup();
    let i = full_inputs(&c);
    c.env.set_auths(&[]);
    assert!(c
        .client
        .try_record_health_score(&c.vet, &c.pet_id, &V1, &i)
        .is_err());
}

// ---- input validation ----------------------------------------------------

#[test]
fn unsupported_algorithm_versions_are_rejected() {
    let c = setup();
    let i = full_inputs(&c);
    for v in [0u32, 3u32] {
        assert_eq!(
            c.client
                .try_record_health_score(&c.vet, &c.pet_id, &v, &i)
                .unwrap_err()
                .unwrap(),
            err(ContractError::UnsupportedAlgorithmVersion)
        );
    }
}

#[test]
fn non_canonical_or_out_of_range_inputs_are_rejected() {
    let c = setup();
    let over = inputs(&c.env, NOW, [101, 0, 0, 0], &[]);
    let unsorted = inputs(
        &c.env,
        NOW,
        [1, 1, 1, 1],
        &[
            (HealthInputKind::Vaccination, c.vaccination_id),
            (HealthInputKind::MedicalRecord, c.record_id),
        ],
    );
    let duplicate = inputs(
        &c.env,
        NOW,
        [1, 1, 1, 1],
        &[
            (HealthInputKind::MedicalRecord, c.record_id),
            (HealthInputKind::MedicalRecord, c.record_id),
        ],
    );
    for i in [over, unsorted, duplicate] {
        assert_eq!(
            c.client
                .try_record_health_score(&c.vet, &c.pet_id, &V1, &i)
                .unwrap_err()
                .unwrap(),
            err(ContractError::InvalidInput)
        );
    }

    let mut many = inputs(&c.env, NOW, [1, 1, 1, 1], &[]);
    for id in 0..=(MAX_HEALTH_SCORE_SOURCES as u64) {
        many.sources.push_back(HealthInputRef {
            kind: HealthInputKind::MedicalRecord,
            id,
        });
    }
    assert_eq!(
        c.client
            .try_record_health_score(&c.vet, &c.pet_id, &V1, &many)
            .unwrap_err()
            .unwrap(),
        err(ContractError::TooManyItems)
    );

    let future = inputs(&c.env, NOW + 1, [1, 1, 1, 1], &[]);
    assert_eq!(
        c.client
            .try_record_health_score(&c.vet, &c.pet_id, &V1, &future)
            .unwrap_err()
            .unwrap(),
        err(ContractError::InvalidTimestamp)
    );
}

#[test]
fn sources_must_exist_be_live_and_belong_to_the_pet() {
    let c = setup();
    let other_pet = register_pet(&c.env, &c.client, &c.owner);
    let foreign = add_medical_record(&c.env, &c.client, other_pet, &c.vet);
    let one = |kind, id| inputs(&c.env, NOW, [1, 1, 1, 1], &[(kind, id)]);

    assert_eq!(
        c.client
            .try_record_health_score(
                &c.vet,
                &c.pet_id,
                &V1,
                &one(HealthInputKind::MedicalRecord, foreign)
            )
            .unwrap_err()
            .unwrap(),
        err(ContractError::PetScopeViolation)
    );
    assert_eq!(
        c.client
            .try_record_health_score(
                &c.vet,
                &c.pet_id,
                &V1,
                &one(HealthInputKind::LabResult, 999)
            )
            .unwrap_err()
            .unwrap(),
        err(ContractError::RecordNotFound)
    );

    c.client
        .delete_medical_record(&c.pet_id, &c.record_id, &c.owner);
    assert_eq!(
        c.client
            .try_record_health_score(
                &c.vet,
                &c.pet_id,
                &V1,
                &one(HealthInputKind::MedicalRecord, c.record_id)
            )
            .unwrap_err()
            .unwrap(),
        err(ContractError::RecordAlreadyDeleted)
    );
    // Sanity: the same call with in-scope sources succeeds.
    let ok = inputs(
        &c.env,
        NOW,
        [1, 1, 1, 1],
        &[(HealthInputKind::Vaccination, c.vaccination_id)],
    );
    c.client.record_health_score(&c.vet, &c.pet_id, &V1, &ok);
}
