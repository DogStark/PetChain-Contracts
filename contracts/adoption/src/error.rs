use soroban_sdk::contracterror;

/// Error codes returned by the adoption contract.
///
/// Codes are grouped by concern so that off-chain clients can map a numeric
/// code back to a stable category. New codes are appended to their group to
/// preserve the existing ABI for already-deployed callers.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // --- Generic / lifecycle (1-9) ---
    /// The contract has not been initialized yet.
    NotInitialized = 1,
    /// The contract has already been initialized.
    AlreadyInitialized = 2,
    /// The caller is not authorized to perform this action.
    Unauthorized = 3,
    /// The requested resource does not exist.
    NotFound = 4,
    /// The operation is not valid for the current state.
    InvalidState = 5,

    // --- Application state machine (10-19) ---
    /// The application is already in a terminal state and cannot transition.
    ApplicationTerminal = 10,
    /// The requested state transition is not allowed from the current state.
    InvalidTransition = 11,
    /// The caller does not hold the role required for this transition.
    RoleNotPermitted = 12,
    /// The application has already been submitted.
    ApplicationAlreadySubmitted = 13,
    /// The application has been withdrawn and can no longer be acted upon.
    ApplicationWithdrawn = 14,

    // --- Input bounds (20-29) ---
    /// A supplied string or byte input exceeds the allowed maximum length.
    InputTooLong = 20,
    /// A numeric input is outside the permitted range.
    InvalidInput = 21,
}
