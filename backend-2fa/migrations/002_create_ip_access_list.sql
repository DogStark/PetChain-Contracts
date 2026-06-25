-- 002_create_ip_access_list.sql
-- Creates the ip_access_list table backing the admin IP allowlist/blocklist
-- endpoints (Issue #701). Entries are matched by CIDR containment in
-- application code; list_type distinguishes allow vs. block rules.

CREATE TABLE IF NOT EXISTS ip_access_list (
    id          BIGSERIAL PRIMARY KEY,
    cidr        TEXT NOT NULL,
    list_type   TEXT NOT NULL CHECK (list_type IN ('allow', 'block')),
    note        TEXT,
    created_by  TEXT NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_ip_access_list_type ON ip_access_list(list_type);
