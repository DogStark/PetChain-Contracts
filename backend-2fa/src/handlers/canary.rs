//! Canary token detection handlers.
//!
//! Contains [`CanaryHandlers`] and associated request/response types for
//! Issue #713 — Canary Token Detection.
use crate::two_factor::{TwoFactorAuth, TwoFactorData};
use crate::webhooks::{SecurityEventType, WebhookManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use super::auth::two_factor_store;
use super::auth::verification_config;
use super::auth::AuthenticatedAdmin;

// ---------------------------------------------------------------------------
// Issue #713 — Canary Token Detection
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct CreateCanaryRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct CreateCanaryResponse {
    pub user_id: String,
    pub secret: String,
    pub qr_code: String,
}

pub struct CanaryHandlers {
    webhook_manager: Arc<WebhookManager>,
}

impl CanaryHandlers {
    pub fn new(webhook_manager: Arc<WebhookManager>) -> Self {
        Self { webhook_manager }
    }

    /// Admin: create a canary TOTP account that looks real but triggers an
    /// alert when any verification is attempted.
    pub fn create_canary(
        admin: &AuthenticatedAdmin,
        req: CreateCanaryRequest,
    ) -> Result<CreateCanaryResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        // Persist Argon2id hashes only; the canary response never exposes
        // backup codes at all, so there is no plaintext to preserve here.
        let hashed_backup_codes = TwoFactorAuth::hash_backup_codes(&setup.backup_codes)?;

        two_factor_store().save(
            &req.user_id,
            TwoFactorData {
                secret: setup.secret.clone(),
                backup_codes: hashed_backup_codes,
                enabled: true,
                algorithm: setup.config.algorithm,
                last_used_step: None,
            },
        )?;

        two_factor_store().set_canary(&req.user_id, true)?;

        two_factor_store().append_audit_log(
            &req.user_id,
            "canary_created",
            &admin.admin_id,
            None,
        )?;

        Ok(CreateCanaryResponse {
            user_id: req.user_id,
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
        })
    }

    /// Verify a TOTP token for a user. If the account is a canary, log a
    /// `CanaryTriggered` audit event and fire the webhook immediately.
    /// The canary account always returns `false` for the verification result
    /// so the attacker gets no useful feedback.
    pub fn verify_with_canary_check(
        &self,
        user_id: &str,
        token: &str,
        ip_address: Option<&str>,
    ) -> Result<bool, String> {
        let store = two_factor_store();

        if store.is_canary(user_id) {
            // Log the trigger event
            let meta = ip_address.map(|ip| format!("ip={}", ip));
            store.append_audit_log(user_id, "CanaryTriggered", user_id, meta.as_deref())?;

            // Fire webhook immediately
            let mut metadata = HashMap::new();
            if let Some(ip) = ip_address {
                metadata.insert("ip".to_string(), ip.to_string());
            }
            metadata.insert("user_id".to_string(), user_id.to_string());
            self.webhook_manager
                .fire(SecurityEventType::CanaryTriggered, user_id, metadata);

            // Return false — canary accounts never grant access
            return Ok(false);
        }

        let data = store.get(user_id)?;
        TwoFactorAuth::verify_token_with_config(
            &data.secret,
            token,
            verification_config(data.algorithm),
        )
    }
}
