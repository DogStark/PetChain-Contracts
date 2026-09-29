// Vet credential issuer key versions, rotation and revocation (Issue #1336).
// Policy: docs/vet-credential-issuers.md.

use crate::*;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{BytesN, Env};

const NOW: u64 = 1_000_000;
const DAY: u64 = 86_400;
const KEY_TTL: u64 = 365 * DAY;

struct Fixture {
    env: Env,
    client: PetChainContractClient<'static>,
    admin: Address,
    issuer: Address,
    vet: Address,
}

fn key(env: &Env, tag: u8) -> BytesN<32> {
    BytesN::from_array(env, &[tag; 32])
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = NOW);
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.init_admin(&admin);

    let vet = Address::generate(&env);
    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Who"),
        &String::from_str(&env, "LIC-001"),
        &String::from_str(&env, "General"),
    );

    let issuer = Address::generate(&env);
    client.register_credential_issuer(&admin, &issuer, &key(&env, 1), &(NOW + KEY_TTL));

    Fixture {
        env,
        client,
        admin,
        issuer,
        vet,
    }
}

impl Fixture {
    fn issue(&self, version: u32, expires_at: u64) -> VetCredential {
        self.client
            .issue_vet_credential(&self.issuer, &version, &self.vet, &expires_at)
    }

    fn issue_err(&self, version: u32, expires_at: u64) -> soroban_sdk::Error {
        self.client
            .try_issue_vet_credential(&self.issuer, &version, &self.vet, &expires_at)
            .unwrap_err()
            .unwrap()
    }

    fn rotate(&self, tag: u8, overlap: u64) -> IssuerKeyVersion {
        let now = self.env.ledger().timestamp();
        self.client.rotate_credential_issuer_key(
            &self.issuer,
            &self.issuer,
            &key(&self.env, tag),
            &(now + KEY_TTL),
            &overlap,
        )
    }

    fn set_time(&self, t: u64) {
        self.env.ledger().with_mut(|l| l.timestamp = t);
    }

    fn status(&self, id: u64) -> VetCredentialStatus {
        self.client.verify_vet_credential(&id)
    }
}

// --- registration / baseline ---

#[test]
fn registered_issuer_mints_valid_credentials() {
    let f = setup();
    let issuer = f.client.get_credential_issuer(&f.issuer).unwrap();
    assert_eq!(issuer.current_version, 1);
    assert_eq!(issuer.revoked_at, None);

    let cred = f.issue(1, NOW + 30 * DAY);
    assert_eq!(cred.key_version, 1);
    assert_eq!(cred.vet, f.vet);
    assert_eq!(f.status(cred.id), VetCredentialStatus::Valid);
    assert_eq!(f.client.get_vet_credential(&cred.id), Some(cred));
}

#[test]
fn registering_twice_or_as_non_admin_fails() {
    let f = setup();
    let err = f
        .client
        .try_register_credential_issuer(&f.admin, &f.issuer, &key(&f.env, 2), &(NOW + KEY_TTL))
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::IssuerAlreadyRegistered.into());

    let stranger = Address::generate(&f.env);
    assert!(f
        .client
        .try_register_credential_issuer(
            &stranger,
            &Address::generate(&f.env),
            &key(&f.env, 3),
            &(NOW + KEY_TTL)
        )
        .is_err());
}

#[test]
fn unknown_issuer_version_and_vet_are_rejected() {
    let f = setup();
    assert_eq!(
        f.issue_err(2, NOW + DAY),
        ContractError::IssuerKeyVersionNotFound.into()
    );

    let err = f
        .client
        .try_issue_vet_credential(&Address::generate(&f.env), &1, &f.vet, &(NOW + DAY))
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::IssuerNotFound.into());

    let err = f
        .client
        .try_issue_vet_credential(&f.issuer, &1, &Address::generate(&f.env), &(NOW + DAY))
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::VetNotFound.into());

    assert_eq!(f.status(999), VetCredentialStatus::NotFound);
}

#[test]
fn credential_cannot_outlive_its_signing_key() {
    let f = setup();
    assert_eq!(
        f.issue_err(1, NOW + KEY_TTL + 1),
        ContractError::InvalidInput.into()
    );
    assert_eq!(f.issue_err(1, NOW), ContractError::InvalidInput.into());
    f.issue(1, NOW + KEY_TTL);
}

// --- rotation ---

#[test]
fn new_credentials_verify_after_rotation() {
    let f = setup();
    let v2 = f.rotate(2, 0);
    assert_eq!(v2.version, 2);
    assert_eq!(
        f.client
            .get_credential_issuer(&f.issuer)
            .unwrap()
            .current_version,
        2
    );

    let cred = f.issue(2, NOW + 30 * DAY);
    assert_eq!(f.status(cred.id), VetCredentialStatus::Valid);
}

#[test]
fn old_credentials_stay_valid_after_rotation_until_they_expire() {
    let f = setup();
    let old = f.issue(1, NOW + 10 * DAY);
    f.rotate(2, 0);

    assert_eq!(f.status(old.id), VetCredentialStatus::Valid);
    f.set_time(NOW + 10 * DAY - 1);
    assert_eq!(f.status(old.id), VetCredentialStatus::Valid);
    f.set_time(NOW + 10 * DAY);
    assert_eq!(f.status(old.id), VetCredentialStatus::Expired);
}

#[test]
fn old_key_mints_only_within_the_overlap_window() {
    let f = setup();
    f.rotate(2, DAY);
    let v1 = f.client.get_issuer_key_version(&f.issuer, &1).unwrap();
    assert_eq!(v1.mint_until, Some(NOW + DAY));

    f.set_time(NOW + DAY - 1);
    let during = f.issue(1, NOW + 30 * DAY);
    assert_eq!(f.status(during.id), VetCredentialStatus::Valid);

    f.set_time(NOW + DAY);
    assert_eq!(
        f.issue_err(1, NOW + 30 * DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );
    // The credential minted during the overlap is unaffected.
    assert_eq!(f.status(during.id), VetCredentialStatus::Valid);
}

#[test]
fn zero_overlap_retires_the_old_key_immediately() {
    let f = setup();
    f.rotate(2, 0);
    assert_eq!(
        f.issue_err(1, NOW + DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );
}

#[test]
fn overlap_is_capped_by_policy_and_by_old_key_expiry() {
    let f = setup();
    let err = f
        .client
        .try_rotate_credential_issuer_key(
            &f.issuer,
            &f.issuer,
            &key(&f.env, 2),
            &(NOW + KEY_TTL),
            &(MAX_ISSUER_ROTATION_OVERLAP_SECS + 1),
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::InvalidInput.into());

    // Old key expires in 1 day; a 7-day overlap is clipped to that expiry.
    let issuer = Address::generate(&f.env);
    f.client
        .register_credential_issuer(&f.admin, &issuer, &key(&f.env, 9), &(NOW + DAY));
    f.client.rotate_credential_issuer_key(
        &issuer,
        &issuer,
        &key(&f.env, 10),
        &(NOW + KEY_TTL),
        &MAX_ISSUER_ROTATION_OVERLAP_SECS,
    );
    assert_eq!(
        f.client
            .get_issuer_key_version(&issuer, &1)
            .unwrap()
            .mint_until,
        Some(NOW + DAY)
    );
}

#[test]
fn expired_key_cannot_mint() {
    let f = setup();
    f.set_time(NOW + KEY_TTL);
    assert_eq!(
        f.issue_err(1, NOW + KEY_TTL + DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );
}

#[test]
fn rotation_rejects_key_reuse() {
    let f = setup();
    f.rotate(2, 0);
    let err = f
        .client
        .try_rotate_credential_issuer_key(
            &f.issuer,
            &f.issuer,
            &key(&f.env, 1),
            &(NOW + KEY_TTL),
            &0,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::IssuerKeyReused.into());
}

#[test]
fn rotation_requires_issuer_or_admin() {
    let f = setup();
    let stranger = Address::generate(&f.env);
    let err = f
        .client
        .try_rotate_credential_issuer_key(
            &stranger,
            &f.issuer,
            &key(&f.env, 2),
            &(NOW + KEY_TTL),
            &0,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::Unauthorized.into());

    let v2 = f.client.rotate_credential_issuer_key(
        &f.admin,
        &f.issuer,
        &key(&f.env, 2),
        &(NOW + KEY_TTL),
        &0,
    );
    assert_eq!(v2.version, 2);
}

// --- revocation ---

#[test]
fn revoking_a_key_version_invalidates_only_its_credentials() {
    let f = setup();
    let old = f.issue(1, NOW + 30 * DAY);
    f.rotate(2, DAY);
    let new = f.issue(2, NOW + 30 * DAY);

    let revoked = f
        .client
        .revoke_credential_issuer_key(&f.admin, &f.issuer, &1);
    assert_eq!(revoked.revoked_at, Some(NOW));

    assert_eq!(f.status(old.id), VetCredentialStatus::KeyVersionRevoked);
    assert_eq!(f.status(new.id), VetCredentialStatus::Valid);
    // Revocation also closes the remaining overlap window.
    assert_eq!(
        f.issue_err(1, NOW + DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );
}

#[test]
fn issuer_recovers_from_current_key_revocation_by_rotating() {
    let f = setup();
    f.client
        .revoke_credential_issuer_key(&f.admin, &f.issuer, &1);
    assert_eq!(
        f.issue_err(1, NOW + DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );

    f.rotate(2, DAY);
    // The revoked key gains no overlap window from the rotation.
    assert_eq!(
        f.client
            .get_issuer_key_version(&f.issuer, &1)
            .unwrap()
            .mint_until,
        None
    );
    assert_eq!(
        f.issue_err(1, NOW + DAY),
        ContractError::IssuerKeyVersionInactive.into()
    );
    let cred = f.issue(2, NOW + DAY);
    assert_eq!(f.status(cred.id), VetCredentialStatus::Valid);
}

#[test]
fn revoked_issuer_cannot_mint_rotate_or_keep_valid_credentials() {
    let f = setup();
    let v1_cred = f.issue(1, NOW + 30 * DAY);
    f.rotate(2, 0);
    let v2_cred = f.issue(2, NOW + 30 * DAY);

    let record = f.client.revoke_credential_issuer(&f.admin, &f.issuer);
    assert_eq!(record.revoked_at, Some(NOW));

    assert_eq!(
        f.issue_err(2, NOW + DAY),
        ContractError::IssuerRevoked.into()
    );
    let err = f
        .client
        .try_rotate_credential_issuer_key(
            &f.issuer,
            &f.issuer,
            &key(&f.env, 3),
            &(NOW + KEY_TTL),
            &0,
        )
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::IssuerRevoked.into());

    assert_eq!(f.status(v1_cred.id), VetCredentialStatus::IssuerRevoked);
    assert_eq!(f.status(v2_cred.id), VetCredentialStatus::IssuerRevoked);
}

#[test]
fn revocations_are_idempotent_and_admin_only() {
    let f = setup();
    let first = f
        .client
        .revoke_credential_issuer_key(&f.admin, &f.issuer, &1);
    f.set_time(NOW + DAY);
    let again = f
        .client
        .revoke_credential_issuer_key(&f.admin, &f.issuer, &1);
    assert_eq!(first, again);

    let stranger = Address::generate(&f.env);
    assert!(f
        .client
        .try_revoke_credential_issuer(&stranger, &f.issuer)
        .is_err());
    assert!(f
        .client
        .try_revoke_credential_issuer_key(&stranger, &f.issuer, &1)
        .is_err());

    let err = f
        .client
        .try_revoke_credential_issuer_key(&f.admin, &f.issuer, &5)
        .unwrap_err()
        .unwrap();
    assert_eq!(err, ContractError::IssuerKeyVersionNotFound.into());
}
