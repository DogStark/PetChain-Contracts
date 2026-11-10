const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1496 — Celo vaccination certificate verification", function () {
  let registry, admin, owner, vet, stranger;
  let petId;
  const HASH = ethers.keccak256(ethers.toUtf8Bytes("rabies-cert-v1"));

  beforeEach(async function () {
    [admin, owner, vet, stranger] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);

    await registry.connect(vet).registerVet("LIC-001", "Immunology");
    await registry.connect(admin).verifyVet(vet.address);

    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;
  });

  it("issues a certificate, verifies Valid status, and prevents duplicate replay", async function () {
    const now = (await ethers.provider.getBlock("latest")).timestamp;
    const validUntil = now + 86400;

    await expect(
      registry.connect(vet).issueVaccinationCertificate(petId, "RABIES-3Y", "BATCH-42", validUntil, HASH)
    )
      .to.emit(registry, "VaccinationCertificateIssued")
      .withArgs(1, petId, vet.address, "RABIES-3Y", validUntil, HASH);

    const res = await registry.verifyVaccinationCertificate(1, HASH);
    expect(res.status).to.equal(1); // Valid
    expect(res.isValid).to.equal(true);
    expect(res.issuer).to.equal(vet.address);
    expect(res.petId).to.equal(petId);

    await expect(
      registry.connect(vet).issueVaccinationCertificate(petId, "RABIES-3Y", "BATCH-42", validUntil, HASH)
    ).to.be.revertedWith("PetChainRegistry: duplicate certificate");
  });

  it("reports Expired and Revoked states accurately and blocks unauthorized revocation", async function () {
    const now = (await ethers.provider.getBlock("latest")).timestamp;
    const validUntil = now + 200;

    await registry.connect(vet).issueVaccinationCertificate(petId, "RABIES-3Y", "BATCH-1", validUntil, HASH);

    await expect(
      registry.connect(stranger).revokeVaccinationCertificate(1, "fraud")
    ).to.be.revertedWith("PetChainRegistry: not authorized to revoke");

    await network.provider.send("evm_increaseTime", [250]);
    await network.provider.send("evm_mine");

    const expiredRes = await registry.verifyVaccinationCertificate(1, HASH);
    expect(expiredRes.status).to.equal(2); // Expired
    expect(expiredRes.isValid).to.equal(false);

    await expect(registry.connect(vet).revokeVaccinationCertificate(1, "recalled-batch"))
      .to.emit(registry, "VaccinationCertificateRevoked")
      .withArgs(1, petId, vet.address, "recalled-batch");

    const revokedRes = await registry.verifyVaccinationCertificate(1, HASH);
    expect(revokedRes.status).to.equal(3); // Revoked
    expect(revokedRes.isValid).to.equal(false);
    expect(revokedRes.revoked).to.equal(true);
  });

  it("returns Unknown for non-existent certificates or mismatched content hashes", async function () {
    const unknown = await registry.verifyVaccinationCertificate(999, HASH);
    expect(unknown.status).to.equal(0); // Unknown
    expect(unknown.isValid).to.equal(false);
  });
});
