const hre = require("hardhat");

async function main() {
  const [deployer] = await hre.ethers.getSigners();
  const admin = process.env.ADMIN_ADDRESS || deployer.address;
  const expectedChainId = hre.network.config.chainId;
  if (!expectedChainId) {
    throw new Error(`Network ${hre.network.name} has no chainId configured`);
  }

  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy(admin, expectedChainId);
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${hre.network.name} (chainId ${expectedChainId}), admin: ${admin}`);

  return address;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
