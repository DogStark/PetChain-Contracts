# Security Policy

`DogStark/PetChain-Contracts` takes the security of our smart contracts and protocol seriously. We appreciate the efforts of security researchers and community members in keeping our ecosystem safe.

## 1. Supported Versions

We apply security updates and bug fixes to the following active versions:

| Version | Supported |
| ------- | --------- |
| Mainnet | :white_check_mark: |
| Testnet | :white_check_mark: |
| Development / Unreleased | :x: |

## 2. Reporting a Vulnerability

**DO NOT create a public GitHub issue, pull request, or discussion for security vulnerabilities.** Public disclosure creates unnecessary risk for our users and funds.

### Contact Information

Please submit all vulnerability reports privately using one of the following channels:

* **Email:** [security@petchain.dog](mailto:security@petchain.dog) *(Update with your team's security email address)*
* **Encrypted Reporting:** [Insert PGP Key or Security Portal Link if applicable]

### What to Include in Your Report

To help us triage and respond to your report quickly, please include:

1. **Description:** A detailed explanation of the issue and potential impact.
2. **Affected Contracts:** Specific smart contract names, functions, or deployed addresses.
3. **Proof of Concept (PoC):** Steps to reproduce or script demonstrating the flaw (using testnets or a local fork environment like Foundry/Hardhat).
4. **Remediation Suggestion:** Any proposed fix or mitigation, if available.
5. **Secrets & Keys Warning:** **NEVER** include private keys, live mainnet secrets, or sensitive production credentials in your report.

## 3. SLA & Response Timelines

We adhere to the following Service Level Agreement (SLA) for security reports:

| Milestone | Target Timeframe |
| --------- | ---------------- |
| **Initial Acknowledgment** | Within 24–48 hours |
| **Triage & Risk Assessment** | Within 3–5 business days |
| **Fix Development & Testing** | Within 14 business days (varies by severity) |
| **Public Disclosure** | Coordinated release after resolution |

## 4. Triage States

Once reported, your submission will move through the following defined states:

1. **`Received`**: The report has been delivered and assigned to our security team.
2. **`In Triage`**: The team is evaluating the severity, validity, and impact of the report.
3. **`Confirmed`**: The vulnerability is verified, and a remediation plan is in progress.
4. **`Patched`**: A fix has been deployed to test networks/mainnet.
5. **`Declined`**: The report was determined to be out of scope, invalid, or an intended design feature.

## 5. Disclosure & Bug Bounty

* We follow a **Coordinated Disclosure** policy. Please give our team adequate time to fix the issue before sharing any details publicly.
* If a bug bounty program is active, qualifying reports submitted in accordance with this policy may be eligible for a reward based on severity (evaluated using CVSS v3/SWC registry guidelines).

Thank you for helping secure the PetChain ecosystem!
