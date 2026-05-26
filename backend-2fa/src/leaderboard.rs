use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Timeframe {
    AllTime,
    Monthly,
    Weekly,
}

impl Timeframe {
    /// Returns the unix timestamp cutoff for the timeframe (seconds since epoch).
    /// `now` is the current unix timestamp in seconds.
    pub fn cutoff(&self, now: u64) -> Option<u64> {
        match self {
            Timeframe::AllTime => None,
            Timeframe::Monthly => Some(now.saturating_sub(30 * 24 * 3600)),
            Timeframe::Weekly => Some(now.saturating_sub(7 * 24 * 3600)),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OrganizerScore {
    pub rank: usize,
    pub organizer: String,
    pub score: u64,
}

/// A single ticket-sale event attributed to an organizer at a point in time.
#[derive(Debug, Clone)]
pub struct TicketSale {
    pub organizer: String,
    pub tickets_sold: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreSubmissionError {
    SuspiciousScore {
        user_id: String,
        attempted_score: u64,
        reason: String,
    },
}

/// A score submission that can be validated for anomalies.
#[derive(Debug, Clone)]
pub struct ScoreSubmission {
    pub user_id: String,
    pub score: u64,
    pub timestamp: u64,
}

/// Configuration for score validation
#[derive(Debug, Clone)]
pub struct ScoreValidationConfig {
    /// Maximum allowed score delta per submission
    pub max_score_delta: u64,
}

impl Default for ScoreValidationConfig {
    fn default() -> Self {
        Self {
            max_score_delta: 1000,
        }
    }
}

/// A flagged score submission that was rejected
#[derive(Debug, Clone, Serialize)]
pub struct FlaggedScoreSubmission {
    pub user_id: String,
    pub attempted_score: u64,
    pub timestamp: u64,
    pub reason: String,
}

/// In-memory storage for flagged submissions (for testing/demo purposes)
pub struct FlaggedScoreStore {
    flagged: std::sync::Mutex<Vec<FlaggedScoreSubmission>>,
}

impl FlaggedScoreStore {
    pub fn new() -> Self {
        Self {
            flagged: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn add_flagged(&self, flagged: FlaggedScoreSubmission) {
        if let Ok(mut store) = self.flagged.lock() {
            store.push(flagged);
        }
    }

    pub fn get_flagged_by_user(&self, user_id: &str) -> Vec<FlaggedScoreSubmission> {
        if let Ok(store) = self.flagged.lock() {
            store.iter()
                .filter(|f| f.user_id == user_id)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_all_flagged(&self) -> Vec<FlaggedScoreSubmission> {
        if let Ok(store) = self.flagged.lock() {
            store.clone()
        } else {
            Vec::new()
        }
    }

    pub fn clear(&self) {
        if let Ok(mut store) = self.flagged.lock() {
            store.clear();
        }
    }
}

impl Default for FlaggedScoreStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate a score submission against the maximum allowed delta.
/// Returns an error if the submission is suspicious.
pub fn validate_score_submission(
    submission: &ScoreSubmission,
    last_known_score: Option<u64>,
    config: &ScoreValidationConfig,
) -> Result<(), ScoreSubmissionError> {
    if let Some(last_score) = last_known_score {
        let delta = if submission.score > last_score {
            submission.score - last_score
        } else {
            last_score - submission.score
        };

        if delta > config.max_score_delta {
            return Err(ScoreSubmissionError::SuspiciousScore {
                user_id: submission.user_id.clone(),
                attempted_score: submission.score,
                reason: format!(
                    "Score delta {} exceeds maximum allowed {}",
                    delta, config.max_score_delta
                ),
            });
        }
    }

    Ok(())
}

/// Rank organizers by total tickets sold within the given timeframe.
/// Returns a JSON-serialisable vec sorted by score descending.
pub fn rank_organizers(
    sales: &[TicketSale],
    timeframe: &Timeframe,
    now: u64,
) -> Vec<OrganizerScore> {
    let cutoff = timeframe.cutoff(now);

    let mut totals: HashMap<String, u64> = HashMap::new();
    for sale in sales {
        if let Some(c) = cutoff {
            if sale.timestamp < c {
                continue;
            }
        }
        *totals.entry(sale.organizer.clone()).or_insert(0) += sale.tickets_sold;
    }

    let mut ranked: Vec<(String, u64)> = totals.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));

    ranked
        .into_iter()
        .enumerate()
        .map(|(i, (organizer, score))| OrganizerScore {
            rank: i + 1,
            organizer,
            score,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> u64 {
        3_000_000
    }

    fn sales() -> Vec<TicketSale> {
        vec![
            TicketSale {
                organizer: "Alice".into(),
                tickets_sold: 50,
                timestamp: now() - 1,
            },
            TicketSale {
                organizer: "Bob".into(),
                tickets_sold: 30,
                timestamp: now() - 1,
            },
            TicketSale {
                organizer: "Alice".into(),
                tickets_sold: 20,
                timestamp: now() - 8 * 24 * 3600,
            }, // >7 days ago
            TicketSale {
                organizer: "Carol".into(),
                tickets_sold: 10,
                timestamp: now() - 31 * 24 * 3600,
            }, // >30 days ago
        ]
    }

    #[test]
    fn all_time_includes_all() {
        let result = rank_organizers(&sales(), &Timeframe::AllTime, now());
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].organizer, "Alice");
        assert_eq!(result[0].score, 70);
        assert_eq!(result[0].rank, 1);
    }

    #[test]
    fn monthly_excludes_old() {
        let result = rank_organizers(&sales(), &Timeframe::Monthly, now());
        // Carol's sale is >30 days old, excluded
        assert!(!result.iter().any(|r| r.organizer == "Carol"));
        assert_eq!(result[0].organizer, "Alice");
        assert_eq!(result[0].score, 70);
    }

    #[test]
    fn weekly_excludes_older_than_7_days() {
        let result = rank_organizers(&sales(), &Timeframe::Weekly, now());
        // Alice's old sale and Carol's sale excluded; Alice only has 50 recent
        let alice = result.iter().find(|r| r.organizer == "Alice").unwrap();
        assert_eq!(alice.score, 50);
        assert!(!result.iter().any(|r| r.organizer == "Carol"));
    }

    #[test]
    fn empty_sales_returns_empty() {
        let result = rank_organizers(&[], &Timeframe::AllTime, now());
        assert!(result.is_empty());
    }

    // -----------------------------------------------------------------------
    // Score Validation Tests
    // -----------------------------------------------------------------------

    #[test]
    fn validate_first_submission_always_passes() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 5000,
            timestamp: now(),
        };
        let config = ScoreValidationConfig::default();

        let result = validate_score_submission(&submission, None, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_small_increase_passes() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 1100,
            timestamp: now(),
        };
        let config = ScoreValidationConfig::default();

        let result = validate_score_submission(&submission, Some(1000), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_small_decrease_passes() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 950,
            timestamp: now(),
        };
        let config = ScoreValidationConfig::default();

        let result = validate_score_submission(&submission, Some(1000), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_exact_delta_limit_passes() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 2000,
            timestamp: now(),
        };
        let config = ScoreValidationConfig {
            max_score_delta: 1000,
        };

        let result = validate_score_submission(&submission, Some(1000), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_exceeding_delta_fails() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 2001,
            timestamp: now(),
        };
        let config = ScoreValidationConfig {
            max_score_delta: 1000,
        };

        let result = validate_score_submission(&submission, Some(1000), &config);
        assert!(result.is_err());

        if let Err(ScoreSubmissionError::SuspiciousScore {
            user_id,
            attempted_score,
            reason,
        }) = result
        {
            assert_eq!(user_id, "user1");
            assert_eq!(attempted_score, 2001);
            assert!(reason.contains("exceeds maximum"));
        } else {
            panic!("Expected SuspiciousScore error");
        }
    }

    #[test]
    fn validate_large_decrease_fails() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 100,
            timestamp: now(),
        };
        let config = ScoreValidationConfig {
            max_score_delta: 1000,
        };

        let result = validate_score_submission(&submission, Some(2000), &config);
        assert!(result.is_err());
    }

    #[test]
    fn validate_zero_score() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 0,
            timestamp: now(),
        };
        let config = ScoreValidationConfig::default();

        let result = validate_score_submission(&submission, Some(500), &config);
        assert!(result.is_err());
    }

    #[test]
    fn custom_config_larger_delta() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 5500,
            timestamp: now(),
        };
        let config = ScoreValidationConfig {
            max_score_delta: 10000,
        };

        let result = validate_score_submission(&submission, Some(500), &config);
        assert!(result.is_ok());
    }

    #[test]
    fn custom_config_smaller_delta() {
        let submission = ScoreSubmission {
            user_id: "user1".into(),
            score: 150,
            timestamp: now(),
        };
        let config = ScoreValidationConfig {
            max_score_delta: 100,
        };

        let result = validate_score_submission(&submission, Some(100), &config);
        assert!(result.is_err());
    }

    #[test]
    fn flagged_score_store_add_and_retrieve() {
        let store = FlaggedScoreStore::new();
        let flagged = FlaggedScoreSubmission {
            user_id: "user1".into(),
            attempted_score: 5000,
            timestamp: now(),
            reason: "Exceeds delta".into(),
        };

        store.add_flagged(flagged.clone());
        let retrieved = store.get_flagged_by_user("user1");

        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].user_id, "user1");
        assert_eq!(retrieved[0].attempted_score, 5000);
    }

    #[test]
    fn flagged_score_store_multiple_users() {
        let store = FlaggedScoreStore::new();

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user1".into(),
            attempted_score: 5000,
            timestamp: now(),
            reason: "Exceeds delta".into(),
        });

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user2".into(),
            attempted_score: 3000,
            timestamp: now(),
            reason: "Suspicious".into(),
        });

        let user1_flagged = store.get_flagged_by_user("user1");
        let user2_flagged = store.get_flagged_by_user("user2");

        assert_eq!(user1_flagged.len(), 1);
        assert_eq!(user2_flagged.len(), 1);
        assert_eq!(user1_flagged[0].user_id, "user1");
        assert_eq!(user2_flagged[0].user_id, "user2");
    }

    #[test]
    fn flagged_score_store_get_all() {
        let store = FlaggedScoreStore::new();

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user1".into(),
            attempted_score: 5000,
            timestamp: now(),
            reason: "Exceeds delta".into(),
        });

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user2".into(),
            attempted_score: 3000,
            timestamp: now(),
            reason: "Suspicious".into(),
        });

        let all = store.get_all_flagged();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn flagged_score_store_clear() {
        let store = FlaggedScoreStore::new();

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user1".into(),
            attempted_score: 5000,
            timestamp: now(),
            reason: "Exceeds delta".into(),
        });

        assert_eq!(store.get_all_flagged().len(), 1);

        store.clear();
        assert_eq!(store.get_all_flagged().len(), 0);
    }

    #[test]
    fn flagged_score_store_user_not_found() {
        let store = FlaggedScoreStore::new();

        store.add_flagged(FlaggedScoreSubmission {
            user_id: "user1".into(),
            attempted_score: 5000,
            timestamp: now(),
            reason: "Exceeds delta".into(),
        });

        let user2_flagged = store.get_flagged_by_user("user2");
        assert!(user2_flagged.is_empty());
    }

    #[test]
    fn score_validation_config_default() {
        let config = ScoreValidationConfig::default();
        assert_eq!(config.max_score_delta, 1000);
    }

    #[test]
    fn score_submission_error_displays_user_id() {
        let error = ScoreSubmissionError::SuspiciousScore {
            user_id: "user123".into(),
            attempted_score: 9999,
            reason: "Test reason".into(),
        };

        match error {
            ScoreSubmissionError::SuspiciousScore {
                user_id,
                attempted_score,
                reason,
            } => {
                assert_eq!(user_id, "user123");
                assert_eq!(attempted_score, 9999);
                assert_eq!(reason, "Test reason");
            }
        }
    }
}
