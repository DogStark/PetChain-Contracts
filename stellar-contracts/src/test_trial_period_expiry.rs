//! Regression tests for issue #1657: trial-period expiry must publish a typed
//! on-chain event on the first interaction after `trial_expires_at`.

use crate::*;
use soroban_sdk::{testutils::Address as _, Env, Val};

const NOW: u64 = 1_700_000_000;
const EXPIRY: u64 = NOW + 100;
const ADOPTION_ID: u64 = 42;

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = NOW);

    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    let adopter = Address::generate(&env);
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
    (env, client, owner, adopter, pet_id)
}

fn trial_events(env: &Env) -> soroban_sdk::Vec<(soroban_sdk::Vec<Val>, Val)> {
    let topic = Symbol::new(env, "TrialPeriodExpired");
    let mut out = soroban_sdk::Vec::new(env);
    for (_, topics, data) in env.events().all().iter() {
        let first = topics
            .get(0)
            .and_then(|t| Symbol::try_from_val(env, &t).ok());
        if first == Some(topic.clone()) {
            out.push_back((topics, data));
        }
    }
    out
}

#[test]
fn auto_convert_emits_expiry_event_with_new_ownership_state() {
    let (env, client, owner, adopter, pet_id) = setup();
    client.start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &EXPIRY, &true);
    assert_eq!(trial_events(&env).len(), 0);

    env.ledger().with_mut(|l| l.timestamp = EXPIRY);
    let event = client.settle_adoption_trial(&ADOPTION_ID).unwrap();

    assert_eq!(event.version, EVENT_SCHEMA_VERSION);
    assert_eq!(event.pet_id, pet_id);
    assert_eq!(event.adoption_id, ADOPTION_ID);
    assert_eq!(event.outcome, TrialOutcome::Converted);
    assert_eq!(event.new_owner, adopter);
    assert_eq!(event.previous_owner, owner);
    assert_eq!(event.timestamp, EXPIRY);
    assert_eq!(client.get_pet(&pet_id).unwrap().owner, adopter);

    // The event is typed and decodable by integrators from raw event data.
    let events = trial_events(&env);
    assert_eq!(events.len(), 1);
    let (topics, data) = events.get(0).unwrap();
    assert_eq!(topics.len(), 3);
    let decoded = TrialPeriodExpiredEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(decoded, event);
}

#[test]
fn auto_reverse_emits_expiry_event_with_previous_owner() {
    let (env, client, owner, adopter, pet_id) = setup();
    client.start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &EXPIRY, &false);

    // During the trial the candidate adopter holds the pet; on expiry without
    // conversion it must go back to the address recorded at trial start.
    client.transfer_pet(&owner, &pet_id, &adopter);
    assert_eq!(client.get_pet(&pet_id).unwrap().owner, adopter);

    env.ledger().with_mut(|l| l.timestamp = EXPIRY);
    let event = client.settle_adoption_trial(&ADOPTION_ID).unwrap();

    assert_eq!(event.outcome, TrialOutcome::Reversed);
    assert_eq!(event.new_owner, owner);
    assert_eq!(event.previous_owner, owner);
    assert_eq!(client.get_pet(&pet_id).unwrap().owner, owner);
    assert_eq!(trial_events(&env).len(), 1);
}

#[test]
fn expiry_event_is_emitted_only_on_first_interaction() {
    let (env, client, _owner, adopter, pet_id) = setup();
    client.start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &EXPIRY, &true);

    env.ledger().with_mut(|l| l.timestamp = EXPIRY);
    assert!(client.settle_adoption_trial(&ADOPTION_ID).is_some());
    assert!(client.settle_adoption_trial(&ADOPTION_ID).is_none());
    let trial = client.get_adoption_trial(&ADOPTION_ID).unwrap();
    assert_eq!(trial.settled, true);
    assert_eq!(trial.outcome, Some(TrialOutcome::Converted));
    assert_eq!(trial.settled_at, EXPIRY);
    assert_eq!(trial_events(&env).len(), 1);
}

#[test]
fn no_expiry_event_before_the_trial_expires() {
    let (env, client, owner, adopter, pet_id) = setup();
    client.start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &EXPIRY, &true);

    assert!(client.settle_adoption_trial(&ADOPTION_ID).is_none());
    assert_eq!(trial_events(&env).len(), 0);
    assert_eq!(client.get_pet(&pet_id).unwrap().owner, owner);

    let trial = client.get_adoption_trial(&ADOPTION_ID).unwrap();
    assert_eq!(trial.settled, false);
    assert_eq!(trial.outcome, None);
    assert_eq!(trial_events(&env).len(), 0);
}

#[test]
fn reading_a_trial_after_expiry_settles_it_lazily() {
    let (env, client, _owner, adopter, pet_id) = setup();
    client.start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &EXPIRY, &true);

    env.ledger().with_mut(|l| l.timestamp = EXPIRY);
    let trial = client.get_adoption_trial(&ADOPTION_ID).unwrap();
    assert_eq!(trial.settled, true);
    assert_eq!(trial.outcome, Some(TrialOutcome::Converted));

    let events = trial_events(&env);
    assert_eq!(events.len(), 1);
    let (_, data) = events.get(0).unwrap();
    let decoded = TrialPeriodExpiredEvent::try_from_val(&env, &data).unwrap();
    assert_eq!(decoded.outcome, TrialOutcome::Converted);
    assert_eq!(decoded.new_owner, adopter);
}

#[test]
fn rejected_trial_creation_leaves_no_state_or_events() {
    let (env, client, _owner, adopter, pet_id) = setup();

    let err = client
        .try_start_adoption_trial(&pet_id, &ADOPTION_ID, &adopter, &NOW, &true)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::InvalidInput.into());
    assert!(client.get_adoption_trial(&ADOPTION_ID).is_none());
    assert_eq!(trial_events(&env).len(), 0);
}
