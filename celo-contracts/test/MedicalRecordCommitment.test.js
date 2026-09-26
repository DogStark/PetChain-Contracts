const { expect } = require("chai");
const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

// Gas regression budgets for core registry methods (issue #1322).
//
// The baseline lives in celo-contracts/gas-baselines.json and is intentionally
// updated only through a documented process: run `npm run gas:update` (or set
// UPDATE_GAS_BASELINE=1) with a written reason in the commit/PR description.
// CI compares measured gas against the stored budget and fails with the method
// name and call shape when a regression exceeds the allowed tolerance.
const BASELINE_PATH = path.join(__dirname, "..", "gas-baselines.json");
const TOLERANCE_BPS = 500; // 5% allowed drift before a budget failure

function loadBaseline() {
  if (!fs.existsSync(BASELINE_PATH)) {
    return { budgets: {} };
  }
  return JSON.parse(fs.readFileSync(BASELINE_PATH, "utf8"));
}

function saveBaseline(baseline) {
  fs.writeFileSync(BASELINE_PATH, JSON.stringify(baseline, null, 2) + "\n");
}

function assertWithinBudget(baseline, key, shape, gasUsed) {
  const budget = baseline.budgets[key];
  if (!budget) {
    // First run records the budget; subsequent runs enforce it.
    baseline.budgets[key] = { shape, gas: gasUsed };
    return;
  }
  const limit = Math.floor((budget.gas * (10000 + TOLERANCE_BPS)) / 10000);
  expect(
    gasUsed,
    `gas regression for ${key} (${shape}): used ${gasUsed}, budget ${budget.gas}, limit ${limit}`
  ).to.be.lte(limit);
}

describe("MedicalRecordCommitment gas budgets", function () {
  let registry;
  let owner;
  let patient;
  let provider;
  let other;
  let baseline;

  const UPDATE = process.env.UPDATE_GAS_BASELINE === "1";

  before(async function () {
    [owner, patient, provider, other] = await ethers.getSigners();
    const Registry = await ethers.getContractFactory("MedicalRecordCommitment");
    registry = await Registry.deploy();
    await registry.waitForDeployment();
    baseline = loadBaseline();
  });

  after(function () {
    if (UPDATE) {
      saveBaseline(baseline);
    }
  });

  it("measures pet registration gas", async function () {
    const tx = await registry.registerPet(patient.address, "pet-1");
    const receipt = await tx.wait();
    assertWithinBudget(
      baseline,
      "registerPet",
      "registerPet(address,string)",
      Number(receipt.gasUsed)
    );
  });

  it("measures pet transfer gas", async function () {
    await (await registry.registerPet(patient.address, "pet-2")).wait();
    const tx = await registry.transferPet(patient.address, other.address, "pet-2");
    const receipt = await tx.wait();
    assertWithinBudget(
      baseline,
      "transferPet",
      "transferPet(address,address,string)",
      Number(receipt.gasUsed)
    );
  });

  it("measures record commitment gas", async function () {
    const tx = await registry.commitRecord(
      patient.address,
      ethers.keccak256(ethers.toUtf8Bytes("record-1"))
    );
    const receipt = await tx.wait();
    assertWithinBudget(
      baseline,
      "commitRecord",
      "commitRecord(address,bytes32)",
      Number(receipt.gasUsed)
    );
  });

  it("measures read method gas", async function () {
    await (await registry.registerPet(patient.address, "pet-3")).wait();
    const gasUsed = await registry.getPet.estimateGas(patient.address, "pet-3");
    assertWithinBudget(
      baseline,
      "getPet",
      "getPet(address,string)",
      Number(gasUsed)
    );
  });
});
