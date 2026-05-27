use crate::two_factor::{RecoveryCodeUsageLog, TwoFactorData, TwoFactorStore};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::collections::HashMap;

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
    match std::env::var("SECRET_PROVIDER").unwrap_or_else(|_| "env".to_string()).as_str() {
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
        let pool = runtime
            .block_on(PgPoolOptions::new().connect(database_url))
            .map_err(|e| e.to_string())?;

        Ok(Self { pool, runtime })
    }

    /// Connect using a SecretProvider to fetch the `secret_key` value.
    pub fn connect_with_provider(provider: &dyn SecretProvider, secret_key: &str) -> Result<Self, String> {
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
}

impl TwoFactorStore for PostgresTwoFactorStore {
    fn save(&self, user_id: &str, data: TwoFactorData) -> Result<(), String> {
        let backup_codes = serde_json::to_string(&data.backup_codes).map_err(|e| e.to_string())?;

        self.block_on(
            sqlx::query(
                r#"
            INSERT INTO user_two_factor (user_id, secret, backup_codes, enabled)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (user_id)
            DO UPDATE SET
                secret = EXCLUDED.secret,
                backup_codes = EXCLUDED.backup_codes,
                enabled = EXCLUDED.enabled,
                updated_at = CURRENT_TIMESTAMP
            "#,
            )
            .bind(user_id)
            .bind(data.secret)
            .bind(backup_codes)
            .bind(data.enabled)
            .execute(&self.pool),
        )?;

        Ok(())
    }

    fn get(&self, user_id: &str) -> Result<TwoFactorData, String> {
        let row = self.block_on(
            sqlx::query_as::<_, (String, String, bool)>(
                r#"
            SELECT secret, backup_codes, enabled
            FROM user_two_factor
            WHERE user_id = $1
            "#,
            )
            .bind(user_id)
            .fetch_optional(&self.pool),
        )?;

        let (secret, backup_codes, enabled) =
            row.ok_or_else(|| format!("No 2FA data found for user: {}", user_id))?;
        let backup_codes = serde_json::from_str(&backup_codes).map_err(|e| e.to_string())?;

        Ok(TwoFactorData {
            secret,
            backup_codes,
            enabled,
        })
    }

    fn delete(&self, user_id: &str) -> Result<(), String> {
        let result = self.block_on(
            sqlx::query("DELETE FROM user_two_factor WHERE user_id = $1")
                .bind(user_id)
                .execute(&self.pool),
        )?;

        if result.rows_affected() == 0 {
            return Err(format!("No 2FA data found for user: {}", user_id));
        }

        Ok(())
    }

    fn update_enabled(&self, user_id: &str, enabled: bool) -> Result<(), String> {
        let result = self.block_on(
            sqlx::query(
                r#"
                UPDATE user_two_factor
                SET enabled = $2, updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .bind(enabled)
            .execute(&self.pool),
        )?;

        if result.rows_affected() == 0 {
            return Err(format!("No 2FA data found for user: {}", user_id));
        }

        Ok(())
    }

    fn update_backup_codes(&self, user_id: &str, codes: Vec<String>) -> Result<(), String> {
        let backup_codes = serde_json::to_string(&codes).map_err(|e| e.to_string())?;
        let result = self.block_on(
            sqlx::query(
                r#"
                UPDATE user_two_factor
                SET backup_codes = $2, updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $1
                "#,
            )
            .bind(user_id)
            .bind(backup_codes)
            .execute(&self.pool),
        )?;

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
        let result = self.block_on(
            sqlx::query(
                r#"
                INSERT INTO recovery_code_usage (user_id, code_index, used_at, ip_address)
                VALUES ($1, $2, CURRENT_TIMESTAMP, $3)
                "#,
            )
            .bind(user_id)
            .bind(code_index)
            .bind(ip_address)
            .execute(&self.pool),
        );

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

        let rows = self.block_on(
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
        )?;

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

}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_data() -> TwoFactorData {
        TwoFactorData {
            secret: "JBSWY3DPEHPK3PXP".to_string(),
            backup_codes: vec!["1111-2222".to_string(), "3333-4444".to_string()],
            enabled: false,
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
}
