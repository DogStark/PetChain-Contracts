use rand::Rng;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorSetup {
    pub secret: String,
    pub qr_code_base64: String,
    pub backup_codes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorData {
    pub secret: String,
    pub backup_codes: Vec<String>,
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Clock drift policy
// ---------------------------------------------------------------------------

/// Controls how many TOTP time-steps either side of "now" are accepted.
///
/// Each step is one TOTP period (30 seconds by default).
///
/// | allowed_steps | Total acceptance window |  Max clock drift  |
/// |---------------|-------------------------|-------------------|
/// |       0       |         30 s            |      ±0 s         |
/// |       1       |         90 s            |      ±30 s        |
/// |       2       |        150 s            |      ±60 s        |
///
/// **Choosing a value**
/// - `0` — strict: only the current 30-second window. Use in high-security
///         environments where the server clock is NTP-synchronised.
/// - `1` — recommended default: tolerates typical mobile clock skew (≤30 s).
/// - `2` — lenient: for environments where device clocks can drift further,
///         at the cost of a slightly wider replay window.
///
/// Values above `2` are accepted but discouraged; they significantly widen
/// the token replay window and offer diminishing usability returns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockDriftPolicy {
    /// Number of time-steps to check before and after the current step.
    pub allowed_steps: u8,
}

impl ClockDriftPolicy {
    /// No tolerance — only the current 30-second window is valid.
    pub const STRICT: Self = Self { allowed_steps: 0 };

    /// ±1 step (±30 s). Recommended for most deployments.
    pub const STANDARD: Self = Self { allowed_steps: 1 };

    /// ±2 steps (±60 s). Use only when devices are known to drift.
    pub const LENIENT: Self = Self { allowed_steps: 2 };

    /// Create a custom policy. `allowed_steps` is clamped to a max of 2
    /// to discourage excessively wide windows.
    pub fn custom(allowed_steps: u8) -> Self {
        Self {
            allowed_steps: allowed_steps.min(2),
        }
    }
}

impl Default for ClockDriftPolicy {
    fn default() -> Self {
        Self::STANDARD
    }
}

// ---------------------------------------------------------------------------
// Core auth logic
// ---------------------------------------------------------------------------

pub struct TwoFactorAuth;

impl TwoFactorAuth {
    pub fn generate_secret() -> String {
        Secret::generate_secret().to_string()
    }

    pub fn setup(user_email: &str, issuer: &str) -> Result<TwoFactorSetup, String> {
        let secret = Self::generate_secret();
        let totp = Self::build_totp(&secret, Some(issuer), user_email)?;

        let qr_code_base64 = totp.get_qr_base64().map_err(|e| e.to_string())?;
        let backup_codes = Self::generate_backup_codes(8);

        Ok(TwoFactorSetup {
            secret,
            qr_code_base64,
            backup_codes,
        })
    }

    /// Verify a token using the default drift policy (STANDARD, ±1 step).
    ///
    /// Prefer [`verify_token_with_policy`] when you need explicit control
    /// over acceptable clock drift.
    pub fn verify_token(secret: &str, token: &str) -> Result<bool, String> {
        Self::verify_token_with_policy(secret, token, ClockDriftPolicy::default())
    }

    /// Verify a token with an explicit clock drift policy.
    ///
    /// Checks the token against the current TOTP step and up to
    /// `policy.allowed_steps` steps in both directions (past and future),
    /// giving a total acceptance window of `(2 * allowed_steps + 1) * 30`
    /// seconds.
    ///
    /// # Example
    /// ```rust
    /// let valid = TwoFactorAuth::verify_token_with_policy(
    ///     &secret,
    ///     &token,
    ///     ClockDriftPolicy::STRICT,
    /// )?;
    /// ```
    pub fn verify_token_with_policy(
        secret: &str,
        token: &str,
        policy: ClockDriftPolicy,
    ) -> Result<bool, String> {
        let totp = Self::build_totp(secret, None, "")?;
        let step_secs: u64 = 30;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs();

        let steps = policy.allowed_steps as i64;

        for offset in -steps..=steps {
            let ts = if offset >= 0 {
                now.saturating_add(offset as u64 * step_secs)
            } else {
                now.saturating_sub((-offset) as u64 * step_secs)
            };

            let expected = totp.generate(ts);
            if expected == token {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn generate_backup_codes(count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut seen = std::collections::HashSet::with_capacity(count);

        while seen.len() < count {
            let code = format!(
                "{:04}-{:04}",
                rng.gen_range(0..10000),
                rng.gen_range(0..10000)
            );
            seen.insert(code);
        }

        seen.into_iter().collect()
    }

    pub fn verify_backup_code(stored_codes: &[String], provided_code: &str) -> Option<usize> {
        stored_codes.iter().position(|code| code == provided_code)
    }

    // -----------------------------------------------------------------------
    // Internal helpers
    // -----------------------------------------------------------------------

    fn build_totp(secret: &str, issuer: Option<&str>, account: &str) -> Result<TOTP, String> {
        TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            Secret::Encoded(secret.to_string())
                .to_bytes()
                .map_err(|e| e.to_string())?,
            issuer.map(str::to_string),
            account.to_string(),
        )
        .map_err(|e| e.to_string())
    }
}
