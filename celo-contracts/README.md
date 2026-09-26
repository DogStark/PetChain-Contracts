# Celo Contracts

Smart contracts and generated bindings for the Celo protocol.

## Generated bindings

Rust and TypeScript bindings are generated from the contract ABIs and checked
into this repository. The checked-in output is the source of truth for
downstream consumers, so it must stay in sync with the deployed interface.

### Reproducible generation

Binding generation must be deterministic on a clean checkout:

- Output is sorted by contract name, then by method name, then by argument
  order, so re-running generation never reorders entries.
- Generated files must not contain machine-local paths, absolute paths,
  timestamps, hostnames, or secrets. Only the contract name, ABI-derived
  signatures, and error discriminants are emitted.
- Regenerate with the pinned toolchain and commit the result. A clean checkout
  followed by regeneration must produce no diff.

### Compatibility checks

CI runs a binding compatibility check that compares freshly generated output
against the checked-in snapshot and fails when any of the following drift:

- public method names
- argument order
- return types
- error codes / error discriminants

A failure means the public interface changed without an explicit migration
note. To land an intentional change, add a version entry (see below) and
regenerate the bindings in the same commit.

### Compatibility exceptions

Any intentional breaking change to a public method signature or error
discriminant requires an explicit version entry. Add the entry to the
compatibility exceptions list, including the contract, the affected method or
error, the previous and new signature, and the version in which the change
ships. The compatibility check only passes a drift when a matching version
entry is present; undocumented drift always fails CI.

## Event schema versioning

Contracts emit events that mobile and frontend indexers consume. An event field
change without a version marker can silently break historical reconstruction,
so every indexed event carries a documented topic and an explicit version
policy.

### Version policy

- Every indexed event includes a `version` field as its first payload member.
  The value is the schema version of that event's payload.
- New fields are **additive**: they are appended after existing fields and the
  `version` is bumped only when the change is not backward compatible.
- Removing, renaming, or reordering an existing field, or changing its type, is
  a breaking change and requires a new version. Indexers must branch on
  `version` to reconstruct historical payloads.
- Private payloads (secrets, PII, or any value not intended for public
  consumption) must never be emitted in an event. Only commitments, hashes, and
  public identifiers are emitted.

### Indexed events

| Event | Topic | Version | Payload fields |
| --- | --- | --- | --- |
| `MedicalRecordCommitted` | `keccak256("MedicalRecordCommitted")` | 1 | `version`, `recordId`, `commitment`, `owner` |
| `MedicalRecordRevoked` | `keccak256("MedicalRecordRevoked")` | 1 | `version`, `recordId`, `owner` |

Topics are the keccak256 hash of the event signature; indexers match on the
topic and then decode the payload using the version-specific layout.

### Migration fixtures

Representative old/new fixtures live under `celo-contracts/test/fixtures/`:

- `events.v1.json` — the historical payload layout for each indexed event.
- `events.v2.json` — the current payload layout, with additive fields only.

Historical fixtures must remain parseable: the compatibility parser reads the
`version` field and decodes the payload with the matching layout, so a v1
fixture still parses after a v2 schema ships.

### Test plan

- Clean-checkout generation test: regenerate bindings from a clean checkout and
  assert the output matches the checked-in snapshot byte-for-byte.
- Intentional-drift fixture: a fixture that mutates a method name, argument
  order, return type, and error discriminant, asserting the compatibility check
  fails each case.
- Compile smoke tests: compile each generated binding target (Rust and
  TypeScript) to confirm the generated output builds.
- Event parser test: parse every fixture in `celo-contracts/test/fixtures/` and
  assert each historical payload still decodes, and that no fixture contains a
  private payload field.
- Event schema snapshot check: CI compares the documented event schema against a
  checked-in snapshot and fails when a topic, version, or payload field drifts
  without a matching version bump.
