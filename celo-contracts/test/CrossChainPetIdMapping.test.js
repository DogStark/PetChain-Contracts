const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1501 — Cross-chain pet identifier mapping", function () {
  let registry, admin, owner, other, vet;

  beforeEach(async function () {
    [admin, owner, other, vet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);
  });

  async function registerPet(signer = owner, name = "Rex") {
    const tx = await registry.connect(signer).registerPet(name, "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    return receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;
  }

  it("links a Celo petId to a normalized canonical PetChain identifier and resolves both ways", async function () {
    const petId = await registerPet(owner);
    await expect(registry.connect(owner).linkCrossChainPetId(petId, "  stellar:pet-001  "))
      .to.emit(registry, "CrossChainPetIdLinked")
      .withArgs(petId, "STELLAR:PET-001", 1, owner.address);

    const mapping = await registry.getCrossChainPetMapping(petId);
    expect(mapping.petId).to.equal(petId);
    expect(mapping.canonicalPetChainId).to.equal("STELLAR:PET-001");
    expect(mapping.schemaVersion).to.equal(1);
    expect(mapping.version).to.equal(1);
    expect(mapping.active).to.equal(true);
    expect(mapping.revoked).to.equal(false);

    expect(await registry.resolvePetIdByCanonicalId("stellar:pet-001")).to.equal(petId);
  });

  it("enforces uniqueness across pets and prevents duplicate active mappings on the same pet", async function () {
    const pet1 = await registerPet(owner, "Rex");
    const pet2 = await registerPet(owner, "Bella");

    await registry.connect(owner).linkCrossChainPetId(pet1, "PETCHAIN-CANON-1");

    await expect(
      registry.connect(owner).linkCrossChainPetId(pet2, "petchain-canon-1")
    ).to.be.revertedWith("PetChainRegistry: canonical id already mapped");

    await expect(
      registry.connect(owner).linkCrossChainPetId(pet1, "PETCHAIN-CANON-2")
    ).to.be.revertedWith("PetChainRegistry: pet already has active mapping");

    expect(await registry.resolvePetIdByCanonicalId("PETCHAIN-CANON-1")).to.equal(pet1);
    expect((await registry.getCrossChainPetMapping(pet2)).active).to.equal(false);
  });

  it("rejects unauthorized callers and leaves state unchanged", async function () {
    const petId = await registerPet(owner);
    await expect(
      registry.connect(other).linkCrossChainPetId(petId, "PETCHAIN-999")
    ).to.be.revertedWith("PetChainRegistry: not authorized to link");

    expect((await registry.getCrossChainPetMapping(petId)).active).to.equal(false);

    await registry.connect(admin).linkCrossChainPetId(petId, "PETCHAIN-999");
    await expect(
      registry.connect(other).revokeCrossChainPetId(petId)
    ).to.be.revertedWith("PetChainRegistry: not authorized to revoke");
    expect((await registry.getCrossChainPetMapping(petId)).active).to.equal(true);
  });

  it("revokes a mapping, blocks replay of the revoked canonical id, and allows re-linking to a fresh id", async function () {
    const pet1 = await registerPet(owner, "Rex");
    const pet2 = await registerPet(owner, "Bella");

    await registry.connect(owner).linkCrossChainPetId(pet1, "PETCHAIN-OLD");
    await expect(registry.connect(owner).revokeCrossChainPetId(pet1))
      .to.emit(registry, "CrossChainPetIdRevoked")
      .withArgs(pet1, "PETCHAIN-OLD", 1, owner.address);

    expect(await registry.isCanonicalPetIdRevoked("petchain-old")).to.equal(true);
    expect(await registry.resolvePetIdByCanonicalId("PETCHAIN-OLD")).to.equal(0);

    // Replay of revoked identifier on same or another pet must fail
    await expect(
      registry.connect(owner).linkCrossChainPetId(pet1, "PETCHAIN-OLD")
    ).to.be.revertedWith("PetChainRegistry: canonical id revoked");
    await expect(
      registry.connect(owner).linkCrossChainPetId(pet2, "PETCHAIN-OLD")
    ).to.be.revertedWith("PetChainRegistry: canonical id revoked");

    // Re-linking pet1 to a fresh canonical ID increments version
    await registry.connect(owner).linkCrossChainPetId(pet1, "PETCHAIN-NEW");
    const updated = await registry.getCrossChainPetMapping(pet1);
    expect(updated.canonicalPetChainId).to.equal("PETCHAIN-NEW");
    expect(updated.version).to.equal(2);
    expect(updated.active).to.equal(true);
  });

  it("enforces identifier length and character boundaries", async function () {
    const petId = await registerPet(owner);
    await expect(
      registry.connect(owner).linkCrossChainPetId(petId, "   ")
    ).to.be.revertedWith("PetChainRegistry: invalid identifier length");

    const maxId = "A".repeat(64);
    await registry.connect(owner).linkCrossChainPetId(petId, maxId);
    expect(await registry.resolvePetIdByCanonicalId(maxId)).to.equal(petId);

    const pet2 = await registerPet(owner, "Bella");
    await expect(
      registry.connect(owner).linkCrossChainPetId(pet2, "A".repeat(65))
    ).to.be.revertedWith("PetChainRegistry: invalid identifier length");

    await expect(
      registry.connect(owner).linkCrossChainPetId(pet2, "PET\x07BAD")
    ).to.be.revertedWith("PetChainRegistry: invalid identifier character");
    expect((await registry.getCrossChainPetMapping(pet2)).active).to.equal(false);
  });

  it("rejects linking for inactive pets and enforces pause guards without mutating state", async function () {
    const petId = await registerPet(owner, "Rex");
    await registry.connect(owner).deactivatePet(petId);

    await expect(
      registry.connect(owner).linkCrossChainPetId(petId, "PETCHAIN-INACTIVE")
    ).to.be.revertedWith("PetChainRegistry: pet inactive");
    expect(await registry.resolvePetIdByCanonicalId("PETCHAIN-INACTIVE")).to.equal(0);

    await registry.connect(owner).reactivatePet(petId);
    await registry.connect(owner).linkCrossChainPetId(petId, "PETCHAIN-PAUSE-TEST");

    await registry.connect(admin).pause();
    await expect(
      registry.connect(owner).revokeCrossChainPetId(petId)
    ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    expect((await registry.getCrossChainPetMapping(petId)).active).to.equal(true);

    await registry.connect(admin).unpause();
    await registry.connect(owner).deactivatePet(petId);
    // Revocation remains permitted after deactivation so owners/admins can tombstone retired mappings.
    await expect(registry.connect(owner).revokeCrossChainPetId(petId))
      .to.emit(registry, "CrossChainPetIdRevoked");
    expect(await registry.isCanonicalPetIdRevoked("PETCHAIN-PAUSE-TEST")).to.equal(true);
  });
});
