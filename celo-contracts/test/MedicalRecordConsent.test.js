const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1497 — Celo consent checks for record reads", function () {
  let registry, admin, owner, vet, reader, stranger;
  let petId;
  const Vaccination = 1;
  const Surgery = 2;
  const SCOPE_VACCINATION = 1 << 1;
  const SCOPE_ALL = 0x1f;

  beforeEach(async function () {
    [admin, owner, vet, reader, stranger] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);

    await registry.connect(vet).registerVet("LIC-001", "General");
    await registry.connect(admin).verifyVet(vet.address);

    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;

    await registry.connect(vet).addMedicalRecord(petId, Vaccination, "rabies", "shot", "");
    await registry.connect(vet).addMedicalRecord(petId, Surgery, "spay", "suture", "");
  });

  it("enforces owner-controlled scopes across all public medical record retrieval methods", async function () {
    await expect(
      registry.connect(owner).grantRecordConsent(petId, reader.address, SCOPE_VACCINATION, 3600)
    )
      .to.emit(registry, "RecordConsentGranted");

    // Reader with Vaccination scope can read Vaccination records, but not Surgery or unfiltered getPetRecords
    const vax = await registry.connect(reader).getPetRecordsByType(petId, Vaccination);
    expect(vax.length).to.equal(1);

    await expect(
      registry.connect(reader).getPetRecordsByType(petId, Surgery)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");

    await expect(
      registry.connect(reader).getPetRecords(petId)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");

    await expect(
      registry.connect(reader).getPetRecordsPaged(petId, 0, 10)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");

    await expect(
      registry.connect(reader).getPetRecordsByDateRange(petId, 0, 9999999999)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");

    // Owner always retains full access
    expect((await registry.connect(owner).getPetRecords(petId)).length).to.equal(2);

    // Granting SCOPE_ALL unlocks full reads for reader
    await registry.connect(owner).grantRecordConsent(petId, reader.address, SCOPE_ALL, 3600);
    expect((await registry.connect(reader).getPetRecords(petId)).length).to.equal(2);
    expect((await registry.connect(reader).getPetRecordsPaged(petId, 0, 10)).length).to.equal(2);
  });

  it("rejects reads after consent revocation or expiry", async function () {
    await registry.connect(owner).grantRecordConsent(petId, reader.address, SCOPE_ALL, 100);
    expect((await registry.connect(reader).getPetRecords(petId)).length).to.equal(2);

    await network.provider.send("evm_increaseTime", [101]);
    await network.provider.send("evm_mine");

    await expect(
      registry.connect(reader).getPetRecords(petId)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");

    await registry.connect(owner).grantRecordConsent(petId, reader.address, SCOPE_ALL, 3600);
    await registry.connect(owner).revokeRecordConsent(petId, reader.address);

    await expect(
      registry.connect(reader).getPetRecords(petId)
    ).to.be.revertedWith("PetChainRegistry: missing record read consent");
  });
});
