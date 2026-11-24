use soroban_sdk::contracterror;

/// Error codes returned by the medical records contract.
///
/// Codes are stable and must not be renumbered; new variants are appended.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MedicalRecordError {
    /// Caller is not authorized to perform this action.
    Unauthorized = 1,
    /// The referenced medical record does not exist.
    RecordNotFound = 2,
    /// The referenced medical record has already been soft-deleted.
    RecordAlreadyDeleted = 3,
    /// The provided vet address is not registered/authorized.
    InvalidVet = 4,
    /// Input validation failed (empty fields, bad ranges, etc.).
    InvalidInput = 5,
    /// The record is referenced by an active insurance claim and cannot be
    /// soft-deleted until that claim is resolved (Accepted or Declined).
    RecordReferencedByActiveClaim = 6,
}
