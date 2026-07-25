use crate::{
    ContractError, Gender, PetChainContract, PetChainContractClient, PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// ── helpers ────────────────────────────────────────────────────────────────

fn register_pet(
    env: &Env,
    client: &PetChainContractClient,
    owner: &Address,
    name: &str,
) -> u64 {
    client.register_pet(
        owner,
        &String::from_str(env, name),
        &String::from_str(env, "2020-01-01"),
        &Gender::Female,
        &Species::Dog,
        &String::from_str(env, "Brown"),
        &String::from_str(env, "Mixed"),
        &20u32,
        &None,
        &PrivacyLevel::Public,
    )
}

/// Create a breeding record between sire and dam, then assign offspring.
/// Returns the breeding record id.
fn breed_and_assign(
    env: &Env,
    client: &PetChainContractClient,
    sire_id: u64,
    dam_id: u64,
    offspring_id: u64,
) -> u64 {
    let record_id = client.add_breeding_record(
        &sire_id,
        &dam_id,
        &env.ledger().timestamp(),
        &String::from_str(env, "litter"),
    );
    assert!(client.add_offspring(&record_id, &offspring_id));
    record_id
}

// ── test: unrelated pair → COI = 0 ─────────────────────────────────────────

#[test]
fn test_coi_unrelated_pair_returns_zero() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    // Two completely unrelated pets
    let pet_a = register_pet(&env, &client, &owner, "Alpha");
    let pet_b = register_pet(&env, &client, &owner, "Beta");

    let coi = client.calculate_coi(&pet_a, &pet_b);
    assert_eq!(coi, 0u32, "COI should be 0 for unrelated pets");
}

// ── test: half-siblings → COI = 1250 bp (1/8) ──────────────────────────────

#[test]
fn test_coi_half_siblings_returns_1250() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    // Grand-sire is the shared ancestor
    let common_sire = register_pet(&env, &client, &owner, "CommonSire");
    let dam_a = register_pet(&env, &client, &owner, "DamA");
    let dam_b = register_pet(&env, &client, &owner, "DamB");

    // Offspring from common_sire × dam_a
    let half_a = register_pet(&env, &client, &owner, "HalfA");
    breed_and_assign(&env, &client, common_sire, dam_a, half_a);

    // Offspring from common_sire × dam_b (same sire → half-siblings)
    let half_b = register_pet(&env, &client, &owner, "HalfB");
    breed_and_assign(&env, &client, common_sire, dam_b, half_b);

    // half_a and half_b share common_sire as parent.
    // Path: half_a → common_sire ← half_b
    //    n1 = 1, n2 = 1  →  10000 / 2^(1+1+1) = 10000 / 8 = 1250 bp
    let coi = client.calculate_coi(&half_a, &half_b);
    assert_eq!(coi, 1250u32, "COI should be 1250 bp for half-siblings");
}

// ── test: first cousins → COI ≈ 313 bp (1/32) ──────────────────────────────

#[test]
fn test_coi_first_cousins_full_siblings_returns_624() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    // Grandparents (shared ancestors)
    let grand_sire = register_pet(&env, &client, &owner, "GrandSire");
    let grand_dam = register_pet(&env, &client, &owner, "GrandDam");

    // Parents of cousin A
    let parent_a = register_pet(&env, &client, &owner, "ParentA");
    breed_and_assign(&env, &client, grand_sire, grand_dam, parent_a);

    // Parents of cousin B
    let parent_b = register_pet(&env, &client, &owner, "ParentB");
    breed_and_assign(&env, &client, grand_sire, grand_dam, parent_b);

    // Cousins
    let cousin_a = register_pet(&env, &client, &owner, "CousinA");
    let unrelated_dam_a = register_pet(&env, &client, &owner, "UnrelatedDamA");
    breed_and_assign(&env, &client, parent_a, unrelated_dam_a, cousin_a);

    let cousin_b = register_pet(&env, &client, &owner, "CousinB");
    let unrelated_dam_b = register_pet(&env, &client, &owner, "UnrelatedDamB");
    breed_and_assign(&env, &client, parent_b, unrelated_dam_b, cousin_b);

    // cousin_a and cousin_b share grand_sire and grand_dam.
    // Through grand_sire: n1 = 2, n2 = 2 → 10000/2^5 = 10000/32 = 312
    // Through grand_dam:  n1 = 2, n2 = 2 → 10000/2^5 = 10000/32 = 312
    // Total ≈ 624 bp … wait, that's double-counting.  Let's compute properly.
    //
    // Each path through a *unique* common ancestor contributes separately:
    //   GrandSire: 10000 / 2^(2+2+1) = 10000 / 32 = 312
    //   GrandDam:  10000 / 2^(2+2+1) = 10000 / 32 = 312
    //   COI = 312 + 312 = 624  (both grandparents are shared because the
    //                           parents are full siblings)
    //
    // Actually the parents are full siblings (same sire and dam), so both
    // grandparents contribute.  312 + 312 = 624 bp.
    let coi = client.calculate_coi(&cousin_a, &cousin_b);
    assert_eq!(coi, 624u32, "COI should be 624 bp for first cousins (both grandparents shared)");
}

// ── test: COI guard rejects high inbreeding ────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #36)")]
fn test_register_breeding_pair_rejects_high_coi() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    // Create half-siblings (COI = 1250)
    let common_sire = register_pet(&env, &client, &owner, "CommonSire");
    let dam_a = register_pet(&env, &client, &owner, "DamA");
    let dam_b = register_pet(&env, &client, &owner, "DamB");
    let half_a = register_pet(&env, &client, &owner, "HalfA");
    breed_and_assign(&env, &client, common_sire, dam_a, half_a);
    let half_b = register_pet(&env, &client, &owner, "HalfB");
    breed_and_assign(&env, &client, common_sire, dam_b, half_b);

    // Attempt to breed with max_coi_bp = 1000 (< 1250 → should panic)
    client.register_breeding_pair(
        &half_a,
        &half_b,
        &env.ledger().timestamp(),
        &String::from_str(&env, "risky"),
        &1000u32,
    );
}

// ── test: COI guard allows low-COI breeding ────────────────────────────────

#[test]
fn test_register_breeding_pair_allows_unrelated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    let sire = register_pet(&env, &client, &owner, "Sire");
    let dam = register_pet(&env, &client, &owner, "Dam");

    // Unrelated pair with max_coi_bp = 1000 (COI = 0 < 1000 → OK)
    let record_id = client.register_breeding_pair(
        &sire,
        &dam,
        &env.ledger().timestamp(),
        &String::from_str(&env, "healthy"),
        &1000u32,
    );
    assert!(record_id > 0);

    let record = client.get_breeding_record(&record_id);
    assert!(record.is_some());
}

// ── test: self-breeding is rejected ─────────────────────────────────────────

#[test]
#[should_panic(expected = "Error(Contract, #37)")]
fn test_register_breeding_pair_rejects_self_breeding() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let owner = Address::generate(&env);

    let pet = register_pet(&env, &client, &owner, "Self");

    client.register_breeding_pair(
        &pet,
        &pet,
        &env.ledger().timestamp(),
        &String::from_str(&env, "self"),
        &10000u32,
    );
}
