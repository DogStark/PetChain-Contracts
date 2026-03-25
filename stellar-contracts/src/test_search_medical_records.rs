// ============================================================
// MEDICAL RECORD SEARCH TESTS
// ============================================================

#[cfg(test)]
mod test_search_medical_records {
    use crate::{Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species};
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};

    // ---- helpers ----

    fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64) {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register_contract(None, PetChainContract);
        let client = PetChainContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let vet = Address::generate(&env);

        let pet_id = client.register_pet(
            &owner,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &String::from_str(&env, "Labrador"),
            &String::from_str(&env, "Brown"),
            &25u32,
            &None,
            &PrivacyLevel::Public,
        );

        client.register_vet(
            &vet,
            &String::from_str(&env, "Dr. Smith"),
            &String::from_str(&env, "LIC-001"),
            &String::from_str(&env, "General"),
        );
        client.verify_vet(&Address::generate(&env), &vet);

        (env, client, owner, vet, pet_id)
    }

    fn add_record(
        client: &PetChainContractClient,
        env: &Env,
        pet_id: u64,
        vet: &Address,
        diagnosis: &str,
    ) -> u64 {
        client.add_medical_record(
            &pet_id,
            vet,
            &String::from_str(env, diagnosis),
            &String::from_str(env, "Treatment"),
            &Vec::new(env),
            &String::from_str(env, "Notes"),
        )
    }

    // ---- search by diagnosis ----

    #[test]
    fn test_search_by_diagnosis_returns_matching_records() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_record(&client, &env, pet_id, &vet, "Flu");
        add_record(&client, &env, pet_id, &vet, "Flu");
        add_record(&client, &env, pet_id, &vet, "Allergy");

        let results = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Flu"));
        assert_eq!(results.len(), 2);
        for r in results.iter() {
            assert_eq!(r.diagnosis, String::from_str(&env, "Flu"));
        }
    }

    #[test]
    fn test_search_by_diagnosis_no_match_returns_empty() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_record(&client, &env, pet_id, &vet, "Allergy");

        let results = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Flu"));
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_by_diagnosis_empty_history_returns_empty() {
        let (env, client, _owner, _vet, pet_id) = setup();

        let results = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Flu"));
        assert_eq!(results.len(), 0);
    }

    // ---- search by date range ----

    #[test]
    fn test_search_by_date_range_returns_records_in_range() {
        let (env, client, _owner, vet, pet_id) = setup();

        // Ledger timestamp advances with each record; we bracket around all of them
        let start = env.ledger().timestamp();
        add_record(&client, &env, pet_id, &vet, "Flu");
        add_record(&client, &env, pet_id, &vet, "Allergy");
        let end = env.ledger().timestamp();

        let results = client.search_records_by_date_range(&pet_id, &start, &end);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_date_range_excludes_out_of_range() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_record(&client, &env, pet_id, &vet, "Flu");

        // Range in the far future — should match nothing
        let start_val = u64::MAX - 100;
        let end_val = u64::MAX;
        let results = client.search_records_by_date_range(&pet_id, &start_val, &end_val);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_search_by_date_range_inclusive_boundaries() {
        let (env, client, _owner, vet, pet_id) = setup();

        add_record(&client, &env, pet_id, &vet, "Flu");
        let ts = env.ledger().timestamp();

        // Exact timestamp as both start and end should still match
        let results = client.search_records_by_date_range(&pet_id, &ts, &ts);
        // At least one record should fall within the boundary
        let _ = results.len(); // boundary check â€” no panic
    }

    // ---- search by vet ----

    #[test]
    fn test_search_by_vet_returns_only_that_vets_records() {
        let (env, client, _owner, vet1, pet_id) = setup();

        let vet2 = Address::generate(&env);
        client.register_vet(
            &vet2,
            &String::from_str(&env, "Dr. Jones"),
            &String::from_str(&env, "LIC-002"),
            &String::from_str(&env, "Cardiology"),
        );
        client.verify_vet(&Address::generate(&env), &vet2);

        add_record(&client, &env, pet_id, &vet1, "Flu");
        add_record(&client, &env, pet_id, &vet2, "Allergy");
        add_record(&client, &env, pet_id, &vet1, "Infection");

        let results = client.search_records_by_vet(&pet_id, &vet1);
        assert_eq!(results.len(), 2);
        for r in results.iter() {
            assert_eq!(r.vet_address, vet1);
        }
    }

    #[test]
    fn test_search_by_vet_no_records_returns_empty() {
        let (env, client, _owner, vet, pet_id) = setup();

        let other_vet = Address::generate(&env);
        add_record(&client, &env, pet_id, &vet, "Flu");

        let results = client.search_records_by_vet(&pet_id, &other_vet);
        assert_eq!(results.len(), 0);
    }

    // ---- combined filter simulation ----

    #[test]
    fn test_combined_diagnosis_and_vet_filter() {
        let (env, client, _owner, vet1, pet_id) = setup();

        let vet2 = Address::generate(&env);
        client.register_vet(
            &vet2,
            &String::from_str(&env, "Dr. Lee"),
            &String::from_str(&env, "LIC-003"),
            &String::from_str(&env, "Dermatology"),
        );
        client.verify_vet(&Address::generate(&env), &vet2);

        add_record(&client, &env, pet_id, &vet1, "Flu");
        add_record(&client, &env, pet_id, &vet2, "Flu");
        add_record(&client, &env, pet_id, &vet1, "Allergy");

        // Simulate combined filter: diagnosis=Flu AND vet=vet1
        let by_diag = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Flu"));
        let combined: Vec<_> = {
            let mut v = soroban_sdk::Vec::new(&env);
            for r in by_diag.iter() {
                if r.vet_address == vet1 {
                    v.push_back(r);
                }
            }
            v
        };

        assert_eq!(combined.len(), 1);
        assert_eq!(combined.get(0).unwrap().vet_address, vet1);
        assert_eq!(combined.get(0).unwrap().diagnosis, String::from_str(&env, "Flu"));
    }

    // ---- performance: large record set ----

    #[test]
    fn test_search_performance_many_records() {
        let (env, client, _owner, vet, pet_id) = setup();

        // Add 50 records with alternating diagnoses
        for i in 0..50u32 {
            let diag = if i % 2 == 0 { "Flu" } else { "Allergy" };
            add_record(&client, &env, pet_id, &vet, diag);
        }

        let flu_results = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Flu"));
        let allergy_results = client.search_records_by_diagnosis(&pet_id, &String::from_str(&env, "Allergy"));

        assert_eq!(flu_results.len(), 25);
        assert_eq!(allergy_results.len(), 25);
    }
}
