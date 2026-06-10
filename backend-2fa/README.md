# PetChain 2FA

Configurable TOTP-based Two-Factor Authentication for PetChain backend with cryptographic agility.

## Features

- Configurable TOTP parameters (SHA1 / SHA256 / SHA512, 6 or 8 digits, custom periods)
- QR code generation for authenticator apps
- 8 backup codes with recovery mechanism
- Enable / disable / verify endpoints
- Rate limiting support
- Postgres-backed storage with migration scripts

## Quick Start

### 1. Add dependencies

```toml
[dependencies]
totp-rs = { version = "5.7.1", features = ["qr", "otpauth", "gen_secret"] }
rand = "0.8"
subtle = "2.6"
```

### 2. Copy source files

```bash
cp src/two_factor.rs  <your-backend>/src/
cp src/handlers.rs    <your-backend>/src/
```

### 3. Set up the database

```bash
psql -U postgres -d petchain_db -f schema.sql
```

Or run the numbered migrations in `migrations/` in order.

### 4. Add routes

See [`examples/example_integration.rs`](examples/example_integration.rs) for a complete Axum setup.

### 5. Update your login flow

```rust
// After password validation:
if user.two_factor_enabled {
    if let Some(token) = request.two_factor_token {
        verify_2fa_token(user.id, token)?;
    } else {
        return "2FA required";
    }
}
```

## API Endpoints

| Endpoint | Method | Purpose |
|---|---|---|
| `/api/2fa/enable` | POST | Generate QR code & backup codes |
| `/api/2fa/verify` | POST | Activate 2FA after scanning QR |
| `/api/auth/login` | POST | Login with 2FA token |
| `/api/2fa/disable` | POST | Disable 2FA |
| `/api/2fa/recover` | POST | Use a backup code |

### Enable 2FA

```
POST /api/2fa/enable
{ "user_id": "user123", "email": "user@example.com" }

→ { "secret": "JBSWY3...", "qr_code": "data:image/png;base64,...", "backup_codes": [...] }
```

### Verify & Activate

```
POST /api/2fa/verify
{ "user_id": "user123", "token": "123456" }

→ { "success": true }
```

### Login with 2FA

```
POST /api/auth/login/2fa
{ "user_id": "user123", "token": "123456" }

→ { "success": true, "auth_token": "jwt_token_here" }
```

### Disable 2FA

```
POST /api/2fa/disable
{ "user_id": "user123", "token": "123456" }

→ { "success": true }
```

### Recover with Backup Code

```
POST /api/2fa/recover
{ "user_id": "user123", "backup_code": "1234-5678" }

→ { "success": true, "auth_token": "jwt_token_here" }
```

## Configuration

### TotpConfig

```rust
pub struct TotpConfig {
    pub algorithm: Algorithm,  // Hash algorithm
    pub digits: usize,         // 6 or 8
    pub period: u64,           // Seconds per window (default 30)
    pub window: u8,            // Clock-skew tolerance (default 1)
}
```

### Predefined presets

```rust
TotpConfig::default()       // SHA1,   6 digits — legacy-compatible default
TotpConfig::legacy_sha1()   // SHA1,   6 digits — explicit legacy
TotpConfig::high_security() // SHA512, 8 digits
```

### Usage

```rust
// Default setup (SHA1, backward-compatible)
let setup = TwoFactorAuth::setup("user@example.com", "PetChain")?;

// High-security setup
let config = TotpConfig::high_security();
let setup = TwoFactorAuth::setup_with_config("user@example.com", "PetChain", config)?;

// Custom configuration
let config = TotpConfig { algorithm: Algorithm::SHA256, digits: 6, period: 30, window: 1 };

// Verification
let ok = TwoFactorAuth::verify_token(&secret, &token)?;
let ok = TwoFactorAuth::verify_token_with_config(&secret, &token, config)?;
```

### Migration from hard-coded SHA1

Existing rows without an `algorithm` column are treated as SHA1 until the user
re-enrolls. New enrollments persist the selected algorithm so SHA256/SHA512
users always verify with the hash they enrolled with.

## Security Checklist

- [ ] Store secrets encrypted in the database
- [ ] Use HTTPS only
- [ ] Rate-limit 2FA endpoints (≤ 5 attempts per minute)
- [ ] Log all 2FA events for audit
- [ ] Invalidate backup codes after single use

## Database Integration

Replace the placeholder store in `handlers.rs` with real database calls:

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

## Testing

```bash
cd backend-2fa
cargo test
```

Compatible authenticator apps: Google Authenticator, Authy, Microsoft Authenticator, any RFC 6238 TOTP app.

## License

MIT
