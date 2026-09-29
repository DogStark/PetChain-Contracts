const { expect } = require("chai");
const { ethers, network } = require("hardhat");
const fc = require("fast-check");

const LOCAL_CHAIN_ID = 31337;
const ALFAJORES_CHAIN_ID = 44787;
const CELO_CHAIN_ID = 42220;

describe("PetChainRegistry invariants", function () {
  let Factory, registry, admin, owner, other, vet;

  async function registerPet(signer = owner) {
    const receipt = await (await registry.connect(signer)
      .registerPet("Rex", "Dog", "Labrador", "2020-01-01")).wait();
    return receipt.logs.find(l => l.fragment && l.fragment.name === "PetRegistered").args.petId;
  }

  beforeEach(async function () {
    [admin, owner, other, vet] = await ethers.getSigners();
    Factory = await ethers.getContractFactory("PetChainRegistry");
    registry = await Factory.deploy(admin.address, network.config.chainId);
  });

  // ---------------------------------------------------------------------------
  // Issue #1323 — constructor / deployment invariants
  // ---------------------------------------------------------------------------
  describe("#1323 — deployment invariants", function () {
    it("exposes the configured admin and deployment chain id", async function () {
      expect(network.config.chainId).to.equal(LOCAL_CHAIN_ID);
      expect(await registry.admin()).to.equal(admin.address);
      expect(await registry.deploymentChainId()).to.equal(LOCAL_CHAIN_ID);
    });

    it("accepts an admin other than the deployer", async function () {
      const r = await Factory.deploy(owner.address, LOCAL_CHAIN_ID);
      expect(await r.admin()).to.equal(owner.address);
      await expect(r.connect(admin).pause()).to.be.revertedWith("PetChainRegistry: not admin");
      await r.connect(owner).pause();
    });

    it("reverts on a zero admin", async function () {
      await expect(Factory.deploy(ethers.ZeroAddress, LOCAL_CHAIN_ID))
        .to.be.revertedWith("PetChainRegistry: zero admin");
    });

    for (const chainId of [ALFAJORES_CHAIN_ID, CELO_CHAIN_ID, 1, 0]) {
      it(`reverts when targeting chain ${chainId} from the local chain`, async function () {
        await expect(Factory.deploy(admin.address, chainId))
          .to.be.revertedWith("PetChainRegistry: wrong chain");
      });
    }

    it("supports only the local and Celo target chain ids", async function () {
      for (const id of [LOCAL_CHAIN_ID, ALFAJORES_CHAIN_ID, CELO_CHAIN_ID]) {
        expect(await registry.isSupportedChainId(id)).to.equal(true);
      }
      for (const id of [0, 1, 137, 44788, 42221]) {
        expect(await registry.isSupportedChainId(id)).to.equal(false);
      }
    });

    it("deployment chain id is immutable (no setter, unchanged by admin actions)", async function () {
      const setters = registry.interface.fragments.filter(
        f => f.type === "function" && !f.constant && /chain/i.test(f.name)
      );
      expect(setters).to.have.length(0);
      await registry.connect(admin).transferAdmin(owner.address);
      expect(await registry.deploymentChainId()).to.equal(LOCAL_CHAIN_ID);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #1324 — storage cleanup policy for deactivated pets
  // ---------------------------------------------------------------------------
  describe("#1324 — deactivated pet cleanup", function () {
    it("removes a deactivated pet from active owner queries only", async function () {
      const a = await registerPet();
      const b = await registerPet();
      await registry.connect(owner).deactivatePet(a);
      expect(await registry.getPetsByOwner(owner.address)).to.deep.equal([b]);
      expect(await registry.getPetsByOwnerPaged(owner.address, 0, 10)).to.deep.equal([b]);
    });

    it("keeps historical ownership and commitment proofs", async function () {
      await registry.connect(vet).registerVet("LIC-1324", "General Practice");
      await registry.connect(admin).verifyVet(vet.address);
      const petId = await registerPet();
      await registry.connect(vet).addMedicalRecord(petId, 0, "diag", "treat", "");
      const commitment = await registry.medicalRecordCommitments(1);

      await registry.connect(owner).deactivatePet(petId);

      const pet = await registry.pets(petId);
      expect(pet.owner).to.equal(owner.address);
      expect(pet.active).to.equal(false);
      const [rec] = await registry.getPetRecords(petId);
      expect(await registry.verifyMedicalRecordCommitment(
        1, 1, petId, vet.address, 0, "diag", "treat", "", rec.timestamp, commitment
      )).to.equal(true);
    });

    it("cleanup is idempotent: repeated deactivation leaves the index unchanged", async function () {
      const a = await registerPet();
      const b = await registerPet();
      await registry.connect(owner).deactivatePet(a);
      await expect(registry.connect(owner).deactivatePet(a))
        .to.be.revertedWith("PetChainRegistry: already inactive");
      expect(await registry.getPetsByOwner(owner.address)).to.deep.equal([b]);
    });

    it("repeated deactivate/reactivate cycles never duplicate index entries", async function () {
      const a = await registerPet();
      const b = await registerPet();
      for (let i = 0; i < 3; i++) {
        await registry.connect(owner).deactivatePet(a);
        expect(await registry.getPetsByOwner(owner.address)).to.deep.equal([b]);
        await registry.connect(owner).reactivatePet(a);
        const ids = (await registry.getPetsByOwner(owner.address)).map(Number).sort();
        expect(ids).to.deep.equal([Number(a), Number(b)]);
      }
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #1325 — identifier normalization fuzzing
  // ---------------------------------------------------------------------------
  describe("#1325 — identifier normalization", function () {
    const INVALID_CHAR = "PetChainRegistry: invalid identifier character";
    const INVALID_LEN = "PetChainRegistry: invalid identifier length";

    // Reference model of the on-chain rules.
    function reference(id) {
      let out = "";
      for (const byte of Buffer.from(id, "utf8")) {
        if (byte === 0x20) continue;
        if (byte < 0x21 || byte > 0x7e) return { error: INVALID_CHAR };
        out += String.fromCharCode(byte >= 0x61 && byte <= 0x7a ? byte - 32 : byte);
      }
      if (out.length === 0 || out.length > 64) return { error: INVALID_LEN };
      return { value: out };
    }

    async function onChain(id) {
      try {
        return { value: await registry.normalizeIdentifier(id) };
      } catch (e) {
        return { error: [INVALID_CHAR, INVALID_LEN].find(r => e.message.includes(r)) || e.message };
      }
    }

    const canonicalId = fc.stringMatching(/^[A-Z0-9:#\-]{1,64}$/);

    // Variant of a canonical id differing only by case and ASCII spaces.
    const variantOf = id => fc.array(fc.tuple(fc.boolean(), fc.nat(2)), {
      minLength: id.length, maxLength: id.length,
    }).map(flags => "  " + [...id].map((c, i) =>
      (flags[i][0] ? c.toLowerCase() : c) + " ".repeat(flags[i][1])).join("") + " ");

    const REGRESSION_SEEDS = [
      "", " ", "   ", "lic-1", "  lic-1  ", "L I C - 1", "LIC\t1", "LIC\n1", "LIC\r1",
      "\u0000", "LIC\u007f", " LIC", "LIC​1", "ＬＩＣ-1", "lıc", "LİC", "café",
      "🐶", "a".repeat(64), "a".repeat(65), " ".repeat(20) + "a".repeat(64) + " ",
    ];

    it("matches the reference model on regression seeds", async function () {
      for (const seed of REGRESSION_SEEDS) {
        expect(await onChain(seed), JSON.stringify(seed)).to.deep.equal(reference(seed));
      }
    });

    it("matches the reference model on fuzzed Unicode and boundary input", async function () {
      await fc.assert(fc.asyncProperty(
        fc.oneof(
          fc.fullUnicodeString({ maxLength: 80 }),
          fc.string({ maxLength: 80 }),
          fc.stringMatching(/^[ a-zA-Z0-9\-]{60,70}$/),
        ),
        async id => {
          expect(await onChain(id)).to.deep.equal(reference(id));
        }
      ), { numRuns: 200, seed: 1325 });
    });

    it("equivalent identifiers resolve identically and collide on registration", async function () {
      await fc.assert(fc.asyncProperty(
        canonicalId.chain(id => fc.tuple(fc.constant(id), variantOf(id))),
        async ([id, variant]) => {
          expect(await registry.normalizeIdentifier(variant)).to.equal(id);
          await registry.connect(vet).registerVet(id, "GP");
          await expect(registry.connect(other).registerVet(variant, "GP"))
            .to.be.revertedWith("PetChainRegistry: license already registered");
        }
      ), { numRuns: 25, seed: 1325 });
    });

    it("distinct canonical identifiers never collide", async function () {
      await fc.assert(fc.asyncProperty(
        canonicalId, canonicalId,
        async (a, b) => {
          fc.pre(a !== b);
          expect(await registry.normalizeIdentifier(a))
            .to.not.equal(await registry.normalizeIdentifier(b));
          await registry.connect(vet).registerVet(a, "GP");
          await registry.connect(other).registerVet(b, "GP");
        }
      ), { numRuns: 25, seed: 1325 });
    });

    it("registerVet rejects invalid identifiers", async function () {
      await expect(registry.connect(vet).registerVet("LIC\u00001", "GP")).to.be.revertedWith(INVALID_CHAR);
      await expect(registry.connect(vet).registerVet("LİC", "GP")).to.be.revertedWith(INVALID_CHAR);
      await expect(registry.connect(vet).registerVet("   ", "GP")).to.be.revertedWith(INVALID_LEN);
    });
  });

  // ---------------------------------------------------------------------------
  // Issue #1326 — failed transfer atomicity
  // ---------------------------------------------------------------------------
  describe("#1326 — transfer atomicity", function () {
    async function snapshot(petId) {
      const pet = await registry.pets(petId);
      return {
        owner: pet.owner,
        active: pet.active,
        ownerPets: (await registry.getPetsByOwner(owner.address)).map(Number),
        otherPets: (await registry.getPetsByOwner(other.address)).map(Number),
        transfers: (await registry.queryFilter(registry.filters.PetTransferred())).length,
      };
    }

    const failures = [
      ["non-owner caller", "PetChainRegistry: not pet owner", async id => [other, id, other.address]],
      ["nonexistent pet", "PetChainRegistry: not pet owner", async () => [owner, 999, other.address]],
      ["paused registry", "EnforcedPause", async id => {
        await registry.connect(admin).pause();
        return [owner, id, other.address];
      }],
      ["zero address recipient", "PetChainRegistry: zero address", async id => [owner, id, ethers.ZeroAddress]],
      ["inactive pet", "PetChainRegistry: pet inactive", async id => {
        await registry.connect(owner).deactivatePet(id);
        return [owner, id, other.address];
      }],
      ["self transfer", "PetChainRegistry: self transfer", async id => [owner, id, owner.address]],
    ];

    for (const [name, reason, arrange] of failures) {
      it(`failed transfer (${name}) changes no owner, index, or event state`, async function () {
        const petId = await registerPet();
        await registerPet(); // a second pet keeps the owner index non-trivial
        const [signer, id, to] = await arrange(petId);
        const before = await snapshot(petId);

        const tx = registry.connect(signer).transferPet(id, to);
        if (reason === "EnforcedPause") {
          await expect(tx).to.be.revertedWithCustomError(registry, reason);
        } else {
          await expect(tx).to.be.revertedWith(reason);
        }

        expect(await snapshot(petId)).to.deep.equal(before);
      });
    }

    it("successful transfer updates all linked reads together", async function () {
      const petId = await registerPet();
      const kept = await registerPet();
      const before = await snapshot(petId);

      await expect(registry.connect(owner).transferPet(petId, other.address))
        .to.emit(registry, "PetTransferred").withArgs(petId, owner.address, other.address);

      const after = await snapshot(petId);
      expect(after.owner).to.equal(other.address);
      expect(after.active).to.equal(true);
      expect(after.ownerPets).to.deep.equal([Number(kept)]);
      expect(after.otherPets).to.deep.equal([Number(petId)]);
      expect(after.transfers).to.equal(before.transfers + 1);
      await expect(registry.connect(owner).transferPet(petId, owner.address))
        .to.be.revertedWith("PetChainRegistry: not pet owner");
    });
  });
});
