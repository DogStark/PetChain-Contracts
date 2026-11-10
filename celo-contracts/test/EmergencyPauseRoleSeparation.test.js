const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1499 — Celo emergency pause role separation", function () {
  let registry, admin, guardian, recovery, user;

  beforeEach(async function () {
    [admin, guardian, recovery, user] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);
  });

  it("enforces separation between admin, pauseGuardian, and recoveryAdmin", async function () {
    await expect(
      registry.connect(admin).setPauseGuardian(admin.address)
    ).to.be.revertedWith("PetChainRegistry: guardian cannot be admin");

    await expect(registry.connect(admin).setPauseGuardian(guardian.address))
      .to.emit(registry, "PauseGuardianUpdated")
      .withArgs(ethers.ZeroAddress, guardian.address);

    await expect(
      registry.connect(admin).setRecoveryAdmin(guardian.address)
    ).to.be.revertedWith("PetChainRegistry: recovery cannot be guardian");

    await expect(registry.connect(admin).setRecoveryAdmin(recovery.address))
      .to.emit(registry, "RecoveryAdminUpdated")
      .withArgs(ethers.ZeroAddress, recovery.address);

    // Pause guardian cannot perform registry admin tasks
    await expect(
      registry.connect(guardian).transferAdmin(guardian.address)
    ).to.be.revertedWith("PetChainRegistry: not admin");
  });

  it("allows only pauseGuardian to trigger emergencyPause and only recoveryAdmin to recover", async function () {
    await registry.connect(admin).setPauseGuardian(guardian.address);
    await registry.connect(admin).setRecoveryAdmin(recovery.address);

    await expect(
      registry.connect(admin).emergencyPause("incident-01")
    ).to.be.revertedWith("PetChainRegistry: not pause guardian");

    await expect(registry.connect(guardian).emergencyPause("incident-01"))
      .to.emit(registry, "EmergencyPaused")
      .withArgs(guardian.address, 1, "incident-01");

    expect(await registry.paused()).to.equal(true);
    expect(await registry.pauseEpoch()).to.equal(1);

    // Guardian and admin cannot bypass recoveryAdmin once configured
    await expect(
      registry.connect(guardian).recoverFromEmergencyPause(1, "fixed")
    ).to.be.revertedWith("PetChainRegistry: not recovery authority");
    await expect(
      registry.connect(admin).recoverFromEmergencyPause(1, "fixed")
    ).to.be.revertedWith("PetChainRegistry: not recovery authority");

    // Stale epoch is rejected
    await expect(
      registry.connect(recovery).recoverFromEmergencyPause(2, "fixed")
    ).to.be.revertedWith("PetChainRegistry: pause epoch mismatch");

    await expect(registry.connect(recovery).recoverFromEmergencyPause(1, "patched-verifier"))
      .to.emit(registry, "EmergencyRecovered")
      .withArgs(recovery.address, 1, "patched-verifier");

    expect(await registry.paused()).to.equal(false);
    expect(await registry.lastRecoveryNote()).to.equal("patched-verifier");
  });

  it("enforces bounded reason and resolutionNote lengths", async function () {
    await registry.connect(admin).setPauseGuardian(guardian.address);
    await registry.connect(admin).setRecoveryAdmin(recovery.address);

    await expect(
      registry.connect(guardian).emergencyPause("")
    ).to.be.revertedWith("PetChainRegistry: invalid pause reason length");

    await expect(
      registry.connect(guardian).emergencyPause("X".repeat(65))
    ).to.be.revertedWith("PetChainRegistry: invalid pause reason length");

    await registry.connect(guardian).emergencyPause("incident");
    await expect(
      registry.connect(recovery).recoverFromEmergencyPause(1, "")
    ).to.be.revertedWith("PetChainRegistry: invalid resolution note length");
  });
});
