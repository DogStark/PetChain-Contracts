//! Claim amount validation (#1207).
//!
//! Assumption: no claim-submission entry point exists in `lib.rs` on `main`;
//! these pure checks are what such an entry point must call before touching
//! reserves. Asset matching is expressed as a comparison of asset ids.

/// Policy terms relevant to a claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoverageTerms {
    pub coverage_limit: u64,
    pub deductible: u64,
    /// Sum of payouts already made under this policy.
    pub cumulative_paid: u64,
    pub asset_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClaimError {
    ZeroAmount,
    AssetMismatch,
    BelowDeductible,
    ExceedsRemainingCoverage,
    Overflow,
}

/// Returns the payable amount for a claim of `amount` in `asset_id`:
/// `amount - deductible`, which must fit in the remaining coverage
/// (`coverage_limit - cumulative_paid`).
pub fn validate_claim(terms: &CoverageTerms, amount: u64, asset_id: u32) -> Result<u64, ClaimError> {
    if amount == 0 {
        return Err(ClaimError::ZeroAmount);
    }
    if asset_id != terms.asset_id {
        return Err(ClaimError::AssetMismatch);
    }
    let payable = amount
        .checked_sub(terms.deductible)
        .filter(|p| *p > 0)
        .ok_or(ClaimError::BelowDeductible)?;
    let new_total = terms
        .cumulative_paid
        .checked_add(payable)
        .ok_or(ClaimError::Overflow)?;
    if new_total > terms.coverage_limit {
        return Err(ClaimError::ExceedsRemainingCoverage);
    }
    Ok(payable)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> CoverageTerms {
        CoverageTerms { coverage_limit: 1000, deductible: 100, cumulative_paid: 200, asset_id: 1 }
    }

    #[test]
    fn valid_claim_returns_net_amount() {
        assert_eq!(validate_claim(&t(), 500, 1), Ok(400));
    }

    #[test]
    fn exact_remaining_coverage_is_allowed_and_one_more_rejected() {
        assert_eq!(validate_claim(&t(), 900, 1), Ok(800));
        assert_eq!(validate_claim(&t(), 901, 1), Err(ClaimError::ExceedsRemainingCoverage));
    }

    #[test]
    fn invalid_inputs_rejected() {
        assert_eq!(validate_claim(&t(), 0, 1), Err(ClaimError::ZeroAmount));
        assert_eq!(validate_claim(&t(), 500, 2), Err(ClaimError::AssetMismatch));
        assert_eq!(validate_claim(&t(), 100, 1), Err(ClaimError::BelowDeductible));
        assert_eq!(validate_claim(&t(), 50, 1), Err(ClaimError::BelowDeductible));
    }

    #[test]
    fn overflow_rejected() {
        let mut terms = t();
        terms.cumulative_paid = u64::MAX;
        assert_eq!(validate_claim(&terms, 500, 1), Err(ClaimError::Overflow));
    }

    #[test]
    fn repeated_claims_cannot_exceed_limit_cumulatively() {
        let mut terms = t();
        while let Ok(p) = validate_claim(&terms, 300, 1) {
            terms.cumulative_paid += p;
        }
        assert!(terms.cumulative_paid <= terms.coverage_limit);
    }
}
