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

- Every indexed event includes a `version` field as its first payload member.
  The value is the schema version of that event's payload.
- New fields are **additive**: they are appended after existing fields and the
  `version` is bumped only when the change is not backward compatible.
- Removing, renaming, or reordering an existing field, or changing its type, is
  a breaking change and requires a new version. Indexers must branch on
  `version` to reconstruct

### Immutability and replacement policy

Commitments are **append-only per record version**. The contract enforces the
following deterministic policy:

- **Append.** The first commitment written for a `recordId` is stored and emits
  `MedicalRecordCommitmentSet(recordId, petId, submitter, version, timestamp, commitment)`
  with `version == 1`.
- **Duplicate.** Re-submitting the exact same commitment for the same
  `recordId` is a no-op: it does not revert, does not overwrite state, and does
  not emit a new event. Clients can safely retry a write.
- **Replacement.** A different commitment for an existing `recordId` is only
  accepted as a new version. The replacement must keep the original `petId` and
  submitter association; a write that changes either reverts. Accepted
  replacements increment `version` and emit the same event with the new
  `version` and `timestamp`.
- **Unauthorized writes.** Only the original submitter (or an authorized vet)
  may append a replacement version; any other caller reverts.

Because every accepted write emits `version` and `timestamp`, clients can
reconstruct the full commitment history for a record by replaying
`MedicalRecordCommitmentSet` events in log order and grouping by `recordId`.

### Client verification flow

1. Read the latest commitment with `medicalRecordCommitments(recordId)` and the
   current `version`.
2. Fetch the record fields from the off-chain store and recompute the
   commitment hash using the encoding above.
3. Call `verifyMedicalRecordCommitment(...)` with the recomputed fields and the
   on-chain commitment. A `true` result proves the off-chain record matches the
   on-chain commitment for that version.
4. To audit history, replay `MedicalRecordCommitmentSet` events for the
   `recordId`, ordering by `version` (and `timestamp`), and verify each version
   independently. A mismatch at any version indicates tampering.

## Deployment manifests

Every deployment is recorded in a checked-in, versioned manifest under
`deployments/<network>.json`. A manifest binds a contract address to the chain
id it was deployed on, the deployed bytecode hash, the compiler settings used,
and the deployment transaction hash. Manifests are the source of truth for
client and deployment tooling; they are validated in CI.

```json
{
  "network": "alfajores",
  "chainId": 44787,
  "contract": "PetChainRegistry",
  "address": "0xDeployedAddress",
  "bytecodeHash": "0x<keccak256 of deployed bytecode>",
  "compiler": { "version": "0.8.20", "optimizer": { "enabled": true, "runs": 200 } },
  "transactionHash": "0x<deployment tx hash>"
}
```

Rules enforced by the manifest checks:

- **Chain binding.** A manifest is rejected when the connected chain id does not
  match its `chainId`; an address can never be reused on a different chain.
- **Bytecode integrity.** The recorded `bytecodeHash` must equal the keccak256
  hash of the deployed artifact's bytecode; a mismatch fails the check.
- **Freshness.** Missing manifests, or manifests whose `bytecodeHash` no longer
  matches the compiled artifact, fail client and deployment checks.
- **No secrets.** Manifests contain only public deployment data — never private
  keys, mnemonics, or API keys.

Generate or refresh a manifest after deploying:

```bash
npx hardhat run scripts/deploy.js --network alfajores
```

## Environment variables

Create a `.env` file in `celo-contracts/` (never commit this file):

```bash
PRIVATE_KEY=your_wallet_private_key
CELOSCAN_API_KEY=your_celoscan_api_key
```

| Variable            | Required for                          |
|---------------------|----------------------------------------|
| `PRIVATE_KEY`        | Signing transactions on `alfajores`/`celo` |
| `CELOSCAN_API_KEY`   | Verifying contracts on Celoscan       |

## Running tests

```bash
npx hardhat test
```

## Medical-record commitments

Each medical record stores a versioned commitment in
`medicalRecordCommitments(recordId)`. The commitment is

```text
keccak256(abi.encode(
  MEDICAL_RECORD_COMMITMENT_DOMAIN,
  MEDICAL_RECORD_COMMITMENT_VERSION,
  recordId, petId, vet, recordType,
  diagnosis, treatment, notes, timestamp
))
```

Use `verifyMedicalRecordCommitment` as a permissionless view. Pass the
canonical record fields and the expected `bytes32` commitment; the contract
performs the Solidity ABI encoding and returns `true` only when every field,
domain, and version matches. Diagnosis and treatment must be non-empty and all
three text fields must be at most `MAX_LONG_LEN` bytes. Invalid, oversized,
unknown, or stale inputs return `false` without writing state.

## Reentrancy and external-call policy

`PetChainRegistry` follows a strict checks-effects-interactions (CEI) policy.
All state mutations (registration, ownership transfer, record writes) are
committed **before** any external call is made, and no external call is allowed
to re-enter a state-mutating entry point. Any future extension that adds a
callback or token transfer must preserve this ordering or be wrapped in a
reentrancy guard.

### External calls

The contract currently makes **no external calls** from its state-mutating
paths. The table below enumerates every external interaction so reviewers can
confirm the CEI invariant holds as the registry grows.

| Entry point            | External call | Guard / ordering                          |
|------------------------|---------------|-------------------------------------------|
| `registerPet`          | none          | effects only; no callback surface         |
| `transferPetOwnership` | none          | ownership updated before any emit         |
| `addMedicalRecord`     | none          | commitment stored before event emission   |
| `verifyMedicalRecordCommitment` | none | pure view; no state writes            |

If a future change introduces an external call (e.g. ERC-20/721 transfer or a
receiver hook), it must:

1. Perform all state effects first (CEI), and
2. Be covered by the malicious-receiver harness in
   `test/reentrancy.test.js` so a regression fails CI.

### Malicious receiver harness

The `test/reentrancy.test.js` suite deploys attacker contracts
(`contracts/test/ReentrantAttacker.sol`) that attempt to re-enter
`registerPet` and `transferPetOwnership` from a callback. The tests assert that
malicious callbacks cannot repeat registration or transfer effects: a reentrant
call reverts and the original state change is applied exactly once. These tests
fail if a protected path becomes reentrant.

## Scripts

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

Deploys `PetChainRegistry`, prints its address, and writes the chain-specific
deployment manifest to `deployments/<network>.json`.

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

Registers a sample pet against an already-deployed `PetChainRegistry`. The
contract address is read from the selected network's manifest
(`deployments/<network>.json`); set `CONTRACT_ADDRESS` to override it. The
script fails if the manifest is missing, stale, or bound to a different chain
id.

```bash
npx hardhat run scripts/register-pet.js --network alfajores

# Explicit override
CONTRACT_ADDRESS=0xDeployedAddress npx hardhat run scripts/register-pet.js --network alfajores
```

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
