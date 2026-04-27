use crate::two_factor::{TwoFactorData, TwoFactorStore};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use tokio::runtime::Runtime;

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
}
