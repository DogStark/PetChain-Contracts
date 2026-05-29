-- 001_create_schema_migrations.sql
-- Creates the schema_migrations tracking table that the migration runner
-- uses to track which migrations have been applied.

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    applied_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    checksum    TEXT NOT NULL
);
