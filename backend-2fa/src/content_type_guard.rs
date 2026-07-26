//! Content-Type enforcement middleware for JSON endpoints.
//!
//! # Overview
//!
//! [`ContentTypeGuard`] is an actix-web middleware that rejects any request
//! whose `Content-Type` header is not `application/json` (or a compatible
//! sub-type such as `application/json; charset=utf-8`).
//!
//! Without this guard actix-web's default JSON extractor will attempt to
//! deserialise whatever bytes arrive in the body regardless of the declared
//! media type.  A client that accidentally sends
//! `application/x-www-form-urlencoded` receives a confusing 400 parse error
//! instead of a clear protocol-level rejection.
//!
//! # Behaviour
//!
//! | Situation | Result |
//! |---|---|
//! | `Content-Type: application/json` | Forwarded to the inner handler |
//! | `Content-Type: application/json; charset=utf-8` | Forwarded (params ignored) |
//! | `Content-Type: text/plain` | **415 Unsupported Media Type** |
//! | `Content-Type` header absent | **415 Unsupported Media Type** |
//! | Request method has no body (GET / HEAD / OPTIONS) | Forwarded (guard skipped) |
//!
//! # Usage
//!
//! Wrap the scope (or individual routes) that must receive JSON:
//!
//! ```rust,ignore
//! use petchain_2fa::ContentTypeGuard;
//!
//! App::new()
//!     .service(
//!         web::scope("/api")
//!             .wrap(ContentTypeGuard)   // ← enforces Content-Type on every route
//!             .route("/2fa/enable",  web::post().to(enable_2fa))
//!             .route("/2fa/verify",  web::post().to(verify_2fa))
//!             .route("/2fa/disable", web::post().to(disable_2fa))
//!             .route("/2fa/recover", web::post().to(recover_2fa))
//!     )
//! ```
//!
//! Alternatively apply it globally via `App::wrap(ContentTypeGuard)` if every
//! route in your application accepts only JSON.  The guard automatically
//! passes through body-less requests (GET / HEAD / OPTIONS) so you do not need
//! to exclude those routes manually.
//!
//! # Closes
//!
//! Issue #1047 – "Add Content-Type: application/json Enforcement Middleware
//! to 2FA JSON Endpoints".

use actix_web::{
    body::BoxBody,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{Method, StatusCode},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use serde_json::json;

// ── Public API ────────────────────────────────────────────────────────────────

/// Middleware factory.  Register with [`actix_web::App::wrap`] or
/// [`actix_web::web::Scope::wrap`].
///
/// See the [module-level documentation](self) for usage examples.
pub struct ContentTypeGuard;

impl<S, B> Transform<S, ServiceRequest> for ContentTypeGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = ContentTypeGuardService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ContentTypeGuardService { service })
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

/// The per-request service produced by [`ContentTypeGuard`].
pub struct ContentTypeGuardService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ContentTypeGuardService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip enforcement for methods that conventionally carry no request body.
        // This avoids false positives on GET / HEAD / OPTIONS (e.g. CORS preflight).
        let method = req.method().clone();
        if method == Method::GET
            || method == Method::HEAD
            || method == Method::OPTIONS
            || method == Method::DELETE
        {
            let fut = self.service.call(req);
            return Box::pin(async move { Ok(fut.await?.map_into_boxed_body()) });
        }

        // Validate the Content-Type header.
        let is_json = req
            .headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|ct| {
                // Accept "application/json" and "application/json; charset=..."
                // Use case-insensitive prefix matching per RFC 7231 §3.1.1.1.
                ct.trim()
                    .to_ascii_lowercase()
                    .starts_with("application/json")
            })
            .unwrap_or(false);

        if !is_json {
            let http_req = req.request().clone();
            let body = json!({
                "code": "UNSUPPORTED_MEDIA_TYPE",
                "message": "Content-Type must be application/json",
            })
            .to_string();

            let response = HttpResponse::build(StatusCode::UNSUPPORTED_MEDIA_TYPE)
                .content_type("application/json")
                .body(body)
                .map_into_boxed_body();

            return Box::pin(async move { Ok(ServiceResponse::new(http_req, response)) });
        }

        // Content-Type is acceptable — forward to the inner service.
        let fut = self.service.call(req);
        Box::pin(async move { Ok(fut.await?.map_into_boxed_body()) })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::StatusCode, test, web, App, HttpResponse};

    async fn dummy_handler() -> HttpResponse {
        HttpResponse::Ok().json(serde_json::json!({ "ok": true }))
    }

    fn build_app() -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        App::new()
            .wrap(ContentTypeGuard)
            .route("/json-only", web::post().to(dummy_handler))
            .route("/get-route", web::get().to(dummy_handler))
    }

    // ── POST with correct Content-Type → 200 ─────────────────────────────────

    #[actix_web::test]
    async fn test_json_content_type_is_allowed() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "application/json"))
            .set_payload("{}")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── POST with charset parameter → 200 ────────────────────────────────────

    #[actix_web::test]
    async fn test_json_content_type_with_charset_is_allowed() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "application/json; charset=utf-8"))
            .set_payload("{}")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── POST with wrong Content-Type → 415 ───────────────────────────────────

    #[actix_web::test]
    async fn test_form_encoded_content_type_returns_415() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "application/x-www-form-urlencoded"))
            .set_payload("field=value")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // ── POST with text/plain Content-Type → 415 ──────────────────────────────

    #[actix_web::test]
    async fn test_text_plain_content_type_returns_415() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "text/plain"))
            .set_payload("hello")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // ── POST with no Content-Type header → 415 ───────────────────────────────

    #[actix_web::test]
    async fn test_missing_content_type_returns_415() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .set_payload("{}")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    // ── 415 response body is valid JSON with expected error code ─────────────

    #[actix_web::test]
    async fn test_415_response_body_is_json_with_error_code() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "text/plain"))
            .set_payload("oops")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let body = test::read_body(resp).await;
        let parsed: serde_json::Value =
            serde_json::from_slice(&body).expect("415 body must be valid JSON");
        assert_eq!(parsed["code"], "UNSUPPORTED_MEDIA_TYPE");
        assert!(parsed["message"].as_str().unwrap().contains("application/json"));
    }

    // ── GET requests bypass the guard (no body expected) ─────────────────────

    #[actix_web::test]
    async fn test_get_request_bypasses_content_type_guard() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::get()
            .uri("/get-route")
            // No Content-Type header at all.
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── OPTIONS (CORS preflight) bypasses the guard ───────────────────────────

    #[actix_web::test]
    async fn test_options_request_bypasses_content_type_guard() {
        let app = test::init_service(
            App::new()
                .wrap(ContentTypeGuard)
                .route("/json-only", web::method(Method::OPTIONS).to(dummy_handler)),
        )
        .await;
        let req = test::TestRequest::default()
            .method(Method::OPTIONS)
            .uri("/json-only")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ── Case-insensitive Content-Type matching ────────────────────────────────

    #[actix_web::test]
    async fn test_content_type_matching_is_case_insensitive() {
        let app = test::init_service(build_app()).await;
        let req = test::TestRequest::post()
            .uri("/json-only")
            .insert_header(("content-type", "Application/JSON"))
            .set_payload("{}")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
