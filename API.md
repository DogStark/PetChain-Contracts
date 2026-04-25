# PetChain Smart Contract API Reference

## Overview

The PetChain smart contract suite manages pet registration, medical records, ownership, and related services on the Stellar (Soroban) network. This document covers all public functions in the main `PetChainContract` and the `VetRegistryContract`.

---

## Table of Contents

1. [Contract Statistics](#1-contract-statistics)
2. [Admin & Multisig Management](#2-admin--multisig-management)
3. [Pet Management](#3-pet-management)
4. [Owner Management](#4-owner-management)
5. [Vet Management](#5-vet-management)
6. [Medical Records](#6-medical-records)
7. [Vaccinations](#7-vaccinations)
8. [Lab Results](#8-lab-results)
9. [Medications](#9-medications)
10. [Treatments](#10-treatments)
11. [Insurance](#11-insurance)
12. [Grooming](#12-grooming)
13. [Nutrition](#13-nutrition)
14. [Behavior & Training](#14-behavior--training)
15. [Activity Tracking](#15-activity-tracking)
16. [Breeding](#16-breeding)
17. [Access Control](#17-access-control)
18. [Emergency Contacts](#18-emergency-contacts)
19. [Lost Pet Alerts](#19-lost-pet-alerts)
20. [Tag Linking](#20-tag-linking)
21. [Consent Management](#21-consent-management)
22. [Vet Availability](#22-vet-availability)
23. [Vet Reviews](#23-vet-reviews)
24. [Pet Multisig Transfer](#24-pet-multisig-transfer)
25. [Contract Upgrade](#25-contract-upgrade)
26. [Vet Registry Contract](#26-vet-registry-contract)

---

## 1. Contract Statistics

### `get_total_pets`

Returns the total number of pets ever registered.

- **Parameters:** `env: Env`
- **Returns:** `u64`

### `get_species_count`

Returns the number of registered pets for a given species.

- **Parameters:** `env: Env`, `species: String`
- **Returns:** `u64`

### `get_active_pets_count`

Returns the number of currently active (non-archived) pets.

- **Parameters:** `env: Env`
- **Returns:** `u64`

---

## 2. Admin & Multisig Management

### `init_admin`

Initialize a single admin for the contract.

- **Auth:** `admin` must sign
- **Parameters:** `env: Env`, `admin: Address`
- **Panics:** If admin already set

### `init_multisig`

Initialize multi-signature admin control.

- **Auth:** `invoker` must sign and must be in `admins` list
- **Parameters:** `env: Env`, `invoker: Address`, `admins: Vec<Address>`, `threshold: u32`
- **Panics:** If admin already set, threshold is 0 or > admins.len(), or invoker not in list

### `propose_action`

Propose a multi-signature admin action.

- **Auth:** `proposer` must be an admin
- **Parameters:** `env: Env`, `proposer: Address`, `action: ProposalAction`, `expires_in: u64`
- **Returns:** `u64` — proposal ID

### `approve_proposal`

Approve a pending multi-signature proposal.

- **Auth:** `admin` must be an admin
- **Parameters:** `env: Env`, `admin: Address`, `proposal_id: u64`
- **Panics:** If proposal not found, already executed, or expired

### `execute_proposal`

Execute a proposal once threshold approvals are met.

- **Parameters:** `env: Env`, `proposal_id: u64`
- **Panics:** If threshold not met, already executed, or expired

### `get_proposal`

Initialize multi-signature admin control.

**Auth:** `invoker` must sign and must be in `admins` list

**Parameters:**

- `env: Env`
- `invoker: Address`
- `admins: Vec<Address>`
- `threshold: u32`

**Panics:** If admin already set, threshold is 0 or > admins.len(), or invoker not in list

---

### `propose_action`

Propose a multi-signature admin action.

**Auth:** `proposer` must be an admin

**Parameters:**

## Statistics

#### `get_vet_stats`
Returns the activity statistics for a given vet.

**Parameters:**
- `env: Env` - Contract environment
- `vet_address: Address` - The vet's address

**Returns:** `VetStats` - Stats struct with the following fields:
- `total_records: u64` - Total medical records added by this vet
- `total_vaccinations: u64` - Total vaccinations administered
- `total_treatments: u64` - Total treatments recorded
- `pets_treated: u64` - Number of unique pets treated

Returns a zeroed `VetStats` if the vet has no recorded activity.

**Example:**
```rust
let stats = client.get_vet_stats(&vet_address);
```

## Future Enhancements

**Returns:** `u64` —
