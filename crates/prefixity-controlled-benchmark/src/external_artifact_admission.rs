//! Versioned, offline admission contract for external research artifacts.
//!
//! This module validates recorded evidence and derives a conservative
//! Prefixity research-use decision. It does not fetch sources, interpret
//! licenses as law, or authorize production planner behavior.

use serde::{Deserialize, Serialize};

pub const EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION: u32 = 1;
pub const EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID: &str = "prefixity.external-artifact-admission.v1";
pub const MAX_MANIFEST_BYTES: usize = 512 * 1024;

const MAX_ARTIFACT_ID: usize = 128;
const MAX_SOURCE_OWNER: usize = 256;
const MAX_REVISION: usize = 256;
const MAX_TEXT: usize = 1024;
const MAX_LOCATOR: usize = 2048;
const MAX_LIST: usize = 64;
const MAX_COUNT: u64 = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArtifactKind {
    BenchmarkDataset,
    TrajectoryDataset,
    ResultArchive,
    SourceRepository,
    EvaluatorFramework,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RevisionKind {
    GitCommit,
    DatasetRevision,
    ContentHash,
    Release,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceState {
    Explicit,
    DeclaredButUnverified,
    Absent,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReference {
    pub locator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRecord {
    pub state: EvidenceState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<EvidenceReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PublicAccessibility {
    Public,
    Ungated,
    Restricted,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParentProjectIdentity {
    pub identifier: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalArtifactAdmissionManifestV1 {
    pub schema_id: String,
    pub schema_version: u32,
    pub artifact_id: String,
    pub artifact_kind: ArtifactKind,
    pub canonical_source: String,
    pub source_owner: String,
    pub immutable_revision_kind: RevisionKind,
    pub immutable_revision: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_project: Option<ParentProjectIdentity>,
    pub public_accessibility: PublicAccessibility,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provenance_evidence: Vec<EvidenceReference>,
    pub framework_code_license: EvidenceRecord,
    pub dataset_artifact_reuse: EvidenceRecord,
    pub underlying_third_party_material: EvidenceRecord,
    pub permission: PermissionEvidence,
    pub third_party_material: ThirdPartyMaterialEvidence,
    pub stable_join: StableJoinEvidence,
    pub gold_independence: GoldIndependenceEvidence,
    pub content: ArtifactContentEvidence,
    pub retention: GitRetentionPolicy,
    pub execution: ExecutionRequirements,
    pub requested_use: RequestedUse,
}

pub type ExternalArtifactAdmissionManifest = ExternalArtifactAdmissionManifestV1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PermissionBasis {
    PermittedExplicit,
    PermittedByRecordedBasis,
    NotPermitted,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationEvidence {
    pub basis: PermissionBasis,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<EvidenceReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PermissionEvidence {
    pub metadata_inspection: OperationEvidence,
    pub local_raw_download: OperationEvidence,
    pub local_raw_read_parse: OperationEvidence,
    pub local_transformation: OperationEvidence,
    pub retain_hashes_identifiers_aggregate_metrics: OperationEvidence,
    pub retain_bounded_structural_metadata: OperationEvidence,
    pub retain_source_excerpts: OperationEvidence,
    pub redistribute_raw_artifact: OperationEvidence,
    pub vendor_raw_artifact: OperationEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaterialPresence {
    ExplicitPresent,
    ExplicitAbsent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterialEvidence {
    pub status: MaterialPresence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<EvidenceReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThirdPartyMaterialEvidence {
    pub source_code: MaterialEvidence,
    pub issue_or_pr_text: MaterialEvidence,
    pub patches: MaterialEvidence,
    pub tests_or_test_patches: MaterialEvidence,
    pub tool_output_source_excerpts: MaterialEvidence,
    pub private_user_data: MaterialEvidence,
    pub unknown_third_party_material: MaterialEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JoinClassification {
    ExactOneToOne,
    ExactOneToMany,
    ExactManyToOne,
    Ambiguous,
    None,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JoinKeyKind {
    StableIdentifier,
    TrajectoryTaskName,
    RepositoryIssue,
    ContentHash,
    None,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JoinAmbiguity {
    None,
    Possible,
    Confirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StableJoinEvidence {
    pub classification: JoinClassification,
    pub key_kind: JoinKeyKind,
    pub left_identifier_description: String,
    pub right_identifier_description: String,
    pub deterministic_exact_match: bool,
    pub ambiguity: JoinAmbiguity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_join_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_bounded_join_count: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GoldIndependence {
    BlindToGold,
    GoldConditioned,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldIndependenceEvidence {
    pub status: GoldIndependence,
    pub evidence_basis: EvidenceRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PresenceStatus {
    ExplicitPresent,
    ExplicitAbsent,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContentSufficiency {
    SufficientForFrontHalf,
    Limited,
    Insufficient,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactContentEvidence {
    pub sufficiency: ContentSufficiency,
    pub chronology_or_order: PresenceStatus,
    pub tool_calls: PresenceStatus,
    pub tool_results: PresenceStatus,
    pub file_reads_or_views: PresenceStatus,
    pub search_or_symbol_activity: PresenceStatus,
    pub observations: PresenceStatus,
    pub edits_or_actions: PresenceStatus,
    pub stable_task_identity: PresenceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GitRetention {
    Track,
    DoNotTrack,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitRetentionPolicy {
    pub raw_external_artifact: GitRetention,
    pub full_trajectories: GitRetention,
    pub source_file_bodies: GitRetention,
    pub source_excerpts: GitRetention,
    pub problem_statements: GitRetention,
    pub patches_or_test_patches: GitRetention,
    pub opaque_task_ids: GitRetention,
    pub hashes: GitRetention,
    pub source_urls_and_revisions: GitRetention,
    pub license_and_provenance_metadata: GitRetention,
    pub structural_metadata: GitRetention,
    pub aggregate_metrics: GitRetention,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionRequirement {
    NotRequired,
    Required,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRequirements {
    pub static_data_parsing: ExecutionRequirement,
    pub archive_decompression: ExecutionRequirement,
    pub third_party_code_execution: ExecutionRequirement,
    pub container_execution: ExecutionRequirement,
    pub network_access: ExecutionRequirement,
    pub provider_model_inference: ExecutionRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequestedUse {
    MetadataResearch,
    ExternalFrontHalfEvaluation,
    LimitedPilot,
    RawRedistribution,
    ReferenceOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionDecision {
    AdmissibleLocalStudy,
    AdmissibleLimitedPilot,
    AdmissibleMetadataOnly,
    AdmissibleRawRedistribution,
    ReferenceOnly,
    BlockedPermission,
    BlockedProvenance,
    BlockedJoin,
    BlockedGoldIndependence,
    BlockedContentSufficiency,
    BlockedExecutionRequirement,
    InvalidManifest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionReasonCode {
    AdmittedForLocalStudy,
    AdmittedForLimitedPilot,
    AdmittedForMetadataResearch,
    AdmittedForRawRedistribution,
    ContentInsufficient,
    DatasetReuseEvidenceNotExplicit,
    ExecutionRequirementNotCleared,
    GoldConditioned,
    GoldIndependenceUnknown,
    InvalidManifest,
    JoinAmbiguous,
    JoinUnavailable,
    MetadataPermissionNotEstablished,
    PermissionNotEstablished,
    PermissionNotExplicit,
    PrivateDataNotCleared,
    ProvenanceIncomplete,
    RawRetentionNotSafe,
    RedistributionNotExplicit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionReason {
    pub code: AdmissionReasonCode,
    pub field: String,
    pub message: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionWarningCode {
    ContentLimited,
    DatasetReuseEvidenceNotExplicit,
    DeclaredEvidenceNotVerified,
    NetworkAccessRequired,
    PublicAccessibilityIsNotPermission,
    ThirdPartyMaterialPresent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionWarning {
    pub code: AdmissionWarningCode,
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionDecisionReport {
    pub schema_id: String,
    pub schema_version: u32,
    pub artifact_id: String,
    pub artifact_kind: ArtifactKind,
    pub canonical_source: String,
    pub immutable_revision: String,
    pub requested_use: RequestedUse,
    pub decision: AdmissionDecision,
    pub reasons: Vec<AdmissionReason>,
    pub blocking_evidence_fields: Vec<String>,
    pub warnings: Vec<AdmissionWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdmissionValidationError {
    #[error("unsupported external admission schema version {found}; expected {expected}")]
    UnsupportedSchemaVersion { found: u32, expected: u32 },
    #[error("invalid external admission manifest at {field}: {message}")]
    InvalidField { field: String, message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum AdmissionError {
    #[error("external admission manifest exceeds {MAX_MANIFEST_BYTES} bytes")]
    OversizedManifest,
    #[error("invalid external admission manifest JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error(transparent)]
    Validation(#[from] AdmissionValidationError),
}

pub fn parse_manifest_json(
    bytes: &[u8],
) -> Result<ExternalArtifactAdmissionManifestV1, AdmissionError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(AdmissionError::OversizedManifest);
    }
    let manifest = serde_json::from_slice(bytes)?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn canonical_manifest_json(
    manifest: &ExternalArtifactAdmissionManifestV1,
) -> Result<Vec<u8>, AdmissionError> {
    validate_manifest(manifest)?;
    serde_json::to_vec(manifest).map_err(AdmissionError::InvalidJson)
}

pub fn validate_manifest(
    manifest: &ExternalArtifactAdmissionManifestV1,
) -> Result<(), AdmissionValidationError> {
    if manifest.schema_id != EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID {
        return Err(invalid(
            "schema_id",
            format!("must be {EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID}"),
        ));
    }
    if manifest.schema_version != EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION {
        return Err(AdmissionValidationError::UnsupportedSchemaVersion {
            found: manifest.schema_version,
            expected: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION,
        });
    }
    validate_bounded(&manifest.artifact_id, MAX_ARTIFACT_ID, "artifact_id")?;
    validate_bounded(&manifest.canonical_source, MAX_LOCATOR, "canonical_source")?;
    validate_bounded(&manifest.source_owner, MAX_SOURCE_OWNER, "source_owner")?;
    validate_bounded(
        &manifest.immutable_revision,
        MAX_REVISION,
        "immutable_revision",
    )?;
    if manifest.provenance_evidence.len() > MAX_LIST {
        return Err(invalid(
            "provenance_evidence",
            format!("contains more than {MAX_LIST} references"),
        ));
    }
    validate_references(&manifest.provenance_evidence, "provenance_evidence")?;
    if let Some(parent) = &manifest.parent_project {
        validate_bounded(
            &parent.identifier,
            MAX_ARTIFACT_ID,
            "parent_project.identifier",
        )?;
        validate_bounded(&parent.reference, MAX_LOCATOR, "parent_project.reference")?;
    }
    validate_record(&manifest.framework_code_license, "framework_code_license")?;
    validate_record(&manifest.dataset_artifact_reuse, "dataset_artifact_reuse")?;
    validate_record(
        &manifest.underlying_third_party_material,
        "underlying_third_party_material",
    )?;
    validate_permissions(&manifest.permission)?;
    validate_third_party(&manifest.third_party_material)?;
    validate_join(&manifest.stable_join)?;
    validate_gold(&manifest.gold_independence)?;
    validate_content(&manifest.content)?;
    validate_retention(&manifest.retention)?;
    validate_execution(&manifest.execution)?;
    Ok(())
}

pub fn derive_admission(manifest: &ExternalArtifactAdmissionManifestV1) -> AdmissionDecisionReport {
    if let Err(error) = validate_manifest(manifest) {
        return invalid_report(manifest, error);
    }

    let mut reasons = Vec::new();
    let mut warnings = Vec::new();
    add_common_warnings(manifest, &mut warnings);

    match manifest.requested_use {
        RequestedUse::MetadataResearch => evaluate_metadata(manifest, &mut reasons),
        RequestedUse::ExternalFrontHalfEvaluation => {
            evaluate_front_half(manifest, &mut reasons, &mut warnings, false)
        }
        RequestedUse::LimitedPilot => {
            evaluate_front_half(manifest, &mut reasons, &mut warnings, true)
        }
        RequestedUse::RawRedistribution => evaluate_redistribution(manifest, &mut reasons),
        RequestedUse::ReferenceOnly => reasons.push(AdmissionReason {
            code: AdmissionReasonCode::AdmittedForMetadataResearch,
            field: "requested_use".to_string(),
            message: "artifact is retained as a reference only".to_string(),
            blocking: false,
        }),
    }

    let decision = if manifest.requested_use == RequestedUse::ReferenceOnly {
        AdmissionDecision::ReferenceOnly
    } else if let Some(decision) = blocking_decision(&reasons) {
        decision
    } else {
        match manifest.requested_use {
            RequestedUse::MetadataResearch => AdmissionDecision::AdmissibleMetadataOnly,
            RequestedUse::ExternalFrontHalfEvaluation => AdmissionDecision::AdmissibleLocalStudy,
            RequestedUse::LimitedPilot => AdmissionDecision::AdmissibleLimitedPilot,
            RequestedUse::RawRedistribution => AdmissionDecision::AdmissibleRawRedistribution,
            RequestedUse::ReferenceOnly => AdmissionDecision::ReferenceOnly,
        }
    };

    if !reasons.iter().any(|reason| reason.blocking) {
        let (code, field, message) = match decision {
            AdmissionDecision::AdmissibleMetadataOnly => (
                AdmissionReasonCode::AdmittedForMetadataResearch,
                "requested_use",
                "recorded evidence supports metadata research only",
            ),
            AdmissionDecision::AdmissibleLocalStudy => (
                AdmissionReasonCode::AdmittedForLocalStudy,
                "requested_use",
                "recorded evidence supports bounded local front-half study",
            ),
            AdmissionDecision::AdmissibleLimitedPilot => (
                AdmissionReasonCode::AdmittedForLimitedPilot,
                "requested_use",
                "recorded evidence supports a bounded limited pilot",
            ),
            AdmissionDecision::AdmissibleRawRedistribution => (
                AdmissionReasonCode::AdmittedForRawRedistribution,
                "requested_use",
                "recorded evidence supports the explicitly requested raw redistribution operation",
            ),
            AdmissionDecision::ReferenceOnly => (
                AdmissionReasonCode::AdmittedForMetadataResearch,
                "requested_use",
                "artifact is retained as a reference only",
            ),
            _ => unreachable!("a blocked decision has a blocking reason"),
        };
        reasons.push(AdmissionReason {
            code,
            field: field.to_string(),
            message: message.to_string(),
            blocking: false,
        });
    }

    let mut report = sort_report_fields(&mut reasons, &mut warnings);
    report.artifact_id = manifest.artifact_id.clone();
    report.artifact_kind = manifest.artifact_kind;
    report.canonical_source = manifest.canonical_source.clone();
    report.immutable_revision = manifest.immutable_revision.clone();
    report.requested_use = manifest.requested_use;
    report.decision = decision;
    report
}

fn evaluate_metadata(
    manifest: &ExternalArtifactAdmissionManifestV1,
    reasons: &mut Vec<AdmissionReason>,
) {
    require_operation(
        reasons,
        &manifest.permission.metadata_inspection,
        "permission.metadata_inspection",
        false,
        true,
    );
    if sensitive_retention_is_unsafe(&manifest.retention, false) {
        reasons.push(blocking_reason(
            AdmissionReasonCode::RawRetentionNotSafe,
            "retention",
            "metadata research may not track raw artifact or trajectory content in git",
        ));
    }
}

fn evaluate_front_half(
    manifest: &ExternalArtifactAdmissionManifestV1,
    reasons: &mut Vec<AdmissionReason>,
    warnings: &mut Vec<AdmissionWarning>,
    limited_pilot: bool,
) {
    require_operation(
        reasons,
        &manifest.permission.local_raw_read_parse,
        "permission.local_raw_read_parse",
        false,
        false,
    );
    require_operation(
        reasons,
        &manifest.permission.local_transformation,
        "permission.local_transformation",
        false,
        false,
    );

    if manifest.provenance_evidence.is_empty()
        || matches!(
            manifest.underlying_third_party_material.state,
            EvidenceState::Unknown | EvidenceState::Absent
        )
    {
        reasons.push(blocking_reason(
            AdmissionReasonCode::ProvenanceIncomplete,
            "provenance_evidence",
            "bounded local research lacks a complete recorded provenance basis",
        ));
    }
    if !private_data_is_cleared(&manifest.third_party_material) {
        reasons.push(blocking_reason(
            AdmissionReasonCode::PrivateDataNotCleared,
            "third_party_material.private_user_data",
            "private or user data is not explicitly cleared for the requested study",
        ));
    }

    match manifest.stable_join.classification {
        JoinClassification::ExactOneToOne
        | JoinClassification::ExactOneToMany
        | JoinClassification::ExactManyToOne => {}
        JoinClassification::Ambiguous => reasons.push(blocking_reason(
            AdmissionReasonCode::JoinAmbiguous,
            "stable_join",
            "the recorded join is ambiguous and cannot support deterministic evaluation",
        )),
        JoinClassification::None | JoinClassification::Unknown => reasons.push(blocking_reason(
            AdmissionReasonCode::JoinUnavailable,
            "stable_join",
            "no deterministic exact join is recorded for the evaluation target",
        )),
    }

    match manifest.gold_independence.status {
        GoldIndependence::BlindToGold => {}
        GoldIndependence::GoldConditioned => reasons.push(blocking_reason(
            AdmissionReasonCode::GoldConditioned,
            "gold_independence.status",
            "gold-conditioned evidence cannot be admitted as primary external evidence",
        )),
        GoldIndependence::Unknown | GoldIndependence::NotApplicable => {
            reasons.push(blocking_reason(
                AdmissionReasonCode::GoldIndependenceUnknown,
                "gold_independence.status",
                "gold independence is not established for the primary external evaluation",
            ))
        }
    }

    if manifest.content.sufficiency == ContentSufficiency::Limited && limited_pilot {
        warnings.push(AdmissionWarning {
            code: AdmissionWarningCode::ContentLimited,
            field: "content.sufficiency".to_string(),
            message: "limited content supports only a bounded pilot, not a full front-half claim"
                .to_string(),
        });
    } else if manifest.content.sufficiency != ContentSufficiency::SufficientForFrontHalf {
        reasons.push(blocking_reason(
            AdmissionReasonCode::ContentInsufficient,
            "content.sufficiency",
            "the artifact does not record sufficient chronological/context evidence",
        ));
    }

    if sensitive_retention_is_unsafe(&manifest.retention, true) {
        reasons.push(blocking_reason(
            AdmissionReasonCode::RawRetentionNotSafe,
            "retention",
            "raw artifact and third-party content are not excluded from tracked Prefixity data",
        ));
    }
    require_execution_clearance(manifest, reasons, warnings);
}

fn evaluate_redistribution(
    manifest: &ExternalArtifactAdmissionManifestV1,
    reasons: &mut Vec<AdmissionReason>,
) {
    require_operation(
        reasons,
        &manifest.permission.redistribute_raw_artifact,
        "permission.redistribute_raw_artifact",
        true,
        false,
    );
    require_operation(
        reasons,
        &manifest.permission.vendor_raw_artifact,
        "permission.vendor_raw_artifact",
        true,
        false,
    );
    if manifest.dataset_artifact_reuse.state != EvidenceState::Explicit
        || manifest.underlying_third_party_material.state != EvidenceState::Explicit
    {
        reasons.push(blocking_reason(
            AdmissionReasonCode::ProvenanceIncomplete,
            "dataset_artifact_reuse",
            "raw redistribution requires explicit recorded artifact and third-party reuse evidence",
        ));
    }
    if !all_material_absent(&manifest.third_party_material) {
        reasons.push(blocking_reason(
            AdmissionReasonCode::ProvenanceIncomplete,
            "third_party_material",
            "raw redistribution is not admitted when third-party or unknown material is recorded",
        ));
    }
    if !redistribution_retention_is_safe(&manifest.retention) {
        reasons.push(blocking_reason(
            AdmissionReasonCode::RawRetentionNotSafe,
            "retention",
            "raw redistribution requires explicit tracking permission for the raw artifact",
        ));
    }
}

fn require_operation(
    reasons: &mut Vec<AdmissionReason>,
    operation: &OperationEvidence,
    field: &str,
    explicit_only: bool,
    metadata: bool,
) {
    match operation.basis {
        PermissionBasis::PermittedExplicit => {}
        PermissionBasis::PermittedByRecordedBasis if !explicit_only => {}
        PermissionBasis::PermittedByRecordedBasis => reasons.push(blocking_reason(
            AdmissionReasonCode::PermissionNotExplicit,
            field,
            "the requested operation has a recorded basis but not explicit permission",
        )),
        PermissionBasis::NotPermitted => reasons.push(blocking_reason(
            AdmissionReasonCode::PermissionNotEstablished,
            field,
            "the requested operation is recorded as not permitted",
        )),
        PermissionBasis::Unknown => reasons.push(blocking_reason(
            if metadata {
                AdmissionReasonCode::MetadataPermissionNotEstablished
            } else {
                AdmissionReasonCode::PermissionNotEstablished
            },
            field,
            "the requested operation has no recorded permission basis",
        )),
    }
}

fn require_execution_clearance(
    manifest: &ExternalArtifactAdmissionManifestV1,
    reasons: &mut Vec<AdmissionReason>,
    warnings: &mut Vec<AdmissionWarning>,
) {
    for (field, requirement) in [
        (
            "execution.provider_model_inference",
            manifest.execution.provider_model_inference,
        ),
        (
            "execution.third_party_code_execution",
            manifest.execution.third_party_code_execution,
        ),
        (
            "execution.container_execution",
            manifest.execution.container_execution,
        ),
    ] {
        if requirement != ExecutionRequirement::NotRequired {
            reasons.push(blocking_reason(
                AdmissionReasonCode::ExecutionRequirementNotCleared,
                field,
                "the requested study requires an execution boundary that is not cleared",
            ));
        }
    }
    if manifest.execution.network_access == ExecutionRequirement::Required {
        warnings.push(AdmissionWarning {
            code: AdmissionWarningCode::NetworkAccessRequired,
            field: "execution.network_access".to_string(),
            message:
                "admission applies only to supplied local evidence; the contract performs no network access"
                    .to_string(),
        });
    }
}

fn add_common_warnings(
    manifest: &ExternalArtifactAdmissionManifestV1,
    warnings: &mut Vec<AdmissionWarning>,
) {
    if matches!(
        manifest.public_accessibility,
        PublicAccessibility::Public | PublicAccessibility::Ungated
    ) {
        warnings.push(AdmissionWarning {
            code: AdmissionWarningCode::PublicAccessibilityIsNotPermission,
            field: "public_accessibility".to_string(),
            message:
                "public or ungated access is recorded as technical accessibility, not permission for another operation"
                    .to_string(),
        });
    }
    if manifest.dataset_artifact_reuse.state != EvidenceState::Explicit {
        warnings.push(AdmissionWarning {
            code: AdmissionWarningCode::DatasetReuseEvidenceNotExplicit,
            field: "dataset_artifact_reuse".to_string(),
            message: "dataset or artifact reuse evidence is not explicit".to_string(),
        });
    }
    if manifest.underlying_third_party_material.state == EvidenceState::Explicit {
        warnings.push(AdmissionWarning {
            code: AdmissionWarningCode::ThirdPartyMaterialPresent,
            field: "underlying_third_party_material".to_string(),
            message: "underlying third-party provenance remains separately governed".to_string(),
        });
    }
    for (field, state) in [
        (
            "framework_code_license",
            manifest.framework_code_license.state,
        ),
        (
            "dataset_artifact_reuse",
            manifest.dataset_artifact_reuse.state,
        ),
        (
            "underlying_third_party_material",
            manifest.underlying_third_party_material.state,
        ),
        (
            "gold_independence.evidence_basis",
            manifest.gold_independence.evidence_basis.state,
        ),
    ] {
        if state == EvidenceState::DeclaredButUnverified {
            warnings.push(AdmissionWarning {
                code: AdmissionWarningCode::DeclaredEvidenceNotVerified,
                field: field.to_string(),
                message: "the manifest records a declaration that was not independently verified"
                    .to_string(),
            });
        }
    }
}

fn blocking_decision(reasons: &[AdmissionReason]) -> Option<AdmissionDecision> {
    let has = |codes: &[AdmissionReasonCode]| {
        reasons
            .iter()
            .any(|reason| reason.blocking && codes.contains(&reason.code))
    };
    if has(&[
        AdmissionReasonCode::PermissionNotEstablished,
        AdmissionReasonCode::PermissionNotExplicit,
        AdmissionReasonCode::MetadataPermissionNotEstablished,
        AdmissionReasonCode::RedistributionNotExplicit,
    ]) {
        return Some(AdmissionDecision::BlockedPermission);
    }
    if has(&[
        AdmissionReasonCode::ProvenanceIncomplete,
        AdmissionReasonCode::PrivateDataNotCleared,
        AdmissionReasonCode::RawRetentionNotSafe,
    ]) {
        return Some(AdmissionDecision::BlockedProvenance);
    }
    if has(&[
        AdmissionReasonCode::JoinAmbiguous,
        AdmissionReasonCode::JoinUnavailable,
    ]) {
        return Some(AdmissionDecision::BlockedJoin);
    }
    if has(&[
        AdmissionReasonCode::GoldConditioned,
        AdmissionReasonCode::GoldIndependenceUnknown,
    ]) {
        return Some(AdmissionDecision::BlockedGoldIndependence);
    }
    if has(&[AdmissionReasonCode::ContentInsufficient]) {
        return Some(AdmissionDecision::BlockedContentSufficiency);
    }
    if has(&[AdmissionReasonCode::ExecutionRequirementNotCleared]) {
        return Some(AdmissionDecision::BlockedExecutionRequirement);
    }
    None
}

fn invalid_report(
    manifest: &ExternalArtifactAdmissionManifestV1,
    error: AdmissionValidationError,
) -> AdmissionDecisionReport {
    AdmissionDecisionReport {
        schema_id: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID.to_string(),
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION,
        artifact_id: manifest.artifact_id.clone(),
        artifact_kind: manifest.artifact_kind,
        canonical_source: manifest.canonical_source.clone(),
        immutable_revision: manifest.immutable_revision.clone(),
        requested_use: manifest.requested_use,
        decision: AdmissionDecision::InvalidManifest,
        reasons: vec![AdmissionReason {
            code: AdmissionReasonCode::InvalidManifest,
            field: "manifest".to_string(),
            message: error.to_string(),
            blocking: true,
        }],
        blocking_evidence_fields: vec!["manifest".to_string()],
        warnings: Vec::new(),
    }
}

fn sort_report_fields(
    reasons: &mut Vec<AdmissionReason>,
    warnings: &mut Vec<AdmissionWarning>,
) -> AdmissionDecisionReport {
    reasons.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.message.cmp(&right.message))
    });
    reasons.dedup();
    warnings.sort_by(|left, right| {
        left.code
            .cmp(&right.code)
            .then_with(|| left.field.cmp(&right.field))
            .then_with(|| left.message.cmp(&right.message))
    });
    warnings.dedup();
    let mut blocking_evidence_fields = reasons
        .iter()
        .filter(|reason| reason.blocking)
        .map(|reason| reason.field.clone())
        .collect::<Vec<_>>();
    blocking_evidence_fields.sort();
    blocking_evidence_fields.dedup();

    AdmissionDecisionReport {
        schema_id: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_ID.to_string(),
        schema_version: EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_VERSION,
        artifact_id: String::new(),
        artifact_kind: ArtifactKind::Other,
        canonical_source: String::new(),
        immutable_revision: String::new(),
        requested_use: RequestedUse::ReferenceOnly,
        decision: AdmissionDecision::InvalidManifest,
        reasons: reasons.clone(),
        blocking_evidence_fields,
        warnings: warnings.clone(),
    }
}

fn validate_record(record: &EvidenceRecord, field: &str) -> Result<(), AdmissionValidationError> {
    validate_references(&record.references, &format!("{field}.references"))?;
    if let Some(note) = &record.note {
        validate_bounded(note, MAX_TEXT, &format!("{field}.note"))?;
    }
    if matches!(
        record.state,
        EvidenceState::Explicit | EvidenceState::DeclaredButUnverified
    ) && record.references.is_empty()
    {
        return Err(invalid(
            field,
            "explicit or declared evidence requires a source reference",
        ));
    }
    Ok(())
}

fn validate_permissions(permission: &PermissionEvidence) -> Result<(), AdmissionValidationError> {
    for (field, operation) in [
        (
            "permission.metadata_inspection",
            &permission.metadata_inspection,
        ),
        (
            "permission.local_raw_download",
            &permission.local_raw_download,
        ),
        (
            "permission.local_raw_read_parse",
            &permission.local_raw_read_parse,
        ),
        (
            "permission.local_transformation",
            &permission.local_transformation,
        ),
        (
            "permission.retain_hashes_identifiers_aggregate_metrics",
            &permission.retain_hashes_identifiers_aggregate_metrics,
        ),
        (
            "permission.retain_bounded_structural_metadata",
            &permission.retain_bounded_structural_metadata,
        ),
        (
            "permission.retain_source_excerpts",
            &permission.retain_source_excerpts,
        ),
        (
            "permission.redistribute_raw_artifact",
            &permission.redistribute_raw_artifact,
        ),
        (
            "permission.vendor_raw_artifact",
            &permission.vendor_raw_artifact,
        ),
    ] {
        validate_references(&operation.references, &format!("{field}.references"))?;
        if let Some(note) = &operation.note {
            validate_bounded(note, MAX_TEXT, &format!("{field}.note"))?;
        }
        if matches!(
            operation.basis,
            PermissionBasis::PermittedExplicit | PermissionBasis::PermittedByRecordedBasis
        ) && operation.references.is_empty()
        {
            return Err(invalid(
                field,
                "permitted operation requires a source reference",
            ));
        }
    }
    Ok(())
}

fn validate_third_party(
    material: &ThirdPartyMaterialEvidence,
) -> Result<(), AdmissionValidationError> {
    for (field, evidence) in [
        ("third_party_material.source_code", &material.source_code),
        (
            "third_party_material.issue_or_pr_text",
            &material.issue_or_pr_text,
        ),
        ("third_party_material.patches", &material.patches),
        (
            "third_party_material.tests_or_test_patches",
            &material.tests_or_test_patches,
        ),
        (
            "third_party_material.tool_output_source_excerpts",
            &material.tool_output_source_excerpts,
        ),
        (
            "third_party_material.private_user_data",
            &material.private_user_data,
        ),
        (
            "third_party_material.unknown_third_party_material",
            &material.unknown_third_party_material,
        ),
    ] {
        validate_references(&evidence.references, &format!("{field}.references"))?;
        if evidence.status != MaterialPresence::Unknown && evidence.references.is_empty() {
            return Err(invalid(
                field,
                "explicit material status requires a source reference",
            ));
        }
    }
    Ok(())
}

fn validate_join(join: &StableJoinEvidence) -> Result<(), AdmissionValidationError> {
    validate_bounded(
        &join.left_identifier_description,
        MAX_TEXT,
        "stable_join.left_identifier_description",
    )?;
    validate_bounded(
        &join.right_identifier_description,
        MAX_TEXT,
        "stable_join.right_identifier_description",
    )?;
    if let Some(count) = join.expected_join_count {
        validate_count(count, "stable_join.expected_join_count")?;
    }
    if let Some(count) = join.observed_bounded_join_count {
        validate_count(count, "stable_join.observed_bounded_join_count")?;
    }
    let exact = matches!(
        join.classification,
        JoinClassification::ExactOneToOne
            | JoinClassification::ExactOneToMany
            | JoinClassification::ExactManyToOne
    );
    if join.deterministic_exact_match != exact {
        return Err(invalid(
            "stable_join.deterministic_exact_match",
            "does not agree with join classification",
        ));
    }
    if exact && (join.key_kind == JoinKeyKind::None || join.ambiguity != JoinAmbiguity::None) {
        return Err(invalid(
            "stable_join",
            "an exact join requires a key kind and no ambiguity",
        ));
    }
    if join.classification == JoinClassification::Ambiguous && join.ambiguity == JoinAmbiguity::None
    {
        return Err(invalid(
            "stable_join.ambiguity",
            "ambiguous classification requires an ambiguity marker",
        ));
    }
    if !exact
        && join.key_kind != JoinKeyKind::None
        && join.classification == JoinClassification::None
    {
        return Err(invalid(
            "stable_join.key_kind",
            "a missing join must use key_kind=NONE",
        ));
    }
    Ok(())
}

fn validate_gold(gold: &GoldIndependenceEvidence) -> Result<(), AdmissionValidationError> {
    validate_record(&gold.evidence_basis, "gold_independence.evidence_basis")?;
    if gold.status == GoldIndependence::BlindToGold
        && matches!(
            gold.evidence_basis.state,
            EvidenceState::Unknown | EvidenceState::Absent | EvidenceState::NotApplicable
        )
    {
        return Err(invalid(
            "gold_independence",
            "BLIND_TO_GOLD requires a separately recorded evidence basis",
        ));
    }
    Ok(())
}

fn validate_content(content: &ArtifactContentEvidence) -> Result<(), AdmissionValidationError> {
    if content.sufficiency == ContentSufficiency::SufficientForFrontHalf
        && (content.chronology_or_order != PresenceStatus::ExplicitPresent
            || content.stable_task_identity != PresenceStatus::ExplicitPresent
            || !matches!(content.observations, PresenceStatus::ExplicitPresent)
                && !matches!(content.edits_or_actions, PresenceStatus::ExplicitPresent)
                && !matches!(content.tool_results, PresenceStatus::ExplicitPresent))
    {
        return Err(invalid(
            "content.sufficiency",
            "sufficient front-half content requires chronology, stable task identity, and observed action/result material",
        ));
    }
    Ok(())
}

fn validate_retention(retention: &GitRetentionPolicy) -> Result<(), AdmissionValidationError> {
    let _ = retention;
    Ok(())
}

fn validate_execution(execution: &ExecutionRequirements) -> Result<(), AdmissionValidationError> {
    let _ = execution;
    Ok(())
}

fn validate_references(
    references: &[EvidenceReference],
    field: &str,
) -> Result<(), AdmissionValidationError> {
    if references.len() > MAX_LIST {
        return Err(invalid(
            field,
            format!("contains more than {MAX_LIST} references"),
        ));
    }
    for (index, reference) in references.iter().enumerate() {
        validate_bounded(
            &reference.locator,
            MAX_LOCATOR,
            &format!("{field}[{index}].locator"),
        )?;
        if let Some(note) = &reference.note {
            validate_bounded(note, MAX_TEXT, &format!("{field}[{index}].note"))?;
        }
    }
    Ok(())
}

fn validate_bounded(
    value: &str,
    maximum: usize,
    field: &str,
) -> Result<(), AdmissionValidationError> {
    if value.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    if value.chars().count() > maximum {
        return Err(invalid(field, format!("exceeds {maximum} characters")));
    }
    if value.chars().any(char::is_control) {
        return Err(invalid(field, "contains a control character"));
    }
    Ok(())
}

fn validate_count(count: u64, field: &str) -> Result<(), AdmissionValidationError> {
    if count > MAX_COUNT {
        return Err(invalid(field, format!("exceeds bound {MAX_COUNT}")));
    }
    Ok(())
}

fn invalid(field: &str, message: impl Into<String>) -> AdmissionValidationError {
    AdmissionValidationError::InvalidField {
        field: field.to_string(),
        message: message.into(),
    }
}

fn blocking_reason(code: AdmissionReasonCode, field: &str, message: &str) -> AdmissionReason {
    AdmissionReason {
        code,
        field: field.to_string(),
        message: message.to_string(),
        blocking: true,
    }
}

fn private_data_is_cleared(material: &ThirdPartyMaterialEvidence) -> bool {
    material.private_user_data.status == MaterialPresence::ExplicitAbsent
}

fn all_material_absent(material: &ThirdPartyMaterialEvidence) -> bool {
    [
        &material.source_code,
        &material.issue_or_pr_text,
        &material.patches,
        &material.tests_or_test_patches,
        &material.tool_output_source_excerpts,
        &material.private_user_data,
        &material.unknown_third_party_material,
    ]
    .iter()
    .all(|evidence| evidence.status == MaterialPresence::ExplicitAbsent)
}

fn sensitive_retention_is_unsafe(retention: &GitRetentionPolicy, require_all: bool) -> bool {
    let sensitive = [
        retention.raw_external_artifact,
        retention.full_trajectories,
        retention.source_file_bodies,
        retention.source_excerpts,
        retention.problem_statements,
        retention.patches_or_test_patches,
    ];
    sensitive.iter().any(|state| {
        *state == GitRetention::Track || (require_all && *state == GitRetention::Unknown)
    })
}

fn redistribution_retention_is_safe(retention: &GitRetentionPolicy) -> bool {
    [
        retention.raw_external_artifact,
        retention.full_trajectories,
        retention.source_file_bodies,
        retention.source_excerpts,
        retention.problem_statements,
        retention.patches_or_test_patches,
    ]
    .iter()
    .all(|state| *state == GitRetention::Track)
}
