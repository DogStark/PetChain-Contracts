# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, please report it privately:

1. **DO NOT** create a public GitHub issue
2. Email: [security@petchain.dog](mailto:security@petchain.dog)
3. Include:
   - Description of the vulnerability and its potential impact
   - Affected contracts, functions, or deployed addresses
   - Steps to reproduce or a proof of concept (use testnets or a local fork, e.g. Foundry/Hardhat)
   - Suggested fix or mitigation (if any)
4. **NEVER** include private keys, live mainnet secrets, or sensitive production credentials in your report.

## Response Timeline

- **24 hours**: Initial response acknowledging receipt
- **72 hours**: Initial assessment and severity classification
- **7 days**: Detailed response with fix timeline

## Triage States

Reports move through these states:

1. **`Received`**: The report has been delivered and assigned to the security team.
2. **`In Triage`**: The team is evaluating the severity, validity, and impact of the report.
3. **`Confirmed`**: The vulnerability is verified, and a remediation plan is in progress.
4. **`Patched`**: A fix has been deployed to test networks or mainnet.
5. **`Declined`**: The report was determined to be out of scope, invalid, or an intended design feature.

## Disclosure & Bug Bounty

- We follow a **Coordinated Disclosure** policy. Please give the team adequate time to fix an issue before sharing any details publicly.
- If a bug bounty program is active, qualifying reports submitted under this policy may be eligible for a reward based on severity (evaluated using CVSS v3 / SWC registry guidelines).

## Security Best Practices

When contributing:
- Always validate inputs
- Use proper authentication checks
- Follow principle of least privilege
- Test edge cases and error conditions
- Review code for potential vulnerabilities

## Encryption Key Derivation

Sensitive pet and owner fields use a deterministic key derived at runtime from:
- a fixed domain separator (`petchain:encryption-key:v1`)
- the current contract address
- admin context (legacy single admin or first multisig admin when configured)

This replaces the previous static all-zero key. As a result:
- encrypted storage no longer mirrors plaintext by default
- key material is contract-scoped and not hardcoded
- rotating admin configuration can change derived key context, so migrations/rollouts should account for backward compatibility expectations

## Audit Record

| Field    | Value |
|----------|-------|
| **Date** | 2026-05-29 |
| **Scope** | `stellar-contracts/src/lib.rs` |
| **Auditor** | Internal |
| **Result** | All 23 security checklist items pass. Fixed: temp vet access expiry now enforced on all read paths via `check_and_expire_access`; scope escalation prevented in `grant_temp_vet_access` with explicit `ScopeEscalation` error. |

The full checklist was confirmed internally and is not published in this repository.
