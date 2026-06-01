use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// Precedence: allowlist always wins over blocklist.
///   1. Allowlist match → Allow
///   2. Blocklist match → Block
///   3. Default         → Allow
#[derive(Debug, Clone, PartialEq)]
pub enum ListType {
    Allow,
    Block,
}

#[derive(Debug, Clone)]
pub struct IpFilterEntry {
    pub id:         String,
    pub list_type:  ListType,
    pub cidr:       String,
    pub network:    IpAddr,
    pub prefix_len: u8,
    pub note:       Option<String>,
}

impl IpFilterEntry {
    pub fn parse(
        id:        impl Into<String>,
        list_type: ListType,
        cidr:      &str,
        note:      Option<String>,
    ) -> Result<Self, String> {
        let (network, prefix_len) = parse_cidr(cidr)?;
        Ok(IpFilterEntry { id: id.into(), list_type, cidr: cidr.to_owned(), network, prefix_len, note })
    }
}

pub fn parse_cidr(cidr: &str) -> Result<(IpAddr, u8), String> {
    if let Some((addr_str, prefix_str)) = cidr.split_once('/') {
        let addr = IpAddr::from_str(addr_str)
            .map_err(|e| format!("Invalid IP '{addr_str}': {e}"))?;
        let prefix: u8 = prefix_str
            .parse()
            .map_err(|_| format!("Invalid prefix '{prefix_str}'"))?;
        let max = if addr.is_ipv4() { 32 } else { 128 };
        if prefix > max {
            return Err(format!("Prefix /{prefix} exceeds maximum /{max}"));
        }
        Ok((addr, prefix))
    } else {
        let addr = IpAddr::from_str(cidr)
            .map_err(|e| format!("Invalid IP '{cidr}': {e}"))?;
        let prefix = if addr.is_ipv4() { 32 } else { 128 };
        Ok((addr, prefix))
    }
}

pub fn ip_in_cidr(candidate: &IpAddr, network: &IpAddr, prefix_len: u8) -> bool {
    match (candidate, network) {
        (IpAddr::V4(c), IpAddr::V4(n)) => ipv4_in_cidr(c, n, prefix_len),
        (IpAddr::V6(c), IpAddr::V6(n)) => ipv6_in_cidr(c, n, prefix_len),
        _ => false,
    }
}

fn ipv4_in_cidr(candidate: &Ipv4Addr, network: &Ipv4Addr, prefix_len: u8) -> bool {
    if prefix_len == 0 { return true; }
    let mask = !0u32 << (32 - prefix_len as u32);
    (u32::from(*candidate) & mask) == (u32::from(*network) & mask)
}

fn ipv6_in_cidr(candidate: &Ipv6Addr, network: &Ipv6Addr, prefix_len: u8) -> bool {
    if prefix_len == 0 { return true; }
    let mask: u128 = !0u128 << (128 - prefix_len as u32);
    (u128::from(*candidate) & mask) == (u128::from(*network) & mask)
}

#[derive(Debug, PartialEq)]
pub enum FilterDecision {
    Allow,
    Block,
}

/// Evaluates the IP filter decision. Allowlist is checked before blocklist;
/// missing from both defaults to Allow.
pub fn evaluate_ip(candidate: &IpAddr, entries: &[IpFilterEntry]) -> FilterDecision {
    for entry in entries.iter().filter(|e| e.list_type == ListType::Allow) {
        if ip_in_cidr(candidate, &entry.network, entry.prefix_len) {
            return FilterDecision::Allow;
        }
    }
    for entry in entries.iter().filter(|e| e.list_type == ListType::Block) {
        if ip_in_cidr(candidate, &entry.network, entry.prefix_len) {
            return FilterDecision::Block;
        }
    }
    FilterDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ip4(s: &str) -> IpAddr { IpAddr::V4(s.parse().unwrap()) }
    fn ip6(s: &str) -> IpAddr { IpAddr::V6(s.parse().unwrap()) }
    fn allow(cidr: &str) -> IpFilterEntry { IpFilterEntry::parse("a", ListType::Allow, cidr, None).unwrap() }
    fn block(cidr: &str) -> IpFilterEntry { IpFilterEntry::parse("b", ListType::Block, cidr, None).unwrap() }

    #[test]
    fn parses_ipv4_cidr() {
        let (addr, prefix) = parse_cidr("192.168.1.0/24").unwrap();
        assert_eq!(prefix, 24);
        assert_eq!(addr, IpAddr::V4("192.168.1.0".parse().unwrap()));
    }

    #[test]
    fn bare_ipv4_treated_as_32() {
        let (_, prefix) = parse_cidr("10.0.0.1").unwrap();
        assert_eq!(prefix, 32);
    }

    #[test]
    fn parses_ipv6_cidr() {
        let (_, prefix) = parse_cidr("2001:db8::/32").unwrap();
        assert_eq!(prefix, 32);
    }

    #[test]
    fn rejects_invalid_ip() {
        assert!(parse_cidr("not.an.ip/24").is_err());
    }

    #[test]
    fn rejects_prefix_too_large() {
        assert!(parse_cidr("10.0.0.0/33").is_err());
    }

    #[test]
    fn ip_in_subnet() {
        let net: IpAddr = "192.168.1.0".parse().unwrap();
        assert!(ip_in_cidr(&ip4("192.168.1.100"), &net, 24));
        assert!(!ip_in_cidr(&ip4("192.168.2.1"),  &net, 24));
    }

    #[test]
    fn slash32_exact_match() {
        let net: IpAddr = "10.0.0.5".parse().unwrap();
        assert!( ip_in_cidr(&ip4("10.0.0.5"), &net, 32));
        assert!(!ip_in_cidr(&ip4("10.0.0.6"), &net, 32));
    }

    #[test]
    fn slash0_matches_all() {
        let net: IpAddr = "0.0.0.0".parse().unwrap();
        assert!(ip_in_cidr(&ip4("1.2.3.4"), &net, 0));
    }

    #[test]
    fn ipv6_cidr_matches() {
        let net: IpAddr = "2001:db8::".parse().unwrap();
        assert!( ip_in_cidr(&ip6("2001:db8::1"), &net, 32));
        assert!(!ip_in_cidr(&ip6("2001:db9::1"), &net, 32));
    }

    #[test]
    fn blocked_ip_returns_block() {
        assert_eq!(evaluate_ip(&ip4("10.1.2.3"), &[block("10.0.0.0/8")]), FilterDecision::Block);
    }

    #[test]
    fn unknown_ip_defaults_to_allow() {
        assert_eq!(evaluate_ip(&ip4("8.8.8.8"), &[block("10.0.0.0/8")]), FilterDecision::Allow);
    }

    #[test]
    fn allowlist_bypasses_blocklist() {
        let entries = vec![block("10.0.0.0/8"), allow("10.0.0.5/32")];
        assert_eq!(evaluate_ip(&ip4("10.0.0.5"), &entries), FilterDecision::Allow);
        assert_eq!(evaluate_ip(&ip4("10.0.0.6"), &entries), FilterDecision::Block);
    }

    #[test]
    fn allowlisted_subnet_bypasses_block_for_range() {
        let entries = vec![block("10.0.0.0/8"), allow("10.50.0.0/16")];
        assert_eq!(evaluate_ip(&ip4("10.50.1.1"), &entries), FilterDecision::Allow);
        assert_eq!(evaluate_ip(&ip4("10.51.1.1"), &entries), FilterDecision::Block);
    }

    #[test]
    fn empty_list_allows_all() {
        assert_eq!(evaluate_ip(&ip4("1.2.3.4"), &[]), FilterDecision::Allow);
    }
}
