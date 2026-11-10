const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1495 — Celo medical record amendment chain", function () {
  let registry, admin, owner, vet, otherVet;
  let petId, recordId;

  beforeEach(async function () {
    [admin, owner, vet, otherVet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);

    await registry.connect(vet).registerVet("LIC-001", "General");
    await registry.connect(admin).verifyVet(vet.address);
    await registry.connect(otherVet).registerVet("LIC-002", "Surgery");
    await registry.connect(admin).verifyVet(otherVet.address);

    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find((l) => l.fragment && l.fragment.name === "PetRegistered").args.petId;

    const rTx = await registry.connect(vet).addMedicalRecord(petId, 0, "orig-diag", "orig-treat", "orig-note");
    const rRec = await rTx.wait();
    recordId = rRec.logs.find((l) => l.fragment && l.fragment.name === "MedicalRecordAdded").args.recordId;
  });

  it("appends chained correction records without mutating the original medical record", async function () {
    await expect(
      registry.connect(vet).appendMedicalRecordAmendment(
        recordId,
        "amended-diag-1",
        "amended-treat-1",
        "amended-note-1",
        "typo in dosage"
      )
    ).to.emit(registry, "MedicalRecordAmended");

    await registry.connect(admin).appendMedicalRecordAmendment(
      recordId,
      "amended-diag-2",
      "amended-treat-2",
      "amended-note-2",
      "lab addendum"
    );

    // Original record remains completely untouched
    const records = await registry.getPetRecords(petId);
    expect(records[0].diagnosis).to.equal("orig-diag");
    expect(records[0].treatment).to.equal("orig-treat");
    expect(records[0].notes).to.equal("orig-note");

    const chain = await registry.getMedicalRecordAmendments(recordId, 0, 10);
    expect(chain.length).to.equal(2);
    expect(chain[0].originalRecordId).to.equal(recordId);
    expect(chain[0].parentAmendmentId).to.equal(0);
    expect(chain[0].author).to.equal(vet.address);
    expect(chain[0].diagnosis).to.equal("amended-diag-1");

    expect(chain[1].parentAmendmentId).to.equal(chain[0].amendmentId);
    expect(chain[1].author).to.equal(admin.address);
    expect(chain[1].diagnosis).to.equal("amended-diag-2");
  });

  it("rejects unauthorized authors, revoked vets, and invalid field bounds", async function () {
    await expect(
      registry.connect(otherVet).appendMedicalRecordAmendment(recordId, "d", "t", "n", "r")
    ).to.be.revertedWith("PetChainRegistry: not authorized to amend");

    await expect(
      registry.connect(vet).appendMedicalRecordAmendment(recordId, "d", "t", "n", "")
    ).to.be.revertedWith("PetChainRegistry: invalid reason length");

    await registry.connect(admin).revokeVet(vet.address);
    await expect(
      registry.connect(vet).appendMedicalRecordAmendment(recordId, "d", "t", "n", "reason")
    ).to.be.revertedWith("PetChainRegistry: not authorized to amend");
  });
});
