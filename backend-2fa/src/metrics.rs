//! Prometheus metrics for the 2FA backend.
//!
//! Exposes a `/metrics` endpoint on an internal-only port (default 9090).
//! The endpoint is unauthenticated; callers must ensure it is not reachable
//! from the public internet (e.g. bind to 127.0.0.1 or an internal VPC
//! interface only).
//!
//! # Metrics
//! | Name | Type | Labels |
//! |------|------|--------|
//! | `totp_verifications_total` | Counter | `result` (`ok`/`fail`) |
//! | `recovery_code_uses_total` | Counter | — |
//! | `rate_limit_hits_total` | Counter | `endpoint`, `reason` |
//! | `rate_limiter_redis_fallback_total` | Counter | — |
//! | `db_pool_active` | Gauge | — |
//! | `db_pool_idle` | Gauge | — |
//! | `request_duration_seconds` | Histogram | `endpoint` |
//! | `leaderboard_ws_connections_total` | Gauge | — |
//!
//! # Registry
//! All metrics are registered with a **dedicated, non-global** [`prometheus::Registry`]
//! stored inside [`Metrics`]. This makes duplicate-registration impossible by
//! construction: the registry is created fresh inside a [`std::sync::OnceLock`],
//! so the initialisation closure runs at most once per process. Any registration
//! failure is logged via [`tracing::error!`] and the metric silently becomes a
//! no-op, rather than panicking the service.

use prometheus::{
    CounterVec, Gauge, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder,
};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Registry singletons
// ---------------------------------------------------------------------------

pub struct Metrics {
    pub totp_verifications_total: CounterVec,
    pub recovery_code_uses_total: prometheus::Counter,
    pub rate_limit_hits_total: CounterVec,
    pub rate_limiter_redis_fallback_total: prometheus::Counter,
    pub db_pool_active: Gauge,
    pub db_pool_idle: Gauge,
    pub request_duration_seconds: HistogramVec,
    /// Tracks the current number of open leaderboard WebSocket connections.
    pub leaderboard_ws_connections_total: Gauge,
    /// Dedicated Prometheus registry. All metrics in this struct are registered
    /// here, never in the global default registry.
    pub(crate) registry: Registry,
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

/// Register `collector` with `registry`, logging an error instead of panicking
/// on failure. Returns the collector so callers can store it after registering.
fn try_register<C>(registry: &Registry, collector: C, name: &str) -> C
where
    C: prometheus::core::Collector + Clone + 'static,
{
    if let Err(e) = registry.register(Box::new(collector.clone())) {
        tracing::error!(
            metric = name,
            error = %e,
            "Prometheus metric registration failed; metric will be silent"
        );
    }
    collector
}

pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(|| {
        let registry = Registry::new();

        let totp_verifications_total = try_register(
            &registry,
            CounterVec::new(
                Opts::new("totp_verifications_total", "Total TOTP verification attempts"),
                &["result"],
            )
            .expect("valid metric name"),
            "totp_verifications_total",
        );

        let recovery_code_uses_total = try_register(
            &registry,
            prometheus::Counter::new("recovery_code_uses_total", "Total recovery code uses")
                .expect("valid metric name"),
            "recovery_code_uses_total",
        );

        let rate_limit_hits_total = try_register(
            &registry,
            CounterVec::new(
                Opts::new("rate_limit_hits_total", "Total rate limit blocks"),
                &["endpoint", "reason"],
            )
            .expect("valid metric name"),
            "rate_limit_hits_total",
        );

        let rate_limiter_redis_fallback_total = try_register(
            &registry,
            prometheus::Counter::new(
                "rate_limiter_redis_fallback_total",
                "Total times DistributedRateLimiter fell back to in-memory due to Redis unavailability",
            )
            .expect("valid metric name"),
            "rate_limiter_redis_fallback_total",
        );

        let db_pool_active = try_register(
            &registry,
            Gauge::new("db_pool_active", "Number of active DB pool connections")
                .expect("valid metric name"),
            "db_pool_active",
        );

        let db_pool_idle = try_register(
            &registry,
            Gauge::new("db_pool_idle", "Number of idle DB pool connections")
                .expect("valid metric name"),
            "db_pool_idle",
        );

        let request_duration_seconds = try_register(
            &registry,
            HistogramVec::new(
                HistogramOpts::new("request_duration_seconds", "HTTP request duration in seconds"),
                &["endpoint"],
            )
            .expect("valid metric name"),
            "request_duration_seconds",
        );

        let leaderboard_ws_connections_total = try_register(
            &registry,
            Gauge::new(
                "leaderboard_ws_connections_total",
                "Current number of open leaderboard WebSocket connections",
            )
            .expect("valid metric name"),
            "leaderboard_ws_connections_total",
        );

        Metrics {
            totp_verifications_total,
            recovery_code_uses_total,
            rate_limit_hits_total,
            rate_limiter_redis_fallback_total,
            db_pool_active,
            db_pool_idle,
            request_duration_seconds,
            leaderboard_ws_connections_total,
            registry,
        }
    })
}

// ---------------------------------------------------------------------------
// Helpers called from handlers / rate limiter
// ---------------------------------------------------------------------------

/// Record a TOTP verification result. `success` maps to label `ok`/`fail`.
pub fn record_totp_verification(success: bool) {
    let label = if success { "ok" } else { "fail" };
    metrics()
        .totp_verifications_total
        .with_label_values(&[label])
        .inc();
}

/// Record a recovery code use.
pub fn record_recovery_code_use() {
    metrics().recovery_code_uses_total.inc();
}

/// Record a rate-limit block with the endpoint and block reason as labels.
///
/// `endpoint` should be a short identifier for the rate-limited route
/// (e.g. `"login"`, `"verify"`, `"disable"`).
/// `reason` describes why the request was blocked
/// (e.g. `"window"` for a sliding-window breach, `"lockout"` for a
/// persistent lockout, `"limit_exceeded"` for a generic counter limit).
pub fn record_rate_limit_hit(endpoint: &str, reason: &str) {
    metrics()
        .rate_limit_hits_total
        .with_label_values(&[endpoint, reason])
        .inc();
}

/// Record a Redis-unavailable fallback in DistributedRateLimiter.
pub fn record_redis_fallback() {
    metrics().rate_limiter_redis_fallback_total.inc();
}

/// Update DB pool gauges.
pub fn set_db_pool_stats(active: f64, idle: f64) {
    metrics().db_pool_active.set(active);
    metrics().db_pool_idle.set(idle);
}

/// Start a request-duration timer for `endpoint`. Drop the returned
/// `HistogramTimer` when the request completes to record the observation.
pub fn start_request_timer(endpoint: &str) -> prometheus::HistogramTimer {
    metrics()
        .request_duration_seconds
        .with_label_values(&[endpoint])
        .start_timer()
}

/// Increment the leaderboard WebSocket connection gauge by 1.
pub fn inc_leaderboard_ws_connections() {
    metrics().leaderboard_ws_connections_total.inc();
}

/// Decrement the leaderboard WebSocket connection gauge by 1.
pub fn dec_leaderboard_ws_connections() {
    metrics().leaderboard_ws_connections_total.dec();
}

// ---------------------------------------------------------------------------
// /metrics handler (framework-agnostic: returns the raw text body)
// ---------------------------------------------------------------------------

/// Render all registered metrics in Prometheus text format.
///
/// Gathers from the dedicated registry owned by [`Metrics`], not the global
/// Prometheus default registry.
pub fn render_metrics() -> Result<String, String> {
    let encoder = TextEncoder::new();
    let metric_families = metrics().registry.gather();
    encoder
        .encode_to_string(&metric_families)
        .map_err(|e| e.to_string())
}

// ---------------------------------------------------------------------------
// Actix-web handler
// ---------------------------------------------------------------------------

/// Actix-web handler for `GET /metrics`.
///
/// Bind the server that mounts this route to an internal address only, e.g.:
/// ```ignore
/// HttpServer::new(|| App::new().route("/metrics", web::get().to(metrics_handler)))
///     .bind("127.0.0.1:9090")?
///     .run()
///     .await
/// ```
pub async fn metrics_handler() -> actix_web::HttpResponse {
    match render_metrics() {
        Ok(body) => actix_web::HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(body),
        Err(e) => actix_web::HttpResponse::InternalServerError().body(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_metrics_contains_expected_names() {
        // Touch each metric so it appears in output even with zero value.
        record_totp_verification(true);
        record_totp_verification(false);
        record_recovery_code_use();
        record_rate_limit_hit("test", "test");
        record_redis_fallback();
        set_db_pool_stats(2.0, 8.0);
        let _timer = start_request_timer("/verify");
        // timer dropped immediately — records a near-zero observation
        inc_leaderboard_ws_connections();
        dec_leaderboard_ws_connections();

        let output = render_metrics().expect("render");
        assert!(output.contains("totp_verifications_total"), "missing totp counter");
        assert!(output.contains("recovery_code_uses_total"), "missing recovery counter");
        assert!(output.contains("rate_limit_hits_total"), "missing rate limit counter");
        assert!(output.contains("db_pool_active"), "missing db_pool_active gauge");
        assert!(output.contains("db_pool_idle"), "missing db_pool_idle gauge");
        assert!(output.contains("request_duration_seconds"), "missing histogram");
        assert!(output.contains("leaderboard_ws_connections_total"), "missing leaderboard ws gauge");
    }

    #[test]
    fn totp_labels_ok_and_fail() {
        record_totp_verification(true);
        record_totp_verification(false);
        let output = render_metrics().expect("render");
        assert!(output.contains(r#"result="ok""#));
        assert!(output.contains(r#"result="fail""#));
    }

    /// Issue #876 — calling `metrics()` a second time must not panic.
    ///
    /// The [`OnceLock`] guarantees the initialisation closure runs exactly once.
    /// The dedicated registry makes duplicate registration structurally
    /// impossible, so there is nothing to panic on even if the function is
    /// called concurrently from multiple threads.
    #[test]
    fn double_init_does_not_panic() {
        let first = metrics() as *const Metrics;
        let second = metrics() as *const Metrics;
        // Both calls must return the same singleton pointer without panicking.
        assert_eq!(first, second, "OnceLock should return the same instance on repeated calls");
    }

    /// Issue #875 — `rate_limit_hits_total` must expose `endpoint` and `reason`
    /// labels so operators can distinguish blocked endpoints and block causes.
    #[test]
    fn rate_limit_hits_labels_appear_in_output() {
        record_rate_limit_hit("login", "window");
        record_rate_limit_hit("verify", "lockout");
        let output = render_metrics().expect("render");
        assert!(
            output.contains(r#"endpoint="login""#),
            "missing endpoint=login label in output"
        );
        assert!(
            output.contains(r#"endpoint="verify""#),
            "missing endpoint=verify label in output"
        );
        assert!(
            output.contains(r#"reason="window""#),
            "missing reason=window label in output"
        );
        assert!(
            output.contains(r#"reason="lockout""#),
            "missing reason=lockout label in output"
        );
    }
}
