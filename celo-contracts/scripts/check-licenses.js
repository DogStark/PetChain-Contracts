#!/usr/bin/env node
/**
 * scripts/check-licenses.js
 *
 * License policy checker for celo-contracts npm dependencies (issue #1349).
 *
 * Usage:
 *   node scripts/check-licenses.js [--json]
 *
 * Reads allowed-licenses.json, runs license-checker against the installed
 * node_modules, and reports every package whose license is not in the approved
 * list. CI violates on the first disallowed package; the output includes the
 * package name, version, resolved license, and the registry source so the
 * dependency, version, source, and violation are all visible in one line.
 *
 * Exit codes:
 *   0 — all licenses approved
 *   1 — one or more violations found
 */

"use strict";

const path = require("path");
const fs = require("fs");
const { execSync } = require("child_process");

const ROOT = path.resolve(__dirname, "..");
const POLICY_FILE = path.join(ROOT, "allowed-licenses.json");
const JSON_MODE = process.argv.includes("--json");

// ---------------------------------------------------------------------------
// Load policy
// ---------------------------------------------------------------------------
let policy;
try {
  policy = JSON.parse(fs.readFileSync(POLICY_FILE, "utf8"));
} catch (err) {
  console.error(`[license-check] ERROR: cannot read policy file ${POLICY_FILE}: ${err.message}`);
  process.exit(1);
}

const allowed = new Set(policy.allowedLicenses || []);
const disallowed = new Set(policy.disallowedLicenses || []);
const exceptions = new Map(
  (policy.allowedExceptions || []).map((e) => [e.name, e])
);

// ---------------------------------------------------------------------------
// Run license-checker
// ---------------------------------------------------------------------------
let licenseData;
try {
  const raw = execSync(
    "npx --yes license-checker --json --production --start " + ROOT,
    { cwd: ROOT, encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] }
  );
  licenseData = JSON.parse(raw);
} catch (err) {
  console.error(`[license-check] ERROR: license-checker failed: ${err.message}`);
  process.exit(1);
}

// ---------------------------------------------------------------------------
// Evaluate each package
// ---------------------------------------------------------------------------
const today = new Date().toISOString().slice(0, 10);
const violations = [];
const warnings = [];

for (const [pkg, info] of Object.entries(licenseData)) {
  // pkg is "name@version"
  const atIdx = pkg.lastIndexOf("@");
  const name = pkg.slice(0, atIdx);
  const version = pkg.slice(atIdx + 1);
  const license = (info.licenses || "UNKNOWN").toString().trim();
  const repository = info.repository || "unknown";

  // Check for an approved exception first.
  if (exceptions.has(name)) {
    const exc = exceptions.get(name);
    if (exc.expires && exc.expires < today) {
      warnings.push(
        `[license-check] WARN  exception expired: ${name}@${version} ` +
        `(license=${license}, expires=${exc.expires}, issue=${exc.issue})`
      );
    }
    continue; // exception covers this package
  }

  // Normalize compound license expressions: "MIT OR Apache-2.0" etc.
  const licenseTokens = license
    .replace(/[()]/g, "")
    .split(/\s+(?:OR|AND)\s+/)
    .map((s) => s.trim());

  const anyDisallowed = licenseTokens.some((l) => disallowed.has(l));
  const allAllowed = licenseTokens.every((l) => allowed.has(l));

  if (anyDisallowed || !allAllowed) {
    violations.push({
      package: name,
      version,
      license,
      repository,
      violation: anyDisallowed ? "disallowed-license" : "unapproved-license",
    });
  }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------
for (const w of warnings) {
  console.warn(w);
}

if (violations.length === 0) {
  console.log(`[license-check] OK — all ${Object.keys(licenseData).length} packages use approved licenses.`);
  process.exit(0);
}

if (JSON_MODE) {
  console.error(JSON.stringify({ violations }, null, 2));
} else {
  console.error(`[license-check] FAIL — ${violations.length} license violation(s):`);
  for (const v of violations) {
    console.error(
      `  ${v.package}@${v.version}  license=${v.license}  source=${v.repository}  violation=${v.violation}`
    );
  }
  console.error(
    "\nTo add an exception, edit celo-contracts/allowed-licenses.json and open a PR.\n" +
    "See docs/dependency-policy.md for the review process."
  );
}

process.exit(1);
