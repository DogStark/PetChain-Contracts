extern crate std;

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    xdr::{Limits, WriteXdr},
};
use std::path::Path;

fn storage_bytes(env: &Env) -> u64 {
    env.to_ledger_snapshot()
        .ledger_entries
        .iter()
        .map(|(_, (entry, _))| entry.to_xdr(Limits::none()).unwrap().len() as u64)
        .sum()
}

fn measure(env: &Env, name: &str, run: impl FnOnce()) {
    let before = storage_bytes(env);
    env.budget().reset_tracker();
    run();
    let instructions = env.budget().cpu_instruction_cost();
    let memory = env.budget().memory_bytes_cost();
    let storage = storage_bytes(env).saturating_sub(before);
    let measurements = [
        ("instructions", instructions),
        ("memory", memory),
        ("storage", storage),
    ];
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../contracts/test/budgets");
    let path = directory.join(std::format!("{name}.json"));

    if std::env::var("UPDATE_RESOURCE_BUDGET_BASELINE").as_deref() == Ok("1") {
        std::fs::create_dir_all(&directory).unwrap();
        let snapshot = serde_json::json!({
            "instructions": instructions,
            "memory": memory,
            "storage": storage,
        });
        std::fs::write(path, std::format!("{snapshot:#}\n")).unwrap();
        return;
    }

    let contents = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("[budget] {name}: {}: {error}", path.display()));
    let baseline: serde_json::Value = serde_json::from_str(&contents).unwrap();
    for (resource, measured) in measurements {
        let limit = baseline[resource].as_u64().unwrap();
        assert!(
            measured <= limit,
            "[budget] {name}: {resource} regressed (measured {measured} > budget {limit})"
        );
    }
}

#[test]
fn resource_budget_workflows() {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();
    let id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &id);
    let owner = Address::generate(&env);
    client.init_admin(&owner);
    let string = |text| String::from_str(&env, text);
    let mut pet = 0;
    measure(&env, "registration", || {
        pet = client.register_pet(
            &owner,
            &string(&"A".repeat(64)),
            &string("2020-01-01"),
            &Gender::Male,
            &Species::Dog,
            &string("Labrador"),
            &string(&"B".repeat(MAX_COLOR_LEN as usize)),
            &10,
            &Some(string(&"1".repeat(MAX_MICROCHIP_ID_LEN))),
            &PrivacyLevel::Public,
        );
    });

    let vet = Address::generate(&env);
    assert!(client.register_vet(
        &vet,
        &string("Vet"),
        &string("BUDGET-1"),
        &string("General")
    ));
    assert!(client.verify_vet(&owner, &vet));
    let diagnosis = string(&"D".repeat(MAX_MEDICAL_DIAGNOSIS_LEN as usize));
    let treatment = string(&"T".repeat(MAX_MEDICAL_TREATMENT_LEN as usize));
    let notes = string(&"N".repeat(MAX_MEDICAL_NOTES_LEN as usize));
    let meds = Vec::new(&env);
    let add_record =
        || client.add_medical_record(&pet, &vet, &diagnosis, &treatment, &meds, &notes);
    // The cursor endpoint has no page cap. Fix the workload at 50 records.
    const PAGE_SIZE: u32 = 50;
    for _ in 1..PAGE_SIZE {
        add_record();
    }
    measure(&env, "record", || {
        add_record();
    });
    measure(&env, "pagination", || {
        let page = client.get_pet_medical_records_cursor(&pet, &0, &PAGE_SIZE);
        assert_eq!(page.items.len(), PAGE_SIZE);
    });

    let grantees: std::vec::Vec<_> = (0..PetChainContract::MAX_CONSENTS_PER_PET)
        .map(|_| Address::generate(&env))
        .collect();
    let (last, others) = grantees.split_last().unwrap();
    for grantee in others {
        client.grant_consent(&pet, &owner, &ConsentType::Insurance, grantee);
    }
    measure(&env, "consent", || {
        client.grant_consent(&pet, &owner, &ConsentType::Insurance, last);
    });

    assert!(client.add_insurance_policy(
        &pet,
        &string("budget-policy"),
        &string("Provider"),
        &string("Medical"),
        &1,
        &1_000,
        &86_400,
    ));
    // The legacy claim endpoint has no description cap.
    let description = string(&"C".repeat(1_000));
    measure(&env, "claim", || {
        let claim = client
            .submit_insurance_claim(&pet, &1_000, &description)
            .unwrap();
        assert!(client.update_insurance_claim_status(&claim, &InsuranceClaimStatus::Paid));
        assert!(client.is_claim_settled(&claim));
    });
}
