# Celo Contracts

This directory contains the Celo-side smart contracts for the PetChain platform.

## Verified-Vet Capability Scopes

Vet verification is represented as a set of explicit capability scopes rather than a single global `verified` flag. Each scope grants a narrowly-defined permission that can be checked independently by consumers (e.g. the vaccination registry, the medical-record store, and the transfer-attestation flow).

### Capability scopes

| Scope | Value | Grants |
| --- | --- | --- |
| `VACCINATION` | `1 << 0` | Ability to sign vaccination records |
| `MEDICAL_RECORD` | `1 << 1` | Ability to write/attest medical records |
| `TRANSFER_ATTESTATION` | `1 << 2` | Ability to attest ownership transfers |

Scopes are stored as a bitmask (`uint8`) per vet address. A vet may hold any combination of scopes; the empty mask (`0`) means the vet is not verified for anything.

### Storage layout

- `mapping(address => uint8) vetScopes` — capability bitmask per vet.
- `mapping(address => uint256) vetScopeUpdatedAt` — block timestamp of the last scope change (used for expiry/replay checks).

Storage keys are namespaced under the existing `petchain.vet.` prefix to preserve the current storage-version conventions. No existing keys are renamed or removed; the legacy `verified` boolean is derived as `vetScopes[vet] != 0` for backward compatibility.

### Authorization

- Only the contract owner (existing `onlyOwner` modifier) may grant or revoke scopes.
- Consumers must call `hasScope(vet, scope)` before performing a scoped action; a missing scope reverts with `UnauthorizedScope`.
- Granting a scope that is already set is a no-op (idempotent) and does not emit an event.

### Error codes

| Code | Name | Meaning |
| --- | --- | --- |
| `E01` | `UnauthorizedScope` | Caller/vet lacks the required capability scope |
| `E02` | `InvalidScope` | Scope value is `0` or contains bits outside the defined mask |
| `E03` | `ScopeExpired` | Scope update is older than the configured TTL for the action |
| `E04` | `NotOwner` | Non-owner attempted to grant/revoke scopes |

### Events

- `ScopeGranted(address indexed vet, uint8 scope, uint256 timestamp)`
- `ScopeRevoked(address indexed vet, uint8 scope, uint256 timestamp)`

Events are emitted only on actual state transitions (no event for idempotent grants/revokes).

### Bounds and limits

- Scope masks are bounded to the defined bits (`0x07`); any higher bit reverts with `InvalidScope`.
- Grant/revoke operate on a single vet and a single scope per call — no unbounded iteration.
- Persistent state growth is O(number of vets), with two fixed-size slots per vet.

### ABI / compatibility

- `grantScope(address vet, uint8 scope)` and `revokeScope(address vet, uint8 scope)` are additive; existing `verify(address)` / `isVerified(address)` remain and map to the full scope mask.
- `hasScope(address vet, uint8 scope) view returns (bool)` is the new read entry point.
- Compatibility fixtures under `contracts/celo/fixtures/` must be regenerated when the ABI changes; the derived `verified` value is preserved so existing fixtures remain valid.

### Tests

Focused tests cover:

- Successful grant/revoke and `hasScope` checks for each scope.
- Unauthorized access: non-owner grant/revoke reverts with `NotOwner`; consumer action without the required scope reverts with `UnauthorizedScope`.
- Boundary values: scope `0` and out-of-range bits revert with `InvalidScope`; full mask `0x07` is accepted.
- Replay/expiry: stale scope updates past the TTL revert with `ScopeExpired`.
- Failed calls leave `vetScopes` and `vetScopeUpdatedAt` unchanged (state-unchanged assertions).
