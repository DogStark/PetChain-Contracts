# Webhook Security Events

This document describes all security event types that can trigger webhook notifications in the backend-2fa service.

## Event Types Overview

Webhook events are emitted when specific security-related actions occur in the 2FA system. Each event includes a standardized payload with event type, user ID, timestamp, and optional metadata.

## Event Type Reference

| Event Type | Trigger Condition | Payload Fields | Currently Emitted |
|------------|-------------------|----------------|-------------------|
| `failed_two_fa` | A TOTP token verification fails (wrong token, expired token, etc.) | `event_type`, `user_id`, `timestamp`, `metadata` (may include `ip_address`, `user_agent`) | ✅ Yes |
| `account_lockout` | Failed 2FA attempts exceed configured threshold (default: 10) | `event_type`, `user_id`, `timestamp`, `metadata` (may include `failed_attempts_count`, `lockout_threshold`, `ip_address`) | ✅ Yes |
| `recovery_code_used` | User successfully authenticates using a backup/recovery code | `event_type`, `user_id`, `timestamp`, `metadata` (may include `ip_address`, `code_index`, `remaining_backup_codes`) | ✅ Yes |
| `canary_triggered` | Authentication attempt made using a known canary/honeypot account | `event_type`, `user_id`, `timestamp`, `metadata` (may include `ip_address`, `user_agent`, `attempted_token`) | ✅ Yes |

## Standard Payload Structure

All webhook events follow this base structure:

```json
{
  "event_type": "string",
  "user_id": "string",
  "timestamp": "number",
  "metadata": {
    "key": "value"
  }
}
```

### Fields

- **event_type** (string): The snake_case identifier for the event type (e.g., `failed_two_fa`)
- **user_id** (string): The user ID associated with the event
- **timestamp** (number): Unix timestamp (seconds since epoch) when the event occurred
- **metadata** (object): Optional key-value pairs with additional context specific to the event type

## Event Details

### failed_two_fa

Emitted when a user fails to provide a valid TOTP token during 2FA verification.

**Use Case**: Alert security teams to potential brute-force attacks or user authentication issues.

**Common Metadata Fields**:
- `ip_address`: IP address of the failed attempt
- `user_agent`: Browser/client identifier
- `attempt_number`: Sequential attempt count for this session

### account_lockout

Emitted when a user account is locked due to excessive failed 2FA attempts.

**Use Case**: Trigger automated incident response workflows when accounts are locked.

**Common Metadata Fields**:
- `failed_attempts_count`: Total number of failed attempts
- `lockout_threshold`: The threshold that was exceeded
- `ip_address`: IP address of the final lockout-triggering attempt

### recovery_code_used

Emitted when a user successfully authenticates using a backup/recovery code.

**Use Case**: Monitor recovery code usage for potential security incidents or user support needs.

**Common Metadata Fields**:
- `ip_address`: IP address of the recovery attempt
- `code_index`: Index of the backup code used (0-based)
- `remaining_backup_codes`: Number of unused backup codes remaining

### canary_triggered

Emitted when a canary/honeypot account is used for authentication attempts.

**Use Case**: Detect credential stuffing attacks or automated reconnaissance by monitoring canary account activity.

**Common Metadata Fields**:
- `ip_address`: IP address of the canary trigger
- `user_agent`: Browser/client identifier
- `attempted_token`: The token that was attempted (if available)

## Webhook Configuration

Webhook URLs can be configured per event type using the `WebhookManager::configure()` method. Multiple URLs can be registered for the same event type, and all registered URLs will receive the event when fired.

Example:
```rust
manager.configure(
    SecurityEventType::FailedTwoFa,
    "https://your-siem.example.com/webhooks/2fa".to_string(),
)?;
```

## Security Considerations

- Webhook payloads are signed with HMAC-SHA256 using the configured signing secret
- The signature is sent in the `X-PetChain-Signature` header
- Consumers should verify the signature to ensure payload authenticity
- Metadata is sanitized to enforce size limits (max 25 entries, 2048 bytes total)

## Rate Limiting and Retries

- Webhook delivery uses exponential backoff for retries (default: 1s, 2s, 4s)
- Maximum retry attempts can be configured via `RetryPolicy`
- Delivery attempts are logged for audit purposes
- Failed deliveries are recorded with error details

## Testing

For testing purposes, use `WebhookManager::new_with_http_allowed()` to allow `http://` URLs, and inject a mock `HttpClient` to avoid real HTTP requests.

## Version History

- **v0.2.0**: Added comprehensive documentation for all event types
- **v0.1.0**: Initial webhook system with four event types
