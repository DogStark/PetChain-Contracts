-- 001_create_schema_migrations.down.sql
-- Rollback: remove the schema_migrations tracking table.

DROP TABLE IF EXISTS schema_migrations;
