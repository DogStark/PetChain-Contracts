//! Error registry tests.
//!
//! These tests pin the numeric discriminants of contract error codes so that
//! off-chain consumers (SDKs, indexers, retry logic) can rely on stable values.
//! New variants must be appended to the end of an error enum; existing
//! discriminants must never be renumbered.

use crate::insurance::InsuranceError;

/// Helper: assert a variant maps to an exact numeric code.
fn assert_code(err: InsuranceError, expected: u32) {
    assert_eq!(
        err as u32, expected,
        "error discriminant changed; off-chain consumers depend on stable codes"
    );
}

/// Helper: assert retry guidance is stable for a variant.
fn assert_retryable(err: InsuranceError, expected: bool) {
    assert_eq!(
        err.is_retryable(),
        expected,
        "retry guidance changed for {:?}",
        err
    );
}

#[test]
fn insurance_enrollment_error_codes_are_stable() {
    // Enrollment workflow error codes (#1470). Appended after existing codes.
    assert_code(InsuranceError::EnrollmentNotFound, 100);
    assert_code(InsuranceError::InvalidEnrollmentTransition, 101);
    assert_code(InsuranceError::EnrollmentAlreadyExists, 102);
    assert_code(InsuranceError::EnrollmentNotDraft, 103);
    assert_code(InsuranceError::EnrollmentDecisionAlreadyFinal, 104);
    assert_code(InsuranceError::PolicyVersionNotFound, 105);
    assert_code(InsuranceError::PolicyVersionMismatch, 106);
    assert_code(InsuranceError::EnrollmentStateLimitExceeded, 107);
    assert_code(InsuranceError::EnrollmentNoteTooLong, 108);
}

#[test]
fn insurance_coverage_snapshot_error_codes_are_stable() {
    // Coverage/exclusion snapshot error codes (#1472). Appended after #1470.
    assert_code(InsuranceError::SnapshotNotFound, 120);
    assert_code(InsuranceError::SnapshotAlreadyExists, 121);
    assert_code(InsuranceError::SnapshotImmutable, 122);
    assert_code(InsuranceError::SnapshotTooLarge, 123);
    assert_code(InsuranceError::SnapshotVersionMismatch, 124);
}

#[test]
fn insurance_enrollment_retry_guidance_is_stable() {
    // Transient / caller-correctable conditions are retryable.
    assert_retryable(InsuranceError::EnrollmentNotFound, false);
    assert_retryable(InsuranceError::InvalidEnrollmentTransition, false);
    assert_retryable(InsuranceError::EnrollmentAlreadyExists, false);
    assert_retryable(InsuranceError::EnrollmentNotDraft, false);
    assert_retryable(InsuranceError::EnrollmentDecisionAlreadyFinal, false);
    assert_retryable(InsuranceError::PolicyVersionNotFound, true);
    assert_retryable(InsuranceError::PolicyVersionMismatch, false);
    assert_retryable(InsuranceError::EnrollmentStateLimitExceeded, true);
    assert_retryable(InsuranceError::EnrollmentNoteTooLong, false);
}

#[test]
fn insurance_coverage_snapshot_retry_guidance_is_stable() {
    assert_retryable(InsuranceError::SnapshotNotFound, true);
    assert_retryable(InsuranceError::SnapshotAlreadyExists, false);
    assert_retryable(InsuranceError::SnapshotImmutable, false);
    assert_retryable(InsuranceError::SnapshotTooLarge, false);
    assert_retryable(InsuranceError::SnapshotVersionMismatch, false);
}

#[test]
fn insurance_error_codes_are_unique() {
    // Guard against accidental duplicate discriminants when appending variants.
    let codes: [u32; 14] = [
        InsuranceError::EnrollmentNotFound as u32,
        InsuranceError::InvalidEnrollmentTransition as u32,
        InsuranceError::EnrollmentAlreadyExists as u32,
        InsuranceError::EnrollmentNotDraft as u32,
        InsuranceError::EnrollmentDecisionAlreadyFinal as u32,
        InsuranceError::PolicyVersionNotFound as u32,
        InsuranceError::PolicyVersionMismatch as u32,
        InsuranceError::EnrollmentStateLimitExceeded as u32,
        InsuranceError::EnrollmentNoteTooLong as u32,
        InsuranceError::SnapshotNotFound as u32,
        InsuranceError::SnapshotAlreadyExists as u32,
        InsuranceError::SnapshotImmutable as u32,
        InsuranceError::SnapshotTooLarge as u32,
        InsuranceError::SnapshotVersionMismatch as u32,
    ];

    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(
                codes[i], codes[j],
                "duplicate error discriminant detected at indices {} and {}",
                i, j
            );
        }
    }
}