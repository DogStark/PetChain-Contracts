# Quick Start Guide - Deploy 2FA in 5 Steps

## Step 1: Add Dependencies to Backend
Add to your backend's `Cargo.toml`:
```toml
totp-rs = { version = "5.5", features = ["qr", "otpauth"] }
qrcode = "0.14"
base64 = "0.22"
rand = "0.8"
```

## Step 2: Copy Files
```bash
# Copy these files to your backend repo:
cp src/two_factor.rs <backend-repo>/src/
cp src/handlers.rs <backend-repo>/src/
```

## Step 3: Database Setup
Run `schema.sql` on your database:
```sql
CREATE TABLE user_two_factor (
    user_id VARCHAR(255) PRIMARY KEY,
    secret TEXT NOT NULL,
    backup_codes TEXT NOT NULL,
    enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Step 4: Add Routes
See `example_integration.rs` for complete route setup.

## Step 5: Update Login Flow
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

## API Endpoints Summary

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/2fa/enable` | POST | Generate QR code & backup codes |
| `/api/2fa/verify` | POST | Activate 2FA after scanning QR |
| `/api/auth/login` | POST | Login with 2FA token |
| `/api/2fa/disable` | POST | Disable 2FA |
| `/api/2fa/recover` | POST | Use backup code |

## Testing
Users can test with any authenticator app:
- Google Authenticator
- Authy
- Microsoft Authenticator

## Security Checklist
- [ ] Encrypt secrets in database
- [ ] Use HTTPS only
- [ ] Rate limit endpoints (5 attempts/min)
- [ ] Log all 2FA events
- [ ] Invalidate used backup codes
