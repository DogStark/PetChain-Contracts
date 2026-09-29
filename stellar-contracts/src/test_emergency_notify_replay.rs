// Replay protection for emergency notifications (Issue #1338).
//
// Covers: idempotent retries, replay after expiry, new nonces for an
// already-notified (pet, event, recipient), distinct events and recipients,
// unknown recipients and request-window validation.

use crate::*;
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{BytesN, Env, Symbol, TryFromVal};

// After the fixture pet's 2020 birthday, which must not be in the future.
const NOW: u64 = 1_800_000_000;

struct Fixture {
    env: Env,
    client: PetChainContractClient<'static>,
    owner: Address,
    pet_id: u64,
    contacts: Vec<EmergencyContact>,
}

fn contact(env: &Env, name: &str, phone: &str, email: &str, priority: u32) -> EmergencyContact {
    EmergencyContact {
        name: String::from_str(env, name),
        phone: String::from_str(env, phone),
        email: String::from_str(env, email),
        relationship: String::from_str(env, "Family"),
        is_primary: priority == 1,
        priority,
    }
}

fn register_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Buddy"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Golden Retriever"),
        &String::from_str(env, "Golden"),
        &25u32,
        &None,
        &PrivacyLevel::Public,
    )
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = NOW);
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = register_pet(&env, &client, &owner);

    let mut contacts = Vec::new(&env);
    contacts.push_back(contact(&env, "Jane", "555-0100", "jane@example.com", 1));
    contacts.push_back(contact(&env, "John", "555-0101", "john@example.com", 2));
    client.set_emergency_contacts(
        &pet_id,
        &contacts,
        &Vec::new(&env),
        &String::from_str(&env, ""),
    );

    Fixture {
        env,
        client,
        owner,
        pet_id,
        contacts,
    }
}

impl Fixture {
    fn recipient(&self, index: u32) -> BytesN<32> {
        self.client
            .get_emergency_recipient_id(&self.pet_id, &self.contacts.get(index).unwrap())
    }

    fn event(&self, tag: u8) -> BytesN<32> {
        BytesN::from_array(&self.env, &[tag; 32])
    }

    fn notify(
        &self,
        event: &BytesN<32>,
        recipient: &BytesN<32>,
        nonce: u64,
        expires_at: u64,
    ) -> EmergencyNotificationReceipt {
        self.client.notify_emergency_recipient(
            &self.owner,
            &self.pet_id,
            event,
            recipient,
            &nonce,
            &expires_at,
        )
    }

    fn try_notify_err(
        &self,
        event: &BytesN<32>,
        recipient: &BytesN<32>,
        nonce: u64,
        expires_at: u64,
    ) -> soroban_sdk::Error {
        self.client
            .try_notify_emergency_recipient(
                &self.owner,
                &self.pet_id,
                event,
                recipient,
                &nonce,
                &expires_at,
            )
            .unwrap_err()
            .unwrap()
    }

    fn set_time(&self, t: u64) {
        self.env.ledger().with_mut(|l| l.timestamp = t);
    }
}

/// Number of `EmergencyRecipientNotified` events published so far.
fn notified_events(env: &Env) -> u32 {
    let topic = Symbol::new(env, "EmergencyRecipientNotified");
    let mut n = 0;
    for (_, topics, _) in env.events().all().iter() {
        let first = topics
            .get(0)
            .and_then(|t| Symbol::try_from_val(env, &t).ok());
        if first == Some(topic.clone()) {
            n += 1;
        }
    }
    n
}

// --- replay / retry ---

#[test]
fn fresh_request_is_accepted_and_recorded() {
    let f = setup();
    let receipt = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);

    assert!(!receipt.replayed);
    let n = &receipt.notification;
    assert_eq!(n.pet_id, f.pet_id);
    assert_eq!(n.nonce, 7);
    assert_eq!(n.submitter, f.owner);
    assert_eq!(n.accepted_at, NOW);
    assert_eq!(n.expires_at, NOW + 600);
    assert_eq!(
        n.request_id,
        emergency_notification_request_id(&f.env, f.pet_id, &f.event(1), &f.recipient(0), 7)
    );
    assert_eq!(
        f.client.get_emergency_notification(&n.request_id),
        Some(n.clone())
    );
}

#[test]
fn exact_retry_is_idempotent_and_does_not_renotify() {
    let f = setup();
    let first = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);
    assert_eq!(notified_events(&f.env), 1);

    f.set_time(NOW + 30);
    // A retry may carry a different expiry; the original request wins.
    let retry = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 900);

    assert!(retry.replayed);
    assert_eq!(retry.notification, first.notification);
    // The replay published no new notification event.
    assert_eq!(notified_events(&f.env), 1);
}

#[test]
fn replay_after_expiry_is_rejected() {
    let f = setup();
    f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);

    // Expiry is inclusive: the request is dead at exactly `expires_at`.
    f.set_time(NOW + 600);
    let err = f.try_notify_err(&f.event(1), &f.recipient(0), 7, NOW + 1_200);
    assert_eq!(err, ContractError::NotificationExpired.into());
}

#[test]
fn expired_nonce_cannot_be_reused_with_a_fresh_window() {
    let f = setup();
    f.notify(&f.event(1), &f.recipient(0), 7, NOW + 60);

    f.set_time(NOW + 10_000);
    let err = f.try_notify_err(&f.event(1), &f.recipient(0), 7, NOW + 10_600);
    assert_eq!(err, ContractError::NotificationExpired.into());
}

#[test]
fn new_nonce_for_already_notified_recipient_is_rejected() {
    let f = setup();
    f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);

    let err = f.try_notify_err(&f.event(1), &f.recipient(0), 8, NOW + 600);
    assert_eq!(err, ContractError::NotificationAlreadySent.into());
}

// --- distinct events / recipients ---

#[test]
fn different_events_remain_distinct() {
    let f = setup();
    let a = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);
    // Same recipient and even the same nonce: a new event is a new notification.
    let b = f.notify(&f.event(2), &f.recipient(0), 7, NOW + 600);

    assert!(!a.replayed);
    assert!(!b.replayed);
    assert_ne!(a.notification.request_id, b.notification.request_id);
}

#[test]
fn different_recipients_of_one_event_remain_distinct() {
    let f = setup();
    let a = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);
    let b = f.notify(&f.event(1), &f.recipient(1), 7, NOW + 600);

    assert!(!b.replayed);
    assert_ne!(a.notification.request_id, b.notification.request_id);
    assert_ne!(f.recipient(0), f.recipient(1));
}

#[test]
fn recipient_ids_are_bound_to_the_pet() {
    let f = setup();
    let other_pet = register_pet(&f.env, &f.client, &f.owner);
    let c = f.contacts.get(0).unwrap();

    assert_ne!(
        f.client.get_emergency_recipient_id(&f.pet_id, &c),
        f.client.get_emergency_recipient_id(&other_pet, &c)
    );
    // A recipient id derived for one pet is not a contact of another pet.
    f.client.set_emergency_contacts(
        &other_pet,
        &f.contacts,
        &Vec::new(&f.env),
        &String::from_str(&f.env, ""),
    );
    let err = f
        .client
        .try_notify_emergency_recipient(
            &f.owner,
            &other_pet,
            &f.event(1),
            &f.recipient(0),
            &7,
            &(NOW + 600),
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::UnknownEmergencyRecipient.into());
}

#[test]
fn unknown_recipient_is_rejected() {
    let f = setup();
    let stranger = contact(&f.env, "Eve", "555-9999", "eve@example.com", 3);
    let id = f.client.get_emergency_recipient_id(&f.pet_id, &stranger);

    let err = f.try_notify_err(&f.event(1), &id, 7, NOW + 600);
    assert_eq!(err, ContractError::UnknownEmergencyRecipient.into());
}

// --- request window ---

#[test]
fn request_already_past_its_expiry_is_rejected() {
    let f = setup();
    let err = f.try_notify_err(&f.event(1), &f.recipient(0), 7, NOW);
    assert_eq!(err, ContractError::NotificationExpired.into());
}

#[test]
fn request_window_longer_than_max_ttl_is_rejected() {
    let f = setup();
    let err = f.try_notify_err(
        &f.event(1),
        &f.recipient(0),
        7,
        NOW + MAX_NOTIFY_REQUEST_TTL_SECS + 1,
    );
    assert_eq!(err, ContractError::InvalidInput.into());

    // Exactly the max is fine.
    f.notify(
        &f.event(1),
        &f.recipient(0),
        7,
        NOW + MAX_NOTIFY_REQUEST_TTL_SECS,
    );
}

// --- authorization ---

#[test]
fn unauthorized_caller_is_rejected() {
    let f = setup();
    let stranger = Address::generate(&f.env);
    let err = f
        .client
        .try_notify_emergency_recipient(
            &stranger,
            &f.pet_id,
            &f.event(1),
            &f.recipient(0),
            &7,
            &(NOW + 600),
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized.into());
}

#[test]
fn responder_retry_of_owner_request_is_idempotent() {
    let f = setup();
    let responder = Address::generate(&f.env);
    f.client.add_emergency_responder(&f.pet_id, &responder);

    let first = f.notify(&f.event(1), &f.recipient(0), 7, NOW + 600);
    let retry = f.client.notify_emergency_recipient(
        &responder,
        &f.pet_id,
        &f.event(1),
        &f.recipient(0),
        &7,
        &(NOW + 600),
    );

    assert!(retry.replayed);
    assert_eq!(retry.notification.submitter, f.owner);
    assert_eq!(retry.notification, first.notification);
}
