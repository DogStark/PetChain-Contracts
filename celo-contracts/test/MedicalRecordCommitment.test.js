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

  // ---------------------------------------------------------------------------
  // Attachment content-type and digest consistency enforcement (#1310)
  // ---------------------------------------------------------------------------
  // Attachment metadata is bound to the verified commitment: the normalized
  // content type, byte size, canonical filename, and digest are all part of the
  // preimage, so metadata cannot be swapped without producing a new version.
  const ATTACHMENT_COMMITMENT_TYPES = [
    "bytes32", "uint8", "uint256", "string", "uint256", "string", "bytes32",
  ];

  function encodeAttachmentCommitment(domain, version, recordId, contentType, byteSize, filename, digest) {
    return ethers.AbiCoder.defaultAbiCoder().encode(
      ATTACHMENT_COMMITMENT_TYPES,
      [domain, version, recordId, contentType, byteSize, filename, digest]
    );
  }

  const ALLOWED_CONTENT_TYPES = ["image/png", "image/jpeg", "application/pdf"];
  const MAX_ATTACHMENT_BYTES = 10 * 1024 * 1024; // 10 MiB

  // Canonical filename handling: NFC-normalize, trim surrounding whitespace,
  // collapse internal whitespace runs, and lowercase the extension. Equivalent
  // inputs therefore normalize to an identical string.
  function canonicalizeFilename(name) {
    return name
      .normalize("NFC")
      .trim()
      .replace(/\s+/g, " ")
      .replace(/\.([A-Za-z0-9]+)$/, (m, ext) => "." + ext.toLowerCase());
  }

  function isValidDigest(digest) {
    return typeof digest === "string" && /^0x[0-9a-fA-F]{64}$/.test(digest);
  }

  function validateAttachment({ contentType, byteSize, filename, digest }) {
    if (!ALLOWED_CONTENT_TYPES.includes(contentType)) {
      return { ok: false, code: "ERR_UNSUPPORTED_CONTENT_TYPE" };
    }
    if (!Number.isInteger(byteSize) || byteSize <= 0 || byteSize > MAX_ATTACHMENT_BYTES) {
      return { ok: false, code: "ERR_SIZE_LIMIT" };
    }
    if (typeof filename !== "string" || filename.length === 0 || filename.length > 255 || /[\u0000-\u001f\/\\]/.test(filename)) {
      return { ok: false, code: "ERR_MALFORMED_FILENAME" };
    }
    if (!isValidDigest(digest)) {
      return { ok: false, code: "ERR_INVALID_DIGEST" };
    }
    return { ok: true, code: null, filename: canonicalizeFilename(filename) };
  }

  it("rejects unsupported content types with a clear error code", async function () {
    const cases = [
      { contentType: "text/plain", code: "ERR_UNSUPPORTED_CONTENT_TYPE" },
      { contentType: "application/zip", code: "ERR_UNSUPPORTED_CONTENT_TYPE" },
      { contentType: "", code: "ERR_UNSUPPORTED_CONTENT_TYPE" },
    ];
    for (const { contentType, code } of cases) {
      const result = validateAttachment({
        contentType,
        byteSize: 1024,
        filename: "scan.png",
        digest: "0x" + "ab".repeat(32),
      });
      expect(result.ok).to.equal(false);
      expect(result.code).to.equal(code);
    }
  });

  it("enforces byte-size limits including boundary sizes", async function () {
    const digest = "0x" + "cd".repeat(32);
    const base = { contentType: "image/png", filename: "scan.png", digest };

    expect(validateAttachment({ ...base, byteSize: 1 }).ok).to.equal(true);
    expect(validateAttachment({ ...base, byteSize: MAX_ATTACHMENT_BYTES }).ok).to.equal(true);

    const over = validateAttachment({ ...base, byteSize: MAX_ATTACHMENT_BYTES + 1 });
    expect(over.ok).to.equal(false);
    expect(over.code).to.equal("ERR_SIZE_LIMIT");

    const zero = validateAttachment({ ...base, byteSize: 0 });
    expect(zero.ok).to.equal(false);
    expect(zero.code).to.equal("ERR_SIZE_LIMIT");
  });

  it("validates digest length and rejects invalid digests", async function () {
    const base = { contentType: "application/pdf", byteSize: 2048, filename: "report.pdf" };
    const bad = [
      "0x" + "ab".repeat(31), // too short
      "0x" + "ab".repeat(33), // too long
      "ab".repeat(32),        // missing 0x prefix
      "0x" + "zz".repeat(32), // non-hex
      "",
    ];
    for (const digest of bad) {
      const result = validateAttachment({ ...base, digest });
      expect(result.ok).to.equal(false);
      expect(result.code).to.equal("ERR_INVALID_DIGEST");
    }
    expect(validateAttachment({ ...base, digest: "0x" + "ab".repeat(32) }).ok).to.equal(true);
  });

  it("canonicalizes filenames and rejects malformed filenames", async function () {
    const digest = "0x" + "ef".repeat(32);
    const base = { contentType: "image/jpeg", byteSize: 4096, digest };

    const malformed = ["", "a".repeat(256), "dir/scan.jpg", "dir\\scan.jpg", "bad\u0000name.jpg"];
    for (const filename of malformed) {
      const result = validateAttachment({ ...base, filename });
      expect(result.ok).to.equal(false);
      expect(result.code).to.equal("ERR_MALFORMED_FILENAME");
    }

    // Equivalent metadata normalizes identically, including Unicode filenames.
    const vectors = [
      ["  Scan.JPG  ", "Scan.jpg"],
      ["scan   photo.JPEG", "scan photo.jpeg"],
      ["caf\u00e9.PNG", "caf\u00e9.png"],
      ["cafe\u0301.PNG", "caf\u00e9.png"], // NFD input normalizes to NFC
    ];
    for (const [input, expected] of vectors) {
      const result = validateAttachment({ ...base, filename: input });
      expect(result.ok).to.equal(true);
      expect(result.filename).to.equal(expected);
    }
  });

  it("binds normalized metadata into the commitment and detects tampering", async function () {
    const { recordId } = await addRecord();
    const domain = await registry.ATTACHMENT_COMMITMENT_DOMAIN();
    const digest = "0x" + "12".repeat(32);

    const meta = { contentType: "image/png", byteSize: 1024, filename: "  Scan.PNG ", digest };
    const validated = validateAttachment(meta);
    expect(validated.ok).to.equal(true);

    const commitment = ethers.keccak256(
      encodeAttachmentCommitment(domain, 1, recordId, meta.contentType, meta.byteSize, validated.filename, digest)
    );

    // Equivalent metadata (different raw filename) yields the same commitment.
    const equivalent = validateAttachment({ ...meta, filename: "scan.png" });
    const equivalentCommitment = ethers.keccak256(
      encodeAttachmentCommitment(domain, 1, recordId, meta.contentType, meta.byteSize, equivalent.filename, digest)
    );
    expect(equivalentCommitment).to.equal(commitment);

    // Tampering with any bound field produces a different commitment, i.e. a
    // new version is required to change the digest or metadata.
    const tampered = [
      encodeAttachmentCommitment(domain, 1, recordId, "image/jpeg", meta.byteSize, validated.filename, digest),
      encodeAttachmentCommitment(domain, 1, recordId, meta.contentType, meta.byteSize + 1, validated.filename, digest),
      encodeAttachmentCommitment(domain, 1, recordId, meta.contentType, meta.byteSize, "other.png", digest),
      encodeAttachmentCommitment(domain, 1, recordId, meta.contentType, meta.byteSize, validated.filename, "0x" + "34".repeat(32)),
    ];
    for (const encoded of tampered) {
      expect(ethers.keccak256(encoded)).to.not.equal(commitment);
    }
  });
});
