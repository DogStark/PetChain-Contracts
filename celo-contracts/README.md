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

## Ownership transfer

`PetChainRegistry` exposes an explicit ownership-transfer path so indexers can
reconstruct an unambiguous owner history.

```solidity
function transferOwnership(address newOwner) external;
event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);
```

Semantics:

- **Authorization.** Only the current owner may call `transferOwnership`. Any
  other caller reverts with `NotOwner`. There is no implicit operator; if an
  operator is ever delegated it must be documented here and enforced in the
  contract before it can transfer.
- **Zero recipient.** `newOwner == address(0)` reverts with `ZeroAddress`.
- **Self transfer.** `newOwner == owner()` reverts with `AlreadyOwner`; a
  no-op transfer is rejected rather than silently accepted.
- **Event.** A successful transfer emits exactly one
  `OwnershipTransferred(previousOwner, newOwner)` event, with the old owner as
  the first indexed argument and the new owner as the second. The event is
  emitted after state is updated, so `owner()` always returns the latest owner
  and matches the event's `newOwner`.
- **Reads.** `owner()` is the single source of truth and returns the latest
  owner consistently before and after the event is observed.

### Tests

`test/PetChainRegistry.test.js` covers the transfer path:

- unauthorized callers revert and leave `owner()` unchanged;
- transfers to `address(0)` and to the current owner revert;
- a successful transfer emits one `OwnershipTransferred` with the correct
  old/new owners and updates `owner()`;
- replaying a transfer from the previous owner reverts after ownership moved.

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
