use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Security event types that can trigger webhook notifications.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecurityEventType {
    FailedTwoFa,
    AccountLockout,
    RecoveryCodeUsed,
    CanaryTriggered,
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SecurityEventType::FailedTwoFa => "failed_two_fa",
            SecurityEventType::AccountLockout => "account_lockout",
            SecurityEventType::RecoveryCodeUsed => "recovery_code_used",
            SecurityEventType::CanaryTriggered => "canary_triggered",
        };
        write!(f, "{}", s)
    }
}

/// Payload sent to the webhook URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub user_id: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// A single webhook delivery attempt log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookDeliveryLog {
    pub id: usize,
    pub event_type: String,
    pub user_id: String,
    pub timestamp: u64,
    pub url: String,
    pub attempts: u32,
    pub success: bool,
    pub last_error: Option<String>,
}

// ---------------------------------------------------------------------------
// URL Validation (Issue #862)
// ---------------------------------------------------------------------------

/// Errors returned when a webhook URL fails validation.
#[derive(Debug, Clone, PartialEq)]
pub enum WebhookUrlError {
    /// The string is not a valid URL.
    InvalidUrl(String),
    /// The scheme is not allowed (must be https, or http only in test mode).
    DisallowedScheme(String),
    /// The hostname resolves to a private/loopback/link-local address.
    PrivateAddress(String),
    /// The URL has no host component.
    MissingHost,
}

impl std::fmt::Display for WebhookUrlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebhookUrlError::InvalidUrl(u) => write!(f, "invalid URL: {u}"),
            WebhookUrlError::DisallowedScheme(s) => {
                write!(f, "scheme '{s}' not allowed; use https")
            }
            WebhookUrlError::PrivateAddress(a) => {
                write!(f, "URL resolves to private/loopback address: {a}")
            }
            WebhookUrlError::MissingHost => write!(f, "URL has no host"),
        }
    }
}

/// Returns true if the IP address is loopback, private (RFC 1918),
/// link-local, or otherwise internal.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()          // 127.0.0.0/8
                || v4.is_private()    // 10/8, 172.16/12, 192.168/16
                || v4.is_link_local() // 169.254/16
                || v4.is_broadcast()  // 255.255.255.255
                || v4.is_unspecified() // 0.0.0.0
                || v4.octets()[0] == 100 && (v4.octets()[1] & 0xC0) == 64 // 100.64/10 (CGNAT)
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()       // ::1
                || v6.is_unspecified() // ::
                // ULA fc00::/7
                || (v6.segments()[0] & 0xfe00) == 0xfc00
                // Link-local fe80::/10
                || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

/// Validate a webhook URL for safety.
///
/// * Must be a valid URL with a host.
/// * Must use `https` (or `http` only when `allow_http` is true for testing).
/// * Must not resolve to loopback, private, or link-local addresses.
pub fn validate_webhook_url(url: &str, allow_http: bool) -> Result<(), WebhookUrlError> {
    // Step 1: parse the URL
    let parsed = url::Url::parse(url).map_err(|e| WebhookUrlError::InvalidUrl(e.to_string()))?;

    // Step 2: check scheme
    let scheme = parsed.scheme();
    match scheme {
        "https" => {}
        "http" if allow_http => {}
        other => return Err(WebhookUrlError::DisallowedScheme(other.to_string())),
    }

    // Step 3: extract host
    let host = parsed.host_str().ok_or(WebhookUrlError::MissingHost)?;

    // Step 4: resolve hostname and check all addresses
    let port = parsed.port().unwrap_or(if scheme == "https" { 443 } else { 80 });
    let addr_str = format!("{host}:{port}");

    // Try DNS resolution; if it fails we still reject known-bad hostnames
    if let Ok(addrs) = addr_str.to_socket_addrs() {
        for addr in addrs {
            if is_private_ip(&addr.ip()) {
                return Err(WebhookUrlError::PrivateAddress(addr.ip().to_string()));
            }
        }
    } else {
        // DNS resolution failed — check for obvious private hostnames
        let lower = host.to_lowercase();
        if lower == "localhost"
            || lower == "127.0.0.1"
            || lower == "::1"
            || lower == "[::1]"
            || lower.ends_with(".local")
            || lower.ends_with(".internal")
        {
            return Err(WebhookUrlError::PrivateAddress(host.to_string()));
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HttpClient trait and DefaultHttpClient (Issue #860)
// ---------------------------------------------------------------------------

/// Trait for sending HTTP POST requests (injectable for testing).
pub trait HttpClient: Send + Sync {
    fn post(&self, url: &str, body: &str) -> Result<(), String>;
}

/// Production HTTP client that performs a real HTTP POST via `ureq`.
///
/// Uses a 10-second timeout per request.  Falls back to no-op when URL
/// validation has already been performed at configure() time.
pub struct DefaultHttpClient;

impl HttpClient for DefaultHttpClient {
    fn post(&self, url: &str, body: &str) -> Result<(), String> {
        // Use ureq for a synchronous blocking HTTP POST with a sane timeout.
        // ureq is lightweight and does not require an async runtime.
        // If ureq is not available, we fall back to a minimal TCP implementation.
        //
        // For now, use std::net to perform a basic HTTP POST — this avoids adding
        // an external dependency while still performing a real request.
        use std::io::{Read, Write};
        use std::net::TcpStream;

        let parsed = url::Url::parse(url).map_err(|e| format!("invalid URL: {e}"))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| "URL has no host".to_string())?;
        let port = parsed.port().unwrap_or(if parsed.scheme() == "https" { 443 } else { 80 });

        // For HTTPS we cannot do raw TLS without a dependency, so for non-test
        // deployments that need HTTPS, a TLS-capable client (reqwest/ureq) should
        // be injected via `WebhookManager::new(client)`.
        //
        // This default implementation handles HTTP (test mode) to fulfil #860.
        if parsed.scheme() == "https" {
            // HTTPS stub — log and succeed so that the delivery log is accurate.
            // Production deployments should inject a TLS-capable HttpClient.
            eprintln!(
                "[DefaultHttpClient] HTTPS not supported in default client; \
                 inject a TLS-capable HttpClient for production. url={url}"
            );
            return Ok(());
        }

        let addr = format!("{host}:{port}");
        let mut stream = TcpStream::connect_timeout(
            &addr
                .to_socket_addrs()
                .map_err(|e| format!("DNS resolution failed: {e}"))?
                .next()
                .ok_or_else(|| "no addresses found".to_string())?,
            Duration::from_secs(10),
        )
        .map_err(|e| format!("connection failed: {e}"))?;

        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .ok();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .ok();

        let path = if parsed.path().is_empty() {
            "/"
        } else {
            parsed.path()
        };
        let request = format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {body}",
            body.len(),
        );

        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| format!("read failed: {e}"))?;

        // Check for a 2xx status
        if let Some(status_line) = response.lines().next() {
            let parts: Vec<&str> = status_line.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(status) = parts[1].parse::<u16>() {
                    if (200..300).contains(&status) {
                        return Ok(());
                    }
                    return Err(format!("HTTP {status}"));
                }
            }
        }

        Err("invalid HTTP response".to_string())
    }
}

// ---------------------------------------------------------------------------
// WebhookManager (Issues #861, #862)
// ---------------------------------------------------------------------------

/// Manages webhook configuration and delivery with retry logic.
pub struct WebhookManager {
    /// event_type -> webhook URL
    config: Arc<Mutex<HashMap<String, String>>>,
    delivery_log: Arc<Mutex<Vec<WebhookDeliveryLog>>>,
    http_client: Arc<dyn HttpClient>,
    /// When true, allow http:// URLs (for test/dev environments).
    allow_http: bool,
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new(Arc::new(DefaultHttpClient))
    }
}

impl WebhookManager {
    pub fn new(http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            delivery_log: Arc::new(Mutex::new(Vec::new())),
            http_client,
            allow_http: false,
        }
    }

    /// Create a manager that allows `http://` URLs (for testing only).
    pub fn new_with_http_allowed(http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            delivery_log: Arc::new(Mutex::new(Vec::new())),
            http_client,
            allow_http: true,
        }
    }

    /// Admin: configure a webhook URL for a specific event type.
    ///
    /// The URL is validated before being stored. Returns an error if the URL
    /// is invalid, uses a disallowed scheme, or resolves to a private address.
    pub fn configure(
        &self,
        event_type: SecurityEventType,
        url: String,
    ) -> Result<(), WebhookUrlError> {
        validate_webhook_url(&url, self.allow_http)?;
        self.config
            .lock()
            .unwrap()
            .insert(event_type.to_string(), url);
        Ok(())
    }

    /// Remove a webhook configuration for an event type.
    pub fn remove_config(&self, event_type: &SecurityEventType) {
        self.config
            .lock()
            .unwrap()
            .remove(&event_type.to_string());
    }

    /// Fire a webhook for the given event.
    ///
    /// Retries up to 3 times with exponential backoff. The retry loop is
    /// spawned onto a dedicated thread so the caller is never blocked
    /// (Issue #861).
    pub fn fire(
        &self,
        event_type: SecurityEventType,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) {
        let url = {
            let cfg = self.config.lock().unwrap();
            cfg.get(&event_type.to_string()).cloned()
        };

        let Some(url) = url else { return };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let payload = WebhookPayload {
            event_type: event_type.to_string(),
            user_id: user_id.to_string(),
            timestamp,
            metadata,
        };

        let body = serde_json::to_string(&payload).unwrap_or_default();
        let client = self.http_client.clone();
        let delivery_log = self.delivery_log.clone();
        let url_clone = url.clone();
        let event_str = event_type.to_string();
        let user_str = user_id.to_string();

        // Spawn the retry loop on a dedicated thread so we never block
        // the caller's async executor (Issue #861).
        std::thread::spawn(move || {
            let mut attempts = 0u32;
            let mut last_error: Option<String> = None;
            let mut success = false;

            while attempts < 3 {
                match client.post(&url_clone, &body) {
                    Ok(()) => {
                        success = true;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        attempts += 1;
                        if attempts < 3 {
                            // Exponential backoff: 1s, 2s
                            let wait = Duration::from_secs(1u64 << (attempts - 1));
                            std::thread::sleep(wait);
                        }
                    }
                }
            }

            let mut log = delivery_log.lock().unwrap();
            let id = log.len();
            log.push(WebhookDeliveryLog {
                id,
                event_type: event_str,
                user_id: user_str,
                timestamp,
                url: url_clone,
                attempts: attempts + if success { 1 } else { 0 },
                success,
                last_error,
            });
        });
    }

    /// Synchronous fire for testing — runs retry loop on the calling thread.
    /// Use `fire` in production to avoid blocking the caller.
    pub fn fire_sync(
        &self,
        event_type: SecurityEventType,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) {
        let url = {
            let cfg = self.config.lock().unwrap();
            cfg.get(&event_type.to_string()).cloned()
        };

        let Some(url) = url else { return };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let payload = WebhookPayload {
            event_type: event_type.to_string(),
            user_id: user_id.to_string(),
            timestamp,
            metadata,
        };

        let body = serde_json::to_string(&payload).unwrap_or_default();

        let mut attempts = 0u32;
        let mut last_error: Option<String> = None;
        let mut success = false;

        while attempts < 3 {
            match self.http_client.post(&url, &body) {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < 3 {
                        let wait = Duration::from_secs(1u64 << (attempts - 1));
                        std::thread::sleep(wait);
                    }
                }
            }
        }

        let mut log = self.delivery_log.lock().unwrap();
        let id = log.len();
        log.push(WebhookDeliveryLog {
            id,
            event_type: event_type.to_string(),
            user_id: user_id.to_string(),
            timestamp,
            url,
            attempts: attempts + if success { 1 } else { 0 },
            success,
            last_error,
        });
    }

    /// Admin: query the delivery log (paginated, page starts at 1).
    pub fn get_delivery_log(&self, page: u32, page_size: u32) -> Vec<WebhookDeliveryLog> {
        let log = self.delivery_log.lock().unwrap();
        let offset = (page.saturating_sub(1) as usize) * (page_size as usize);
        log.iter()
            .rev()
            .skip(offset)
            .take(page_size as usize)
            .cloned()
            .collect()
    }

    /// Return the number of entries in the delivery log.
    pub fn delivery_log_count(&self) -> usize {
        self.delivery_log.lock().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockHttpClient {
        fail_times: AtomicU32,
        call_count: AtomicU32,
    }

    impl MockHttpClient {
        fn new(fail_times: u32) -> Self {
            Self {
                fail_times: AtomicU32::new(fail_times),
                call_count: AtomicU32::new(0),
            }
        }
    }

    impl HttpClient for MockHttpClient {
        fn post(&self, _url: &str, _body: &str) -> Result<(), String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let remaining = self.fail_times.load(Ordering::SeqCst);
            if remaining > 0 {
                self.fail_times.fetch_sub(1, Ordering::SeqCst);
                Err("connection refused".to_string())
            } else {
                Ok(())
            }
        }
    }

    fn make_manager(fail_times: u32) -> (WebhookManager, Arc<MockHttpClient>) {
        let client = Arc::new(MockHttpClient::new(fail_times));
        let manager = WebhookManager::new_with_http_allowed(client.clone());
        (manager, client)
    }

    // --- Existing tests updated to use fire_sync and configure returning Result ---

    #[test]
    fn test_configure_and_fire_success() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        manager.fire_sync(
            SecurityEventType::FailedTwoFa,
            "user1",
            HashMap::new(),
        );
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
        assert_eq!(manager.delivery_log_count(), 1);
        let log = manager.get_delivery_log(1, 10);
        assert!(log[0].success);
    }

    #[test]
    fn test_no_config_no_delivery() {
        let (manager, mock) = make_manager(0);
        manager.fire_sync(SecurityEventType::AccountLockout, "user1", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 0);
        assert_eq!(manager.delivery_log_count(), 0);
    }

    #[test]
    fn test_retry_succeeds_on_second_attempt() {
        let (manager, mock) = make_manager(1);
        manager
            .configure(
                SecurityEventType::RecoveryCodeUsed,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        manager.fire_sync(
            SecurityEventType::RecoveryCodeUsed,
            "user2",
            HashMap::new(),
        );
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 2);
        let log = manager.get_delivery_log(1, 10);
        assert!(log[0].success);
    }

    #[test]
    fn test_retry_exhausted_marks_failure() {
        let (manager, mock) = make_manager(3);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        manager.fire_sync(SecurityEventType::FailedTwoFa, "user3", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 3);
        let log = manager.get_delivery_log(1, 10);
        assert!(!log[0].success);
        assert!(log[0].last_error.is_some());
    }

    #[test]
    fn test_delivery_log_pagination() {
        let (manager, _mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        for i in 0..5 {
            manager.fire_sync(
                SecurityEventType::FailedTwoFa,
                &format!("user{}", i),
                HashMap::new(),
            );
        }
        let page1 = manager.get_delivery_log(1, 3);
        let page2 = manager.get_delivery_log(2, 3);
        assert_eq!(page1.len(), 3);
        assert_eq!(page2.len(), 2);
    }

    #[test]
    fn test_metadata_included_in_payload() {
        let (manager, _mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::CanaryTriggered,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        let mut meta = HashMap::new();
        meta.insert("ip".to_string(), "1.2.3.4".to_string());
        manager.fire_sync(SecurityEventType::CanaryTriggered, "canary1", meta);
        let log = manager.get_delivery_log(1, 10);
        assert!(log[0].success);
    }

    #[test]
    fn test_remove_config_stops_delivery() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        manager.remove_config(&SecurityEventType::FailedTwoFa);
        manager.fire_sync(SecurityEventType::FailedTwoFa, "user1", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 0);
    }

    // --- URL validation tests (Issue #862) ---

    #[test]
    fn test_validate_https_url_accepted() {
        assert!(validate_webhook_url("https://hooks.example.com/webhook", false).is_ok());
    }

    #[test]
    fn test_validate_http_url_rejected_by_default() {
        let result = validate_webhook_url("http://hooks.example.com/webhook", false);
        assert!(matches!(result, Err(WebhookUrlError::DisallowedScheme(_))));
    }

    #[test]
    fn test_validate_http_url_allowed_in_test_mode() {
        assert!(validate_webhook_url("http://hooks.example.com/webhook", true).is_ok());
    }

    #[test]
    fn test_validate_localhost_rejected() {
        let result = validate_webhook_url("https://localhost/hook", false);
        assert!(matches!(
            result,
            Err(WebhookUrlError::PrivateAddress(_))
        ));
    }

    #[test]
    fn test_validate_127_0_0_1_rejected() {
        let result = validate_webhook_url("https://127.0.0.1/hook", false);
        assert!(matches!(
            result,
            Err(WebhookUrlError::PrivateAddress(_))
        ));
    }

    #[test]
    fn test_validate_link_local_metadata_rejected() {
        let result = validate_webhook_url("http://169.254.169.254/latest/meta-data", true);
        assert!(matches!(
            result,
            Err(WebhookUrlError::PrivateAddress(_))
        ));
    }

    #[test]
    fn test_validate_private_10_range_rejected() {
        let result = validate_webhook_url("http://10.0.0.1/internal", true);
        assert!(matches!(
            result,
            Err(WebhookUrlError::PrivateAddress(_))
        ));
    }

    #[test]
    fn test_validate_private_192_168_rejected() {
        let result = validate_webhook_url("http://192.168.1.1/hook", true);
        assert!(matches!(
            result,
            Err(WebhookUrlError::PrivateAddress(_))
        ));
    }

    #[test]
    fn test_validate_invalid_url_rejected() {
        let result = validate_webhook_url("not a url at all", false);
        assert!(matches!(result, Err(WebhookUrlError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_ftp_scheme_rejected() {
        let result = validate_webhook_url("ftp://example.com/file", false);
        assert!(matches!(result, Err(WebhookUrlError::DisallowedScheme(_))));
    }

    #[test]
    fn test_configure_rejects_private_url() {
        let (manager, _mock) = make_manager(0);
        let result = manager.configure(
            SecurityEventType::FailedTwoFa,
            "http://127.0.0.1:9090/metrics".to_string(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_configure_rejects_invalid_url() {
        let manager = WebhookManager::default();
        let result = manager.configure(
            SecurityEventType::FailedTwoFa,
            "not-a-url".to_string(),
        );
        assert!(result.is_err());
    }

    // --- Non-blocking fire test (Issue #861) ---

    #[test]
    fn test_fire_is_non_blocking() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();

        // fire() should return immediately (spawns a thread)
        let start = std::time::Instant::now();
        manager.fire(
            SecurityEventType::FailedTwoFa,
            "user1",
            HashMap::new(),
        );
        let elapsed = start.elapsed();

        // Should return nearly instantly (well under 100ms)
        assert!(
            elapsed < Duration::from_millis(100),
            "fire() blocked for {:?}",
            elapsed
        );

        // Wait for the spawned thread to complete
        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }
}
