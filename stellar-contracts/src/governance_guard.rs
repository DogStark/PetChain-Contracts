//! Voting-identity helpers for multisig governance proposals (Issues #1210, #1211).
//!
//! Identity model (assumption, documented): a governance voter is identified
//! by its Soroban `Address`. There is no vote delegation in the contract, so
//! one address is exactly one vote. A proposal's `approvals` list is expected
//! to hold each address at most once (`approve_proposal` enforces this), but
//! tallies must not rely on that alone: any duplicated entry is counted once.

use soroban_sdk::{Address, Vec};

/// Returns the number of distinct addresses in `approvals`.
pub fn distinct_count(approvals: &Vec<Address>) -> u32 {
    let mut seen: Vec<Address> = Vec::new(approvals.env());
    for a in approvals.iter() {
        if !seen.contains(&a) {
            seen.push_back(a);
        }
    }
    seen.len()
}

/// Returns the distinct addresses of `approvals` that are members of `eligible`.
pub fn eligible_approvals(approvals: &Vec<Address>, eligible: &Vec<Address>) -> u32 {
    let mut seen: Vec<Address> = Vec::new(approvals.env());
    for a in approvals.iter() {
        if eligible.contains(&a) && !seen.contains(&a) {
            seen.push_back(a);
        }
    }
    seen.len()
}

/// Builds a de-duplicated eligible-voter list from the current admin set
/// (multisig list plus the legacy single admin, if any).
pub fn dedupe(list: &Vec<Address>, extra: Option<Address>) -> Vec<Address> {
    let mut out: Vec<Address> = Vec::new(list.env());
    for a in list.iter() {
        if !out.contains(&a) {
            out.push_back(a);
        }
    }
    if let Some(a) = extra {
        if !out.contains(&a) {
            out.push_back(a);
        }
    }
    out
}
