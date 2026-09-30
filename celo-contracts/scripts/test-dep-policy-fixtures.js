#!/usr/bin/env node
/**
 * scripts/test-dep-policy-fixtures.js
 *
 * Fixture validation for the dependency-policy test suite (issue #1349).
 *
 * Verifies that:
 *   1. The disallowed-npm-license fixture contains all required fields.
 *   2. Every license named in the fixture's disallowedLicensesReference array
 *      is present in allowed-licenses.json [disallowedLicenses].
 *   3. The fixture's simulatedPackage.license is NOT in the approved list.
 *   4. The disallowed-rust-license.toml fixture exists and contains the
 *      expected license identifier and source header.
 *
 * Exit codes:
 *   0 — all assertions pass
 *   1 — one or more assertions failed
 *
 * Usage:
 *   node scripts/test-dep-policy-fixtures.js
 */

"use strict";

const fs = require("fs");
const path = require("path");

const ROOT = path.resolve(__dirname, "..");
const FIXTURE_DIR = path.resolve(ROOT, "..", "fixtures", "dep-policy");
const POLICY_FILE = path.join(ROOT, "allowed-licenses.json");
const RUST_FIXTURE = path.join(FIXTURE_DIR, "disallowed-rust-license.toml");
const NPM_FIXTURE = path.join(FIXTURE_DIR, "disallowed-npm-license.json");

let failures = 0;

function assert(cond, msg) {
  if (!cond) {
    console.error(`  FAIL  ${msg}`);
    failures++;
  } else {
    console.log(`  OK    ${msg}`);
  }
}

function fileExists(filePath) {
  try {
    fs.accessSync(filePath);
    return true;
  } catch {
    return false;
  }
}

console.log("[dep-policy-fixtures] Running fixture integrity checks...\n");

// ---------------------------------------------------------------------------
// 1. Files exist
// ---------------------------------------------------------------------------
console.log("# File existence");
assert(fileExists(RUST_FIXTURE), `disallowed-rust-license.toml exists at ${RUST_FIXTURE}`);
assert(fileExists(NPM_FIXTURE), `disallowed-npm-license.json exists at ${NPM_FIXTURE}`);
assert(fileExists(POLICY_FILE), `allowed-licenses.json exists at ${POLICY_FILE}`);
console.log();

// ---------------------------------------------------------------------------
// 2. Policy file is valid JSON with required fields
// ---------------------------------------------------------------------------
console.log("# Policy file structure");
let policy;
try {
  policy = JSON.parse(fs.readFileSync(POLICY_FILE, "utf8"));
  assert(true, "allowed-licenses.json parses as valid JSON");
} catch (err) {
  assert(false, `allowed-licenses.json parses as valid JSON: ${err.message}`);
  process.exit(1);
}
assert(Array.isArray(policy.allowedLicenses), "policy.allowedLicenses is an array");
assert(Array.isArray(policy.disallowedLicenses), "policy.disallowedLicenses is an array");
assert(Array.isArray(policy.allowedExceptions), "policy.allowedExceptions is an array");
console.log();

// ---------------------------------------------------------------------------
// 3. npm fixture is valid and consistent with the policy
// ---------------------------------------------------------------------------
console.log("# npm fixture integrity");
let npmFixture;
try {
  npmFixture = JSON.parse(fs.readFileSync(NPM_FIXTURE, "utf8"));
  assert(true, "disallowed-npm-license.json parses as valid JSON");
} catch (err) {
  assert(false, `disallowed-npm-license.json parses as valid JSON: ${err.message}`);
  process.exit(1);
}

const requiredFields = ["simulatedPackage", "expectedViolation", "expectedOutput", "mitigation", "disallowedLicensesReference"];
for (const f of requiredFields) {
  assert(f in npmFixture, `fixture has required field '${f}'`);
}

const simLicense = npmFixture.simulatedPackage?.license;
const approvedSet = new Set(policy.allowedLicenses || []);
const disallowedSet = new Set(policy.disallowedLicenses || []);

assert(
  simLicense && !approvedSet.has(simLicense),
  `simulatedPackage.license '${simLicense}' is NOT in the approved list (correct fixture)`
);
assert(
  simLicense && disallowedSet.has(simLicense),
  `simulatedPackage.license '${simLicense}' IS in the disallowed list (correct fixture)`
);

// Every license in the fixture reference must also appear in policy
const fixtureRef = npmFixture.disallowedLicensesReference || [];
for (const lic of fixtureRef) {
  assert(
    disallowedSet.has(lic),
    `fixture reference license '${lic}' exists in policy.disallowedLicenses`
  );
}
console.log();

// ---------------------------------------------------------------------------
// 4. Rust fixture contains expected markers
// ---------------------------------------------------------------------------
console.log("# Rust fixture integrity");
const rustContent = fs.readFileSync(RUST_FIXTURE, "utf8");
assert(rustContent.includes("GPL-3.0-only"), "Rust fixture mentions 'GPL-3.0-only'");
assert(rustContent.includes("crates.io-index"), "Rust fixture mentions the crates.io source URL");
assert(rustContent.includes("error[L002]"), "Rust fixture includes expected cargo-deny error code L002");
assert(rustContent.includes("error[S001]"), "Rust fixture includes expected cargo-deny error code S001");
console.log();

// ---------------------------------------------------------------------------
// Summary
// ---------------------------------------------------------------------------
if (failures === 0) {
  console.log(`[dep-policy-fixtures] All checks passed.`);
  process.exit(0);
} else {
  console.error(`[dep-policy-fixtures] ${failures} check(s) failed.`);
  process.exit(1);
}
