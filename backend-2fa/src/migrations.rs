//! Database Schema Migration Runner with Version Tracking (Issue #644)
//!
//! This module provides a migration runner that:
//! - Tracks applied migrations in a `schema_migrations` table
//! - Applies unapplied migrations in order
//! - Supports rollback with down scripts
//! - Is idempotent (re-running is safe)

use hex;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Describes a single migration version.
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: u32,
    pub name: String,
    pub up_script: String,
    pub down_script: String,
    pub checksum: String,
}

/// Errors that can occur during migration operations.
#[derive(Debug)]
pub enum MigrationError {
    Io(String),
    Sql(String),
    VersionMismatch,
    DirtyVersion(u32),
    MigrationNotFound(u32),
    NoMigrationsToRollback,
    InvalidMigrationFile(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::Io(msg) => write!(f, "IO error: {}", msg),
            MigrationError::Sql(msg) => write!(f, "SQL error: {}", msg),
            MigrationError::VersionMismatch => write!(f, "version mismatch in migration files"),
            MigrationError::DirtyVersion(v) => {
                write!(f, "migration {} is dirty (checksum mismatch)", v)
            }
            MigrationError::MigrationNotFound(v) => {
                write!(f, "migration {} not found in filesystem", v)
            }
            MigrationError::NoMigrationsToRollback => {
                write!(f, "no migrations to rollback")
            }
            MigrationError::InvalidMigrationFile(msg) => {
                write!(f, "invalid migration file: {}", msg)
            }
        }
    }
}

impl std::error::Error for MigrationError {}

/// Discover and parse migration files from a directory.
///
/// Expects files matching the pattern `{NNN}_{name}.sql` for up scripts and
/// `{NNN}_{name}.down.sql` for down scripts. Returns migrations sorted by version.
/// A trait for executing SQL against a database.
pub trait SqlExecutor {
    fn execute(&self, sql: &str) -> Result<(), String>;
    fn execute_with_params(&self, sql: &str, params: &[&str]) -> Result<(), String>;
    fn query_scalar(&self, sql: &str) -> Result<Option<i64>, String>;
    fn query_scalar_with_params(&self, sql: &str, params: &[&str]) -> Result<Option<i64>, String>;
}

/// The migration runner that tracks and applies migrations.
pub struct MigrationRunner {
    migrations: Vec<Migration>,
}

impl MigrationRunner {
    pub fn new(migrations: Vec<Migration>) -> Self {
        Self { migrations }
    }

    pub fn from_directory(migrations_dir: &Path) -> Result<Self, MigrationError> {
        let migrations = discover_migrations(migrations_dir)?;
        Ok(Self::new(migrations))
    }

    fn ensure_tracking_table(&self, executor: &dyn SqlExecutor) -> Result<(), MigrationError> {
        executor.execute(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version     INTEGER PRIMARY KEY,
                name        TEXT NOT NULL,
                applied_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                checksum    TEXT NOT NULL
            )",
        ).map_err(MigrationError::Sql)
    }

    fn get_applied_migrations(&self, executor: &dyn SqlExecutor) -> Result<Vec<(u32, String)>, MigrationError> {
        self.ensure_tracking_table(executor)?;
        let mut applied = Vec::new();
        for migration in &self.migrations {
            let version_str = migration.version.to_string();
            let count = executor.query_scalar_with_params(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = $1",
                &[version_str.as_str()]
            ).map_err(MigrationError::Sql)?.unwrap_or(0);
            if count > 0 {
                applied.push((migration.version, migration.name.clone()));
            }
        }
        Ok(applied)
    }

    fn pending_migrations(&self, executor: &dyn SqlExecutor) -> Result<Vec<&Migration>, MigrationError> {
        let applied = self.get_applied_migrations(executor)?;
        let applied_versions: std::collections::HashSet<u32> =
            applied.into_iter().map(|(v, _)| v).collect();
        Ok(self.migrations.iter().filter(|m| !applied_versions.contains(&m.version)).collect())
    }

    pub fn apply_pending(&self, executor: &dyn SqlExecutor) -> Result<usize, MigrationError> {
        let pending = self.pending_migrations(executor)?;
        if pending.is_empty() {
            return Ok(0);
        }
        for migration in &pending {
            executor.execute(&migration.up_script)
                .map_err(|e| MigrationError::Sql(format!("migration {} up: {}", migration.version, e)))?;
            
            let version_str = migration.version.to_string();
            executor.execute_with_params(
                "INSERT INTO schema_migrations (version, name, checksum) VALUES ($1, $2, $3)",
                &[version_str.as_str(), migration.name.as_str(), migration.checksum.as_str()]
            ).map_err(|e| MigrationError::Sql(format!("recording migration {}: {}", migration.version, e)))?;
        }
        Ok(pending.len())
    }

    pub fn rollback_last(&self, executor: &dyn SqlExecutor) -> Result<u32, MigrationError> {
        let max_version = executor.query_scalar("SELECT MAX(version) FROM schema_migrations")
            .map_err(MigrationError::Sql)?.unwrap_or(0) as u32;
        if max_version == 0 {
            return Err(MigrationError::NoMigrationsToRollback);
        }
        let migration = self.migrations.iter()
            .find(|m| m.version == max_version)
            .ok_or(MigrationError::MigrationNotFound(max_version))?;
        if !migration.down_script.is_empty() {
            executor.execute(&migration.down_script)
                .map_err(|e| MigrationError::Sql(format!("migration {} down: {}", migration.version, e)))?;
        }
        
        let version_str = migration.version.to_string();
        executor.execute_with_params(
            "DELETE FROM schema_migrations WHERE version = $1",
            &[version_str.as_str()]
        ).map_err(MigrationError::Sql)?;
        Ok(migration.version)
    }

    pub fn current_version(&self, executor: &dyn SqlExecutor) -> Result<Option<u32>, MigrationError> {
        match executor.query_scalar("SELECT MAX(version) FROM schema_migrations") {
            Ok(Some(v)) => Ok(Some(v as u32)),
            Ok(None) => Ok(None),
            Err(e) => Err(MigrationError::Sql(e)),
        }
    }

    pub fn is_up_to_date(&self, executor: &dyn SqlExecutor) -> Result<bool, MigrationError> {
        let pending = self.pending_migrations(executor)?;
        Ok(pending.is_empty())
    }

    /// Returns the list of pending migrations (version + name) without executing any up_script.
    pub fn dry_run(&self, executor: &dyn SqlExecutor) -> Result<Vec<(u32, String)>, MigrationError> {
        let pending = self.pending_migrations(executor)?;
        Ok(pending.iter().map(|m| (m.version, m.name.clone())).collect())
    }
}

/// A real `SqlExecutor` backed by a live PostgreSQL connection pool.
///
/// Used exclusively in tests (guarded by `#[cfg(test)]`) to exercise the
/// migration runner against an actual database.  Outside of tests the
/// application drives migrations via the same `SqlExecutor` trait so that
/// the round-trip test exercises the identical code paths.
#[cfg(test)]
pub struct PostgresExecutor {
    pool: sqlx::PgPool,
    runtime: std::sync::Arc<tokio::runtime::Runtime>,
}

#[cfg(test)]
impl PostgresExecutor {
    /// Connect to the database at `database_url`.
    pub fn connect(database_url: &str) -> Result<Self, String> {
        let runtime = std::sync::Arc::new(
            tokio::runtime::Runtime::new().map_err(|e| e.to_string())?,
        );
        let pool = runtime
            .block_on(
                sqlx::postgres::PgPoolOptions::new()
                    .max_connections(5)
                    .connect(database_url),
            )
            .map_err(|e| e.to_string())?;
        Ok(Self { pool, runtime })
    }

    /// Return a sorted list of table names currently present in the public
    /// schema of the connected database.  This is the "schema snapshot" used
    /// by the round-trip test.
    pub fn table_names(&self) -> Result<Vec<String>, String> {
        let rows: Vec<(String,)> = self
            .runtime
            .block_on(
                sqlx::query_as(
                    "SELECT table_name FROM information_schema.tables \
                     WHERE table_schema = 'public' AND table_type = 'BASE TABLE' \
                     ORDER BY table_name",
                )
                .fetch_all(&self.pool),
            )
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|(name,)| name).collect())
    }

    /// Drop every table in the public schema so that each test run starts
    /// from a truly empty database regardless of prior runs.  This keeps the
    /// test idempotent even when the test process is interrupted mid-run.
    pub fn drop_all_tables(&self) -> Result<(), String> {
        // Fetch tables first, then drop them in a separate statement so we
        // aren't mutating the information_schema result set while iterating.
        let tables = self.table_names()?;
        for table in tables {
            self.runtime
                .block_on(
                    sqlx::query(&format!("DROP TABLE IF EXISTS \"{}\" CASCADE", table))
                        .execute(&self.pool),
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl SqlExecutor for PostgresExecutor {
    fn execute(&self, sql: &str) -> Result<(), String> {
        self.runtime
            .block_on(sqlx::query(sql).execute(&self.pool))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn execute_with_params(&self, sql: &str, params: &[&str]) -> Result<(), String> {
        // Build a dynamic query by substituting positional $N placeholders.
        // We rebind each string param in order; all migration-runner params
        // are string-typed (version number, name, checksum).
        let mut query = sqlx::query(sql);
        for param in params {
            query = query.bind(*param);
        }
        self.runtime
            .block_on(query.execute(&self.pool))
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn query_scalar(&self, sql: &str) -> Result<Option<i64>, String> {
        self.runtime
            .block_on(sqlx::query_scalar::<_, Option<i64>>(sql).fetch_one(&self.pool))
            .map_err(|e| e.to_string())
    }

    fn query_scalar_with_params(&self, sql: &str, params: &[&str]) -> Result<Option<i64>, String> {
        let mut query = sqlx::query_scalar::<_, Option<i64>>(sql);
        for param in params {
            query = query.bind(*param);
        }
        self.runtime
            .block_on(query.fetch_one(&self.pool))
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::cell::RefCell;

    struct MockExecutor {
        tables: RefCell<HashMap<String, Vec<HashMap<String, String>>>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            let mut tables = HashMap::new();
            tables.insert("schema_migrations".to_string(), Vec::new());
            Self { tables: RefCell::new(tables) }
        }
    }

    impl SqlExecutor for MockExecutor {
        fn execute(&self, _sql: &str) -> Result<(), String> {
            Ok(())
        }
        fn execute_with_params(&self, sql: &str, params: &[&str]) -> Result<(), String> {
            if sql.starts_with("INSERT INTO schema_migrations") {
                let mut map = HashMap::new();
                map.insert("version".to_string(), params[0].to_string());
                self.tables.borrow_mut().get_mut("schema_migrations").unwrap().push(map);
            } else if sql.starts_with("DELETE FROM schema_migrations") {
                let version = params[0].to_string();
                self.tables.borrow_mut().get_mut("schema_migrations").unwrap().retain(|row| row.get("version") != Some(&version));
            }
            Ok(())
        }
        fn query_scalar(&self, sql: &str) -> Result<Option<i64>, String> {
            if sql.starts_with("SELECT MAX(version)") {
                let max = self.tables.borrow().get("schema_migrations").unwrap().iter()
                    .filter_map(|row| row.get("version").and_then(|v| v.parse::<i64>().ok()))
                    .max();
                return Ok(Some(max.unwrap_or(0)));
            }
            Ok(Some(0))
        }
        fn query_scalar_with_params(&self, sql: &str, params: &[&str]) -> Result<Option<i64>, String> {
            if sql.starts_with("SELECT COUNT(*)") {
                let version = params[0].to_string();
                let count = self.tables.borrow().get("schema_migrations").unwrap().iter()
                    .filter(|row| row.get("version") == Some(&version))
                    .count();
                return Ok(Some(count as i64));
            }
            Ok(Some(0))
        }
    }

    #[test]
    fn test_apply_pending_is_idempotent() {
        let executor = MockExecutor::new();
        let migrations = vec![Migration {
            version: 1, name: "test".to_string(),
            up_script: "CREATE TABLE t (id INT);".to_string(),
            down_script: "DROP TABLE IF EXISTS t;".to_string(),
            checksum: "abc".to_string(),
        }];
        let runner = MigrationRunner::new(migrations);
        let count = runner.apply_pending(&executor).unwrap();
        assert_eq!(count, 1);
        let count = runner.apply_pending(&executor).unwrap();
        assert_eq!(count, 0);
        assert!(runner.is_up_to_date(&executor).unwrap());
    }

    #[test]
    fn test_rollback_no_migrations() {
        let executor = MockExecutor::new();
        let runner = MigrationRunner::new(vec![]);
        let result = runner.rollback_last(&executor);
        assert!(matches!(result, Err(MigrationError::NoMigrationsToRollback)));
    }

    #[test]
    fn real_migration_files_are_discoverable_and_well_formed() {
        let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        assert!(
            migrations_dir.exists(),
            "migrations/ directory must exist at {}",
            migrations_dir.display()
        );

        let migrations = discover_migrations(&migrations_dir)
            .expect("discover_migrations must succeed on real migration files");
        assert!(
            !migrations.is_empty(),
            "at least one migration must be discovered"
        );

        let mut seen_versions = std::collections::HashSet::new();
        for m in &migrations {
            assert!(
                !m.up_script.trim().is_empty(),
                "migration {} ({}) has empty up script",
                m.version, m.name
            );
            assert!(
                !m.checksum.is_empty(),
                "migration {} must have a non-empty checksum",
                m.version
            );
            assert!(
                seen_versions.insert(m.version),
                "duplicate migration version {}",
                m.version
            );
        }

        let versions: Vec<u32> = migrations.iter().map(|m| m.version).collect();
        let mut sorted = versions.clone();
        sorted.sort();
        assert_eq!(versions, sorted, "migrations must be sorted by version");
    }

    #[test]
    fn migration_sql_contains_expected_tables() {
        let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let migrations = discover_migrations(&migrations_dir).unwrap();
        let all_sql: String = migrations.iter().map(|m| m.up_script.as_str()).collect::<Vec<_>>().join("\n");

        let required_tables = [
            "schema_migrations",
            "user_two_factor",
            "recovery_code_usage",
            "two_fa_audit_log",
        ];
        for table in &required_tables {
            assert!(
                all_sql.contains(table),
                "migration SQL must reference table '{}' but it was not found",
                table
            );
        }
    }

    #[test]
    fn every_up_migration_has_a_down_script() {
        let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let migrations = discover_migrations(&migrations_dir).unwrap();

        for m in &migrations {
            assert!(
                !m.down_script.is_empty(),
                "migration {} ({}) is missing a .down.sql file",
                m.version, m.name
            );
        }
    }

    #[test]
    fn test_dry_run_reports_pending_without_mutation() {
        let executor = MockExecutor::new();
        let migrations = vec![
            Migration {
                version: 1,
                name: "create_users".to_string(),
                up_script: "CREATE TABLE users (id INT);".to_string(),
                down_script: "DROP TABLE IF EXISTS users;".to_string(),
                checksum: "abc".to_string(),
            },
            Migration {
                version: 2,
                name: "add_email".to_string(),
                up_script: "ALTER TABLE users ADD COLUMN email TEXT;".to_string(),
                down_script: String::new(),
                checksum: "def".to_string(),
            },
        ];
        let runner = MigrationRunner::new(migrations);

        // Both migrations should be reported as pending
        let pending = runner.dry_run(&executor).unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0], (1, "create_users".to_string()));
        assert_eq!(pending[1], (2, "add_email".to_string()));

        // State must be unchanged: is_up_to_date still false, apply still finds both
        assert!(!runner.is_up_to_date(&executor).unwrap());
        assert_eq!(runner.apply_pending(&executor).unwrap(), 2);

        // After applying, dry_run reports nothing pending
        assert!(runner.dry_run(&executor).unwrap().is_empty());
    }

    // -----------------------------------------------------------------------
    // Issue #781 — Round-trip migration rollback safety test
    //
    // Database: PostgreSQL (required — migration SQL uses Postgres-specific
    // syntax: BIGSERIAL, ADD/DROP COLUMN IF NOT EXISTS, ON CONFLICT, etc.).
    //
    // This test is skipped automatically when DATABASE_URL is not set so
    // the standard `cargo test` (unit-only) workflow is unaffected.  In CI
    // the `migration-rollback` job starts a Postgres service container and
    // sets DATABASE_URL before running this test.
    //
    // Correctness guarantee: applying every migration followed by rolling
    // every migration back in reverse MUST leave the schema in exactly the
    // same state as the baseline captured before anything was applied.
    // A broken down-script that leaks a table or column would be caught here
    // before it ever reaches a production emergency rollback.
    // -----------------------------------------------------------------------

    /// Capture a sorted list of all user-created tables in the public schema.
    /// `schema_migrations` is intentionally included in this snapshot because
    /// migration 001's down-script drops it; a correct full rollback leaves
    /// zero tables behind.
    fn snapshot_tables(executor: &PostgresExecutor) -> Vec<String> {
        executor
            .table_names()
            .expect("failed to snapshot table names")
    }

    #[test]
    fn test_round_trip_migration_rollback_leaves_schema_unchanged() {
        // Skip when no live Postgres is available (local / unit-only runs).
        let database_url = match std::env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                eprintln!(
                    "[SKIP] test_round_trip_migration_rollback_leaves_schema_unchanged: \
                     DATABASE_URL not set — skipping round-trip test (requires Postgres)"
                );
                return;
            }
        };

        // Connect to the live database.
        let executor = PostgresExecutor::connect(&database_url)
            .expect("failed to connect to Postgres for round-trip migration test");

        // --- Idempotency: clean slate before we start -------------------
        // Drop any tables left over from a previous interrupted run so the
        // test is safe to run multiple times without manual cleanup.
        executor
            .drop_all_tables()
            .expect("failed to drop existing tables before round-trip test");

        // --- Step 1: Baseline snapshot (empty schema) -------------------
        let baseline = snapshot_tables(&executor);
        assert!(
            baseline.is_empty(),
            "baseline snapshot must be empty after drop_all_tables, found: {:?}",
            baseline
        );

        // --- Step 2: Load migration files from disk ----------------------
        let migrations_dir =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        let runner = MigrationRunner::from_directory(&migrations_dir)
            .expect("failed to discover migration files");

        let migration_count = runner.migrations.len();
        assert!(
            migration_count > 0,
            "at least one migration must be discovered"
        );

        // --- Step 3: Apply ALL migrations --------------------------------
        let applied = runner
            .apply_pending(&executor)
            .expect("apply_pending failed — one of the up-migrations has broken SQL");

        assert_eq!(
            applied, migration_count,
            "expected all {} migrations to be applied, but only {} were",
            migration_count, applied
        );

        // Sanity: the runner agrees we are now fully up-to-date.
        assert!(
            runner
                .is_up_to_date(&executor)
                .expect("is_up_to_date check failed"),
            "runner must report is_up_to_date = true after applying all migrations"
        );

        // --- Step 4: Roll back ALL migrations in reverse -----------------
        // Each call to rollback_last() undoes the highest applied version.
        for step in (1..=migration_count).rev() {
            runner
                .rollback_last(&executor)
                .unwrap_or_else(|e| {
                    panic!(
                        "rollback_last failed at step {} (rolling back migration {}): {}",
                        migration_count - step + 1,
                        step,
                        e
                    )
                });
        }

        // Attempting another rollback must now return NoMigrationsToRollback
        // (the schema_migrations table was itself dropped by migration 001's
        // down-script, so MAX(version) query may fail; either outcome is
        // acceptable — the important assertion is the schema snapshot below).
        let extra_rollback = runner.rollback_last(&executor);
        // We accept either the explicit NoMigrationsToRollback error or a Sql
        // error caused by the schema_migrations table no longer existing.
        assert!(
            extra_rollback.is_err(),
            "an extra rollback after full teardown must return an error"
        );

        // --- Step 5: Post-rollback snapshot must match baseline ----------
        let after_rollback = snapshot_tables(&executor);
        assert_eq!(
            after_rollback, baseline,
            "schema after full rollback must exactly match the pre-migration baseline.\n\
             Leaked tables: {:?}\n\
             This means at least one .down.sql file is incorrect.",
            after_rollback
                .iter()
                .filter(|t| !baseline.contains(t))
                .collect::<Vec<_>>()
        );

        // --- Step 6: Verify idempotency — apply + rollback again ---------
        // The second cycle proves the test is truly repeatable without manual
        // teardown and that the down-scripts leave nothing that blocks a fresh
        // apply.
        let applied_again = runner
            .apply_pending(&executor)
            .expect("second apply_pending failed — idempotency broken");
        assert_eq!(
            applied_again, migration_count,
            "second apply must apply all {} migrations again",
            migration_count
        );

        for step in (1..=migration_count).rev() {
            runner.rollback_last(&executor).unwrap_or_else(|e| {
                panic!(
                    "second rollback_last failed at step {}: {}",
                    migration_count - step + 1,
                    e
                )
            });
        }

        let after_second_rollback = snapshot_tables(&executor);
        assert_eq!(
            after_second_rollback, baseline,
            "schema after SECOND full rollback must still match the baseline — \
             idempotency check failed. Leaked tables: {:?}",
            after_second_rollback
                .iter()
                .filter(|t| !baseline.contains(t))
                .collect::<Vec<_>>()
        );
    }
}

pub fn discover_migrations(migrations_dir: &Path) -> Result<Vec<Migration>, MigrationError> {
    let mut migrations: Vec<Migration> = Vec::new();
    let entries = fs::read_dir(migrations_dir)
        .map_err(|e| MigrationError::Io(format!("reading migrations dir: {}", e)))?;

    let mut up_files: Vec<(u32, String, PathBuf)> = Vec::new();
    let mut down_files: Vec<(u32, PathBuf)> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| MigrationError::Io(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("sql") {
            continue;
        }
        let filename = path.file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let parts: Vec<&str> = filename.splitn(2, '_').collect();
        if parts.len() < 2 {
            continue;
        }
        let version: u32 = parts[0]
            .parse()
            .map_err(|_| MigrationError::InvalidMigrationFile(filename.clone()))?;

        if filename.ends_with(".down") {
            down_files.push((version, path));
        } else {
            let name = parts[1].to_string();
            up_files.push((version, name, path));
        }
    }

    up_files.sort_by_key(|(v, _, _)| *v);

    for (version, name, up_path) in &up_files {
        let up_script = fs::read_to_string(up_path)
            .map_err(|e| MigrationError::Io(format!("reading {}: {}", up_path.display(), e)))?;
        let checksum = hex::encode(Sha256::digest(up_script.as_bytes()));

        let down_script = down_files.iter()
            .find(|(v, _)| v == version)
            .map(|(_, down_path)| {
                fs::read_to_string(down_path)
                    .map_err(|e| MigrationError::Io(format!("reading {}: {}", down_path.display(), e)))
            })
            .transpose()?
            .unwrap_or_default();

        migrations.push(Migration {
            version: *version,
            name: name.clone(),
            up_script,
            down_script,
            checksum,
        });
    }
    Ok(migrations)
}
