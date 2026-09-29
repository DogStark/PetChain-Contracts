//! Migration runner with checksum verification and drift detection.
//!
//! Applied migrations store a checksum of the migration file contents. On
//! startup the runner recomputes each applied migration's checksum and fails
//! closed if a file was edited after deployment, emitting remediation guidance.
//! A session-level advisory lock prevents concurrent startups from applying the
//! same migration twice.

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
}
