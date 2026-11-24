-- 007_create_adoption_case_audit.sql
-- Creates the adoption_case_audit table backing PostgresAdoptionCaseAuditStore
-- (Issue #1613), persisting adoption case transitions recorded for the
-- GET /admin/adoption/cases/{case_id}/audit export (Issue #1460) across restarts.
-- The in-memory AdoptionCaseAuditStore remains for tests and demos only.

CREATE TABLE IF NOT EXISTS adoption_case_audit (
    event_id     BIGSERIAL PRIMARY KEY,
    case_id      VARCHAR(64) NOT NULL,
    transition   VARCHAR(64) NOT NULL,
    actor        VARCHAR(64) NOT NULL,
    recorded_at  BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_adoption_case_audit_case_id ON adoption_case_audit(case_id, event_id);
