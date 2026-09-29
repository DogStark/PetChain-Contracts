//! Administrative action reason codes and audit events.
//!
//! Every administrative mutation (policy, access, lifecycle) must carry a
//! bounded [`ReasonCode`]. Audit events emitted here identify the action, the
//! target, the actor, the old value, the new value, and the schema version,
//! and never contain secrets.

use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol};

/// Schema version for admin audit events. Bump when the event shape changes.
pub const ADMIN_AUDIT_SCHEMA_VERSION: u32 = 1;

/// Bounded, enumerable set of reason codes for administrative mutations.
///
/// Reason codes are intentionally a closed set: callers cannot pass arbitrary
/// unbounded strings, so the audit vocabulary stays machine-readable and
/// stable across releases.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ReasonCode {
    /// Initial configuration of a policy, role, or lifecycle state.
    InitialSetup = 0,
    /// Routine, scheduled administrative update.
    RoutineUpdate = 1,
    /// Correcting a previously misconfigured value.
    Correction = 2,
    /// Emergency response to an incident.
    Emergency = 3,
    /// Governance or multisig approved change.
    Governance = 4,
    /// Compliance or regulatory requirement.
    Compliance = 5,
    /// Deprecation or wind-down of a feature or role.
    Deprecation = 6,
}

impl ReasonCode {
    /// Numeric discriminant stored in the audit event.
    pub fn code(&self) -> u32 {
        *self as u32
    }

    /// Human-readable label for off-chain indexing and documentation.
    pub fn label(&self) -> Symbol {
        match self {
            ReasonCode::InitialSetup => symbol_short!("init_setup"),
            ReasonCode::RoutineUpdate => symbol_short!("routine"),
            ReasonCode::Correction => symbol_short!("correction"),
            ReasonCode::Emergency => symbol_short!("emergency"),
            ReasonCode::Governance => symbol_short!("governance"),
            ReasonCode::Compliance => symbol_short!("compliance"),
            ReasonCode::Deprecation => symbol_short!("deprecation"),
        }
    }
}

/// Kind of administrative mutation being audited.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AdminAction {
    PolicyUpdate = 0,
    AccessGrant = 1,
    AccessRevoke = 2,
    LifecycleChange = 3,
}

/// Audit event payload. Contains no secrets: only identifiers, the bounded
/// reason code, and the old/new values being changed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminAuditEvent {
    pub schema_version: u32,
    pub action: AdminAction,
    pub target: Address,
    pub actor: Address,
    pub reason_code: u32,
    pub reason_label: Symbol,
    pub old_value: i128,
    pub new_value: i128,
}

/// Emit a structured audit event for an administrative mutation.
///
/// The event topic is stable (`admin` / `audit`) so indexers can subscribe to
/// all administrative changes, while the payload carries the full context.
pub fn emit_admin_audit(
    env: &Env,
    action: AdminAction,
    target: Address,
    actor: Address,
    reason: ReasonCode,
    old_value: i128,
    new_value: i128,
) {
    let event = AdminAuditEvent {
        schema_version: ADMIN_AUDIT_SCHEMA_VERSION,
        action,
        target,
        actor,
        reason_code: reason.code(),
        reason_label: reason.label(),
        old_value,
        new_value,
    };
    env.events()
        .publish((symbol_short!("admin"), symbol_short!("audit")), event);
}

/// Guard that rejects a no-op administrative mutation (old == new).
///
/// Duplicate actions are rejected so the audit trail only records real
/// state transitions.
pub fn require_state_change(old_value: i128, new_value: i128) {
    if old_value == new_value {
        panic!("admin: duplicate action, no state change");
    }
}

/// Guard that rejects an unauthorized actor for an administrative mutation.
pub fn require_authorized(actor: &Address, authorized: bool) {
    if !authorized {
        panic!("admin: unauthorized action");
    }
    actor.require_auth();
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Events};
    use soroban_sdk::{Env, IntoVal, TryFromVal};

    #[test]
    fn reason_codes_are_bounded_and_documented() {
        // Every variant maps to a stable numeric code and a label.
        let all = [
            ReasonCode::InitialSetup,
            ReasonCode::RoutineUpdate,
            ReasonCode::Correction,
            ReasonCode::Emergency,
            ReasonCode::Governance,
            ReasonCode::Compliance,
            ReasonCode::Deprecation,
        ];
        for (i, code) in all.iter().enumerate() {
            assert_eq!(code.code(), i as u32);
            // Label is a bounded symbol, never an arbitrary string.
            assert!(!code.label().to_string().is_empty());
        }
    }

    #[test]
    fn unauthorized_action_is_rejected() {
        let env = Env::default();
        let actor = Address::generate(&env);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            require_authorized(&actor, false);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_action_is_rejected() {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            require_state_change(42, 42);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn audit_event_is_complete_and_secret_free() {
        let env = Env::default();
        let target = Address::generate(&env);
        let actor = Address::generate(&env);

        emit_admin_audit(
            &env,
            AdminAction::PolicyUpdate,
            target.clone(),
            actor.clone(),
            ReasonCode::Governance,
            10,
            20,
        );

        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let (_, _, data) = events.get(0).unwrap();
        let event = AdminAuditEvent::try_from_val(&env, &data).unwrap();

        assert_eq!(event.schema_version, ADMIN_AUDIT_SCHEMA_VERSION);
        assert_eq!(event.action, AdminAction::PolicyUpdate);
        assert_eq!(event.target, target);
        assert_eq!(event.actor, actor);
        assert_eq!(event.reason_code, ReasonCode::Governance.code());
        assert_eq!(event.reason_label, ReasonCode::Governance.label());
        assert_eq!(event.old_value, 10);
        assert_eq!(event.new_value, 20);
    }

    #[test]
    fn audit_event_roundtrips_through_val() {
        let env = Env::default();
        let event = AdminAuditEvent {
            schema_version: ADMIN_AUDIT_SCHEMA_VERSION,
            action: AdminAction::AccessGrant,
            target: Address::generate(&env),
            actor: Address::generate(&env),
            reason_code: ReasonCode::Emergency.code(),
            reason_label: ReasonCode::Emergency.label(),
            old_value: 0,
            new_value: 1,
        };
        let val = event.clone().into_val(&env);
        let decoded = AdminAuditEvent::try_from_val(&env, &val).unwrap();
        assert_eq!(decoded, event);
    }
}
