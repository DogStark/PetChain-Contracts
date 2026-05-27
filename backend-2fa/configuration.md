# TwoFactorAuth Configuration Guide

## Overview

The TwoFactorAuth implementation now supports configurable TOTP parameters to ensure cryptographic agility and meet varying security requirements.

## Configuration Options

### TotpConfig Structure

```rust
pub struct TotpConfig {
    pub algorithm: Algorithm,  // Hash algorithm
    pub digits: usize,        // Number of digits in token
    pub period: u64,          // Time window in seconds
    pub window: u8,           // Clock skew tolerance
}
```

### Supported Algorithms

- **SHA1**: Legacy support (not recommended for new implementations)
- **SHA256**: Default, recommended for most use cases
- **SHA512**: High security applications

### Predefined Configurations

#### Default Configuration (Recommended)
```rust
TotpConfig::default()
// SHA256, 6 digits, 30s period, 1 window
```

#### Legacy SHA1 Configuration
```rust
TotpConfig::legacy_sha1()
// SHA1, 6 digits, 30s period, 1 window
```

#### High Security Configuration
```rust
TotpConfig::high_security()
// SHA512, 8 digits, 30s period, 1 window
```

## Usage Examples

### Setup with Default Configuration
```rust
let setup = TwoFactorAuth::setup("user@example.com", "MyApp")?;
// Uses SHA256 by default
```

### Setup with Custom Configuration
```rust
let config = TotpConfig::high_security();
let setup = TwoFactorAuth::setup_with_config("user@example.com", "MyApp", config)?;
```

### Token Verification
```rust
// Default verification (SHA256)
let is_valid = TwoFactorAuth::verify_token(&secret, &token)?;

// Custom configuration verification
let config = TotpConfig::legacy_sha1();
let is_valid = TwoFactorAuth::verify_token_with_config(&secret, &token, config)?;
```

## Migration Guide

### From Hard-coded SHA1

**Before:**
```rust
// Hard-coded SHA1, 6 digits, 30s, window 1
let setup = TwoFactorAuth::setup("user@example.com", "MyApp")?;
```

**After (Backward Compatible):**
```rust
// Option 1: Use default (SHA256 - recommended)
let setup = TwoFactorAuth::setup("user@example.com", "MyApp")?;

// Option 2: Explicit SHA1 for backward compatibility
let config = TotpConfig::legacy_sha1();
let setup = TwoFactorAuth::setup_with_config("user@example.com", "MyApp", config)?;
```

### Storing Configuration

When storing 2FA data, include the configuration:

```rust
#[derive(Serialize, Deserialize)]
struct TwoFactorData {
    secret: String,
    backup_codes: Vec<String>,
    enabled: bool,
    config: TotpConfig,  // Store the configuration used
}
```

## Security Considerations

1. **Algorithm Choice**: 
   - Use SHA256 or SHA512 for new implementations
   - SHA1 is provided for legacy compatibility only

2. **Digits**: 
   - 6 digits: Standard, good balance of security and usability
   - 8 digits: Higher security, slightly less user-friendly

3. **Period**: 
   - 30 seconds: Standard TOTP period
   - Shorter periods increase security but reduce usability

4. **Window**: 
   - 1: Minimal clock skew tolerance (recommended)
   - Higher values increase tolerance but reduce security

## Testing

The implementation includes comprehensive tests covering:
- All supported algorithms (SHA1, SHA256, SHA512)
- Different digit configurations (6 and 8 digits)
- Algorithm mismatch scenarios
- Backward compatibility

Run tests with:
```bash
cargo test
```

## Compatibility

- **Backward Compatible**: Existing SHA1 implementations continue to work
- **Forward Compatible**: New configurations can be added without breaking changes
- **Standard Compliant**: Follows RFC 6238 TOTP specification