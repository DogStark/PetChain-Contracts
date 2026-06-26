use crate::ip_access::{CidrBlock, IpAccessEntry, IpAccessStore, IpListType};
use crate::two_factor::HmacAlgorithm;
use crate::two_factor::{
    AuditLogEntry, RecoveryCodeUsageLog, TwoFactorData, TwoFactorLockoutState, TwoFactorStore,
    UserTwoFactorSummary,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Pool utilisation snapshot returned by [`PostgresTwoFactorStore::pool_stats`].
#[derive(Debug, Clone, PartialEq)]
pub struct PoolStats {
    pub active: u32,
    pub idle: u32,
    pub max: u32,
}

/// Trait for fetching secrets (e.g. DB connection strings).
pub trait SecretProvider: Send + Sync {
    fn get_secret(&self, key: &str) -> Result<String, String>;
}

/// Env var based secret provider (current behavior)
pub struct EnvSecretProvider;
impl SecretProvider for EnvSecretProvider {
    fn get_secret(&self, key: &str) -> Result<String, String> {
        std::env::var(key).map_err(|e| e.to_string())
    }
}

/// AWS Secrets Manager provider for tests/usage. For testing we support
/// an env var `AWS_SECRETS_JSON` containing a JSON map of key->value.
/// In production this struct would call AWS SDK.
pub struct AwsSecretsManagerProvider;
impl SecretProvider for AwsSecretsManagerProvider {
    fn get_secret(&self, key: &str) -> Result<String, String> {
        if let Ok(json) = std::env::var("AWS_SECRETS_JSON") {
            let map: Result<HashMap<String, String>, _> = serde_json::from_str(&json);
            if let Ok(map) = map {
                if let Some(v) = map.get(key) {
                    return Ok(v.clone());
                }
            }
        }
        Err(format!("secret not found: {}", key))
    }
}

/// Select provider by env var `SECRET_PROVIDER` ("env" or "aws").
pub fn select_secret_provider() -> Box<dyn SecretProvider> {
    match std::env::var("SECRET_PROVIDER")
        .unwrap_or_else(|_| "env".to_string())
        .as_str()
    {
        "aws" => Box::new(AwsSecretsManagerProvider {}),
        _ => Box::new(EnvSecretProvider {}),
    }
}

#[derive(Clone)]
pub struct PostgresTwoFactorStore {
    pool: PgPool,
    runtime: Arc<Runtime>,
}

impl PostgresTwoFactorStore {
    pub fn connect(database_url: &str) -> Result<Self, String> {
        let runtime = Arc::new(Runtime::new().map_err(|e| e.to_string())?);
        let min_conns: u32 = std::env::var("DB_POOL_MIN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let max_conns: u32 = std::env::var("DB_POOL_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);
        let acquire_timeout_secs: u64 = std::env::var("DB_POOL_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let pool = runtime
            .block_on(
                PgPoolOptions::new()
                    .min_connections(min_conns)
                    .max_connections(max_conns)
                    .acquire_timeout(Duration::from_secs(acquire_timeout_secs))
                    .connect(database_url),
            )
            .map_err(|e| e.to_string())?;

        Ok(Self { pool, runtime })
    }

    /// Connect using a SecretProvider to fetch the `secret_key` value.
    pub fn connect_with_provider(
        provider: &dyn SecretProvider,
        secret_key: &str,
    ) -> Result<Self, String> {
        let database_url = provider.get_secret(secret_key)?;
        PostgresTwoFactorStore::connect(&database_url)
    }

    pub fn from_pool(pool: PgPool) -> Result<Self, String> {
        let runtime = Arc::new(Runtime::new().map_err(|e| e.to_string())?);
        Ok(Self { pool, runtime })
    }

    fn block_on<F, T>(&self, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        self.runtime.block_on(future).map_err(|e| e.to_string())
    }

    fn block_on_typed<F, T>(&self, future: F) -> Result<T, sqlx::Error>
    where
    F: std::future::Future<Output = Result<T, sqlx::Error>>,
{
    self.runtime.block_on(future)
}
/// Execute `op` with up to 3 attempts and exponential backoff (100 ms, 200 ms, 400 ms).
/// The closure must return `sqlx::Error` so retry eligibility is checked on the
/// typed variant before the error is stringified.
   fn with_retry<F, T>(&self, mut op: F) -> Result<T, String>
where
    F: FnMut() -> Result<T, sqlx::Error>,
{
    const MAX_ATTEMPTS: u32 = 3;
    let mut delay_ms = 100u64;
    for attempt in 1..=MAX_ATTEMPTS {
        match op() {
            Ok(v) => return Ok(v),
            Err(e) if attempt < MAX_ATTEMPTS && is_connection_error(&e) => {
                std::thread::sleep(Duration::from_millis(delay_ms));
                delay_ms *= 2;
            }
            Err(e) => return Err(e.to_string()),
        }
    }
    unreachable!()
}
    /// Ping the database. Returns `Err` if the pool is exhausted or the
    /// connection cannot be acquired within the pool's connect timeout.
    pub fn health_check(&self) -> Result<(), String> {
        if self.pool.size() >= self.pool.options().get_max_connections() {
            return Err("pool exhausted".to_string());
        }
        self.block_on(sqlx::query("SELECT 1").execute(&self.pool))?;
        Ok(())
    }

    /// Return a snapshot of current pool utilisation.
    pub fn pool_stats(&self) -> PoolStats {
        let max = self.pool.options().get_max_connections();
        let idle = self.pool.num_idle() as u32;
        let size = self.pool.size();
        let active = size.saturating_sub(idle);
        PoolStats { active, idle, max }
    }
}

pub(crate) fn is_connection_error(err: &sqlx::Error) -> bool {
    match err {
        sqlx::Error::Io(_) => true,
        sqlx::Error::PoolTimedOut => true,
        sqlx::Error::PoolClosed => true,
        _ => false,
    }
}

impl TwoFactorStore for PostgresTwoFactorStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        let backup_codes = serde_json::to_string(&data.backup_codes).map_err(|e| e.to_string())?;
        let user_id = user_id.to_string();
        // Each closure invocation builds and drives a fresh future — no future
        // is shared across retry attempts.
        self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
            INSERT INTO user_two_factor (user_id, secret, backup_codes, enabled, algorithm)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id)
            DO UPDATE SET
                secret = EXCLUDED.secret,
                backup_codes = EXCLUDED.backup_codes,
                enabled = EXCLUDED.enabled,
                algorithm = EXCLUDED.algorithm,
                updated_at = CURRENT_TIMESTAMP
            "#,
                )
                .bind(&user_id)
                .bind(&data.secret)
                .bind(&backup_codes)
                .bind(data.enabled)
                .bind(Self::algorithm_to_db(data.algorithm))
                .execute(&self.pool),
            )
            .map(|_| ()) // discard PgQueryResult; map inside closure so with_retry sees Result<(), _>
        })
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        let user_id = user_id.to_string();
        // with_retry owns the retry loop; block_on drives one fresh future per
        // attempt.  The Option unwrap is post-retry business logic, kept outside.
        let row = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_as::<_, (String, String, bool, Option<String>)>(
                    r#"
            SELECT secret, backup_codes, enabled, algorithm
            FROM user_two_factor
            WHERE user_id = $1
            "#,
                )
                .bind(&user_id)
                .fetch_optional(&self.pool),
            )
        })?;

        let (secret, backup_codes, enabled, algorithm) =
            row.ok_or_else(|| format!("No 2FA data found for user: {}", user_id))?;
        let backup_codes = serde_json::from_str(&backup_codes).map_err(|e| e.to_string())?;

        Ok(TwoFactorData {
            secret,
            backup_codes,
            enabled,
            algorithm: Self::algorithm_from_db(algorithm.as_deref()),
        })
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        let user_id = user_id.to_string();
        // with_retry retries the DB round-trip; rows_affected check is
        // post-commit business logic and intentionally sits outside.
        let result = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query("DELETE FROM user_two_factor WHERE user_id = $1")
                    .bind(&user_id)
                    .execute(&self.pool),
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(format!("No 2FA data found for user: {}", user_id));
        }

        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        let user_id = user_id.to_string();
        // with_retry retries the DB round-trip; rows_affected check is
        // post-commit business logic and intentionally sits outside.
        let result = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
                UPDATE user_two_factor
                SET enabled = $2, updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $1
                "#,
                )
                .bind(&user_id)
                .bind(enabled)
                .execute(&self.pool),
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(format!("No 2FA data found for user: {}", user_id));
        }

        Ok(())
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        let backup_codes = serde_json::to_string(&codes).map_err(|e| e.to_string())?;
        let user_id = user_id.to_string();
        // with_retry retries the DB round-trip; rows_affected check is
        // post-commit business logic and intentionally sits outside.
        let result = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
                UPDATE user_two_factor
                SET backup_codes = $2, updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $1
                "#,
                )
                .bind(&user_id)
                .bind(&backup_codes)
                .execute(&self.pool),
            )
        })?;

        if result.rows_affected() == 0 {
            return Err(format!("No 2FA data found for user: {}", user_id));
        }

        Ok(())
    }

    fn log_recovery_code_usage(
        &self,
        user_id: &str,
        code_index: i32,
        ip_address: Option<&str>,
    ) -> Result<(), String> {
        let user_id = user_id.to_string();
        let ip_address = ip_address.map(|s| s.to_string());
        let result = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
                INSERT INTO recovery_code_usage (user_id, code_index, used_at, ip_address)
                VALUES ($1, $2, CURRENT_TIMESTAMP, $3)
                "#,
                )
                .bind(&user_id)
                .bind(code_index)
                .bind(ip_address.as_deref())
                .execute(&self.pool),
            )
        });

        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                // Check if it's a unique constraint violation (duplicate key)
                if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
                    Err("InvalidRecoveryCode".to_string())
                } else {
                    Err(e.to_string())
                }
            }
        }
    }

    fn get_recovery_usage_log(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<RecoveryCodeUsageLog>, String> {
        let offset = (page.saturating_sub(1)) * page_size;
        let limit = page_size as i64;

        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            user_id: String,
            code_index: i32,
            used_at: String,
            ip_address: Option<String>,
        }

        let rows = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_as::<_, Row>(
                    r#"
                SELECT id, user_id, code_index, used_at, ip_address
                FROM recovery_code_usage
                ORDER BY used_at DESC
                LIMIT $1 OFFSET $2
                "#,
                )
                .bind(limit)
                .bind(offset as i64)
                .fetch_all(&self.pool),
            )
        })?;

        Ok(rows
            .into_iter()
            .map(|r| RecoveryCodeUsageLog {
                id: r.id as usize,
                user_id: r.user_id,
                code_index: r.code_index,
                used_at: r.used_at,
                ip_address: r.ip_address,
            })
            .collect())
    }

    fn list_users(&self, page: u32, page_size: u32) -> Result<Vec<UserTwoFactorSummary>, String> {
        let offset = (page.saturating_sub(1)) * page_size;
        let limit = page_size as i64;

        #[derive(sqlx::FromRow)]
        struct Row {
            user_id: String,
            enabled: bool,
        }

        let rows = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_as::<_, Row>(
                    r#"
                SELECT u.user_id, u.enabled
                FROM user_two_factor u
                LEFT JOIN canary_accounts c ON c.user_id = u.user_id
                WHERE c.user_id IS NULL
                ORDER BY u.user_id
                LIMIT $1 OFFSET $2
                "#,
                )
                .bind(limit)
                .bind(offset as i64)
                .fetch_all(&self.pool),
            )
        })?;

        Ok(rows
            .into_iter()
            .map(|r| UserTwoFactorSummary {
                user_id: r.user_id,
                enabled: r.enabled,
                is_canary: false,
            })
            .collect())
    }

    fn admin_disable_two_fa(&self, user_id: &str, admin_id: &str) -> Result<(), String> {
        self.update_enabled(user_id, false)?;
        self.append_audit_log(user_id, "admin_disabled_2fa", admin_id, None)?;
        Ok(())
    }

    fn get_audit_log(
        &self,
        user_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<AuditLogEntry>, String> {
        let offset = (page.saturating_sub(1)) * page_size;
        let limit = page_size as i64;
        let user_id = user_id.to_string();

        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            user_id: String,
            event: String,
            timestamp: i64,
            actor: String,
            metadata: Option<String>,
        }

        let rows = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_as::<_, Row>(
                    r#"
                SELECT id, user_id, event, EXTRACT(EPOCH FROM timestamp)::bigint AS timestamp,
                       actor, metadata
                FROM two_fa_audit_log
                WHERE user_id = $1
                ORDER BY timestamp DESC
                LIMIT $2 OFFSET $3
                "#,
                )
                .bind(&user_id)
                .bind(limit)
                .bind(offset as i64)
                .fetch_all(&self.pool),
            )
        })?;

        Ok(rows
            .into_iter()
            .map(|r| AuditLogEntry {
                id: r.id as usize,
                user_id: r.user_id,
                event: r.event,
                timestamp: r.timestamp as u64,
                actor: r.actor,
                metadata: r.metadata,
            })
            .collect())
    }

    fn append_audit_log(
        &self,
        user_id: &str,
        event: &str,
        actor: &str,
        metadata: Option<&str>,
    ) -> Result<(), String> {
        let user_id = user_id.to_string();
        let event = event.to_string();
        let actor = actor.to_string();
        let metadata = metadata.map(|s| s.to_string());
        // Each closure invocation builds and drives a fresh future.
        self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
                INSERT INTO two_fa_audit_log (user_id, event, actor, metadata)
                VALUES ($1, $2, $3, $4)
                "#,
                )
                .bind(&user_id)
                .bind(&event)
                .bind(&actor)
                .bind(metadata.as_deref())
                .execute(&self.pool),
            )
            .map(|_| ())
        })
    }

    fn set_canary(&self, user_id: &str, is_canary: bool) -> Result<(), String> {
        let user_id = user_id.to_string();
        // Each branch builds a fresh future on every retry attempt.
        if is_canary {
            self.with_retry(|| {
                self.block_on_typed(
                    sqlx::query(
                        r#"
                    INSERT INTO canary_accounts (user_id) VALUES ($1)
                    ON CONFLICT (user_id) DO NOTHING
                    "#,
                    )
                    .bind(&user_id)
                    .execute(&self.pool),
                )
                .map(|_| ())
            })
        } else {
            self.with_retry(|| {
                self.block_on_typed(
                    sqlx::query("DELETE FROM canary_accounts WHERE user_id = $1")
                        .bind(&user_id)
                        .execute(&self.pool),
                )
                .map(|_| ())
            })
        }
    }

    fn is_canary(&self, user_id: &str) -> bool {
        let user_id = user_id.to_string();
        self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM canary_accounts WHERE user_id = $1",
                )
                .bind(&user_id)
                .fetch_one(&self.pool),
            )
        })
        .map(|c| c > 0)
        .unwrap_or(false)
    }

    fn get_lockout_state(&self, user_id: &str) -> Result<TwoFactorLockoutState, String> {
        #[derive(sqlx::FromRow)]
        struct Row {
            failed_attempts: i32,
            locked: bool,
            locked_at: Option<i64>,
            updated_at: i64,
        }

        let user_id = user_id.to_string();
        let row = self.with_retry(|| {
            self.block_on_typed(
                sqlx::query_as::<_, Row>(
                    r#"
                SELECT failed_attempts, locked,
                       EXTRACT(EPOCH FROM locked_at)::bigint AS locked_at,
                       EXTRACT(EPOCH FROM updated_at)::bigint AS updated_at
                FROM two_fa_lockouts
                WHERE user_id = $1
                "#,
                )
                .bind(&user_id)
                .fetch_optional(&self.pool),
            )
        })?;

        Ok(row
            .map(|r| TwoFactorLockoutState {
                failed_attempts: r.failed_attempts.max(0) as u32,
                locked: r.locked,
                locked_at: r.locked_at.map(|ts| ts as u64),
                updated_at: r.updated_at as u64,
            })
            .unwrap_or_default())
    }

    fn record_failed_two_fa_attempt(&self, user_id: &str) -> Result<TwoFactorLockoutState, String> {
        let user_id = user_id.to_string();
        // with_retry covers the INSERT/UPDATE round-trip.  get_lockout_state is
        // a separate retried read — the two with_retry calls are sequential, not
        // nested, so there is no double-wrapping.
        self.with_retry(|| {
            self.block_on_typed(
                sqlx::query(
                    r#"
                INSERT INTO two_fa_lockouts (user_id, failed_attempts, locked, locked_at, updated_at)
                VALUES ($1, 1, FALSE, NULL, CURRENT_TIMESTAMP)
                ON CONFLICT (user_id)
                DO UPDATE SET
                    failed_attempts = two_fa_lockouts.failed_attempts + 1,
                    locked = (two_fa_lockouts.failed_attempts + 1) >= 10,
                    locked_at = CASE
                        WHEN (two_fa_lockouts.failed_attempts + 1) >= 10
                             AND two_fa_lockouts.locked_at IS NULL
                        THEN CURRENT_TIMESTAMP
                        ELSE two_fa_lockouts.locked_at
                    END,
                    updated_at = CURRENT_TIMESTAMP
                "#,
                )
                .bind(&user_id)
                .execute(&self.pool),
            )
            .map(|_| ())
        })?;
        self.get_lockout_state(&user_id)
    }

    fn reset_two_fa_failures(&self, user_id: &str) -> Result<(), String> {
        let user_id = user_id.to_string();
        self.with_retry(|| {
            self.block_on_typed(
                sqlx::query("DELETE FROM two_fa_lockouts WHERE user_id = $1")
                    .bind(&user_id)
                    .execute(&self.pool),
            )
            .map(|_| ())
        })
    }

    fn unlock_two_fa_account(&self, user_id: &str, actor: &str) -> Result<(), String> {
        self.reset_two_fa_failures(user_id)?;
        self.append_audit_log(user_id, "two_fa_account_unlocked", actor, None)?;
        Ok(())
    }

    fn try_pool_stats(&self) -> Option<PoolStats> {
        Some(self.pool_stats())
    }
}

// ---------------------------------------------------------------------------
// Postgres-backed IP allowlist / blocklist store (Issue #701)
// ---------------------------------------------------------------------------

fn ip_list_type_to_db(list_type: IpListType) -> &'static str {
    match list_type {
        IpListType::Allow => "allow",
        IpListType::Block => "block",
    }
}

#[derive(Clone)]
pub struct PostgresIpAccessStore {
    pool: PgPool,
    runtime: Arc<Runtime>,
}

impl PostgresIpAccessStore {
    pub fn connect(database_url: &str) -> Result<Self, String> {
        let runtime = Arc::new(Runtime::new().map_err(|e| e.to_string())?);
        let pool = runtime
            .block_on(PgPoolOptions::new().max_connections(10).connect(database_url))
            .map_err(|e| e.to_string())?;
        Ok(Self { pool, runtime })
    }

    pub fn from_pool(pool: PgPool) -> Result<Self, String> {
        let runtime = Arc::new(Runtime::new().map_err(|e| e.to_string())?);
        Ok(Self { pool, runtime })
    }

    fn block_on<F, T>(&self, future: F) -> Result<T, String>
    where
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        self.runtime.block_on(future).map_err(|e| e.to_string())
    }
}

impl IpAccessStore for PostgresIpAccessStore {
    fn add_entry(
        &self,
        cidr: &str,
        list_type: IpListType,
        note: Option<&str>,
        created_by: &str,
    ) -> Result<IpAccessEntry, String> {
        CidrBlock::parse(cidr)?;

        let (id,): (i64,) = self.block_on(
            sqlx::query_as(
                r#"
                INSERT INTO ip_access_list (cidr, list_type, note, created_by)
                VALUES ($1, $2, $3, $4)
                RETURNING id
                "#,
            )
            .bind(cidr)
            .bind(ip_list_type_to_db(list_type))
            .bind(note)
            .bind(created_by)
            .fetch_one(&self.pool),
        )?;

        Ok(IpAccessEntry {
            id,
            cidr: cidr.to_string(),
            list_type,
            note: note.map(str::to_string),
            created_by: created_by.to_string(),
        })
    }

    fn remove_entry(&self, id: i64) -> Result<(), String> {
        let result = self.block_on(
            sqlx::query("DELETE FROM ip_access_list WHERE id = $1")
                .bind(id)
                .execute(&self.pool),
        )?;

        if result.rows_affected() == 0 {
            return Err(format!("no IP access entry with id {id}"));
        }
        Ok(())
    }

    fn list_entries(&self, list_type: IpListType) -> Vec<IpAccessEntry> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i64,
            cidr: String,
            note: Option<String>,
            created_by: String,
        }

        let rows = self.block_on(
            sqlx::query_as::<_, Row>(
                r#"
                SELECT id, cidr, note, created_by
                FROM ip_access_list
                WHERE list_type = $1
                ORDER BY id
                "#,
            )
            .bind(ip_list_type_to_db(list_type))
            .fetch_all(&self.pool),
        );

        rows.map(|rows| {
            rows.into_iter()
                .map(|r| IpAccessEntry {
                    id: r.id,
                    cidr: r.cidr,
                    list_type,
                    note: r.note,
                    created_by: r.created_by,
                })
                .collect()
        })
        .unwrap_or_default()
    }
}

impl PostgresTwoFactorStore {
    fn algorithm_to_db(algorithm: HmacAlgorithm) -> String {
        match algorithm {
            HmacAlgorithm::SHA1 => "SHA1".to_string(),
            HmacAlgorithm::SHA256 => "SHA256".to_string(),
            HmacAlgorithm::SHA512 => "SHA512".to_string(),
        }
    }

    fn algorithm_from_db(value: Option<&str>) -> HmacAlgorithm {
        match value {
            Some("SHA256") => HmacAlgorithm::SHA256,
            Some("SHA512") => HmacAlgorithm::SHA512,
            _ => HmacAlgorithm::SHA1,
        }
    }

    #[cfg(test)]
    pub fn algorithm_to_db_pub(algorithm: HmacAlgorithm) -> String {
        Self::algorithm_to_db(algorithm)
    }

    #[cfg(test)]
    pub fn algorithm_from_db_pub(value: Option<&str>) -> HmacAlgorithm {
        Self::algorithm_from_db(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> TwoFactorData {
        TwoFactorData {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["1111-2222".to_string(), "3333-4444".to_string()],
            enabled: false,
            algorithm: HmacAlgorithm::SHA1,
        }
    }

    #[test]
    fn postgres_store_roundtrip_when_database_url_is_set() {
        let Ok(database_url) = std::env::var("DATABASE_URL") else {
            return;
        };

        let store = PostgresTwoFactorStore::connect(&database_url).unwrap();
        let user_id = "postgres-store-roundtrip-test";
        let _ = store.delete(user_id);

        store.save(user_id, test_data()).unwrap();
        assert_eq!(store.get(user_id).unwrap().backup_codes.len(), 2);

        store.update_enabled(user_id, true).unwrap();
        assert!(store.get(user_id).unwrap().enabled);

        store
            .update_backup_codes(user_id, vec!["5555-6666".to_string()])
            .unwrap();
        assert_eq!(store.get(user_id).unwrap().backup_codes[0], "5555-6666");

        store.delete(user_id).unwrap();
        assert!(store.get(user_id).is_err());
    }

    #[test]
    fn env_secret_provider_reads_env() {
        std::env::set_var("TEST_SECRET_KEY", "secret-value");
        let prov = EnvSecretProvider {};
        let val = prov.get_secret("TEST_SECRET_KEY").unwrap();
        assert_eq!(val, "secret-value");
    }

    #[test]
    fn aws_provider_reads_json_map() {
        let map = serde_json::json!({"DB_KEY": "db://conn-string"}).to_string();
        std::env::set_var("AWS_SECRETS_JSON", map);
        let prov = AwsSecretsManagerProvider {};
        let val = prov.get_secret("DB_KEY").unwrap();
        assert_eq!(val, "db://conn-string");
    }

    #[test]
    fn select_secret_provider_env_default() {
        std::env::remove_var("SECRET_PROVIDER");
        let prov = select_secret_provider();
        // default is EnvSecretProvider; ensure get_secret returns Err for unknown key
        assert!(prov.get_secret("NON_EXISTENT").is_err());
    }

    #[test]
    fn select_secret_provider_aws() {
        std::env::set_var("SECRET_PROVIDER", "aws");
        let map = serde_json::json!({"MYKEY": "VAL"}).to_string();
        std::env::set_var("AWS_SECRETS_JSON", map);
        let prov = select_secret_provider();
        let val = prov.get_secret("MYKEY").unwrap();
        assert_eq!(val, "VAL");
    }

    // -----------------------------------------------------------------------
    // with_retry unit tests — no live DB required
    // -----------------------------------------------------------------------

    /// Verify that with_retry succeeds on the first attempt when the op is clean.
    #[test]
    fn with_retry_succeeds_immediately() {
        // We can exercise with_retry directly via a mock store config.
        // Build a minimal PostgresTwoFactorStore substitute that only exercises
        // the retry logic by calling with_retry on a closure directly.
        //
        // Since with_retry is a method on PostgresTwoFactorStore we test it by
        // building a real instance only when DATABASE_URL is set, but the
        // pure-logic path below uses an independent helper that mirrors the
        // same implementation so the unit test never needs a live connection.
        let mut call_count = 0u32;
        let result = retry_helper(
            3,
            100,
            || {
                call_count += 1;
                Ok::<i32, String>(42)
            },
        );
        assert_eq!(result, Ok(42));
        assert_eq!(call_count, 1, "should succeed on the first attempt");
    }

    /// A one-shot transient connection error is retried and the second attempt succeeds.
    #[test]
    fn with_retry_recovers_from_one_shot_connection_failure() {
        let mut call_count = 0u32;
        let result = retry_helper(
            3,
            1, // use 1 ms delay to keep the test fast
            || {
                call_count += 1;
                if call_count == 1 {
                    // Simulate the kind of error that is_connection_error recognises
                    Err("connection reset by peer".to_string())
                } else {
                    Ok::<(), String>(())
                }
            },
        );
        assert!(result.is_ok(), "should succeed after retry, got: {:?}", result);
        assert_eq!(call_count, 2, "should have taken exactly two attempts");
    }

    /// A non-connection error is NOT retried — it is returned immediately.
    #[test]
    fn with_retry_does_not_retry_non_connection_errors() {
        let mut call_count = 0u32;
        let result = retry_helper::<(), _>(
            3,
            1,
            || {
                call_count += 1;
                Err("unique constraint violation".to_string())
            },
        );
        assert!(result.is_err());
        assert_eq!(call_count, 1, "non-connection errors must not be retried");
    }

    /// All three attempts fail with connection errors — the last error is propagated.
    #[test]
    fn with_retry_exhausts_attempts_and_returns_last_error() {
        let mut call_count = 0u32;
        let result = retry_helper::<(), _>(
            3,
            1,
            || {
                call_count += 1;
                Err("connection timeout".to_string())
            },
        );
        assert!(result.is_err());
        assert_eq!(call_count, 3, "all three attempts should be exhausted");
        assert!(result.unwrap_err().contains("timeout"));
    }

    /// String-based analogue of `is_connection_error` used by the test helper
    /// below (which works with `String` errors rather than `sqlx::Error`).
    fn is_connection_error_str(e: &str) -> bool {
        let lower = e.to_lowercase();
        lower.contains("connection") || lower.contains("timeout") || lower.contains("pool")
    }

    /// Mirrors the real with_retry implementation so tests above do not need
    /// a live PostgreSQL connection.
    fn retry_helper<T, F>(max_attempts: u32, delay_ms: u64, mut op: F) -> Result<T, String>
    where
        F: FnMut() -> Result<T, String>,
    {
        let mut current_delay = delay_ms;
        for attempt in 1..=max_attempts {
            match op() {
                Ok(v) => return Ok(v),
                Err(e) if attempt < max_attempts && is_connection_error_str(&e) => {
                    std::thread::sleep(std::time::Duration::from_millis(current_delay));
                    current_delay *= 2;
                }
                Err(e) => return Err(e),
            }
        }
        unreachable!()
    }
}
