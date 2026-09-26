const fs = require("fs");
const path = require("path");
const hre = require("hardhat");

// Usage: npx hardhat run scripts/register-pet.js --network <alfajores|celo|hardhat>
// The deployed address is read from the chain-specific deployment manifest.
const MANIFEST_PATH = path.join(__dirname, "..", "deployments", "PetChainRegistry.json");

function loadManifest() {
  if (!fs.existsSync(MANIFEST_PATH)) {
    throw new Error(
      `Missing deployment manifest at ${MANIFEST_PATH}. Deploy the contract first.`
    );
  }

  let manifest;
  try {
    manifest = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8"));
  } catch (error) {
    throw new Error(`Stale or malformed deployment manifest: ${error.message}`);
  }

  if (!manifest || !manifest.address || !manifest.chainId) {
    throw new Error("Deployment manifest is missing address or chainId");
  }

  return manifest;
}

async function main() {
  const manifest = loadManifest();

  const network = await hre.ethers.provider.getNetwork();
  const activeChainId = Number(network.chainId);
  const manifestChainId = Number(manifest.chainId);

  if (activeChainId !== manifestChainId) {
    throw new Error(
      `Manifest chainId ${manifestChainId} does not match active chainId ${activeChainId}. ` +
        "Refusing to use a manifest from a different chain."
    );
  }

  const contractAddress = process.env.CONTRACT_ADDRESS || manifest.address;

  const registry = await hre.ethers.getContractAt("PetChainRegistry", contractAddress);

  const tx = await registry.registerPet("Rex", "Dog", "Labrador", "2020-01-01");
  const receipt = await tx.wait();

  const event = receipt.logs
    .map((log) => {
      try {
        return registry.interface.parseLog(log);
      } catch {
        return null;
      }
    })
    .find((parsed) => parsed && parsed.name === "PetRegistered");

  console.log(`Pet registered with petId: ${event.args.petId}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
