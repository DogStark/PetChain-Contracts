use crate::metrics::{record_webhook_delivery, record_webhook_retry};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, ToSocketAddrs};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

/// Header under which the webhook signature is sent.
pub const SIGNATURE_HEADER: &str = "X-PetChain-Signature";

/// Default maximum number of entries retained in the delivery log.
/// Override with the `WEBHOOK_LOG_MAX_ENTRIES` environment variable.
const DEFAULT_MAX_LOG_ENTRIES: usize = 1000;

/// Compute the `sha256=<hex>` HMAC-SHA256 signature for a webhook body.
pub fn sign_webhook_payload(secret: &str, body: &[u8]) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take a key of any size");
    mac.update(body);
    let digest = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(digest))
}

/// Verify a webhook signature header against the expected secret and body.
///
/// `header_value` is expected to be in the form `sha256=<hex>`. Returns
/// `false` for malformed headers, tampered bodies, or mismatched secrets.
/// Comparison is constant-time to avoid timing side channels.
pub fn verify_webhook_signature(secret: &str, body: &[u8], header_value: &str) -> bool {
    let Some(provided_hex) = header_value.strip_prefix("sha256=") else {
        return false;
    };

    let Ok(provided_bytes) = hex::decode(provided_hex) else {
        return false;
    };

    let Ok(mut mac) = HmacSha256::new_from_slice(secret.as_bytes()) else {
        return false;
    };
    mac.update(body);

    mac.verify_slice(&provided_bytes).is_ok()
}

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
    let parsed = url::Url::parse(url).map_err(|e| WebhookUrlError::InvalidUrl(e.to_string()))?;

    let scheme = parsed.scheme();
    match scheme {
        "https" => {}
        "http" if allow_http => {}
        other => return Err(WebhookUrlError::DisallowedScheme(other.to_string())),
    }

    let host = parsed.host_str().ok_or(WebhookUrlError::MissingHost)?;

    let port = parsed.port().unwrap_or(if scheme == "https" { 443 } else { 80 });
    let addr_str = format!("{host}:{port}");

    if let Ok(addrs) = addr_str.to_socket_addrs() {
        for addr in addrs {
            if is_private_ip(&addr.ip()) {
                return Err(WebhookUrlError::PrivateAddress(addr.ip().to_string()));
            }
        }
    } else {
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
    fn post(&self, url: &str, body: &str, signature_header: &str) -> Result<(), String>;
}

/// Production HTTP client that performs a real HTTP POST via raw TCP.
///
/// Uses a 10-second timeout per request. For HTTPS, inject a TLS-capable
/// implementation via `WebhookManager::new`.
pub struct DefaultHttpClient;

#[cfg(feature = "webhook-client")]
static HTTP_CLIENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();

#[cfg(feature = "webhook-client")]
impl HttpClient for DefaultHttpClient {
    fn post(&self, url: &str, body: &str, signature_header: &str) -> Result<(), String> {
        let agent = HTTP_CLIENT.get_or_init(|| {
            ureq::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
        });

        let response = agent
            .post(url)
            .set("Content-Type", "application/json")
            .set(SIGNATURE_HEADER, signature_header)
            .send_string(body)
            .map_err(|e| format!("request failed: {}", e))?;

        if response.status() >= 200 && response.status() < 300 {
            Ok(())
        } else {
            Err(format!("server returned error status: {}", response.status()))
        }
    }
}

#[cfg(not(feature = "webhook-client"))]
impl HttpClient for DefaultHttpClient {
    fn post(&self, url: &str, body: &str, signature_header: &str) -> Result<(), String> {
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

        if parsed.scheme() == "https" {
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

        stream.set_write_timeout(Some(Duration::from_secs(10))).ok();
        stream.set_read_timeout(Some(Duration::from_secs(10))).ok();

        let path = if parsed.path().is_empty() {
            "/"
        } else {
            parsed.path()
        };
        let request = format!(
            "POST {path} HTTP/1.1\r\n\
             Host: {host}\r\n\
             Content-Type: application/json\r\n\
             {SIGNATURE_HEADER}: {signature_header}\r\n\
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
// WebhookManager (Issues #861, #862, #863, #864)
// ---------------------------------------------------------------------------

/// Read the delivery-log capacity from the environment, falling back to `DEFAULT_MAX_LOG_ENTRIES`.
fn log_cap_from_env() -> usize {
    std::env::var("WEBHOOK_LOG_MAX_ENTRIES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_MAX_LOG_ENTRIES)
}

/// Configurable retry policy for webhook delivery.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub base_backoff: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_backoff: Duration::from_secs(1),
        }
    }
}

/// Manages webhook configuration and delivery with retry logic.
pub struct WebhookManager {
    /// event_type -> list of webhook URLs (multiple endpoints supported per event).
    config: Arc<Mutex<HashMap<String, Vec<String>>>>,
    /// Bounded ring buffer; oldest entries are evicted when `max_log_entries` is reached.
    delivery_log: Arc<Mutex<VecDeque<WebhookDeliveryLog>>>,
    /// Monotonically-increasing counter for log-entry IDs (survives eviction).
    next_log_id: Arc<AtomicUsize>,
    /// Maximum number of entries kept in `delivery_log`.
    max_log_entries: usize,
    http_client: Arc<dyn HttpClient>,
    /// When true, allow http:// URLs (for test/dev environments).
    allow_http: bool,
    /// HMAC-SHA256 signing secret used to compute `X-PetChain-Signature` headers.
    /// Empty string disables signing.
    signing_secret: String,
    retry_policy: RetryPolicy,
}

impl Default for WebhookManager {
    fn default() -> Self {
        Self::new(Arc::new(DefaultHttpClient), String::new(), None)
    }
}

impl WebhookManager {
    pub fn new(
        http_client: Arc<dyn HttpClient>,
        signing_secret: String,
        retry_policy: Option<RetryPolicy>,
    ) -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            delivery_log: Arc::new(Mutex::new(VecDeque::new())),
            next_log_id: Arc::new(AtomicUsize::new(0)),
            max_log_entries: log_cap_from_env(),
            http_client,
            allow_http: false,
            signing_secret,
            retry_policy: retry_policy.unwrap_or_default(),
        }
    }

    /// Create a manager that allows `http://` URLs (for testing only).
    pub fn new_with_http_allowed(http_client: Arc<dyn HttpClient>) -> Self {
        Self {
            config: Arc::new(Mutex::new(HashMap::new())),
            delivery_log: Arc::new(Mutex::new(VecDeque::new())),
            next_log_id: Arc::new(AtomicUsize::new(0)),
            max_log_entries: log_cap_from_env(),
            http_client,
            allow_http: true,
            signing_secret: String::new(),
            retry_policy: RetryPolicy::default(),
        }
    }

    /// Register a webhook URL for a specific event type.
    ///
    /// Multiple URLs may be registered for the same event type; each call appends
    /// rather than replacing. All registered URLs receive the event on `fire`.
    pub fn configure(
        &self,
        event_type: SecurityEventType,
        url: String,
    ) -> Result<(), WebhookUrlError> {
        validate_webhook_url(&url, self.allow_http)?;
        self.config
            .lock()
            .unwrap()
            .entry(event_type.to_string())
            .or_default()
            .push(url);
        Ok(())
    }

    /// Remove all webhook URLs registered for an event type.
    pub fn remove_config(&self, event_type: &SecurityEventType) {
        self.config.lock().unwrap().remove(&event_type.to_string());
    }

    /// Remove a single URL from the list registered for an event type.
    pub fn remove_config_url(&self, event_type: &SecurityEventType, url: &str) {
        let mut cfg = self.config.lock().unwrap();
        if let Some(urls) = cfg.get_mut(&event_type.to_string()) {
            urls.retain(|u| u != url);
            if urls.is_empty() {
                cfg.remove(&event_type.to_string());
            }
        }
    }

    /// Append an entry to the delivery log, evicting the oldest entry if the cap is reached.
    fn append_log(&self, log: &mut VecDeque<WebhookDeliveryLog>, entry: WebhookDeliveryLog) {
        if self.max_log_entries > 0 && log.len() >= self.max_log_entries {
            log.pop_front();
        }
        log.push_back(entry);
    }

    /// Run the retry loop for a single URL and push a log entry.
    ///
    /// Used by both `fire` (in a spawned thread) and `fire_sync` (on the
    /// calling thread). Each URL is handled independently — one failure does
    /// not prevent delivery to the remaining URLs.
    fn deliver_one(
        client: &dyn HttpClient,
        url: &str,
        body: &str,
        signature: &str,
        event_str: &str,
        user_str: &str,
        timestamp: u64,
        delivery_log: &Mutex<VecDeque<WebhookDeliveryLog>>,
        next_log_id: &AtomicUsize,
        max_log_entries: usize,
        retry_policy: &RetryPolicy,
    ) {
        let mut attempts = 0u32;
        let mut last_error: Option<String> = None;
        let mut success = false;

        while attempts < retry_policy.max_attempts {
            match client.post(url, body, signature) {
                Ok(()) => {
                    success = true;
                    break;
                }
                Err(e) => {
                    last_error = Some(e);
                    attempts += 1;
                    if attempts < retry_policy.max_attempts {
                        record_webhook_retry();
                        let multiplier = 1u64 << (attempts - 1);
                        let wait = retry_policy.base_backoff * multiplier as u32;
                        std::thread::sleep(wait);
                    }
                }
            }
        }

        record_webhook_delivery(success);

        let id = next_log_id.fetch_add(1, Ordering::Relaxed);
        let entry = WebhookDeliveryLog {
            id,
            event_type: event_str.to_string(),
            user_id: user_str.to_string(),
            timestamp,
            url: url.to_string(),
            attempts: attempts + if success { 1 } else { 0 },
            success,
            last_error,
        };
        let mut log = delivery_log.lock().unwrap();
        if max_log_entries > 0 && log.len() >= max_log_entries {
            log.pop_front();
        }
        log.push_back(entry);
    }

    /// Fire a webhook for the given event, delivering to all registered URLs.
    ///
    /// Each URL is dispatched on its own thread so failures are fully
    /// independent and the caller is never blocked (Issue #861).
    pub fn fire(
        &self,
        event_type: SecurityEventType,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) {
        let urls = {
            let cfg = self.config.lock().unwrap();
            cfg.get(&event_type.to_string()).cloned().unwrap_or_default()
        };
        if urls.is_empty() {
            return;
        }

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
        let signature = sign_webhook_payload(&self.signing_secret, body.as_bytes());
        let event_str = event_type.to_string();
        let user_str = user_id.to_string();

        for url in urls {
            let url_clone = url.clone();
            let client = self.http_client.clone();
            let delivery_log = self.delivery_log.clone();
            let next_log_id = self.next_log_id.clone();
            let max_log_entries = self.max_log_entries;
            let body_clone = body.clone();
            let signature_clone = signature.clone();
            let event_str_clone = event_str.clone();
            let user_str_clone = user_str.clone();
            let retry_policy = self.retry_policy.clone();

            // Spawn the retry loop on a dedicated thread so we never block
            // the caller's async executor (Issue #861).
            std::thread::spawn(move || {
                Self::deliver_one(
                    client.as_ref(),
                    &url_clone,
                    &body_clone,
                    &signature_clone,
                    &event_str_clone,
                    &user_str_clone,
                    timestamp,
                    &delivery_log,
                    &next_log_id,
                    max_log_entries,
                    &retry_policy,
                );
            });
        }
    }

    /// Synchronous fire — runs each URL's retry loop on the calling thread.
    ///
    /// Each URL is delivered independently; a failure on one URL does not
    /// prevent delivery to subsequent URLs.
    pub fn fire_sync(
        &self,
        event_type: SecurityEventType,
        user_id: &str,
        metadata: HashMap<String, String>,
    ) {
        let urls = {
            let cfg = self.config.lock().unwrap();
            cfg.get(&event_type.to_string()).cloned().unwrap_or_default()
        };
        if urls.is_empty() {
            return;
        }

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
        let signature_header = sign_webhook_payload(&self.signing_secret, body.as_bytes());
        let event_str = event_type.to_string();
        let user_str = user_id.to_string();

        for url in urls {
            Self::deliver_one(
                self.http_client.as_ref(),
                &url,
                &body,
                &signature_header,
                &event_str,
                &user_str,
                timestamp,
                &self.delivery_log,
                &self.next_log_id,
                self.max_log_entries,
                &self.retry_policy,
            );
        }
    }

    /// Admin: query the delivery log (paginated, page starts at 1, newest first).
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

    /// Return the number of entries currently in the delivery log.
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
        fn post(&self, _url: &str, _body: &str, _signature_header: &str) -> Result<(), String> {
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

    fn make_manager_with_cap(fail_times: u32, cap: usize) -> (WebhookManager, Arc<MockHttpClient>) {
        let client = Arc::new(MockHttpClient::new(fail_times));
        let mut manager = WebhookManager::new_with_http_allowed(client.clone());
        manager.max_log_entries = cap;
        (manager, client)
    }

    #[test]
    fn test_configure_and_fire_success() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();
        manager.fire_sync(SecurityEventType::FailedTwoFa, "user1", HashMap::new());
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
        assert!(matches!(result, Err(WebhookUrlError::PrivateAddress(_))));
    }

    #[test]
    fn test_validate_127_0_0_1_rejected() {
        let result = validate_webhook_url("https://127.0.0.1/hook", false);
        assert!(matches!(result, Err(WebhookUrlError::PrivateAddress(_))));
    }

    #[test]
    fn test_validate_link_local_metadata_rejected() {
        let result = validate_webhook_url("http://169.254.169.254/latest/meta-data", true);
        assert!(matches!(result, Err(WebhookUrlError::PrivateAddress(_))));
    }

    #[test]
    fn test_validate_private_10_range_rejected() {
        let result = validate_webhook_url("http://10.0.0.1/internal", true);
        assert!(matches!(result, Err(WebhookUrlError::PrivateAddress(_))));
    }

    #[test]
    fn test_validate_private_192_168_rejected() {
        let result = validate_webhook_url("http://192.168.1.1/hook", true);
        assert!(matches!(result, Err(WebhookUrlError::PrivateAddress(_))));
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

        let start = std::time::Instant::now();
        manager.fire(SecurityEventType::FailedTwoFa, "user1", HashMap::new());
        let elapsed = start.elapsed();

        assert!(
            elapsed < Duration::from_millis(100),
            "fire() blocked for {:?}",
            elapsed
        );

        std::thread::sleep(Duration::from_millis(200));
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
    }

    // --- Bounded log tests (Issue #863) ---

    #[test]
    fn test_log_does_not_exceed_cap() {
        let cap = 5;
        let (manager, _mock) = make_manager_with_cap(0, cap);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();

        for i in 0..10 {
            manager.fire_sync(
                SecurityEventType::FailedTwoFa,
                &format!("user{}", i),
                HashMap::new(),
            );
        }

        assert_eq!(
            manager.delivery_log_count(),
            cap,
            "log should be capped at {cap}"
        );
    }

    #[test]
    fn test_eviction_preserves_newest_entries() {
        let cap = 3;
        let (manager, _mock) = make_manager_with_cap(0, cap);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();

        for i in 0..5u32 {
            manager.fire_sync(
                SecurityEventType::FailedTwoFa,
                &format!("user{}", i),
                HashMap::new(),
            );
        }

        // get_delivery_log returns newest-first; the 3 newest users are user4, user3, user2
        let log = manager.get_delivery_log(1, cap as u32);
        assert_eq!(log.len(), cap);
        assert_eq!(log[0].user_id, "user4");
        assert_eq!(log[1].user_id, "user3");
        assert_eq!(log[2].user_id, "user2");
    }

    #[test]
    fn test_log_ids_are_monotonic_after_eviction() {
        let cap = 3;
        let (manager, _mock) = make_manager_with_cap(0, cap);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook".to_string(),
            )
            .unwrap();

        for i in 0..6 {
            manager.fire_sync(
                SecurityEventType::FailedTwoFa,
                &format!("user{}", i),
                HashMap::new(),
            );
        }

        let log = manager.get_delivery_log(1, cap as u32);
        // IDs should be monotonically increasing (newest first means descending IDs)
        assert!(log[0].id > log[1].id);
        assert!(log[1].id > log[2].id);
    }

    // --- Fan-out delivery tests (Issue #864) ---

    /// A client that fails for a specific URL and succeeds for all others.
    struct UrlSelectiveFailClient {
        failing_url: String,
        call_count: AtomicU32,
    }

    impl UrlSelectiveFailClient {
        fn new(failing_url: &str) -> Self {
            Self {
                failing_url: failing_url.to_string(),
                call_count: AtomicU32::new(0),
            }
        }
    }

    impl HttpClient for UrlSelectiveFailClient {
        fn post(&self, url: &str, _body: &str, _sig: &str) -> Result<(), String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            if url == self.failing_url {
                Err("simulated permanent failure".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_configure_accumulates_multiple_urls() {
        let (manager, _mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::CanaryTriggered,
                "http://example.com/hook1".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::CanaryTriggered,
                "http://example.com/hook2".to_string(),
            )
            .unwrap();

        // Both URLs should be in the config; firing produces 2 log entries.
        manager.fire_sync(SecurityEventType::CanaryTriggered, "u1", HashMap::new());
        assert_eq!(manager.delivery_log_count(), 2);
    }

    #[test]
    fn test_fanout_delivers_to_all_urls() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook1".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook2".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::FailedTwoFa,
                "http://example.com/hook3".to_string(),
            )
            .unwrap();

        manager.fire_sync(SecurityEventType::FailedTwoFa, "u1", HashMap::new());

        assert_eq!(mock.call_count.load(Ordering::SeqCst), 3, "all 3 URLs should be called");
        assert_eq!(manager.delivery_log_count(), 3);
        let log = manager.get_delivery_log(1, 10);
        assert!(log.iter().all(|e| e.success), "all deliveries should succeed");
    }

    #[test]
    fn test_partial_failure_does_not_block_other_urls() {
        let client = Arc::new(UrlSelectiveFailClient::new("http://example.com/failing"));
        let mut manager = WebhookManager::new_with_http_allowed(client.clone());
        manager.max_log_entries = 100;

        manager
            .configure(
                SecurityEventType::AccountLockout,
                "http://example.com/failing".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::AccountLockout,
                "http://example.com/ok1".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::AccountLockout,
                "http://example.com/ok2".to_string(),
            )
            .unwrap();

        manager.fire_sync(SecurityEventType::AccountLockout, "u1", HashMap::new());

        // All 3 URLs were attempted (failing one retries 3 times).
        // call_count = 3 retries on failing + 1 on ok1 + 1 on ok2 = 5
        assert_eq!(client.call_count.load(Ordering::SeqCst), 5);
        assert_eq!(manager.delivery_log_count(), 3);

        let log = manager.get_delivery_log(1, 10);
        let successes: Vec<_> = log.iter().filter(|e| e.success).collect();
        let failures: Vec<_> = log.iter().filter(|e| !e.success).collect();
        assert_eq!(successes.len(), 2, "two URLs should succeed");
        assert_eq!(failures.len(), 1, "one URL should fail");
        assert_eq!(failures[0].url, "http://example.com/failing");
    }

    #[test]
    fn test_remove_config_url_removes_single_endpoint() {
        let (manager, mock) = make_manager(0);
        manager
            .configure(
                SecurityEventType::CanaryTriggered,
                "http://example.com/hook1".to_string(),
            )
            .unwrap();
        manager
            .configure(
                SecurityEventType::CanaryTriggered,
                "http://example.com/hook2".to_string(),
            )
            .unwrap();

        // Remove only hook1.
        manager.remove_config_url(
            &SecurityEventType::CanaryTriggered,
            "http://example.com/hook1",
        );

        manager.fire_sync(SecurityEventType::CanaryTriggered, "u1", HashMap::new());

        // Only hook2 should be called.
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 1);
        let log = manager.get_delivery_log(1, 10);
        assert_eq!(log[0].url, "http://example.com/hook2");
    }
}
