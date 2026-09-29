-- 001_create_schema_migrations.sql
-- Creates the schema_migrations tracking table that the migration runner
-- uses to track which migrations have been applied.
--
-- The `checksum` column stores a hash of the migration file contents at the
-- time it was applied. On startup the runner recomputes the checksum for every
-- already-applied migration and fails closed if it no longer matches, so an
-- edited migration file cannot silently produce schema drift.
--
-- Remediation when drift is detected:
--   * If the edit was intentional, re-baseline the migration by updating the
--     stored checksum to the new file hash (after verifying the schema change
--     was actually applied), or
--   * Roll the change back into a new forward migration instead of editing an
--     applied file, then restore the original file contents.

CREATE TABLE IF NOT EXISTS schema_migrations (
    version     INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    applied_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    checksum    TEXT NOT NULL
);

-- Guard against concurrent startup applying the same migration twice: the
-- runner takes a session-level advisory lock keyed on this constant before it
-- inspects or applies any migration, and releases it when startup completes.
-- The lock is advisory so it is a no-op on engines that do not support it.
