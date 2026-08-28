//! Discriminant/variant stability tests for every storage-key enum
//! (Issue #1148).
//!
//! Each `*_tag` function below is an *exhaustive* match from a key enum to
//! its variant name. It never needs to be called to do its job: if a
//! future change renames, removes, or adds a variant without updating the
//! match here, the match becomes non-exhaustive (or references a variant
//! that no longer exists) and **fails to compile**. That forces anyone
//! changing a storage-key enum to consciously update this file.
//!
//! The accompanying `#[test]` functions pin the tag returned for each
//! variant that can be constructed without dummy data (fieldless
//! variants), as a runtime double-check alongside the compile-time guard.
//!
//! Append-only rule: new variants must be *added*, never inserted in place
//! of a renamed/removed one. See docs/abi-migrations.md for the process to
//! follow when an intentional change is needed.

use crate::*;

#[allow(dead_code)]
fn insurance_key_tag(v: &InsuranceKey) -> &'static str {
    match v {
        InsuranceKey::Policy(_) => "Policy",
        InsuranceKey::Claim(_) => "Claim",
        InsuranceKey::ClaimCount => "ClaimCount",
        InsuranceKey::PetClaimCount(_) => "PetClaimCount",
        InsuranceKey::PetClaimIndex(_) => "PetClaimIndex",
        InsuranceKey::PetPolicyCount(_) => "PetPolicyCount",
        InsuranceKey::PetPolicyIndex(_) => "PetPolicyIndex",
        InsuranceKey::FlaggedClaimCount => "FlaggedClaimCount",
        InsuranceKey::FlaggedClaimIndex(_) => "FlaggedClaimIndex",
    }
}

#[test]
fn insurance_key_variant_tags_are_pinned() {
    assert_eq!(insurance_key_tag(&InsuranceKey::ClaimCount), "ClaimCount");
    assert_eq!(insurance_key_tag(&InsuranceKey::FlaggedClaimCount), "FlaggedClaimCount");
}

#[allow(dead_code)]
fn behavior_key_tag(v: &BehaviorKey) -> &'static str {
    match v {
        BehaviorKey::BehaviorRecord(_) => "BehaviorRecord",
        BehaviorKey::BehaviorRecordCount => "BehaviorRecordCount",
        BehaviorKey::PetBehaviorCount(_) => "PetBehaviorCount",
        BehaviorKey::PetBehaviorIndex(_) => "PetBehaviorIndex",
        BehaviorKey::TrainingMilestone(_) => "TrainingMilestone",
        BehaviorKey::TrainingMilestoneCount => "TrainingMilestoneCount",
        BehaviorKey::PetMilestoneCount(_) => "PetMilestoneCount",
        BehaviorKey::PetMilestoneIndex(_) => "PetMilestoneIndex",
    }
}

#[test]
fn behavior_key_variant_tags_are_pinned() {
    assert_eq!(behavior_key_tag(&BehaviorKey::BehaviorRecordCount), "BehaviorRecordCount");
    assert_eq!(behavior_key_tag(&BehaviorKey::TrainingMilestoneCount), "TrainingMilestoneCount");
}

#[allow(dead_code)]
fn activity_key_tag(v: &ActivityKey) -> &'static str {
    match v {
        ActivityKey::ActivityRecord(_) => "ActivityRecord",
        ActivityKey::ActivityRecordCount => "ActivityRecordCount",
        ActivityKey::PetActivityCount(_) => "PetActivityCount",
        ActivityKey::PetActivityIndex(_) => "PetActivityIndex",
        ActivityKey::PetActivityStreak(_) => "PetActivityStreak",
        ActivityKey::PetStreakLastRecordDate(_) => "PetStreakLastRecordDate",
        ActivityKey::ActivityIdempotencyKey(_) => "ActivityIdempotencyKey",
        ActivityKey::IdempotencyWindow => "IdempotencyWindow",
    }
}

#[test]
fn activity_key_variant_tags_are_pinned() {
    assert_eq!(activity_key_tag(&ActivityKey::ActivityRecordCount), "ActivityRecordCount");
    assert_eq!(activity_key_tag(&ActivityKey::IdempotencyWindow), "IdempotencyWindow");
}

#[allow(dead_code)]
fn breeding_key_tag(v: &BreedingKey) -> &'static str {
    match v {
        BreedingKey::BreedingRecord(_) => "BreedingRecord",
        BreedingKey::BreedingRecordCount => "BreedingRecordCount",
        BreedingKey::PetBreedingCount(_) => "PetBreedingCount",
        BreedingKey::PetBreedingIndex(_) => "PetBreedingIndex",
        BreedingKey::PetOffspringCount(_) => "PetOffspringCount",
        BreedingKey::PetOffspringIndex(_) => "PetOffspringIndex",
        BreedingKey::ParentPair(_) => "ParentPair",
        BreedingKey::LineageDepth(_) => "LineageDepth",
        BreedingKey::BreedingOffspringCount(_) => "BreedingOffspringCount",
        BreedingKey::BreedingOffspringIndex(_) => "BreedingOffspringIndex",
    }
}

#[test]
fn breeding_key_variant_tags_are_pinned() {
    assert_eq!(breeding_key_tag(&BreedingKey::BreedingRecordCount), "BreedingRecordCount");
}

#[allow(dead_code)]
fn genetics_key_tag(v: &GeneticsKey) -> &'static str {
    match v {
        GeneticsKey::PetTraits(_) => "PetTraits",
        GeneticsKey::PredictedTraits(_) => "PredictedTraits",
    }
}

#[allow(dead_code)]
fn grooming_key_tag(v: &GroomingKey) -> &'static str {
    match v {
        GroomingKey::GroomingRecord(_) => "GroomingRecord",
        GroomingKey::GroomingRecordCount => "GroomingRecordCount",
        GroomingKey::PetGroomingCount(_) => "PetGroomingCount",
        GroomingKey::PetGroomingIndex(_) => "PetGroomingIndex",
        GroomingKey::Groomer(_) => "Groomer",
        GroomingKey::GroomerRatingCount => "GroomerRatingCount",
        GroomingKey::RecurringSchedule(_) => "RecurringSchedule",
        GroomingKey::RecurringScheduleCount => "RecurringScheduleCount",
        GroomingKey::PetScheduleCount(_) => "PetScheduleCount",
        GroomingKey::PetScheduleIndex(_) => "PetScheduleIndex",
        GroomingKey::GroomerSlotIndex(_) => "GroomerSlotIndex",
        GroomingKey::GroomerSlotCount(_) => "GroomerSlotCount",
    }
}

#[test]
fn grooming_key_variant_tags_are_pinned() {
    assert_eq!(grooming_key_tag(&GroomingKey::GroomingRecordCount), "GroomingRecordCount");
    assert_eq!(grooming_key_tag(&GroomingKey::GroomerRatingCount), "GroomerRatingCount");
    assert_eq!(grooming_key_tag(&GroomingKey::RecurringScheduleCount), "RecurringScheduleCount");
}

#[allow(dead_code)]
fn error_registry_key_tag(v: &ErrorRegistryKey) -> &'static str {
    match v {
        ErrorRegistryKey::ErrorMessage(_) => "ErrorMessage",
        ErrorRegistryKey::SupportedLanguages => "SupportedLanguages",
    }
}

#[test]
fn error_registry_key_variant_tags_are_pinned() {
    assert_eq!(error_registry_key_tag(&ErrorRegistryKey::SupportedLanguages), "SupportedLanguages");
}

#[allow(dead_code)]
fn nutrition_key_tag(v: &NutritionKey) -> &'static str {
    match v {
        NutritionKey::DietPlan(_) => "DietPlan",
        NutritionKey::DietPlanCount => "DietPlanCount",
        NutritionKey::PetDietCount(_) => "PetDietCount",
        NutritionKey::PetDietByIndex(_) => "PetDietByIndex",
        NutritionKey::WeightEntry(_) => "WeightEntry",
        NutritionKey::WeightCount => "WeightCount",
        NutritionKey::PetWeightCount(_) => "PetWeightCount",
        NutritionKey::PetWeightByIndex(_) => "PetWeightByIndex",
        NutritionKey::NutritionVersion(_) => "NutritionVersion",
        NutritionKey::PetNutritionVersionCount(_) => "PetNutritionVersionCount",
        NutritionKey::CurrentNutritionVersion(_) => "CurrentNutritionVersion",
        NutritionKey::DailyNutritionSummary(_) => "DailyNutritionSummary",
        NutritionKey::NutritionPlan(_) => "NutritionPlan",
        NutritionKey::NutritionPlanCount => "NutritionPlanCount",
        NutritionKey::PetNutritionPlanCount(_) => "PetNutritionPlanCount",
        NutritionKey::PetNutritionPlanIndex(_) => "PetNutritionPlanIndex",
    }
}

#[test]
fn nutrition_key_variant_tags_are_pinned() {
    assert_eq!(nutrition_key_tag(&NutritionKey::DietPlanCount), "DietPlanCount");
    assert_eq!(nutrition_key_tag(&NutritionKey::WeightCount), "WeightCount");
    assert_eq!(nutrition_key_tag(&NutritionKey::NutritionPlanCount), "NutritionPlanCount");
}

#[allow(dead_code)]
fn data_key_tag(v: &DataKey) -> &'static str {
    match v {
        DataKey::Pet(_) => "Pet",
        DataKey::PetCount => "PetCount",
        DataKey::PetOwner(_) => "PetOwner",
        DataKey::OwnerPetIndex(_) => "OwnerPetIndex",
        DataKey::PetCountByOwner(_) => "PetCountByOwner",
        DataKey::SpeciesPetCount(_) => "SpeciesPetCount",
        DataKey::SpeciesPetIndex(_) => "SpeciesPetIndex",
        DataKey::Vet(_) => "Vet",
        DataKey::VetLicense(_) => "VetLicense",
        DataKey::VetCount => "VetCount",
        DataKey::VetIndex(_) => "VetIndex",
        DataKey::Admin => "Admin",
        DataKey::VetLicenseVerified(_) => "VetLicenseVerified",
        DataKey::VetSpecializations(_) => "VetSpecializations",
        DataKey::ContractVersion => "ContractVersion",
        DataKey::AccessGrant(_) => "AccessGrant",
        DataKey::AccessGrantCount(_) => "AccessGrantCount",
        DataKey::AccessGrantIndex(_) => "AccessGrantIndex",
        DataKey::PetDelegationCount(_) => "PetDelegationCount",
        DataKey::DecryptionToken(_) => "DecryptionToken",
        DataKey::EmergencyAccessLogs(_) => "EmergencyAccessLogs",
        DataKey::EmergencyAuditLog(_) => "EmergencyAuditLog",
        DataKey::EmergencyResponders(_) => "EmergencyResponders",
        DataKey::EmergencyNotifyRateLimit(_) => "EmergencyNotifyRateLimit",
        DataKey::BreedMetadata(_) => "BreedMetadata",
        DataKey::SpeciesBreedList(_) => "SpeciesBreedList",
        DataKey::CallerNonce(_) => "CallerNonce",
        DataKey::ClaimDocuments(_) => "ClaimDocuments",
        DataKey::PetStorageUsage(_) => "PetStorageUsage",
        DataKey::PetStorageQuota(_) => "PetStorageQuota",
        DataKey::GlobalStorageQuota => "GlobalStorageQuota",
        DataKey::NonceHistory(_) => "NonceHistory",
        DataKey::NonceMaxUse(_) => "NonceMaxUse",
        DataKey::NonceUsage(_) => "NonceUsage",
        DataKey::RetentionPeriod => "RetentionPeriod",
        DataKey::MaxSubscriptionsPerAddress => "MaxSubscriptionsPerAddress",
    }
}

#[test]
fn data_key_variant_tags_are_pinned() {
    assert_eq!(data_key_tag(&DataKey::PetCount), "PetCount");
    assert_eq!(data_key_tag(&DataKey::VetCount), "VetCount");
    assert_eq!(data_key_tag(&DataKey::Admin), "Admin");
    assert_eq!(data_key_tag(&DataKey::ContractVersion), "ContractVersion");
    assert_eq!(data_key_tag(&DataKey::GlobalStorageQuota), "GlobalStorageQuota");
    assert_eq!(data_key_tag(&DataKey::RetentionPeriod), "RetentionPeriod");
    assert_eq!(data_key_tag(&DataKey::MaxSubscriptionsPerAddress), "MaxSubscriptionsPerAddress");
}

#[allow(dead_code)]
fn treatment_key_tag(v: &TreatmentKey) -> &'static str {
    match v {
        TreatmentKey::Treatment(_) => "Treatment",
        TreatmentKey::TreatmentCount => "TreatmentCount",
        TreatmentKey::PetTreatmentCount(_) => "PetTreatmentCount",
        TreatmentKey::PetTreatmentIndex(_) => "PetTreatmentIndex",
    }
}

#[test]
fn treatment_key_variant_tags_are_pinned() {
    assert_eq!(treatment_key_tag(&TreatmentKey::TreatmentCount), "TreatmentCount");
}

#[allow(dead_code)]
fn subscription_key_tag(v: &SubscriptionKey) -> &'static str {
    match v {
        SubscriptionKey::Subscription(_) => "Subscription",
        SubscriptionKey::SubscriptionCount => "SubscriptionCount",
        SubscriptionKey::SubscriberSubscriptionCount(_) => "SubscriberSubscriptionCount",
        SubscriptionKey::SubscriberSubscriptionIndex(_) => "SubscriberSubscriptionIndex",
    }
}

#[test]
fn subscription_key_variant_tags_are_pinned() {
    assert_eq!(subscription_key_tag(&SubscriptionKey::SubscriptionCount), "SubscriptionCount");
}

#[allow(dead_code)]
fn tag_key_tag(v: &TagKey) -> &'static str {
    match v {
        TagKey::Tag(_) => "Tag",
        TagKey::PetTagId(_) => "PetTagId",
        TagKey::TagNonce => "TagNonce",
        TagKey::PetTagCount => "PetTagCount",
    }
}

#[test]
fn tag_key_variant_tags_are_pinned() {
    assert_eq!(tag_key_tag(&TagKey::TagNonce), "TagNonce");
    assert_eq!(tag_key_tag(&TagKey::PetTagCount), "PetTagCount");
}

#[allow(dead_code)]
fn medical_key_tag(v: &MedicalKey) -> &'static str {
    match v {
        MedicalKey::LabResult(_) => "LabResult",
        MedicalKey::LabResultCount => "LabResultCount",
        MedicalKey::PetLabResultIndex(_) => "PetLabResultIndex",
        MedicalKey::PetLabResultCount(_) => "PetLabResultCount",
        MedicalKey::MedicalRecord(_) => "MedicalRecord",
        MedicalKey::MedicalRecordCount => "MedicalRecordCount",
        MedicalKey::PetMedicalRecordIndex(_) => "PetMedicalRecordIndex",
        MedicalKey::PetMedicalRecordCount(_) => "PetMedicalRecordCount",
        MedicalKey::MedicalRecordAmendment(_) => "MedicalRecordAmendment",
        MedicalKey::MedicalRecordAmendmentCount(_) => "MedicalRecordAmendmentCount",
        MedicalKey::KeywordRecordCount(_) => "KeywordRecordCount",
        MedicalKey::KeywordRecordIndex(_) => "KeywordRecordIndex",
        MedicalKey::GlobalMedication(_) => "GlobalMedication",
        MedicalKey::MedicationCount => "MedicationCount",
        MedicalKey::PetMedicationCount(_) => "PetMedicationCount",
        MedicalKey::PetMedicationIndex(_) => "PetMedicationIndex",
        MedicalKey::Vaccination(_) => "Vaccination",
        MedicalKey::VaccinationCount => "VaccinationCount",
        MedicalKey::PetVaccinationCount(_) => "PetVaccinationCount",
        MedicalKey::PetVaccinationByIndex(_) => "PetVaccinationByIndex",
        MedicalKey::CertificateAnchor(_) => "CertificateAnchor",
        MedicalKey::ScannerRegistry => "ScannerRegistry",
        MedicalKey::RetentionPeriod => "RetentionPeriod",
    }
}

#[test]
fn medical_key_variant_tags_are_pinned() {
    assert_eq!(medical_key_tag(&MedicalKey::LabResultCount), "LabResultCount");
    assert_eq!(medical_key_tag(&MedicalKey::MedicalRecordCount), "MedicalRecordCount");
    assert_eq!(medical_key_tag(&MedicalKey::MedicationCount), "MedicationCount");
    assert_eq!(medical_key_tag(&MedicalKey::VaccinationCount), "VaccinationCount");
    assert_eq!(medical_key_tag(&MedicalKey::ScannerRegistry), "ScannerRegistry");
    assert_eq!(medical_key_tag(&MedicalKey::RetentionPeriod), "RetentionPeriod");
}

#[allow(dead_code)]
fn review_key_tag(v: &ReviewKey) -> &'static str {
    match v {
        ReviewKey::VetReview(_) => "VetReview",
        ReviewKey::VetReviewCount => "VetReviewCount",
        ReviewKey::VetReviewByVetIndex(_) => "VetReviewByVetIndex",
        ReviewKey::VetReviewCountByVet(_) => "VetReviewCountByVet",
        ReviewKey::VetReviewByOwnerVet(_) => "VetReviewByOwnerVet",
    }
}

#[test]
fn review_key_variant_tags_are_pinned() {
    assert_eq!(review_key_tag(&ReviewKey::VetReviewCount), "VetReviewCount");
}

#[allow(dead_code)]
fn alert_key_tag(v: &AlertKey) -> &'static str {
    match v {
        AlertKey::LostPetAlert(_) => "LostPetAlert",
        AlertKey::LostPetAlertCount => "LostPetAlertCount",
        AlertKey::ActiveLostPetAlerts => "ActiveLostPetAlerts",
        AlertKey::AlertSightings(_) => "AlertSightings",
    }
}

#[test]
fn alert_key_variant_tags_are_pinned() {
    assert_eq!(alert_key_tag(&AlertKey::LostPetAlertCount), "LostPetAlertCount");
    assert_eq!(alert_key_tag(&AlertKey::ActiveLostPetAlerts), "ActiveLostPetAlerts");
}

#[allow(dead_code)]
fn consent_key_tag(v: &ConsentKey) -> &'static str {
    match v {
        ConsentKey::Consent(_) => "Consent",
        ConsentKey::ConsentCount => "ConsentCount",
        ConsentKey::PetConsentIndex(_) => "PetConsentIndex",
        ConsentKey::PetConsentCount(_) => "PetConsentCount",
    }
}

#[test]
fn consent_key_variant_tags_are_pinned() {
    assert_eq!(consent_key_tag(&ConsentKey::ConsentCount), "ConsentCount");
}

#[allow(dead_code)]
fn cross_chain_key_tag(v: &CrossChainKey) -> &'static str {
    match v {
        CrossChainKey::PetChainMapping(_) => "PetChainMapping",
        CrossChainKey::ChainLookup(_) => "ChainLookup",
    }
}

#[allow(dead_code)]
fn system_key_tag(v: &SystemKey) -> &'static str {
    match v {
        SystemKey::PetOwnershipRecord(_) => "PetOwnershipRecord",
        SystemKey::OwnershipRecordCount => "OwnershipRecordCount",
        SystemKey::PetOwnershipRecordCount(_) => "PetOwnershipRecordCount",
        SystemKey::PetOwnershipRecordIndex(_) => "PetOwnershipRecordIndex",
        SystemKey::Admins => "Admins",
        SystemKey::AdminThreshold => "AdminThreshold",
        SystemKey::AdminQuorumPercent => "AdminQuorumPercent",
        SystemKey::PendingConfig => "PendingConfig",
        SystemKey::Proposal(_) => "Proposal",
        SystemKey::ProposalCount => "ProposalCount",
        SystemKey::PendingThresholdChange => "PendingThresholdChange",
        SystemKey::AdminTimelockConfig => "AdminTimelockConfig",
        SystemKey::ProposalVeto(_) => "ProposalVeto",
        SystemKey::ProposalVetoCount(_) => "ProposalVetoCount",
        SystemKey::VetAvailability(_) => "VetAvailability",
        SystemKey::VetAvailabilityCount(_) => "VetAvailabilityCount",
        SystemKey::VetAvailabilityByDate(_) => "VetAvailabilityByDate",
        SystemKey::PetMultisigConfig(_) => "PetMultisigConfig",
        SystemKey::PetTransferProposal(_) => "PetTransferProposal",
        SystemKey::PetTransferProposalCount => "PetTransferProposalCount",
        SystemKey::PetActiveProposals(_) => "PetActiveProposals",
        SystemKey::EncryptionNonceCounter => "EncryptionNonceCounter",
        SystemKey::StatCacheTTL => "StatCacheTTL",
        SystemKey::StatCache(_) => "StatCache",
        SystemKey::LabThreshold => "LabThreshold",
        SystemKey::CustodyChain(_) => "CustodyChain",
        SystemKey::HealthScoreCacheTtl => "HealthScoreCacheTtl",
        SystemKey::StatisticsSnapshot(_) => "StatisticsSnapshot",
        SystemKey::SnapshotCount => "SnapshotCount",
        SystemKey::SnapshotIndex(_) => "SnapshotIndex",
        SystemKey::UpgradeProposal(_) => "UpgradeProposal",
        SystemKey::UpgradeProposalCount => "UpgradeProposalCount",
        SystemKey::RollbackDeadline => "RollbackDeadline",
        SystemKey::PreviousWasmHash => "PreviousWasmHash",
        SystemKey::StorageVersion => "StorageVersion",
        SystemKey::AdminActivityLog(_) => "AdminActivityLog",
        SystemKey::AdminActivityCount => "AdminActivityCount",
    }
}

#[test]
fn system_key_variant_tags_are_pinned() {
    assert_eq!(system_key_tag(&SystemKey::OwnershipRecordCount), "OwnershipRecordCount");
    assert_eq!(system_key_tag(&SystemKey::Admins), "Admins");
    assert_eq!(system_key_tag(&SystemKey::AdminThreshold), "AdminThreshold");
    assert_eq!(system_key_tag(&SystemKey::AdminQuorumPercent), "AdminQuorumPercent");
    assert_eq!(system_key_tag(&SystemKey::PendingConfig), "PendingConfig");
    assert_eq!(system_key_tag(&SystemKey::ProposalCount), "ProposalCount");
    assert_eq!(system_key_tag(&SystemKey::PendingThresholdChange), "PendingThresholdChange");
    assert_eq!(system_key_tag(&SystemKey::AdminTimelockConfig), "AdminTimelockConfig");
    assert_eq!(system_key_tag(&SystemKey::PetTransferProposalCount), "PetTransferProposalCount");
    assert_eq!(system_key_tag(&SystemKey::EncryptionNonceCounter), "EncryptionNonceCounter");
    assert_eq!(system_key_tag(&SystemKey::StatCacheTTL), "StatCacheTTL");
    assert_eq!(system_key_tag(&SystemKey::LabThreshold), "LabThreshold");
    assert_eq!(system_key_tag(&SystemKey::HealthScoreCacheTtl), "HealthScoreCacheTtl");
    assert_eq!(system_key_tag(&SystemKey::SnapshotCount), "SnapshotCount");
    assert_eq!(system_key_tag(&SystemKey::UpgradeProposalCount), "UpgradeProposalCount");
    assert_eq!(system_key_tag(&SystemKey::RollbackDeadline), "RollbackDeadline");
    assert_eq!(system_key_tag(&SystemKey::PreviousWasmHash), "PreviousWasmHash");
    assert_eq!(system_key_tag(&SystemKey::StorageVersion), "StorageVersion");
    assert_eq!(system_key_tag(&SystemKey::AdminActivityCount), "AdminActivityCount");
}

#[allow(dead_code)]
fn vet_key_tag(v: &VetKey) -> &'static str {
    match v {
        VetKey::VetStats(_) => "VetStats",
        VetKey::VetPetTreated(_) => "VetPetTreated",
        VetKey::VetPetCount(_) => "VetPetCount",
        VetKey::VetTreatmentIndex(_) => "VetTreatmentIndex",
        VetKey::VetTreatmentCount(_) => "VetTreatmentCount",
        VetKey::VetVaccinationIndex(_) => "VetVaccinationIndex",
        VetKey::VetVaccinationCount(_) => "VetVaccinationCount",
    }
}

#[allow(dead_code)]
fn stats_key_tag(v: &StatsKey) -> &'static str {
    match v {
        StatsKey::ActivePetsCount => "ActivePetsCount",
    }
}

#[test]
fn stats_key_variant_tags_are_pinned() {
    assert_eq!(stats_key_tag(&StatsKey::ActivePetsCount), "ActivePetsCount");
}

#[allow(dead_code)]
fn stat_series_key_tag(v: &StatSeriesKey) -> &'static str {
    match v {
        StatSeriesKey::Count(_) => "Count",
        StatSeriesKey::Point(_) => "Point",
    }
}

#[allow(dead_code)]
fn feature_key_tag(v: &FeatureKey) -> &'static str {
    match v {
        FeatureKey::Rg(_) => "Rg",
        FeatureKey::Gr(_) => "Gr",
        FeatureKey::Gc => "Gc",
        FeatureKey::Ar(_) => "Ar",
        FeatureKey::Ac => "Ac",
        FeatureKey::Br(_) => "Br",
        FeatureKey::Bc => "Bc",
        FeatureKey::BP => "BP",
        FeatureKey::BN => "BN",
    }
}

#[test]
fn feature_key_variant_tags_are_pinned() {
    assert_eq!(feature_key_tag(&FeatureKey::Gc), "Gc");
    assert_eq!(feature_key_tag(&FeatureKey::Ac), "Ac");
    assert_eq!(feature_key_tag(&FeatureKey::Bc), "Bc");
    assert_eq!(feature_key_tag(&FeatureKey::BP), "BP");
    assert_eq!(feature_key_tag(&FeatureKey::BN), "BN");
}

#[allow(dead_code)]
fn reference_range_key_tag(v: &ReferenceRangeKey) -> &'static str {
    match v {
        ReferenceRangeKey::SpeciesBiomarker(_) => "SpeciesBiomarker",
    }
}

#[allow(dead_code)]
fn param_key_tag(v: &ParamKey) -> &'static str {
    match v {
        ParamKey::GlobalStorageQuota => "GlobalStorageQuota",
        ParamKey::HealthScoreCacheTtl => "HealthScoreCacheTtl",
        ParamKey::AdminThreshold => "AdminThreshold",
    }
}

#[test]
fn param_key_variant_tags_are_pinned() {
    assert_eq!(param_key_tag(&ParamKey::GlobalStorageQuota), "GlobalStorageQuota");
    assert_eq!(param_key_tag(&ParamKey::HealthScoreCacheTtl), "HealthScoreCacheTtl");
    assert_eq!(param_key_tag(&ParamKey::AdminThreshold), "AdminThreshold");
}

#[allow(dead_code)]
fn dispute_key_tag(v: &DisputeKey) -> &'static str {
    match v {
        DisputeKey::Dispute(_) => "Dispute",
        DisputeKey::DisputeCount => "DisputeCount",
        DisputeKey::AppealWindow => "AppealWindow",
        DisputeKey::Arbitrator => "Arbitrator",
        DisputeKey::PetDisputesCount(_) => "PetDisputesCount",
        DisputeKey::PetDisputesIndex(_) => "PetDisputesIndex",
        DisputeKey::DisputeEvidence(_, _) => "DisputeEvidence",
        DisputeKey::DisputeEvidenceCount(_) => "DisputeEvidenceCount",
        DisputeKey::PartyEvidenceCount(_, _) => "PartyEvidenceCount",
        DisputeKey::DisputeVoteByVoter(_, _) => "DisputeVoteByVoter",
        DisputeKey::DisputeVoters(_) => "DisputeVoters",
    }
}

#[test]
fn dispute_key_variant_tags_are_pinned() {
    assert_eq!(dispute_key_tag(&DisputeKey::DisputeCount), "DisputeCount");
    assert_eq!(dispute_key_tag(&DisputeKey::AppealWindow), "AppealWindow");
    assert_eq!(dispute_key_tag(&DisputeKey::Arbitrator), "Arbitrator");
}

