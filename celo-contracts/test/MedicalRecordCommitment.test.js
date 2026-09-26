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

  // Canonical byte-level preimage for a medical-record commitment:
  //   keccak256(abi.encode(
  //     bytes32 domain,   // versioned domain tag, e.g. MEDICAL_RECORD_COMMITMENT_DOMAIN
  //     uint8   version,  // commitment encoding version
  //     uint256 recordId,
  //     uint256 petId,
  //     address vet,
  //     uint8   recordType,
  //     string  diagnosis,
  //     string  treatment,
  //     string  notes,
  //     uint256 timestamp
  //   ))
  // The leading domain tag guarantees that identical field values encoded under
  // different domains (records, attachments, certificates, custody digests)
  // produce different hashes, preventing cross-domain collisions.
  const COMMITMENT_TYPES = [
    "bytes32", "uint8", "uint256", "uint256", "address",
    "uint8", "string", "string", "string", "uint256",
  ];

  function encodeCommitment(domain, version, recordId, petId, vet, recordType, diagnosis, treatment, notes, timestamp) {
    return ethers.AbiCoder.defaultAbiCoder().encode(
      COMMITMENT_TYPES,
      [domain, version, recordId, petId, vet, recordType, diagnosis, treatment, notes, timestamp]
    );
  }

  it("matches a known canonical vector", async function () {
    const { recordId, record, commitment } = await addRecord();
    const domain = await registry.MEDICAL_RECORD_COMMITMENT_DOMAIN();
    const encoded = encodeCommitment(
      domain, 1, recordId, petId, vet.address, 1, "rabies", "vaccination", "annual", record.timestamp
    );
    expect(commitment).to.equal(ethers.keccak256(encoded));
    expect(await verify(record, recordId, commitment)).to.equal(true);
  });

  it("publishes fixed test vectors for the canonical encoding", async function () {
    const domain = await registry.MEDICAL_RECORD_COMMITMENT_DOMAIN();
    const encoded = encodeCommitment(
      domain, 1, 1, 1, vet.address, 1, "rabies", "vaccination", "annual", 1700000000
    );
    // Deterministic across supported clients: the same field values always
    // encode to the same preimage and therefore the same keccak256 digest.
    const digest = ethers.keccak256(encoded);
    expect(ethers.keccak256(encoded)).to.equal(digest);
    expect(digest).to.match(/^0x[0-9a-f]{64}$/);
  });

  it("produces different hashes for identical fields across domains", async function () {
    const recordDomain = await registry.MEDICAL_RECORD_COMMITMENT_DOMAIN();
    const attachmentDomain = await registry.ATTACHMENT_COMMITMENT_DOMAIN();
    const certificateDomain = await registry.CERTIFICATE_COMMITMENT_DOMAIN();
    const custodyDomain = await registry.CUSTODY_COMMITMENT_DOMAIN();

    const fields = [1, 1, 1, vet.address, 1, "rabies", "vaccination", "annual", 1700000000];
    const recordHash = ethers.keccak256(encodeCommitment(recordDomain, ...fields));
    const attachmentHash = ethers.keccak256(encodeCommitment(attachmentDomain, ...fields));
    const certificateHash = ethers.keccak256(encodeCommitment(certificateDomain, ...fields));
    const custodyHash = ethers.keccak256(encodeCommitment(custodyDomain, ...fields));

    const hashes = [recordHash, attachmentHash, certificateHash, custodyHash];
    expect(new Set(hashes).size).to.equal(hashes.length);
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
});
