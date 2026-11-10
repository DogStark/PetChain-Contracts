const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1494 — Celo verified-vet capability scopes", function () {
  let registry, admin, owner, vet, recipient;
  let petId;
  const CAP_MEDICAL = 1 << 0;
  const CAP_VACCINATION = 1 << 1;
  const CAP_SURGERY = 1 << 2;
  const CAP_TRANSFER = 1 << 3;

  beforeEach(async function () {
    [admin, owner, vet, recipient] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);

    await registry.connect(vet).registerVet("LIC-CAP", "Scoped Vet");
    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;
  });

  it("enforces fine-grained capability scopes for vaccination, surgery, and transfer attestation", async function () {
    await expect(
      registry.connect(admin).grantVetCapabilities(vet.address, CAP_VACCINATION, 0)
    )
      .to.emit(registry, "VetCapabilitiesGranted")
      .withArgs(vet.address, CAP_VACCINATION, 0, 1);

    // Allowed: Vaccination (RecordType = 1)
    await registry.connect(vet).addMedicalRecord(petId, 1, "rabies", "vaccine", "");

    // Blocked: Checkup (RecordType = 0) and Surgery (RecordType = 2)
    await expect(
      registry.connect(vet).addMedicalRecord(petId, 0, "checkup", "exam", "")
    ).to.be.revertedWith("PetChainRegistry: missing vet capability");
    await expect(
      registry.connect(vet).addMedicalRecord(petId, 2, "spay", "surgery", "")
    ).to.be.revertedWith("PetChainRegistry: missing vet capability");

    // Blocked: Transfer attestation
    const hash = ethers.keccak256(ethers.toUtf8Bytes("fit-for-transfer"));
    await expect(
      registry.connect(vet).attestPetTransferHealth(petId, recipient.address, hash, 3600)
    ).to.be.revertedWith("PetChainRegistry: missing vet capability");

    // Grant transfer capability and attest
    await registry.connect(admin).grantVetCapabilities(vet.address, CAP_VACCINATION | CAP_TRANSFER, 0);
    await expect(
      registry.connect(vet).attestPetTransferHealth(petId, recipient.address, hash, 3600)
    ).to.emit(registry, "PetTransferHealthAttested");
  });

  it("revokes individual capabilities and enforces capability expiry", async function () {
    const now = (await ethers.provider.getBlock("latest")).timestamp;
    await registry.connect(admin).grantVetCapabilities(
      vet.address,
      CAP_MEDICAL | CAP_SURGERY,
      now + 100
    );

    await registry.connect(admin).revokeVetCapabilities(vet.address, CAP_SURGERY);
    expect(await registry.hasVetCapability(vet.address, CAP_SURGERY)).to.equal(false);
    expect(await registry.hasVetCapability(vet.address, CAP_MEDICAL)).to.equal(true);

    await network.provider.send("evm_increaseTime", [105]);
    await network.provider.send("evm_mine");

    expect(await registry.hasVetCapability(vet.address, CAP_MEDICAL)).to.equal(false);
    await expect(
      registry.connect(vet).addMedicalRecord(petId, 0, "checkup", "exam", "")
    ).to.be.revertedWith("PetChainRegistry: missing vet capability");
  });
});
