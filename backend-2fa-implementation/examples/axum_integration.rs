//! Axum integration example for PetChain 2FA.
//! Run with: cargo run --example axum_integration

use axum::{extract::State, routing::post, Json, Router};
use petchain_2fa::handlers::{
    AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest,
    LoginWithTwoFactorRequest, RecoverWithBackupRequest, TwoFactorHandlers,
    VerifyTwoFactorRequest,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    tf: Arc<TwoFactorHandlers>,
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

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    requires_2fa: bool,
    user_id: Option<String>,
    token: Option<String>,
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<serde_json::Value> {
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
                    Ok(true) => Json(serde_json::json!({
                        "success": true, "requires_2fa": false,
                        "user_id": user_id, "token": "generated_jwt_token"
                    })),
                    Ok(false) => Json(serde_json::json!({
                        "success": false, "requires_2fa": true
                    })),
                    Err(e) => Json(serde_json::json!({ "error": e })),
                }
            }
            None => Json(serde_json::json!({
                "success": false, "requires_2fa": true, "user_id": user_id
            })),
        }
    } else {
        Json(serde_json::json!({
            "success": true, "requires_2fa": false,
            "user_id": user_id, "token": "generated_jwt_token"
        }))
    }
}

// ---------------------------------------------------------------------------
// Endpoint 2 – POST /api/2fa/enable
// ---------------------------------------------------------------------------

async fn enable_2fa(Json(req): Json<EnableTwoFactorRequest>) -> Json<serde_json::Value> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::enable_two_factor(&caller, req) {
        Ok(resp) => Json(serde_json::json!({
            "secret": resp.secret,
            "qr_code": resp.qr_code,
            "backup_codes": resp.backup_codes,
        })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 3 – POST /api/2fa/verify
// ---------------------------------------------------------------------------

async fn verify_2fa(
    State(state): State<AppState>,
    Json(req): Json<VerifyTwoFactorRequest>,
) -> Json<serde_json::Value> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.verify_and_activate(&caller, req) {
        Ok(true) => Json(serde_json::json!({ "success": true })),
        Ok(false) => Json(serde_json::json!({ "success": false, "error": "Invalid token" })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 4 – POST /api/2fa/disable
// ---------------------------------------------------------------------------

async fn disable_2fa(
    State(state): State<AppState>,
    Json(req): Json<DisableTwoFactorRequest>,
) -> Json<serde_json::Value> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.disable_two_factor(&caller, req) {
        Ok(true) => Json(serde_json::json!({ "success": true })),
        Ok(false) => Json(serde_json::json!({ "success": false, "error": "Invalid token" })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 5 – POST /api/2fa/recover
// ---------------------------------------------------------------------------

async fn recover_2fa(Json(req): Json<RecoverWithBackupRequest>) -> Json<serde_json::Value> {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::recover_with_backup(&caller, req) {
        Ok(resp) => Json(serde_json::json!({
            "success": true,
            "new_secret": resp.new_secret,
            "new_backup_codes": resp.new_backup_codes,
        })),
        Err(e) => Json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Server bootstrap
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let state = AppState {
        tf: Arc::new(TwoFactorHandlers::new()),
    };

    let app = Router::new()
        .route("/api/auth/login", post(login))
        .route("/api/2fa/enable", post(enable_2fa))
        .route("/api/2fa/verify", post(verify_2fa))
        .route("/api/2fa/disable", post(disable_2fa))
        .route("/api/2fa/recover", post(recover_2fa))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Listening on http://127.0.0.1:8080");
    axum::serve(listener, app).await.unwrap();
}
