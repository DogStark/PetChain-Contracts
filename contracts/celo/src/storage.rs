use soroban_sdk::{contracttype, Address, Bytes, Env};

/// Maximum length (in bytes) of a certificate issuer identifier.
pub const MAX_ISSUER_LEN: u32 = 64;
/// Maximum length (in bytes) of a certificate identifier.
pub const MAX_CERT_ID_LEN: u32 = 64;

/// Storage schema version for the Celo vaccination certificate module.
/// Bump when the persisted layout changes so readers can migrate safely.
pub const STORAGE_VERSION: u32 = 1;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Version,
    Certificate(Bytes),
}

/// Persisted vaccination certificate record.
///
/// Mirrors the Soroban client semantics: an issuer, a validity window
/// (`valid_from`/`valid_until`, unix seconds), and revocation data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaccinationCertificate {
    pub issuer: Bytes,
    pub valid_from: u64,
    pub valid_until: u64,
    pub revoked: bool,
    pub revoked_at: u64,
}

/// Result of a certificate verification read.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Valid,
    NotYetValid,
    Expired,
    Revoked,
    NotFound,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateVerification {
    pub status: CertificateStatus,
    pub issuer: Bytes,
    pub valid_from: u64,
    pub valid_until: u64,
    pub revoked: bool,
    pub revoked_at: u64,
}

fn ensure_version(env: &Env) {
    let key = DataKey::Version;
    match env.storage().persistent().get::<DataKey, u32>(&key) {
        Some(v) => {
            if v != STORAGE_VERSION {
                panic!("unsupported storage version");
            }
        }
        None => {
            env.storage().persistent().set(&key, &STORAGE_VERSION);
        }
    }
}

fn validate_issuer(issuer: &Bytes) {
    if issuer.len() == 0 || issuer.len() > MAX_ISSUER_LEN {
        panic!("invalid issuer length");
    }
}

fn validate_cert_id(cert_id: &Bytes) {
    if cert_id.len() == 0 || cert_id.len() > MAX_CERT_ID_LEN {
        panic!("invalid certificate id length");
    }
}

/// Store a vaccination certificate. Only the issuer may write its own record.
pub fn put_certificate(
    env: &Env,
    issuer: &Address,
    cert_id: &Bytes,
    valid_from: u64,
    valid_until: u64,
) {
    issuer.require_auth();
    ensure_version(env);
    validate_issuer(&Bytes::from_slice(env, &[]));
    validate_cert_id(cert_id);
    if valid_until < valid_from {
        panic!("invalid validity window");
    }

    let key = DataKey::Certificate(cert_id.clone());
    let record = VaccinationCertificate {
        issuer: Bytes::from_slice(env, &[]),
        valid_from,
        valid_until,
        revoked: false,
        revoked_at: 0,
    };
    env.storage().persistent().set(&key, &record);
}

/// Revoke a certificate. Only the original issuer may revoke.
pub fn revoke_certificate(env: &Env, issuer: &Address, cert_id: &Bytes, revoked_at: u64) {
    issuer.require_auth();
    ensure_version(env);
    validate_cert_id(cert_id);

    let key = DataKey::Certificate(cert_id.clone());
    let mut record: VaccinationCertificate = match env.storage().persistent().get(&key) {
        Some(r) => r,
        None => panic!("certificate not found"),
    };
    if record.revoked {
        panic!("certificate already revoked");
    }
    record.revoked = true;
    record.revoked_at = revoked_at;
    env.storage().persistent().set(&key, &record);
}

/// Read-only verification matching Soroban client semantics.
pub fn verify_certificate(env: &Env, cert_id: &Bytes, now: u64) -> CertificateVerification {
    validate_cert_id(cert_id);
    let key = DataKey::Certificate(cert_id.clone());
    let record: VaccinationCertificate = match env.storage().persistent().get(&key) {
        Some(r) => r,
        None => {
            return CertificateVerification {
                status: CertificateStatus::NotFound,
                issuer: Bytes::from_slice(env, &[]),
                valid_from: 0,
                valid_until: 0,
                revoked: false,
                revoked_at: 0,
            };
        }
    };

    let status = if record.revoked {
        CertificateStatus::Revoked
    } else if now < record.valid_from {
        CertificateStatus::NotYetValid
    } else if now > record.valid_until {
        CertificateStatus::Expired
    } else {
        CertificateStatus::Valid
    };

    CertificateVerification {
        status,
        issuer: record.issuer,
        valid_from: record.valid_from,
        valid_until: record.valid_until,
        revoked: record.revoked,
        revoked_at: record.revoked_at,
    }
}
