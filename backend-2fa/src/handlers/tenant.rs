//! Multi-tenant 2FA handlers.
//!
//! Contains [`MultiTenantHandlers`] and [`TenantProvisioningHandlers`] for
//! multi-tenant 2FA support.
use crate::error::ApiError;
use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter, TenantRateLimitKey};
use crate::two_factor::{
    TenantConfig, TenantRegistry, TenantScopedStore, TwoFactorAuth, TwoFactorData,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::auth::{
    AuthenticatedAdmin, AuthenticatedUser, EnableTwoFactorResponse, two_factor_store,
    verification_config,
};

// ---------------------------------------------------------------------------
// Multi-tenant support (Issue: multi-tenant 2FA)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct ProvisionTenantRequest {
    pub tenant_id: String,
    pub name: String,
    pub max_users: u32,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
}

#[derive(Debug, Serialize)]
pub struct ProvisionTenantResponse {
    pub tenant_id: String,
    pub name: String,
    pub max_users: u32,
    pub totp_issuer: String,
    pub rate_limit_max_failures: u32,
    /// `true` if `tenant_id` already existed and this call returned the
    /// existing tenant's config instead of creating a new one. Lets
    /// infrastructure automation safely retry `POST /tenant/provision`
    /// without erroring or creating duplicates.
    pub already_existed: bool,
}

/// Maximum length for `TenantConfig::tenant_id`.
const MAX_TENANT_ID_LEN: usize = 64;
/// Maximum length for `TenantConfig::name`.
const MAX_TENANT_NAME_LEN: usize = 128;

/// Validates a [`TenantConfig`] before it is persisted by `provision_tenant`.
///
/// - `tenant_id`: non-empty, at most 64 characters, alphanumeric plus hyphens only.
/// - `max_users`: must be >= 1.
/// - `name`: non-empty, at most 128 characters.
///
/// On failure, returns a `BAD_REQUEST` [`ApiError`] naming the offending field
/// in `details.field`.
fn validate_tenant_config(config: &TenantConfig) -> Result<(), ApiError> {
    let bad_field = |field: &str, message: String| {
        ApiError::bad_request(message, Some(serde_json::json!({ "field": field })))
    };

    if config.tenant_id.is_empty() {
        return Err(bad_field("tenant_id", "tenant_id must not be empty".into()));
    }
    if config.tenant_id.len() > MAX_TENANT_ID_LEN {
        return Err(bad_field(
            "tenant_id",
            format!("tenant_id must be at most {MAX_TENANT_ID_LEN} characters"),
        ));
    }
    if !config
        .tenant_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(bad_field(
            "tenant_id",
            "tenant_id must contain only alphanumeric characters and hyphens".into(),
        ));
    }

    if config.max_users < 1 {
        return Err(bad_field("max_users", "max_users must be >= 1".into()));
    }

    if config.name.is_empty() {
        return Err(bad_field("name", "name must not be empty".into()));
    }
    if config.name.len() > MAX_TENANT_NAME_LEN {
        return Err(bad_field(
            "name",
            format!("name must be at most {MAX_TENANT_NAME_LEN} characters"),
        ));
    }

    Ok(())
}

/// Handlers that operate within a single tenant's namespace.
/// All user data is scoped to the tenant; cross-tenant access is rejected
/// at the `TenantScopedStore` level.
pub struct MultiTenantHandlers {
    store: TenantScopedStore,
    limiter: Arc<dyn RateLimiter>,
}

impl MultiTenantHandlers {
    pub fn new(store: TenantScopedStore) -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            store,
        }
    }

    pub fn with_limiter(store: TenantScopedStore, limiter: Arc<dyn RateLimiter>) -> Self {
        Self { store, limiter }
    }

    pub fn enable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        email: &str,
    ) -> Result<EnableTwoFactorResponse, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        if let Ok(existing) = self.store.get(user_id) {
            if existing.enabled {
                return Err(
                    "2FA is already enabled. To re-enroll, you must first disable it.".to_string(),
                );
            }
        }

        let setup = TwoFactorAuth::setup(email, self.store.issuer())?;

        // Persist Argon2id hashes only — the plaintext codes are returned to
        // the caller once below and never stored.
        let hashed_backup_codes = TwoFactorAuth::hash_backup_codes(&setup.backup_codes)?;

        self.store.save(
            user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: hashed_backup_codes,
                enabled: false,
                algorithm: setup.config.algorithm,
                last_used_step: None,
            },
        )?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
            otpauth_uri: setup.otpauth_uri,
        })
    }

    pub fn verify_and_activate(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let key = TenantRateLimitKey::new(&self.store.config.tenant_id, "verify", user_id);
        if let RateLimitResult::Blocked {
            retry_after_secs, ..
        } = self.limiter.record_failure(key.as_str())
        {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    retry_after_secs
                ),
                retry_after_secs,
            )
            .to_string());
        }

        let data = self.store.get(user_id)?;
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, true)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }

    pub fn disable_two_factor(
        &self,
        caller: &AuthenticatedUser,
        user_id: &str,
        token: &str,
    ) -> Result<bool, String> {
        caller.authorize(user_id).map_err(|e| e.to_string())?;

        let key = TenantRateLimitKey::new(&self.store.config.tenant_id, "disable", user_id);
        if let RateLimitResult::Blocked {
            retry_after_secs, ..
        } = self.limiter.record_failure(key.as_str())
        {
            return Err(ApiError::rate_limited(
                format!(
                    "Too many failed attempts. Retry after {} seconds.",
                    retry_after_secs
                ),
                retry_after_secs,
            )
            .to_string());
        }

        let data = self.store.get(user_id)?;
        if !data.enabled {
            return Ok(false);
        }
        let result = TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )?;
        if result {
            self.store.update_enabled(user_id, false)?;
            self.limiter.record_success(key.as_str());
        }
        Ok(result)
    }
}

/// Super-admin handler for tenant provisioning.
pub struct TenantProvisioningHandlers {
    registry: Arc<TenantRegistry>,
}

impl TenantProvisioningHandlers {
    pub fn new(registry: Arc<TenantRegistry>) -> Self {
        Self { registry }
    }

    /// Provision a tenant (super-admin only — caller must be verified externally).
    ///
    /// Idempotent: calling this repeatedly with the same `tenant_id` never
    /// errors or creates a duplicate. The first call creates the tenant and
    /// returns `already_existed: false`; subsequent calls return the
    /// existing tenant's config with `already_existed: true`. This lets
    /// infrastructure automation safely retry provisioning on failure.
    pub fn provision_tenant(
        &self,
        _super_admin: &AuthenticatedAdmin,
        req: ProvisionTenantRequest,
    ) -> Result<ProvisionTenantResponse, ApiError> {
        let config = TenantConfig {
            tenant_id: req.tenant_id.clone(),
            name: req.name.clone(),
            max_users: req.max_users,
            totp_issuer: req.totp_issuer.clone(),
            rate_limit_max_failures: req.rate_limit_max_failures,
            lockout_threshold: 10,
        };

        validate_tenant_config(&config)?;

        let (existing_or_new, already_existed) = self
            .registry
            .provision(config)
            .map_err(|e| ApiError::internal_error(e, None))?;

        Ok(ProvisionTenantResponse {
            tenant_id: existing_or_new.tenant_id,
            name: existing_or_new.name,
            max_users: existing_or_new.max_users,
            totp_issuer: existing_or_new.totp_issuer,
            rate_limit_max_failures: existing_or_new.rate_limit_max_failures,
            already_existed,
        })
    }

    pub fn get_tenant_config(&self, tenant_id: &str) -> Option<TenantConfig> {
        self.registry.get_config(tenant_id)
    }
}

/// Build a [`TenantScopedStore`] from the singleton store for use in tests.
#[cfg(test)]
pub fn make_tenant_store(tenant_id: &str) -> TenantScopedStore {
    let config = TenantConfig::new(tenant_id);
    TenantScopedStore::new(two_factor_store(), config)
}
