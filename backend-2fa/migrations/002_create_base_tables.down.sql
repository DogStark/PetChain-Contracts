-- 002_create_base_tables.down.sql
-- Rollback: drop base 2FA tables in reverse dependency order.

DROP TABLE IF EXISTS canary_accounts;
DROP TABLE IF EXISTS two_fa_audit_log;
DROP TABLE IF EXISTS recovery_code_usage;
DROP TABLE IF EXISTS user_two_factor;
