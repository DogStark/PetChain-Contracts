const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1500 — Celo ownership transfer acceptance & history", function () {
  let registry, admin, owner, recipient, stranger;

  beforeEach(async function () {
    [admin, owner, recipient, stranger] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);
  });

  async function registerPet(signer = owner) {
    const tx = await registry.connect(signer).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    return receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;
  }

  it("requires recipient acceptance before changing owner and records verifiable ownership history", async function () {
    const petId = await registerPet(owner);

    await expect(registry.connect(owner).proposePetTransfer(petId, recipient.address, 3600))
      .to.emit(registry, "PetTransferProposed");

    // Owner has not changed before acceptance
    expect((await registry.pets(petId)).owner).to.equal(owner.address);

    await expect(registry.connect(recipient).acceptPetTransfer(petId))
      .to.emit(registry, "PetTransferAccepted")
      .withArgs(petId, owner.address, recipient.address, 1);

    expect((await registry.pets(petId)).owner).to.equal(recipient.address);
    expect(await registry.getPetsByOwner(owner.address)).to.deep.equal([]);
    expect(await registry.getPetsByOwner(recipient.address)).to.deep.equal([petId]);

    const history = await registry.getPetOwnershipHistory(petId, 0, 10);
    expect(history.length).to.equal(1);
    expect(history[0].previousOwner).to.equal(owner.address);
    expect(history[0].newOwner).to.equal(recipient.address);
    expect(history[0].schemaVersion).to.equal(1);
    expect(history[0].transferSequence).to.equal(1);
  });

  it("rejects unauthorized proposers, wrong acceptors, and replay of accepted transfers", async function () {
    const petId = await registerPet(owner);

    await expect(
      registry.connect(stranger).proposePetTransfer(petId, recipient.address, 3600)
    ).to.be.revertedWith("PetChainRegistry: not pet owner");

    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 3600);

    await expect(
      registry.connect(stranger).acceptPetTransfer(petId)
    ).to.be.revertedWith("PetChainRegistry: not transfer recipient");

    await registry.connect(recipient).acceptPetTransfer(petId);

    await expect(
      registry.connect(recipient).acceptPetTransfer(petId)
    ).to.be.revertedWith("PetChainRegistry: no pending transfer");
  });

  it("expires pending transfers after ttlSeconds and allows cancellation", async function () {
    const petId = await registerPet(owner);
    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 120);

    await network.provider.send("evm_increaseTime", [121]);
    await network.provider.send("evm_mine");

    await expect(
      registry.connect(recipient).acceptPetTransfer(petId)
    ).to.be.revertedWith("PetChainRegistry: transfer expired");
    expect((await registry.pets(petId)).owner).to.equal(owner.address);

    // Owner can cancel or re-propose
    await expect(registry.connect(owner).cancelPetTransfer(petId))
      .to.emit(registry, "PetTransferCancelled")
      .withArgs(petId, owner.address, recipient.address, 1);

    await expect(
      registry.connect(recipient).acceptPetTransfer(petId)
    ).to.be.revertedWith("PetChainRegistry: no pending transfer");
  });

  it("enforces TTL bounds", async function () {
    const petId = await registerPet(owner);
    await expect(
      registry.connect(owner).proposePetTransfer(petId, recipient.address, 59)
    ).to.be.revertedWith("PetChainRegistry: invalid transfer ttl");

    await expect(
      registry.connect(owner).proposePetTransfer(petId, recipient.address, 31 * 24 * 3600)
    ).to.be.revertedWith("PetChainRegistry: invalid transfer ttl");

    // Exact boundary values succeed
    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 60);
    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 30 * 24 * 3600);
    expect((await registry.pendingPetTransfers(petId)).nonce).to.equal(2);
  });

  it("invalidates pending offers on direct transfer (ABA replay protection), paginates history, and enforces pause", async function () {
    const petId = await registerPet(owner);

    await expect(
      registry.connect(owner).proposePetTransfer(petId, ethers.ZeroAddress, 3600)
    ).to.be.revertedWith("PetChainRegistry: zero address");
    await expect(
      registry.connect(owner).proposePetTransfer(petId, owner.address, 3600)
    ).to.be.revertedWith("PetChainRegistry: self transfer");

    // Owner proposes to recipient, then directly transfers pet to stranger and stranger transfers back to owner (ABA)
    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 3600);
    await registry.connect(owner).transferPet(petId, stranger.address);
    await registry.connect(stranger).transferPet(petId, owner.address);

    // Stale offer from before the ABA cycle cannot be accepted
    await expect(
      registry.connect(recipient).acceptPetTransfer(petId)
    ).to.be.revertedWith("PetChainRegistry: no pending transfer");

    // Record two two-step transfers to verify history pagination
    await registry.connect(owner).proposePetTransfer(petId, recipient.address, 3600);
    await registry.connect(admin).pause();
    await expect(
      registry.connect(recipient).acceptPetTransfer(petId)
    ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    await registry.connect(admin).unpause();

    await registry.connect(recipient).acceptPetTransfer(petId);
    await registry.connect(recipient).proposePetTransfer(petId, stranger.address, 3600);
    await registry.connect(stranger).acceptPetTransfer(petId);

    const page1 = await registry.getPetOwnershipHistory(petId, 0, 1);
    const page2 = await registry.getPetOwnershipHistory(petId, 1, 5);
    const empty = await registry.getPetOwnershipHistory(petId, 5, 5);
    expect(page1.length).to.equal(1);
    expect(page1[0].transferSequence).to.equal(1);
    expect(page2.length).to.equal(1);
    expect(page2[0].transferSequence).to.equal(2);
    expect(page2[0].newOwner).to.equal(stranger.address);
    expect(empty.length).to.equal(0);
  });
});
