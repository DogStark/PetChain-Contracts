# PetChain 2FA – Configuration Guide

This document covers runtime configuration for the `petchain-2fa` crate, with a
dedicated section on CORS for teams integrating a browser-based SPA client.

---

## Table of contents

1. [TOTP configuration](#1-totp-configuration)
2. [CORS configuration for browser clients](#2-cors-configuration-for-browser-clients)
3. [Rate limiting](#3-rate-limiting)
4. [Environment variables reference](#4-environment-variables-reference)
5. [Security checklist](#5-security-checklist)

---

## 1. TOTP configuration

The `TotpConfig` struct controls the TOTP algorithm, digit count, time period,
and clock-skew window.

```rust
pub struct TotpConfig {
    pub algorithm: Algorithm, // SHA1 | SHA256 | SHA512
    pub digits: usize,        // 6 (standard) or 8 (high-security)
    pub period: u64,          // seconds per window – default 30
    pub window: u8,           // adjacent windows to accept – default 1
}
```

### Presets

| Preset | Algorithm | Digits | Notes |
|---|---|---|---|
| `TotpConfig::default()` | SHA1 | 6 | RFC 6238 baseline; widest authenticator support |
| `TotpConfig::legacy_sha1()` | SHA1 | 6 | Explicit alias for the default |
| `TotpConfig::high_security()` | SHA512 | 8 | Use for elevated-risk accounts |

### Choosing an algorithm

- **SHA1** is the de-facto default. Every mainstream authenticator app supports it.
- **SHA256 / SHA512** provide stronger HMAC digests but require an authenticator
  that explicitly supports them (e.g. Aegis, andOTP). Verify with your users
  before switching.
- Existing rows without an `algorithm` column are treated as SHA1 until the user
  re-enrolls.

---

## 2. CORS configuration for browser clients

> **Why CORS matters for 2FA endpoints**
>
> A browser SPA that calls `/api/2fa/enable`, `/api/2fa/verify`, or
> `/api/auth/login` directly will trigger a CORS preflight (`OPTIONS`) request.
> If the server does not respond with the correct `Access-Control-*` headers the
> browser blocks the request – the user sees a network error and 2FA never
> completes.
>
> Because these endpoints handle TOTP secrets and backup codes, an overly
> permissive CORS policy is also a security risk. The guidance below explains
> the minimum safe configuration.

### 2.1 Add the dependency

```toml
# Cargo.toml
[dependencies]
actix-cors = { version = "0.7" }
```

### 2.2 Minimal safe setup

```rust
use actix_cors::Cors;
use actix_web::http;

fn build_cors(allowed_origin: &str) -> Cors {
    Cors::default()
        .allowed_origin(allowed_origin)          // explicit origin, never "*"
        .allowed_methods(vec!["POST", "OPTIONS"]) // only what your routes use
        .allowed_headers(vec![
            http::header::CONTENT_TYPE,           // required for JSON bodies
            http::header::AUTHORIZATION,          // JWT bearer token
        ])
        .expose_headers(vec![http::header::CONTENT_TYPE])
        .max_age(3600)
}
```

Register it as the **outermost** middleware in your `App` so that preflight
responses receive CORS headers even when an inner middleware short-circuits:

```rust
App::new()
    .wrap(ErrorResponseMiddleware)
    .wrap(middleware::Logger::default())
    .wrap(build_cors(&allowed_origin)) // outermost – wraps everything inside
    .route("/api/2fa/enable",  web::post().to(enable_2fa))
    .route("/api/2fa/verify",  web::post().to(verify_2fa))
    .route("/api/2fa/disable", web::post().to(disable_2fa))
    .route("/api/2fa/recover", web::post().to(recover_2fa))
    .route("/api/auth/login",  web::post().to(login))
```

### 2.3 Reading the origin from the environment

Hard-coding an origin in source code means every environment (local, staging,
production) needs a separate binary or build flag. Read it from an environment
variable instead:

```rust
let allowed_origin = std::env::var("CORS_ALLOWED_ORIGIN")
    .expect("CORS_ALLOWED_ORIGIN must be set");

let cors = build_cors(&allowed_origin);
```

Typical values:

| Environment | `CORS_ALLOWED_ORIGIN` |
|---|---|
| Local development | `http://localhost:5173` |
| Staging | `https://staging.petchain.example.com` |
| Production | `https://app.petchain.example.com` |

### 2.4 Security considerations

#### ❌ Never combine a wildcard origin with credentials

```rust
// WRONG – browsers reject this; actix-cors panics at runtime
Cors::default()
    .allow_any_origin()
    .supports_credentials()
```

The [Fetch specification](https://fetch.spec.whatwg.org/#cors-protocol-and-credentials)
forbids `Access-Control-Allow-Origin: *` alongside
`Access-Control-Allow-Credentials: true`. Any library that silently allows this
combination is non-compliant.

#### ✅ Use an explicit origin when sending credentials

If your SPA sends cookies or an `Authorization` header you **must** name the
origin explicitly:

```rust
Cors::default()
    .allowed_origin("https://app.petchain.example.com")
    .supports_credentials() // only safe with an explicit origin
```

#### Restrict methods to what your routes actually handle

All 2FA endpoints in this crate use `POST`. Do not add `GET`, `PUT`, `DELETE`,
or `PATCH` unless you have routes for them. A smaller allowed-methods list
reduces the attack surface for cross-site request forgery.

#### Restrict allowed headers

Allow only the headers the SPA actually sends:

| Header | Why it is needed |
|---|---|
| `Content-Type` | JSON request bodies (`application/json`) |
| `Authorization` | JWT bearer token for authenticated endpoints |

Avoid `allow_any_header()` in production – it permits custom headers that could
be used to bypass WAF rules or leak information.

#### `max_age` caching

Setting `max_age(3600)` tells the browser to cache the preflight response for
one hour. This eliminates the extra `OPTIONS` round-trip before every `POST`
and reduces latency for end users. Do not set it to `0` in production.

#### Multiple allowed origins

If you need to support more than one origin (e.g. a web app and a mobile WebView
on a different domain), list them individually rather than using a wildcard:

```rust
Cors::default()
    .allowed_origin("https://app.petchain.example.com")
    .allowed_origin("https://mobile.petchain.example.com")
    .allowed_methods(vec!["POST", "OPTIONS"])
    .allowed_headers(vec![
        http::header::CONTENT_TYPE,
        http::header::AUTHORIZATION,
    ])
    .max_age(3600)
```

Alternatively, use `allowed_origin_fn` for dynamic validation:

```rust
use actix_web::http::header::HeaderValue;

Cors::default()
    .allowed_origin_fn(|origin: &HeaderValue, _req_head| {
        let allowed = ["https://app.petchain.example.com",
                       "https://mobile.petchain.example.com"];
        origin.to_str()
            .map(|o| allowed.contains(&o))
            .unwrap_or(false)
    })
```

> **Note:** `allowed_origin_fn` causes the server to echo back the request's
> `Origin` in `Access-Control-Allow-Origin`. Ensure your list is maintained
> carefully – any origin that passes the predicate is fully trusted.

### 2.5 Local development with a proxy (alternative approach)

If you want to avoid CORS configuration entirely during development, configure
your SPA dev server to proxy API requests to the backend:

**Vite (`vite.config.ts`)**

```ts
export default {
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
};
```

**Create React App (`package.json`)**

```json
{
  "proxy": "http://localhost:8080"
}
```

With a dev proxy, browser requests to `/api/2fa/enable` are forwarded
server-side (no CORS preflight). You still need proper CORS on the backend for
staging and production where the SPA and the API are on different origins.

---

## 3. Rate limiting

The `petchain-2fa` crate ships a `rate_limit_middleware` module. Configure
thresholds via environment variables or directly in the `RateLimitConfig` struct.

Recommended minimums for 2FA endpoints:

| Endpoint | Max requests | Window |
|---|---|---|
| `/api/2fa/enable` | 5 | 1 minute |
| `/api/2fa/verify` | 5 | 1 minute |
| `/api/2fa/disable` | 5 | 1 minute |
| `/api/2fa/recover` | 3 | 1 minute |
| `/api/auth/login` | 10 | 1 minute |

See `src/rate_limiter.rs` and `src/rate_limit_middleware.rs` for implementation
details.

---

## 4. Environment variables reference

| Variable | Required | Example | Description |
|---|---|---|---|
| `CORS_ALLOWED_ORIGIN` | Yes (browser clients) | `https://app.petchain.example.com` | Single explicit origin allowed to call the API from a browser. |
| `DATABASE_URL` | Yes | `postgres://user:pass@localhost/petchain_db` | PostgreSQL connection string. |
| `JWT_SECRET` | Yes | `<random 32-byte hex>` | Secret for signing JWT tokens. |
| `RUST_LOG` | No | `info,petchain_2fa=debug` | Log level filter for `tracing-subscriber`. |
| `RATE_LIMIT_MAX_REQUESTS` | No | `5` | Override default rate-limit threshold. |
| `RATE_LIMIT_WINDOW_SECS` | No | `60` | Override rate-limit window in seconds. |

Copy `.env.example` to `.env` and fill in the values before running locally.

---

## 5. Security checklist

- [ ] `CORS_ALLOWED_ORIGIN` is set to an explicit HTTPS origin in staging and production
- [ ] Wildcard origin (`*`) is **not** used when `Authorization` headers or cookies are present
- [ ] 2FA endpoints are behind HTTPS; the server does not accept plain HTTP in production
- [ ] Rate limiting is enabled on all 2FA endpoints (≤ 5 attempts per minute)
- [ ] TOTP secrets are stored encrypted at rest in the database
- [ ] Backup codes are hashed (not stored in plain text) and invalidated after single use
- [ ] All 2FA events (enable, verify, disable, recover, failed attempts) are logged for audit
- [ ] `JWT_SECRET` is at least 256 bits of entropy and rotated on a schedule
- [ ] `max_age` on the CORS preflight is set to a sensible value (e.g. `3600`)
