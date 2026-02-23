// Example integration with Actix-web backend
// Copy this to your backend's main.rs or routes module

use actix_web::{web, App, HttpResponse, HttpServer, middleware};
use serde::{Deserialize, Serialize};

// Import your 2FA modules
mod two_factor;
mod handlers;
use handlers::*;

// Example: Modified login endpoint with 2FA support
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

async fn login(req: web::Json<LoginRequest>) -> HttpResponse {
    // 1. Validate username/password (your existing logic)
    // let user = db.authenticate(&req.email, &req.password)?;
    
    let user_id = "user123"; // From DB
    let has_2fa_enabled = true; // From DB: user.two_factor_enabled
    
    // 2. Check if 2FA is enabled
    if has_2fa_enabled {
        if let Some(token) = &req.two_factor_token {
            // Verify 2FA token
            match TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
                user_id: user_id.to_string(),
                token: token.clone(),
            }) {
                Ok(true) => {
                    // Generate JWT token
                    let jwt = "generated_jwt_token"; // Your JWT logic
                    return HttpResponse::Ok().json(LoginResponse {
                        success: true,
                        requires_2fa: false,
                        user_id: Some(user_id.to_string()),
                        token: Some(jwt.to_string()),
                    });
                }
                _ => {
                    return HttpResponse::Unauthorized().json(LoginResponse {
                        success: false,
                        requires_2fa: true,
                        user_id: None,
                        token: None,
                    });
                }
            }
        } else {
            // Request 2FA token
            return HttpResponse::Ok().json(LoginResponse {
                success: false,
                requires_2fa: true,
                user_id: Some(user_id.to_string()),
                token: None,
            });
        }
    }
    
    // No 2FA required, proceed with normal login
    let jwt = "generated_jwt_token";
    HttpResponse::Ok().json(LoginResponse {
        success: true,
        requires_2fa: false,
        user_id: Some(user_id.to_string()),
        token: Some(jwt.to_string()),
    })
}

// 2FA Setup endpoint
async fn enable_2fa(req: web::Json<EnableTwoFactorRequest>) -> HttpResponse {
    match TwoFactorHandlers::enable_two_factor(req.into_inner()) {
        Ok(response) => {
            // Store in database before returning
            // db.save_two_factor_setup(&response)?;
            HttpResponse::Ok().json(response)
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

// Verify and activate 2FA
async fn verify_2fa(req: web::Json<VerifyTwoFactorRequest>) -> HttpResponse {
    match TwoFactorHandlers::verify_and_activate(req.into_inner()) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Ok(false) => HttpResponse::BadRequest().json(serde_json::json!({"success": false, "error": "Invalid token"})),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// Disable 2FA
async fn disable_2fa(req: web::Json<DisableTwoFactorRequest>) -> HttpResponse {
    match TwoFactorHandlers::disable_two_factor(req.into_inner()) {
        Ok(true) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Ok(false) => HttpResponse::BadRequest().json(serde_json::json!({"success": false, "error": "Invalid token"})),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

// Recovery with backup code
async fn recover_2fa(req: web::Json<RecoverWithBackupRequest>) -> HttpResponse {
    match TwoFactorHandlers::recover_with_backup(req.into_inner()) {
        Ok(true) => {
            let jwt = "generated_jwt_token"; // Your JWT logic
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "token": jwt
            }))
        }
        Ok(false) => HttpResponse::BadRequest().json(serde_json::json!({"success": false, "error": "Invalid backup code"})),
        Err(e) => HttpResponse::InternalServerError().body(e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            // Auth routes
            .route("/api/auth/login", web::post().to(login))
            // 2FA routes
            .route("/api/2fa/enable", web::post().to(enable_2fa))
            .route("/api/2fa/verify", web::post().to(verify_2fa))
            .route("/api/2fa/disable", web::post().to(disable_2fa))
            .route("/api/2fa/recover", web::post().to(recover_2fa))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
