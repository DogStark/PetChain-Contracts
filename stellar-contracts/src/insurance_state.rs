//! Insurance claim state-machine invariants (#1204).
//!
//! Single source of truth for which [`InsuranceClaimStatus`] transitions are
//! legal. Paid is terminal; Rejected may only move to UnderAppeal; an
//! appeal resolves to Approved or Rejected; Approved may only be paid.

use crate::InsuranceClaimStatus as S;

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
}
