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
