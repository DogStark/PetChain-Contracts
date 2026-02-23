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
