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
//! | `rate_limit_hits_total` | Counter | — |
//! | `db_pool_active` | Gauge | — |
//! | `db_pool_idle` | Gauge | — |
//! | `request_duration_seconds` | Histogram | `endpoint` |

use prometheus::{
    register_counter_vec, register_gauge, register_histogram_vec, CounterVec, Gauge, HistogramVec,
    TextEncoder,
};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Registry singletons
// ---------------------------------------------------------------------------

pub struct Metrics {
    pub totp_verifications_total: CounterVec,
    pub recovery_code_uses_total: prometheus::Counter,
    pub rate_limit_hits_total: prometheus::Counter,
    pub db_pool_active: Gauge,
    pub db_pool_idle: Gauge,
    pub request_duration_seconds: HistogramVec,
}

static METRICS: OnceLock<Metrics> = OnceLock::new();

pub fn metrics() -> &'static Metrics {
    METRICS.get_or_init(|| Metrics {
        totp_verifications_total: register_counter_vec!(
            "totp_verifications_total",
            "Total TOTP verification attempts",
            &["result"]
        )
        .expect("register totp_verifications_total"),

        recovery_code_uses_total: prometheus::register_counter!(
            "recovery_code_uses_total",
            "Total recovery code uses"
        )
        .expect("register recovery_code_uses_total"),

        rate_limit_hits_total: prometheus::register_counter!(
            "rate_limit_hits_total",
            "Total rate limit blocks"
        )
        .expect("register rate_limit_hits_total"),

        db_pool_active: register_gauge!(
            "db_pool_active",
            "Number of active DB pool connections"
        )
        .expect("register db_pool_active"),

        db_pool_idle: register_gauge!(
            "db_pool_idle",
            "Number of idle DB pool connections"
        )
        .expect("register db_pool_idle"),

        request_duration_seconds: register_histogram_vec!(
            "request_duration_seconds",
            "HTTP request duration in seconds",
            &["endpoint"]
        )
        .expect("register request_duration_seconds"),
    })
}

// ---------------------------------------------------------------------------
// Helpers called from handlers / rate limiter
// ---------------------------------------------------------------------------

/// Record a TOTP verification result. `success` maps to label `ok`/`fail`.
pub fn record_totp_verification(success: bool) {
    let label = if success { "ok" } else { "fail" };
    metrics().totp_verifications_total.with_label_values(&[label]).inc();
}

/// Record a recovery code use.
pub fn record_recovery_code_use() {
    metrics().recovery_code_uses_total.inc();
}

/// Record a rate-limit block.
pub fn record_rate_limit_hit() {
    metrics().rate_limit_hits_total.inc();
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

// ---------------------------------------------------------------------------
// /metrics handler (framework-agnostic: returns the raw text body)
// ---------------------------------------------------------------------------

/// Render all registered metrics in Prometheus text format.
pub fn render_metrics() -> Result<String, String> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
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
        record_rate_limit_hit();
        set_db_pool_stats(2.0, 8.0);
        let _timer = start_request_timer("/verify");
        // timer dropped immediately — records a near-zero observation

        let output = render_metrics().expect("render");
        assert!(output.contains("totp_verifications_total"), "missing totp counter");
        assert!(output.contains("recovery_code_uses_total"), "missing recovery counter");
        assert!(output.contains("rate_limit_hits_total"), "missing rate limit counter");
        assert!(output.contains("db_pool_active"), "missing db_pool_active gauge");
        assert!(output.contains("db_pool_idle"), "missing db_pool_idle gauge");
        assert!(output.contains("request_duration_seconds"), "missing histogram");
    }

    #[test]
    fn totp_labels_ok_and_fail() {
        record_totp_verification(true);
        record_totp_verification(false);
        let output = render_metrics().expect("render");
        assert!(output.contains(r#"result="ok""#));
        assert!(output.contains(r#"result="fail""#));
    }
}
