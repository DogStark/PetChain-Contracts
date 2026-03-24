use crate::rate_limiter::{InMemoryRateLimiter, RateLimitResult, RateLimiter};
use crate::two_factor::{ClockDriftPolicy, TwoFactorAuth, TwoFactorData, TwoFactorSetup};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct EnableTwoFactorRequest {
    pub user_id: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct EnableTwoFactorResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginWithTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct DisableTwoFactorRequest {
    pub user_id: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RecoverWithBackupRequest {
    pub user_id: String,
    pub backup_code: String,
}

// ---------------------------------------------------------------------------
// Handler struct
// ---------------------------------------------------------------------------

pub struct TwoFactorHandlers {
    limiter: Arc<dyn RateLimiter>,
    /// Drift policy applied to all TOTP verification calls.
    /// Defaults to ClockDriftPolicy::STANDARD (±1 step / ±30 s).
    drift_policy: ClockDriftPolicy,
}

impl TwoFactorHandlers {
    /// Create handlers with the default in-memory rate limiter and
    /// standard drift policy (±1 step).
    pub fn new() -> Self {
        Self {
            limiter: Arc::new(InMemoryRateLimiter::default()),
            drift_policy: ClockDriftPolicy::default(),
        }
    }

    /// Create handlers with a custom limiter and the default drift policy.
    pub fn with_limiter(limiter: Arc<dyn RateLimiter>) -> Self {
        Self {
            limiter,
            drift_policy: ClockDriftPolicy::default(),
        }
    }

    /// Create handlers with a custom limiter and an explicit drift policy.
    pub fn with_limiter_and_policy(
        limiter: Arc<dyn RateLimiter>,
        drift_policy: ClockDriftPolicy,
    ) -> Self {
        Self {
            limiter,
            drift_policy,
        }
    }

    /// Override the drift policy on an existing instance.
    pub fn with_drift_policy(mut self, policy: ClockDriftPolicy) -> Self {
        self.drift_policy = policy;
        self
    }

    // -----------------------------------------------------------------------
    // Rate limit key helpers
    // -----------------------------------------------------------------------

    fn verify_activate_key(user_id: &str) -> String {
        format!("2fa:verify_activate:{}", user_id)
    }

    fn login_key(user_id: &str) -> String {
        format!("2fa:login:{}", user_id)
    }

    fn disable_key(user_id: &str) -> String {
        format!("2fa:disable:{}", user_id)
    }

    // -----------------------------------------------------------------------
    // Endpoints
    // -----------------------------------------------------------------------

    // POST /api/2fa/enable
    pub fn enable_two_factor(
        &self,
        req: EnableTwoFactorRequest,
    ) -> Result<EnableTwoFactorResponse, String> {
        let setup = TwoFactorAuth::setup(&req.email, "PetChain")?;

        // Store in database: user_id -> TwoFactorData { secret, backup_codes, enabled: false }
        // db.save_two_factor_setup(&req.user_id, &setup)?;

        Ok(EnableTwoFactorResponse {
            secret: setup.secret,
            qr_code: setup.qr_code_base64,
            backup_codes: setup.backup_codes,
        })
    }

    // POST /api/2fa/verify
    pub fn verify_and_activate(&self, req: VerifyTwoFactorRequest) -> Result<bool, String> {
        let key = Self::verify_activate_key(&req.user_id);

        match self.limiter.record_failure(&key) {
            RateLimitResult::Blocked { retry_after_secs } => {
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds.",
                    retry_after_secs
                ));
            }
            RateLimitResult::Allowed { .. } => {}
        }

        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        let secret = "PLACEHOLDER_SECRET"; // Replace with DB fetch

        let is_valid =
            TwoFactorAuth::verify_token_with_policy(secret, &req.token, self.drift_policy)?;

        if is_valid {
            self.limiter.record_success(&key);
            // db.update_two_factor_enabled(&req.user_id, true)?;
        }

        Ok(is_valid)
    }

    // POST /api/auth/login/2fa
    pub fn verify_login_token(&self, req: LoginWithTwoFactorRequest) -> Result<bool, String> {
        let key = Self::login_key(&req.user_id);

        match self.limiter.record_failure(&key) {
            RateLimitResult::Blocked { retry_after_secs } => {
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds.",
                    retry_after_secs
                ));
            }
            RateLimitResult::Allowed { .. } => {}
        }

        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        let secret = "PLACEHOLDER_SECRET"; // Replace with DB fetch

        let is_valid =
            TwoFactorAuth::verify_token_with_policy(secret, &req.token, self.drift_policy)?;

        if is_valid {
            self.limiter.record_success(&key);
        }

        Ok(is_valid)
    }

    // POST /api/2fa/disable
    pub fn disable_two_factor(&self, req: DisableTwoFactorRequest) -> Result<bool, String> {
        let key = Self::disable_key(&req.user_id);

        match self.limiter.record_failure(&key) {
            RateLimitResult::Blocked { retry_after_secs } => {
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds.",
                    retry_after_secs
                ));
            }
            RateLimitResult::Allowed { .. } => {}
        }

        // let two_factor_data = db.get_two_factor_data(&req.user_id)?;
        let secret = "PLACEHOLDER_SECRET"; // Replace with DB fetch

        let is_valid =
            TwoFactorAuth::verify_token_with_policy(secret, &req.token, self.drift_policy)?;

        if is_valid {
            self.limiter.record_success(&key);
            // db.delete_two_factor_data(&req.user_id)?;
        }

        Ok(is_valid)
    }

    // POST /api/2fa/recover
    pub fn recover_with_backup(&self, req: RecoverWithBackupRequest) -> Result<bool, String> {
        // let mut two_factor_data: TwoFactorData = sqlx::query_as!(
        //     TwoFactorData,
        //     "SELECT secret, backup_codes, enabled FROM user_two_factor WHERE user_id = $1",
        //     req.user_id
        // )
        // .fetch_one(&pool)
        // .await
        // .map_err(|_| "User 2FA data not found".to_string())?;
        //
        // let backup_codes: Vec<String> = serde_json::from_str(&two_factor_data.backup_codes)
        //     .map_err(|_| "Failed to parse backup codes".to_string())?;

        let mut backup_codes: Vec<String> = Vec::new(); // Replace with DB fetch

        if let Some(index) = TwoFactorAuth::verify_backup_code(&backup_codes, &req.backup_code) {
            backup_codes.remove(index);

            // let updated_codes = serde_json::to_string(&backup_codes)
            //     .map_err(|_| "Failed to serialize backup codes".to_string())?;
            //
            // sqlx::query!(
            //     "UPDATE user_two_factor SET backup_codes = $1, updated_at = NOW() WHERE user_id = $2",
            //     updated_codes,
            //     req.user_id
            // )
            // .execute(&pool)
            // .await
            // .map_err(|_| "Failed to update backup codes".to_string())?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for TwoFactorHandlers {
    fn default() -> Self {
        Self::new()
    }
}
