#![no_std]
#![allow(clippy::too_many_arguments)]

//! Pet-record schema compatibility fixtures & client decoder validator (Issue #1502).
//!
//! Validates and normalizes prior (`v1`) and current (`v2`) client-facing
//! payloads for `PetProfile`, `MedicalRecord`, `Consent`, and `Insurance`
//! schemas, applying canonical defaults when decoding `v1` fixtures so
//! generated client decoders stay forward- and backward-compatible.

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, String,
};

pub const STORAGE_SCHEMA_VERSION: u32 = 2;
pub const EVENT_SCHEMA_VERSION: u32 = 1;

pub const PRIOR_SCHEMA_V1: u32 = 1;
pub const CURRENT_SCHEMA_V2: u32 = 2;

pub const MAX_TEXT_FIELD_LEN: u32 = 128;
pub const MAX_COPAY_BPS: u32 = 10_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SchemaCompatError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    UnsupportedSchemaVersion = 4,
    FieldTooLong = 5,
    EmptyRequiredField = 6,
    InvalidCoverageTerms = 7,
    ExpiredConsentRecord = 8,
    ReplayDetected = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalPetProfileV2 {
    pub decoded_from_version: u32,
    pub pet_id: u64,
    pub name: String,
    pub species: String,
    pub microchip_id: String,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalMedicalRecordV2 {
    pub decoded_from_version: u32,
    pub record_id: u64,
    pub pet_id: u64,
    pub diagnosis: String,
    pub attachment_digest: String,
    pub soft_deleted: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalConsentV2 {
    pub decoded_from_version: u32,
    pub consent_id: u64,
    pub pet_id: u64,
    pub scope_code: String,
    pub policy_version: u32,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalInsuranceV2 {
    pub decoded_from_version: u32,
    pub policy_id: u64,
    pub pet_id: u64,
    pub coverage_limit: i128,
    pub deductible: i128,
    pub copay_bps: u32,
    pub active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaDecodedEvent {
    pub version: u32,
    pub domain_code: u32,
    pub source_schema_version: u32,
    pub entity_id: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    StorageVersion,
    Admin,
    CanonicalPet(u64),
    CanonicalMedical(u64),
    CanonicalConsent(u64),
    CanonicalInsurance(u64),
    UsedImportNonce((Address, u64)),
}

#[contract]
pub struct ClientSchemaCompatContract;

#[contractimpl]
impl ClientSchemaCompatContract {
    pub fn init(env: Env, admin: Address) -> Result<(), SchemaCompatError> {
        if env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(SchemaCompatError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::StorageVersion, &STORAGE_SCHEMA_VERSION);
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    /// Decode a `v1` or `v2` pet profile payload into `CanonicalPetProfileV2`.
    pub fn decode_pet_profile(
        env: Env,
        schema_version: u32,
        pet_id: u64,
        name: String,
        species: String,
        microchip_id: Option<String>,
        active: Option<bool>,
    ) -> Result<CanonicalPetProfileV2, SchemaCompatError> {
        Self::validate_version(schema_version)?;
        Self::validate_non_empty_text(&name)?;
        Self::validate_non_empty_text(&species)?;

        let normalized_chip = match (schema_version, microchip_id) {
            (PRIOR_SCHEMA_V1, None) => String::from_str(&env, "UNASSIGNED-V1"),
            (_, Some(chip)) => {
                Self::validate_non_empty_text(&chip)?;
                chip
            }
            (CURRENT_SCHEMA_V2, None) => return Err(SchemaCompatError::EmptyRequiredField),
            _ => return Err(SchemaCompatError::UnsupportedSchemaVersion),
        };

        Ok(CanonicalPetProfileV2 {
            decoded_from_version: schema_version,
            pet_id,
            name,
            species,
            microchip_id: normalized_chip,
            active: active.unwrap_or(true),
        })
    }

    /// Decode a `v1` or `v2` medical record payload into `CanonicalMedicalRecordV2`.
    pub fn decode_medical_record(
        env: Env,
        schema_version: u32,
        record_id: u64,
        pet_id: u64,
        diagnosis: String,
        attachment_digest: Option<String>,
        soft_deleted: Option<bool>,
    ) -> Result<CanonicalMedicalRecordV2, SchemaCompatError> {
        Self::validate_version(schema_version)?;
        Self::validate_non_empty_text(&diagnosis)?;

        let normalized_digest = match (schema_version, attachment_digest) {
            (PRIOR_SCHEMA_V1, None) => String::from_str(&env, "none"),
            (_, Some(d)) => {
                Self::validate_non_empty_text(&d)?;
                d
            }
            (CURRENT_SCHEMA_V2, None) => return Err(SchemaCompatError::EmptyRequiredField),
            _ => return Err(SchemaCompatError::UnsupportedSchemaVersion),
        };

        Ok(CanonicalMedicalRecordV2 {
            decoded_from_version: schema_version,
            record_id,
            pet_id,
            diagnosis,
            attachment_digest: normalized_digest,
            soft_deleted: soft_deleted.unwrap_or(false),
        })
    }

    /// Decode a `v1` or `v2` consent payload into `CanonicalConsentV2`.
    pub fn decode_consent(
        env: Env,
        schema_version: u32,
        consent_id: u64,
        pet_id: u64,
        scope_code: Option<String>,
        policy_version: Option<u32>,
        expires_at: u64,
    ) -> Result<CanonicalConsentV2, SchemaCompatError> {
        Self::validate_version(schema_version)?;
        if expires_at <= env.ledger().timestamp() {
            return Err(SchemaCompatError::ExpiredConsentRecord);
        }

        let normalized_scope = match (schema_version, scope_code) {
            (PRIOR_SCHEMA_V1, None) => String::from_str(&env, "LEGACY_READ_ONLY"),
            (_, Some(sc)) => {
                Self::validate_non_empty_text(&sc)?;
                sc
            }
            (CURRENT_SCHEMA_V2, None) => return Err(SchemaCompatError::EmptyRequiredField),
            _ => return Err(SchemaCompatError::UnsupportedSchemaVersion),
        };

        Ok(CanonicalConsentV2 {
            decoded_from_version: schema_version,
            consent_id,
            pet_id,
            scope_code: normalized_scope,
            policy_version: policy_version.unwrap_or(1),
            expires_at,
        })
    }

    /// Decode a `v1` or `v2` insurance policy payload into `CanonicalInsuranceV2`.
    pub fn decode_insurance(
        env: Env,
        schema_version: u32,
        policy_id: u64,
        pet_id: u64,
        coverage_limit: i128,
        deductible: Option<i128>,
        copay_bps: Option<u32>,
        active: Option<bool>,
    ) -> Result<CanonicalInsuranceV2, SchemaCompatError> {
        let _ = env;
        Self::validate_version(schema_version)?;
        if coverage_limit <= 0 {
            return Err(SchemaCompatError::InvalidCoverageTerms);
        }

        let norm_deductible = deductible.unwrap_or(0);
        let norm_copay = copay_bps.unwrap_or(0);
        if norm_deductible < 0 || norm_deductible > coverage_limit || norm_copay > MAX_COPAY_BPS {
            return Err(SchemaCompatError::InvalidCoverageTerms);
        }

        Ok(CanonicalInsuranceV2 {
            decoded_from_version: schema_version,
            policy_id,
            pet_id,
            coverage_limit,
            deductible: norm_deductible,
            copay_bps: norm_copay,
            active: active.unwrap_or(true),
        })
    }

    /// Persist a decoded canonical pet profile with replay and authorization guards.
    pub fn store_decoded_pet(
        env: Env,
        caller: Address,
        profile: CanonicalPetProfileV2,
        nonce: u64,
    ) -> Result<(), SchemaCompatError> {
        if !env.storage().instance().has(&DataKey::StorageVersion) {
            return Err(SchemaCompatError::NotInitialized);
        }
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SchemaCompatError::NotInitialized)?;
        if caller != admin {
            return Err(SchemaCompatError::Unauthorized);
        }

        let nonce_key = DataKey::UsedImportNonce((caller, nonce));
        if env.storage().persistent().has(&nonce_key) {
            return Err(SchemaCompatError::ReplayDetected);
        }

        env.storage()
            .persistent()
            .set(&DataKey::CanonicalPet(profile.pet_id), &profile);
        env.storage().persistent().set(&nonce_key, &true);

        env.events().publish(
            (symbol_short!("sch_dec"), profile.pet_id),
            SchemaDecodedEvent {
                version: EVENT_SCHEMA_VERSION,
                domain_code: 1,
                source_schema_version: profile.decoded_from_version,
                entity_id: profile.pet_id,
            },
        );
        Ok(())
    }

    fn validate_version(v: u32) -> Result<(), SchemaCompatError> {
        if v != PRIOR_SCHEMA_V1 && v != CURRENT_SCHEMA_V2 {
            return Err(SchemaCompatError::UnsupportedSchemaVersion);
        }
        Ok(())
    }

    fn validate_non_empty_text(s: &String) -> Result<(), SchemaCompatError> {
        if s.is_empty() {
            return Err(SchemaCompatError::EmptyRequiredField);
        }
        if s.len() > MAX_TEXT_FIELD_LEN {
            return Err(SchemaCompatError::FieldTooLong);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};

    fn setup() -> (Env, ClientSchemaCompatContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);

        let cid = env.register_contract(None, ClientSchemaCompatContract);
        let client = ClientSchemaCompatContractClient::new(&env, &cid);

        let admin = Address::generate(&env);
        client.init(&admin);
        (env, client, admin)
    }

    #[test]
    fn test_decode_v1_and_v2_json_fixtures_across_all_four_domains() {
        let (env, client, admin) = setup();
        let raw = include_str!("../../../fixtures/client-schema-compat/pet_record_schemas.json");
        let parsed: serde_json::Value = serde_json::from_str(raw).expect("valid JSON");

        // 1. PetProfile v1 and v2
        let p_v1 = &parsed["pet_profile"]["v1"];
        let dec_p1 = client.decode_pet_profile(
            &1,
            &p_v1["pet_id"].as_u64().unwrap(),
            &String::from_str(&env, p_v1["name"].as_str().unwrap()),
            &String::from_str(&env, p_v1["species"].as_str().unwrap()),
            &None,
            &None,
        );
        assert_eq!(dec_p1.microchip_id, String::from_str(&env, "UNASSIGNED-V1"));
        assert!(dec_p1.active);

        let p_v2 = &parsed["pet_profile"]["v2"];
        let dec_p2 = client.decode_pet_profile(
            &2,
            &p_v2["pet_id"].as_u64().unwrap(),
            &String::from_str(&env, p_v2["name"].as_str().unwrap()),
            &String::from_str(&env, p_v2["species"].as_str().unwrap()),
            &Some(String::from_str(
                &env,
                p_v2["microchip_id"].as_str().unwrap(),
            )),
            &Some(p_v2["active"].as_bool().unwrap()),
        );
        assert_eq!(dec_p2.pet_id, dec_p1.pet_id);
        client.store_decoded_pet(&admin, &dec_p2, &1);

        // 2. MedicalRecord v1 and v2
        let m_v1 = &parsed["medical_record"]["v1"];
        let dec_m1 = client.decode_medical_record(
            &1,
            &m_v1["record_id"].as_u64().unwrap(),
            &m_v1["pet_id"].as_u64().unwrap(),
            &String::from_str(&env, m_v1["diagnosis"].as_str().unwrap()),
            &None,
            &None,
        );
        assert_eq!(dec_m1.attachment_digest, String::from_str(&env, "none"));
        assert!(!dec_m1.soft_deleted);

        let m_v2 = &parsed["medical_record"]["v2"];
        let dec_m2 = client.decode_medical_record(
            &2,
            &m_v2["record_id"].as_u64().unwrap(),
            &m_v2["pet_id"].as_u64().unwrap(),
            &String::from_str(&env, m_v2["diagnosis"].as_str().unwrap()),
            &Some(String::from_str(
                &env,
                m_v2["attachment_digest"].as_str().unwrap(),
            )),
            &Some(m_v2["soft_deleted"].as_bool().unwrap()),
        );
        assert_eq!(dec_m2.record_id, dec_m1.record_id);

        // 3. Consent v1 and v2
        let c_v1 = &parsed["consent"]["v1"];
        let dec_c1 = client.decode_consent(
            &1,
            &c_v1["consent_id"].as_u64().unwrap(),
            &c_v1["pet_id"].as_u64().unwrap(),
            &None,
            &None,
            &c_v1["expires_at"].as_u64().unwrap(),
        );
        assert_eq!(
            dec_c1.scope_code,
            String::from_str(&env, "LEGACY_READ_ONLY")
        );

        let c_v2 = &parsed["consent"]["v2"];
        let dec_c2 = client.decode_consent(
            &2,
            &c_v2["consent_id"].as_u64().unwrap(),
            &c_v2["pet_id"].as_u64().unwrap(),
            &Some(String::from_str(&env, c_v2["scope_code"].as_str().unwrap())),
            &Some(c_v2["policy_version"].as_u64().unwrap() as u32),
            &c_v2["expires_at"].as_u64().unwrap(),
        );
        assert_eq!(dec_c2.policy_version, 2);

        // 4. Insurance v1 and v2
        let i_v1 = &parsed["insurance"]["v1"];
        let dec_i1 = client.decode_insurance(
            &1,
            &i_v1["policy_id"].as_u64().unwrap(),
            &i_v1["pet_id"].as_u64().unwrap(),
            &(i_v1["coverage_limit"].as_i64().unwrap() as i128),
            &None,
            &None,
            &None,
        );
        assert_eq!(dec_i1.deductible, 0);

        let i_v2 = &parsed["insurance"]["v2"];
        let dec_i2 = client.decode_insurance(
            &2,
            &i_v2["policy_id"].as_u64().unwrap(),
            &i_v2["pet_id"].as_u64().unwrap(),
            &(i_v2["coverage_limit"].as_i64().unwrap() as i128),
            &Some(i_v2["deductible"].as_i64().unwrap() as i128),
            &Some(i_v2["copay_bps"].as_u64().unwrap() as u32),
            &Some(i_v2["active"].as_bool().unwrap()),
        );
        assert_eq!(dec_i2.copay_bps, 1_000);
    }

    #[test]
    fn test_unsupported_version_expired_consent_and_replay_rejected() {
        let (env, client, admin) = setup();
        let bad_ver = client.try_decode_pet_profile(
            &99,
            &1,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "Dog"),
            &None,
            &None,
        );
        assert_eq!(
            bad_ver,
            Err(Ok(SchemaCompatError::UnsupportedSchemaVersion))
        );

        let expired = client.try_decode_consent(&1, &10, &1, &None, &None, &500);
        assert_eq!(expired, Err(Ok(SchemaCompatError::ExpiredConsentRecord)));

        let prof = client.decode_pet_profile(
            &1,
            &1,
            &String::from_str(&env, "Buddy"),
            &String::from_str(&env, "Dog"),
            &None,
            &None,
        );
        client.store_decoded_pet(&admin, &prof, &88);
        let replay = client.try_store_decoded_pet(&admin, &prof, &88);
        assert_eq!(replay, Err(Ok(SchemaCompatError::ReplayDetected)));
    }
}
