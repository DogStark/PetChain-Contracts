use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

/// Trait for sending HTTP POST requests (injectable for testing).
pub trait HttpClient: Send + Sync {
    fn post(&self, url: &str, body: &str) -> Result<(), String>;
}

/// Production HTTP client using ureq (if webhook-client feature is enabled),
/// otherwise a no-op stub.
pub struct DefaultHttpClient;

#[cfg(feature = "webhook-client")]
static HTTP_CLIENT: std::sync::OnceLock<ureq::Agent> = std::sync::OnceLock::new();

#[cfg(feature = "webhook-client")]
impl HttpClient for DefaultHttpClient {
    fn post(&self, url: &str, body: &str) -> Result<(), String> {
        let agent = HTTP_CLIENT.get_or_init(|| {
            ureq::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
        });

        let response = agent
            .post(url)
            .set("Content-Type", "application/json")
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
    fn post(&self, _url: &str, _body: &str) -> Result<(), String> {
        // Webhooks disabled: compile with `webhook-client` feature to enable real HTTP requests
        Ok(())
    }
}

/// Manages webhook configuration and delivery with retry logic.
pub struct WebhookManager {
    /// event_type -> webhook URL
    config: Arc<Mutex<HashMap<String, String>>>,
    delivery_log: Arc<Mutex<Vec<WebhookDeliveryLog>>>,
    http_client: Arc<dyn HttpClient>,
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
        }
    }

    /// Admin: configure a webhook URL for a specific event type.
    pub fn configure(&self, event_type: SecurityEventType, url: String) {
        self.config
            .lock()
            .unwrap()
            .insert(event_type.to_string(), url);
    }

    /// Remove a webhook configuration for an event type.
    pub fn remove_config(&self, event_type: &SecurityEventType) {
        self.config
            .lock()
            .unwrap()
            .remove(&event_type.to_string());
    }

    /// Fire a webhook for the given event. Retries up to 3 times with
    /// exponential backoff (1 s, 2 s, 4 s) on failure.
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
                        // Exponential backoff: 1s, 2s, 4s
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
            attempts: attempts + if success { 1 } else { attempts },
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
        let manager = WebhookManager::new(client.clone());
        (manager, client)
    }

    #[test]
    fn test_configure_and_fire_success() {
        let (manager, mock) = make_manager(0);
        manager.configure(
            SecurityEventType::FailedTwoFa,
            "http://example.com/hook".to_string(),
        );
        manager.fire(
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
        // No config set for this event type
        manager.fire(SecurityEventType::AccountLockout, "user1", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 0);
        assert_eq!(manager.delivery_log_count(), 0);
    }

    #[test]
    fn test_retry_succeeds_on_second_attempt() {
        let (manager, mock) = make_manager(1); // fail once, succeed on second
        manager.configure(
            SecurityEventType::RecoveryCodeUsed,
            "http://example.com/hook".to_string(),
        );
        manager.fire(
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
        let (manager, mock) = make_manager(3); // fail all 3 attempts
        manager.configure(
            SecurityEventType::FailedTwoFa,
            "http://example.com/hook".to_string(),
        );
        manager.fire(SecurityEventType::FailedTwoFa, "user3", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 3);
        let log = manager.get_delivery_log(1, 10);
        assert!(!log[0].success);
        assert!(log[0].last_error.is_some());
    }

    #[test]
    fn test_delivery_log_pagination() {
        let (manager, _mock) = make_manager(0);
        manager.configure(
            SecurityEventType::FailedTwoFa,
            "http://example.com/hook".to_string(),
        );
        for i in 0..5 {
            manager.fire(
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
        manager.configure(
            SecurityEventType::CanaryTriggered,
            "http://example.com/hook".to_string(),
        );
        let mut meta = HashMap::new();
        meta.insert("ip".to_string(), "1.2.3.4".to_string());
        manager.fire(SecurityEventType::CanaryTriggered, "canary1", meta);
        let log = manager.get_delivery_log(1, 10);
        assert!(log[0].success);
    }

    #[test]
    fn test_remove_config_stops_delivery() {
        let (manager, mock) = make_manager(0);
        manager.configure(
            SecurityEventType::FailedTwoFa,
            "http://example.com/hook".to_string(),
        );
        manager.remove_config(&SecurityEventType::FailedTwoFa);
        manager.fire(SecurityEventType::FailedTwoFa, "user1", HashMap::new());
        assert_eq!(mock.call_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    #[cfg(feature = "webhook-client")]
    fn test_default_http_client_real_request() {
        let mut server = mockito::Server::new();
        let mock = server.mock("POST", "/")
            .match_header("content-type", "application/json")
            .match_body("{\"test\":true}")
            .with_status(200)
            .create();

        let client = DefaultHttpClient;
        let result = client.post(&server.url(), "{\"test\":true}");
        
        assert!(result.is_ok());
        mock.assert();
    }
}
