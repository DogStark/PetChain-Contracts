//! Advisory-lock migration runner ported from the removed `backend-2fa-implementation`
//! crate (Issue #1217). Unlike `migrations`, it serialises concurrent runners via a
//! Postgres advisory lock and verifies checksums of applied migrations. It is kept as
//! a separate module so no behaviour of the canonical runner changes; consolidating
//! the two runners is follow-up work.
//!
//! ## Lock strategy (Issue #1663)
//!
//! The migration lock is **transaction-scoped** (`pg_advisory_xact_lock`), not
//! session-scoped (`pg_try_advisory_lock`). The lock is acquired inside the same
//! transaction that performs the migration work, so Postgres releases it
//! automatically when that transaction commits or rolls back. This guarantees the
//! lock is freed even if the process panics mid-migration or is killed with
//! `SIGKILL` (OOM killer, container restart) — cases where a `Drop` impl would never
//! run and a session-scoped lock could linger on a recycled pooled connection.
//!
//! Because the lock lifetime is tied to the transaction, the executor must run the
//! migration inside a single transaction: `begin_transaction` opens it,
//! `acquire_advisory_lock` issues `pg_advisory_xact_lock`, and `commit_transaction`
//! (or `rollback_transaction` on error) ends it, releasing the lock.

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
    /// Opens the transaction that will hold the transaction-scoped advisory lock.
    fn begin_transaction(&self) -> Result<(), String>;
    /// Acquires a transaction-scoped advisory lock (`pg_advisory_xact_lock`).
    /// The lock is released automatically when the surrounding transaction ends.
    fn acquire_advisory_lock(&self, lock_id: i64) -> Result<(), String>;
    /// Commits the transaction, which releases the transaction-scoped advisory lock.
    fn commit_transaction(&self) -> Result<(), String>;
    /// Rolls back the transaction, which releases the transaction-scoped advisory lock.
    fn rollback_transaction(&self) -> Result<(), String>;
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
        // Open the transaction that will own the transaction-scoped advisory lock.
        self.executor
            .begin_transaction()
            .map_err(MigrationError::LockError)?;

        // `pg_advisory_xact_lock` blocks until the lock is available and is released
        // automatically when this transaction commits or rolls back — including on
        // panic or SIGKILL, since Postgres ends the transaction with the session.
        if let Err(err) = self.executor.acquire_advisory_lock(MIGRATION_LOCK_ID) {
            let _ = self.executor.rollback_transaction();
            return Err(MigrationError::LockError(err));
        }

        match self.apply_pending_inner() {
            Ok(applied) => {
                self.executor
                    .commit_transaction()
                    .map_err(MigrationError::ExecutionError)?;
                Ok(applied)
            }
            Err(err) => {
                // Roll back so the transaction-scoped lock is released even on failure.
                let _ = self.executor.rollback_transaction();
                Err(err)
            }
        }
    }

    fn apply_pending_inner(&self) -> Result<Vec<u32>, MigrationError> {
        let applied = self
            .executor
            .query_applied_migrations()
            .map_err(MigrationError::ExecutionError)?;

        for migration in &self.migrations {
            if let Some(stored_checksum) = applied.get(&migration.version) {
                if stored_checksum != &migration.checksum {
                    return Err(MigrationError::DirtyVersion(migration.version));
                }
            }
        }

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

        fn begin_transaction(&self) -> Result<(), String> {
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

        fn commit_transaction(&self) -> Result<(), String> {
            // Transaction-scoped lock is released on commit.
            *self.lock_held.lock().unwrap() = false;
            Ok(())
        }

        fn rollback_transaction(&self) -> Result<(), String> {
            // Transaction-scoped lock is released on rollback.
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

        fn begin_transaction(&self) -> Result<(), String> {
            (**self).begin_transaction()
        }

        fn acquire_advisory_lock(&self, lock_id: i64) -> Result<(), String> {
            (**self).acquire_advisory_lock(lock_id)
        }

        fn commit_transaction(&self) -> Result<(), String> {
            (**self).commit_transaction()
        }

        fn rollback_transaction(&self) -> Result<(), String> {
            (**self).rollback_transaction()
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

            fn begin_transaction(&self) -> Result<(), String> {
                Ok(())
            }

            fn acquire_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = true;
                Ok(())
            }

            fn commit_transaction(&self) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = false;
                Ok(())
            }

            fn rollback_transaction(&self) -> Result<(), String> {
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
    fn apply_pending_releases_lock_on_mid_migration_failure() {
        // Simulates a failure partway through applying migrations (e.g. a DB error
        // on the second migration). The transaction-scoped lock must be released
        // via rollback so a subsequent deployment can acquire it.
        struct MidFailureExecutor {
            lock_held: Mutex<bool>,
            executed: Mutex<Vec<String>>,
        }

        impl SqlExecutor for MidFailureExecutor {
            fn execute(&self, sql: &str) -> Result<(), String> {
                let mut executed = self.executed.lock().unwrap();
                executed.push(sql.to_string());
                if sql.contains("BAD") {
                    return Err("execution failed mid-migration".to_string());
                }
                Ok(())
            }

            fn query_applied_migrations(&self) -> Result<HashMap<u32, String>, String> {
                Ok(HashMap::new())
            }

            fn record_migration(&self, _version: u32, _checksum: &str) -> Result<(), String> {
                Ok(())
            }

            fn begin_transaction(&self) -> Result<(), String> {
                Ok(())
            }

            fn acquire_advisory_lock(&self, _lock_id: i64) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = true;
                Ok(())
            }

            fn commit_transaction(&self) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = false;
                Ok(())
            }

            fn rollback_transaction(&self) -> Result<(), String> {
                *self.lock_held.lock().unwrap() = false;
                Ok(())
            }
        }

        let executor = MidFailureExecutor {
            lock_held: Mutex::new(false),
            executed: Mutex::new(Vec::new()),
        };
        let migrations = vec![
            Migration::new(1, "create users", "CREATE TABLE users (id INT)"),
            Migration::new(2, "will fail", "BAD SQL"),
        ];
        let runner = MigrationRunner::new(executor, migrations);

        assert!(runner.apply_pending().is_err());
        // Lock must be released after the mid-migration failure.
        assert!(!*runner.executor.lock_held.lock().unwrap());
        // The first migration ran before the failure.
        assert_eq!(runner.executor.executed.lock().unwrap().len(), 2);
    }
}
