//! Actix-web integration example for PetChain 2FA.
//! Run with: cargo run --example example_integration

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use petchain_2fa::handlers::{
    AuthenticatedUser, DisableTwoFactorRequest, EnableTwoFactorRequest,
    LoginWithTwoFactorRequest, RecoverWithBackupRequest, TwoFactorHandlers,
    VerifyTwoFactorRequest,
};
use serde::{Deserialize, Serialize};

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
) -> HttpResponse {
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
                    Ok(true) => HttpResponse::Ok().json(LoginResponse {
                        success: true,
                        requires_2fa: false,
                        user_id: Some(user_id.to_string()),
                        token: Some("generated_jwt_token".to_string()),
                    }),
                    Ok(false) => HttpResponse::Unauthorized().json(LoginResponse {
                        success: false,
                        requires_2fa: true,
                        user_id: None,
                        token: None,
                    }),
                    Err(e) => HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": e })),
                }
            }
            None => HttpResponse::Ok().json(LoginResponse {
                success: false,
                requires_2fa: true,
                user_id: Some(user_id.to_string()),
                token: None,
            }),
        }
    } else {
        HttpResponse::Ok().json(LoginResponse {
            success: true,
            requires_2fa: false,
            user_id: Some(user_id.to_string()),
            token: Some("generated_jwt_token".to_string()),
        })
    }
}

// ---------------------------------------------------------------------------
// Endpoint 2 – POST /api/2fa/enable
// ---------------------------------------------------------------------------

async fn enable_2fa(req: web::Json<EnableTwoFactorRequest>) -> HttpResponse {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::enable_two_factor(&caller, req.into_inner()) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 3 – POST /api/2fa/verify
// ---------------------------------------------------------------------------

async fn verify_2fa(
    state: web::Data<AppState>,
    req: web::Json<VerifyTwoFactorRequest>,
) -> HttpResponse {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.verify_and_activate(&caller, req.into_inner()) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
        Ok(false) => HttpResponse::BadRequest()
            .json(serde_json::json!({ "success": false, "error": "Invalid token" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 4 – POST /api/2fa/disable
// ---------------------------------------------------------------------------

async fn disable_2fa(
    state: web::Data<AppState>,
    req: web::Json<DisableTwoFactorRequest>,
) -> HttpResponse {
    let caller = AuthenticatedUser::new(&req.user_id);
    match state.tf.disable_two_factor(&caller, req.into_inner()) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
        Ok(false) => HttpResponse::BadRequest()
            .json(serde_json::json!({ "success": false, "error": "Invalid token" })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Endpoint 5 – POST /api/2fa/recover
// ---------------------------------------------------------------------------

async fn recover_2fa(req: web::Json<RecoverWithBackupRequest>) -> HttpResponse {
    let caller = AuthenticatedUser::new(&req.user_id);
    match TwoFactorHandlers::recover_with_backup(&caller, req.into_inner()) {
        Ok(response) => HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "new_secret": response.new_secret,
            "new_backup_codes": response.new_backup_codes,
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({ "error": e })),
    }
}

// ---------------------------------------------------------------------------
// Server bootstrap
// ---------------------------------------------------------------------------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState {
        tf: TwoFactorHandlers::new(),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(middleware::Logger::default())
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
