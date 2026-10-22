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

pub const MAX_COPAY_BPS: u32 = 10_000;
pub const MAX_CONDITION_NAME_LEN: u32 = 64;
pub const MAX_CONDITIONS_PER_POLICY: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConditionUsage {
    pub period_start: u64,
    pub paid: u64,
    pub pending: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BenefitSettlement {
    pub deductible_applied: u64,
    pub copay: u64,
    pub insurer_payment: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BenefitError {
    InvalidTerms,
    InvalidServiceDate,
    WaitingPeriod,
    ZeroAmount,
    ArithmeticOverflow,
    CoverageLimitExceeded,
    PeriodRegression,
    PendingAmountMismatch,
}

pub fn validate_condition_terms(
    coverage_start: u64,
    waiting_period_secs: u64,
    period_secs: u64,
    condition_limit: u64,
    copay_bps: u32,
    policy_expiry: u64,
) -> Result<u64, BenefitError> {
    if period_secs == 0
        || condition_limit == 0
        || copay_bps > MAX_COPAY_BPS
        || coverage_start > policy_expiry
    {
        return Err(BenefitError::InvalidTerms);
    }

    let waiting_period_end = coverage_start
        .checked_add(waiting_period_secs)
        .ok_or(BenefitError::ArithmeticOverflow)?;
    if waiting_period_end >= policy_expiry {
        return Err(BenefitError::InvalidTerms);
    }
    Ok(waiting_period_end)
}

pub fn validate_service_date(
    coverage_start: u64,
    waiting_period_end: u64,
    service_date: u64,
    policy_expiry: u64,
    now: u64,
) -> Result<(), BenefitError> {
    if service_date < coverage_start || service_date >= policy_expiry || service_date > now {
        return Err(BenefitError::InvalidServiceDate);
    }
    if service_date < waiting_period_end {
        return Err(BenefitError::WaitingPeriod);
    }
    Ok(())
}

/// Applies the deductible first, then rounds the copay down to the nearest
/// smallest asset unit. The insurer pays the remainder after copay.
pub fn calculate_benefit_settlement(
    amount: u64,
    deductible: u64,
    copay_bps: u32,
) -> Result<BenefitSettlement, BenefitError> {
    if amount == 0 {
        return Err(BenefitError::ZeroAmount);
    }
    if copay_bps > MAX_COPAY_BPS {
        return Err(BenefitError::InvalidTerms);
    }

    let deductible_applied = amount.min(deductible);
    let after_deductible = amount - deductible_applied;
    let copay = ((after_deductible as u128 * copay_bps as u128) / MAX_COPAY_BPS as u128) as u64;
    Ok(BenefitSettlement {
        deductible_applied,
        copay,
        insurer_payment: after_deductible - copay,
    })
}

pub fn reserve_condition_claim(
    current: Option<ConditionUsage>,
    period_start: u64,
    condition_limit: u64,
    amount: u64,
) -> Result<ConditionUsage, BenefitError> {
    let mut usage = match current {
        Some(current) if period_start < current.period_start => {
            return Err(BenefitError::PeriodRegression)
        }
        Some(current) if period_start == current.period_start => current,
        Some(_) | None => ConditionUsage {
            period_start,
            paid: 0,
            pending: 0,
        },
    };

    let reserved_total = usage
        .paid
        .checked_add(usage.pending)
        .and_then(|total| total.checked_add(amount))
        .ok_or(BenefitError::ArithmeticOverflow)?;
    if reserved_total > condition_limit {
        return Err(BenefitError::CoverageLimitExceeded);
    }
    usage.pending = usage
        .pending
        .checked_add(amount)
        .ok_or(BenefitError::ArithmeticOverflow)?;
    Ok(usage)
}

pub fn finalize_condition_claim(
    current: ConditionUsage,
    amount: u64,
    approved: bool,
    condition_limit: u64,
) -> Result<ConditionUsage, BenefitError> {
    let pending = current
        .pending
        .checked_sub(amount)
        .ok_or(BenefitError::PendingAmountMismatch)?;
    let paid = if approved {
        current
            .paid
            .checked_add(amount)
            .ok_or(BenefitError::ArithmeticOverflow)?
    } else {
        current.paid
    };
    if paid
        .checked_add(pending)
        .ok_or(BenefitError::ArithmeticOverflow)?
        > condition_limit
    {
        return Err(BenefitError::CoverageLimitExceeded);
    }
    Ok(ConditionUsage {
        period_start: current.period_start,
        paid,
        pending,
    })
}

/// Maximum number of coverage entries allowed in an enrollment snapshot.
pub const MAX_COVERAGE_ENTRIES: u32 = 32;
/// Maximum number of exclusion entries allowed in an enrollment snapshot.
pub const MAX_EXCLUSION_ENTRIES: u32 = 32;

/// Enrollment application states (#1470).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnrollmentState {
    Draft,
    Submitted,
    UnderReview,
    Accepted,
    Declined,
}

impl EnrollmentState {
    /// Allowed transitions: Draft -> Submitted -> UnderReview -> Accepted|Declined.
    pub fn can_transition_to(self, next: EnrollmentState) -> bool {
        matches!(
            (self, next),
            (EnrollmentState::Draft, EnrollmentState::Submitted)
                | (EnrollmentState::Submitted, EnrollmentState::UnderReview)
                | (EnrollmentState::UnderReview, EnrollmentState::Accepted)
                | (EnrollmentState::UnderReview, EnrollmentState::Declined)
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, EnrollmentState::Accepted | EnrollmentState::Declined)
    }
}

/// Immutable coverage/exclusion snapshot bound to a policy version (#1472).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CoverageSnapshot {
    pub policy_version: u32,
    pub coverage_entries: u32,
    pub exclusion_entries: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnrollmentError {
    /// Coverage or exclusion vector exceeds the bounded maximum.
    TooManyEntries,
    /// Decision references a policy version different from the application's.
    PolicyVersionMismatch,
    /// Snapshot was taken against a different policy version than the decision.
    SnapshotVersionMismatch,
    /// Transition is not permitted from the current state (includes replay).
    InvalidTransition,
    /// Application is already in a terminal state (replay of a decision).
    AlreadyDecided,
    /// Application has expired and can no longer be decided.
    Expired,
}

/// Validates the bounded sizes of an enrollment application's snapshot inputs.
pub fn validate_enrollment_inputs(
    coverage_entries: u32,
    exclusion_entries: u32,
) -> Result<(), EnrollmentError> {
    if coverage_entries > MAX_COVERAGE_ENTRIES || exclusion_entries > MAX_EXCLUSION_ENTRIES {
        return Err(EnrollmentError::TooManyEntries);
    }
    Ok(())
}

/// Validates a state transition for an application bound to `application_version`.
/// `now`/`expires_at` are ledger timestamps; `expires_at == 0` means no expiry.
pub fn validate_transition(
    current: EnrollmentState,
    next: EnrollmentState,
    application_version: u32,
    decision_version: u32,
    now: u64,
    expires_at: u64,
) -> Result<(), EnrollmentError> {
    if current.is_terminal() {
        return Err(EnrollmentError::AlreadyDecided);
    }
    if expires_at != 0 && now >= expires_at {
        return Err(EnrollmentError::Expired);
    }
    if application_version != decision_version {
        return Err(EnrollmentError::PolicyVersionMismatch);
    }
    if !current.can_transition_to(next) {
        return Err(EnrollmentError::InvalidTransition);
    }
    Ok(())
}

/// Validates that a snapshot matches the policy version at acceptance (#1472).
pub fn validate_snapshot_at_acceptance(
    snapshot: &CoverageSnapshot,
    policy_version: u32,
) -> Result<(), EnrollmentError> {
    validate_enrollment_inputs(snapshot.coverage_entries, snapshot.exclusion_entries)?;
    if snapshot.policy_version != policy_version {
        return Err(EnrollmentError::SnapshotVersionMismatch);
    }
    Ok(())
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

    #[test]
    fn enrollment_input_sizes_are_bounded() {
        assert_eq!(validate_enrollment_inputs(0, 0), Ok(()));
        assert_eq!(
            validate_enrollment_inputs(MAX_COVERAGE_ENTRIES, MAX_EXCLUSION_ENTRIES),
            Ok(())
        );
        assert_eq!(
            validate_enrollment_inputs(MAX_COVERAGE_ENTRIES + 1, 0),
            Err(EnrollmentError::TooManyEntries)
        );
        assert_eq!(
            validate_enrollment_inputs(0, MAX_EXCLUSION_ENTRIES + 1),
            Err(EnrollmentError::TooManyEntries)
        );
    }

    #[test]
    fn happy_path_transitions_succeed() {
        assert_eq!(
            validate_transition(EnrollmentState::Draft, EnrollmentState::Submitted, 3, 3, 0, 0),
            Ok(())
        );
        assert_eq!(
            validate_transition(EnrollmentState::Submitted, EnrollmentState::UnderReview, 3, 3, 0, 0),
            Ok(())
        );
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Accepted, 3, 3, 0, 0),
            Ok(())
        );
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Declined, 3, 3, 0, 0),
            Ok(())
        );
    }

    #[test]
    fn mismatched_policy_versions_rejected() {
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Accepted, 3, 4, 0, 0),
            Err(EnrollmentError::PolicyVersionMismatch)
        );
    }

    #[test]
    fn invalid_transitions_rejected() {
        assert_eq!(
            validate_transition(EnrollmentState::Draft, EnrollmentState::Accepted, 3, 3, 0, 0),
            Err(EnrollmentError::InvalidTransition)
        );
        assert_eq!(
            validate_transition(EnrollmentState::Submitted, EnrollmentState::Draft, 3, 3, 0, 0),
            Err(EnrollmentError::InvalidTransition)
        );
    }

    #[test]
    fn replay_of_decision_rejected() {
        assert_eq!(
            validate_transition(EnrollmentState::Accepted, EnrollmentState::Accepted, 3, 3, 0, 0),
            Err(EnrollmentError::AlreadyDecided)
        );
        assert_eq!(
            validate_transition(EnrollmentState::Declined, EnrollmentState::Accepted, 3, 3, 0, 0),
            Err(EnrollmentError::AlreadyDecided)
        );
    }

    #[test]
    fn expiry_boundary_rejected() {
        // now == expires_at is expired; now < expires_at is allowed.
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Accepted, 3, 3, 100, 100),
            Err(EnrollmentError::Expired)
        );
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Accepted, 3, 3, 99, 100),
            Ok(())
        );
        // expires_at == 0 disables expiry.
        assert_eq!(
            validate_transition(EnrollmentState::UnderReview, EnrollmentState::Accepted, 3, 3, u64::MAX, 0),
            Ok(())
        );
    }

    #[test]
    fn snapshot_must_match_policy_version_at_acceptance() {
        let snap = CoverageSnapshot {
            policy_version: 5,
            coverage_entries: MAX_COVERAGE_ENTRIES,
            exclusion_entries: MAX_EXCLUSION_ENTRIES,
        };
        assert_eq!(validate_snapshot_at_acceptance(&snap, 5), Ok(()));
        assert_eq!(
            validate_snapshot_at_acceptance(&snap, 6),
            Err(EnrollmentError::SnapshotVersionMismatch)
        );
        let oversized = CoverageSnapshot {
            policy_version: 5,
            coverage_entries: MAX_COVERAGE_ENTRIES + 1,
            exclusion_entries: 0,
        };
        assert_eq!(
            validate_snapshot_at_acceptance(&oversized, 5),
            Err(EnrollmentError::TooManyEntries)
        );
    }

    #[test]
    fn waiting_period_and_service_date_boundaries_are_enforced() {
        let waiting_end = validate_condition_terms(100, 20, 365, 1_000, 2_500, 1_000).unwrap();
        assert_eq!(waiting_end, 120);
        assert_eq!(validate_service_date(100, waiting_end, 119, 1_000, 200), Err(BenefitError::WaitingPeriod));
        assert_eq!(validate_service_date(100, waiting_end, 120, 1_000, 200), Ok(()));
        assert_eq!(validate_service_date(100, waiting_end, 99, 1_000, 200), Err(BenefitError::InvalidServiceDate));
        assert_eq!(validate_service_date(100, waiting_end, 201, 1_000, 200), Err(BenefitError::InvalidServiceDate));
        assert_eq!(validate_service_date(100, waiting_end, 1_000, 1_000, 2_000), Err(BenefitError::InvalidServiceDate));
    }

    #[test]
    fn invalid_and_overflowing_condition_terms_are_rejected() {
        assert_eq!(validate_condition_terms(1, 0, 0, 1, 0, 100), Err(BenefitError::InvalidTerms));
        assert_eq!(validate_condition_terms(1, 0, 1, 1, 10_001, 100), Err(BenefitError::InvalidTerms));
        assert_eq!(validate_condition_terms(u64::MAX, 1, 1, 1, 0, u64::MAX), Err(BenefitError::ArithmeticOverflow));
        assert_eq!(validate_condition_terms(10, 90, 1, 1, 0, 100), Err(BenefitError::InvalidTerms));
    }

    #[test]
    fn deductible_copay_rounding_and_amount_boundaries_are_deterministic() {
        assert_eq!(calculate_benefit_settlement(0, 0, 0), Err(BenefitError::ZeroAmount));
        assert_eq!(calculate_benefit_settlement(101, 1, 3_333), Ok(BenefitSettlement {
            deductible_applied: 1,
            copay: 33,
            insurer_payment: 67,
        }));
        assert_eq!(calculate_benefit_settlement(u64::MAX, u64::MAX, 10_000), Ok(BenefitSettlement {
            deductible_applied: u64::MAX,
            copay: 0,
            insurer_payment: 0,
        }));
        assert_eq!(calculate_benefit_settlement(u64::MAX, 0, 10_000), Ok(BenefitSettlement {
            deductible_applied: 0,
            copay: u64::MAX,
            insurer_payment: 0,
        }));
    }

    #[test]
    fn condition_reservations_enforce_limits_and_period_reset_boundaries() {
        let first = reserve_condition_claim(None, 100, 100, 60).unwrap();
        assert_eq!(first, ConditionUsage { period_start: 100, paid: 0, pending: 60 });
        assert_eq!(reserve_condition_claim(Some(first), 100, 100, 41), Err(BenefitError::CoverageLimitExceeded));
        assert_eq!(reserve_condition_claim(Some(first), 101, 100, 1), Ok(ConditionUsage {
            period_start: 101,
            paid: 0,
            pending: 1,
        }));
        assert_eq!(finalize_condition_claim(first, 60, true, 100).unwrap(), ConditionUsage {
            period_start: 100,
            paid: 60,
            pending: 0,
        });
        assert_eq!(finalize_condition_claim(first, 60, false, 100).unwrap(), ConditionUsage {
            period_start: 100,
            paid: 0,
            pending: 0,
        });
        assert_eq!(reserve_condition_claim(Some(ConditionUsage {
            period_start: 100,
            paid: 100,
            pending: 0,
        }), 101, 100, 100).unwrap(), ConditionUsage {
            period_start: 101,
            paid: 0,
            pending: 100,
        });
    }
}
