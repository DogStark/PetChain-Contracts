const hre = require("hardhat");

// Networks that require a fully validated production configuration before any
// transaction is signed. Local development networks are intentionally excluded
// so `hardhat node` / `hardhat test` remain usable without extra env vars.
const PRODUCTION_NETWORKS = ["celo", "celo-mainnet", "celo-alfajores", "alfajores"];

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

async function main() {
  const networkName = hre.network.name;
  assertDeployable(networkName);

  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  const chainId = Number((await hre.ethers.provider.getNetwork()).chainId);

  // Machine-readable deployment summary, printed only after confirmation.
  const summary = {
    network: networkName,
    chainId,
    contract: "PetChainRegistry",
    address,
    rpc: redactRpc(process.env.CELO_RPC_URL),
    deployer: redact(process.env.DEPLOYER_PRIVATE_KEY),
  };

  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${networkName} (chainId: ${chainId})`);
  console.log(`DEPLOYMENT_SUMMARY=${JSON.stringify(summary)}`);

  return address;
}

main().catch((error) => {
  console.error(error.message || error);
  process.exitCode = 1;
});
