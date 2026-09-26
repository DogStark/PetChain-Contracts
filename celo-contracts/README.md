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

## Scripts

### `scripts/deploy.js`

Deploys `PetChainRegistry` and prints its address.

```bash
# Local network (no env vars needed)
npx hardhat run scripts/deploy.js --network hardhat

# Celo Alfajores testnet
npx hardhat run scripts/deploy.js --network alfajores

# Celo mainnet
npx hardhat run scripts/deploy.js --network celo
```

### `scripts/register-pet.js`

Registers a sample pet against an already-deployed `PetChainRegistry`. Requires
`CONTRACT_ADDRESS` to be set to the address printed by `deploy.js`.

```bash
CONTRACT_ADDRESS=0xDeployedAddress npx hardhat run scripts/register-pet.js --network alfajores
```

## Networks

| Network    | Chain ID | RPC URL                                   |
|------------|----------|--------------------------------------------|
| `alfajores`| 44787    | https://alfajores-forno.celo-testnet.org    |
| `celo`     | 42220    | https://forno.celo.org                      |
