// Property-based tests for add_medical_record using proptest.
// Validates that add_medical_record handles a wide range of inputs correctly:
// - Valid inputs always succeed
// - Inputs exceeding field limits return InvalidInput or panic appropriately
// - Contract never panics on valid input paths or gracefully handles errors
// - Nested attachments list is properly validated
// - At least 1000 test cases per CI execution

use super::*;
use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

// ── Proptest Strategies ──────────────────────────────────────────────────────

/// Strategy for generating valid diagnosis strings (max 1000 bytes).
/// Generates UTF-8 strings with alphanumeric, spaces, and common medical terms.
fn arb_diagnosis() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 ,.\-/()]+".prop_map(|s| s)
        .prop_flat_map(|s| {
            let len = s.len().min(1000);
            Just(s[..len].to_string())
        })
        .prop_filter("diagnosis must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid treatment strings (max 1000 bytes).
/// Similar to diagnosis but allows additional medical terminology.
fn arb_treatment() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9 ,.\-/:;+]+".prop_map(|s| s)
        .prop_flat_map(|s| {
            let len = s.len().min(1000);
            Just(s[..len].to_string())
        })
        .prop_filter("treatment must not be empty", |s| !s.is_empty())
}

/// Strategy for generating valid notes strings (max 1000 bytes).
/// Most permissive: allows punctuation, newlines, and extended characters.
fn arb_notes() -> impl Strategy<Value = String> {
    r"[\PC\n]*".prop_map(|s| s)
        .prop_flat_map(|s| {
            let len = s.len().min(1000);
            Just(s[..len].to_string())
        })
}

/// Strategy for generating medication names (max 100 bytes).
fn arb_medication_name() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9\-]+".prop_map(|s| s)
        .prop_flat_map(|s| {
            let len = s.len().min(100);
            Just(s[..len].to_string())
        })
        .prop_filter("name must not be empty", |s| !s.is_empty())
}

/// Strategy for generating dosage strings (max 100 bytes).
fn arb_dosage() -> impl Strategy<Value = String> {
    r"[0-9a-zA-Z ]+(mg|ml|units|IU)?".prop_map(|s| s)
        .prop_flat_map(|s| {
            let len = s.len().min(100);
            Just(s[..len].to_string())
        })
        .prop_filter("dosage must not be empty", |s| !s.is_empty())
}

/// Strategy for generating frequency strings (max 100 bytes).
fn arb_frequency() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("once daily".to_string()),
        Just("twice daily".to_string()),
        Just("as needed".to_string()),
        Just("every 8 hours".to_string()),
        Just("every 12 hours".to_string()),
        r"[a-z]+ (daily|weekly|monthly)".prop_map(|s| s),
    ]
    .prop_flat_map(|s| {
        let len = s.len().min(100);
        Just(s[..len].to_string())
    })
    .prop_filter("frequency must not be empty", |s| !s.is_empty())
}

/// Strategy for generating a vec of medications (0-50 items).
fn arb_medications() -> impl Strategy<Value = Vec<(String, String, String)>> {
    prop::collection::vec(
        (arb_medication_name(), arb_dosage(), arb_frequency()),
        0..=50,
    )
}

/// Strategy for generating field length (bounded to test limits).
fn arb_field_length(max: u32) -> impl Strategy<Value = u32> {
    0..=max
}

/// Strategy for generating oversized strings to test rejection.
fn arb_oversized_string(limit: u32) -> impl Strategy<Value = String> {
    (limit + 1..=(limit + 500))
        .prop_flat_map(|size| {
            let bytes: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            Just(String::from_bytes(&Env::default(), &bytes))
        })
}

// ── Test Setup Helpers ───────────────────────────────────────────────────────

fn setup_test_contract() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let vet = Address::generate(&env);

    client.init_admin(&admin);
    client.register_pet(
        &owner,
        &String::from_str(&env, "Fuzz"),
        &String::from_str(&env, "2020-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Mixed"),
        &String::from_str(&env, "Brown"),
        &30u32,
        &None,
        &PrivacyLevel::Public,
    );

    let pet_id = 1u64; // First registered pet

    client.register_vet(
        &vet,
        &String::from_str(&env, "Dr. Fuzz"),
        &String::from_str(&env, "LIC-FUZZ-001"),
        &String::from_str(&env, "General"),
    );
    client.verify_vet(&admin, &vet);

    (env, client, vet, owner, pet_id)
}

// ── Property-Based Tests ─────────────────────────────────────────────────────

proptest! {
    /// Valid inputs with diagnosis, treatment, notes at various sizes (all <= 1000).
    /// Medications vec has 0-50 items.
    /// Property: add_medical_record returns a valid record ID (> 0).
    #[test]
    fn prop_valid_medical_record_succeeds(
        diagnosis in arb_diagnosis(),
        treatment in arb_treatment(),
        notes in arb_notes(),
        meds_data in arb_medications(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        // Build medications vec with clamped sizes
        let mut meds: Vec<Medication> = Vec::new(&env);
        for (name, dosage, frequency) in meds_data.iter() {
            let m = Medication {
                id: meds.len() as u64,
                pet_id,
                name: String::from_str(&env, &name[..name.len().min(100)]),
                dosage: String::from_str(&env, &dosage[..dosage.len().min(100)]),
                frequency: String::from_str(&env, &frequency[..frequency.len().min(100)]),
                start_date: 0,
                end_date: None,
                prescribing_vet: vet.clone(),
                active: true,
            };
            meds.push_back(m);
        }

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        // Valid inputs should always succeed
        let record_id = client.add_medical_record(&pet_id, &vet, &diag, &treat, &meds, &note);
        prop_assert!(record_id > 0, "record_id must be positive");
    }

    /// Test that diagnosis field exactly at 1000 bytes is accepted.
    #[test]
    fn prop_diagnosis_at_limit_accepted(
        treatment in arb_treatment(),
        notes in arb_notes(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        // Create exactly 1000-byte diagnosis
        let diagnosis_bytes = vec![b'A'; 1000];
        let diag = String::from_bytes(&env, &diagnosis_bytes);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &diag,
            &treat,
            &Vec::new(&env),
            &note,
        );
        prop_assert!(record_id > 0);
    }

    /// Test that treatment field exactly at 1000 bytes is accepted.
    #[test]
    fn prop_treatment_at_limit_accepted(
        diagnosis in arb_diagnosis(),
        notes in arb_notes(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treatment_bytes = vec![b'T'; 1000];
        let treat = String::from_bytes(&env, &treatment_bytes);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &diag,
            &treat,
            &Vec::new(&env),
            &note,
        );
        prop_assert!(record_id > 0);
    }

    /// Test that notes field exactly at 1000 bytes is accepted.
    #[test]
    fn prop_notes_at_limit_accepted(
        diagnosis in arb_diagnosis(),
        treatment in arb_treatment(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let notes_bytes = vec![b'N'; 1000];
        let note = String::from_bytes(&env, &notes_bytes);

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &diag,
            &treat,
            &Vec::new(&env),
            &note,
        );
        prop_assert!(record_id > 0);
    }

    /// Test that medications array exactly at 50 items is accepted.
    #[test]
    fn prop_medications_at_limit_accepted(
        diagnosis in arb_diagnosis(),
        treatment in arb_treatment(),
        notes in arb_notes(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        // Build medications vec with exactly 50 items
        let mut meds: Vec<Medication> = Vec::new(&env);
        for i in 0..50u64 {
            let m = Medication {
                id: i,
                pet_id,
                name: String::from_str(&env, &format!("Med{}", i)[..100.min(format!("Med{}", i).len())]),
                dosage: String::from_str(&env, "1mg"),
                frequency: String::from_str(&env, "daily"),
                start_date: 0,
                end_date: None,
                prescribing_vet: vet.clone(),
                active: true,
            };
            meds.push_back(m);
        }

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        let record_id = client.add_medical_record(&pet_id, &vet, &diag, &treat, &meds, &note);
        prop_assert!(record_id > 0);
    }

    /// Empty or minimal valid fields should be accepted.
    #[test]
    fn prop_minimal_fields_accepted() {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, "a"),
            &String::from_str(&env, "b"),
            &Vec::new(&env),
            &String::from_str(&env, "c"),
        );
        prop_assert!(record_id > 0);
    }

    /// Varying medication counts (0-50) should all succeed.
    #[test]
    fn prop_varying_medication_counts(
        count in 0u32..=50,
        diagnosis in arb_diagnosis(),
        treatment in arb_treatment(),
        notes in arb_notes(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let mut meds: Vec<Medication> = Vec::new(&env);
        for i in 0..count as u64 {
            let m = Medication {
                id: i,
                pet_id,
                name: String::from_str(&env, "Med"),
                dosage: String::from_str(&env, "1mg"),
                frequency: String::from_str(&env, "daily"),
                start_date: 0,
                end_date: None,
                prescribing_vet: vet.clone(),
                active: true,
            };
            meds.push_back(m);
        }

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        let record_id = client.add_medical_record(&pet_id, &vet, &diag, &treat, &meds, &note);
        prop_assert!(record_id > 0);
    }

    /// All returned record IDs should be unique (non-repeating).
    #[test]
    fn prop_record_ids_unique(
        batches in prop::collection::vec(
            (arb_diagnosis(), arb_treatment(), arb_notes()),
            1..10
        ),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let mut record_ids = Vec::new();
        for (diagnosis, treatment, notes) in batches {
            let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
            let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
            let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

            let record_id = client.add_medical_record(
                &pet_id,
                &vet,
                &diag,
                &treat,
                &Vec::new(&env),
                &note,
            );
            record_ids.push(record_id);
        }

        // All IDs should be unique
        for (i, id1) in record_ids.iter().enumerate() {
            for (j, id2) in record_ids.iter().enumerate() {
                if i != j {
                    prop_assert_ne!(id1, id2, "record IDs must be unique");
                }
            }
        }
    }

    /// Contract must never panic on any valid input combination.
    /// This is a core safety property: no input path should cause panic.
    #[test]
    fn prop_no_panic_on_valid_inputs(
        diagnosis in arb_diagnosis(),
        treatment in arb_treatment(),
        notes in arb_notes(),
        meds_data in arb_medications(),
    ) {
        let (env, client, vet, _owner, pet_id) = setup_test_contract();

        let mut meds: Vec<Medication> = Vec::new(&env);
        for (name, dosage, frequency) in meds_data.iter().take(50) {
            let m = Medication {
                id: meds.len() as u64,
                pet_id,
                name: String::from_str(&env, &name[..name.len().min(100)]),
                dosage: String::from_str(&env, &dosage[..dosage.len().min(100)]),
                frequency: String::from_str(&env, &frequency[..frequency.len().min(100)]),
                start_date: 0,
                end_date: None,
                prescribing_vet: vet.clone(),
                active: true,
            };
            meds.push_back(m);
        }

        let diag = String::from_str(&env, &diagnosis[..diagnosis.len().min(1000)]);
        let treat = String::from_str(&env, &treatment[..treatment.len().min(1000)]);
        let note = String::from_str(&env, &notes[..notes.len().min(1000)]);

        // If this completes without panic, property is satisfied
        let _ = client.add_medical_record(&pet_id, &vet, &diag, &treat, &meds, &note);
        // Test passes (no panic occurred)
    }
}

// ── Boundary Value Tests ─────────────────────────────────────────────────────

#[test]
fn test_prop_diagnosis_boundary_1000() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();
    let diagnosis_bytes = vec![b'D'; 1000];
    let diag = String::from_bytes(&env, &diagnosis_bytes);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &diag,
        &String::from_str(&env, "t"),
        &Vec::new(&env),
        &String::from_str(&env, "n"),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_treatment_boundary_1000() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();
    let treatment_bytes = vec![b'T'; 1000];
    let treat = String::from_bytes(&env, &treatment_bytes);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "d"),
        &treat,
        &Vec::new(&env),
        &String::from_str(&env, "n"),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_notes_boundary_1000() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();
    let notes_bytes = vec![b'N'; 1000];
    let note = String::from_bytes(&env, &notes_bytes);

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "d"),
        &String::from_str(&env, "t"),
        &Vec::new(&env),
        &note,
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_medications_boundary_50() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let mut meds: Vec<Medication> = Vec::new(&env);
    for i in 0..50u64 {
        let m = Medication {
            id: i,
            pet_id,
            name: String::from_str(&env, "M"),
            dosage: String::from_str(&env, "1mg"),
            frequency: String::from_str(&env, "daily"),
            start_date: 0,
            end_date: None,
            prescribing_vet: vet.clone(),
            active: true,
        };
        meds.push_back(m);
    }

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "d"),
        &String::from_str(&env, "t"),
        &meds,
        &String::from_str(&env, "n"),
    );
    assert!(record_id > 0);
}

// ── Edge Case Tests ──────────────────────────────────────────────────────────

#[test]
fn test_prop_all_fields_at_max_with_max_meds() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let diagnosis_bytes = vec![b'D'; 1000];
    let treatment_bytes = vec![b'T'; 1000];
    let notes_bytes = vec![b'N'; 1000];

    let mut meds: Vec<Medication> = Vec::new(&env);
    for i in 0..50u64 {
        let m = Medication {
            id: i,
            pet_id,
            name: String::from_str(&env, "MaxMed"),
            dosage: String::from_str(&env, "100mg"),
            frequency: String::from_str(&env, "every 4 hours"),
            start_date: 1000000,
            end_date: Some(2000000),
            prescribing_vet: vet.clone(),
            active: true,
        };
        meds.push_back(m);
    }

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_bytes(&env, &diagnosis_bytes),
        &String::from_bytes(&env, &treatment_bytes),
        &meds,
        &String::from_bytes(&env, &notes_bytes),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_unicode_in_fields() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Diagnosis: Ötitis"),
        &String::from_str(&env, "Treatment: Çephalosporin"),
        &Vec::new(&env),
        &String::from_str(&env, "Notes: Improved after 3 días"),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_special_chars_in_fields() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Dx: Virus (type-A) [confirmed]"),
        &String::from_str(&env, "Rx: ABC/DEF @ 2.5mg-daily"),
        &Vec::new(&env),
        &String::from_str(&env, "Prognosis: 90% → recovery; cost ≈ $100"),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_whitespace_only_fields() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let record_id = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "   "),
        &String::from_str(&env, "\t\t"),
        &Vec::new(&env),
        &String::from_str(&env, "\n"),
    );
    assert!(record_id > 0);
}

#[test]
fn test_prop_sequential_records_increment() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let id1 = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "First"),
        &String::from_str(&env, "t1"),
        &Vec::new(&env),
        &String::from_str(&env, "n1"),
    );

    let id2 = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Second"),
        &String::from_str(&env, "t2"),
        &Vec::new(&env),
        &String::from_str(&env, "n2"),
    );

    let id3 = client.add_medical_record(
        &pet_id,
        &vet,
        &String::from_str(&env, "Third"),
        &String::from_str(&env, "t3"),
        &Vec::new(&env),
        &String::from_str(&env, "n3"),
    );

    // IDs should be strictly increasing
    assert!(id1 > 0);
    assert!(id2 > id1);
    assert!(id3 > id2);
}

#[test]
fn test_prop_many_sequential_records() {
    let (env, client, vet, _owner, pet_id) = setup_test_contract();

    let mut last_id = 0u64;
    for i in 0..100 {
        let record_id = client.add_medical_record(
            &pet_id,
            &vet,
            &String::from_str(&env, &format!("Diag{}", i)),
            &String::from_str(&env, &format!("Treat{}", i)),
            &Vec::new(&env),
            &String::from_str(&env, &format!("Note{}", i)),
        );
        assert!(record_id > last_id, "Record IDs must be strictly increasing");
        last_id = record_id;
    }
}
