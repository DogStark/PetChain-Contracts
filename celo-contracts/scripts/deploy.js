const fs = require("fs");
const path = require("path");
const hre = require("hardhat");

// Networks that require a fully validated production configuration before any
// transaction is signed. Local development networks are intentionally excluded
// so `hardhat node` / `hardhat test` remain usable without extra env vars.
const PRODUCTION_NETWORKS = ["celo", "celo-mainnet", "celo-alfajores", "alfajores"];

// Versioned deployment manifest location. The manifest binds a contract address
// to a chain id, bytecode hash, compiler settings and deployment transaction so
// it can be validated in CI and consumed by clients.
const MANIFEST_DIR = path.join(__dirname, "..", "deployments");
const MANIFEST_VERSION = 1;

// Redact secrets so private keys and full RPC URLs are never printed.
function redact(value) {
  if (!value) return "<unset>";
  const str = String(value);
  if (str.length <= 8) return "***";
  return `${str.slice(0, 4)}...${str.slice(-4)}`;
}

function redactRpc(url) {
  if (!url) return "<unset>";
  try {
    const parsed = new URL(url);
    return `${parsed.protocol}//${parsed.host}/***`;
  } catch (_) {
    return "***";
  }
}

// Fail closed: refuse to deploy when required configuration is missing for the
// selected network. This runs before any contract factory is created, so no
// transaction is ever signed with an incomplete configuration.
function assertDeployable(networkName) {
  const isProduction = PRODUCTION_NETWORKS.includes(networkName);
  if (!isProduction) return;

  const missing = [];
  if (!process.env.CELO_RPC_URL) missing.push("CELO_RPC_URL");
  if (!process.env.DEPLOYER_PRIVATE_KEY) missing.push("DEPLOYER_PRIVATE_KEY");

  if (missing.length > 0) {
    throw new Error(
      `Refusing to deploy to "${networkName}": missing required configuration: ${missing.join(
        ", "
      )}. Set these environment variables before deploying.`
    );
  }
}

// Build a versioned manifest that binds the deployed address to the chain id,
// the deployed bytecode hash, the compiler settings and the deployment tx.
// Secrets (private keys, RPC URLs) are deliberately excluded.
function buildManifest({ networkName, chainId, address, bytecodeHash, txHash, compiler }) {
  return {
    version: MANIFEST_VERSION,
    network: networkName,
    chainId,
    contract: "PetChainRegistry",
    address,
    bytecodeHash,
    compiler,
    transaction: txHash,
  };
}

function manifestPath(networkName) {
  return path.join(MANIFEST_DIR, `${networkName}.json`);
}

function writeManifest(networkName, manifest) {
  fs.mkdirSync(MANIFEST_DIR, { recursive: true });
  fs.writeFileSync(manifestPath(networkName), `${JSON.stringify(manifest, null, 2)}\n`);
}

async function main() {
  const networkName = hre.network.name;
  assertDeployable(networkName);

  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  const chainId = Number((await hre.ethers.provider.getNetwork()).chainId);

  const deploymentTx = registry.deploymentTransaction();
  const txHash = deploymentTx ? deploymentTx.hash : null;

  // Hash the deployed artifact bytecode so the manifest can be verified against
  // the compiled artifact in CI.
  const artifact = await hre.artifacts.readArtifact("PetChainRegistry");
  const bytecodeHash = hre.ethers.keccak256(artifact.bytecode);

  const compiler = {
    version: hre.config.solidity.compilers[0].version,
    settings: hre.config.solidity.compilers[0].settings || {},
  };

  const manifest = buildManifest({
    networkName,
    chainId,
    address,
    bytecodeHash,
    txHash,
    compiler,
  });

  writeManifest(networkName, manifest);

  // Machine-readable deployment summary, printed only after confirmation.
  const summary = {
    network: networkName,
    chainId,
    contract: "PetChainRegistry",
    address,
    bytecodeHash,
    rpc: redactRpc(process.env.CELO_RPC_URL),
    deployer: redact(process.env.DEPLOYER_PRIVATE_KEY),
  };

  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${networkName} (chainId: ${chainId})`);
  console.log(`Manifest written to: ${manifestPath(networkName)}`);
  console.log(`DEPLOYMENT_SUMMARY=${JSON.stringify(summary)}`);

  return address;
}

main().catch((error) => {
  console.error(error.message || error);
  process.exitCode = 1;
});
