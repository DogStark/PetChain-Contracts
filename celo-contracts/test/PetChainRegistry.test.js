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
    });
  });
});
