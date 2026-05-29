-- 002_create_base_tables.sql
-- Creates the base 2FA tables from the schema.

CREATE TABLE IF NOT EXISTS user_two_factor (
    user_id VARCHAR(255) PRIMARY KEY,
    secret TEXT NOT NULL,
    backup_codes TEXT NOT NULL,
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_user_two_factor_enabled ON user_two_factor(enabled);

CREATE TABLE IF NOT EXISTS recovery_code_usage (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    code_index INT NOT NULL,
    used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, code_index)
);

CREATE TABLE IF NOT EXISTS two_fa_audit_log (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    event VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    metadata TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_log_user_id ON two_fa_audit_log(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON two_fa_audit_log(timestamp DESC);

CREATE TABLE IF NOT EXISTS canary_accounts (
    user_id VARCHAR(255) PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
