//! Storage keys for consent purpose/data-scope versioning (#1201) and
//! bounded consent cleanup cursors (#1203). Kept separate from `ConsentKey`
//! so existing exhaustive key tables and discriminants are untouched.

use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConsentPolicyKey {
    /// Current global consent policy (purpose + data-scope) version.
    PolicyVersion,
    /// Policy version a given consent id was granted/renewed under.
    ConsentVersion(u64),
    /// Resumable cleanup cursor for a pet (next 1-based index to inspect).
    CleanupCursor(u64),
}
