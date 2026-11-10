const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1498 — Celo cursor-based paginated pet medical history", function () {
  let registry, admin, owner, vet;
  let petId;

  beforeEach(async function () {
    [admin, owner, vet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);

    await registry.connect(vet).registerVet("LIC-001", "General");
    await registry.connect(admin).verifyVet(vet.address);

    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;

    for (let i = 1; i <= 5; i++) {
      await registry.connect(vet).addMedicalRecord(petId, 0, `diag-${i}`, `treat-${i}`, `note-${i}`);
    }
  });

  it("paginates forward (oldest-first) deterministically with cursors", async function () {
    const p1 = await registry.getPetMedicalHistoryCursor(petId, 0, 2, false);
    expect(p1.totalCount).to.equal(5);
    expect(p1.records.map((r) => Number(r.recordId))).to.deep.equal([1, 2]);
    expect(p1.hasMore).to.equal(true);
    expect(p1.nextCursor).to.equal(2);

    const p2 = await registry.getPetMedicalHistoryCursor(petId, p1.nextCursor, 2, false);
    expect(p2.records.map((r) => Number(r.recordId))).to.deep.equal([3, 4]);
    expect(p2.hasMore).to.equal(true);
    expect(p2.nextCursor).to.equal(4);

    const p3 = await registry.getPetMedicalHistoryCursor(petId, p2.nextCursor, 2, false);
    expect(p3.records.map((r) => Number(r.recordId))).to.deep.equal([5]);
    expect(p3.hasMore).to.equal(false);
    expect(p3.nextCursor).to.equal(0);
  });

  it("paginates backward (newest-first) deterministically with cursors", async function () {
    const p1 = await registry.getPetMedicalHistoryCursor(petId, 0, 2, true);
    expect(p1.records.map((r) => Number(r.recordId))).to.deep.equal([5, 4]);
    expect(p1.hasMore).to.equal(true);
    expect(p1.nextCursor).to.equal(4);

    const p2 = await registry.getPetMedicalHistoryCursor(petId, p1.nextCursor, 3, true);
    expect(p2.records.map((r) => Number(r.recordId))).to.deep.equal([3, 2, 1]);
    expect(p2.hasMore).to.equal(false);
    expect(p2.nextCursor).to.equal(0);
  });

  it("enforces page size bounds and rejects foreign or unknown cursors", async function () {
    await expect(
      registry.getPetMedicalHistoryCursor(petId, 0, 0, false)
    ).to.be.revertedWith("PetChainRegistry: invalid page limit");

    await expect(
      registry.getPetMedicalHistoryCursor(petId, 0, 51, false)
    ).to.be.revertedWith("PetChainRegistry: invalid page limit");

    await expect(
      registry.getPetMedicalHistoryCursor(petId, 999, 2, false)
    ).to.be.revertedWith("PetChainRegistry: invalid cursor");
  });
});
