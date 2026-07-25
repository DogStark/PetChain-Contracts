const hre = require("hardhat");

// Usage: CONTRACT_ADDRESS=0x... npx hardhat run scripts/register-pet.js --network <alfajores|celo|hardhat>
async function main() {
  const contractAddress = process.env.CONTRACT_ADDRESS;
  if (!contractAddress) {
    throw new Error("Set CONTRACT_ADDRESS to the deployed PetChainRegistry address");
  }

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
