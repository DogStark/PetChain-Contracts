# celo-contracts

`PetChainRegistry` is the Solidity contract that backs PetChain on the Celo network.

## Setup

```bash
cd celo-contracts
npm install
npx hardhat compile
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

## Scripts

### `scripts/deploy.js`

Deploys `PetChainRegistry`, prints its address, and writes the chain-specific
deployment manifest to `deployments/<network>.json`.

```bash
# Local network (no env vars needed)
npx hardhat run scripts/deploy.js --network hardhat

# Celo Alfajores testnet
npx hardhat run scripts/deploy.js --network alfajores

# Celo mainnet
npx hardhat run scripts/deploy.js --network celo
```

### `scripts/register-pet.js`

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

## Networks

| Network    | Chain ID | RPC URL                                   |
|------------|----------|--------------------------------------------|
| `alfajores`| 44787    | https://alfajores-forno.celo-testnet.org    |
| `celo`     | 42220    | https://forno.celo.org                      |
