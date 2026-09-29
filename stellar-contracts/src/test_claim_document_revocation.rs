// Claim-document status/version lifecycle tests (Issue #1341).
use crate::{
    ClaimDocumentStatus, ContractError, Gender, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    vec, Address, BytesN, Env, Error, String,
};

const CLAIM: u64 = 7;

struct Ctx<'a> {
    env: Env,
    client: PetChainContractClient<'a>,
    admin: Address,
    owner: Address,
    pet_id: u64,
}

fn register_pet(env: &Env, client: &PetChainContractClient, owner: &Address) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, "Rex"),
        &String::from_str(env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(env, "Labrador"),
        &String::from_str(env, "Brown"),
        &20,
        &None,
        &PrivacyLevel::Public,
    )
}

fn setup<'a>() -> Ctx<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    client.init_admin(&admin);
    let pet_id = register_pet(&env, &client, &owner);
    Ctx {
        env,
        client,
        admin,
        owner,
        pet_id,
    }
}

fn digest(env: &Env, b: u8) -> BytesN<32> {
    BytesN::from_array(env, &[b; 32])
}

fn err(e: ContractError) -> Error {
    Error::from(e)
}

// ---- status/version semantics -------------------------------------------

#[test]
fn submitted_document_is_active_version_one() {
    let c = setup();
    let d = digest(&c.env, 1);
    let idx = c
        .client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d);
    assert_eq!(idx, 0);

    let doc = c.client.get_claim_document(&CLAIM, &0).unwrap();
    assert_eq!(doc.status, ClaimDocumentStatus::Active);
    assert_eq!(doc.version, 1);
    assert_eq!(doc.supersedes, None);
    assert_eq!(doc.submitted_by, c.owner);
    assert!(c.client.is_claim_document_acceptable(&CLAIM, &0));
    // Legacy integrity check keeps working by index.
    assert!(c.client.verify_claim_document(&CLAIM, &0, &d));
}

#[test]
fn supersede_links_versions_and_blocks_old_version_from_approval() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    let new_idx = c
        .client
        .supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 2));
    assert_eq!(new_idx, 1);

    let old = c.client.get_claim_document(&CLAIM, &0).unwrap();
    let new = c.client.get_claim_document(&CLAIM, &1).unwrap();
    assert_eq!(old.status, ClaimDocumentStatus::Superseded);
    assert_eq!(old.superseded_by, Some(1));
    assert_eq!(old.status_changed_by, Some(c.owner.clone()));
    assert_eq!(new.status, ClaimDocumentStatus::Active);
    assert_eq!(new.version, 2);
    assert_eq!(new.supersedes, Some(0));

    assert_eq!(
        c.client
            .try_approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32])
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentSuperseded)
    );
    // A superseded document cannot be superseded again.
    assert_eq!(
        c.client
            .try_supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 3))
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentSuperseded)
    );
    c.client
        .approve_claim(&c.admin, &CLAIM, &vec![&c.env, 1u32]);
}

// ---- revocation ----------------------------------------------------------

#[test]
fn revoked_document_is_auditable_but_cannot_back_new_approval() {
    let c = setup();
    let d = digest(&c.env, 1);
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d);
    c.env.ledger().with_mut(|l| l.timestamp = 5_000);
    let reason = String::from_str(&c.env, "forged invoice");
    c.client
        .revoke_claim_document(&c.owner, &CLAIM, &0, &reason);

    // Auditable: record, digest, actor, time and reason are retained.
    let doc = c.client.get_claim_document(&CLAIM, &0).unwrap();
    assert_eq!(doc.status, ClaimDocumentStatus::Revoked);
    assert_eq!(doc.digest, d);
    assert_eq!(doc.status_changed_by, Some(c.owner.clone()));
    assert_eq!(doc.status_changed_at, Some(5_000));
    assert_eq!(doc.revocation_reason, Some(reason));
    assert!(c.client.verify_claim_document(&CLAIM, &0, &d));

    // Not acceptable for a new approval.
    assert!(!c.client.is_claim_document_acceptable(&CLAIM, &0));
    assert_eq!(
        c.client
            .try_approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32])
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
    assert!(c.client.get_claim_settlement(&CLAIM).is_none());
}

#[test]
fn revoked_superseded_version_is_terminal() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    c.client
        .supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 2));
    let reason = String::from_str(&c.env, "withdrawn");
    c.client
        .revoke_claim_document(&c.admin, &CLAIM, &0, &reason);
    assert_eq!(
        c.client.get_claim_document(&CLAIM, &0).unwrap().status,
        ClaimDocumentStatus::Revoked
    );
    // Revoked -> anything is rejected.
    assert_eq!(
        c.client
            .try_revoke_claim_document(&c.admin, &CLAIM, &0, &reason)
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
    assert_eq!(
        c.client
            .try_supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 3))
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
    // The replacement version is unaffected.
    assert!(c.client.is_claim_document_acceptable(&CLAIM, &1));
}

#[test]
fn revoked_digest_cannot_be_resubmitted_on_any_claim() {
    let c = setup();
    let d = digest(&c.env, 9);
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d);
    c.client
        .revoke_claim_document(&c.owner, &CLAIM, &0, &String::from_str(&c.env, "x"));

    assert_eq!(
        c.client
            .try_submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d)
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
    assert_eq!(
        c.client
            .try_submit_claim_document(&c.owner, &c.pet_id, &(CLAIM + 1), &d)
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 10));
    assert_eq!(
        c.client
            .try_supersede_claim_document(&c.owner, &CLAIM, &1, &d)
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentRevoked)
    );
}

#[test]
fn duplicate_digest_within_claim_is_rejected() {
    let c = setup();
    let d = digest(&c.env, 1);
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d);
    assert_eq!(
        c.client
            .try_submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &d)
            .unwrap_err()
            .unwrap(),
        err(ContractError::DuplicateClaimDocument)
    );
}

// ---- settled claims: documented historical behavior ------------------------

#[test]
fn settlement_is_immutable_and_survives_later_revocation() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 2));
    let settlement = c
        .client
        .approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32, 1u32]);
    assert_eq!(settlement.pet_id, c.pet_id);
    assert_eq!(
        settlement.doc_digests,
        vec![&c.env, digest(&c.env, 1), digest(&c.env, 2)]
    );
    assert_eq!(settlement.doc_versions, vec![&c.env, 1u32, 1u32]);
    assert!(!c.client.settlement_has_revoked_documents(&CLAIM));

    // Revocation after settlement is still allowed (fraud discovered later)...
    c.client
        .revoke_claim_document(&c.admin, &CLAIM, &1, &String::from_str(&c.env, "fraud"));
    // ...but does not unwind the settlement; it is flagged for audit instead.
    assert_eq!(c.client.get_claim_settlement(&CLAIM).unwrap(), settlement);
    assert!(c.client.settlement_has_revoked_documents(&CLAIM));
}

#[test]
fn settled_claim_rejects_further_approval_and_document_changes() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    c.client
        .approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32]);

    let settled = err(ContractError::ClaimAlreadySettled);
    assert_eq!(
        c.client
            .try_approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32])
            .unwrap_err()
            .unwrap(),
        settled
    );
    assert_eq!(
        c.client
            .try_submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 2))
            .unwrap_err()
            .unwrap(),
        settled
    );
    assert_eq!(
        c.client
            .try_supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 2))
            .unwrap_err()
            .unwrap(),
        settled
    );
}

// ---- authorization of status transitions ---------------------------------

#[test]
fn only_pet_owner_can_submit_or_supersede() {
    let c = setup();
    let stranger = Address::generate(&c.env);
    assert_eq!(
        c.client
            .try_submit_claim_document(&stranger, &c.pet_id, &CLAIM, &digest(&c.env, 1))
            .unwrap_err()
            .unwrap(),
        err(ContractError::NotPetOwner)
    );
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    for actor in [stranger, c.admin.clone()] {
        assert_eq!(
            c.client
                .try_supersede_claim_document(&actor, &CLAIM, &0, &digest(&c.env, 2))
                .unwrap_err()
                .unwrap(),
            err(ContractError::NotPetOwner)
        );
    }
}

#[test]
fn only_owner_or_admin_can_revoke() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    let stranger = Address::generate(&c.env);
    assert_eq!(
        c.client
            .try_revoke_claim_document(&stranger, &CLAIM, &0, &String::from_str(&c.env, "x"))
            .unwrap_err()
            .unwrap(),
        err(ContractError::Unauthorized)
    );
    assert_eq!(
        c.client.get_claim_document(&CLAIM, &0).unwrap().status,
        ClaimDocumentStatus::Active
    );
}

#[test]
fn only_admin_can_approve() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    assert_eq!(
        c.client
            .try_approve_claim(&c.owner, &CLAIM, &vec![&c.env, 0u32])
            .unwrap_err()
            .unwrap(),
        err(ContractError::Unauthorized)
    );
}

#[test]
fn transitions_require_signature() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    c.env.set_auths(&[]);
    assert!(c
        .client
        .try_revoke_claim_document(&c.owner, &CLAIM, &0, &String::from_str(&c.env, "x"))
        .is_err());
    assert!(c
        .client
        .try_supersede_claim_document(&c.owner, &CLAIM, &0, &digest(&c.env, 2))
        .is_err());
    assert!(c
        .client
        .try_approve_claim(&c.admin, &CLAIM, &vec![&c.env, 0u32])
        .is_err());
    assert_eq!(
        c.client.get_claim_document(&CLAIM, &0).unwrap().status,
        ClaimDocumentStatus::Active
    );
}

// ---- scope and input validation ------------------------------------------

#[test]
fn claim_is_bound_to_first_pet() {
    let c = setup();
    let other_owner = Address::generate(&c.env);
    let other_pet = register_pet(&c.env, &c.client, &other_owner);
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    assert_eq!(
        c.client
            .try_submit_claim_document(&other_owner, &other_pet, &CLAIM, &digest(&c.env, 2))
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimPetMismatch)
    );
}

#[test]
fn approval_indices_must_be_non_empty_canonical_and_existing() {
    let c = setup();
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 1));
    c.client
        .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 2));
    let invalid = err(ContractError::InvalidInput);
    for indices in [
        vec![&c.env],
        vec![&c.env, 1u32, 0u32],
        vec![&c.env, 0u32, 0u32],
    ] {
        assert_eq!(
            c.client
                .try_approve_claim(&c.admin, &CLAIM, &indices)
                .unwrap_err()
                .unwrap(),
            invalid
        );
    }
    assert_eq!(
        c.client
            .try_approve_claim(&c.admin, &CLAIM, &vec![&c.env, 5u32])
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentNotFound)
    );
    assert_eq!(
        c.client
            .try_approve_claim(&c.admin, &99u64, &vec![&c.env, 0u32])
            .unwrap_err()
            .unwrap(),
        err(ContractError::ClaimDocumentNotFound)
    );
}

#[test]
fn document_count_is_bounded() {
    let c = setup();
    for i in 0..crate::MAX_CLAIM_DOCUMENTS {
        c.client
            .submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, i as u8));
    }
    assert_eq!(
        c.client
            .try_submit_claim_document(&c.owner, &c.pet_id, &CLAIM, &digest(&c.env, 200))
            .unwrap_err()
            .unwrap(),
        err(ContractError::TooManyItems)
    );
}
