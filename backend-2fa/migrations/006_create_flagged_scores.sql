-- 006_create_flagged_scores.sql
-- Creates the flagged_scores table backing PostgresFlaggedScoreStore
-- (Issue #789), persisting leaderboard score submissions rejected by
-- validate_score_submission (delta or z-score anomaly) across restarts.
-- The in-memory FlaggedScoreStore implementation remains available for
-- tests and for deployments without a configured database.

CREATE TABLE IF NOT EXISTS flagged_scores (
    id          BIGSERIAL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    score       BIGINT NOT NULL,
    reason      TEXT NOT NULL,
    flagged_at  TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_flagged_scores_user_id ON flagged_scores(user_id);
CREATE INDEX IF NOT EXISTS idx_flagged_scores_flagged_at ON flagged_scores(flagged_at);
