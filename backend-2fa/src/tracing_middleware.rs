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

        // Parse traceparent header for W3C Trace Context propagation
        let trace_context = req
            .headers()
            .get(TRACEPARENT_HEADER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| TraceContext::parse(s))
            .or_else(|| {
                // Fall back to generating a fresh trace context
                Some(TraceContext {
                    trace_id: hex::encode(rand::random::<[u8; 16]>()),
                    parent_span_id: "0000000000000000".to_string(),
                    flags: "01".to_string(),
                })
            });

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
}
