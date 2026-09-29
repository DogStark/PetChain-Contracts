// Breeding eligibility, cooldown boundary and replay tests (Issue #1342).
use crate::{
    BreedingCheck, BreedingPolicy, ContractError, Gender, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species, DEFAULT_DAM_COOLDOWN_SECS, DEFAULT_MAX_COI_BP,
    DEFAULT_MIN_BREEDING_AGE_SECS, DEFAULT_SIRE_COOLDOWN_SECS,
};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger as _, MockAuth, MockAuthInvoke},
    Address, Env, Error, IntoVal, String,
};

const NOW: u64 = 2_000_000_000;
const BIRTH: u64 = 1_000_000_000;
const MIN_AGE: u64 = 1_000;
const SIRE_COOLDOWN: u64 = 100;
const DAM_COOLDOWN: u64 = 500;
/// Earliest date at which a pet born at BIRTH is old enough.
const ADULT: u64 = BIRTH + MIN_AGE;

struct Ctx<'a> {
    env: Env,
    client: PetChainContractClient<'a>,
    contract_id: Address,
    admin: Address,
    owner: Address,
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
    client.set_breeding_policy(
        &admin,
        &BreedingPolicy {
            min_age_secs: MIN_AGE,
            sire_cooldown_secs: SIRE_COOLDOWN,
            dam_cooldown_secs: DAM_COOLDOWN,
            max_coi_bp: DEFAULT_MAX_COI_BP,
        },
    );
    Ctx {
        env,
        client,
        contract_id,
        admin,
        owner,
    }
}

fn ts_string(env: &Env, mut v: u64) -> String {
    let mut buf = [0u8; 20];
    let mut i = buf.len();
    loop {
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    String::from_bytes(env, &buf[i..])
}

fn pet(c: &Ctx, owner: &Address, gender: Gender, species: Species, birth: u64) -> u64 {
    let id = c.client.register_pet(
        owner,
        &String::from_str(&c.env, "Pet"),
        &ts_string(&c.env, birth),
        &gender,
        &species,
        &String::from_str(&c.env, "Mix"),
        &String::from_str(&c.env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Private,
    );
    c.client.activate_pet(&id);
    id
}

fn sire(c: &Ctx) -> u64 {
    pet(c, &c.owner, Gender::Male, Species::Dog, BIRTH)
}

fn dam(c: &Ctx) -> u64 {
    pet(c, &c.owner, Gender::Female, Species::Dog, BIRTH)
}

fn notes(env: &Env) -> String {
    String::from_str(env, "")
}

fn err(e: ContractError) -> Error {
    Error::from(e)
}

fn mate_err(c: &Ctx, s: u64, d: u64, date: u64) -> Error {
    c.client
        .try_record_mating(&c.owner, &s, &d, &date, &notes(&c.env))
        .unwrap_err()
        .unwrap()
}

// ---- happy path ----------------------------------------------------------

#[test]
fn eligible_mating_records_parent_and_cooldown_relationship() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d, &ADULT),
        BreedingCheck::Eligible
    );
    let id = c
        .client
        .record_mating(&c.owner, &s, &d, &ADULT, &notes(&c.env));

    let rec = c.client.get_breeding_record(&id).unwrap();
    assert_eq!((rec.sire_id, rec.dam_id, rec.breeding_date), (s, d, ADULT));
    assert_eq!(rec.breeder, c.owner);
    assert_eq!(c.client.get_last_mating_date(&s), Some(ADULT));
    assert_eq!(c.client.get_last_mating_date(&d), Some(ADULT));
    assert_eq!(c.client.get_breeding_count(&s), 1);
    assert_eq!(c.client.get_breeding_count(&d), 1);
}

#[test]
fn default_policy_applies_when_unset() {
    let env = Env::default();
    let client = PetChainContractClient::new(&env, &env.register_contract(None, PetChainContract));
    let p = client.get_breeding_policy();
    assert_eq!(p.min_age_secs, DEFAULT_MIN_BREEDING_AGE_SECS);
    assert_eq!(p.sire_cooldown_secs, DEFAULT_SIRE_COOLDOWN_SECS);
    assert_eq!(p.dam_cooldown_secs, DEFAULT_DAM_COOLDOWN_SECS);
    assert_eq!(p.max_coi_bp, DEFAULT_MAX_COI_BP);
}

// ---- ineligible parents --------------------------------------------------

#[test]
fn self_breeding_and_missing_parents_are_rejected() {
    let c = setup();
    let s = sire(&c);
    assert_eq!(mate_err(&c, s, s, ADULT), err(ContractError::SelfBreeding));
    assert_eq!(mate_err(&c, s, 999, ADULT), err(ContractError::PetNotFound));
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &999, &ADULT),
        BreedingCheck::ParentNotFound
    );
}

#[test]
fn inactive_or_archived_parents_are_rejected() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    c.client.deactivate_pet(&d);
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d, &ADULT),
        BreedingCheck::ParentInactive
    );
    assert_eq!(
        mate_err(&c, s, d, ADULT),
        err(ContractError::ParentInactive)
    );

    c.client.activate_pet(&d);
    c.client.archive_pet(&s);
    assert_eq!(
        mate_err(&c, s, d, ADULT),
        err(ContractError::ParentInactive)
    );
}

#[test]
fn incompatible_sex_or_species_is_rejected() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    let male2 = pet(&c, &c.owner, Gender::Male, Species::Dog, BIRTH);
    let cat = pet(&c, &c.owner, Gender::Female, Species::Cat, BIRTH);
    let unknown = pet(&c, &c.owner, Gender::Unknown, Species::Dog, BIRTH);

    assert_eq!(
        c.client.check_breeding_eligibility(&s, &cat, &ADULT),
        BreedingCheck::IncompatibleSpecies
    );
    for (a, b) in [(s, male2), (d, s), (s, unknown)] {
        assert_eq!(
            c.client.check_breeding_eligibility(&a, &b, &ADULT),
            BreedingCheck::IncompatibleSex
        );
        assert_eq!(
            mate_err(&c, a, b, ADULT),
            err(ContractError::IncompatibleParents)
        );
    }
    assert_eq!(
        mate_err(&c, s, cat, ADULT),
        err(ContractError::IncompatibleParents)
    );
}

#[test]
fn unauthorized_caller_is_rejected() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    let stranger = Address::generate(&c.env);
    assert_eq!(
        c.client
            .try_record_mating(&stranger, &s, &d, &ADULT, &notes(&c.env))
            .unwrap_err()
            .unwrap(),
        err(ContractError::NotPetOwner)
    );
    c.env.set_auths(&[]);
    assert!(c
        .client
        .try_record_mating(&c.owner, &s, &d, &ADULT, &notes(&c.env))
        .is_err());
    assert_eq!(c.client.get_last_mating_date(&s), None);
}

#[test]
fn other_parents_owner_must_co_sign() {
    let c = setup();
    let stud_owner = Address::generate(&c.env);
    let s = pet(&c, &stud_owner, Gender::Male, Species::Dog, BIRTH);
    let d = dam(&c);
    let n = notes(&c.env);
    let invoke = MockAuthInvoke {
        contract: &c.contract_id,
        fn_name: "record_mating",
        args: (c.owner.clone(), s, d, ADULT, n.clone()).into_val(&c.env),
        sub_invokes: &[],
    };
    let dam_sig = MockAuth {
        address: &c.owner,
        invoke: &invoke,
    };
    let stud_sig = MockAuth {
        address: &stud_owner,
        invoke: &invoke,
    };

    // Dam owner alone is not enough.
    c.env.mock_auths(core::slice::from_ref(&dam_sig));
    assert!(c
        .client
        .try_record_mating(&c.owner, &s, &d, &ADULT, &n)
        .is_err());

    // Both owners sign: accepted.
    c.env.mock_auths(&[dam_sig, stud_sig]);
    c.client.record_mating(&c.owner, &s, &d, &ADULT, &n);
    assert_eq!(c.client.get_last_mating_date(&s), Some(ADULT));
}

// ---- age and date boundaries ---------------------------------------------

#[test]
fn minimum_age_is_enforced_at_exact_boundary() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d, &(ADULT - 1)),
        BreedingCheck::ParentTooYoung
    );
    assert_eq!(
        mate_err(&c, s, d, ADULT - 1),
        err(ContractError::ParentTooYoung)
    );
    c.client
        .record_mating(&c.owner, &s, &d, &ADULT, &notes(&c.env));
}

#[test]
fn younger_parent_governs_age_check() {
    let c = setup();
    let s = sire(&c);
    let young_dam = pet(&c, &c.owner, Gender::Female, Species::Dog, BIRTH + 50);
    assert_eq!(
        c.client
            .check_breeding_eligibility(&s, &young_dam, &(ADULT + 49)),
        BreedingCheck::ParentTooYoung
    );
    assert_eq!(
        c.client
            .check_breeding_eligibility(&s, &young_dam, &(ADULT + 50)),
        BreedingCheck::Eligible
    );
}

#[test]
fn future_or_pre_birth_dates_are_rejected() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d, &(NOW + 1)),
        BreedingCheck::InvalidBreedingDate
    );
    assert_eq!(
        mate_err(&c, s, d, NOW + 1),
        err(ContractError::InvalidTimestamp)
    );
    assert_eq!(
        mate_err(&c, s, d, BIRTH - 1),
        err(ContractError::InvalidTimestamp)
    );
    // Exactly "now" is allowed.
    c.client
        .record_mating(&c.owner, &s, &d, &NOW, &notes(&c.env));
}

// ---- cooldown boundaries -------------------------------------------------

#[test]
fn dam_cooldown_is_enforced_at_exact_boundary() {
    let c = setup();
    let (s1, s2, s3, d) = (sire(&c), sire(&c), sire(&c), dam(&c));
    c.client
        .record_mating(&c.owner, &s1, &d, &ADULT, &notes(&c.env));

    let just_before = ADULT + DAM_COOLDOWN - 1;
    assert_eq!(
        c.client.check_breeding_eligibility(&s2, &d, &just_before),
        BreedingCheck::DamCooldownActive
    );
    assert_eq!(
        mate_err(&c, s2, d, just_before),
        err(ContractError::BreedingCooldownActive)
    );
    c.client
        .record_mating(&c.owner, &s3, &d, &(ADULT + DAM_COOLDOWN), &notes(&c.env));
    assert_eq!(
        c.client.get_last_mating_date(&d),
        Some(ADULT + DAM_COOLDOWN)
    );
}

#[test]
fn sire_cooldown_is_enforced_at_exact_boundary() {
    let c = setup();
    let (s, d1, d2, d3) = (sire(&c), dam(&c), dam(&c), dam(&c));
    c.client
        .record_mating(&c.owner, &s, &d1, &ADULT, &notes(&c.env));

    let just_before = ADULT + SIRE_COOLDOWN - 1;
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d2, &just_before),
        BreedingCheck::SireCooldownActive
    );
    assert_eq!(
        mate_err(&c, s, d2, just_before),
        err(ContractError::BreedingCooldownActive)
    );
    c.client
        .record_mating(&c.owner, &s, &d3, &(ADULT + SIRE_COOLDOWN), &notes(&c.env));
}

#[test]
fn backdated_mating_cannot_bypass_cooldown() {
    let c = setup();
    let (s1, s2, d) = (sire(&c), sire(&c), dam(&c));
    let later = ADULT + 10 * DAM_COOLDOWN;
    c.client
        .record_mating(&c.owner, &s1, &d, &later, &notes(&c.env));
    // Well before the recorded mating: still rejected, dates are monotonic.
    assert_eq!(
        mate_err(&c, s2, d, ADULT),
        err(ContractError::BreedingCooldownActive)
    );
}

// ---- replay / idempotency ------------------------------------------------

#[test]
fn replayed_mating_is_idempotent() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    let id = c
        .client
        .record_mating(&c.owner, &s, &d, &ADULT, &String::from_str(&c.env, "first"));
    let events_before = c.env.events().all().len();

    // Same identity, even with different notes: no new event, record or
    // cooldown movement, and the cooldown it created does not reject it.
    let replay = c.client.record_mating(
        &c.owner,
        &s,
        &d,
        &ADULT,
        &String::from_str(&c.env, "second"),
    );
    assert_eq!(replay, id);
    assert_eq!(c.env.events().all().len(), events_before);
    assert_eq!(c.client.get_breeding_count(&s), 1);
    assert_eq!(c.client.get_breeding_count(&d), 1);
    assert_eq!(
        c.client.get_breeding_record(&id).unwrap().notes,
        String::from_str(&c.env, "first")
    );
    assert!(c.client.get_breeding_record(&(id + 1)).is_none());
}

#[test]
fn replay_still_requires_authorization() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    c.client
        .record_mating(&c.owner, &s, &d, &ADULT, &notes(&c.env));
    let stranger = Address::generate(&c.env);
    assert_eq!(
        c.client
            .try_record_mating(&stranger, &s, &d, &ADULT, &notes(&c.env))
            .unwrap_err()
            .unwrap(),
        err(ContractError::NotPetOwner)
    );
}

// ---- policy administration -----------------------------------------------

#[test]
fn only_admin_can_set_policy_and_bounds_are_checked() {
    let c = setup();
    let policy = c.client.get_breeding_policy();
    assert_eq!(
        c.client
            .try_set_breeding_policy(&c.owner, &policy)
            .unwrap_err()
            .unwrap(),
        err(ContractError::Unauthorized)
    );
    let mut bad = policy.clone();
    bad.max_coi_bp = 10_001;
    assert_eq!(
        c.client
            .try_set_breeding_policy(&c.admin, &bad)
            .unwrap_err()
            .unwrap(),
        err(ContractError::InvalidInput)
    );
}

#[test]
fn inbreeding_threshold_is_part_of_eligibility() {
    let c = setup();
    let (s, d) = (sire(&c), dam(&c));
    let mut policy = c.client.get_breeding_policy();
    policy.max_coi_bp = 0; // any COI (even 0) is >= 0: always exceeded
    c.client.set_breeding_policy(&c.admin, &policy);
    assert_eq!(
        c.client.check_breeding_eligibility(&s, &d, &ADULT),
        BreedingCheck::InbreedingThresholdExceeded
    );
    assert_eq!(
        mate_err(&c, s, d, ADULT),
        err(ContractError::InbreedingThresholdExceeded)
    );
}
