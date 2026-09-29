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

## Deployment invariants

The constructor takes `(initialAdmin, expectedChainId)` and reverts when the
admin is the zero address, when `expectedChainId != block.chainid`, or when the
chain is not supported (`31337` local, `44787` Alfajores, `42220` Celo).
`admin()` and the immutable `deploymentChainId()` are queryable after deployment.

## Deactivated pet retention

Deactivating a pet removes it from the owner index, so it no longer appears in
`getPetsByOwner`/`getPetsByOwnerPaged` and the index is bounded by live pets.
`pets(petId)` (including the last owner), medical records and their commitments
are retained for historical ownership and commitment proofs. Reactivation
re-adds the pet to the index exactly once.

## Identifier normalization

`normalizeIdentifier` defines the canonical form used for licence uniqueness
(and available to clients for chip/QR identifiers): ASCII spaces are removed,
`a-z` is upper-cased, any other byte outside printable ASCII (control characters
and all non-ASCII Unicode) is rejected, and the result must be 1-`MAX_SHORT_LEN`
bytes.

## Deterministic local fixtures

Client repositories (mobile and frontend) can verify pet ownership, record
integrity, and access-control responses without a live network or seeded
secrets by consuming the canonical fixture set emitted by
`scripts/generate-fixtures.js`.

```bash
# Emit the canonical fixture JSON to stdout
npx hardhat run scripts/generate-fixtures.js --network hardhat

# Write it to a file for consumers
npx hardhat run scripts/generate-fixtures.js --network hardhat > fixtures/pet-ownership.json
```

The fixtures are deterministic across clean runs: ids, owners, digests, events,
and expected outcomes are derived from fixed inputs with stable ordering and no
timestamps or randomness. All addresses and keys are clearly-labeled test-only
placeholders — no private keys or production addresses are committed.

### Fixture shape

```jsonc
{
  "version": 1,
  "network": "celo",
  "pets": [
    {
      "petId": "0x...",            // canonical pet id (bytes32)
      "owner": "0x...",            // test-only owner address
      "recordDigest": "0x...",     // keccak256 commitment digest
      "events": [
        { "name": "PetRegistered", "petId": "0x...", "owner": "0x..." }
      ]
    }
  ],
  "cases": [
    {
      "id": "valid-ownership",
      "kind": "ownership",
      "petId": "0x...",
      "caller": "0x...",
      "expected": true
    },
    {
      "id": "mismatched-owner",
      "kind": "ownership",
      "petId": "0x...",
      "caller": "0x...",
      "expected": false
    },
    {
      "id": "valid-record",
      "kind": "record",
      "petId": "0x...",
      "recordDigest": "0x...",
      "expected": true
    },
    {
      "id": "tampered-digest",
      "kind": "record",
      "petId": "0x...",
      "recordDigest": "0x...",
      "expected": false
    },
    {
      "id": "unauthorized-access",
      "kind": "access",
      "petId": "0x...",
      "caller": "0x...",
      "expected": false
    }
  ]
}
```

Both positive and negative verification cases are included. The same documented
JSON shape is emitted for the Celo and Stellar paths so mobile and frontend
consumers can share a single parser.

## Scripts

### `scripts/deploy.js`

Deploys `PetChainRegistry` with the network's configured chain id and prints its address.
The admin defaults to the deployer; set `ADMIN_ADDRESS` to override.

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
