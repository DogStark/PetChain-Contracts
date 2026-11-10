const { expect } = require("chai");
const { ethers, network } = require("hardhat");

describe("Issue #1493 — Celo registry pet profile versioning", function () {
  let registry, admin, owner, stranger;

  beforeEach(async function () {
    [admin, owner, stranger] = await ethers.getSigners();
    const Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);
  });

  it("assigns CURRENT_PET_SCHEMA_VERSION (2) to newly registered pets and preserves V1 compatibility for legacy records", async function () {
    await registry.connect(owner).registerPet("Rex", "Dog", "Labrador", "2020-01-01");
    expect(await registry.getPetSchemaVersion(1)).to.equal(2);

    await registry.connect(owner).registerLegacyPetV1("Oldie", "Cat", "Siamese", "2018-05-05");
    expect(await registry.getPetSchemaVersion(2)).to.equal(1);

    const legacyProfile = await registry.getVersionedPetProfile(2);
    expect(legacyProfile.schemaVersion).to.equal(1);
    expect(legacyProfile.revision).to.equal(1);
    expect(legacyProfile.name).to.equal("Oldie");
    expect(legacyProfile.microchipId).to.equal("");
  });

  it("migrates a V1 record to V2 once and rejects replay or unauthorized migration", async function () {
    await registry.connect(owner).registerLegacyPetV1("Oldie", "Cat", "Siamese", "2018-05-05");

    await expect(
      registry.connect(stranger).migratePetProfileToV2(1, "chip-123", "ipfs://meta")
    ).to.be.revertedWith("PetChainRegistry: not authorized to migrate");

    await expect(
      registry.connect(owner).updatePetProfileV2(1, "chip-123", "ipfs://meta")
    ).to.be.revertedWith("PetChainRegistry: migration required before v2 update");

    await expect(
      registry.connect(owner).migratePetProfileToV2(1, "chip-123", "ipfs://meta")
    )
      .to.emit(registry, "PetProfileMigrated")
      .withArgs(1, 1, 2, 1);

    const migrated = await registry.getVersionedPetProfile(1);
    expect(migrated.schemaVersion).to.equal(2);
    expect(migrated.microchipId).to.equal("CHIP-123");
    expect(migrated.metadataUri).to.equal("ipfs://meta");

    await expect(
      registry.connect(owner).migratePetProfileToV2(1, "chip-456", "ipfs://meta2")
    ).to.be.revertedWith("PetChainRegistry: already at current version");

    await registry.connect(owner).updatePetProfileV2(1, "chip-456", "ipfs://meta2");
    const updated = await registry.getVersionedPetProfile(1);
    expect(updated.revision).to.equal(2);
    expect(updated.microchipId).to.equal("CHIP-456");
  });
});
