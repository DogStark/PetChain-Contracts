//! Axum integration example for PetChain 2FA — hardened for production.
//! Run with: cargo run --example axum_integration

use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use petchain_2fa::handlers::{
    AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest,
    LoginWithTwoFactorRequest, RecoverWithBackupRequest, TwoFactorHandlers,
    VerifyTwoFactorRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum accepted request body size (64 KiB).
const MAX_BODY_BYTES: usize = 65_536;

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    tf: Arc<TwoFactorHandlers>,
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ApiError {
    error: String,
    request_id: String,
}

impl ApiError {
    fn new(request_id: &str, msg: impl Into<String>) -> Self {
        Self {
            error: msg.into(),
            request_id: request_id.to_string(),
        }
    }
}

fn err_response(status: StatusCode, request_id: &str, msg: impl Into<String>) -> Response {
    (status, Json(ApiError::new(request_id, msg))).into_response()
}

// ---------------------------------------------------------------------------
// Middleware: inject request ID
// ---------------------------------------------------------------------------

async fn request_id_middleware(mut req: Request, next: Next) -> Response {
    let id = Uuid::new_v4().to_string();
    req.extensions_mut().insert(RequestId(id.clone()));
    let mut resp = next.run(req).await;
    resp.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&id).unwrap_or(HeaderValue::from_static("unknown")),
    );
    resp
}

#[derive(Clone)]
struct RequestId(String);

fn get_request_id(req: &Request) -> String {
    req.extensions()
        .get::<RequestId>()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

// ---------------------------------------------------------------------------
// Middleware: security headers
// ---------------------------------------------------------------------------

async fn security_headers_middleware(req: Request, next: Next) -> Response {
    let mut resp = next.run(req).await;
    let headers = resp.headers_mut();
    headers.insert(
        header::HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    headers.insert(
        header::HeaderName::from_static("strict-transport-security"),
        HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
    );
    resp
}

// ---------------------------------------------------------------------------
// Middleware: content-type check (JSON only)
// ---------------------------------------------------------------------------

async fn require_json_middleware(req: Request, next: Next) -> Response {
    let content_type = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.starts_with("application/json") {
        return (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Json(serde_json::json!({ "error": "Content-Type must be application/json" })),
        )
            .into_response();
    }
    next.run(req).await
}

// ---------------------------------------------------------------------------
// Endpoint 1 – POST /api/auth/login
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    two_factor_token: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    req: Request,
) -> Response {
    let request_id = get_request_id(&req);
    let (parts, body) = req.into_parts();
    let bytes = match axum::body::to_bytes(body, MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => return err_response(StatusCode::PAYLOAD_TOO_LARGE, &request_id, "Request body too large"),
    };
    let payload: LoginRequest = match serde_json::from_slice(&bytes) {
        Ok(p) => p,
        Err(e) => return err_response(StatusCode::UNPROCESSABLE_ENTITY, &request_id, e.to_string()),
    };

    let _ = (&payload.email, &payload.password);
    let user_id = "user123";
    let has_2fa_enabled = true;

    if has_2fa_enabled {
        match &payload.two_factor_token {
            Some(token) => {
                let caller = AuthenticatedUser::new(user_id);
                match state.tf.verify_login_token(
                    &caller,
                    LoginWithTwoFactorRequest {
                        user_id: user_id.to_string(),
                        token: token.clone(),
                    },
                ) {
                    Ok(true) => Json(serde_json::json!({
                        "success": true, "requires_2fa": false,
                        "user_id": user_id, "token": "generated_jwt_token",
                        "request_id": request_id,
                    })).into_response(),
                    Ok(false) => err_response(StatusCode::UNAUTHORIZED, &request_id, "Invalid 2FA token"),
                    Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &request_id, e),
                }
            }
            None => Json(serde_json::json!({
                "success": false, "requires_2fa": true,
                "user_id": user_id, "request_id": request_id,
            })).into_response(),
        }
    } else {
        Json(serde_json::json!({
            "success": true, "requires_2fa": false,
            "user_id": user_id, "token": "generated_jwt_token",
            "request_id": request_id,
        })).into_response()
    }
}

// ---------------------------------------------------------------------------
// Endpoint 2 – POST /api/2fa/enable
// ---------------------------------------------------------------------------

async fn enable_2fa(req: Request) -> Response {
    let request_id = get_request_id(&req);
    let body = match axum::body::to_bytes(req.into_body(), MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => return err_response(StatusCode::PAYLOAD_TOO_LARGE, &request_id, "Request body too large"),
    };
    let payload: EnableTwoFactorRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => return err_response(StatusCode::UNPROCESSABLE_ENTITY, &request_id, e.to_string()),
    };
    let caller = AuthenticatedUser::new(&payload.user_id);
    match TwoFactorHandlers::enable_two_factor(&caller, payload) {
        Ok(resp) => Json(serde_json::json!({
            "secret": resp.secret,
            "qr_code": resp.qr_code,
            "backup_codes": resp.backup_codes,
            "request_id": request_id,
        })).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &request_id, e),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 3 – POST /api/2fa/verify
// ---------------------------------------------------------------------------

async fn verify_2fa(State(state): State<AppState>, req: Request) -> Response {
    let request_id = get_request_id(&req);
    let body = match axum::body::to_bytes(req.into_body(), MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => return err_response(StatusCode::PAYLOAD_TOO_LARGE, &request_id, "Request body too large"),
    };
    let payload: VerifyTwoFactorRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => return err_response(StatusCode::UNPROCESSABLE_ENTITY, &request_id, e.to_string()),
    };
    let caller = AuthenticatedUser::new(&payload.user_id);
    match state.tf.verify_and_activate(&caller, payload) {
        Ok(true) => Json(serde_json::json!({ "success": true, "request_id": request_id })).into_response(),
        Ok(false) => err_response(StatusCode::BAD_REQUEST, &request_id, "Invalid token"),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &request_id, e),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 4 – POST /api/2fa/disable
// ---------------------------------------------------------------------------

async fn disable_2fa(State(state): State<AppState>, req: Request) -> Response {
    let request_id = get_request_id(&req);
    let body = match axum::body::to_bytes(req.into_body(), MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => return err_response(StatusCode::PAYLOAD_TOO_LARGE, &request_id, "Request body too large"),
    };
    let payload: DisableTwoFactorRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => return err_response(StatusCode::UNPROCESSABLE_ENTITY, &request_id, e.to_string()),
    };
    let caller = AuthenticatedUser::new(&payload.user_id);
    match state.tf.disable_two_factor(&caller, payload) {
        Ok(true) => Json(serde_json::json!({ "success": true, "request_id": request_id })).into_response(),
        Ok(false) => err_response(StatusCode::BAD_REQUEST, &request_id, "Invalid token"),
        Err(e) => err_response(StatusCode::INTERNAL_SERVER_ERROR, &request_id, e),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 5 – POST /api/2fa/recover
// ---------------------------------------------------------------------------

async fn recover_2fa(req: Request) -> Response {
    let request_id = get_request_id(&req);
    let body = match axum::body::to_bytes(req.into_body(), MAX_BODY_BYTES).await {
        Ok(b) => b,
        Err(_) => return err_response(StatusCode::PAYLOAD_TOO_LARGE, &request_id, "Request body too large"),
    };
    let payload: RecoverWithBackupRequest = match serde_json::from_slice(&body) {
        Ok(p) => p,
        Err(e) => return err_response(StatusCode::UNPROCESSABLE_ENTITY, &request_id, e.to_string()),
    };
    let caller = AuthenticatedUser::new(&payload.user_id);
    match TwoFactorHandlers::recover_with_backup(&caller, payload) {
        Ok(resp) => Json(serde_json::json!({
            "success": true,
            "new_secret": resp.new_secret,
            "new_backup_codes": resp.new_backup_codes,
            "request_id": request_id,
        })).into_response(),
        Err(e) => err_response(StatusCode::BAD_REQUEST, &request_id, e),
    }
}

// ---------------------------------------------------------------------------
// Server bootstrap
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state = AppState {
        tf: Arc::new(TwoFactorHandlers::new()),
    };

    let app = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/2fa/enable", post(enable_2fa))
        .route("/api/2fa/verify", post(verify_2fa))
        .route("/api/2fa/disable", post(disable_2fa))
        .route("/api/2fa/recover", post(recover_2fa))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(request_id_middleware))
                .layer(middleware::from_fn(security_headers_middleware))
                .layer(middleware::from_fn(require_json_middleware)),
        )
        .layer(axum::extract::DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind("127.0.0.1:8080").await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("Failed to bind: {e}");
            std::process::exit(1);
        }
    };
    tracing::info!("Listening on http://127.0.0.1:8080");
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("Server error: {e}");
        std::process::exit(1);
    }
}
