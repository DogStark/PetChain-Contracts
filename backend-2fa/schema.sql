-- Database schema for 2FA implementation

-- Table to store user 2FA settings
CREATE TABLE user_two_factor (
    user_id VARCHAR(255) PRIMARY KEY,
    secret TEXT NOT NULL,
    backup_codes TEXT NOT NULL, -- JSON array of backup codes
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

-- Index for faster lookups
CREATE INDEX idx_user_two_factor_enabled ON user_two_factor(enabled);

-- Example queries:

-- Insert new 2FA setup
-- INSERT INTO user_two_factor (user_id, secret, backup_codes, enabled) 
-- VALUES (?, ?, ?, false);

-- Get 2FA data for user
-- SELECT secret, backup_codes, enabled FROM user_two_factor WHERE user_id = ?;

-- Enable 2FA after verification
-- UPDATE user_two_factor SET enabled = true WHERE user_id = ?;

-- Update backup codes after one is used
-- UPDATE user_two_factor SET backup_codes = ? WHERE user_id = ?;

-- Disable/Delete 2FA
-- DELETE FROM user_two_factor WHERE user_id = ?;

-- Table to audit recovery code usage (single-use enforcement)
CREATE TABLE recovery_code_usage (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    code_index INT NOT NULL,
    used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, code_index)
);

-- Audit log for 2FA admin actions and security events (Issue #688, #713)
CREATE TABLE two_fa_audit_log (
    id SERIAL PRIMARY KEY,
    user_id VARCHAR(255) NOT NULL,
    event VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    metadata TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_user_id ON two_fa_audit_log(user_id);
CREATE INDEX idx_audit_log_timestamp ON two_fa_audit_log(timestamp DESC);

-- Canary token accounts (Issue #713)
-- Accounts in this table are excluded from normal user listings and trigger
-- alerts on any verification attempt.
CREATE TABLE canary_accounts (
    user_id VARCHAR(255) PRIMARY KEY,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
