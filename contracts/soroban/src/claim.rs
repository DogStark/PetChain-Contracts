use soroban_sdk::{contracterror, contracttype, Address, Env, String, Vec};

/// Maximum length (in bytes) allowed for a secondary payer reference.
pub const MAX_SECONDARY_PAYER_REF_LEN: u32 = 64;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ClaimError {
    Unauthorized = 1,
    NotFound = 2,
    AlreadyProcessed = 3,
    Expired = 4,
    InvalidAmount = 5,
    /// Aggregate reimbursement across primary and secondary payers would
    /// exceed the claim's eligible expenses.
    ReimbursementExceedsEligible = 6,
    /// Secondary payer reference exceeds `MAX_SECONDARY_PAYER_REF_LEN`.
    SecondaryPayerRefTooLong = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claim {
    pub claimant: Address,
    pub eligible_expenses: i128,
    pub primary_reimbursed: i128,
    pub secondary_reimbursed: i128,
    /// Bounded reference to the secondary payer (e.g. COB coordination id).
    pub secondary_payer_ref: Option<String>,
    pub processed: bool,
    pub expiry_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClaimStorage {
    pub version: u32,
    pub claims: Vec<Claim>,
}

pub const CLAIM_STORAGE_VERSION: u32 = 2;

fn load_storage(env: &Env) -> ClaimStorage {
    env.storage()
        .persistent()
        .get(&"claims")
        .unwrap_or(ClaimStorage {
            version: CLAIM_STORAGE_VERSION,
            claims: Vec::new(env),
        })
}

fn save_storage(env: &Env, storage: &ClaimStorage) {
    env.storage().persistent().set(&"claims", storage);
}

fn validate_secondary_ref(env: &Env, reference: &Option<String>) -> Result<(), ClaimError> {
    if let Some(r) = reference {
        if r.len() > MAX_SECONDARY_PAYER_REF_LEN {
            return Err(ClaimError::SecondaryPayerRefTooLong);
        }
    }
    let _ = env;
    Ok(())
}

/// Record a reimbursement from a payer against a claim, enforcing that the
/// aggregate (primary + secondary) never exceeds eligible expenses.
pub fn record_reimbursement(
    env: &Env,
    claimant: &Address,
    claim_index: u32,
    amount: i128,
    is_secondary: bool,
    secondary_payer_ref: Option<String>,
) -> Result<(), ClaimError> {
    claimant.require_auth();

    if amount <= 0 {
        return Err(ClaimError::InvalidAmount);
    }

    validate_secondary_ref(env, &secondary_payer_ref)?;

    let mut storage = load_storage(env);
    let mut claim = storage
        .claims
        .get(claim_index)
        .ok_or(ClaimError::NotFound)?;

    if claim.claimant != *claimant {
        return Err(ClaimError::Unauthorized);
    }
    if claim.processed {
        return Err(ClaimError::AlreadyProcessed);
    }
    if env.ledger().sequence() > claim.expiry_ledger {
        return Err(ClaimError::Expired);
    }

    let new_primary = if is_secondary {
        claim.primary_reimbursed
    } else {
        claim.primary_reimbursed
            .checked_add(amount)
            .ok_or(ClaimError::InvalidAmount)?
    };
    let new_secondary = if is_secondary {
        claim
            .secondary_reimbursed
            .checked_add(amount)
            .ok_or(ClaimError::InvalidAmount)?
    } else {
        claim.secondary_reimbursed
    };

    let aggregate = new_primary
        .checked_add(new_secondary)
        .ok_or(ClaimError::InvalidAmount)?;
    if aggregate > claim.eligible_expenses {
        return Err(ClaimError::ReimbursementExceedsEligible);
    }

    claim.primary_reimbursed = new_primary;
    claim.secondary_reimbursed = new_secondary;
    if is_secondary {
        if let Some(r) = secondary_payer_ref {
            claim.secondary_payer_ref = Some(r);
        }
    }

    storage.claims.set(claim_index, claim);
    save_storage(env, &storage);

    env.events().publish(
        ("claim", "reimbursement"),
        (claim_index, amount, is_secondary, aggregate),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

    fn setup(env: &Env, eligible: i128, expiry: u32) -> (Address, u32) {
        let claimant = Address::generate(env);
        let claim = Claim {
            claimant: claimant.clone(),
            eligible_expenses: eligible,
            primary_reimbursed: 0,
            secondary_reimbursed: 0,
            secondary_payer_ref: None,
            processed: false,
            expiry_ledger: expiry,
        };
        let mut storage = ClaimStorage {
            version: CLAIM_STORAGE_VERSION,
            claims: Vec::new(env),
        };
        storage.claims.push_back(claim);
        save_storage(env, &storage);
        (claimant, 0)
    }

    #[test]
    fn secondary_reimbursement_within_cap_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        let (claimant, idx) = setup(&env, 1000, 100);
        let reference = Some(String::from_str(&env, "COB-123"));
        record_reimbursement(&env, &claimant, idx, 400, false, None).unwrap();
        record_reimbursement(&env, &claimant, idx, 600, true, reference.clone()).unwrap();
        let storage = load_storage(&env);
        let claim = storage.claims.get(idx).unwrap();
        assert_eq!(claim.primary_reimbursed, 400);
        assert_eq!(claim.secondary_reimbursed, 600);
        assert_eq!(claim.secondary_payer_ref, reference);
    }

    #[test]
    fn exact_cap_allowed_cap_plus_one_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (claimant, idx) = setup(&env, 1000, 100);
        record_reimbursement(&env, &claimant, idx, 1000, false, None).unwrap();
        let before = load_storage(&env);
        let res = record_reimbursement(&env, &claimant, idx, 1, true, None);
        assert_eq!(res, Err(ClaimError::ReimbursementExceedsEligible));
        let after = load_storage(&env);
        assert_eq!(before, after);
    }

    #[test]
    fn over_length_reference_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (claimant, idx) = setup(&env, 1000, 100);
        let long = String::from_str(&env, &"a".repeat((MAX_SECONDARY_PAYER_REF_LEN + 1) as usize));
        let before = load_storage(&env);
        let res = record_reimbursement(&env, &claimant, idx, 100, true, Some(long));
        assert_eq!(res, Err(ClaimError::SecondaryPayerRefTooLong));
        assert_eq!(before, load_storage(&env));
    }

    #[test]
    fn unauthorized_claimant_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (_claimant, idx) = setup(&env, 1000, 100);
        let other = Address::generate(&env);
        let before = load_storage(&env);
        let res = record_reimbursement(&env, &other, idx, 100, false, None);
        assert_eq!(res, Err(ClaimError::Unauthorized));
        assert_eq!(before, load_storage(&env));
    }

    #[test]
    fn expired_claim_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (claimant, idx) = setup(&env, 1000, 10);
        env.ledger().set_sequence_number(11);
        let before = load_storage(&env);
        let res = record_reimbursement(&env, &claimant, idx, 100, false, None);
        assert_eq!(res, Err(ClaimError::Expired));
        assert_eq!(before, load_storage(&env));
    }

    #[test]
    fn replay_after_processed_rejected() {
        let env = Env::default();
        env.mock_all_auths();
        let (claimant, idx) = setup(&env, 1000, 100);
        let mut storage = load_storage(&env);
        let mut claim = storage.claims.get(idx).unwrap();
        claim.processed = true;
        storage.claims.set(idx, claim);
        save_storage(&env, &storage);
        let before = load_storage(&env);
        let res = record_reimbursement(&env, &claimant, idx, 100, false, None);
        assert_eq!(res, Err(ClaimError::AlreadyProcessed));
        assert_eq!(before, load_storage(&env));
    }
}
