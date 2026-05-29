-- Database schema for 2FA implementation

-- Tenant registry (super-admin provisioned)
CREATE TABLE tenants (
    tenant_id VARCHAR(255) PRIMARY KEY,
    totp_issuer VARCHAR(255) NOT NULL DEFAULT 'PetChain',
    rate_limit_max_failures INT NOT NULL DEFAULT 5,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Table to store user 2FA settings
CREATE TABLE user_two_factor (
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(tenant_id),
    user_id VARCHAR(255) NOT NULL,
    secret TEXT NOT NULL,
    backup_codes TEXT NOT NULL, -- JSON array of backup codes
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (tenant_id, user_id)
);

-- Index for faster lookups
CREATE INDEX idx_user_two_factor_enabled ON user_two_factor(tenant_id, enabled);

-- Table to audit recovery code usage (single-use enforcement)
CREATE TABLE recovery_code_usage (
    id SERIAL PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    code_index INT NOT NULL,
    used_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_address VARCHAR(45),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tenant_id, user_id, code_index)
);

-- Audit log for 2FA admin actions and security events (Issue #688, #713)
CREATE TABLE two_fa_audit_log (
    id SERIAL PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    event VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    metadata TEXT,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_audit_log_tenant_user ON two_fa_audit_log(tenant_id, user_id);
CREATE INDEX idx_audit_log_timestamp ON two_fa_audit_log(timestamp DESC);

-- Canary token accounts (Issue #713)
CREATE TABLE canary_accounts (
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (tenant_id, user_id)
);

-- Persistent 2FA lockout state. Redis stores the hot failure counter, but this
-- table is the source of truth for full account lockout after 10 failures.
CREATE TABLE two_fa_lockouts (
    user_id VARCHAR(255) PRIMARY KEY,
    failed_attempts INT NOT NULL DEFAULT 0,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    locked_at TIMESTAMP NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
