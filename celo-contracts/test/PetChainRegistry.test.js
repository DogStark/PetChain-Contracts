const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PetChainRegistry", function () {
  let registry;
  let admin, owner, other, vet;

  const PET = { name: "Rex", species: "Dog", breed: "Labrador", birthday: "2020-01-01" };

  beforeEach(async function () {
    [admin, owner, other, vet] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy();

    // Register and verify a vet
    await registry.connect(vet).registerVet("LIC-001");
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
        registry.connect(vet).addMedicalRecord(petId, "flu", "rest", "")
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
          petId, `Diag${i}`, `Treat${i}`, ""
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
        registry.connect(vet).addMedicalRecord(petId, ok1000, ok1000, ok1000)
      ).to.not.be.reverted;
    });

    it("accepts empty notes (notes is optional)", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "flu", "rest", "")
      ).to.not.be.reverted;
    });

    it("rejects empty diagnosis", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "", "rest", "")
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("rejects diagnosis over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, long1001, "rest", "")
      ).to.be.revertedWith("PetChainRegistry: invalid diagnosis length");
    });

    it("rejects empty treatment", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "flu", "", "")
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("rejects treatment over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "flu", long1001, "")
      ).to.be.revertedWith("PetChainRegistry: invalid treatment length");
    });

    it("rejects notes over 1000 chars", async function () {
      await expect(
        registry.connect(vet).addMedicalRecord(petId, "flu", "rest", long1001)
      ).to.be.revertedWith("PetChainRegistry: notes too long");
    });
  });
});
