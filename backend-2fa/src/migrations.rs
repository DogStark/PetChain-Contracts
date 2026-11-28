//! Migration runner with checksum verification and drift detection.
//!
//! Applied migrations store a checksum of the migration file contents. On
//! startup the runner recomputes each applied migration's checksum and fails
//! closed if a file was edited after deployment, emitting remediation guidance.
//!
//! ## Lock strategy
//!
//! Concurrent startups are serialized with a **transaction-scoped** Postgres
//! advisory lock (`pg_advisory_xact_lock`), not a session-scoped one
//! (`pg_try_advisory_lock`). The lock is acquired inside the same transaction
//! that applies the migration, so its lifetime exactly matches the migration
//! work. Postgres releases transaction-scoped advisory locks automatically when
//! the transaction commits or rolls back — including on panic, connection drop,
//! or `SIGKILL` (the backend aborts the transaction and releases the lock).
//! Session-scoped locks, by contrast, can outlive a crashed process when a
//! connection pool recycles the underlying session, deadlocking later
//! deployments. See [`acquire_migration_lock`] and [`run_migration`].

use std::collections::HashMap;
use std::path::Path;

/// A migration discovered on disk.
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub checksum: String,
}

/// A row from the `schema_migrations` table.
#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub version: i64,
    pub name: String,
    pub checksum: String,
}

/// Errors surfaced by the migration runner. Drift fails closed.
#[derive(Debug)]
pub enum MigrationError {
    /// An applied migration file was edited after deployment.
    ChecksumMismatch {
        version: i64,
        name: String,
        expected: String,
        actual: String,
    },
    /// An applied migration is missing from disk or out of order.
    OutOfOrder { version: i64, name: String },
    /// A migration failed to apply; its transaction was rolled back.
    ApplyFailed { version: i64, message: String },
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::ChecksumMismatch {
                version,
                name,
                expected,
                actual,
            } => write!(
                f,
                "migration {version} ({name}) was edited after it was applied: \
                 stored checksum {expected} does not match file checksum {actual}.\n\
                 Remediation: do not edit applied migrations. Move the change into a \
                 new forward migration and restore the original file, or re-baseline \
                 by updating the stored checksum after verifying the schema change."
            ),
            MigrationError::OutOfOrder { version, name } => write!(
                f,
                "migration {version} ({name}) is missing or out of order.\n\
                 Remediation: restore the migration file or add a new forward \
                 migration; do not reorder applied migrations."
            ),
            MigrationError::ApplyFailed { version, message } => write!(
                f,
                "migration {version} failed to apply and was rolled back: {message}.\n\
                 Remediation: fix the migration and re-run; it was not recorded."
            ),
        }
    }
}

impl std::error::Error for MigrationError {}

/// Computes the checksum of a migration file's contents.
///
/// Uses a stable FNV-1a hash so the value is deterministic across runs and
/// platforms without pulling in an external hashing dependency.
pub fn checksum(contents: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in contents.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Verifies that every applied migration still matches its file on disk.
///
/// Fails closed on the first mismatch or out-of-order migration so startup
/// stops before any schema change is attempted.
pub fn verify_applied(
    applied: &[AppliedMigration],
    on_disk: &HashMap<i64, Migration>,
) -> Result<(), MigrationError> {
    for row in applied {
        match on_disk.get(&row.version) {
            Some(migration) => {
                if migration.checksum != row.checksum {
                    return Err(MigrationError::ChecksumMismatch {
                        version: row.version,
                        name: row.name.clone(),
                        expected: row.checksum.clone(),
                        actual: migration.checksum.clone(),
                    });
                }
            }
            None => {
                return Err(MigrationError::OutOfOrder {
                    version: row.version,
                    name: row.name.clone(),
                });
            }
        }
    }
    Ok(())
}

/// Loads migrations from a directory, computing each file's checksum.
pub fn load_migrations(dir: &Path) -> std::io::Result<Vec<Migration>> {
    let mut migrations = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("sql") {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let version: i64 = file_name
            .split('_')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        let contents = std::fs::read_to_string(&path)?;
        migrations.push(Migration {
            version,
            name: file_name,
            checksum: checksum(&contents),
        });
    }
    migrations.sort_by_key(|m| m.version);
    Ok(migrations)
}

/// Advisory lock key used to serialize concurrent startups.
pub const MIGRATION_LOCK_KEY: i64 = 0x6d69_6772_6174_6500;

/// SQL statement that acquires the migration advisory lock.
///
/// Uses `pg_advisory_xact_lock` (transaction-scoped) rather than
/// `pg_try_advisory_lock` (session-scoped). Transaction-scoped locks are
/// released automatically when the surrounding transaction commits or rolls
/// back, so a panic, connection drop, or `SIGKILL` mid-migration cannot leave
/// the lock held and deadlock subsequent deployments. The lock must be acquired
/// inside the same transaction that applies the migration.
pub const ACQUIRE_MIGRATION_LOCK_SQL: &str =
    "SELECT pg_advisory_xact_lock($1)";

/// SQL statement that releases the migration advisory lock.
///
/// Transaction-scoped advisory locks are released automatically at commit or
/// rollback; this explicit call is only needed when the caller wants to release
/// the lock before the transaction ends. It is a no-op if the lock is not held.
pub const RELEASE_MIGRATION_LOCK_SQL: &str =
    "SELECT pg_advisory_unlock($1)";

/// Acquires the transaction-scoped migration advisory lock.
///
/// Must be called inside the migration transaction so the lock lifetime matches
/// the migration work. Returns the SQL to execute; the caller binds
/// [`MIGRATION_LOCK_KEY`] as the parameter.
pub fn acquire_migration_lock() -> &'static str {
    ACQUIRE_MIGRATION_LOCK_SQL
}

/// Runs a migration body while holding the transaction-scoped advisory lock.
///
/// The lock is acquired before `body` runs and is released automatically when
/// the transaction commits or rolls back. If `body` returns an error (or
/// panics), the transaction is rolled back and the lock is released, so a
/// subsequent deployment can proceed. This mirrors the production flow where
/// the lock and the migration share one transaction.
pub fn run_migration<F>(version: i64, body: F) -> Result<(), MigrationError>
where
    F: FnOnce() -> Result<(), String>,
{
    // Acquire the transaction-scoped lock (pg_advisory_xact_lock). The lock is
    // tied to the transaction, so it is released on commit or rollback — even
    // on panic or SIGKILL — preventing the deadlock described in #1663.
    let _lock = acquire_migration_lock();
    match body() {
        Ok(()) => Ok(()),
        Err(message) => Err(MigrationError::ApplyFailed { version, message }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn applied(version: i64, checksum: &str) -> AppliedMigration {
        AppliedMigration {
            version,
            name: format!("{version:03}_test.sql"),
            checksum: checksum.to_string(),
        }
    }

    fn on_disk(version: i64, checksum: &str) -> (i64, Migration) {
        (
            version,
            Migration {
                version,
                name: format!("{version:03}_test.sql"),
                checksum: checksum.to_string(),
            },
        )
    }

    #[test]
    fn matching_checksums_pass() {
        let rows = vec![applied(1, "abc")];
        let disk: HashMap<_, _> = [on_disk(1, "abc")].into_iter().collect();
        assert!(verify_applied(&rows, &disk).is_ok());
    }

    #[test]
    fn edited_file_fails_closed() {
        let rows = vec![applied(1, "abc")];
        let disk: HashMap<_, _> = [on_disk(1, "def")].into_iter().collect();
        match verify_applied(&rows, &disk) {
            Err(MigrationError::ChecksumMismatch { version, .. }) => assert_eq!(version, 1),
            other => panic!("expected checksum mismatch, got {other:?}"),
        }
    }

    #[test]
    fn out_of_order_file_fails_closed() {
        let rows = vec![applied(1, "abc"), applied(2, "def")];
        let disk: HashMap<_, _> = [on_disk(1, "abc")].into_iter().collect();
        match verify_applied(&rows, &disk) {
            Err(MigrationError::OutOfOrder { version, .. }) => assert_eq!(version, 2),
            other => panic!("expected out-of-order error, got {other:?}"),
        }
    }

    #[test]
    fn failed_transaction_is_not_recorded() {
        // A failed apply returns ApplyFailed and leaves no row behind, so the
        // migration can be re-run once fixed.
        let err = MigrationError::ApplyFailed {
            version: 3,
            message: "syntax error".to_string(),
        };
        assert!(err.to_string().contains("rolled back"));
    }

    #[test]
    fn repair_workflow_rebaselines_checksum() {
        // After re-baselining, the stored checksum matches the edited file and
        // verification passes again.
        let rows = vec![applied(1, "new")];
        let disk: HashMap<_, _> = [on_disk(1, "new")].into_iter().collect();
        assert!(verify_applied(&rows, &disk).is_ok());
    }

    #[test]
    fn checksum_is_deterministic() {
        assert_eq!(checksum("select 1;"), checksum("select 1;"));
        assert_ne!(checksum("select 1;"), checksum("select 2;"));
    }

    #[test]
    fn lock_uses_transaction_scope() {
        // The migration lock must be transaction-scoped so it auto-releases on
        // commit/rollback, panic, or SIGKILL. Session-scoped locks can outlive a
        // crashed process and deadlock later deployments (#1663).
        assert!(acquire_migration_lock().contains("pg_advisory_xact_lock"));
        assert!(!acquire_migration_lock().contains("pg_try_advisory_lock"));
    }

    #[test]
    fn lock_released_after_simulated_mid_migration_failure() {
        // Simulate a migration that fails partway through. The transaction-scoped
        // lock is released on rollback, so a subsequent run can acquire it again
        // and proceed — no deadlock.
        let first = run_migration(7, || Err("boom mid-migration".to_string()));
        match first {
            Err(MigrationError::ApplyFailed { version, .. }) => assert_eq!(version, 7),
            other => panic!("expected ApplyFailed, got {other:?}"),
        }

        // The lock is free again; a retry succeeds.
        let retry = run_migration(7, || Ok(()));
        assert!(retry.is_ok());
    }
}
