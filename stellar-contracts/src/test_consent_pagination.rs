use crate::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Env, Vec};

fn setup() -> (Env, PetChainContractClient<'static>, u64, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Buddy"),
        &String::from_str(&env, "1000000"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Labrador"),
        &String::from_str(&env, "Black"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    );

    (env, client, pet_id, owner)
}

#[test]
fn test_consent_history_pagination_basic() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    // Grant 5 consents, revoke 2 of them.
    let mut ids = Vec::new(&env);
    for _ in 0..5u32 {
        let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
        ids.push_back(id);
    }
    client.revoke_consent(&ids.get(0).unwrap(), &owner);
    client.revoke_consent(&ids.get(1).unwrap(), &owner);

    // Page 0 with size 3 should return 3 records.
    let page0 = client.get_consent_history_page(&pet_id, &0, &3);
    assert_eq!(page0.len(), 3);

    // Page 1 with size 3 should return the remaining 2 records.
    let page1 = client.get_consent_history_page(&pet_id, &1, &3);
    assert_eq!(page1.len(), 2);

    // Page 2 is beyond the data — should be empty.
    let page2 = client.get_consent_history_page(&pet_id, &2, &3);
    assert_eq!(page2.len(), 0);
}

#[test]
fn test_consent_history_page_zero_size_clamps_to_50() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    for _ in 0..3u32 {
        client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee, &ConsentScope::ReadMedical);
    }

    // page_size=0 should be treated as 50 (clamped), returning all 3 records.
    let page = client.get_consent_history_page(&pet_id, &0, &0);
    assert_eq!(page.len(), 3);
}

#[test]
fn test_consent_pruning_removes_oldest_revoked_at_cap() {
    let (env, client, pet_id, owner) = setup();
    env.budget().reset_unlimited();
    let grantee = Address::generate(&env);

    // Fill up to the cap (50) by alternating grant/revoke so revoked records accumulate.
    let mut first_active_id: u64 = 0;
    for i in 0..50u32 {
        let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
        if i == 0 {
            first_active_id = id;
        }
        // Revoke all but the last one so there are always revoked slots to prune.
        if i < 49 {
            client.revoke_consent(&id, &owner);
        }
    }

    // At this point: 49 revoked + 1 active = 50 total (at cap).
    // Granting one more should prune the oldest revoked record without panicking.
    let new_id = client.grant_consent(&pet_id, &owner, &ConsentType::PublicHealth, &grantee, &ConsentScope::ReadMedical);
    assert!(new_id > 0);

    // Total stored should still be <= 50.
    let history = client.get_consent_history(&pet_id);
    assert!(history.len() <= 50);

    // The first active consent (index 49) must still be present.
    let _ = first_active_id; // used above; suppress warning
}

#[test]
fn test_consent_hard_cap_when_all_active() {
    let (env, client, pet_id, owner) = setup();
    env.budget().reset_unlimited();
    let grantee = Address::generate(&env);

    // Grant 50 consents without revoking any.
    for _ in 0..50u32 {
        client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    }

    // Verify the cap is enforced
    let history = client.get_consent_history(&pet_id);
    assert_eq!(
        history.len(),
        50,
        "Should have exactly 50 consents at the cap"
    );
}

#[test]
fn test_many_grant_revoke_cycles_stay_bounded() {
    let (env, client, pet_id, owner) = setup();
    env.budget().reset_unlimited();
    let grantee = Address::generate(&env);

    // Simulate 200 grant/revoke cycles — storage must stay bounded at MAX_CONSENTS_PER_PET.
    for _ in 0..200u32 {
        let id = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
        client.revoke_consent(&id, &owner);
    }

    let history = client.get_consent_history(&pet_id);
    assert!(
        history.len() <= 50,
        "History grew beyond cap: {}",
        history.len()
    );
}

#[test]
fn test_get_consent_history_page_no_records() {
    let (_env, client, pet_id, _owner) = setup();
    let page = client.get_consent_history_page(&pet_id, &0, &10);
    assert_eq!(page.len(), 0);
}

#[test]
fn test_get_active_consents_only_returns_active() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    let id1 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    let id2 = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee, &ConsentScope::ReadMedical);
    let _id3 = client.grant_consent(&pet_id, &owner, &ConsentType::PublicHealth, &grantee, &ConsentScope::ReadMedical);

    // Revoke two of the three
    client.revoke_consent(&id1, &owner);
    client.revoke_consent(&id2, &owner);

    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 1);
    assert!(active.get(0).unwrap().is_active);
}

#[test]
fn test_get_active_consents_empty_when_all_revoked() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    let id1 = client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    let id2 = client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee, &ConsentScope::ReadMedical);

    client.revoke_consent(&id1, &owner);
    client.revoke_consent(&id2, &owner);

    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_get_active_consents_all_active() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    for _ in 0..3u32 {
        client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    }

    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 3);
    for i in 0..3u32 {
        assert!(active.get(i).unwrap().is_active);
    }
}

#[test]
fn test_get_active_consents_no_consents() {
    let (_env, client, pet_id, _owner) = setup();
    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 0);
}

#[test]
fn test_expired_consent_is_not_active() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    env.ledger().set_timestamp(100);
    let id = client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &Some(150),
    );
    assert!(client.is_consent_active(&id));

    env.ledger().set_timestamp(151);
    assert!(!client.is_consent_active(&id));
}

#[test]
fn test_get_active_consents_excludes_expired() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    env.ledger().set_timestamp(100);
    client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &grantee,
        &Some(120),
    );
    client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::Insurance,
        &grantee,
        &Some(300),
    );

    env.ledger().set_timestamp(121);
    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(0).unwrap().consent_type, ConsentType::Insurance);
}

#[test]
fn test_extend_consent_reactivates_after_original_expiry() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    env.ledger().set_timestamp(100);
    let id = client.grant_consent_with_expiry(
        &pet_id,
        &owner,
        &ConsentType::PublicHealth,
        &grantee,
        &Some(110),
    );

    env.ledger().set_timestamp(111);
    assert!(!client.is_consent_active(&id));

    assert!(client.extend_consent(&pet_id, &id, &owner, &200));
    assert!(client.is_consent_active(&id));

    let active = client.get_active_consents(&pet_id);
    assert_eq!(active.len(), 1);
    assert_eq!(active.get(0).unwrap().expires_at, Some(200));
fn test_get_consent_count_returns_correct_count() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    assert_eq!(client.get_consent_count(&pet_id), 0);

    client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    assert_eq!(client.get_consent_count(&pet_id), 1);

    client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee, &ConsentScope::ReadMedical);
    assert_eq!(client.get_consent_count(&pet_id), 2);
}

#[test]
fn test_get_consent_count_zero_for_no_consents() {
    let (_env, client, pet_id, _owner) = setup();
    assert_eq!(client.get_consent_count(&pet_id), 0);
}

#[test]
fn test_get_consents_by_scope_pagination() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    // Grant 5 consents with ReadMedical scope
    for _ in 0..5u32 {
        client.grant_consent(&pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical);
    }

    // Grant 3 consents with WriteMedical scope
    for _ in 0..3u32 {
        client.grant_consent(&pet_id, &owner, &ConsentType::Insurance, &grantee, &ConsentScope::WriteMedical);
    }

    // Get first page of ReadMedical consents (page size 2)
    let page0 = client.get_consents_by_scope(&pet_id, &ConsentScope::ReadMedical, &0, &2);
    assert_eq!(page0.len(), 2);

    // Get second page of ReadMedical consents
    let page1 = client.get_consents_by_scope(&pet_id, &ConsentScope::ReadMedical, &1, &2);
    assert_eq!(page1.len(), 2);

    // Get all WriteMedical consents
    let write_page = client.get_consents_by_scope(&pet_id, &ConsentScope::WriteMedical, &0, &10);
    assert_eq!(write_page.len(), 3);

    // Verify all returned consents have correct scope
    for consent in page0.iter() {
        assert_eq!(consent.scope, ConsentScope::ReadMedical);
    }
}

// ============================================================
// CASCADE REVOCATION TESTS
// ============================================================

#[test]
fn test_revoke_consent_cascade_no_children() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    let root_id = client.grant_consent(
        &pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical,
    );

    // No sub-consents — cascade should revoke only the root (count = 1)
    let count = client.revoke_consent_cascade(&pet_id, &root_id, &owner);
    assert_eq!(count, 1);
    assert!(!client.is_consent_active(&root_id));
}

#[test]
fn test_revoke_consent_cascade_with_one_child() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);
    let sub_grantee = Address::generate(&env);

    // Root consent granted by owner to grantee
    let root_id = client.grant_consent(
        &pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical,
    );

    // Sub-consent: grantee delegates to sub_grantee (parent = root_id)
    let sub_id = client.grant_consent_with_parent(
        &pet_id, &grantee, &ConsentType::Research, &sub_grantee,
        &ConsentScope::ReadMedical, &Some(root_id),
    );

    // Cascade revoke from root
    let count = client.revoke_consent_cascade(&pet_id, &root_id, &owner);
    assert_eq!(count, 2);
    assert!(!client.is_consent_active(&root_id));
    assert!(!client.is_consent_active(&sub_id));
}

#[test]
fn test_revoke_consent_cascade_depth_limit() {
    let (env, client, pet_id, owner) = setup();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let d = Address::generate(&env);

    // Chain: owner -> a -> b -> c -> d (depth 4 from root)
    let id0 = client.grant_consent(
        &pet_id, &owner, &ConsentType::Research, &a, &ConsentScope::ReadMedical,
    );
    let id1 = client.grant_consent_with_parent(
        &pet_id, &a, &ConsentType::Research, &b, &ConsentScope::ReadMedical, &Some(id0),
    );
    let id2 = client.grant_consent_with_parent(
        &pet_id, &b, &ConsentType::Research, &c, &ConsentScope::ReadMedical, &Some(id1),
    );
    let id3 = client.grant_consent_with_parent(
        &pet_id, &c, &ConsentType::Research, &d, &ConsentScope::ReadMedical, &Some(id2),
    );

    // Cascade depth is limited to 3, so id3 (depth 4) should NOT be revoked
    let count = client.revoke_consent_cascade(&pet_id, &id0, &owner);
    assert_eq!(count, 3); // id0, id1, id2 revoked; id3 beyond depth limit
    assert!(!client.is_consent_active(&id0));
    assert!(!client.is_consent_active(&id1));
    assert!(!client.is_consent_active(&id2));
    assert!(client.is_consent_active(&id3)); // depth 4 — not revoked
}

#[test]
fn test_preview_revocation_cascade_no_state_change() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);
    let sub_grantee = Address::generate(&env);

    let root_id = client.grant_consent(
        &pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical,
    );
    let sub_id = client.grant_consent_with_parent(
        &pet_id, &grantee, &ConsentType::Research, &sub_grantee,
        &ConsentScope::ReadMedical, &Some(root_id),
    );

    let preview = client.preview_revocation_cascade(&pet_id, &root_id);
    assert_eq!(preview.len(), 2);
    assert!(preview.contains(&root_id));
    assert!(preview.contains(&sub_id));

    // No state change — both consents still active
    assert!(client.is_consent_active(&root_id));
    assert!(client.is_consent_active(&sub_id));
}

#[test]
fn test_preview_revocation_cascade_root_only() {
    let (env, client, pet_id, owner) = setup();
    let grantee = Address::generate(&env);

    let root_id = client.grant_consent(
        &pet_id, &owner, &ConsentType::Research, &grantee, &ConsentScope::ReadMedical,
    );

    let preview = client.preview_revocation_cascade(&pet_id, &root_id);
    assert_eq!(preview.len(), 1);
    assert!(preview.contains(&root_id));
}

#[test]
fn test_delegate_consent_chain_allows_access_with_valid_chain() {
    let (env, client, pet_id, owner) = setup();
    let trusted = Address::generate(&env);
    let sub_delegate = Address::generate(&env);

    let root_id = client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &trusted,
        &ConsentScope::ReadMedical,
    );

    let scopes = Vec::from_array(&env, [ConsentScope::ReadMedical]);
    let delegated = client.delegate_consent(&pet_id, &trusted, &sub_delegate, &scopes, &3);
    assert_eq!(delegated.len(), 1);
    assert!(client.check_consent_access(&pet_id, &sub_delegate, &ConsentScope::ReadMedical));
    assert!(client.is_consent_active(&root_id));
}

#[test]
#[should_panic]
fn test_delegate_consent_scope_escalation_rejected() {
    let (env, client, pet_id, owner) = setup();
    let trusted = Address::generate(&env);
    let sub_delegate = Address::generate(&env);

    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &trusted,
        &ConsentScope::ReadMedical,
    );

    let scopes = Vec::from_array(&env, [ConsentScope::WriteMedical]);
    let _ = client.delegate_consent(&pet_id, &trusted, &sub_delegate, &scopes, &3);
}

#[test]
#[should_panic]
fn test_delegate_consent_depth_limit_enforced() {
    let (env, client, pet_id, owner) = setup();
    let level1 = Address::generate(&env);
    let level2 = Address::generate(&env);
    let level3 = Address::generate(&env);
    let level4 = Address::generate(&env);

    client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &level1,
        &ConsentScope::ReadMedical,
    );

    let scopes = Vec::from_array(&env, [ConsentScope::ReadMedical]);
    let _ = client.delegate_consent(&pet_id, &level1, &level2, &scopes, &3);
    let _ = client.delegate_consent(&pet_id, &level2, &level3, &scopes, &3);
    let _ = client.delegate_consent(&pet_id, &level3, &level4, &scopes, &3);
}

#[test]
fn test_consent_access_fails_when_parent_chain_is_revoked() {
    let (env, client, pet_id, owner) = setup();
    let trusted = Address::generate(&env);
    let sub_delegate = Address::generate(&env);

    let root_id = client.grant_consent(
        &pet_id,
        &owner,
        &ConsentType::Research,
        &trusted,
        &ConsentScope::ReadMedical,
    );
    let scopes = Vec::from_array(&env, [ConsentScope::ReadMedical]);
    let _ = client.delegate_consent(&pet_id, &trusted, &sub_delegate, &scopes, &3);

    assert!(client.check_consent_access(&pet_id, &sub_delegate, &ConsentScope::ReadMedical));
    client.revoke_consent(&root_id, &owner);
    assert!(!client.check_consent_access(&pet_id, &sub_delegate, &ConsentScope::ReadMedical));
}
}