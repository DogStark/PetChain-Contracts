//! Checked-arithmetic reserve ledger for insurance flows (#1206).
//!
//! Assumption: `lib.rs` on `main` has no claim-payout entry points, so the flows
//! are modelled here as a pure ledger. Invariant:
//! `reserve == premiums + fees_in - fees_out - refunds - payouts`.

/// Reserve accounting state. All amounts are in the smallest asset unit.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReserveLedger {
    pub reserve: u64,
    pub premiums: u64,
    pub fees_in: u64,
    pub fees_out: u64,
    pub refunds: u64,
    pub payouts: u64,
}

/// Errors from ledger operations; the ledger is unchanged on error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LedgerError {
    Overflow,
    InsufficientReserve,
}

/// Terminal outcome of an enrollment decision, as seen by the ledger.
///
/// The ledger only distinguishes the two terminal decisions; draft/submitted/
/// under-review states never touch the reserve and are handled by the
/// enrollment state machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnrollmentDecision {
    /// Enrollment accepted: the premium is retained in the reserve.
    Accepted,
    /// Enrollment declined: the premium is refunded out of the reserve.
    Declined,
}

/// Errors from enrollment-decision accounting; the ledger is unchanged on error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionError {
    /// The decision amount would overflow a ledger counter.
    Overflow,
    /// The reserve cannot cover the refund for a declined enrollment.
    InsufficientReserve,
    /// The decision amount is zero, which is not a meaningful enrollment.
    ZeroAmount,
}

impl From<LedgerError> for DecisionError {
    fn from(err: LedgerError) -> Self {
        match err {
            LedgerError::Overflow => DecisionError::Overflow,
            LedgerError::InsufficientReserve => DecisionError::InsufficientReserve,
        }
    }
}

impl ReserveLedger {
    /// Checks the conservation invariant.
    pub fn is_conserved(&self) -> bool {
        let inflow = self.premiums.checked_add(self.fees_in);
        let outflow = self
            .fees_out
            .checked_add(self.refunds)
            .and_then(|v| v.checked_add(self.payouts));
        match (inflow, outflow) {
            (Some(i), Some(o)) => i.checked_sub(o) == Some(self.reserve),
            _ => false,
        }
    }

    /// Records a premium payment into the reserve.
    pub fn collect_premium(&mut self, amount: u64) -> Result<(), LedgerError> {
        let reserve = self.reserve.checked_add(amount).ok_or(LedgerError::Overflow)?;
        let premiums = self.premiums.checked_add(amount).ok_or(LedgerError::Overflow)?;
        self.reserve = reserve;
        self.premiums = premiums;
        Ok(())
    }

    /// Records a fee collected into the reserve.
    pub fn collect_fee(&mut self, amount: u64) -> Result<(), LedgerError> {
        let reserve = self.reserve.checked_add(amount).ok_or(LedgerError::Overflow)?;
        let fees = self.fees_in.checked_add(amount).ok_or(LedgerError::Overflow)?;
        self.reserve = reserve;
        self.fees_in = fees;
        Ok(())
    }

    /// Withdraws a fee from the reserve.
    pub fn withdraw_fee(&mut self, amount: u64) -> Result<(), LedgerError> {
        let reserve = self.reserve.checked_sub(amount).ok_or(LedgerError::InsufficientReserve)?;
        let fees = self.fees_out.checked_add(amount).ok_or(LedgerError::Overflow)?;
        self.reserve = reserve;
        self.fees_out = fees;
        Ok(())
    }

    /// Refunds a premium out of the reserve.
    pub fn refund(&mut self, amount: u64) -> Result<(), LedgerError> {
        let reserve = self.reserve.checked_sub(amount).ok_or(LedgerError::InsufficientReserve)?;
        let refunds = self.refunds.checked_add(amount).ok_or(LedgerError::Overflow)?;
        self.reserve = reserve;
        self.refunds = refunds;
        Ok(())
    }

    /// Pays a claim out of the reserve.
    pub fn pay_claim(&mut self, amount: u64) -> Result<(), LedgerError> {
        let reserve = self.reserve.checked_sub(amount).ok_or(LedgerError::InsufficientReserve)?;
        let payouts = self.payouts.checked_add(amount).ok_or(LedgerError::Overflow)?;
        self.reserve = reserve;
        self.payouts = payouts;
        Ok(())
    }

    /// Applies an enrollment decision to the ledger.
    ///
    /// `Accepted` keeps the already-collected premium in the reserve (no
    /// mutation). `Declined` refunds `amount` out of the reserve. All
    /// arithmetic is checked and the ledger is left untouched on any error.
    pub fn apply_enrollment_decision(
        &mut self,
        decision: EnrollmentDecision,
        amount: u64,
    ) -> Result<(), DecisionError> {
        if amount == 0 {
            return Err(DecisionError::ZeroAmount);
        }
        match decision {
            EnrollmentDecision::Accepted => Ok(()),
            EnrollmentDecision::Declined => {
                // Compute both new values before mutating so a failure in the
                // second step cannot leave a partially applied refund.
                let reserve = self
                    .reserve
                    .checked_sub(amount)
                    .ok_or(DecisionError::InsufficientReserve)?;
                let refunds = self
                    .refunds
                    .checked_add(amount)
                    .ok_or(DecisionError::Overflow)?;
                self.reserve = reserve;
                self.refunds = refunds;
                Ok(())
            }
        }
    }

    /// Runs `f` and asserts the ledger is unchanged when it fails.
    ///
    /// Used to guard enrollment transitions and coverage-snapshot writes: a
    /// failed transition must never leave the reserve partially mutated.
    pub fn with_unchanged_on_error<T, E, F>(&mut self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut Self) -> Result<T, E>,
    {
        let before = *self;
        match f(self) {
            Ok(value) => Ok(value),
            Err(err) => {
                debug_assert_eq!(*self, before, "ledger mutated on failed operation");
                *self = before;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_flow_is_conserved() {
        let mut l = ReserveLedger::default();
        l.collect_premium(1000).unwrap();
        l.collect_fee(50).unwrap();
        l.pay_claim(400).unwrap();
        l.refund(100).unwrap();
        l.withdraw_fee(50).unwrap();
        assert_eq!(l.reserve, 500);
        assert!(l.is_conserved());
    }

    #[test]
    fn exact_boundary_drains_reserve_to_zero() {
        let mut l = ReserveLedger::default();
        l.collect_premium(300).unwrap();
        l.pay_claim(300).unwrap();
        assert_eq!(l.reserve, 0);
        assert_eq!(l.pay_claim(1), Err(LedgerError::InsufficientReserve));
        assert!(l.is_conserved());
    }

    #[test]
    fn failed_ops_leave_state_unchanged() {
        let mut l = ReserveLedger::default();
        l.collect_premium(10).unwrap();
        let before = l;
        assert_eq!(l.refund(11), Err(LedgerError::InsufficientReserve));
        assert_eq!(l.withdraw_fee(11), Err(LedgerError::InsufficientReserve));
        assert_eq!(l, before);
    }

    #[test]
    fn overflow_is_rejected_without_mutation() {
        let mut l = ReserveLedger::default();
        l.collect_premium(u64::MAX).unwrap();
        let before = l;
        assert_eq!(l.collect_premium(1), Err(LedgerError::Overflow));
        assert_eq!(l.collect_fee(1), Err(LedgerError::Overflow));
        assert_eq!(l, before);
        assert!(l.is_conserved());
    }

    #[test]
    fn accepted_decision_keeps_reserve() {
        let mut l = ReserveLedger::default();
        l.collect_premium(500).unwrap();
        let before = l;
        l.apply_enrollment_decision(EnrollmentDecision::Accepted, 500).unwrap();
        assert_eq!(l, before);
        assert!(l.is_conserved());
    }

    #[test]
    fn declined_decision_refunds_premium() {
        let mut l = ReserveLedger::default();
        l.collect_premium(500).unwrap();
        l.apply_enrollment_decision(EnrollmentDecision::Declined, 500).unwrap();
        assert_eq!(l.reserve, 0);
        assert_eq!(l.refunds, 500);
        assert!(l.is_conserved());
    }

    #[test]
    fn declined_decision_boundary_and_failures_leave_state_unchanged() {
        let mut l = ReserveLedger::default();
        l.collect_premium(300).unwrap();

        // Boundary: refunding exactly the reserve succeeds.
        let mut exact = l;
        exact.apply_enrollment_decision(EnrollmentDecision::Declined, 300).unwrap();
        assert_eq!(exact.reserve, 0);

        // One unit over the reserve fails without mutation.
        let before = l;
        assert_eq!(
            l.apply_enrollment_decision(EnrollmentDecision::Declined, 301),
            Err(DecisionError::InsufficientReserve)
        );
        assert_eq!(l, before);

        // Zero amount is rejected for both decisions without mutation.
        assert_eq!(
            l.apply_enrollment_decision(EnrollmentDecision::Accepted, 0),
            Err(DecisionError::ZeroAmount)
        );
        assert_eq!(
            l.apply_enrollment_decision(EnrollmentDecision::Declined, 0),
            Err(DecisionError::ZeroAmount)
        );
        assert_eq!(l, before);
        assert!(l.is_conserved());
    }

    #[test]
    fn declined_decision_overflow_is_rejected_without_mutation() {
        let mut l = ReserveLedger::default();
        l.collect_premium(u64::MAX).unwrap();
        l.refund(u64::MAX).unwrap();
        // Reserve is now zero but refunds are saturated; a further refund must
        // fail on the refunds counter overflow, not on the reserve.
        l.collect_premium(10).unwrap();
        let before = l;
        assert_eq!(
            l.apply_enrollment_decision(EnrollmentDecision::Declined, 10),
            Err(DecisionError::Overflow)
        );
        assert_eq!(l, before);
        assert!(l.is_conserved());
    }

    #[test]
    fn replay_of_declined_decision_fails_without_mutation() {
        let mut l = ReserveLedger::default();
        l.collect_premium(100).unwrap();
        l.apply_enrollment_decision(EnrollmentDecision::Declined, 100).unwrap();
        let before = l;
        // Replaying the same decision must not refund twice.
        assert_eq!(
            l.apply_enrollment_decision(EnrollmentDecision::Declined, 100),
            Err(DecisionError::InsufficientReserve)
        );
        assert_eq!(l, before);
        assert!(l.is_conserved());
    }

    #[test]
    fn with_unchanged_on_error_restores_ledger() {
        let mut l = ReserveLedger::default();
        l.collect_premium(100).unwrap();
        let before = l;

        let result: Result<(), DecisionError> = l.with_unchanged_on_error(|ledger| {
            // Simulate a transition that mutates then fails.
            ledger.collect_premium(50).unwrap();
            Err(DecisionError::InsufficientReserve)
        });
        assert_eq!(result, Err(DecisionError::InsufficientReserve));
        assert_eq!(l, before);

        // Success path keeps the mutation.
        let result: Result<(), DecisionError> = l.with_unchanged_on_error(|ledger| {
            ledger.collect_premium(50).map_err(DecisionError::from)
        });
        assert_eq!(result, Ok(()));
        assert_eq!(l.reserve, 150);
        assert!(l.is_conserved());
    }

    #[test]
    fn pseudo_random_sequences_preserve_conservation() {
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut l = ReserveLedger::default();
        for _ in 0..5000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let amt = (seed >> 40) % 10_000;
            let _ = match (seed >> 33) % 5 {
                0 => l.collect_premium(amt),
                1 => l.collect_fee(amt),
                2 => l.withdraw_fee(amt),
                3 => l.refund(amt),
                _ => l.pay_claim(amt),
            };
            assert!(l.is_conserved());
        }
    }
}
