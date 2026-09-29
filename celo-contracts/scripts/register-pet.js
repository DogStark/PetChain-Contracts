const fs = require("fs");
const path = require("path");
const hre = require("hardhat");

// Usage: npx hardhat run scripts/register-pet.js --network <alfajores|celo|hardhat>
// The deployed address is read from the chain-specific deployment manifest.
//
// Optional env overrides:
//   CONTRACT_ADDRESS - overrides the address from the deployment manifest
//   OWNER_ADDRESS    - owner of the pet record (defaults to the signer)
//   VET_ADDRESS      - vet address recorded for the pet (defaults to the signer)
//   CHIP_ID          - unique chip identifier (defaults to a deterministic value)
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
  if (!contractAddress) {
    throw new Error("Set CONTRACT_ADDRESS to the deployed PetChainRegistry address");
  }

  const [signer] = await hre.ethers.getSigners();
  const registry = await hre.ethers.getContractAt("PetChainRegistry", contractAddress);

  const owner = process.env.OWNER_ADDRESS || signer.address;
  const vet = process.env.VET_ADDRESS || signer.address;
  const chipId = process.env.CHIP_ID || `chip-${Date.now()}`;

  // Mirror the registry's trust-boundary validation so invalid inputs fail
  // fast with a clear message instead of an opaque revert.
  if (!hre.ethers.isAddress(owner) || owner === hre.ethers.ZeroAddress) {
    throw new Error("OWNER_ADDRESS must be a non-zero address");
  }
  if (!hre.ethers.isAddress(vet) || vet === hre.ethers.ZeroAddress) {
    throw new Error("VET_ADDRESS must be a non-zero address");
  }
  if (!chipId || chipId.trim().length === 0) {
    throw new Error("CHIP_ID must be a non-empty identifier");
  }

  const tx = await registry.registerPet(owner, vet, chipId, "Rex", "Dog", "Labrador", "2020-01-01");
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

  if (!event) {
    throw new Error("PetRegistered event not found in transaction receipt");
  }

  console.log(`Pet registered with petId: ${event.args.petId}`);
  console.log(`  owner: ${event.args.owner}`);
  console.log(`  chipId: ${event.args.chipId}`);
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
