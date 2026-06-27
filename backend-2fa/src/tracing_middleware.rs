/// Unified tracing middleware that injects an `x-request-id` into every log
/// entry for a request, enabling end-to-end correlation across log lines.
/// Also supports W3C Trace Context propagation via the traceparent header.
///
/// Usage (actix-web):
/// ```rust
/// use actix_web::App;
/// use petchain_2fa::tracing_middleware::RequestIdMiddleware;
/// let _app = App::new().wrap(RequestIdMiddleware);
/// ```
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use serde_json::Value;
use tracing::Instrument;
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const TRACEPARENT_HEADER: &str = "traceparent";

/// Represents a parsed W3C Trace Context traceparent header
#[derive(Clone, Debug)]
pub struct TraceContext {
    pub trace_id: String,
    pub parent_span_id: String,
    pub flags: String,
}

impl TraceContext {
    /// Parse a traceparent header value (version-traceid-parentid-flags)
    pub fn parse(header_value: &str) -> Option<Self> {
        let parts: Vec<&str> = header_value.split('-').collect();
        if parts.len() != 4 {
            return None;
        }

        let version = parts[0];
        let trace_id = parts[1];
        let parent_span_id = parts[2];
        let flags = parts[3];

        // Validate format: version (2 hex), trace_id (32 hex), parent_span_id (16 hex), flags (2 hex)
        if version.len() != 2
            || trace_id.len() != 32
            || parent_span_id.len() != 16
            || flags.len() != 2
        {
            return None;
        }

        // Validate all parts are valid hex
        if !version.chars().all(|c| c.is_ascii_hexdigit())
            || !trace_id.chars().all(|c| c.is_ascii_hexdigit())
            || !parent_span_id.chars().all(|c| c.is_ascii_hexdigit())
            || !flags.chars().all(|c| c.is_ascii_hexdigit())
        {
            return None;
        }

        Some(TraceContext {
            trace_id: trace_id.to_string(),
            parent_span_id: parent_span_id.to_string(),
            flags: flags.to_string(),
        })
    }

    /// Generate a traceparent header value
    pub fn to_header(&self) -> String {
        format!(
            "00-{}-{}-{}",
            self.trace_id, self.parent_span_id, self.flags
        )
    }
}

/// List of sensitive fields that should be redacted in logs
const SENSITIVE_FIELDS: &[&str] = &[
    "totp_code",
    "secret",
    "recovery_code",
    "password",
    "token",
    "backup_code",
];

/// Sanitize JSON by redacting sensitive fields
pub fn sanitize_json_body(body: &str) -> String {
    match serde_json::from_str::<Value>(body) {
        Ok(mut json_value) => {
            sanitize_value(&mut json_value);
            json_value.to_string()
        }
        Err(_) => "[binary]".to_string(),
    }
}

/// Recursively sanitize a JSON value by replacing sensitive field values with [REDACTED]
fn sanitize_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if SENSITIVE_FIELDS.contains(&key.as_str()) {
                    *val = Value::String("[REDACTED]".to_string());
                } else {
                    sanitize_value(val);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_value(item);
            }
        }
        _ => {}
    }
}

/// Actix-web middleware factory.
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIdMiddlewareService { service })
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Reuse incoming request-id or generate a new one.
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        // Parse traceparent header for W3C Trace Context propagation.
        // Use synthesize_trace_context which respects TRACE_CONTEXT_AUTOGENERATE.
        let trace_context = synthesize_trace_context(
            req
                .headers()
                .get(TRACEPARENT_HEADER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| TraceContext::parse(s)),
        );

        if let Some(tc) = &trace_context {
            tracing::debug!(
                trace_id = %tc.trace_id,
                parent_span_id = %tc.parent_span_id,
                "Trace context processed"
            );
        }

        // Store on request extensions so handlers can read it.
        req.extensions_mut().insert(RequestId(request_id.clone()));
        if let Some(tc) = trace_context.clone() {
            req.extensions_mut().insert(tc);
        }

        let method = req.method().to_string();
        let path = req.path().to_string();
        let trace_id = trace_context
            .as_ref()
            .map(|tc| tc.trace_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let span = tracing::info_span!(
            "http_request",
            request_id = %request_id,
            trace_id = %trace_id,
            method = %method,
            path = %path,
        );

        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.instrument(span).await?;
            // Echo the request-id back in the response headers.
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static(REQUEST_ID_HEADER),
                actix_web::http::header::HeaderValue::from_str(&request_id)
                    .unwrap_or_else(|_| actix_web::http::header::HeaderValue::from_static("")),
            );

            // Propagate traceparent header in response if available
            if let Some(tc) = trace_context {
                if let Ok(header_val) =
                    actix_web::http::header::HeaderValue::from_str(&tc.to_header())
                {
                    res.headers_mut().insert(
                        actix_web::http::header::HeaderName::from_static(TRACEPARENT_HEADER),
                        header_val,
                    );
                }
            }

            Ok(res)
        })
    }
}

/// Newtype wrapper stored in request extensions.
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Check the `TRACE_CONTEXT_AUTOGENERATE` env var; synthesis is skipped when the
/// value is exactly `"0"`. Any other value (or unset) preserves the default behaviour.
fn autogenerate_trace_context_enabled() -> bool {
    std::env::var("TRACE_CONTEXT_AUTOGENERATE").as_deref() != Ok("0")
}

/// Given an already-parsed trace context (from an incoming header or `None`),
/// decide what to propagate:
/// - If a parsed context exists, return it as-is.
/// - If none exists and autogenerate is enabled, synthesise a fresh one.
/// - If none exists and autogenerate is disabled, return `None`.
pub fn synthesize_trace_context(incoming: Option<TraceContext>) -> Option<TraceContext> {
    incoming.or_else(|| {
        if autogenerate_trace_context_enabled() {
            Some(TraceContext {
                trace_id: hex::encode(rand::random::<[u8; 16]>()),
                parent_span_id: "0000000000000000".to_string(),
                flags: "01".to_string(),
            })
        } else {
            None
        }
    })
}

/// Initialise a `tracing-subscriber` that emits structured JSON logs.
/// Call once at application startup before building the Actix app.
pub fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let _ = fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_trace_id_is_32_lowercase_hex() {
        let trace_id = hex::encode(rand::random::<[u8; 16]>());
        assert_eq!(trace_id.len(), 32, "trace_id must be 32 chars");
        assert!(
            trace_id.chars().all(|c| c.is_ascii_hexdigit()),
            "trace_id must be hex"
        );
        // Ensure TraceContext::parse accepts it
        let header = format!("00-{trace_id}-0000000000000000-01");
        assert!(TraceContext::parse(&header).is_some());
    }

    #[test]
    fn parse_rejects_empty_string() {
        assert!(TraceContext::parse("").is_none());
    }

    #[test]
    fn parse_rejects_wrong_segment_count() {
        assert!(TraceContext::parse("00").is_none());
        assert!(TraceContext::parse("00-abc").is_none());
        assert!(TraceContext::parse("00-abc-def").is_none());
        assert!(TraceContext::parse("00-a-b-c-d").is_none());
    }

    #[test]
    fn parse_rejects_non_hex_characters() {
        assert!(TraceContext::parse(
            "zz-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ-00f067aa0ba902b7-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-ghijklmnopqrstuv-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-xx"
        ).is_none());
    }

    #[test]
    fn parse_rejects_wrong_length_segments() {
        assert!(TraceContext::parse(
            "0-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-4bf92f3577b34da6a3ce929d0e0e473-00f067aa0ba902b7-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b-01"
        ).is_none());
        assert!(TraceContext::parse(
            "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-0"
        ).is_none());
    }

    #[test]
    fn parse_rejects_multibyte_unicode_that_reports_misleading_len() {
        let multibyte_32 = "é".repeat(16);
        assert_eq!(multibyte_32.chars().count(), 16);
        assert!(multibyte_32.len() > 32);
        let header = format!("00-{}-0000000000000000-01", multibyte_32);
        assert!(TraceContext::parse(&header).is_none());
    }

    #[test]
    fn parse_rejects_embedded_nulls_and_control_chars() {
        let with_null = "00-4bf92f3577b34da6a3ce929d\x000e4736-00f067aa0ba902b7-01";
        assert!(TraceContext::parse(with_null).is_none());

        let with_newline = "00-4bf92f3577b34da6a3ce929d\n0e4736-00f067aa0ba902b7-01";
        assert!(TraceContext::parse(with_newline).is_none());
    }

    #[test]
    fn parse_rejects_leading_trailing_dashes_and_only_dashes() {
        assert!(TraceContext::parse("---").is_none());
        assert!(TraceContext::parse("-00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01").is_none());
        assert!(TraceContext::parse("00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-").is_none());
    }

    #[test]
    fn parse_never_panics_on_random_input() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..10_000 {
            let len = rng.gen_range(0..256);
            let bytes: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
            let input = String::from_utf8_lossy(&bytes);
            let _ = TraceContext::parse(&input);
        }

        for _ in 0..5_000 {
            let ndashes = rng.gen_range(0..6);
            let mut parts = Vec::new();
            for _ in 0..=ndashes {
                let seg_len = rng.gen_range(0..40);
                let seg: String = (0..seg_len)
                    .map(|_| {
                        let charset = b"0123456789abcdefABCDEF \x00\n\txyz";
                        charset[rng.gen_range(0..charset.len())] as char
                    })
                    .collect();
                parts.push(seg);
            }
            let input = parts.join("-");
            let _ = TraceContext::parse(&input);
        }
    }

    #[test]
    fn parse_accepts_all_valid_hex_combinations() {
        let valid = "00-0123456789abcdef0123456789abcdef-0123456789abcdef-00";
        assert!(TraceContext::parse(valid).is_some());

        let upper_hex = "00-0123456789ABCDEF0123456789ABCDEF-0123456789ABCDEF-FF";
        assert!(TraceContext::parse(upper_hex).is_some());
    }

    #[test]
    fn traceparent_parse_roundtrip() {
        let tc = TraceContext {
            trace_id: "4bf92f3577b34da6a3ce929d0e0e4736".to_string(),
            parent_span_id: "00f067aa0ba902b7".to_string(),
            flags: "01".to_string(),
        };
        let header = tc.to_header();
        let parsed = TraceContext::parse(&header).expect("should parse");
        assert_eq!(parsed.trace_id, tc.trace_id);
        assert_eq!(parsed.parent_span_id, tc.parent_span_id);
        assert_eq!(parsed.flags, tc.flags);
    }

    // ── Issue #882: Array-of-objects sanitization ──────────────────────────

    #[test]
    fn sanitize_json_body_redacts_sensitive_fields_in_array_of_objects() {
        let payload = serde_json::json!([
            { "token": "abc123", "username": "alice" },
            { "token": "def456", "username": "bob", "totp_code": "987654" },
            { "username": "charlie", "secret": "JBSWY3DPEHPK3PXP" },
        ])
        .to_string();

        let sanitized = sanitize_json_body(&payload);
        let parsed: serde_json::Value =
            serde_json::from_str(&sanitized).expect("sanitized output must be valid JSON");

        match &parsed {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                // First object
                assert_eq!(arr[0]["username"], "alice");
                assert_eq!(arr[0]["token"], "[REDACTED]");
                // Second object
                assert_eq!(arr[1]["username"], "bob");
                assert_eq!(arr[1]["token"], "[REDACTED]");
                assert_eq!(arr[1]["totp_code"], "[REDACTED]");
                // Third object
                assert_eq!(arr[2]["username"], "charlie");
                assert_eq!(arr[2]["secret"], "[REDACTED]");
            }
            other => panic!("expected JSON array, got {other:?}"),
        }

        // Verify the raw sensitive values do NOT appear in the output
        assert!(!sanitized.contains("abc123"), "raw token abc123 leaked");
        assert!(!sanitized.contains("def456"), "raw token def456 leaked");
        assert!(!sanitized.contains("987654"), "raw totp_code 987654 leaked");
        assert!(
            !sanitized.contains("JBSWY3DPEHPK3PXP"),
            "raw secret leaked"
        );
    }

    #[test]
    fn sanitize_json_body_redacts_sensitive_fields_in_deeply_nested_arrays() {
        let payload = serde_json::json!({
            "batch": [
                { "token": "sensitive1" },
                { "nested": { "token": "sensitive2" } },
                { "deep": [ { "secret": "very-sensitive" } ] }
            ]
        })
        .to_string();

        let sanitized = sanitize_json_body(&payload);
        let parsed: serde_json::Value =
            serde_json::from_str(&sanitized).expect("sanitized output must be valid JSON");

        assert_eq!(parsed["batch"][0]["token"], "[REDACTED]");
        assert_eq!(parsed["batch"][1]["nested"]["token"], "[REDACTED]");
        assert_eq!(parsed["batch"][2]["deep"][0]["secret"], "[REDACTED]");

        assert!(!sanitized.contains("sensitive1"));
        assert!(!sanitized.contains("sensitive2"));
        assert!(!sanitized.contains("very-sensitive"));
    }

    // ── Issue #881: TRACE_CONTEXT_AUTOGENERATE flag ────────────────────────

    #[test]
    fn trace_context_autogenerate_disabled_returns_none_when_no_header() {
        // When TRACE_CONTEXT_AUTOGENERATE=0, synthesis should be skipped
        std::env::set_var("TRACE_CONTEXT_AUTOGENERATE", "0");

        let result = synthesize_trace_context(None);
        assert!(result.is_none(), "expected None when autogenerate is disabled and no header present");

        std::env::remove_var("TRACE_CONTEXT_AUTOGENERATE");
    }

    #[test]
    fn trace_context_autogenerate_enabled_synthesizes_when_no_header() {
        // Default behaviour (unset or any value other than "0") should synthesise
        std::env::remove_var("TRACE_CONTEXT_AUTOGENERATE");

        let result = synthesize_trace_context(None);
        assert!(result.is_some(), "expected Some trace context when autogenerate is enabled");
        if let Some(tc) = result {
            assert_eq!(tc.trace_id.len(), 32);
            assert_eq!(tc.parent_span_id, "0000000000000000");
            assert_eq!(tc.flags, "01");
        }
    }

    #[test]
    fn trace_context_autogenerate_disabled_still_parses_incoming_header() {
        std::env::set_var("TRACE_CONTEXT_AUTOGENERATE", "0");

        let header = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let ctx = TraceContext::parse(header);
        assert!(ctx.is_some(), "incoming header should still be parsed");

        let result = synthesize_trace_context(ctx);
        assert!(result.is_some(), "should return the parsed trace context even when autogenerate is disabled");

        std::env::remove_var("TRACE_CONTEXT_AUTOGENERATE");
    }
}
