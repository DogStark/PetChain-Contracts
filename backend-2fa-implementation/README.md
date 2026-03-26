# PetChain 2FA Implementation

Configurable TOTP-based Two-Factor Authentication for PetChain backend with cryptographic agility.

## Features

✅ **Configurable TOTP parameters** (SHA1/SHA256/SHA512, 6/8 digits, custom periods)  
✅ **Cryptographic agility** - Future-proof algorithm support  
✅ QR code generation for authenticator apps  
✅ 8 backup codes generation  
✅ 2FA enable/disable endpoints  
✅ Verify 2FA token on login  
✅ Recovery mechanism with backup codes  
✅ **Backward compatibility** with existing SHA1 implementations  

## Configuration Options

### Default Configuration (Recommended)
- **Algorithm**: SHA256 (secure, modern)
- **Digits**: 6 (standard)
- **Period**: 30 seconds (standard)
- **Window**: 1 (minimal clock skew tolerance)

### Available Configurations
```rust
// Default (SHA256)
let setup = TwoFactorAuth::setup("user@example.com", "PetChain")?;

// Legacy SHA1 (backward compatibility)
let config = TotpConfig::legacy_sha1();
let setup = TwoFactorAuth::setup_with_config("user@example.com", "PetChain", config)?;

// High Security (SHA512, 8 digits)
let config = TotpConfig::high_security();
let setup = TwoFactorAuth::setup_with_config("user@example.com", "PetChain", config)?;

// Custom configuration
let config = TotpConfig {
    algorithm: Algorithm::SHA256,
    digits: 6,
    period: 30,
    window: 1,
};
```

## Migration from Hard-coded SHA1

**✅ Backward Compatible**: Existing SHA1 implementations continue to work  
**✅ Gradual Migration**: Migrate users to SHA256 over time  
**✅ Configuration Storage**: Store TOTP config with user data  

See [CONFIGURATION.md](CONFIGURATION.md) for detailed migration guide.  

## API Endpoints

### 1. Enable 2FA (Generate QR & Backup Codes)
```
POST /api/2fa/enable
Content-Type: application/json

{
  "user_id": "user123",
  "email": "user@example.com"
}

Response:
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code": "data:image/png;base64,iVBORw0KG...",
  "backup_codes": [
    "1234-5678",
    "2345-6789",
    ...
  ]
}
```

### 2. Verify & Activate 2FA
```
POST /api/2fa/verify
Content-Type: application/json

{
  "user_id": "user123",
  "token": "123456"
}

Response:
{
  "success": true
}
```

### 3. Login with 2FA
```
POST /api/auth/login/2fa
Content-Type: application/json

{
  "user_id": "user123",
  "token": "123456"
}

Response:
{
  "success": true,
  "auth_token": "jwt_token_here"
}
```

### 4. Disable 2FA
```
POST /api/2fa/disable
Content-Type: application/json

{
  "user_id": "user123",
  "token": "123456"
}

Response:
{
  "success": true
}
```

### 5. Recover with Backup Code
```
POST /api/2fa/recover
Content-Type: application/json

{
  "user_id": "user123",
  "backup_code": "1234-5678"
}

Response:
{
  "success": true,
  "auth_token": "jwt_token_here"
}
```

## Integration Steps

### 1. Add to your backend's Cargo.toml
```toml
[dependencies]
totp-rs = { version = "5.7.1", features = ["qr", "otpauth", "gen_secret"] }
rand = "0.8"
subtle = "2.6"
```

### 2. Copy files to your backend
```
cp src/two_factor.rs <your-backend>/src/
cp src/handlers.rs <your-backend>/src/
```

### 3. Setup database
```bash
# Run schema.sql on your database
mysql -u root -p petchain_db < schema.sql
# or
psql -U postgres -d petchain_db -f schema.sql
```

### 4. Add routes to your web framework

**Actix-web example:**
```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use petchain_2fa::handlers::*;

async fn enable_2fa(req: web::Json<EnableTwoFactorRequest>) -> HttpResponse {
    match TwoFactorHandlers::enable_two_factor(req.into_inner()) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/2fa/enable", web::post().to(enable_2fa))
            // Add other routes...
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

**Axum example:**


```rust
use axum::{
    routing::post,
    Json, Router,
    http::StatusCode,
    response::IntoResponse,
};
use petchain_2fa::handlers::*;

async fn enable_2fa(
    Json(req): Json<EnableTwoFactorRequest>,
) -> impl IntoResponse {
    match TwoFactorHandlers::enable_two_factor(req) {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Failed to enable 2FA: {}", e),
        ),
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/2fa/enable", post(enable_2fa));
    
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### 5. Update login flow
```rust
// After username/password validation:
if user.two_factor_enabled {
    // Return 2FA required response
    return HttpResponse::Ok().json({
        "requires_2fa": true,
        "user_id": user.id
    });
}

// On 2FA token submission:
let is_valid = TwoFactorHandlers::verify_login_token(LoginWithTwoFactorRequest {
    user_id: user.id,
    token: provided_token,
})?;

if is_valid {
    // Generate JWT and return
}
```

## Database Integration

Replace placeholders in `handlers.rs` with your database calls:

```rust
// Example with sqlx (PostgreSQL)
let two_factor_data: TwoFactorData = sqlx::query_as!(
    TwoFactorData,
    "SELECT secret, backup_codes, enabled FROM user_two_factor WHERE user_id = $1",
    user_id
)
.fetch_one(&pool)
.await?;
```

## Security Notes

- Store secrets encrypted in database
- Use HTTPS only
- Rate limit 2FA endpoints (max 5 attempts per minute)
- Invalidate backup codes after use
- Log all 2FA events for audit

## Testing

```bash
cd backend-2fa-implementation
cargo test
```

## Mobile App Integration

Users scan QR code with:
- Google Authenticator
- Authy
- Microsoft Authenticator
- Any TOTP-compatible app

## License
MIT
