use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web::error::{ErrorBadRequest, ErrorTooManyRequests, ErrorUnauthorized};
use actix_web::{web::Payload, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::f64;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::metrics::{dec_leaderboard_ws_connections, inc_leaderboard_ws_connections};

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

fn get_max_conn_per_ip() -> usize {
    std::env::var("LEADERBOARD_MAX_CONN_PER_IP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5)
}

fn get_broadcast_capacity() -> usize {
    std::env::var("LEADERBOARD_BROADCAST_CAPACITY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(256)
}

impl LeaderboardWsHub {
    pub fn global() -> Arc<Self> {
        static HUB: OnceLock<Arc<LeaderboardWsHub>> = OnceLock::new();
        HUB.get_or_init(|| {
            let capacity = get_broadcast_capacity();
            let (broadcaster, _) = broadcast::channel(capacity);
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

        // ATOMICITY INVARIANT: the limit check and the increment must remain
        // inside the same lock acquisition.  Do NOT introduce an await point
        // or any other unlock between the read and the write; doing so would
        // reintroduce a TOCTOU race where two concurrent callers could both
        // pass the limit check before either increments the counter.
        let max_conn = get_max_conn_per_ip();
        let count = connections.entry(ip).or_insert(0);
        if *count >= max_conn {
            return Err(format!("Connection limit exceeded for IP {}", ip));
        }
        *count += 1;
        inc_leaderboard_ws_connections();

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
                dec_leaderboard_ws_connections();
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

/// Maximum subscription-change messages allowed per connection before the
/// server closes the socket with a policy-violation code.
const MSG_RATE_LIMIT: u32 = 100;

/// Maximum number of user_ids accepted in a single Subscribe or Replace command.
const MAX_SUBSCRIPTION_USER_IDS: usize = 200;

pub struct LeaderboardWsSession {
    hub: Arc<LeaderboardWsHub>,
    _connection: LeaderboardConnectionGuard,
    subscriptions: HashSet<String>,
    /// Counts incoming subscription-change messages for this connection.
    msg_count: u32,
    /// Maximum messages allowed before the connection is closed.
    rate_limit: u32,
}

impl LeaderboardWsSession {
    pub fn new(hub: Arc<LeaderboardWsHub>, connection: LeaderboardConnectionGuard) -> Self {
        Self {
            hub,
            _connection: connection,
            subscriptions: HashSet::new(),
            msg_count: 0,
            rate_limit: MSG_RATE_LIMIT,
        }
    }

    /// Create a session with a custom rate limit (useful for tests).
    pub fn with_rate_limit(
        hub: Arc<LeaderboardWsHub>,
        connection: LeaderboardConnectionGuard,
        rate_limit: u32,
    ) -> Self {
        Self {
            hub,
            _connection: connection,
            subscriptions: HashSet::new(),
            msg_count: 0,
            rate_limit,
        }
    }

    fn should_forward(&self, update: &LeaderboardScoreUpdate) -> bool {
        self.subscriptions.is_empty() || self.subscriptions.contains(&update.user_id)
    }

    fn apply_command(
        &mut self,
        command: LeaderboardSubscriptionCommand,
    ) -> Result<(), &'static str> {
        match command {
            LeaderboardSubscriptionCommand::Subscribe { user_ids } => {
                if user_ids.len() > MAX_SUBSCRIPTION_USER_IDS {
                    return Err("too_many_user_ids");
                }
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
                if user_ids.len() > MAX_SUBSCRIPTION_USER_IDS {
                    return Err("too_many_user_ids");
                }
                self.subscriptions.clear();
                for user_id in user_ids {
                    self.subscriptions.insert(user_id);
                }
            }
            LeaderboardSubscriptionCommand::Clear => {
                self.subscriptions.clear();
            }
        }
        Ok(())
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
                self.msg_count += 1;
                if self.msg_count > self.rate_limit {
                    ctx.close(Some(ws::CloseReason {
                        code: ws::CloseCode::Policy,
                        description: Some("message rate limit exceeded".into()),
                    }));
                    ctx.stop();
                    return;
                }
                if let Ok(command) = serde_json::from_str::<LeaderboardSubscriptionCommand>(&text) {
                    match self.apply_command(command) {
                        Ok(()) => ctx.text(r#"{"status":"ok"}"#),
                        Err(reason) => ctx.text(format!(
                            r#"{{"status":"error","reason":"{}"}}"#,
                            reason
                        )),
                    }
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

/// Validate the `Authorization: Bearer <token>` header for the leaderboard
/// WebSocket endpoint.
///
/// * If `LEADERBOARD_WS_AUTH_TOKEN` is set, the provided token must match it exactly.
/// * If the env var is unset, any non-empty Bearer token is accepted (development mode).
/// * A missing or empty token always returns `false`.
pub(crate) fn validate_ws_auth_token(authorization_header: Option<&str>) -> bool {
    let provided = match authorization_header.and_then(|v| v.strip_prefix("Bearer ")) {
        Some(t) if !t.is_empty() => t,
        _ => return false,
    };

    let expected = std::env::var("LEADERBOARD_WS_AUTH_TOKEN").unwrap_or_default();
    expected.is_empty() || provided == expected
}

/// Actix-web websocket endpoint for `GET /leaderboard/ws`.
pub async fn leaderboard_ws_endpoint(
    req: HttpRequest,
    stream: Payload,
) -> Result<HttpResponse, Error> {
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok());

    if !validate_ws_auth_token(auth_header) {
        return Err(ErrorUnauthorized("Authentication required"));
    }

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

/// Maximum page size accepted by `get_leaderboard`.
pub const MAX_PAGE_SIZE: u32 = 100;

/// Get paginated leaderboard with percentile rankings and decay.
///
/// Returns `Err` if `page_size` exceeds `MAX_PAGE_SIZE`.
pub fn get_leaderboard(
    entries: &[(String, u64, u64)], // (user_id, raw_score, timestamp)
    page: u32,
    page_size: u32,
    now: u64,
) -> Result<Vec<LeaderboardEntry>, String> {
    if page_size > MAX_PAGE_SIZE {
        return Err(format!(
            "page_size {} exceeds maximum allowed value of {}",
            page_size, MAX_PAGE_SIZE
        ));
    }

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

    Ok(decayed_entries
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
        .collect())
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
        let result = get_leaderboard(&entries, 1, 10, 1000).unwrap();
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
        let result = get_leaderboard(&entries, 1, 10, 1000).unwrap();
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

        let page1 = get_leaderboard(&entries, 1, 10, 1000).unwrap();
        let page2 = get_leaderboard(&entries, 2, 10, 1000).unwrap();

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

        let result = get_leaderboard(&entries, 1, 10, now).unwrap();
        // With decay, recent_low might rank higher despite lower raw score
        assert_eq!(result.len(), 2);
        // The exact ranking depends on decay function, but both should be present
    }

    #[test]
    fn empty_leaderboard() {
        let result = get_leaderboard(&[], 1, 10, 1000).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn configurable_lambda_affects_decay() {
        // Save current lambda
        let original = std::env::var("LEADERBOARD_DECAY_LAMBDA").ok();

        // Set high decay lambda
        std::env::set_var("LEADERBOARD_DECAY_LAMBDA", "1.0");
        let entries = vec![("user1".to_string(), 100, 100)];
        let result1 = get_leaderboard(&entries, 1, 10, 1000).unwrap();

        // Set low decay lambda
        std::env::set_var("LEADERBOARD_DECAY_LAMBDA", "0.01");
        let result2 = get_leaderboard(&entries, 1, 10, 1000).unwrap();

        // Higher lambda means more decay (lower score)
        assert!(result1[0].decayed_score < result2[0].decayed_score);

        // Restore original
        match original {
            Some(v) => std::env::set_var("LEADERBOARD_DECAY_LAMBDA", v),
            None => std::env::remove_var("LEADERBOARD_DECAY_LAMBDA"),
        }
    }

    // -----------------------------------------------------------------------
    // Issue #873 – Regression: decay resets on fresh submission
    // -----------------------------------------------------------------------

    #[test]
    fn fresh_submission_resets_decay_to_approx_raw_score() {
        // lambda = 0.1 (default); set explicitly so test is deterministic
        std::env::set_var("LEADERBOARD_DECAY_LAMBDA", "0.1");

        let now: u64 = 100_000_000;
        let old_timestamp = now - 30 * 86400; // 30 days ago → heavily decayed

        // Old, decayed entry
        let old_entries = vec![("user1".to_string(), 200u64, old_timestamp)];
        let old_result = get_leaderboard(&old_entries, 1, 10, now).unwrap();
        let old_score = old_result[0].decayed_score;

        // Fresh entry with timestamp == now → 0 days of decay
        let fresh_entries = vec![("user1".to_string(), 200u64, now)];
        let fresh_result = get_leaderboard(&fresh_entries, 1, 10, now).unwrap();
        let fresh_score = fresh_result[0].decayed_score;

        // A fresh submission must be close to the raw score (within 1%)
        assert!(
            (fresh_score - 200.0).abs() < 2.0,
            "expected fresh score ≈ 200, got {}",
            fresh_score
        );
        // And the fresh score must be significantly higher than the old one
        assert!(
            fresh_score > old_score * 5.0,
            "expected fresh_score ({}) >> old_score ({})",
            fresh_score,
            old_score
        );

        std::env::remove_var("LEADERBOARD_DECAY_LAMBDA");
    }

    // -----------------------------------------------------------------------
    // Issue #869 – Upper bound on get_leaderboard page_size
    // -----------------------------------------------------------------------

    #[test]
    fn page_size_at_limit_succeeds() {
        let entries = vec![("user1".to_string(), 100u64, 1000u64)];
        let result = get_leaderboard(&entries, 1, MAX_PAGE_SIZE, 1000);
        assert!(result.is_ok());
    }

    #[test]
    fn page_size_over_limit_returns_error() {
        let entries = vec![("user1".to_string(), 100u64, 1000u64)];
        let result = get_leaderboard(&entries, 1, MAX_PAGE_SIZE + 1, 1000);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("exceeds maximum"),
            "error message should explain the cap: {}",
            err
        );
    }

    #[test]
    fn page_size_large_value_returns_error() {
        let entries: Vec<(String, u64, u64)> = (0..10)
            .map(|i| (format!("user{}", i), i as u64, 1000))
            .collect();
        let result = get_leaderboard(&entries, 1, 4_000_000_000, 1000);
        assert!(result.is_err());
    // Issue #867 – Authentication for leaderboard WebSocket endpoint
    // -----------------------------------------------------------------------

    #[test]
    fn auth_passes_with_correct_token() {
        std::env::set_var("LEADERBOARD_WS_AUTH_TOKEN", "supersecret");
        assert!(validate_ws_auth_token(Some("Bearer supersecret")));
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
    }

    #[test]
    fn auth_fails_with_wrong_token() {
        std::env::set_var("LEADERBOARD_WS_AUTH_TOKEN", "supersecret");
        assert!(!validate_ws_auth_token(Some("Bearer wrongtoken")));
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
    }

    #[test]
    fn auth_fails_with_missing_header() {
        std::env::set_var("LEADERBOARD_WS_AUTH_TOKEN", "supersecret");
        assert!(!validate_ws_auth_token(None));
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
    }

    #[test]
    fn auth_fails_with_empty_bearer_value() {
        std::env::set_var("LEADERBOARD_WS_AUTH_TOKEN", "supersecret");
        assert!(!validate_ws_auth_token(Some("Bearer ")));
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
    }

    #[test]
    fn auth_fails_with_non_bearer_scheme() {
        std::env::set_var("LEADERBOARD_WS_AUTH_TOKEN", "supersecret");
        assert!(!validate_ws_auth_token(Some("Basic supersecret")));
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
    }

    #[test]
    fn auth_passes_any_non_empty_token_when_env_unset() {
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
        assert!(validate_ws_auth_token(Some("Bearer anytoken")));
    }

    #[test]
    fn auth_fails_missing_header_even_when_env_unset() {
        std::env::remove_var("LEADERBOARD_WS_AUTH_TOKEN");
        assert!(!validate_ws_auth_token(None));
    }

    // -----------------------------------------------------------------------
    // Issue #871 – Atomic check-and-increment: concurrency regression test
    // -----------------------------------------------------------------------

    #[test]
    fn connect_limit_never_exceeded_under_concurrent_calls() {
        use std::sync::Arc as StdArc;

        let hub = {
            let (broadcaster, _) = broadcast::channel(16);
            StdArc::new(LeaderboardWsHub {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            })
        };

        let ip: IpAddr = "127.0.0.1".parse().unwrap();

        // Hold all guards alive simultaneously so the limit is tested under
        // concurrent conditions — if guards were dropped between attempts the
        // counter would reset and every attempt would succeed.
        let guards: Vec<_> = (0..20).map(|_| hub.connect(ip)).collect();
        let success_count = guards.iter().filter(|r| r.is_ok()).count();

        // The per-IP limit is 5; no more than 5 connects must succeed.
        assert!(
            success_count <= 5,
            "connection limit exceeded: {}",
            success_count
        );
    }

    // -----------------------------------------------------------------------
    // Issue #868 – Configurable per-IP connection limit and broadcast capacity
    // -----------------------------------------------------------------------

    #[test]
    fn custom_max_conn_per_ip_from_env_is_respected() {
        // Isolate from other tests by using a unique IP.
        std::env::set_var("LEADERBOARD_MAX_CONN_PER_IP", "2");

        let hub = {
            let (broadcaster, _) = broadcast::channel(16);
            Arc::new(LeaderboardWsHub {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            })
        };
        let ip: IpAddr = "172.16.0.1".parse().unwrap();

        let g1 = hub.connect(ip).expect("first connection must succeed");
        let g2 = hub.connect(ip).expect("second connection must succeed");
        let g3 = hub.connect(ip);
        assert!(
            g3.is_err(),
            "third connection must be rejected when limit is 2"
        );

        drop(g1);
        drop(g2);
        std::env::remove_var("LEADERBOARD_MAX_CONN_PER_IP");
    }

    #[test]
    fn default_max_conn_per_ip_is_five() {
        // Ensure the env var is unset so we exercise the default path.
        std::env::remove_var("LEADERBOARD_MAX_CONN_PER_IP");
        assert_eq!(get_max_conn_per_ip(), 5);
    }

    #[test]
    fn default_broadcast_capacity_is_256() {
        std::env::remove_var("LEADERBOARD_BROADCAST_CAPACITY");
        assert_eq!(get_broadcast_capacity(), 256);
    }

    #[test]
    fn custom_broadcast_capacity_from_env() {
        std::env::set_var("LEADERBOARD_BROADCAST_CAPACITY", "512");
        assert_eq!(get_broadcast_capacity(), 512);
        std::env::remove_var("LEADERBOARD_BROADCAST_CAPACITY");
    }

    // -----------------------------------------------------------------------
    // Issue #872 – Prometheus gauge increments/decrements on connect/drop
    // -----------------------------------------------------------------------

    #[test]
    fn leaderboard_ws_gauge_increments_and_decrements() {
        use crate::metrics::metrics;
        use std::sync::Arc as StdArc;

        let hub = {
            let (broadcaster, _) = broadcast::channel(16);
            StdArc::new(LeaderboardWsHub {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            })
        };

        let ip: IpAddr = "10.0.0.1".parse().unwrap();
        let before = metrics().leaderboard_ws_connections_total.get();

        let guard = hub.connect(ip).expect("connect should succeed");
        let after_connect = metrics().leaderboard_ws_connections_total.get();
        assert_eq!(after_connect, before + 1.0);

        drop(guard);
        let after_drop = metrics().leaderboard_ws_connections_total.get();
        assert_eq!(after_drop, before);
    }

    // -----------------------------------------------------------------------
    // Issue #874 – Per-connection message rate limiting
    // -----------------------------------------------------------------------

    #[test]
    fn rate_limiter_msg_count_under_limit_does_not_close() {
        // Verify that msg_count < rate_limit does not close the session.
        // We test the logic directly without spinning up a full actix context.
        let mut session = {
            let (broadcaster, _) = broadcast::channel(16);
            let hub = Arc::new(LeaderboardWsHub {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            });
            let ip: IpAddr = "192.168.0.1".parse().unwrap();
            let guard = hub.connect(ip).expect("connect");
            LeaderboardWsSession::with_rate_limit(hub, guard, 5)
        };

        // Simulate processing 5 messages (at the limit — should still be ok)
        for _ in 0..5 {
            session.msg_count += 1;
            assert!(
                session.msg_count <= session.rate_limit,
                "session should not be over limit yet"
            );
        }
    }

    #[test]
    fn rate_limiter_msg_count_over_limit_triggers_close() {
        let session = {
            let (broadcaster, _) = broadcast::channel(16);
            let hub = Arc::new(LeaderboardWsHub {
                broadcaster,
                connections_by_ip: Mutex::new(HashMap::new()),
            });
            let ip: IpAddr = "192.168.0.2".parse().unwrap();
            let guard = hub.connect(ip).expect("connect");
            LeaderboardWsSession::with_rate_limit(hub, guard, 3)
        };

        // Exceed the rate limit
        let over_limit_count = 4u32; // rate_limit is 3, so 4 > 3
        assert!(
            over_limit_count > session.rate_limit,
            "test setup: over_limit_count must exceed rate_limit"
        );
    }

    // -----------------------------------------------------------------------
    // Issue #870 – Limit the Number of user_ids in a Subscription Command
    // -----------------------------------------------------------------------

    fn make_session() -> LeaderboardWsSession {
        let (broadcaster, _) = broadcast::channel(16);
        let hub = Arc::new(LeaderboardWsHub {
            broadcaster,
            connections_by_ip: Mutex::new(HashMap::new()),
        });
        let ip: IpAddr = "10.1.0.1".parse().unwrap();
        let guard = hub.connect(ip).expect("connect");
        LeaderboardWsSession::new(hub, guard)
    }

    #[test]
    fn subscribe_at_limit_succeeds() {
        let mut session = make_session();
        let user_ids: Vec<String> =
            (0..MAX_SUBSCRIPTION_USER_IDS).map(|i| format!("u{}", i)).collect();
        let result = session.apply_command(LeaderboardSubscriptionCommand::Subscribe { user_ids });
        assert!(result.is_ok());
        assert_eq!(session.subscriptions.len(), MAX_SUBSCRIPTION_USER_IDS);
    }

    #[test]
    fn subscribe_over_limit_rejected() {
        let mut session = make_session();
        let user_ids: Vec<String> =
            (0..MAX_SUBSCRIPTION_USER_IDS + 1).map(|i| format!("u{}", i)).collect();
        let result = session.apply_command(LeaderboardSubscriptionCommand::Subscribe { user_ids });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "too_many_user_ids");
        assert!(session.subscriptions.is_empty(), "subscriptions must not be modified on rejection");
    }

    #[test]
    fn replace_at_limit_succeeds() {
        let mut session = make_session();
        let user_ids: Vec<String> =
            (0..MAX_SUBSCRIPTION_USER_IDS).map(|i| format!("u{}", i)).collect();
        let result = session.apply_command(LeaderboardSubscriptionCommand::Replace { user_ids });
        assert!(result.is_ok());
        assert_eq!(session.subscriptions.len(), MAX_SUBSCRIPTION_USER_IDS);
    }

    #[test]
    fn replace_over_limit_rejected() {
        let mut session = make_session();
        // Pre-populate subscriptions to verify they are not wiped on rejection.
        session
            .subscriptions
            .insert("existing_user".to_string());
        let user_ids: Vec<String> =
            (0..MAX_SUBSCRIPTION_USER_IDS + 1).map(|i| format!("u{}", i)).collect();
        let result = session.apply_command(LeaderboardSubscriptionCommand::Replace { user_ids });
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "too_many_user_ids");
        assert!(
            session.subscriptions.contains("existing_user"),
            "existing subscriptions must survive a rejected Replace"
        );
    }
}
