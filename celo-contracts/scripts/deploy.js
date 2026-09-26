const hre = require("hardhat");

// Expected chain ids for the supported targets. Localnet (hardhat) uses 31337.
const EXPECTED_CHAIN_IDS = {
  hardhat: 31337,
  localhost: 31337,
  celo: 42220,
  alfajores: 44787,
};

// Read-only methods the deployed registry must expose.
const REQUIRED_METHODS = [
  "owner",
  "registerPet",
  "getPet",
  "totalPets",
];

function fail(message) {
  console.error(`[verify] FAIL: ${message}`);
  process.exitCode = 1;
}

function ok(message) {
  console.log(`[verify] OK: ${message}`);
}

async function verifyChainId(expectedChainId) {
  const network = await hre.ethers.provider.getNetwork();
  const chainId = Number(network.chainId);
  if (expectedChainId !== undefined && chainId !== expectedChainId) {
    fail(`chain id mismatch: expected ${expectedChainId}, got ${chainId}`);
    return false;
  }
  ok(`chain id ${chainId}`);
  return true;
}

async function verifyCode(address, artifact) {
  const code = await hre.ethers.provider.getCode(address);
  if (!code || code === "0x") {
    fail(`no contract code at ${address}`);
    return false;
  }
  const onchainHash = hre.ethers.keccak256(code);
  const artifactHash = hre.ethers.keccak256(artifact.deployedBytecode);
  if (onchainHash !== artifactHash) {
    fail(`bytecode hash mismatch at ${address}: onchain ${onchainHash}, artifact ${artifactHash}`);
    return false;
  }
  ok(`code hash matches artifact at ${address}`);
  return true;
}

async function verifyMethods(address, artifact) {
  const iface = new hre.ethers.Interface(artifact.abi);
  const missing = REQUIRED_METHODS.filter((name) => !iface.getFunction(name));
  if (missing.length > 0) {
    fail(`missing required methods: ${missing.join(", ")}`);
    return false;
  }
  ok(`required methods present: ${REQUIRED_METHODS.join(", ")}`);
  return true;
}

async function verifyAdmin(address, artifact) {
  const registry = new hre.ethers.Contract(address, artifact.abi, hre.ethers.provider);
  const owner = await registry.owner();
  if (!owner || owner === hre.ethers.ZeroAddress) {
    fail(`admin not initialized (owner is ${owner})`);
    return false;
  }
  ok(`admin initialized: ${owner}`);
  return true;
}

async function verifyHealth(address, artifact) {
  const registry = new hre.ethers.Contract(address, artifact.abi, hre.ethers.provider);
  const total = await registry.totalPets();
  ok(`health read totalPets() = ${total}`);
  return true;
}

async function verifyDeployment(address, artifact, expectedChainId) {
  const checks = [
    await verifyChainId(expectedChainId),
    await verifyCode(address, artifact),
    await verifyMethods(address, artifact),
    await verifyAdmin(address, artifact),
    await verifyHealth(address, artifact),
  ];
  return checks.every(Boolean);
}

async function main() {
  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${hre.network.name}`);

  const expectedChainId = EXPECTED_CHAIN_IDS[hre.network.name];
  const artifact = await hre.artifacts.readArtifact("PetChainRegistry");
  const verified = await verifyDeployment(address, artifact, expectedChainId);
  if (!verified) {
    process.exitCode = 1;
  }

  return address;
}

if (require.main === module) {
  main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });
}

module.exports = { main, verifyDeployment, EXPECTED_CHAIN_IDS, REQUIRED_METHODS };
