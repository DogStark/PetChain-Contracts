# Insurance Benefit Sections (#1473–#1475)

The benefit-section API adds condition-specific insurance terms without
changing the serialized `InsurancePolicy` or `InsuranceClaim` structs.

## Configuration

The pet owner calls `set_condition_coverage` once per `(pet_id, policy_id,
condition)`. Configuration is immutable after creation. Terms include the
coverage start, waiting-period end, period duration, coverage limit,
deductible, and copay in basis points. The call requires owner authorization;
the referenced policy must be active. A waiting period ending at or after
policy expiry, a zero period/limit, or a copay above 10,000 basis points is
rejected.

There may be at most 32 condition sections per pet and per policy. Condition
names and policy identifiers are limited to 64 bytes. The new APIs search at
most the newest 32 policy entries when resolving a policy id.

## Claim Dates And Limits

The owner submits a condition claim with its service date. Dates before
coverage start, at or after policy expiry, or later than the current ledger
timestamp are invalid. Service dates strictly before the waiting-period end are
rejected; a service date equal to the waiting-period end is covered.

The condition period containing the service date is anchored at the coverage
start. Each `(pet, policy, condition, period_start)` has separate `paid` and
`pending` counters. Submission reserves the insurer payment against
`paid + pending`; settlement moves the reservation to `paid`, and rejection
releases it. A new period gets a new usage key. At most 256 benefit claims can
be submitted per pet and per policy, bounding the number of period-usage
entries.

## Settlement Calculation

The deductible is applied first, capped at the claim amount. Copay is then
calculated on the remaining eligible amount and rounded down to the smallest
asset unit:

`copay = floor((amount - deductible_applied) * copay_bps / 10_000)`

`insurer_payment = amount - deductible_applied - copay`

The contract computes the multiplication in `u128`; all cumulative counter
updates use checked `u64` arithmetic. A zero claim amount is rejected. A
100%-copay claim has zero insurer payment and does not consume the coverage
limit.

Only an admin may finalize a condition claim. It can be finalized once. The
legacy status, appeal, and flagged-claim payout paths reject condition claims
with `BenefitClaimMustUseSettlement`, so these paths cannot skip the reserved
amount accounting.

## Events And Errors

`condition_cover` emits `InsuranceConditionCoverageConfiguredEvent` once when
immutable terms are set. It includes the effective start, waiting-period end,
period, limit, deductible and copay. `get_insurance_benefit_claim` returns the
persisted deductible, copay, insurer-payment and period details.
`benefit_claim` emits
`InsuranceBenefitClaimSubmittedEvent` with the service date and calculated
deductible, copay, and insurer payment.
`benefit_settled` emits `InsuranceBenefitSettledEvent` with the period and the
approval result. Both use `EVENT_SCHEMA_VERSION`.

New `ContractError` codes are append-only:

| Code | Error | Meaning |
|---:|---|---|
| 209 | `BenefitConditionNotFound` | No terms exist for the condition. |
| 210 | `BenefitConditionLimitReached` | Policy already has 32 conditions. |
| 211 | `BenefitWaitingPeriod` | Service date falls before waiting-period end. |
| 212 | `BenefitInvalidServiceDate` | Date is before start, after expiry, or in the future. |
| 213 | `BenefitCoverageLimitExceeded` | Paid plus pending would exceed the period limit. |
| 214 | `BenefitClaimNotPending` | Claim has no valid pending reservation. |
| 215 | `BenefitClaimAlreadyFinalized` | Claim was already finalized. |
| 216 | `BenefitInvalidTerms` | Terms are inconsistent or outside allowed bounds. |
| 217 | `BenefitClaimLimitReached` | Policy already has 256 benefit claims. |
| 218 | `BenefitClaimMustUseSettlement` | Legacy lifecycle method cannot process this claim. |
| 219 | `BenefitTermsAlreadyConfigured` | Configured terms are immutable. |
| 220 | `EnrollmentNotFound` | Enrollment does not exist. |
| 221 | `EnrollmentInvalidTransition` | Enrollment transition is not permitted. |
| 222 | `EnrollmentAlreadyDecided` | Enrollment decision has already been applied. |
| 223 | `EnrollmentLimitReached` | Per-pet enrollment cap is reached. |
| 224 | `EnrollmentSnapshotTooLarge` | Enrollment coverage snapshot exceeds its bound. |
| 225 | `InsurancePolicyInactive` | Policy is inactive or expired. |
| 226 | `InsurancePolicyNotFound` | Policy id is not registered for this pet. |
| 227 | `BenefitClaimLimitPerPetReached` | Pet already has 256 benefit claims. |

## Storage Compatibility

New `InsuranceKey` variants are appended for condition terms, period usage,
policy and pet condition counts, benefit-claim metadata, and bounded policy and
pet claim counts. No existing storage struct or key changes shape. New storage
is created only as users configure condition sections and submit claims; the
explicit limits above bound benefit state growth. No storage migration or
schema version bump is required. `InvalidTimestamp` remains at its
published code 169; `StaleMigration` remains 170.