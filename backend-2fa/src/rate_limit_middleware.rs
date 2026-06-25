//! Rate limit header middleware for actix-web.
//!
//! Wraps every response with `X-RateLimit-Limit`, `X-RateLimit-Remaining`,
//! and `X-RateLimit-Reset` headers taken from a [`RateLimiter`].
//! When the limit is exceeded the middleware short-circuits with
//! `429 Too Many Requests` and adds a `Retry-After` header.

use crate::rate_limiter::RateLimiter;
use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header::HeaderName, header::HeaderValue, StatusCode},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use serde_json::json;
use std::sync::Arc;

pub const HEADER_LIMIT: &str = "x-ratelimit-limit";
pub const HEADER_REMAINING: &str = "x-ratelimit-remaining";
pub const HEADER_RESET: &str = "x-ratelimit-reset";
pub const HEADER_RETRY_AFTER: &str = "retry-after";

// ── Factory ─────────────────────────────────────────────────────────────────

pub struct RateLimitMiddleware<F>
where
    F: Fn(&ServiceRequest) -> String + Clone + 'static,
{
    limiter: Arc<dyn RateLimiter>,
    key_fn: F,
}

impl<F> RateLimitMiddleware<F>
where
    F: Fn(&ServiceRequest) -> String + Clone + 'static,
{
    pub fn new(limiter: Arc<dyn RateLimiter>, key_fn: F) -> Self {
        Self { limiter, key_fn }
    }
}

impl<S, B, F> Transform<S, ServiceRequest> for RateLimitMiddleware<F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
    F: Fn(&ServiceRequest) -> String + Clone + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RateLimitMiddlewareService<S, F>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddlewareService {
            service,
            limiter: Arc::clone(&self.limiter),
            key_fn: self.key_fn.clone(),
        })
    }
}

// ── Service ──────────────────────────────────────────────────────────────────

pub struct RateLimitMiddlewareService<S, F>
where
    F: Fn(&ServiceRequest) -> String + Clone + 'static,
{
    service: S,
    limiter: Arc<dyn RateLimiter>,
    key_fn: F,
}

impl<S, B, F> Service<ServiceRequest> for RateLimitMiddlewareService<S, F>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
    F: Fn(&ServiceRequest) -> String + Clone + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = (self.key_fn)(&req);
        let limiter = Arc::clone(&self.limiter);
        let fut = self.service.call(req);

        Box::pin(async move {
            let result = limiter.record_failure(&key);

            let limit_val = result.limit();
            let remaining_val = result.remaining();
            let reset_val = result.reset_at();

            if result.is_blocked() {
                let retry_after = result.retry_after_secs();

                let body = json!({
                    "code": "RATE_LIMIT_EXCEEDED",
                    "message": "Too many requests",
                    "retry_after_secs": retry_after,
                })
                .to_string();

                let http_req = actix_web::test::TestRequest::default().to_http_request();
                let mut response = HttpResponse::build(StatusCode::TOO_MANY_REQUESTS)
                    .content_type("application/json")
                    .body(body);

                insert_rate_limit_headers(
                    response.headers_mut(),
                    limit_val,
                    remaining_val,
                    reset_val,
                );
                insert_header(
                    response.headers_mut(),
                    HEADER_RETRY_AFTER,
                    &retry_after.to_string(),
                );

                return Ok(ServiceResponse::new(http_req, response));
            }

            // Allowed — forward to the inner handler.
            let mut res = fut.await?.map_into_boxed_body();
            insert_rate_limit_headers(res.headers_mut(), limit_val, remaining_val, reset_val);
            Ok(res)
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn insert_header(headers: &mut actix_web::http::header::HeaderMap, name: &str, value: &str) {
    if let (Ok(hname), Ok(hval)) = (
        HeaderName::from_bytes(name.as_bytes()),
        HeaderValue::from_str(value),
    ) {
        headers.insert(hname, hval);
    }
}

fn insert_rate_limit_headers(
    headers: &mut actix_web::http::header::HeaderMap,
    limit: u32,
    remaining: u32,
    reset_at: u64,
) {
    insert_header(headers, HEADER_LIMIT, &limit.to_string());
    insert_header(headers, HEADER_REMAINING, &remaining.to_string());
    insert_header(headers, HEADER_RESET, &reset_at.to_string());
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rate_limiter::InMemoryRateLimiter;
    use actix_web::{test, web, App, HttpResponse};
    use std::sync::Arc;

    async fn ok_handler() -> HttpResponse {
        HttpResponse::Ok().body("ok")
    }

    /// Test 1: Allowed request — response includes X-RateLimit-Limit,
    /// X-RateLimit-Remaining, X-RateLimit-Reset headers.
    #[actix_web::test]
    async fn test_allowed_request_includes_rate_limit_headers() {
        let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(5, 60, 300));

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(Arc::clone(&limiter), |_req| {
                    "test-user".to_string()
                }))
                .route("/test", web::get().to(ok_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let headers = resp.headers();
        assert!(
            headers.contains_key(HEADER_LIMIT),
            "X-RateLimit-Limit header missing"
        );
        assert!(
            headers.contains_key(HEADER_REMAINING),
            "X-RateLimit-Remaining header missing"
        );
        assert!(
            headers.contains_key(HEADER_RESET),
            "X-RateLimit-Reset header missing"
        );
    }

    /// Test 2: Rate-limited request — status is 429 and Retry-After header exists.
    #[actix_web::test]
    async fn test_rate_limited_request_returns_429_with_retry_after() {
        // max_failures = 0 means even the first call exceeds the limit immediately.
        let limiter: Arc<dyn RateLimiter> = Arc::new(InMemoryRateLimiter::new(0, 60, 300));

        let app = test::init_service(
            App::new()
                .wrap(RateLimitMiddleware::new(Arc::clone(&limiter), |_req| {
                    "blocked-user".to_string()
                }))
                .route("/test", web::get().to(ok_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(
            resp.status(),
            StatusCode::TOO_MANY_REQUESTS,
            "Expected 429 status for rate-limited request"
        );

        let headers = resp.headers();
        assert!(
            headers.contains_key(HEADER_RETRY_AFTER),
            "Retry-After header missing on 429 response"
        );
    }
}
