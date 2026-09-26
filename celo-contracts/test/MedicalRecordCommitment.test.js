const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PetChainRegistry medical-record commitments", function () {
  let registry, admin, owner, other, vet, petId;

  beforeEach(async function () {
    [admin, owner, other, vet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy();
    await registry.connect(vet).registerVet("LIC-COMMIT", "General Practice");
    await registry.connect(admin).verifyVet(vet.address);
    const tx = await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    const receipt = await tx.wait();
    petId = receipt.logs.find(log => log.fragment?.name === "PetRegistered").args.petId;
  });

  async function addRecord() {
    const tx = await registry.connect(vet).addMedicalRecord(petId, 1, "rabies", "vaccination", "annual");
    const receipt = await tx.wait();
    const recordId = receipt.logs.find(log => log.fragment?.name === "MedicalRecordAdded").args.recordId;
    const record = (await registry.getPetRecords(petId))[0];
    return { recordId, record, commitment: await registry.medicalRecordCommitments(recordId) };
  }

  async function verify(record, recordId, commitment, overrides = {}) {
    return registry.connect(other).verifyMedicalRecordCommitment(
      recordId,
      overrides.version ?? 1,
      overrides.petId ?? record.petId,
      overrides.vet ?? record.vet,
      overrides.recordType ?? record.recordType,
      overrides.diagnosis ?? record.diagnosis,
      overrides.treatment ?? record.treatment,
      overrides.notes ?? record.notes,
      overrides.timestamp ?? record.timestamp,
      commitment
    );
  }

  it("matches a known canonical vector", async function () {
    const { recordId, record, commitment } = await addRecord();
    const domain = await registry.MEDICAL_RECORD_COMMITMENT_DOMAIN();
    const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
      ["bytes32", "uint8", "uint256", "uint256", "address", "uint8", "string", "string", "string", "uint256"],
      [domain, 1, recordId, petId, vet.address, 1, "rabies", "vaccination", "annual", record.timestamp]
    );
    expect(commitment).to.equal(ethers.keccak256(encoded));
    expect(await verify(record, recordId, commitment)).to.equal(true);
  });

  it("rejects altered fields and commitment versions", async function () {
    const { recordId, record, commitment } = await addRecord();
    expect(await verify(record, recordId, commitment, { notes: "altered" })).to.equal(false);
    expect(await verify(record, recordId, commitment, { version: 2 })).to.equal(false);
  });

  it("returns false for malformed or oversized input without reverting", async function () {
    const { recordId, record, commitment } = await addRecord();
    expect(await verify(record, recordId, commitment, { diagnosis: "" })).to.equal(false);
    expect(await verify(record, recordId, commitment, { notes: "x".repeat(1001) })).to.equal(false);
    expect(await registry.connect(other).verifyMedicalRecordCommitment(
      999, 1, petId, vet.address, 1, "rabies", "vaccination", "annual", record.timestamp, commitment
    )).to.equal(false);
  });

  it("is permissionless, while record correction remains authorized", async function () {
    const { recordId, record, commitment } = await addRecord();
    expect(await verify(record, recordId, commitment)).to.equal(true);
    await expect(registry.connect(other).correctMedicalRecord(recordId, "hack", "hack", ""))
      .to.be.revertedWith("PetChainRegistry: not authorized");
  });

  it("is append-only: duplicate commitment writes are deterministic and do not overwrite history", async function () {
    const { recordId, record, commitment } = await addRecord();
    const before = await registry.medicalRecordCommitments(recordId);
    const versionBefore = await registry.medicalRecordCommitmentVersion(recordId);

    // Re-submitting the identical commitment must be a deterministic no-op.
    await expect(registry.connect(vet).commitMedicalRecord(recordId, commitment))
      .to.not.be.reverted;

    expect(await registry.medicalRecordCommitments(recordId)).to.equal(before);
    expect(await registry.medicalRecordCommitmentVersion(recordId)).to.equal(versionBefore);
    expect(await verify(record, recordId, commitment)).to.equal(true);
  });

  it("rejects a replacement that silently changes the pet or submitter association", async function () {
    const { recordId, record, commitment } = await addRecord();

    // A replacement commitment bound to a different pet must be rejected.
    const domain = await registry.MEDICAL_RECORD_COMMITMENT_DOMAIN();
    const wrongPet = ethers.AbiCoder.defaultAbiCoder().encode(
      ["bytes32", "uint8", "uint256", "uint256", "address", "uint8", "string", "string", "string", "uint256"],
      [domain, 1, recordId, petId + 1n, vet.address, 1, "rabies", "vaccination", "annual", record.timestamp]
    );
    await expect(registry.connect(vet).commitMedicalRecord(recordId, ethers.keccak256(wrongPet)))
      .to.be.revertedWith("PetChainRegistry: commitment mismatch");

    // A replacement submitted by an unauthorized signer must be rejected.
    await expect(registry.connect(other).commitMedicalRecord(recordId, commitment))
      .to.be.revertedWith("PetChainRegistry: not authorized");

    // The original commitment and association remain intact.
    expect(await registry.medicalRecordCommitments(recordId)).to.equal(commitment);
    expect(await verify(record, recordId, commitment)).to.equal(true);
  });

  it("emits versioned, timestamped events so clients can reconstruct commitment history", async function () {
    const { recordId, record, commitment } = await addRecord();

    const tx = await registry.connect(vet).commitMedicalRecord(recordId, commitment);
    const receipt = await tx.wait();
    const event = receipt.logs.find(log => log.fragment?.name === "MedicalRecordCommitmentUpdated");
    expect(event).to.not.equal(undefined);
    expect(event.args.recordId).to.equal(recordId);
    expect(event.args.petId).to.equal(record.petId);
    expect(event.args.submitter).to.equal(vet.address);
    expect(event.args.commitment).to.equal(commitment);
    expect(event.args.version).to.equal(1);
    expect(event.args.timestamp).to.equal(record.timestamp);

    // Replaying the same commitment yields a deterministic, ordered history.
    const replay = await (await registry.connect(vet).commitMedicalRecord(recordId, commitment)).wait();
    const replayEvent = replay.logs.find(log => log.fragment?.name === "MedicalRecordCommitmentUpdated");
    expect(replayEvent.args.version).to.equal(event.args.version);
    expect(replayEvent.args.commitment).to.equal(event.args.commitment);
  });
});
