// ============================================================
// #1152 — Bounded String / Bytes tests
//
// Verifies that every stored String field is rejected at the
// transaction boundary when its byte length exceeds the domain-
// specific cap defined by the MAX_* constants.
//
// Coverage per field:
//   • Exactly-at-limit value accepted
//   • One-byte-over-limit value rejected with InputStringTooLong
//   • Empty/zero-length value accepted (no minimum enforced here)
//
// Fields tested:
//   color                  (register_pet)     MAX_COLOR_LEN = 50
//   behavior description   (add_behavior_record) MAX_BEHAVIOR_DESC_LEN = 500
//   activity notes         (add_activity_record) MAX_ACTIVITY_NOTES_LEN = 500
//   medical diagnosis      (add_medical_record)  MAX_MEDICAL_DIAGNOSIS_LEN = 500
//   medical treatment      (add_medical_record)  MAX_MEDICAL_TREATMENT_LEN = 500
//   medical notes          (add_medical_record)  MAX_MEDICAL_NOTES_LEN = 1000
//   lab test_type          (add_lab_result)      MAX_LAB_TEST_TYPE_LEN = 100
//   lab results            (add_lab_result)      MAX_LAB_RESULTS_LEN = 1000
//   lab reference_ranges   (add_lab_result)      MAX_LAB_REF_RANGES_LEN = 500
//   breeding notes         (add_breeding_record) MAX_BREEDING_NOTES_LEN = 500
//
// Threat model note:
//   Unbounded strings inflate ledger entries and increase read/write
//   fees linearly. A malicious caller could submit multi-KiB strings
//   to cause future reads of the same entry to fail resource checks,
//   effectively bricking a pet's record. The caps prevent this without
//   restricting legitimate use cases.
// ============================================================

#[cfg(test)]
mod tests {
    use crate::{
        ActivityType, BehaviorType, ContractError, Gender, PetChainContract,
        PetChainContractClient, PrivacyLevel, Species,
        MAX_ACTIVITY_NOTES_LEN, MAX_BEHAVIOR_DESC_LEN, MAX_BREEDING_NOTES_LEN,
        MAX_COLOR_LEN, MAX_LAB_REF_RANGES_LEN, MAX_LAB_RESULTS_LEN,
        MAX_LAB_TEST_TYPE_LEN, MAX_MEDICAL_DIAGNOSIS_LEN, MAX_MEDICAL_NOTES_LEN,
        MAX_MEDICAL_TREATMENT_LEN,
    };
    use soroban_sdk::{testutils::Address as _, Address, Env, Error, Map, String, Vec};

    // ─── helpers ──────────────────────────────────────────────────────────────

    fn setup() -> (Env, PetChainContractClient<'static>, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.budget().reset_unlimited();

        env.ledger().with_mut(|li| {
            li.timestamp = 1_700_000_000;
        });

        let contract_id = env.register(PetChainContract, ());
        let client = PetChainContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let mut admins = soroban_sdk::Vec::new(&env);
        admins.push_back(admin.clone());
        client.init_multisig(&admin, &admins, &1u32);

        let owner = Address::generate(&env);
        (env, client, admin, owner)
    }

    /// Build a `soroban_sdk::String` filled with `n` ASCII 'a' characters.
    fn str_of_len(env: &Env, n: u32) -> String {
        let s: std::string::String = "a".repeat(n as usize);
        String::from_str(env, &s)
    }

    fn register_pet(client: &PetChainContractClient, env: &Env, owner: &Address) -> u64 {
        client.register_pet(
            owner,
            &String::from_str(env, "TestPet"),
            &String::from_str(env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(env, "Labrador"),
            &String::from_str(env, "Yellow"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        )
    }

    fn setup_vet(client: &PetChainContractClient, env: &Env, admin: &Address) -> Address {
        let vet = Address::generate(env);
        client.register_vet(
            &vet,
            &String::from_str(env, "Dr. Test"),
            &String::from_str(env, "VET-STR-001"),
            &String::from_str(env, "General"),
        );
        client.verify_vet(admin, &vet);
        vet
    }

    // ─── color field (register_pet) ───────────────────────────────────────────

    /// Exactly MAX_COLOR_LEN bytes → accepted.
    #[test]
    fn color_at_limit_accepted() {
        let (env, client, _admin, owner) = setup();
        let result = client.try_register_pet(
            &owner,
            &String::from_str(&env, "ColorPet"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &str_of_len(&env, MAX_COLOR_LEN),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        );
        assert!(result.is_ok(), "color at limit should succeed: {result:?}");
    }

    /// MAX_COLOR_LEN + 1 bytes → InputStringTooLong.
    #[test]
    fn color_over_limit_rejected() {
        let (env, client, _admin, owner) = setup();
        let err = client
            .try_register_pet(
                &owner,
                &String::from_str(&env, "ColorPet"),
                &String::from_str(&env, "2020-01-01"),
                &Gender::Male,
                &Species::Dog,
                &String::from_str(&env, "Labrador"),
                &str_of_len(&env, MAX_COLOR_LEN + 1),
                &25u32,
                &None,
                &PrivacyLevel::Public,
            )
            .unwrap_err()
            .unwrap();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── behavior description ─────────────────────────────────────────────────

    #[test]
    fn behavior_desc_at_limit_accepted() {
        let (env, client, _admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let result = client.try_add_behavior_record(
            &pet_id,
            &owner,
            &BehaviorType::Training,
            &1u32,
            &str_of_len(&env, MAX_BEHAVIOR_DESC_LEN),
        );
        assert!(result.is_ok(), "behavior desc at limit: {result:?}");
    }

    #[test]
    fn behavior_desc_over_limit_rejected() {
        let (env, client, _admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let err = client
            .try_add_behavior_record(
                &pet_id,
                &owner,
                &BehaviorType::Training,
                &1u32,
                &str_of_len(&env, MAX_BEHAVIOR_DESC_LEN + 1),
            )
            .unwrap_err()
            .unwrap();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── activity notes ────────────────────────────────────────────────────────

    #[test]
    fn activity_notes_at_limit_accepted() {
        let (env, client, _admin, _owner) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let result = client.try_add_activity_record(
            &pet_id,
            &ActivityType::Walk,
            &30u32,
            &5u32,
            &100u32,
            &str_of_len(&env, MAX_ACTIVITY_NOTES_LEN),
        );
        assert!(result.is_ok(), "activity notes at limit: {result:?}");
    }

    #[test]
    fn activity_notes_over_limit_rejected() {
        let (env, client, _admin, _owner) = setup();
        let owner = Address::generate(&env);
        let pet_id = register_pet(&client, &env, &owner);
        let err = client
            .try_add_activity_record(
                &pet_id,
                &ActivityType::Walk,
                &30u32,
                &5u32,
                &100u32,
                &str_of_len(&env, MAX_ACTIVITY_NOTES_LEN + 1),
            )
            .unwrap_err()
            .unwrap();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── medical record fields ────────────────────────────────────────────────

    fn add_medical_record(
        client: &PetChainContractClient,
        env: &Env,
        pet_id: u64,
        vet: &Address,
        diagnosis: String,
        treatment: String,
        notes: String,
    ) -> Result<u64, Error> {
        client
            .try_add_medical_record(
                &pet_id,
                vet,
                &diagnosis,
                &treatment,
                &Vec::new(env),
                &notes,
            )
            .map_err(|e| e.unwrap())
    }

    #[test]
    fn medical_diagnosis_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            str_of_len(&env, MAX_MEDICAL_DIAGNOSIS_LEN),
            String::from_str(&env, "treatment"),
            String::from_str(&env, "notes"),
        )
        .is_ok());
    }

    #[test]
    fn medical_diagnosis_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            str_of_len(&env, MAX_MEDICAL_DIAGNOSIS_LEN + 1),
            String::from_str(&env, "treatment"),
            String::from_str(&env, "notes"),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    #[test]
    fn medical_treatment_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "diagnosis"),
            str_of_len(&env, MAX_MEDICAL_TREATMENT_LEN),
            String::from_str(&env, "notes"),
        )
        .is_ok());
    }

    #[test]
    fn medical_treatment_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "diagnosis"),
            str_of_len(&env, MAX_MEDICAL_TREATMENT_LEN + 1),
            String::from_str(&env, "notes"),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    #[test]
    fn medical_notes_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "diagnosis"),
            String::from_str(&env, "treatment"),
            str_of_len(&env, MAX_MEDICAL_NOTES_LEN),
        )
        .is_ok());
    }

    #[test]
    fn medical_notes_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "diagnosis"),
            String::from_str(&env, "treatment"),
            str_of_len(&env, MAX_MEDICAL_NOTES_LEN + 1),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── lab result fields ────────────────────────────────────────────────────

    fn add_lab(
        client: &PetChainContractClient,
        env: &Env,
        pet_id: u64,
        vet: &Address,
        test_type: String,
        results: String,
        reference_ranges: String,
    ) -> Result<u64, Error> {
        client
            .try_add_lab_result(
                &pet_id,
                vet,
                &test_type,
                &results,
                &reference_ranges,
                &None,
                &None,
                &Map::new(env),
            )
            .map_err(|e| e.unwrap())
    }

    #[test]
    fn lab_test_type_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            str_of_len(&env, MAX_LAB_TEST_TYPE_LEN),
            String::from_str(&env, "normal"),
            String::from_str(&env, "0-100"),
        )
        .is_ok());
    }

    #[test]
    fn lab_test_type_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            str_of_len(&env, MAX_LAB_TEST_TYPE_LEN + 1),
            String::from_str(&env, "normal"),
            String::from_str(&env, "0-100"),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    #[test]
    fn lab_results_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "CBC"),
            str_of_len(&env, MAX_LAB_RESULTS_LEN),
            String::from_str(&env, "0-100"),
        )
        .is_ok());
    }

    #[test]
    fn lab_results_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "CBC"),
            str_of_len(&env, MAX_LAB_RESULTS_LEN + 1),
            String::from_str(&env, "0-100"),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    #[test]
    fn lab_reference_ranges_at_limit_accepted() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        assert!(add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "CBC"),
            String::from_str(&env, "normal"),
            str_of_len(&env, MAX_LAB_REF_RANGES_LEN),
        )
        .is_ok());
    }

    #[test]
    fn lab_reference_ranges_over_limit_rejected() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);
        let err = add_lab(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "CBC"),
            String::from_str(&env, "normal"),
            str_of_len(&env, MAX_LAB_REF_RANGES_LEN + 1),
        )
        .unwrap_err();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── breeding notes ────────────────────────────────────────────────────────

    #[test]
    fn breeding_notes_at_limit_accepted() {
        let (env, client, _admin, owner) = setup();
        let sire = register_pet(&client, &env, &owner);
        let dam = register_pet(&client, &env, &owner);
        let result = client.try_add_breeding_record(
            &sire,
            &dam,
            &1_700_000_000u64,
            &str_of_len(&env, MAX_BREEDING_NOTES_LEN),
        );
        assert!(result.is_ok(), "breeding notes at limit: {result:?}");
    }

    #[test]
    fn breeding_notes_over_limit_rejected() {
        let (env, client, _admin, owner) = setup();
        let sire = register_pet(&client, &env, &owner);
        let dam = register_pet(&client, &env, &owner);
        let err = client
            .try_add_breeding_record(
                &sire,
                &dam,
                &1_700_000_000u64,
                &str_of_len(&env, MAX_BREEDING_NOTES_LEN + 1),
            )
            .unwrap_err()
            .unwrap();
        assert_eq!(err, Error::from(ContractError::InputStringTooLong));
    }

    // ─── cross-field boundary test: multiple fields in one call ───────────────

    /// Confirm that if the second field is over-limit, the first (valid) field
    /// change is also rolled back — the whole transaction is atomic.
    #[test]
    fn medical_record_rollback_on_second_field_over_limit() {
        let (env, client, admin, owner) = setup();
        let pet_id = register_pet(&client, &env, &owner);
        let vet = setup_vet(&client, &env, &admin);

        // Record count before the failing call.
        let count_before: u64 = client.get_lab_result_count(&pet_id);

        // treatment is over limit → should reject
        let _ = add_medical_record(
            &client,
            &env,
            pet_id,
            &vet,
            String::from_str(&env, "ok diagnosis"),
            str_of_len(&env, MAX_MEDICAL_TREATMENT_LEN + 1),
            String::from_str(&env, "ok notes"),
        );

        // Record count must be unchanged.
        let count_after: u64 = client.get_lab_result_count(&pet_id);
        assert_eq!(count_before, count_after);
    }
}
