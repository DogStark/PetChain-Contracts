const hre = require("hardhat");

async function main() {
  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${hre.network.name}`);

  return address;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
