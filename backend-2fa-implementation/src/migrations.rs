use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

const MIGRATION_LOCK_ID: i64 = 0x5065_7443_6861_696E;

#[derive(Debug)]
pub enum MigrationError {
    ExecutionError(String),
    DirtyVersion(u32),
    LockError(String),
}

impl fmt::Display for MigrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationError::ExecutionError(msg) => write!(f, "migration execution error: {}", msg),
            MigrationError::DirtyVersion(v) => {
                write!(f, "migration {} is dirty (checksum mismatch)", v)
            }
            MigrationError::LockError(msg) => write!(f, "migration lock error: {}", msg),
        }
    }
}

pub trait SqlExecutor {
    fn execute(&self, sql: &str) -> Result<(), String>;
    fn query_applied_migrations(&self) -> Result<HashMap<u32, String>, String>;
    fn record_migration(&self, version: u32, checksum: &str) -> Result<(), String>;
    fn acquire_advisory_lock(&self, lock_id: i64) -> Result<(), String>;
    fn release_advisory_lock(&self, lock_id: i64) -> Result<(), String>;
}

#[derive(Clone, Debug)]
pub struct Migration {
    pub version: u32,
    pub description: String,
    pub sql: String,
    pub checksum: String,
}

impl Migration {
    pub fn new(version: u32, description: &str, sql: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(sql.as_bytes());
        let checksum = format!("{:x}", hasher.finalize());
        Self {
            version,
            description: description.to_string(),
            sql: sql.to_string(),
            checksum,
        }
    }
}

pub struct MigrationRunner<E: SqlExecutor> {
    executor: E,
    migrations: Vec<Migration>,
}

impl<E: SqlExecutor> MigrationRunner<E> {
    pub fn new(executor: E, migrations: Vec<Migration>) -> Self {
        Self {
            executor,
            migrations,
        }
    }

    pub fn apply_pending(&self) -> Result<Vec<u32>, MigrationError> {
        self.executor
            .acquire_advisory_lock(MIGRATION_LOCK_ID)
            .map_err(MigrationError::LockError)?;

        let result = self.apply_pending_inner();

        let _ = self.executor.release_advisory_lock(MIGRATION_LOCK_ID);

        result
    }

    fn apply_pending_inner(&self) -> Result<Vec<u32>, MigrationError> {
        let applied = self
            .executor
            .query_applied_migrations()
            .map_err(MigrationError::ExecutionError)?;

        let mut applied_versions = Vec::new();

        for migration in &self.migrations {
            if applied.contains_key(&migration.version) {
                continue;
            }

            self.executor
                .execute(&migration.sql)
                .map_err(MigrationError::ExecutionError)?;

            self.executor
                .record_migration(migration.version, &migration.checksum)
                .map_err(MigrationError::ExecutionError)?;

            applied_versions.push(migration.version);
        }

        Ok(applied_versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockExecutor {
        applied: Mutex<HashMap<u32, String>>,
        lock_held: Mutex<bool>,
        executed_sql: Mutex<Vec<String>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                applied: Mutex::new(HashMap::new()),
                lock_held: Mutex::new(false),
                executed_sql: Mutex::new(Vec::new()),
            }
        }

        fn with_applied(applied: HashMap<u32, String>) -> Self {
            Self {
                applied: Mutex::new(applied),
                lock_held: Mutex::new(false),
                executed_sql: Mutex::new(Vec::new()),
            }
        }
    }

    impl SqlExecutor for MockExecutor {
        fn execute(&self, sql: &str) -> Result<(), String> {
            self.executed_sql.lock().unwrap().push(sql.to_string());
            Ok(())
        }

        fn query_applied_migrations(&self) -> Result<HashMap<u32, String>, String> {
            Ok(self.applied.lock().unwrap().clone())
        }

        fn record_migration(&self, version: u32, checksum: &str) -> Result<(), String> {
            self.applied
                .lock()
                .unwrap()
                .insert(version, checksum.to_string());
            Ok(())
        }

        fn acquire_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
            let mut held = self.lock_held.lock().unwrap();
            if *held {
                return Err("lock already held".to_string());
            }
            *held = true;
            Ok(())
        }

        fn release_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
            *self.lock_held.lock().unwrap() = false;
            Ok(())
        }
    }

    impl SqlExecutor for Arc<MockExecutor> {
        fn execute(&self, sql: &str) -> Result<(), String> {
            (**self).execute(sql)
        }

        fn query_applied_migrations(&self) -> Result<HashMap<u32, String>, String> {
            (**self).query_applied_migrations()
        }

        fn record_migration(&self, version: u32, checksum: &str) -> Result<(), String> {
            (**self).record_migration(version, checksum)
        }

        fn acquire_advisory_lock(&self, lock_id: i64) -> Result<(), String> {
            (**self).acquire_advisory_lock(lock_id)
        }

        fn release_advisory_lock(&self, lock_id: i64) -> Result<(), String> {
            (**self).release_advisory_lock(lock_id)
        }
    }

    #[test]
    fn apply_pending_acquires_and_releases_lock() {
        let executor = MockExecutor::new();
        let migrations = vec![Migration::new(1, "create users", "CREATE TABLE users (id INT)")];
        let runner = MigrationRunner::new(executor, migrations);

        let applied = runner.apply_pending().unwrap();
        assert_eq!(applied, vec![1]);
        assert!(!*runner.executor.lock_held.lock().unwrap());
    }

    #[test]
    fn apply_pending_releases_lock_on_error() {
        struct FailingExecutor {
            lock_held: Mutex<bool>,
        }

        impl SqlExecutor for FailingExecutor {
            fn execute(&self, _sql: &str) -> Result<(), String> {
                Err("execution failed".to_string())
            }

            fn query_applied_migrations(&self) -> Result<HashMap<u32, String>, String> {
                Ok(HashMap::new())
            }

            fn record_migration(&self, _version: u32, _checksum: &str) -> Result<(), String> {
                Ok(())
            }

            fn acquire_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = true;
                Ok(())
            }

            fn release_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = false;
                Ok(())
            }
        }

        let executor = FailingExecutor {
            lock_held: Mutex::new(false),
        };
        let migrations = vec![Migration::new(1, "will fail", "BAD SQL")];
        let runner = MigrationRunner::new(executor, migrations);

        assert!(runner.apply_pending().is_err());
        assert!(!*runner.executor.lock_held.lock().unwrap());
    }

    #[test]
    fn concurrent_runners_serialize_via_lock() {
        let shared = Arc::new(MockExecutor::new());
        let migrations = vec![Migration::new(1, "create table", "CREATE TABLE t (id INT)")];

        let runner1 = MigrationRunner::new(Arc::clone(&shared), migrations.clone());
        let runner2 = MigrationRunner::new(Arc::clone(&shared), migrations);

        let result1 = runner1.apply_pending().unwrap();
        assert_eq!(result1, vec![1]);

        let result2 = runner2.apply_pending().unwrap();
        assert!(result2.is_empty(), "second runner should find migration already applied");
    }

    #[test]
    fn skips_already_applied_migrations() {
        let m = Migration::new(1, "create users", "CREATE TABLE users (id INT)");
        let mut pre_applied = HashMap::new();
        pre_applied.insert(1, m.checksum.clone());

        let executor = MockExecutor::with_applied(pre_applied);
        let runner = MigrationRunner::new(executor, vec![m]);

        let applied = runner.apply_pending().unwrap();
        assert!(applied.is_empty());
        assert!(runner.executor.executed_sql.lock().unwrap().is_empty());
    }
}
