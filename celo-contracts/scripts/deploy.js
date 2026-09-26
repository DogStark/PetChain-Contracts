const hre = require("hardhat");

// Privileged (admin/emergency) actions must be replay-resistant and bound to
// the contract + network context. This helper tracks the per-action sequence
// (nonce) that privileged calls consume, so a replayed authorization is
// rejected and a nonce is only consumed once the action succeeds.
const PRIVILEGED_ACTIONS = ["setPolicy", "pause", "unpause", "emergencyWithdraw"];

function createNonceTracker(contractAddress, networkName) {
  const consumed = new Map();

  function context() {
    return { contract: contractAddress.toLowerCase(), network: networkName };
  }

  function nextSequence(action) {
    if (!PRIVILEGED_ACTIONS.includes(action)) {
      throw new Error(`Unknown privileged action: ${action}`);
    }
    return (consumed.get(action) || 0) + 1;
  }

  // Executes a privileged action with replay-resistant nonce semantics.
  // The nonce is only marked consumed after the transaction succeeds, so a
  // failed transaction can be retried with the same nonce.
  async function runPrivileged(action, send) {
    const sequence = nextSequence(action);
    const ctx = context();

    const tx = await send({ action, sequence, ...ctx });
    const receipt = await tx.wait();

    consumed.set(action, sequence);

    // Audit event exposes action type and sequence without secret material.
    console.log(
      JSON.stringify({
        event: "privileged-action",
        action,
        sequence,
        contract: ctx.contract,
        network: ctx.network,
        txHash: receipt.hash,
      })
    );

    return receipt;
  }

  return { runPrivileged, nextSequence, context };
}

// ---------------------------------------------------------------------------
// Deterministic local fixtures for client verification of pet ownership.
//
// These fixtures are sanitized: they use clearly-labeled test-only placeholder
// addresses and never contain private keys or production addresses. They are
// fully deterministic (stable ordering, no timestamps, no randomness) so that
// client repositories (mobile + frontend) can verify ownership, record
// integrity, and access-control responses without a live network.
// ---------------------------------------------------------------------------

// Test-only placeholder addresses. These are NOT production addresses and are
// derived from a fixed, documented label so they are reproducible across runs.
const TEST_ONLY_ADDRESSES = {
  owner: "0x0000000000000000000000000000000000000a11",
  otherOwner: "0x0000000000000000000000000000000000000b22",
  unauthorized: "0x0000000000000000000000000000000000000c33",
};

// Canonical pet ids are stable, human-readable, and network-agnostic so both
// the Celo and Stellar paths can reference the same fixture set.
const CANONICAL_PETS = [
  { petId: "pet-0001", owner: TEST_ONLY_ADDRESSES.owner },
  { petId: "pet-0002", owner: TEST_ONLY_ADDRESSES.otherOwner },
];

// Deterministic record digest: a stable, non-cryptographic digest over the
// canonical record fields. It is intentionally reproducible so clients can
// assert integrity without depending on a live chain or secret material.
function recordDigest(record) {
  const canonical = [
    record.petId,
    record.owner,
    record.name,
    record.species,
    String(record.birthYear),
  ].join("|");

  // FNV-1a (32-bit) over the canonical string, rendered as fixed-width hex.
  let hash = 0x811c9dc5;
  for (let i = 0; i < canonical.length; i += 1) {
    hash ^= canonical.charCodeAt(i);
    hash = Math.imul(hash, 0x01000193) >>> 0;
  }
  return `0x${hash.toString(16).padStart(8, "0")}`;
}

const CANONICAL_RECORDS = [
  { petId: "pet-0001", owner: TEST_ONLY_ADDRESSES.owner, name: "Rex", species: "dog", birthYear: 2019 },
  { petId: "pet-0002", owner: TEST_ONLY_ADDRESSES.otherOwner, name: "Milo", species: "cat", birthYear: 2021 },
];

// Expected verification outcomes cover both positive and negative cases so
// clients can assert access-control behavior deterministically.
function buildVerificationCases() {
  const records = CANONICAL_RECORDS.map((record) => ({
    ...record,
    digest: recordDigest(record),
  }));

  const [first, second] = records;

  return [
    {
      caseId: "ownership-valid",
      petId: first.petId,
      caller: first.owner,
      expected: { authorized: true, reason: "owner-match" },
    },
    {
      caseId: "ownership-mismatch",
      petId: first.petId,
      caller: second.owner,
      expected: { authorized: false, reason: "owner-mismatch" },
    },
    {
      caseId: "record-integrity-valid",
      petId: first.petId,
      digest: first.digest,
      expected: { valid: true, reason: "digest-match" },
    },
    {
      caseId: "record-integrity-tampered",
      petId: first.petId,
      digest: "0xdeadbeef",
      expected: { valid: false, reason: "digest-mismatch" },
    },
    {
      caseId: "access-unauthorized",
      petId: second.petId,
      caller: TEST_ONLY_ADDRESSES.unauthorized,
      expected: { authorized: false, reason: "unauthorized-caller" },
    },
  ];
}

// Emits the canonical fixture set for a given network path. The shape is
// identical for Celo and Stellar so mobile and frontend consumers can share
// the same documented JSON contract.
function buildFixtures(networkPath) {
  const records = CANONICAL_RECORDS.map((record) => ({
    ...record,
    digest: recordDigest(record),
  }));

  return {
    network: networkPath,
    pets: CANONICAL_PETS.map((pet) => ({ ...pet })),
    records,
    events: records.map((record) => ({
      event: "PetRegistered",
      petId: record.petId,
      owner: record.owner,
      digest: record.digest,
    })),
    verification: buildVerificationCases(),
  };
}

function emitFixtures() {
  const fixtures = {
    version: 1,
    celo: buildFixtures("celo"),
    stellar: buildFixtures("stellar"),
  };
  console.log(JSON.stringify(fixtures, null, 2));
  return fixtures;
}

async function main() {
  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${hre.network.name}`);

  // Bind privileged authorization to this contract + network context.
  const nonces = createNonceTracker(address, hre.network.name);
  console.log(
    `Privileged nonce context: ${JSON.stringify(nonces.context())}`
  );

  // Emit deterministic local fixtures for client verification.
  emitFixtures();

  return address;
}

module.exports = {
  createNonceTracker,
  recordDigest,
  buildFixtures,
  emitFixtures,
  TEST_ONLY_ADDRESSES,
  CANONICAL_PETS,
  CANONICAL_RECORDS,
};

if (require.main === module) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}
