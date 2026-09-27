const hre = require("hardhat");

// Usage: CONTRACT_ADDRESS=0x... npx hardhat run scripts/register-pet.js --network <alfajores|celo|hardhat>
//
// Optional env overrides:
//   OWNER_ADDRESS  - owner of the pet record (defaults to the signer)
//   VET_ADDRESS    - vet address recorded for the pet (defaults to the signer)
//   CHIP_ID        - unique chip identifier (defaults to a deterministic value)
async function main() {
  const contractAddress = process.env.CONTRACT_ADDRESS;
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
