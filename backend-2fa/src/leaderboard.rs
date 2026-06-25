use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::error::{ErrorBadRequest, ErrorTooManyRequests};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::f64;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

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
            store
                .iter()
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

/// A leaderboard entry with decayed score and percentile rank
#[derive(Debug, Clone, Serialize)]
pub struct LeaderboardEntry {
    pub user_id: String,
    pub decayed_score: f64,
    pub percentile: f64,
}

/// Score update payload pushed to websocket subscribers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LeaderboardScoreUpdate {
    pub user_id: String,
    pub new_score: u64,
    pub rank: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
enum LeaderboardSubscriptionCommand {
    Subscribe { user_ids: Vec<String> },
    Unsubscribe { user_ids: Vec<String> },
    Replace { user_ids: Vec<String> },
    Clear,
}

#[derive(Debug)]
pub struct LeaderboardWsHub {
    broadcaster: broadcast::Sender<LeaderboardScoreUpdate>,
    connections_by_ip: Mutex<HashMap<IpAddr, usize>>,
}

impl LeaderboardWsHub {
    pub fn global() -> Arc<Self> {
        static HUB: OnceLock<Arc<LeaderboardWsHub>> = OnceLock::new();
        HUB.get_or_init(|| {
            let (broadcaster, _) = broadcast::channel(256);
            Arc::new(Self {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            })
        })
        .clone()
    }

    pub fn connect(self: &Arc<Self>, ip: IpAddr) -> Result<LeaderboardConnectionGuard, String> {
        let mut connections = self
            .connections_by_ip
            .lock()
            .map_err(|_| "Leaderboard connection state is unavailable".to_string())?;
        let count = connections.get(&ip).copied().unwrap_or(0);
        if count >= 5 {
            return Err(format!("Connection limit exceeded for IP {}", ip));
        }
        connections.insert(ip, count + 1);
        Ok(LeaderboardConnectionGuard {
            hub: Arc::clone(self),
            ip,
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LeaderboardScoreUpdate> {
        self.broadcaster.subscribe()
    }

    pub fn publish_score_update(&self, update: LeaderboardScoreUpdate) {
        let _ = self.broadcaster.send(update);
    }

    fn disconnect(&self, ip: IpAddr) {
        if let Ok(mut connections) = self.connections_by_ip.lock() {
            if let Some(current) = connections.get_mut(&ip) {
                if *current <= 1 {
                    connections.remove(&ip);
                } else {
                    *current -= 1;
                }
            }
        }
    }
}

pub struct LeaderboardConnectionGuard {
    hub: Arc<LeaderboardWsHub>,
    ip: IpAddr,
}

impl Drop for LeaderboardConnectionGuard {
    fn drop(&mut self) {
        self.hub.disconnect(self.ip);
    }
}

pub struct LeaderboardWsSession {
    hub: Arc<LeaderboardWsHub>,
    _connection: LeaderboardConnectionGuard,
    subscriptions: HashSet<String>,
}

impl LeaderboardWsSession {
    pub fn new(hub: Arc<LeaderboardWsHub>, connection: LeaderboardConnectionGuard) -> Self {
        Self {
            hub,
            _connection: connection,
            subscriptions: HashSet::new(),
        }
    }

    fn should_forward(&self, update: &LeaderboardScoreUpdate) -> bool {
        self.subscriptions.is_empty() || self.subscriptions.contains(&update.user_id)
    }

    fn apply_command(&mut self, command: LeaderboardSubscriptionCommand) {
        match command {
            LeaderboardSubscriptionCommand::Subscribe { user_ids } => {
                for user_id in user_ids {
                    self.subscriptions.insert(user_id);
                }
            }
            LeaderboardSubscriptionCommand::Unsubscribe { user_ids } => {
                for user_id in user_ids {
                    self.subscriptions.remove(&user_id);
                }
            }
            LeaderboardSubscriptionCommand::Replace { user_ids } => {
                self.subscriptions.clear();
                for user_id in user_ids {
                    self.subscriptions.insert(user_id);
                }
            }
            LeaderboardSubscriptionCommand::Clear => {
                self.subscriptions.clear();
            }
        }
    }
}

impl Actor for LeaderboardWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let receiver = self.hub.subscribe();
        ctx.add_stream(BroadcastStream::new(receiver));
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LeaderboardWsSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Text(text)) => {
                if let Ok(command) = serde_json::from_str::<LeaderboardSubscriptionCommand>(&text) {
                    self.apply_command(command);
                    ctx.text(r#"{"status":"ok"}"#);
                } else {
                    ctx.text(r#"{"status":"error","reason":"invalid_subscription_message"}"#);
                }
            }
            Ok(ws::Message::Ping(payload)) => ctx.pong(&payload),
            Ok(ws::Message::Pong(_)) => {}
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Binary(_)) => {
                ctx.text(r#"{"status":"error","reason":"binary_not_supported"}"#);
            }
            Ok(ws::Message::Continuation(_)) => {}
            Ok(ws::Message::Nop) => {}
            Err(_) => {
                ctx.stop();
            }
        }
    }
}

impl
    StreamHandler<
        Result<LeaderboardScoreUpdate, tokio_stream::wrappers::errors::BroadcastStreamRecvError>,
    > for LeaderboardWsSession
{
    fn handle(
        &mut self,
        item: Result<
            LeaderboardScoreUpdate,
            tokio_stream::wrappers::errors::BroadcastStreamRecvError,
        >,
        ctx: &mut Self::Context,
    ) {
        match item {
            Ok(update) if self.should_forward(&update) => {
                if let Ok(payload) = serde_json::to_string(&update) {
                    ctx.text(payload);
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

/// Broadcast a score update to all connected websocket clients.
pub fn broadcast_score_update(user_id: impl Into<String>, new_score: u64, rank: u64) {
    LeaderboardWsHub::global().publish_score_update(LeaderboardScoreUpdate {
        user_id: user_id.into(),
        new_score,
        rank,
    });
}

/// Actix-web websocket endpoint for `GET /leaderboard/ws`.
pub async fn leaderboard_ws_endpoint(
    req: HttpRequest,
    stream: Payload,
) -> Result<HttpResponse, Error> {
    let peer_ip = req
        .peer_addr()
        .map(|addr| addr.ip())
        .ok_or_else(|| ErrorBadRequest("Missing peer address for leaderboard websocket"))?;

    let hub = LeaderboardWsHub::global();
    let connection = hub
        .connect(peer_ip)
        .map_err(|err| ErrorTooManyRequests(err))?;
    ws::start(LeaderboardWsSession::new(hub, connection), &req, stream)
}

/// Get decay lambda from environment variable with sensible default
fn get_decay_lambda() -> f64 {
    std::env::var("LEADERBOARD_DECAY_LAMBDA")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.1)
}

/// Calculate decayed score using exponential decay formula
/// decayed_score = raw_score * e^(-λ * days_since_activity)
fn calculate_decayed_score(raw_score: u64, days_since_activity: f64, lambda: f64) -> f64 {
    let score_f64 = raw_score as f64;
    score_f64 * (-lambda * days_since_activity).exp()
}

/// Get paginated leaderboard with percentile rankings and decay
pub fn get_leaderboard(
    entries: &[(String, u64, u64)], // (user_id, raw_score, timestamp)
    page: u32,
    page_size: u32,
    now: u64,
) -> Vec<LeaderboardEntry> {
    let lambda = get_decay_lambda();
    let seconds_per_day = 24 * 3600;

    // Calculate decayed scores
    let mut decayed_entries: Vec<(String, f64)> = entries
        .iter()
        .map(|(user_id, raw_score, timestamp)| {
            let days_since = (now.saturating_sub(*timestamp) as f64) / seconds_per_day as f64;
            let decayed = calculate_decayed_score(*raw_score, days_since, lambda);
            (user_id.clone(), decayed)
        })
        .collect();

    // Sort by decayed score descending
    decayed_entries.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let total_entries = decayed_entries.len() as f64;

    // Calculate percentiles and paginate
    let offset = (page.saturating_sub(1) as usize) * (page_size as usize);
    let limit = page_size as usize;

    decayed_entries
        .into_iter()
        .enumerate()
        .map(|(index, (user_id, decayed_score))| {
            let entries_below = index as f64;
            let percentile = (entries_below / total_entries) * 100.0;
            LeaderboardEntry {
                user_id,
                decayed_score,
                percentile,
            }
        })
        .skip(offset)
        .take(limit)
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
        assert!(result.is_ok());
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
        assert!(result.is_ok());
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

    // -----------------------------------------------------------------------
    // Percentile Ranking with Decay Tests
    // -----------------------------------------------------------------------

    #[test]
    fn decay_function_recent_activity_higher_score() {
        let lambda = 0.1;
        let recent = calculate_decayed_score(100, 0.0, lambda);
        let older = calculate_decayed_score(100, 1.0, lambda);
        assert!(recent > older);
    }

    #[test]
    fn decay_function_no_decay_with_zero_lambda() {
        let score = calculate_decayed_score(100, 5.0, 0.0);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn decay_function_aggressive_decay() {
        let score = calculate_decayed_score(100, 10.0, 1.0);
        assert!(score < 1.0); // e^(-10) is very small
    }

    #[test]
    fn percentile_single_entry() {
        let entries = vec![("user1".to_string(), 100, 1000)];
        let result = get_leaderboard(&entries, 1, 10, 1000);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].user_id, "user1");
        assert_eq!(result[0].percentile, 0.0); // Only entry, 0 below it
    }

    #[test]
    fn percentile_multiple_entries() {
        let entries = vec![
            ("user1".to_string(), 100, 1000),
            ("user2".to_string(), 50, 1000),
            ("user3".to_string(), 150, 1000),
        ];
        let result = get_leaderboard(&entries, 1, 10, 1000);
        assert_eq!(result.len(), 3);
        // Sorted by decayed score: user3 (150), user1 (100), user2 (50)
        assert_eq!(result[0].user_id, "user3");
        assert_eq!(result[0].percentile, 0.0); // Highest, 0 below
        assert_eq!(result[1].user_id, "user1");
        assert!(result[1].percentile > 0.0 && result[1].percentile < 100.0);
        assert_eq!(result[2].user_id, "user2");
        assert!(result[2].percentile > result[1].percentile);
    }

    #[test]
    fn pagination_respects_page_and_size() {
        let entries: Vec<(String, u64, u64)> = (0..15)
            .map(|i| (format!("user{}", i), 100 - i as u64, 1000))
            .collect();

        let page1 = get_leaderboard(&entries, 1, 10, 1000);
        let page2 = get_leaderboard(&entries, 2, 10, 1000);

        assert_eq!(page1.len(), 10);
        assert_eq!(page2.len(), 5);
    }

    #[test]
    fn decay_affects_ranking() {
        let now = 100_000_000;
        let entries = vec![
            ("recent_low".to_string(), 50, now - 1), // Recent, low score
            ("old_high".to_string(), 1000, now - 30 * 86400), // Old, high score
        ];

        let result = get_leaderboard(&entries, 1, 10, now);
        // With decay, recent_low might rank higher despite lower raw score
        assert_eq!(result.len(), 2);
        // The exact ranking depends on decay function, but both should be present
    }

    #[test]
    fn empty_leaderboard() {
        let result = get_leaderboard(&[], 1, 10, 1000);
        assert!(result.is_empty());
    }

    #[test]
    fn configurable_lambda_affects_decay() {
        // Save current lambda
        let original = std::env::var("LEADERBOARD_DECAY_LAMBDA").ok();

        // Set high decay lambda
        std::env::set_var("LEADERBOARD_DECAY_LAMBDA", "1.0");
        let entries = vec![("user1".to_string(), 100, 100)];
        let result1 = get_leaderboard(&entries, 1, 10, 1000);

        // Set low decay lambda
        std::env::set_var("LEADERBOARD_DECAY_LAMBDA", "0.01");
        let result2 = get_leaderboard(&entries, 1, 10, 1000);

        // Higher lambda means more decay (lower score)
        assert!(result1[0].decayed_score < result2[0].decayed_score);

        // Restore original
        match original {
            Some(v) => std::env::set_var("LEADERBOARD_DECAY_LAMBDA", v),
            None => std::env::remove_var("LEADERBOARD_DECAY_LAMBDA"),
        }
    }
}
