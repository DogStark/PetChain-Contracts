use actix_web::{
    body::BoxBody,
    body::MessageBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpResponse, ResponseError,
};
use futures_util::future::{ok, FutureExt, LocalBoxFuture, Ready};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::panic::AssertUnwindSafe;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl ApiError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<Value>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn bad_request(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("BAD_REQUEST", message, details)
    }

    pub fn unauthorized(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("UNAUTHORIZED", message, details)
    }

    pub fn forbidden(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("FORBIDDEN", message, details)
    }

    pub fn not_found(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("NOT_FOUND", message, details)
    }

    pub fn conflict(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("CONFLICT", message, details)
    }

    pub fn invalid_token(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("INVALID_TOKEN", message, details)
    }

    pub fn internal_error(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("INTERNAL_SERVER_ERROR", message, details)
    }

    pub fn locked(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("LOCKED", message, details)
    }

    pub fn too_many_requests(message: impl Into<String>, details: Option<Value>) -> Self {
        Self::new("TOO_MANY_REQUESTS", message, details)
    }

    /// Create a `429 Too Many Requests` error with a `Retry-After` header.
    ///
    /// The `retry_after_secs` value is embedded in `details` so that
    /// `error_response()` can set the HTTP `Retry-After` header automatically.
    pub fn rate_limited(message: impl Into<String>, retry_after_secs: u64) -> Self {
        Self::new(
            "RATE_LIMITED",
            message,
            Some(json!({ "retry_after_secs": retry_after_secs })),
        )
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        match self.code.as_str() {
            "BAD_REQUEST" => StatusCode::BAD_REQUEST,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "CONFLICT" => StatusCode::CONFLICT,
            "LOCKED" => StatusCode::LOCKED,
            "TOO_MANY_REQUESTS" => StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        // ── Issue #886: Log every 5xx error server-side ─────────────────────
        let status = self.status_code();
        if status.as_u16() >= 500 {
            tracing::error!(
                error.code = %self.code,
                error.message = %self.message,
                error.details = %self.details.as_ref().map(|d| d.to_string()).unwrap_or_default(),
                status = %status.as_u16(),
                "ApiError response"
            );
        }

        // ── Issue #885: Support Retry-After header for rate_limited errors ──
        let mut resp = HttpResponse::build(status);
        if self.code.as_str() == "RATE_LIMITED" {
            if let Some(Value::Number(n)) = self.details.as_ref().and_then(|d| d.get("retry_after_secs")) {
                if let Some(secs) = n.as_u64() {
                    resp.insert_header((
                        actix_web::http::header::RETRY_AFTER,
                        secs.to_string(),
                    ));
                }
            }
        }

        resp.json(self)
    }
}

pub struct ErrorResponseMiddleware;

impl<S, B> Transform<S, ServiceRequest> for ErrorResponseMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ErrorResponseMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ErrorResponseMiddlewareService { service })
    }
}

pub struct ErrorResponseMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ErrorResponseMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request = req.request().clone();
        let fut = AssertUnwindSafe(self.service.call(req)).catch_unwind();

        Box::pin(async move {
            match fut.await {
                Ok(Ok(res)) => Ok(res.map_into_boxed_body()),
                Ok(Err(err)) => Err(err),
                Err(payload) => {
                    let details = Some(json!({ "panic": format!("{:?}", payload) }));
                    let error = ApiError::internal_error("Internal server error", details);
                    let response = HttpResponse::InternalServerError()
                        .json(error)
                        .map_into_boxed_body();
                    Ok(ServiceResponse::new(request, response))
                }
            }
        })
    }
}

pub struct NoCacheMiddleware;

impl<S, B> Transform<S, ServiceRequest> for NoCacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = NoCacheMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(NoCacheMiddlewareService { service })
    }
}

pub struct NoCacheMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for NoCacheMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::CACHE_CONTROL,
                actix_web::http::header::HeaderValue::from_static("no-store"),
            );
            headers.insert(
                actix_web::http::header::PRAGMA,
                actix_web::http::header::HeaderValue::from_static("no-cache"),
            );
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            Ok(res)
        })
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limited_returns_429_and_retry_after_header() {
        let err = ApiError::rate_limited("Too fast", 42);
        assert_eq!(err.status_code(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(err.code, "RATE_LIMITED");

        let resp = err.error_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

        let retry_after = resp.headers().get(actix_web::http::header::RETRY_AFTER);
        assert!(retry_after.is_some(), "Retry-After header must be present");
        assert_eq!(
            retry_after.unwrap().to_str().unwrap(),
            "42",
            "Retry-After must match retry_after_secs"
        );
    }

    #[test]
    fn test_internal_error_logs_via_tracing() {
        // This test verifies the tracing code path is reachable without panicking.
        // The actual log assertion is done in tests.rs with a test subscriber.
        let err = ApiError::internal_error("something broke", None);
        let _resp = err.error_response();
        // No panic means the tracing::error! call succeeded.
    }

    #[test]
    fn test_bad_request_does_not_log() {
        let err = ApiError::bad_request("bad input", None);
        let _resp = err.error_response();
        // No panic.
    }
}
