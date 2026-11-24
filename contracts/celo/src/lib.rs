use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Bytes, Env, Symbol};

/// Maximum length (in bytes) accepted for a certificate issuer identifier.
const MAX_ISSUER_LEN: u32 = 64;
/// Maximum length (in bytes) accepted for a certificate identifier.
const MAX_CERT_ID_LEN: u32 = 64;
/// Current storage schema version for certificate records.
const CERT_STORAGE_VERSION: u32 = 1;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CeloError {
    /// Caller is not authorized to perform the operation.
    Unauthorized = 1,
    /// Issuer identifier exceeds `MAX_ISSUER_LEN`.
    IssuerTooLong = 2,
    /// Certificate identifier exceeds `MAX_CERT_ID_LEN`.
    CertIdTooLong = 3,
    /// Validity window is invalid (end <= start).
    InvalidValidity = 4,
    /// Certificate does not exist.
    NotFound = 5,
    /// Certificate has been revoked.
    Revoked = 6,
    /// Certificate is not yet valid or has expired.
    NotValid = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub version: u32,
    pub issuer: Bytes,
    pub valid_from: u64,
    pub valid_until: u64,
    pub revoked: bool,
    pub revoked_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerificationResult {
    pub issuer: Bytes,
    pub valid: bool,
    pub revoked: bool,
    pub valid_from: u64,
    pub valid_until: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Certificate(Bytes),
}

#[contract]
pub struct CeloContract;

#[contractimpl]
impl CeloContract {
    /// Initialize the contract with an admin authorized to issue/revoke certificates.
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// Issue (or overwrite) a vaccination certificate. Only the admin may call this.
    pub fn issue_certificate(
        env: Env,
        issuer: Address,
        cert_id: Bytes,
        valid_from: u64,
        valid_until: u64,
    ) -> Result<(), CeloError> {
        issuer.require_auth();
        Self::require_admin(&env, &issuer)?;

        if cert_id.len() > MAX_CERT_ID_LEN {
            return Err(CeloError::CertIdTooLong);
        }
        if valid_until <= valid_from {
            return Err(CeloError::InvalidValidity);
        }

        let record = Certificate {
            version: CERT_STORAGE_VERSION,
            issuer: cert_id.clone(),
            valid_from,
            valid_until,
            revoked: false,
            revoked_at: 0,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Certificate(cert_id), &record);
        Ok(())
    }

    /// Revoke a certificate. Only the admin may call this.
    pub fn revoke_certificate(
        env: Env,
        issuer: Address,
        cert_id: Bytes,
    ) -> Result<(), CeloError> {
        issuer.require_auth();
        Self::require_admin(&env, &issuer)?;

        if cert_id.len() > MAX_CERT_ID_LEN {
            return Err(CeloError::CertIdTooLong);
        }

        let key = DataKey::Certificate(cert_id);
        let mut record: Certificate = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(CeloError::NotFound)?;

        if record.revoked {
            return Err(CeloError::Revoked);
        }

        record.revoked = true;
        record.revoked_at = env.ledger().timestamp();
        env.storage().persistent().set(&key, &record);
        Ok(())
    }

    /// Read-only verification matching Soroban client semantics.
    pub fn verify_certificate(env: Env, cert_id: Bytes) -> Result<VerificationResult, CeloError> {
        if cert_id.len() > MAX_CERT_ID_LEN {
            return Err(CeloError::CertIdTooLong);
        }

        let record: Certificate = env
            .storage()
            .persistent()
            .get(&DataKey::Certificate(cert_id))
            .ok_or(CeloError::NotFound)?;

        let now = env.ledger().timestamp();
        let within_window = now >= record.valid_from && now <= record.valid_until;
        let valid = !record.revoked && within_window;

        Ok(VerificationResult {
            issuer: record.issuer,
            valid,
            revoked: record.revoked,
            valid_from: record.valid_from,
            valid_until: record.valid_until,
        })
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), CeloError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(CeloError::Unauthorized)?;
        if &admin != caller {
            return Err(CeloError::Unauthorized);
        }
        Ok(())
    }
}
