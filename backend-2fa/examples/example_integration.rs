//! Actix-web integration example for PetChain 2FA.
//! Run with: cargo run --example example_integration
//!
//! # CORS for Browser-Based SPA Clients
//!
//! When a browser SPA calls these 2FA endpoints directly it triggers CORS preflight
//! requests. The server must respond with the correct `Access-Control-*` headers or
//! the browser will block the request entirely.
//!
//! ## Key rules
//!
//! 1. **Never use a wildcard origin (`*`) when credentials are involved.**
//!    Cookies and `Authorization` headers are credentials. Mixing `allow_any_origin()`
//!    with `supports_credentials()` is rejected by every modern browser and by the
//!    `actix-cors` crate itself at runtime.
//!
//! 2. **List origins explicitly.** Hard-code `https://your-spa.example.com` (or read
//!    from an environment variable). Do not derive the allowed origin from the
//!    incoming `Origin` header – that defeats the purpose of CORS entirely.
//!
//! 3. **Restrict to the methods your routes actually use.**
//!    All 2FA endpoints here use `POST`. Add `OPTIONS` so preflight succeeds.
//!    Do not add `DELETE`, `PUT`, etc. unless you have routes for them.
//!
//! 4. **Restrict headers to what the client sends.**
//!    At minimum: `Content-Type` (JSON body) and `Authorization` (JWT bearer).
//!    Avoid `allow_any_header()` in production.
//!
//! 5. **Set `max_age`.** Caches the preflight response in the browser so your SPA
//!    does not send an `OPTIONS` request before every single API call.
//!
//! See `backend-2fa/configuration.md` for full security considerations and
//! environment-specific configuration guidance.

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpResponse, HttpServer};
use petchain_2fa::handlers::{
    AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest, LoginWithTwoFactorRequest,
    RecoverWithBackupRequest, TwoFactorHandlers, VerifyTwoFactorRequest,
};
use petchain_2fa::{ApiError, ErrorResponseMiddleware};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ── Issue #884: Maximum accepted request body size (256 KiB) ────────────────
const MAX_JSON_PAYLOAD_SIZE: usize = 256 * 1024;

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

struct AppState {
    tf: TwoFactorHandlers,
}

// ---------------------------------------------------------------------------
// Request / response types for the login endpoint
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    two_factor_token: Option<String>,
}

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    requires_2fa: bool,
    user_id: Option<String>,
    token: Option<String>,
}

// ---------------------------------------------------------------------------
// Endpoint 1 – POST /api/auth/login
// ---------------------------------------------------------------------------

async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    // Placeholder: validate email/password against your database.
    let _ = (&req.email, &req.password);
    let user_id = "user123"; // replace with real DB lookup
    let has_2fa_enabled = true; // replace with user.two_factor_enabled from DB

    if has_2fa_enabled {
        match &req.two_factor_token {
            Some(token) => {
                let caller = AuthenticatedUser::new(user_id);
                match state.tf.verify_login_token(
                    &caller,
                    LoginWithTwoFactorRequest {
                        user_id: user_id.to_string(),
                        token: token.clone(),
                    },
                ) {
                    Ok(true) => Ok(HttpResponse::Ok().json(LoginResponse {
                        success: true,
                        requires_2fa: false,
                        user_id: Some(user_id.to_string()),
                        token: Some("generated_jwt_token".to_string()),
                    })),
                    Ok(false) => Err(ApiError::unauthorized(
                        "Invalid two-factor authentication token",
                        None,
                    )),
                    Err(e) => Err(ApiError::internal_error(
                        "2FA verification failed",
                        Some(json!({ "error": e })),
                    )),
                }
            }
            None => Ok(HttpResponse::Ok().json(LoginResponse {
                success: false,
                requires_2fa: true,
                user_id: Some(user_id.to_string()),
                token: None,
            })),
        }
    } else {
        Ok(HttpResponse::Ok().json(LoginResponse {
            success: true,
            requires_2fa: false,
            user_id: Some(user_id.to_string()),
            token: Some("generated_jwt_token".to_string()),
        }))
    }
}

// ---------------------------------------------------------------------------
// Endpoint 2 – POST /api/2fa/enable
// ---------------------------------------------------------------------------

async fn enable_2fa(req: web::Json<EnableTwoFactorRequest>) -> Result<HttpResponse, ApiError> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::enable_two_factor(&caller, req.into_inner()) {
        Ok(response) => Ok(HttpResponse::Ok().json(response)),
        Err(e) => Err(ApiError::bad_request(
            "Failed to enable two-factor authentication",
            Some(json!({ "error": e })),
        )),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 3 – POST /api/2fa/verify
// ---------------------------------------------------------------------------

async fn verify_2fa(
    state: web::Data<AppState>,
    req: web::Json<VerifyTwoFactorRequest>,
) -> Result<HttpResponse, ApiError> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.verify_and_activate(&caller, req.into_inner()) {
        Ok(true) => Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true }))),
        Ok(false) => Err(ApiError::invalid_token("Invalid token", None)),
        Err(e) => Err(ApiError::internal_error(
            "2FA verification failed",
            Some(json!({ "error": e })),
        )),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 4 – POST /api/2fa/disable
// ---------------------------------------------------------------------------

async fn disable_2fa(
    state: web::Data<AppState>,
    req: web::Json<DisableTwoFactorRequest>,
) -> Result<HttpResponse, ApiError> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.disable_two_factor(&caller, req.into_inner()) {
        Ok(true) => Ok(HttpResponse::Ok().json(serde_json::json!({ "success": true }))),
        Ok(false) => Err(ApiError::invalid_token("Invalid token", None)),
        Err(e) => Err(ApiError::internal_error(
            "2FA disable failed",
            Some(json!({ "error": e })),
        )),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 5 – POST /api/2fa/recover
// ---------------------------------------------------------------------------

async fn recover_2fa(req: web::Json<RecoverWithBackupRequest>) -> Result<HttpResponse, ApiError> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::recover_with_backup(&caller, req.into_inner()) {
        Ok(response) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "new_secret": response.new_secret,
            "new_backup_codes": response.new_backup_codes,
        }))),
        Err(e) => Err(ApiError::bad_request(
            "Failed to recover two-factor authentication",
            Some(json!({ "error": e })),
        )),
    }
}

// ---------------------------------------------------------------------------
// CORS configuration helper
// ---------------------------------------------------------------------------

/// Build a CORS middleware instance scoped to what the 2FA endpoints need.
///
/// # Security contract
///
/// - `allowed_origin` **must** be an explicit origin (scheme + host + optional port).
///   Never pass `"*"` here when credentials (`Authorization` / cookies) are in play.
/// - Only `POST` and `OPTIONS` are allowed – these are the only methods used by any
///   2FA endpoint in this example. Expand the list only when you add new routes.
/// - Only the headers the SPA actually sends are allowed. `Content-Type` is required
///   for JSON bodies; `Authorization` carries the JWT bearer token.
/// - `max_age(3600)` tells the browser to cache the preflight response for one hour,
///   avoiding a redundant `OPTIONS` round-trip before every API call.
///
/// # Environment-specific usage
///
/// In production read the origin from an environment variable so you never
/// accidentally ship a hard-coded development URL:
///
/// ```rust
/// let origin = std::env::var("CORS_ALLOWED_ORIGIN")
///     .unwrap_or_else(|_| "https://app.petchain.example.com".to_string());
/// let cors = build_cors(&origin);
/// ```
///
/// For local development you might set `CORS_ALLOWED_ORIGIN=http://localhost:5173`.
/// See `backend-2fa/configuration.md` for the full guidance.
fn build_cors(allowed_origin: &str) -> Cors {
    Cors::default()
        // ✅ Explicit origin – required when credentials are involved.
        //    Never use .allow_any_origin() together with .supports_credentials().
        .allowed_origin(allowed_origin)
        // Only the methods the 2FA routes actually use.
        .allowed_methods(vec!["POST", "OPTIONS"])
        // Headers the SPA sends with every authenticated request.
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,
            http::header::AUTHORIZATION,
        ])
        // Allow the browser to read the response body (needed for JSON APIs).
        .expose_headers(vec![http::header::CONTENT_TYPE])
        // Cache the preflight for 1 hour – reduces OPTIONS round-trips.
        .max_age(3600)
}

// ---------------------------------------------------------------------------
// Server bootstrap
// ---------------------------------------------------------------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        tf: TwoFactorHandlers::new(),
    });

    // Read the allowed origin from the environment so the binary never
    // contains a hard-coded production URL.
    // Set CORS_ALLOWED_ORIGIN=http://localhost:5173 for local dev.
    let allowed_origin = std::env::var("CORS_ALLOWED_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:5173".to_string());

    HttpServer::new(move || {
        // IMPORTANT: .wrap() applies middleware in reverse registration order.
        // Cors must be registered BEFORE (i.e., wrapped AFTER) the logger so
        // that preflight OPTIONS requests receive CORS headers even when the
        // logger or error middleware short-circuits the response.
        let cors = build_cors(&allowed_origin);
        // ── Issue #884: explicit JSON body size limit ───────────────────────
        let json_cfg = web::JsonConfig::default()
            .limit(MAX_JSON_PAYLOAD_SIZE)
            .error_handler(|err, _req| {
                let resp = ApiError::bad_request(
                    format!("Request body too large or invalid JSON: {}", err),
                    None,
                );
                actix_web::Error::from(resp)
            });

        App::new()
            .app_data(state.clone())
            .app_data(json_cfg)
            .wrap(ErrorResponseMiddleware)
            .wrap(middleware::Logger::default())
            // Cors wraps outermost so every response – including preflight –
            // gets the Access-Control-* headers.
            .wrap(cors)
            .route("/api/auth/login", web::post().to(login))
            .route("/api/2fa/enable", web::post().to(enable_2fa))
            .route("/api/2fa/verify", web::post().to(verify_2fa))
            .route("/api/2fa/disable", web::post().to(disable_2fa))
            .route("/api/2fa/recover", web::post().to(recover_2fa))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
