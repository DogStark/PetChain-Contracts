/// Unified tracing middleware that injects an `x-request-id` into every log
/// entry for a request, enabling end-to-end correlation across log lines.
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

/// List of sensitive fields that should be redacted in logs
const SENSITIVE_FIELDS: &[&str] = &["totp_code", "secret", "recovery_code", "password", "token", "backup_code"];

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

        // Store on request extensions so handlers can read it.
        req.extensions_mut().insert(RequestId(request_id.clone()));

        let method = req.method().to_string();
        let path = req.path().to_string();

        let span = tracing::info_span!(
            "http_request",
            request_id = %request_id,
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
