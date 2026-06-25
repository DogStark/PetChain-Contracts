//! IP allowlist / blocklist management for the 2FA backend (Issue #701).
//!
//! Entries are matched by CIDR containment so a single rule like
//! `192.168.1.0/24` covers an entire range; a bare IP (no `/`) is treated as
//! a single-address block (`/32` for IPv4, `/128` for IPv6).
//!
//! The allowlist always takes precedence over the blocklist: if an address
//! matches any allow entry it is let through even if it also matches a block
//! entry. An address that matches neither list is allowed by default.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Mutex;

/// Which list an entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IpListType {
    Allow,
    Block,
}

/// A single CIDR rule stored on the allow or block list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpAccessEntry {
    pub id: i64,
    pub cidr: String,
    pub list_type: IpListType,
    pub note: Option<String>,
    pub created_by: String,
}

/// Outcome of checking an address against the access lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpAccessDecision {
    Allowed,
    Blocked,
}

/// A parsed CIDR block. Supports IPv4 and IPv6; a bare address (no `/n`) is
/// treated as a single-host block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CidrBlock {
    network: IpAddr,
    prefix_len: u8,
}

impl CidrBlock {
    pub fn parse(input: &str) -> Result<Self, String> {
        let (addr_part, prefix_part) = match input.split_once('/') {
            Some((addr, prefix)) => (addr, Some(prefix)),
            None => (input, None),
        };

        let network: IpAddr = addr_part
            .trim()
            .parse()
            .map_err(|_| format!("invalid IP address in CIDR '{input}'"))?;

        let max_len: u8 = match network {
            IpAddr::V4(_) => 32,
            IpAddr::V6(_) => 128,
        };

        let prefix_len = match prefix_part {
            Some(p) => p
                .trim()
                .parse::<u8>()
                .map_err(|_| format!("invalid CIDR prefix in '{input}'"))?,
            None => max_len,
        };

        if prefix_len > max_len {
            return Err(format!(
                "CIDR prefix /{prefix_len} exceeds maximum /{max_len} for '{input}'"
            ));
        }

        Ok(Self { network, prefix_len })
    }

    pub fn contains(&self, ip: &IpAddr) -> bool {
        match (self.network, ip) {
            (IpAddr::V4(net), IpAddr::V4(addr)) => {
                let mask = v4_mask(self.prefix_len);
                (u32::from(net) & mask) == (u32::from(*addr) & mask)
            }
            (IpAddr::V6(net), IpAddr::V6(addr)) => {
                let mask = v6_mask(self.prefix_len);
                (u128::from(net) & mask) == (u128::from(*addr) & mask)
            }
            // An IPv4 rule never matches an IPv6 address and vice versa, even
            // though some IPv6 addresses are IPv4-mapped — callers should
            // normalize addresses before checking if that mapping matters.
            _ => false,
        }
    }
}

fn v4_mask(prefix_len: u8) -> u32 {
    if prefix_len == 0 {
        0
    } else {
        u32::MAX << (32 - prefix_len)
    }
}

fn v6_mask(prefix_len: u8) -> u128 {
    if prefix_len == 0 {
        0
    } else {
        u128::MAX << (128 - prefix_len)
    }
}

/// Storage abstraction for IP access rules, implemented by an in-memory
/// store (tests / single-instance deployments) and a Postgres-backed store
/// (`PostgresIpAccessStore` in `db.rs`) for production.
pub trait IpAccessStore: Send + Sync {
    fn add_entry(
        &self,
        cidr: &str,
        list_type: IpListType,
        note: Option<&str>,
        created_by: &str,
    ) -> Result<IpAccessEntry, String>;

    fn remove_entry(&self, id: i64) -> Result<(), String>;

    fn list_entries(&self, list_type: IpListType) -> Vec<IpAccessEntry>;

    /// Decide whether `ip` should be allowed through. Allowlist entries take
    /// precedence over blocklist entries; an address matching neither list
    /// is allowed.
    fn check(&self, ip: IpAddr) -> IpAccessDecision {
        let matches = |entries: Vec<IpAccessEntry>| {
            entries.into_iter().any(|entry| {
                CidrBlock::parse(&entry.cidr)
                    .map(|block| block.contains(&ip))
                    .unwrap_or(false)
            })
        };

        if matches(self.list_entries(IpListType::Allow)) {
            return IpAccessDecision::Allowed;
        }
        if matches(self.list_entries(IpListType::Block)) {
            return IpAccessDecision::Blocked;
        }
        IpAccessDecision::Allowed
    }
}

/// Thread-safe in-memory `IpAccessStore`. Suitable for tests and as the
/// default store when no database is configured.
#[derive(Default)]
pub struct InMemoryIpAccessStore {
    entries: Mutex<HashMap<i64, IpAccessEntry>>,
    next_id: AtomicI64,
}

impl InMemoryIpAccessStore {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(1),
        }
    }
}

impl IpAccessStore for InMemoryIpAccessStore {
    fn add_entry(
        &self,
        cidr: &str,
        list_type: IpListType,
        note: Option<&str>,
        created_by: &str,
    ) -> Result<IpAccessEntry, String> {
        CidrBlock::parse(cidr)?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let entry = IpAccessEntry {
            id,
            cidr: cidr.to_string(),
            list_type,
            note: note.map(str::to_string),
            created_by: created_by.to_string(),
        };

        self.entries.lock().expect("ip access store lock poisoned").insert(id, entry.clone());
        Ok(entry)
    }

    fn remove_entry(&self, id: i64) -> Result<(), String> {
        self.entries
            .lock()
            .expect("ip access store lock poisoned")
            .remove(&id)
            .map(|_| ())
            .ok_or_else(|| format!("no IP access entry with id {id}"))
    }

    fn list_entries(&self, list_type: IpListType) -> Vec<IpAccessEntry> {
        self.entries
            .lock()
            .expect("ip access store lock poisoned")
            .values()
            .filter(|entry| entry.list_type == list_type)
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// actix-web middleware
// ---------------------------------------------------------------------------

use crate::error::ApiError;
use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::sync::Arc;

/// Rejects requests from blocked IPs with `403`, before they reach any
/// handler. The client address is taken from the TCP peer address (not
/// `X-Forwarded-For`), since that header can be spoofed by the client unless
/// a trusted proxy strips/overwrites it — deployments behind a proxy should
/// terminate TLS there and forward the real peer address instead.
pub struct IpAccessMiddleware {
    store: Arc<dyn IpAccessStore>,
}

impl IpAccessMiddleware {
    pub fn new(store: Arc<dyn IpAccessStore>) -> Self {
        Self { store }
    }
}

impl<S, B> Transform<S, ServiceRequest> for IpAccessMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = IpAccessMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(IpAccessMiddlewareService { service, store: self.store.clone() })
    }
}

pub struct IpAccessMiddlewareService<S> {
    service: S,
    store: Arc<dyn IpAccessStore>,
}

impl<S, B> Service<ServiceRequest> for IpAccessMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let blocked = req
            .peer_addr()
            .map(|addr| self.store.check(addr.ip()) == IpAccessDecision::Blocked)
            .unwrap_or(false);

        if blocked {
            let request = req.request().clone();
            let error = ApiError::forbidden("IP address is blocked", None);
            let response = HttpResponse::Forbidden().json(error).map_into_boxed_body();
            return Box::pin(async move { Ok(ServiceResponse::new(request, response)) });
        }

        let fut = self.service.call(req);
        Box::pin(async move { Ok(fut.await?.map_into_boxed_body()) })
    }
}
