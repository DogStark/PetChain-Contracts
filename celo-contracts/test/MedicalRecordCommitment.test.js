const { expect } = require("chai");
const { ethers } = require("hardhat");
const fc = require("fast-check");

// Bounded-collection limits mirrored from PetChainRegistry.
const MAX_DIAGNOSIS_BYTES = 1000;
const MAX_TREATMENT_BYTES = 1000;
const MAX_NOTES_BYTES = 1000;
const MAX_RECORD_TYPE_BYTES = 64;

// Documented proptest case count for CI (see celo-contracts/README.md).
const FUZZ_CASES = 100;

// fast-check configuration that prints a reproducible seed and minimized case.
const FC_CONFIG = { numRuns: FUZZ_CASES, verbose: true };

function byteLength(value) {
  return ethers.toUtf8Bytes(value).length;
}

// Generators for boundary, empty, Unicode, and oversized bounded strings.
const boundaryString = (maxBytes) =>
  fc.string({ maxLength: maxBytes }).map((s) => {
    const bytes = ethers.toUtf8Bytes(s);
    return bytes.length <= maxBytes ? s : ethers.toUtf8String(bytes.slice(0, maxBytes));
  });

const oversizedString = (maxBytes) =>
  fc.string({ minLength: 1, maxLength: 32 }).map((s) => s + "x".repeat(maxBytes + 1));

const unicodeString = (maxBytes) =>
  fc.array(fc.constantFrom("\u00e9", "\u0301", "\u4e2d", "\u{1f600}", "a"), { maxLength: 16 })
    .map((parts) => parts.join(""))
    .filter((s) => byteLength(s) <= maxBytes);

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

  it("fuzzes bounded strings: oversized values always fail before storage work", async function () {
    const { recordId, record, commitment } = await addRecord();
    await fc.assert(
      fc.asyncProperty(
        oversizedString(MAX_DIAGNOSIS_BYTES),
        oversizedString(MAX_NOTES_BYTES),
        async (diagnosis, notes) => {
          expect(await verify(record, recordId, commitment, { diagnosis })).to.equal(false);
          expect(await verify(record, recordId, commitment, { notes })).to.equal(false);
        }
      ),
      FC_CONFIG
    );
  });

  it("fuzzes boundary and empty values against endpoint policy", async function () {
    const { recordId, record, commitment } = await addRecord();
    await fc.assert(
      fc.asyncProperty(
        boundaryString(MAX_DIAGNOSIS_BYTES),
        boundaryString(MAX_NOTES_BYTES),
        async (diagnosis, notes) => {
          // Empty diagnosis is rejected by policy; non-empty bounded values are accepted.
          const expected = diagnosis.length > 0;
          expect(await verify(record, recordId, commitment, { diagnosis, notes })).to.equal(expected);
        }
      ),
      FC_CONFIG
    );
  });

  it("fuzzes Unicode inputs so normalization cannot bypass byte limits", async function () {
    const { recordId, record, commitment } = await addRecord();
    await fc.assert(
      fc.asyncProperty(
        unicodeString(MAX_DIAGNOSIS_BYTES),
        unicodeString(MAX_NOTES_BYTES),
        async (diagnosis, notes) => {
          expect(byteLength(diagnosis)).to.be.at.most(MAX_DIAGNOSIS_BYTES);
          expect(byteLength(notes)).to.be.at.most(MAX_NOTES_BYTES);
          const expected = diagnosis.length > 0;
          expect(await verify(record, recordId, commitment, { diagnosis, notes })).to.equal(expected);
        }
      ),
      FC_CONFIG
    );
  });

  it("fuzzes recordType byte limits on the bounded vector input", async function () {
    const { recordId, record, commitment } = await addRecord();
    await fc.assert(
      fc.asyncProperty(
        boundaryString(MAX_RECORD_TYPE_BYTES),
        oversizedString(MAX_RECORD_TYPE_BYTES),
        async (bounded, oversized) => {
          expect(await verify(record, recordId, commitment, { recordType: bounded })).to.equal(true);
          expect(await verify(record, recordId, commitment, { recordType: oversized })).to.equal(false);
        }
      ),
      FC_CONFIG
    );
  });
});
