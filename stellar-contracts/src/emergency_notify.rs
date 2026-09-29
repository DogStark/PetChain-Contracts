//! Storage keys, value types and canonical hashing for replay-protected
//! emergency notifications (Issue #1338).
//!
//! A notification request is bound to `(pet_id, event_id, recipient, nonce)`.
//! The contract methods (`notify_emergency_recipient`, ...) live in `lib.rs`
//! for the same reason as the dispute domain (see `disputes.rs`).
//!
//! Replay rules:
//! * The same request (same pet, event, recipient and nonce) submitted again
//!   while it is still inside its validity window is an idempotent retry: the
//!   original receipt is returned and no new notification is emitted.
//! * The same request submitted after its window has closed is rejected with
//!   `NotificationExpired` — an expired nonce can never be reused.
//! * A *new* nonce for a `(pet, event, recipient)` that was already notified
//!   is rejected with `NotificationAlreadySent`, so a contact is notified at
//!   most once per emergency event. Escalations must use a new event id.
//! * Different event ids are always distinct notifications.
//!
//! Recipients are identified by a salted digest of the contact's channels
//! (see [`emergency_recipient_id`]) so no phone number or e-mail address is
//! written to contract storage or events.

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contracttype, Address, Bytes, BytesN, Env};

use crate::EmergencyContact;

/// Domain tag for [`emergency_notification_request_id`].
pub const EMERGENCY_NOTIFY_REQUEST_DOMAIN: &[u8] = b"petchain:emergency-notify:v1";

/// Domain tag for [`emergency_recipient_id`].
pub const EMERGENCY_RECIPIENT_DOMAIN: &[u8] = b"petchain:emergency-recipient:v1";

/// Longest validity window a notification request may ask for. Bounds how
/// long a captured request stays replayable as an idempotent retry.
pub const MAX_NOTIFY_REQUEST_TTL_SECS: u64 = 3_600;

#[contracttype]
pub enum EmergencyNotifyKey {
    /// request_id -> EmergencyNotification
    Request(BytesN<32>),
    /// (pet_id, event_id, recipient) -> request_id of the accepted notification
    Delivery((u64, BytesN<32>, BytesN<32>)),
}

/// An accepted, replay-protected emergency notification.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyNotification {
    pub request_id: BytesN<32>,
    pub pet_id: u64,
    pub event_id: BytesN<32>,
    pub recipient: BytesN<32>,
    pub nonce: u64,
    pub submitter: Address,
    pub accepted_at: u64,
    pub expires_at: u64,
}

/// Result of `notify_emergency_recipient`. `replayed` is true when the call
/// was an idempotent retry of an already-accepted request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmergencyNotificationReceipt {
    pub notification: EmergencyNotification,
    pub replayed: bool,
}

/// `sha256(domain || u64be(pet_id) || event_id || recipient || u64be(nonce))`
pub fn emergency_notification_request_id(
    env: &Env,
    pet_id: u64,
    event_id: &BytesN<32>,
    recipient: &BytesN<32>,
    nonce: u64,
) -> BytesN<32> {
    let mut preimage = Bytes::from_slice(env, EMERGENCY_NOTIFY_REQUEST_DOMAIN);
    preimage.extend_from_slice(&pet_id.to_be_bytes());
    preimage.append(&Bytes::from(event_id.clone()));
    preimage.append(&Bytes::from(recipient.clone()));
    preimage.extend_from_slice(&nonce.to_be_bytes());
    env.crypto().sha256(&preimage).into()
}

/// `sha256(domain || u64be(pet_id) || xdr(phone) || xdr(email))`
///
/// Salting with `pet_id` keeps the same person's id different across pets,
/// so recipient ids cannot be correlated between pets.
pub fn emergency_recipient_id(env: &Env, pet_id: u64, contact: &EmergencyContact) -> BytesN<32> {
    let mut preimage = Bytes::from_slice(env, EMERGENCY_RECIPIENT_DOMAIN);
    preimage.extend_from_slice(&pet_id.to_be_bytes());
    preimage.append(&contact.phone.clone().to_xdr(env));
    preimage.append(&contact.email.clone().to_xdr(env));
    env.crypto().sha256(&preimage).into()
}
