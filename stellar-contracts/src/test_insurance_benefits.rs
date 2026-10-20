use crate::{
    ContractError, Gender, InsuranceClaimStatus, PetChainContract, PetChainContractClient,
    PrivacyLevel, Species,
};
use soroban_sdk::{testutils::Address as _, testutils::Ledger, Address, Env, Error, String};

fn setup() -> (Env, PetChainContractClient<'static>, Address, Address, u64, String) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, PetChainContract);
    let client = PetChainContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.init_admin(&admin);
    let owner = Address::generate(&env);
    let pet_id = client.register_pet(
        &owner,
        &String::from_str(&env, "Milo"),
        &String::from_str(&env, "2021-01-01"),
        &Gender::Male,
        &Species::Dog,
        &String::from_str(&env, "Brown"),
        &String::from_str(&env, "Mixed"),
        &10,
        &None,
        &PrivacyLevel::Public,
    );
    let policy_id = String::from_str(&env, "POL-BENEFITS");
    client.add_insurance_policy(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "Example Mutual"),
        &String::from_str(&env, "Comprehensive"),
        &100,
        &10_000,
        &10_000,
    );
    (env, client, owner, admin, pet_id, policy_id)
}

fn configure(
    env: &Env,
    client: &PetChainContractClient,
    owner: &Address,
    pet_id: u64,
    policy_id: &String,
    waiting_period: u64,
    period: u64,
    limit: u64,
    deductible: u64,
    copay_bps: u32,
) {
    client.set_condition_coverage(
        owner,
        &pet_id,
        policy_id,
        &String::from_str(env, "orthopedic"),
        &waiting_period,
        &period,
        &limit,
        &deductible,
        &copay_bps,
    );
}

#[test]
fn benefit_claim_uses_waiting_period_deductible_and_copay() {
    let (env, client, owner, admin, pet_id, policy_id) = setup();
    env.ledger().set_timestamp(300);
    configure(&env, &client, &owner, pet_id, &policy_id, 100, 1_000, 1_000, 100, 2_000);
    env.ledger().set_timestamp(500);

    let before_waiting_end = client.try_submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &399,
        &1_000,
        &String::from_str(&env, "Treatment"),
    ).unwrap_err().unwrap();
    assert_eq!(before_waiting_end, Error::from(ContractError::BenefitWaitingPeriod));
    assert!(client.get_pet_insurance_claims(&pet_id).is_empty());
    assert!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &300,
    ).is_none());

    let claim_id = client.submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &400,
        &1_000,
        &String::from_str(&env, "Treatment"),
    );
    let benefit_claim = client.get_insurance_benefit_claim(&claim_id).unwrap();
    assert_eq!(benefit_claim.deductible_applied, 100);
    assert_eq!(benefit_claim.copay, 180);
    assert_eq!(benefit_claim.insurer_payment, 720);
    let usage = client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &300,
    ).unwrap();
    assert_eq!(usage.pending, 720);
    assert_eq!(usage.paid, 0);

    client.settle_condition_claim(&admin, &claim_id, &true);
    let replay = client.try_settle_condition_claim(&admin, &claim_id, &true)
        .unwrap_err().unwrap();
    assert_eq!(replay, Error::from(ContractError::BenefitClaimAlreadyFinalized));
    assert!(client.is_claim_settled(&claim_id));
    let settled = client.get_insurance_claim(&claim_id).unwrap();
    assert_eq!(settled.status, InsuranceClaimStatus::Paid);
    let usage = client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &300,
    ).unwrap();
    assert_eq!(usage.pending, 0);
    assert_eq!(usage.paid, 720);
}

#[test]
fn condition_sections_are_bounded_per_policy() {
    let (env, client, owner, _admin, pet_id, policy_id) = setup();
    for index in 0..32u32 {
        let name = format!("condition-{index}");
        client.set_condition_coverage(
            &owner,
            &pet_id,
            &policy_id,
            &String::from_str(&env, &name),
            &0,
            &1_000,
            &1_000,
            &0,
            &0,
        );
    }

    let over_limit = client.try_set_condition_coverage(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "condition-32"),
        &0,
        &1_000,
        &1_000,
        &0,
        &0,
    ).unwrap_err().unwrap();
    assert_eq!(over_limit, Error::from(ContractError::BenefitConditionLimitReached));
    assert!(client.get_condition_coverage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "condition-32"),
    ).is_none());
}

#[test]
fn owner_and_admin_authorization_are_enforced_without_mutation() {
    let (env, client, owner, admin, pet_id, policy_id) = setup();
    let stranger = Address::generate(&env);
    let unauthorized_configuration = client.try_set_condition_coverage(
        &stranger,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
        &1_000,
        &1_000,
        &0,
        &0,
    ).unwrap_err().unwrap();
    assert_eq!(unauthorized_configuration, Error::from(ContractError::Unauthorized));
    assert!(client.get_condition_coverage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
    ).is_none());

    configure(&env, &client, &owner, pet_id, &policy_id, 0, 1_000, 1_000, 0, 0);
    env.ledger().set_timestamp(1);
    let claim_id = client.submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &1,
        &100,
        &String::from_str(&env, "Treatment"),
    );
    let unauthorized_settlement = client.try_settle_condition_claim(&stranger, &claim_id, &true)
        .unwrap_err().unwrap();
    assert_eq!(unauthorized_settlement, Error::from(ContractError::Unauthorized));
    assert_eq!(client.get_insurance_claim(&claim_id).unwrap().status, InsuranceClaimStatus::Pending);
    assert_eq!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
    ).unwrap().pending, 100);
    assert!(client.get_admins().contains(admin));
}

#[test]
fn expired_policy_and_reconfigured_terms_fail_without_state_changes() {
    let (env, client, owner, _admin, pet_id, policy_id) = setup();
    configure(&env, &client, &owner, pet_id, &policy_id, 0, 1_000, 1_000, 0, 0);
    let terms = client.get_condition_coverage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
    ).unwrap();

    let reconfiguration = client.try_set_condition_coverage(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
        &1_000,
        &2_000,
        &0,
        &0,
    ).unwrap_err().unwrap();
    assert_eq!(reconfiguration, Error::from(ContractError::BenefitTermsAlreadyConfigured));
    assert_eq!(client.get_condition_coverage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
    ).unwrap(), terms);

    env.ledger().set_timestamp(10_000);
    let expired = client.try_submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &9_999,
        &100,
        &String::from_str(&env, "Treatment"),
    ).unwrap_err().unwrap();
    assert_eq!(expired, Error::from(ContractError::InsurancePolicyInactive));
    assert!(client.get_pet_insurance_claims(&pet_id).is_empty());
    assert!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
    ).is_none());
}

#[test]
fn condition_limit_is_shared_by_pending_and_paid_claims_then_resets_by_period() {
    let (env, client, owner, _admin, pet_id, policy_id) = setup();
    env.ledger().set_timestamp(100);
    configure(&env, &client, &owner, pet_id, &policy_id, 0, 100, 1_000, 0, 0);
    env.ledger().set_timestamp(160);
    let _claim_id = client.submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &150,
        &1_000,
        &String::from_str(&env, "Treatment"),
    );
    let before = client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &100,
    ).unwrap();
    assert_eq!(before.pending, 1_000);

    let over_limit = client.try_submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &151,
        &1,
        &String::from_str(&env, "Treatment"),
    ).unwrap_err().unwrap();
    assert_eq!(over_limit, Error::from(ContractError::BenefitCoverageLimitExceeded));
    assert_eq!(client.get_pet_insurance_claims(&pet_id).len(), 1);
    assert_eq!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &100,
    ).unwrap(), before);

    env.ledger().set_timestamp(250);
    let next_period_claim = client.submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &200,
        &1_000,
        &String::from_str(&env, "Treatment"),
    );
    assert_eq!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &200,
    ).unwrap().pending, 1_000);
    assert_eq!(client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &100,
    ).unwrap().pending, 1_000);
    assert_eq!(client.get_insurance_claim(&next_period_claim).unwrap().status, InsuranceClaimStatus::Pending);
}

#[test]
fn condition_claim_cannot_be_settled_twice_or_through_legacy_path() {
    let (env, client, owner, admin, pet_id, policy_id) = setup();
    configure(&env, &client, &owner, pet_id, &policy_id, 0, 1_000, 1_000, 0, 0);
    let claim_id = client.submit_condition_claim(
        &owner,
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
        &100,
        &String::from_str(&env, "Treatment"),
    );

    let legacy_attempt = client.try_update_insurance_claim_status(
        &claim_id,
        &InsuranceClaimStatus::Paid,
    ).unwrap_err().unwrap();
    assert_eq!(legacy_attempt, Error::from(ContractError::BenefitClaimMustUseSettlement));
    client.settle_condition_claim(&admin, &claim_id, &false);
    let replay = client.try_settle_condition_claim(&admin, &claim_id, &true)
        .unwrap_err().unwrap();
    assert_eq!(replay, Error::from(ContractError::BenefitClaimAlreadyFinalized));
    assert_eq!(client.get_insurance_claim(&claim_id).unwrap().status, InsuranceClaimStatus::Rejected);
    let usage = client.get_condition_period_usage(
        &pet_id,
        &policy_id,
        &String::from_str(&env, "orthopedic"),
        &0,
    ).unwrap();
    assert_eq!(usage.pending, 0);
    assert_eq!(usage.paid, 0);
}