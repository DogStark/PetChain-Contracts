use soroban_sdk::contracterror;

/// Error codes returned by the Celo vaccination certificate contract.
///
/// New codes are appended so existing numeric values remain stable for
/// off-chain clients and compatibility fixtures.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The contract has already been initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,
    /// The caller is not authorized to perform this action.
    Unauthorized = 3,
    /// The requested certificate does not exist.
    CertificateNotFound = 4,
    /// The certificate has been revoked and can no longer be verified.
    CertificateRevoked = 5,
    /// The certificate is not yet valid (current ledger time is before `valid_from`).
    CertificateNotYetValid = 6,
    /// The certificate has expired (current ledger time is at or after `valid_until`).
    CertificateExpired = 7,
    /// The supplied issuer identifier exceeds the maximum allowed length.
    IssuerTooLong = 8,
    /// The supplied certificate identifier exceeds the maximum allowed length.
    CertificateIdTooLong = 9,
    /// The supplied issuer identifier is empty.
    IssuerEmpty = 10,
    /// The supplied certificate identifier is empty.
    CertificateIdEmpty = 11,
    /// The validity window is invalid (`valid_from` must be before `valid_until`).
    InvalidValidityWindow = 12,
    /// A certificate with the same identifier already exists.
    CertificateAlreadyExists = 13,
}
