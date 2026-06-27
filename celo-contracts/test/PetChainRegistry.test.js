const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("PetChainRegistry", function () {
  let registry;
  let admin, owner, other, vet;

  const PET = { name: "Rex", species: "Dog", breed: "Labrador", birthday: "2020-01-01" };

  beforeEach(async function () {
    [admin, owner, other, vet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy();

    // Register and verify a vet
    await registry.connect(vet).registerVet("LIC-001", "General Practice");
    await registry.connect(admin).verifyVet(vet.address);
  });

  // ---------------------------------------------------------------------------
  // Helpers
  // ---------------------------------------------------------------------------
  async function registerPet(signer = owner) {
    const tx = await registry.connect(signer).registerPet(
      PET.name, PET.species, PET.breed, PET.birthday
    );
    const receipt = await tx.wait();
    const event = receipt.logs.find(
      l => l.fragment && l.fragment.name === "PetRegistered"
    );
    return event.args.petId;
  }

  // ---------------------------------------------------------------------------
  // Issue #922 — medical record type / category
  // ---------------------------------------------------------------------------
  describe("#922 — getPetRecordsByType", function () {
    // RecordType { Checkup, Vaccination, Surgery, LabResult, Other }
    const Vaccination = 1;
    const Surgery = 2;
    let petId;

    beforeEach(async function () {
      petId = await registerPet();
      await registry.connect(vet).addMedicalRecord(petId, Vaccination, "rabies", "shot", "");
      await registry.connect(vet).addMedicalRecord(petId, Surgery, "fracture", "cast", "");
      await registry.connect(vet).addMedicalRecord(petId, Vaccination, "parvo", "shot", "");
    });

    it("stores the record type on the record", async function () {
      const records = await registry.getPetRecords(petId);
      expect(records[0].recordType).to.equal(Vaccination);
      expect(records[1].recordType).to.equal(Surgery);
    });

    it("returns only records matching the requested type", async function () {
      const vaccinations = await registry.getPetRecordsByType(petId, Vaccination);
      expect(vaccinations.length).to.equal(2);
      expect(vaccinations.every(r => Number(r.recordType) === Vaccination)).to.equal(true);

      const surgeries = await registry.getPetRecordsByType(petId, Surgery);
      expect(surgeries.length).to.equal(1);
    });

    it("returns an empty array when no records match the type", async function () {
      const labResults = await registry.getPetRecordsByType(petId, 3); // LabResult
      expect(labResults.length).to.equal(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #923 — getPetRecordsByDateRange
  // ---------------------------------------------------------------------------
  describe("#923 — getPetRecordsByDateRange", function () {
    let petId, t0, t1, t2;

    beforeEach(async function () {
      petId = await registerPet();
      const base = (await ethers.provider.getBlock("latest")).timestamp;
      t0 = base + 1000;
      t1 = base + 2000;
      t2 = base + 3000;
      for (const ts of [t0, t1, t2]) {
        await network.provider.send("evm_setNextBlockTimestamp", [ts]);
        await registry.connect(vet).addMedicalRecord(petId, "diag", "treat", "");
      }
    });

    it("returns all record IDs for a full-history range", async function () {
      const ids = await registry.getPetRecordsByDateRange(petId, 0, t2);
      expect(ids.length).to.equal(3);
    });

    it("returns only the records inside a partial-overlap range", async function () {
      const ids = await registry.getPetRecordsByDateRange(petId, t1, t1);
      expect(ids.length).to.equal(1);
      expect(ids[0]).to.equal(2); // second record added
    });

    it("returns an empty array for a range with no records", async function () {
      const ids = await registry.getPetRecordsByDateRange(petId, t2 + 1000, t2 + 2000);
      expect(ids.length).to.equal(0);
  // Issue #924 — core flows & access-control revert paths
  // ---------------------------------------------------------------------------
  describe("#924 — vet management", function () {
    it("registers a vet and emits VetRegistered", async function () {
      await expect(registry.connect(other).registerVet("LIC-XYZ"))
        .to.emit(registry, "VetRegistered")
        .withArgs(other.address, "LIC-XYZ");
      const v = await registry.vets(other.address);
      expect(v.licenseNumber).to.equal("LIC-XYZ");
      expect(v.isVerified).to.equal(false);
    });

    it("reverts registering with an empty license number", async function () {
      await expect(registry.connect(other).registerVet(""))
        .to.be.revertedWith("PetChainRegistry: empty licenseNumber");
    });

    it("admin verifies a vet and emits VetVerified", async function () {
      await registry.connect(other).registerVet("LIC-XYZ");
      await expect(registry.connect(admin).verifyVet(other.address))
        .to.emit(registry, "VetVerified")
        .withArgs(other.address);
      expect((await registry.vets(other.address)).isVerified).to.equal(true);
    });

    it("admin revokes a vet (verified flag cleared)", async function () {
      await registry.connect(admin).revokeVet(vet.address);
      const v = await registry.vets(vet.address);
      expect(v.isVerified).to.equal(false);
      expect(v.isRevoked).to.equal(true);
    });

    it("verifyVet reverts for a revoked vet", async function () {
      await registry.connect(admin).revokeVet(vet.address);
      await expect(registry.connect(admin).verifyVet(vet.address))
        .to.be.revertedWith("PetChainRegistry: vet is revoked");
    });

    it("onlyAdmin: non-admin cannot verify", async function () {
      await registry.connect(other).registerVet("LIC-XYZ");
      await expect(registry.connect(other).verifyVet(other.address))
        .to.be.revertedWith("PetChainRegistry: not admin");
    });

    it("onlyAdmin: non-admin cannot revoke", async function () {
      await expect(registry.connect(other).revokeVet(vet.address))
        .to.be.revertedWith("PetChainRegistry: not admin");
    });
  });

  describe("#924 — pet management", function () {
    it("registers a pet owned by the caller and marked active", async function () {
      const petId = await registerPet();
      const pet = await registry.pets(petId);
      expect(pet.owner).to.equal(owner.address);
      expect(pet.active).to.equal(true);
      expect(await registry.getPetsByOwner(owner.address)).to.deep.equal([petId]);
    });

    it("transfers a pet and emits PetTransferred", async function () {
      const petId = await registerPet();
      await expect(registry.connect(owner).transferPet(petId, other.address))
        .to.emit(registry, "PetTransferred")
        .withArgs(petId, owner.address, other.address);
      expect((await registry.pets(petId)).owner).to.equal(other.address);
    });

    it("transferPet reverts on the zero address", async function () {
      const petId = await registerPet();
      await expect(registry.connect(owner).transferPet(petId, ethers.ZeroAddress))
        .to.be.revertedWith("PetChainRegistry: zero address");
    });

    it("onlyPetOwner: non-owner cannot transfer", async function () {
      const petId = await registerPet();
      await expect(registry.connect(other).transferPet(petId, other.address))
        .to.be.revertedWith("PetChainRegistry: not pet owner");
    });

    it("onlyPetOwner: non-owner cannot deactivate", async function () {
      const petId = await registerPet();
      await expect(registry.connect(other).deactivatePet(petId))
        .to.be.revertedWith("PetChainRegistry: not pet owner");
    });
  });

  describe("#924 — medical records", function () {
    it("a verified vet adds a record which is stored and emitted", async function () {
      const petId = await registerPet();
      await expect(registry.connect(vet).addMedicalRecord(petId, "flu", "rest", "note"))
        .to.emit(registry, "MedicalRecordAdded")
        .withArgs(petId, 1, vet.address);
      const records = await registry.getPetRecords(petId);
      expect(records.length).to.equal(1);
      expect(records[0].diagnosis).to.equal("flu");
      expect(records[0].vet).to.equal(vet.address);
    });

    it("onlyVerifiedVet: an unregistered address cannot add a record", async function () {
      const petId = await registerPet();
      await expect(registry.connect(other).addMedicalRecord(petId, "flu", "rest", ""))
        .to.be.revertedWith("PetChainRegistry: not a verified vet");
    });

    it("onlyVerifiedVet: a registered but unverified vet cannot add a record", async function () {
      const petId = await registerPet();
      await registry.connect(other).registerVet("LIC-UNVERIFIED");
      await expect(registry.connect(other).addMedicalRecord(petId, "flu", "rest", ""))
        .to.be.revertedWith("PetChainRegistry: not a verified vet");
    });

    it("reverts adding a record for an inactive pet", async function () {
      const petId = await registerPet();
      await registry.connect(owner).deactivatePet(petId);
      await expect(registry.connect(vet).addMedicalRecord(petId, "flu", "rest", ""))
        .to.be.revertedWith("PetChainRegistry: pet inactive");
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #916 — VetRevoked event
  // ---------------------------------------------------------------------------
  describe("#916 — emit VetRevoked on revokeVet", function () {
    it("emits VetRevoked with the correct vet address", async function () {
      await expect(registry.connect(admin).revokeVet(vet.address))
        .to.emit(registry, "VetRevoked")
        .withArgs(vet.address);
    });

    it("revoked vet cannot add medical records", async function () {
      const petId = await registerPet();
      await registry.connect(admin).revokeVet(vet.address);
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "flu", "rest", "")
      ).to.be.revertedWith("PetChainRegistry: not a verified vet");
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #916 — PetDeactivated event
  // ---------------------------------------------------------------------------
  describe("#916 — emit PetDeactivated on deactivatePet", function () {
    it("emits PetDeactivated with the correct petId", async function () {
      const petId = await registerPet();
      await expect(registry.connect(owner).deactivatePet(petId))
        .to.emit(registry, "PetDeactivated")
        .withArgs(petId);
    });

    it("deactivated pet has active == false", async function () {
      const petId = await registerPet();
      await registry.connect(owner).deactivatePet(petId);
      const pet = await registry.pets(petId);
      expect(pet.active).to.equal(false);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #917 — reactivatePet
  // ---------------------------------------------------------------------------
  describe("#917 — reactivatePet", function () {
    it("deactivate-then-reactivate restores active state and emits PetReactivated", async function () {
      const petId = await registerPet();
      await registry.connect(owner).deactivatePet(petId);
      await expect(registry.connect(owner).reactivatePet(petId))
        .to.emit(registry, "PetReactivated")
        .withArgs(petId);
      const pet = await registry.pets(petId);
      expect(pet.active).to.equal(true);
    });

    it("reverts if pet is already active", async function () {
      const petId = await registerPet();
      await expect(registry.connect(owner).reactivatePet(petId))
        .to.be.revertedWith("PetChainRegistry: already active");
    });

    it("only pet owner can reactivate", async function () {
      const petId = await registerPet();
      await registry.connect(owner).deactivatePet(petId);
      await expect(registry.connect(other).reactivatePet(petId))
        .to.be.revertedWith("PetChainRegistry: not pet owner");
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #918 — getPetsByOwnerPaged
  // ---------------------------------------------------------------------------
  describe("#918 — getPetsByOwnerPaged", function () {
    beforeEach(async function () {
      // Register 5 pets for owner
      for (let i = 0; i < 5; i++) {
        await registry.connect(owner).registerPet(
          `Pet${i}`, "Dog", "Mix", "2021-01-01"
        );
      }
    });

    it("returns correct page in the middle", async function () {
      const page = await registry.getPetsByOwnerPaged(owner.address, 1, 2);
      expect(page.length).to.equal(2);
    });

    it("returns partial last page", async function () {
      const page = await registry.getPetsByOwnerPaged(owner.address, 4, 10);
      expect(page.length).to.equal(1);
    });

    it("returns empty array when offset >= total", async function () {
      const page = await registry.getPetsByOwnerPaged(owner.address, 10, 3);
      expect(page.length).to.equal(0);
    });

    it("returns empty array when limit == 0", async function () {
      const page = await registry.getPetsByOwnerPaged(owner.address, 0, 0);
      expect(page.length).to.equal(0);
    });

    it("returns empty for address with no pets", async function () {
      const page = await registry.getPetsByOwnerPaged(other.address, 0, 10);
      expect(page.length).to.equal(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #918 — getPetRecordsPaged
  // ---------------------------------------------------------------------------
  describe("#918 — getPetRecordsPaged", function () {
    let petId;

    beforeEach(async function () {
      petId = await registerPet();
      // Add 5 records
      for (let i = 0; i < 5; i++) {
        await registry.connect(vet).addMedicalRecord(
          petId, 0, `Diag${i}`, `Treat${i}`, ""
        );
      }
    });

    it("returns correct page in the middle", async function () {
      const page = await registry.getPetRecordsPaged(petId, 1, 2);
      expect(page.length).to.equal(2);
    });

    it("returns partial last page", async function () {
      const page = await registry.getPetRecordsPaged(petId, 4, 10);
      expect(page.length).to.equal(1);
    });

    it("returns empty array when offset >= total", async function () {
      const page = await registry.getPetRecordsPaged(petId, 10, 3);
      expect(page.length).to.equal(0);
    });

    it("returns empty array when limit == 0", async function () {
      const page = await registry.getPetRecordsPaged(petId, 0, 0);
      expect(page.length).to.equal(0);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #914 — transferPet removes petId from previous owner's _ownerPets
  // ---------------------------------------------------------------------------
  describe("#914 — transferPet removes stale _ownerPets entry", function () {
    it("pet no longer appears in previous owner's getPetsByOwner after transfer", async function () {
      const petId = await registerPet();

      await registry.connect(owner).transferPet(petId, other.address);

      const fromPets = await registry.getPetsByOwner(owner.address);
      expect(fromPets.map(id => id.toString())).to.not.include(petId.toString());

      const toPets = await registry.getPetsByOwner(other.address);
      expect(toPets.map(id => id.toString())).to.include(petId.toString());
    });

    it("multiple transfers leave no stale entries in intermediate owners", async function () {
      const petId = await registerPet();

      await registry.connect(owner).transferPet(petId, other.address);
      await registry.connect(other).transferPet(petId, admin.address);

      const ownerPets = await registry.getPetsByOwner(owner.address);
      const otherPets = await registry.getPetsByOwner(other.address);
      const adminPets = await registry.getPetsByOwner(admin.address);

      expect(ownerPets.map(id => id.toString())).to.not.include(petId.toString());
      expect(otherPets.map(id => id.toString())).to.not.include(petId.toString());
      expect(adminPets.map(id => id.toString())).to.include(petId.toString());
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #919 — registerPet input length validation
  // ---------------------------------------------------------------------------
  describe("#919 — registerPet string validation", function () {
    const long65 = "a".repeat(65);

    it("accepts fields at exactly MAX_SHORT_LEN (64)", async function () {
      const s64 = "a".repeat(64);
      await expect(
        registry.connect(owner).registerPet(s64, s64, s64, s64)
      ).to.not.be.reverted;
    });

    it("rejects empty name", async function () {
      await expect(
        registry.connect(owner).registerPet("", "Dog", "Mix", "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid name length");
    });

    it("rejects name over 64 chars", async function () {
      await expect(
        registry.connect(owner).registerPet(long65, "Dog", "Mix", "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid name length");
    });

    it("rejects empty species", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", "", "Mix", "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid species length");
    });

    it("rejects species over 64 chars", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", long65, "Mix", "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid species length");
    });

    it("rejects empty breed", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", "Dog", "", "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid breed length");
    });

    it("rejects breed over 64 chars", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", "Dog", long65, "2020-01-01")
      ).to.be.revertedWith("PetChainRegistry: invalid breed length");
    });

    it("rejects empty birthday", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", "Dog", "Mix", "")
      ).to.be.revertedWith("PetChainRegistry: invalid birthday length");
    });

    it("rejects birthday over 64 chars", async function () {
      await expect(
        registry.connect(owner).registerPet("Rex", "Dog", "Mix", long65)
      ).to.be.revertedWith("PetChainRegistry: invalid birthday length");
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #919 — addMedicalRecord input length validation
  // ---------------------------------------------------------------------------
  describe("#919 — addMedicalRecord string validation", function () {
    let petId;
    const long1001 = "a".repeat(1001);
    const ok1000 = "a".repeat(1000);

    beforeEach(async function () {
      petId = await registerPet();
    });

    it("accepts fields at exactly MAX_LONG_LEN (1000)", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, ok1000, ok1000, ok1000)
      ).to.not.be.reverted;
    });

    it("accepts empty notes (notes is optional)", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "flu", "rest", "")
      ).to.not.be.reverted;
    });

    it("rejects empty diagnosis", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "", "rest", "")
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("rejects diagnosis over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, long1001, "rest", "")
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("rejects empty treatment", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "flu", "", "")
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("rejects treatment over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "flu", long1001, "")
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("rejects notes over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, 0, "flu", "rest", long1001)
      ).to.be.revertedWith("PetChainRegistry: notes too long");
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #927 — case-insensitive vet license uniqueness
  // ---------------------------------------------------------------------------
  describe("#927 — case-insensitive license uniqueness", function () {
    it("rejects a different-case duplicate of an existing license from another address", async function () {
      await expect(registry.connect(other).registerVet("lic-001", "Surgery"))
        .to.be.revertedWith("PetChainRegistry: license already registered");
    });

    it("rejects an exact-case duplicate from another address", async function () {
      await expect(registry.connect(other).registerVet("LIC-001", "Surgery"))
        .to.be.revertedWith("PetChainRegistry: license already registered");
    });

    it("preserves the originally-submitted casing on the Vet struct", async function () {
      await registry.connect(other).registerVet("AbC123", "Dermatology");
      const v = await registry.vets(other.address);
      expect(v.licenseNumber).to.equal("AbC123");
    });

    it("allows the same address to re-register with a new license, freeing the old one", async function () {
      await registry.connect(vet).registerVet("LIC-999", "General Practice");
      await registry.connect(other).registerVet("lic-001", "Surgery");
      const v = await registry.vets(other.address);
      expect(v.licenseNumber).to.equal("lic-001");
    });

    it("allows distinct license numbers from different addresses", async function () {
      await expect(registry.connect(other).registerVet("LIC-002", "Surgery"))
        .to.not.be.reverted;
  // Issue — correctMedicalRecord
  // ---------------------------------------------------------------------------
  describe("correctMedicalRecord", function () {
    let petId;
    let recordId;

    beforeEach(async function () {
      petId = await registerPet();
      // vet adds the initial record
      const tx = await registry.connect(vet).addMedicalRecord(
        petId, "Initial diagnosis", "Initial treatment", "Initial notes"
      );
      const receipt = await tx.wait();
      const event = receipt.logs.find(
        l => l.fragment && l.fragment.name === "MedicalRecordAdded"
      );
      recordId = event.args.recordId;
    });

    // --- authorised corrections ---

    it("original vet can correct the record and state is updated", async function () {
      await registry.connect(vet).correctMedicalRecord(
        recordId, "Corrected diagnosis", "Corrected treatment", "Corrected notes"
      );
      const records = await registry.getPetRecords(petId);
      expect(records[0].diagnosis).to.equal("Corrected diagnosis");
      expect(records[0].treatment).to.equal("Corrected treatment");
      expect(records[0].notes).to.equal("Corrected notes");
    });

    it("admin can correct the record", async function () {
      await registry.connect(admin).correctMedicalRecord(
        recordId, "Admin corrected diag", "Admin corrected treat", ""
      );
      const records = await registry.getPetRecords(petId);
      expect(records[0].diagnosis).to.equal("Admin corrected diag");
    });

    it("emits MedicalRecordCorrected preserving original values", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(
          recordId, "New diag", "New treat", "New notes"
        )
      )
        .to.emit(registry, "MedicalRecordCorrected")
        .withArgs(
          recordId,
          petId,
          vet.address,
          "Initial diagnosis",
          "Initial treatment",
          "Initial notes",
          "New diag",
          "New treat",
          "New notes"
        );
    });

    it("admin correction emits MedicalRecordCorrected with admin as correctedBy", async function () {
      await expect(
        registry.connect(admin).correctMedicalRecord(
          recordId, "Admin diag", "Admin treat", ""
        )
      )
        .to.emit(registry, "MedicalRecordCorrected")
        .withArgs(
          recordId,
          petId,
          admin.address,
          "Initial diagnosis",
          "Initial treatment",
          "Initial notes",
          "Admin diag",
          "Admin treat",
          ""
        );
    });

    it("allows empty notes in correction (notes is optional)", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(
          recordId, "Diag", "Treat", ""
        )
      ).to.not.be.reverted;
      const records = await registry.getPetRecords(petId);
      expect(records[0].notes).to.equal("");
    });

    it("accepts fields at exactly MAX_LONG_LEN (1000 chars)", async function () {
      const s1000 = "a".repeat(1000);
      await expect(
        registry.connect(vet).correctMedicalRecord(recordId, s1000, s1000, s1000)
      ).to.not.be.reverted;
    });

    // --- unauthorised corrections ---

    it("reverts when called by a different verified vet (not the original)", async function () {
      const [, , , , vet2] = await ethers.getSigners();
      await registry.connect(vet2).registerVet("LIC-002");
      await registry.connect(admin).verifyVet(vet2.address);

      await expect(
        registry.connect(vet2).correctMedicalRecord(
          recordId, "Hack diag", "Hack treat", ""
        )
      ).to.be.revertedWith("PetChainRegistry: not authorised to correct record");
    });

    it("reverts when called by the pet owner (not vet or admin)", async function () {
      await expect(
        registry.connect(owner).correctMedicalRecord(
          recordId, "Owner diag", "Owner treat", ""
        )
      ).to.be.revertedWith("PetChainRegistry: not authorised to correct record");
    });

    it("reverts when called by an arbitrary address", async function () {
      await expect(
        registry.connect(other).correctMedicalRecord(
          recordId, "Other diag", "Other treat", ""
        )
      ).to.be.revertedWith("PetChainRegistry: not authorised to correct record");
    });

    // --- input validation ---

    it("reverts on empty diagnosis", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(recordId, "", "Treat", "")
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("reverts on diagnosis over 1000 chars", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(
          recordId, "a".repeat(1001), "Treat", ""
        )
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("reverts on empty treatment", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(recordId, "Diag", "", "")
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("reverts on treatment over 1000 chars", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(
          recordId, "Diag", "a".repeat(1001), ""
        )
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("reverts on notes over 1000 chars", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(
          recordId, "Diag", "Treat", "a".repeat(1001)
        )
      ).to.be.revertedWith("PetChainRegistry: notes too long");
    });

    it("reverts when recordId does not exist", async function () {
      await expect(
        registry.connect(vet).correctMedicalRecord(9999, "Diag", "Treat", "")
      ).to.be.revertedWith("PetChainRegistry: record does not exist");
    });
  });

  // ---------------------------------------------------------------------------
  // Admin transfer — transferAdmin
  // ---------------------------------------------------------------------------
  describe("transferAdmin", function () {
    it("emits AdminTransferred with correct previous and new admin", async function () {
      await expect(registry.connect(admin).transferAdmin(other.address))
        .to.emit(registry, "AdminTransferred")
        .withArgs(admin.address, other.address);
    });

    it("updates the admin state variable", async function () {
      await registry.connect(admin).transferAdmin(other.address);
      expect(await registry.admin()).to.equal(other.address);
    });

    it("old admin loses onlyAdmin access after transfer", async function () {
      await registry.connect(vet).registerVet("LIC-NEW");
      await registry.connect(admin).transferAdmin(other.address);
      // original admin can no longer call verifyVet
      await expect(
        registry.connect(admin).verifyVet(vet.address)
      ).to.be.revertedWith("PetChainRegistry: not admin");
    });

    it("new admin can exercise onlyAdmin functions", async function () {
      await registry.connect(vet).registerVet("LIC-NEW");
      await registry.connect(admin).transferAdmin(other.address);
      // new admin (other) can now verify vets
      await expect(
        registry.connect(other).verifyVet(vet.address)
      ).to.not.be.reverted;
    });

    it("reverts when called by non-admin", async function () {
      await expect(
        registry.connect(owner).transferAdmin(other.address)
      ).to.be.revertedWith("PetChainRegistry: not admin");
    });

    it("reverts when newAdmin is the zero address", async function () {
      await expect(
        registry.connect(admin).transferAdmin(ethers.ZeroAddress)
      ).to.be.revertedWith("PetChainRegistry: zero address");
  // Issue #928 — Pausable emergency stop
  // ---------------------------------------------------------------------------
  describe("#928 — Pausable emergency stop", function () {
    it("only admin can pause", async function () {
      await expect(registry.connect(owner).pause())
        .to.be.revertedWith("PetChainRegistry: not admin");
    });

    it("only admin can unpause", async function () {
      await registry.connect(admin).pause();
      await expect(registry.connect(owner).unpause())
        .to.be.revertedWith("PetChainRegistry: not admin");
    });

    it("admin can pause and paused() reflects state", async function () {
      await registry.connect(admin).pause();
      expect(await registry.paused()).to.equal(true);
    });

    it("blocks registerPet while paused", async function () {
      await registry.connect(admin).pause();
      await expect(
        registry.connect(owner).registerPet(PET.name, PET.species, PET.breed, PET.birthday)
      ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    });

    it("blocks transferPet while paused", async function () {
      const petId = await registerPet();
      await registry.connect(admin).pause();
      await expect(
        registry.connect(owner).transferPet(petId, other.address)
      ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    });

    it("blocks addMedicalRecord while paused", async function () {
      const petId = await registerPet();
      await registry.connect(admin).pause();
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "flu", "rest", "")
      ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    });

    it("blocks registerVet while paused", async function () {
      await registry.connect(admin).pause();
      await expect(
        registry.connect(other).registerVet("LIC-002", "Surgery")
      ).to.be.revertedWithCustomError(registry, "EnforcedPause");
    });

    it("resumes normal operation after unpause", async function () {
      await registry.connect(admin).pause();
      await registry.connect(admin).unpause();
      expect(await registry.paused()).to.equal(false);
      await expect(
        registry.connect(owner).registerPet(PET.name, PET.species, PET.breed, PET.birthday)
      ).to.not.be.reverted;
    });
  });
});
