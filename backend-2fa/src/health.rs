use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SubsystemStatus {
    Healthy,
    Unhealthy(String),
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub healthy: bool,
    pub subsystems: HashMap<String, SubsystemStatus>,
}

pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self) -> SubsystemStatus;
}

pub struct PostgresHealthCheck<'a> {
    store: &'a crate::db::PostgresTwoFactorStore,
}

impl<'a> PostgresHealthCheck<'a> {
    pub fn new(store: &'a crate::db::PostgresTwoFactorStore) -> Self {
        Self { store }
    }
}

impl HealthCheck for PostgresHealthCheck<'_> {
    fn name(&self) -> &str {
        "postgres"
    }

    fn check(&self) -> SubsystemStatus {
        match self.store.health_check() {
            Ok(()) => SubsystemStatus::Healthy,
            Err(e) => SubsystemStatus::Unhealthy(e),
        }
    }
}

pub struct RedisHealthCheck {
    client: redis::Client,
}

impl RedisHealthCheck {
    pub fn new(redis_url: &str) -> Result<Self, String> {
        let client = redis::Client::open(redis_url).map_err(|e| e.to_string())?;
        Ok(Self { client })
    }
}

impl HealthCheck for RedisHealthCheck {
    fn name(&self) -> &str {
        "redis"
    }

    fn check(&self) -> SubsystemStatus {
        match self.client.get_connection() {
            Ok(mut con) => {
                let result: Result<String, _> = redis::cmd("PING").query(&mut con);
                match result {
                    Ok(_) => SubsystemStatus::Healthy,
                    Err(e) => SubsystemStatus::Unhealthy(e.to_string()),
                }
            }
            Err(e) => SubsystemStatus::Unhealthy(e.to_string()),
        }
    }
}

pub struct WebhookHealthCheck<'a> {
    manager: &'a crate::webhooks::WebhookManager,
}

impl<'a> WebhookHealthCheck<'a> {
    pub fn new(manager: &'a crate::webhooks::WebhookManager) -> Self {
        Self { manager }
    }
}

impl HealthCheck for WebhookHealthCheck<'_> {
    fn name(&self) -> &str {
        "webhooks"
    }

    fn check(&self) -> SubsystemStatus {
        let _ = self.manager.delivery_log_count();
        SubsystemStatus::Healthy
    }
}

pub struct HealthAggregator<'a> {
    checks: Vec<Box<dyn HealthCheck + 'a>>,
}

impl<'a> HealthAggregator<'a> {
    pub fn new() -> Self {
        Self { checks: Vec::new() }
    }

    pub fn add_check(mut self, check: Box<dyn HealthCheck + 'a>) -> Self {
        self.checks.push(check);
        self
    }

    pub fn check_all(&self) -> HealthReport {
        let mut subsystems = HashMap::new();
        let mut all_healthy = true;

        for check in &self.checks {
            let status = check.check();
            if status != SubsystemStatus::Healthy {
                all_healthy = false;
            }
            subsystems.insert(check.name().to_string(), status);
        }

        HealthReport {
            healthy: all_healthy,
            subsystems,
        }
    }
}
