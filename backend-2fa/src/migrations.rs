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
        ).map_err(|e| MigrationError::Sql(e))
    }

    fn get_applied_migrations(&self, executor: &dyn SqlExecutor) -> Result<Vec<(u32, String)>, MigrationError> {
        self.ensure_tracking_table(executor)?;
        let mut applied = Vec::new();
        for migration in &self.migrations {
            let version_str = migration.version.to_string();
            let count = executor.query_scalar_with_params(
                "SELECT COUNT(*) FROM schema_migrations WHERE version = $1",
                &[version_str.as_str()]
            ).map_err(|e| MigrationError::Sql(e))?.unwrap_or(0);
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
            .map_err(|e| MigrationError::Sql(e))?.unwrap_or(0) as u32;
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
        ).map_err(|e| MigrationError::Sql(e))?;
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
