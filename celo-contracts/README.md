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
