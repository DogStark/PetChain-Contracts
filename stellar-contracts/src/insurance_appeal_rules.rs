//! Appeal window and single-active-appeal rules (#1208).
//!
//! Assumption: `appeal_claim` is not present in `lib.rs` on `main`; this
//! module provides the state machine it must enforce.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppealState {
    None,
    Active,
    /// Terminal: no further appeal may be filed.
    Resolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppealError {
    ClaimNotRejected,
    WindowExpired,
    AppealAlreadyActive,
    AlreadyResolved,
    NotActive,
}

/// Checks whether an appeal may be filed. The window is inclusive:
/// `now <= rejected_at + window`.
pub fn can_file_appeal(
    claim_rejected: bool,
    rejected_at: u64,
    window: u64,
    now: u64,
    state: AppealState,
) -> Result<(), AppealError> {
    if !claim_rejected {
        return Err(AppealError::ClaimNotRejected);
    }
    match state {
        AppealState::Active => return Err(AppealError::AppealAlreadyActive),
        AppealState::Resolved => return Err(AppealError::AlreadyResolved),
        AppealState::None => {}
    }
    if now > rejected_at.saturating_add(window) {
        return Err(AppealError::WindowExpired);
    }
    Ok(())
}

/// Resolves an active appeal into the terminal state.
pub fn resolve_appeal(state: AppealState) -> Result<AppealState, AppealError> {
    match state {
        AppealState::Active => Ok(AppealState::Resolved),
        AppealState::Resolved => Err(AppealError::AlreadyResolved),
        AppealState::None => Err(AppealError::NotActive),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_boundary_is_inclusive() {
        assert_eq!(can_file_appeal(true, 100, 50, 150, AppealState::None), Ok(()));
        assert_eq!(
            can_file_appeal(true, 100, 50, 151, AppealState::None),
            Err(AppealError::WindowExpired)
        );
    }

    #[test]
    fn only_rejected_claims_can_be_appealed() {
        assert_eq!(
            can_file_appeal(false, 0, 10, 1, AppealState::None),
            Err(AppealError::ClaimNotRejected)
        );
    }

    #[test]
    fn duplicate_active_appeal_rejected() {
        assert_eq!(
            can_file_appeal(true, 0, 10, 1, AppealState::Active),
            Err(AppealError::AppealAlreadyActive)
        );
    }

    #[test]
    fn resolution_is_terminal_and_idempotent_failure() {
        let s = resolve_appeal(AppealState::Active).unwrap();
        assert_eq!(s, AppealState::Resolved);
        assert_eq!(resolve_appeal(s), Err(AppealError::AlreadyResolved));
        assert_eq!(
            can_file_appeal(true, 0, 10, 1, s),
            Err(AppealError::AlreadyResolved)
        );
        assert_eq!(resolve_appeal(AppealState::None), Err(AppealError::NotActive));
    }

    #[test]
    fn huge_window_does_not_overflow() {
        assert_eq!(can_file_appeal(true, u64::MAX, u64::MAX, u64::MAX, AppealState::None), Ok(()));
    }
}
