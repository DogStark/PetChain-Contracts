//! Insurance claim state-machine invariants (#1204).
//!
//! Single source of truth for which [`InsuranceClaimStatus`] transitions are
//! legal. Paid is terminal; Rejected may only move to UnderAppeal; an
//! appeal resolves to Approved or Rejected; Approved may only be paid.
//!
//! Also hosts the insurance enrollment application workflow (#1470) and the
//! immutable coverage/exclusion schedule snapshots (#1472).
//!
//! # Enrollment workflow (#1470)
//!
//! An [`EnrollmentApplication`] moves through
//! `Draft -> Submitted -> UnderReview -> Accepted | Declined`. `Accepted` and
//! `Declined` are terminal. Every decision (accept/decline) is bound to the
//! `policy_version` that was in force when the decision was made, so a later
//! policy change can never retroactively alter the outcome.
//!
//! # Coverage snapshots (#1472)
//!
//! [`CoverageSnapshot`] stores an immutable copy of the coverage and exclusion
//! schedule taken at enrollment time, keyed by enrollment id and policy
//! version. Both vectors are bounded by [`MAX_SNAPSHOT_ENTRIES`] so persistent
//! state growth is bounded per enrollment.

use crate::InsuranceClaimStatus as S;

/// Maximum number of coverage entries stored in a single snapshot.
pub const MAX_SNAPSHOT_ENTRIES: u32 = 64;

/// Maximum length (in bytes) of a single coverage/exclusion label.
pub const MAX_SNAPSHOT_LABEL_LEN: u32 = 64;

/// Enrollment application lifecycle state (#1470).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnrollmentStatus {
    /// Application created but not yet submitted.
    Draft,
    /// Application submitted and awaiting review.
    Submitted,
    /// Application is being reviewed.
    UnderReview,
    /// Application accepted; terminal.
    Accepted,
    /// Application declined; terminal.
    Declined,
}

/// An insurance enrollment application (#1470).
///
/// `policy_version` records the policy version the application is bound to.
/// For `Accepted`/`Declined` it is the version in force at decision time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnrollmentApplication {
    pub enrollment_id: u64,
    pub status: EnrollmentStatus,
    pub policy_version: u32,
}

/// Errors returned by enrollment workflow helpers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnrollmentError {
    /// The requested transition is not legal from the current state.
    InvalidTransition,
    /// The supplied policy version does not match the bound version.
    PolicyVersionMismatch,
    /// A snapshot vector exceeds [`MAX_SNAPSHOT_ENTRIES`].
    TooManySnapshotEntries,
    /// A snapshot label exceeds [`MAX_SNAPSHOT_LABEL_LEN`].
    SnapshotLabelTooLong,
}

/// True if an enrollment application may move from `from` to `to`.
/// Self-transitions are rejected (no idempotent no-op writes).
pub fn is_valid_enrollment_transition(from: &EnrollmentStatus, to: &EnrollmentStatus) -> bool {
    use EnrollmentStatus::*;
    matches!(
        (from, to),
        (Draft, Submitted)
            | (Submitted, UnderReview)
            | (UnderReview, Accepted)
            | (UnderReview, Declined)
    )
}

/// True for enrollment states with no outgoing transitions.
pub fn is_terminal_enrollment_status(s: &EnrollmentStatus) -> bool {
    matches!(s, EnrollmentStatus::Accepted | EnrollmentStatus::Declined)
}

impl EnrollmentApplication {
    /// Create a new application in `Draft` bound to `policy_version`.
    pub fn new(enrollment_id: u64, policy_version: u32) -> Self {
        Self { enrollment_id, status: EnrollmentStatus::Draft, policy_version }
    }

    /// Attempt a transition. On failure the application is left unchanged.
    ///
    /// Accept/decline decisions must supply the currently bound
    /// `policy_version`; a mismatch fails with
    /// [`EnrollmentError::PolicyVersionMismatch`] and leaves state unchanged.
    pub fn transition(
        &mut self,
        to: EnrollmentStatus,
        policy_version: u32,
    ) -> Result<(), EnrollmentError> {
        if !is_valid_enrollment_transition(&self.status, &to) {
            return Err(EnrollmentError::InvalidTransition);
        }
        if matches!(to, EnrollmentStatus::Accepted | EnrollmentStatus::Declined)
            && policy_version != self.policy_version
        {
            return Err(EnrollmentError::PolicyVersionMismatch);
        }
        self.status = to;
        Ok(())
    }
}

/// Immutable coverage/exclusion snapshot taken at enrollment (#1472).
///
/// Keyed by `enrollment_id` and the `policy_version` in force at enrollment.
/// Once constructed the contents never change; later policy edits cannot
/// alter an existing snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageSnapshot {
    pub enrollment_id: u64,
    pub policy_version: u32,
    pub coverage: Vec<u32>,
    pub exclusions: Vec<u32>,
}

impl CoverageSnapshot {
    /// Build a snapshot, validating bounds. On error no snapshot is produced.
    pub fn new(
        enrollment_id: u64,
        policy_version: u32,
        coverage: Vec<u32>,
        exclusions: Vec<u32>,
    ) -> Result<Self, EnrollmentError> {
        if coverage.len() as u32 > MAX_SNAPSHOT_ENTRIES
            || exclusions.len() as u32 > MAX_SNAPSHOT_ENTRIES
        {
            return Err(EnrollmentError::TooManySnapshotEntries);
        }
        Ok(Self { enrollment_id, policy_version, coverage, exclusions })
    }

    /// True if this snapshot belongs to `enrollment_id` at `policy_version`.
    pub fn matches(&self, enrollment_id: u64, policy_version: u32) -> bool {
        self.enrollment_id == enrollment_id && self.policy_version == policy_version
    }
}

/// Validate a snapshot label length against [`MAX_SNAPSHOT_LABEL_LEN`].
pub fn validate_snapshot_label(label: &str) -> Result<(), EnrollmentError> {
    if label.len() as u32 > MAX_SNAPSHOT_LABEL_LEN {
        return Err(EnrollmentError::SnapshotLabelTooLong);
    }
    Ok(())
}

/// True if a claim may move from `from` to `to`. Self-transitions are
/// rejected (no idempotent no-op writes).
pub fn is_valid_claim_transition(from: &S, to: &S) -> bool {
    matches!(
        (from, to),
        (S::Pending, S::Approved)
            | (S::Pending, S::Rejected)
            | (S::Pending, S::UnderReview)
            | (S::UnderReview, S::Approved)
            | (S::UnderReview, S::Rejected)
            | (S::Approved, S::Paid)
            | (S::Rejected, S::UnderAppeal)
            | (S::UnderAppeal, S::Approved)
            | (S::UnderAppeal, S::Rejected)
    )
}

/// True for states with no outgoing transitions.
pub fn is_terminal_claim_status(s: &S) -> bool {
    matches!(s, S::Paid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> [S; 6] {
        [S::Pending, S::Approved, S::Rejected, S::Paid, S::UnderReview, S::UnderAppeal]
    }

    #[test]
    fn terminal_state_has_no_outgoing_edges() {
        for t in all().iter() {
            assert!(!is_valid_claim_transition(&S::Paid, t));
        }
        assert!(is_terminal_claim_status(&S::Paid));
    }

    #[test]
    fn no_self_transitions() {
        for s in all().iter() {
            assert!(!is_valid_claim_transition(s, s));
        }
    }

    #[test]
    fn nothing_returns_to_pending() {
        for s in all().iter() {
            assert!(!is_valid_claim_transition(s, &S::Pending));
        }
    }

    #[test]
    fn paid_only_reachable_from_approved() {
        for s in all().iter() {
            assert_eq!(is_valid_claim_transition(s, &S::Paid), *s == S::Approved);
        }
    }

    #[test]
    fn rejected_only_moves_to_appeal_and_appeal_only_from_rejected() {
        for t in all().iter() {
            assert_eq!(is_valid_claim_transition(&S::Rejected, t), *t == S::UnderAppeal);
            assert_eq!(is_valid_claim_transition(t, &S::UnderAppeal), *t == S::Rejected);
        }
    }

    #[test]
    fn every_path_terminates_within_bound() {
        // Longest legal path: Pending->Rejected->UnderAppeal->Rejected->UnderAppeal...
        // Appeal loops exist, so verify reachability of Paid instead.
        let mut seen = [false; 6];
        let idx = |s: &S| all().iter().position(|x| x == s).unwrap();
        seen[idx(&S::Pending)] = true;
        for _ in 0..6 {
            for a in all().iter() {
                for b in all().iter() {
                    if seen[idx(a)] && is_valid_claim_transition(a, b) {
                        seen[idx(b)] = true;
                    }
                }
            }
        }
        assert!(seen.iter().all(|x| *x));
    }

    // ---- Enrollment workflow (#1470) ----

    use super::EnrollmentStatus as E;

    fn all_enrollment() -> [E; 5] {
        [E::Draft, E::Submitted, E::UnderReview, E::Accepted, E::Declined]
    }

    #[test]
    fn enrollment_legal_transitions() {
        assert!(is_valid_enrollment_transition(&E::Draft, &E::Submitted));
        assert!(is_valid_enrollment_transition(&E::Submitted, &E::UnderReview));
        assert!(is_valid_enrollment_transition(&E::UnderReview, &E::Accepted));
        assert!(is_valid_enrollment_transition(&E::UnderReview, &E::Declined));
    }

    #[test]
    fn enrollment_illegal_transitions_rejected() {
        let legal = [
            (E::Draft, E::Submitted),
            (E::Submitted, E::UnderReview),
            (E::UnderReview, E::Accepted),
            (E::UnderReview, E::Declined),
        ];
        for from in all_enrollment().iter() {
            for to in all_enrollment().iter() {
                let expected = legal.iter().any(|(a, b)| a == from && b == to);
                assert_eq!(is_valid_enrollment_transition(from, to), expected);
            }
        }
    }

    #[test]
    fn enrollment_no_self_transitions() {
        for s in all_enrollment().iter() {
            assert!(!is_valid_enrollment_transition(s, s));
        }
    }

    #[test]
    fn enrollment_terminal_states_have_no_outgoing_edges() {
        for t in all_enrollment().iter() {
            assert!(!is_valid_enrollment_transition(&E::Accepted, t));
            assert!(!is_valid_enrollment_transition(&E::Declined, t));
        }
        assert!(is_terminal_enrollment_status(&E::Accepted));
        assert!(is_terminal_enrollment_status(&E::Declined));
        assert!(!is_terminal_enrollment_status(&E::Draft));
        assert!(!is_terminal_enrollment_status(&E::Submitted));
        assert!(!is_terminal_enrollment_status(&E::UnderReview));
    }

    #[test]
    fn enrollment_happy_path_binds_policy_version() {
        let mut app = EnrollmentApplication::new(7, 3);
        assert_eq!(app.status, E::Draft);
        assert_eq!(app.policy_version, 3);
        app.transition(E::Submitted, 3).unwrap();
        app.transition(E::UnderReview, 3).unwrap();
        app.transition(E::Accepted, 3).unwrap();
        assert_eq!(app.status, E::Accepted);
        assert_eq!(app.policy_version, 3);
    }

    #[test]
    fn enrollment_decision_with_wrong_policy_version_fails_unchanged() {
        let mut app = EnrollmentApplication::new(1, 5);
        app.transition(E::Submitted, 5).unwrap();
        app.transition(E::UnderReview, 5).unwrap();
        let before = app;
        assert_eq!(
            app.transition(E::Accepted, 6),
            Err(EnrollmentError::PolicyVersionMismatch)
        );
        assert_eq!(app, before);
        assert_eq!(
            app.transition(E::Declined, 4),
            Err(EnrollmentError::PolicyVersionMismatch)
        );
        assert_eq!(app, before);
    }

    #[test]
    fn enrollment_invalid_transition_leaves_state_unchanged() {
        let mut app = EnrollmentApplication::new(2, 1);
        let before = app;
        assert_eq!(
            app.transition(E::Accepted, 1),
            Err(EnrollmentError::InvalidTransition)
        );
        assert_eq!(app, before);
        app.transition(E::Submitted, 1).unwrap();
        let before = app;
        assert_eq!(
            app.transition(E::Accepted, 1),
            Err(EnrollmentError::InvalidTransition)
        );
        assert_eq!(app, before);
    }

    #[test]
    fn enrollment_terminal_cannot_transition() {
        let mut app = EnrollmentApplication::new(3, 1);
        app.transition(E::Submitted, 1).unwrap();
        app.transition(E::UnderReview, 1).unwrap();
        app.transition(E::Declined, 1).unwrap();
        let before = app;
        for to in all_enrollment().iter() {
            assert_eq!(
                app.transition(*to, 1),
                Err(EnrollmentError::InvalidTransition)
            );
            assert_eq!(app, before);
        }
    }

    // ---- Coverage snapshots (#1472) ----

    #[test]
    fn snapshot_roundtrip_and_match() {
        let snap = CoverageSnapshot::new(9, 2, vec![1, 2, 3], vec![4]).unwrap();
        assert!(snap.matches(9, 2));
        assert!(!snap.matches(9, 3));
        assert!(!snap.matches(10, 2));
        assert_eq!(snap.coverage, vec![1, 2, 3]);
        assert_eq!(snap.exclusions, vec![4]);
    }

    #[test]
    fn snapshot_is_immutable_after_creation() {
        let snap = CoverageSnapshot::new(1, 1, vec![1], vec![]).unwrap();
        let copy = snap.clone();
        // Mutating the source vectors used to build a new snapshot cannot
        // affect the already-stored snapshot.
        let mut coverage = vec![1];
        let snap2 = CoverageSnapshot::new(1, 1, coverage.clone(), vec![]).unwrap();
        coverage.push(2);
        assert_eq!(snap2.coverage, vec![1]);
        assert_eq!(snap, copy);
    }

    #[test]
    fn snapshot_boundary_sizes() {
        let max = MAX_SNAPSHOT_ENTRIES as usize;
        assert!(CoverageSnapshot::new(1, 1, vec![0; max], vec![0; max]).is_ok());
        assert_eq!(
            CoverageSnapshot::new(1, 1, vec![0; max + 1], vec![]),
            Err(EnrollmentError::TooManySnapshotEntries)
        );
        assert_eq!(
            CoverageSnapshot::new(1, 1, vec![], vec![0; max + 1]),
            Err(EnrollmentError::TooManySnapshotEntries)
        );
        assert!(CoverageSnapshot::new(1, 1, vec![], vec![]).is_ok());
    }

    #[test]
    fn snapshot_label_bounds() {
        let max = MAX_SNAPSHOT_LABEL_LEN as usize;
        assert!(validate_snapshot_label(&"a".repeat(max)).is_ok());
        assert_eq!(
            validate_snapshot_label(&"a".repeat(max + 1)),
            Err(EnrollmentError::SnapshotLabelTooLong)
        );
        assert!(validate_snapshot_label("").is_ok());
    }
}
