use prefixity_controlled_benchmark::{
    canonical_manifest_json, derive_admission, parse_manifest_json, validate_manifest,
    AdmissionDecision, AdmissionReasonCode, AdmissionValidationError, ExecutionRequirement,
    ExternalArtifactAdmissionManifestV1, GoldIndependence, JoinAmbiguity, JoinClassification,
    JoinKeyKind, PermissionBasis, RequestedUse, MAX_MANIFEST_BYTES,
};

fn base_manifest() -> ExternalArtifactAdmissionManifestV1 {
    parse_manifest_json(include_bytes!(
        "fixtures/external-artifact-admission/explicit-local-study-pass.json"
    ))
    .expect("synthetic fixture must be valid")
}

fn decision(manifest: &ExternalArtifactAdmissionManifestV1) -> AdmissionDecision {
    derive_admission(manifest).decision
}

#[test]
fn explicit_local_study_fixture_is_admitted() {
    let manifest = base_manifest();
    let report = derive_admission(&manifest);
    assert_eq!(report.artifact_id, "fictional-trajectory-pass-v1");
    assert_eq!(
        report.requested_use,
        RequestedUse::ExternalFrontHalfEvaluation
    );
    assert_eq!(report.decision, AdmissionDecision::AdmissibleLocalStudy);
    assert!(report.blocking_evidence_fields.is_empty());
}

#[test]
fn public_accessibility_does_not_imply_raw_permission_and_metadata_can_differ() {
    let mut manifest = base_manifest();
    manifest.public_accessibility = prefixity_controlled_benchmark::PublicAccessibility::Ungated;
    manifest.permission.local_raw_read_parse.basis = PermissionBasis::Unknown;
    assert_eq!(decision(&manifest), AdmissionDecision::BlockedPermission);
    assert!(derive_admission(&manifest)
        .warnings
        .iter()
        .any(|warning| warning.code
            == prefixity_controlled_benchmark::AdmissionWarningCode::PublicAccessibilityIsNotPermission));

    manifest.requested_use = RequestedUse::MetadataResearch;
    assert_eq!(
        decision(&manifest),
        AdmissionDecision::AdmissibleMetadataOnly
    );
}

#[test]
fn code_license_does_not_imply_dataset_permission() {
    let mut manifest = base_manifest();
    manifest.dataset_artifact_reuse.state = prefixity_controlled_benchmark::EvidenceState::Absent;
    manifest.permission.local_raw_read_parse.basis = PermissionBasis::Unknown;
    assert_eq!(decision(&manifest), AdmissionDecision::BlockedPermission);
    assert!(derive_admission(&manifest).warnings.iter().any(|warning| {
        warning.code
            == prefixity_controlled_benchmark::AdmissionWarningCode::DatasetReuseEvidenceNotExplicit
    }));
}

#[test]
fn gold_conditioning_and_unknown_gold_block_front_half() {
    let mut conditioned = base_manifest();
    conditioned.gold_independence.status = GoldIndependence::GoldConditioned;
    assert_eq!(
        decision(&conditioned),
        AdmissionDecision::BlockedGoldIndependence
    );

    let mut unknown = base_manifest();
    unknown.gold_independence.status = GoldIndependence::Unknown;
    assert_eq!(
        decision(&unknown),
        AdmissionDecision::BlockedGoldIndependence
    );
}

#[test]
fn unstable_join_blocks_front_half() {
    let mut manifest = base_manifest();
    manifest.stable_join.classification = JoinClassification::None;
    manifest.stable_join.key_kind = JoinKeyKind::None;
    manifest.stable_join.deterministic_exact_match = false;
    manifest.stable_join.ambiguity = JoinAmbiguity::None;
    assert_eq!(decision(&manifest), AdmissionDecision::BlockedJoin);
}

#[test]
fn final_patch_only_content_blocks_front_half() {
    let mut manifest = base_manifest();
    manifest.content.sufficiency = prefixity_controlled_benchmark::ContentSufficiency::Limited;
    manifest.content.chronology_or_order =
        prefixity_controlled_benchmark::PresenceStatus::ExplicitAbsent;
    assert_eq!(
        decision(&manifest),
        AdmissionDecision::BlockedContentSufficiency
    );
}

#[test]
fn limited_pilot_is_distinct_from_full_front_half() {
    let mut manifest = base_manifest();
    manifest.requested_use = RequestedUse::LimitedPilot;
    manifest.content.sufficiency = prefixity_controlled_benchmark::ContentSufficiency::Limited;
    assert_eq!(
        decision(&manifest),
        AdmissionDecision::AdmissibleLimitedPilot
    );
}

#[test]
fn redistribution_requires_explicit_permission_and_retention() {
    let mut manifest = base_manifest();
    manifest.requested_use = RequestedUse::RawRedistribution;
    manifest.permission.redistribute_raw_artifact.basis = PermissionBasis::PermittedByRecordedBasis;
    manifest.permission.vendor_raw_artifact.basis = PermissionBasis::PermittedByRecordedBasis;
    manifest.permission.redistribute_raw_artifact.references =
        manifest.permission.local_raw_read_parse.references.clone();
    manifest.permission.vendor_raw_artifact.references =
        manifest.permission.local_raw_read_parse.references.clone();
    manifest.retention.raw_external_artifact = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.full_trajectories = prefixity_controlled_benchmark::GitRetention::Track;
    let report = derive_admission(&manifest);
    assert_eq!(report.decision, AdmissionDecision::BlockedPermission);
}

#[test]
fn explicit_raw_redistribution_is_a_distinct_admission_level() {
    let mut manifest = base_manifest();
    manifest.requested_use = RequestedUse::RawRedistribution;
    let references = manifest.permission.local_raw_read_parse.references.clone();
    manifest.permission.redistribute_raw_artifact.basis = PermissionBasis::PermittedExplicit;
    manifest.permission.redistribute_raw_artifact.references = references.clone();
    manifest.permission.vendor_raw_artifact.basis = PermissionBasis::PermittedExplicit;
    manifest.permission.vendor_raw_artifact.references = references;
    manifest.retention.raw_external_artifact = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.full_trajectories = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.source_file_bodies = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.source_excerpts = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.problem_statements = prefixity_controlled_benchmark::GitRetention::Track;
    manifest.retention.patches_or_test_patches =
        prefixity_controlled_benchmark::GitRetention::Track;
    assert_eq!(
        decision(&manifest),
        AdmissionDecision::AdmissibleRawRedistribution
    );
}

#[test]
fn dangerous_execution_requirement_blocks_front_half() {
    let mut manifest = base_manifest();
    manifest.execution.provider_model_inference = ExecutionRequirement::Required;
    assert_eq!(
        decision(&manifest),
        AdmissionDecision::BlockedExecutionRequirement
    );
}

#[test]
fn reasons_are_deterministic_and_stably_ordered() {
    let mut manifest = base_manifest();
    manifest.permission.local_raw_read_parse.basis = PermissionBasis::Unknown;
    manifest.stable_join.classification = JoinClassification::Unknown;
    manifest.stable_join.key_kind = JoinKeyKind::None;
    manifest.stable_join.deterministic_exact_match = false;
    manifest.gold_independence.status = GoldIndependence::Unknown;
    let first = derive_admission(&manifest);
    let second = derive_admission(&manifest);
    assert_eq!(first, second);
    assert_eq!(first.decision, AdmissionDecision::BlockedPermission);
    assert!(first
        .reasons
        .windows(2)
        .all(|window| window[0].code <= window[1].code));
    assert!(first
        .reasons
        .iter()
        .any(|reason| reason.code == AdmissionReasonCode::PermissionNotEstablished));
}

#[test]
fn json_fixture_round_trips_and_report_identity_is_preserved() {
    let manifest = base_manifest();
    let encoded = canonical_manifest_json(&manifest).unwrap();
    let decoded = parse_manifest_json(&encoded).unwrap();
    assert_eq!(decoded, manifest);
    assert_eq!(canonical_manifest_json(&decoded).unwrap(), encoded);
    let report = derive_admission(&decoded);
    assert_eq!(report.schema_id, "prefixity.external-artifact-admission.v1");
    assert_eq!(report.immutable_revision, "fictional-revision-001");
}

#[test]
fn unsupported_versions_and_unknown_fields_are_rejected_without_panicking() {
    let mut unsupported = base_manifest();
    unsupported.schema_version = 2;
    assert!(matches!(
        validate_manifest(&unsupported),
        Err(AdmissionValidationError::UnsupportedSchemaVersion { .. })
    ));
    assert_eq!(decision(&unsupported), AdmissionDecision::InvalidManifest);

    let mut value = serde_json::to_value(base_manifest()).unwrap();
    value["unexpected"] = serde_json::json!("rejected");
    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(parse_manifest_json(&bytes).is_err());
    assert!(parse_manifest_json(b"{").is_err());
}

#[test]
fn oversized_manifest_is_rejected_before_json_parsing() {
    let bytes = vec![b'x'; MAX_MANIFEST_BYTES + 1];
    assert!(matches!(
        parse_manifest_json(&bytes),
        Err(prefixity_controlled_benchmark::AdmissionError::OversizedManifest)
    ));
}
