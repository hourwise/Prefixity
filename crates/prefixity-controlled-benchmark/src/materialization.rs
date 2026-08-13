//! P0-L13 inert candidate materialization and transformation-fidelity proof.
//!
//! This module reconstructs a neutral request from an approved P0-L11 layout
//! candidate, checks it against the P0-L12 identity/evidence boundary, and
//! returns it only with a deterministic internal safety certificate. It never
//! executes, projects, or applies the request.

use crate::candidate_evaluation::{CandidateEvaluation, ClaimPermission};
use crate::conformance::ConformanceRequest;
use crate::context_stability::{
    analyze_context_stability, ContextSegmentAnalysis, ContextStabilityInputs,
};
use crate::diff::{CacheImpactAssessment, ChangeCategory, DiffState, RequestDiff};
use crate::error::{BenchmarkError, MaterializationErrorCode};
use crate::hashing::canonical_hash;
use crate::layout_planner::{
    layout_metrics, reordered_request, CandidateSafetyStatus, ContextLayoutPlan, LayoutCandidate,
    LayoutTransformationKind,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MATERIALIZATION_SCHEMA_ID: &str = "prefixity.candidate-materialization";
pub const MATERIALIZATION_SCHEMA_VERSION: u32 = 1;
pub const SAFETY_CERTIFICATE_SCHEMA_ID: &str = "prefixity.materialization-safety-certificate";
pub const SAFETY_CERTIFICATE_SCHEMA_VERSION: u32 = 1;
pub const EXPERIMENT_PAIR_SCHEMA_ID: &str = "prefixity.candidate-experiment-pair";
pub const EXPERIMENT_PAIR_SCHEMA_VERSION: u32 = 1;
pub const MAX_MATERIALIZATION_PROVENANCE: usize = 16;
pub const MAX_EXPERIMENT_CASE_ID_BYTES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificationStatus {
    CertifiedForExperimentMaterialization,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterializationInvariant {
    SourceIdentity,
    CandidateIdentity,
    AuthorizedTransformation,
    ContentConservation,
    ArtifactConservation,
    ToolConservation,
    EnvelopeConservation,
    TrustConservation,
    ProvenanceConservation,
    OrderChangeOnly,
    RequestDiffAgreement,
    StructuralReanalysis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantResult {
    pub invariant: MaterializationInvariant,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequestDiffReference {
    pub source_request_fingerprint: String,
    pub materialized_request_fingerprint: String,
    pub planned_diff_fingerprint: String,
    pub actual_diff_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializationSafetyCertificate {
    pub schema_id: String,
    pub schema_version: u32,
    pub source_identity_verified: bool,
    pub candidate_identity_verified: bool,
    pub authorized_transformation_verified: bool,
    pub content_conservation: bool,
    pub artifact_conservation: bool,
    pub tool_conservation: bool,
    pub envelope_conservation: bool,
    pub trust_conservation: bool,
    pub provenance_conservation: bool,
    pub order_change_only: bool,
    pub request_diff_reference: RequestDiffReference,
    pub invariant_results: Vec<InvariantResult>,
    pub certification_status: CertificationStatus,
    pub provenance: BTreeMap<String, String>,
}

impl MaterializationSafetyCertificate {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != SAFETY_CERTIFICATE_SCHEMA_ID
            || self.schema_version != SAFETY_CERTIFICATE_SCHEMA_VERSION
        {
            return Err(BenchmarkError::validation(
                "unsupported materialization safety certificate schema",
            ));
        }
        if self.provenance.len() > MAX_MATERIALIZATION_PROVENANCE {
            return Err(BenchmarkError::validation(
                "materialization certificate provenance exceeds its bound",
            ));
        }
        for (key, value) in &self.provenance {
            validate_text(key, "certificate provenance key")?;
            validate_text(value, "certificate provenance value")?;
        }
        for (field, value) in [
            (
                "certificate source request fingerprint",
                &self.request_diff_reference.source_request_fingerprint,
            ),
            (
                "certificate materialized request fingerprint",
                &self.request_diff_reference.materialized_request_fingerprint,
            ),
            (
                "certificate planned diff fingerprint",
                &self.request_diff_reference.planned_diff_fingerprint,
            ),
            (
                "certificate actual diff fingerprint",
                &self.request_diff_reference.actual_diff_fingerprint,
            ),
        ] {
            validate_hash(value, field)?;
        }
        let expected = [
            (
                MaterializationInvariant::SourceIdentity,
                self.source_identity_verified,
            ),
            (
                MaterializationInvariant::CandidateIdentity,
                self.candidate_identity_verified,
            ),
            (
                MaterializationInvariant::AuthorizedTransformation,
                self.authorized_transformation_verified,
            ),
            (
                MaterializationInvariant::ContentConservation,
                self.content_conservation,
            ),
            (
                MaterializationInvariant::ArtifactConservation,
                self.artifact_conservation,
            ),
            (
                MaterializationInvariant::ToolConservation,
                self.tool_conservation,
            ),
            (
                MaterializationInvariant::EnvelopeConservation,
                self.envelope_conservation,
            ),
            (
                MaterializationInvariant::TrustConservation,
                self.trust_conservation,
            ),
            (
                MaterializationInvariant::ProvenanceConservation,
                self.provenance_conservation,
            ),
            (
                MaterializationInvariant::OrderChangeOnly,
                self.order_change_only,
            ),
            (MaterializationInvariant::RequestDiffAgreement, true),
            (MaterializationInvariant::StructuralReanalysis, true),
        ];
        if self.invariant_results.len() != expected.len()
            || self
                .invariant_results
                .iter()
                .zip(expected)
                .any(|(actual, (kind, passed))| actual.invariant != kind || actual.passed != passed)
        {
            return Err(BenchmarkError::validation(
                "materialization certificate invariants are incomplete or non-deterministic",
            ));
        }
        if self.certification_status != CertificationStatus::CertifiedForExperimentMaterialization
            || expected.iter().any(|(_, passed)| !passed)
            || self.invariant_results.iter().any(|result| !result.passed)
        {
            return Err(BenchmarkError::validation(
                "materialization certificate is not a successful certification",
            ));
        }
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        crate::hashing::canonical_json(self)
            .map_err(|error| BenchmarkError::validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaterializedCandidate {
    pub schema_id: String,
    pub schema_version: u32,
    pub source_request_fingerprint: String,
    pub candidate_fingerprint: String,
    pub evaluation_fingerprint: String,
    pub materialized_request: ConformanceRequest,
    pub materialized_request_fingerprint: String,
    pub safety_certificate: MaterializationSafetyCertificate,
    pub provenance: BTreeMap<String, String>,
}

impl MaterializedCandidate {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != MATERIALIZATION_SCHEMA_ID
            || self.schema_version != MATERIALIZATION_SCHEMA_VERSION
        {
            return Err(BenchmarkError::validation(
                "unsupported materialized candidate schema",
            ));
        }
        validate_hash(
            &self.source_request_fingerprint,
            "source request fingerprint",
        )?;
        validate_hash(&self.candidate_fingerprint, "candidate fingerprint")?;
        validate_hash(&self.evaluation_fingerprint, "evaluation fingerprint")?;
        validate_hash(
            &self.materialized_request_fingerprint,
            "materialized request fingerprint",
        )?;
        self.materialized_request.validate()?;
        if self.materialized_request.request_fingerprint()? != self.materialized_request_fingerprint
        {
            return Err(BenchmarkError::validation(
                "materialized request fingerprint is not traceable",
            ));
        }
        if self
            .safety_certificate
            .request_diff_reference
            .source_request_fingerprint
            != self.source_request_fingerprint
            || self
                .safety_certificate
                .request_diff_reference
                .materialized_request_fingerprint
                != self.materialized_request_fingerprint
        {
            return Err(BenchmarkError::validation(
                "materialization certificate request identity is not traceable",
            ));
        }
        self.safety_certificate.validate()?;
        validate_provenance(&self.provenance)?;
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        crate::hashing::canonical_json(self)
            .map_err(|error| BenchmarkError::validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateExperimentPair {
    pub schema_id: String,
    pub schema_version: u32,
    pub source_request_fingerprint: String,
    pub candidate_request_fingerprint: String,
    pub candidate_fingerprint: String,
    pub safety_certificate_fingerprint: String,
    pub source_case_id: String,
    pub candidate_case_id: String,
    pub pair_fingerprint: String,
    pub provenance: BTreeMap<String, String>,
}

impl CandidateExperimentPair {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != EXPERIMENT_PAIR_SCHEMA_ID
            || self.schema_version != EXPERIMENT_PAIR_SCHEMA_VERSION
        {
            return Err(BenchmarkError::validation(
                "unsupported candidate experiment pair schema",
            ));
        }
        for (field, value) in [
            (
                "source request fingerprint",
                &self.source_request_fingerprint,
            ),
            (
                "candidate request fingerprint",
                &self.candidate_request_fingerprint,
            ),
            ("candidate fingerprint", &self.candidate_fingerprint),
            (
                "safety certificate fingerprint",
                &self.safety_certificate_fingerprint,
            ),
            ("pair fingerprint", &self.pair_fingerprint),
        ] {
            validate_hash(value, field)?;
        }
        validate_case_id(&self.source_case_id)?;
        validate_case_id(&self.candidate_case_id)?;
        if self.source_case_id == self.candidate_case_id {
            return Err(BenchmarkError::validation(
                "control and candidate case IDs must differ",
            ));
        }
        validate_provenance(&self.provenance)?;
        if self.pair_fingerprint != pair_fingerprint(self) {
            return Err(BenchmarkError::validation(
                "candidate experiment pair fingerprint is not deterministic",
            ));
        }
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        crate::hashing::canonical_json(self)
            .map_err(|error| BenchmarkError::validation(error.to_string()))
    }
}

pub fn materialize_candidate(
    source_request: &ConformanceRequest,
    plan: &ContextLayoutPlan,
    candidate: &LayoutCandidate,
    evaluation: &CandidateEvaluation,
    stability_inputs: &ContextStabilityInputs,
    provenance: BTreeMap<String, String>,
) -> Result<MaterializedCandidate, BenchmarkError> {
    source_request.validate()?;
    stability_inputs.validate()?;
    validate_provenance(&provenance)?;

    let source_fingerprint = source_request.request_fingerprint()?;
    if source_fingerprint != plan.source_request_fingerprint
        || source_fingerprint != candidate.request_diff.left_request_fingerprint
    {
        return Err(materialization(
            MaterializationErrorCode::StaleSourceRequest,
            "source request fingerprint no longer matches the P0-L11 plan",
        ));
    }

    validate_authorized_transformation(
        candidate,
        &plan.source_stability_analysis,
        stability_inputs,
    )?;
    if candidate.safety != CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints {
        return Err(materialization(
            MaterializationErrorCode::CandidateSafetyRejected,
            "P0-L11 did not establish ordering safety",
        ));
    }
    plan.validate().map_err(|error| {
        materialization(
            MaterializationErrorCode::CandidateIdentityMismatch,
            format!("P0-L11 plan is not valid: {error}"),
        )
    })?;
    let planned_candidate = plan
        .candidates
        .iter()
        .find(|planned| planned.candidate_id == candidate.candidate_id)
        .ok_or_else(|| {
            materialization(
                MaterializationErrorCode::CandidateIdentityMismatch,
                "candidate is not present in the supplied P0-L11 plan",
            )
        })?;
    if planned_candidate != candidate {
        return Err(materialization(
            MaterializationErrorCode::CandidateIdentityMismatch,
            "candidate does not exactly match the plan record",
        ));
    }
    if !is_hash(&candidate.layout_fingerprint) {
        return Err(materialization(
            MaterializationErrorCode::CandidateIdentityMismatch,
            "candidate layout fingerprint is invalid",
        ));
    }
    validate_evaluation_identity(evaluation, candidate, &source_fingerprint)?;

    let source_analysis = analyze_context_stability(source_request, stability_inputs)?;
    if source_analysis != plan.source_stability_analysis {
        return Err(materialization(
            MaterializationErrorCode::StructuralReanalysisMismatch,
            "P0-L10 source re-analysis does not match the planned source analysis",
        ));
    }
    let order = candidate
        .ordered_segments
        .iter()
        .map(|reference| reference.source_position)
        .collect::<Vec<_>>();
    let expected_layout_fingerprint =
        crate::layout_planner::layout_fingerprint_for_order(&source_analysis, &order)?;
    if expected_layout_fingerprint != candidate.layout_fingerprint {
        return Err(materialization(
            MaterializationErrorCode::CandidateIdentityMismatch,
            "candidate fingerprint does not match its ordered segment layout",
        ));
    }
    let materialized_request = reordered_request(source_request, &source_analysis, &order)
        .map_err(|error| {
            materialization(
                MaterializationErrorCode::ArtifactMissing,
                format!("candidate reconstruction failed: {error}"),
            )
        })?;
    let materialized_request_fingerprint = materialized_request.request_fingerprint()?;
    let actual_diff = crate::diff::request_diff(source_request, &materialized_request)?;
    validate_reorder_diff(&actual_diff)?;
    let planned_diff_fingerprint = canonical_hash(&candidate.request_diff)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let actual_diff_fingerprint = canonical_hash(&actual_diff)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    if candidate.request_diff != actual_diff {
        return Err(materialization(
            MaterializationErrorCode::PlannedActualDiffMismatch,
            "the P0-L11 planned RequestDiff differs from the reconstructed RequestDiff",
        ));
    }
    let materialized_analysis = analyze_context_stability(&materialized_request, stability_inputs)?;
    if materialized_analysis != candidate.resulting_analysis
        || layout_metrics(&materialized_analysis) != candidate.structural_effect.candidate
    {
        return Err(materialization(
            MaterializationErrorCode::StructuralReanalysisMismatch,
            "P0-L10 re-analysis does not match the candidate structural result",
        ));
    }

    let content_conservation = content_conserved(source_request, &materialized_request);
    let artifact_conservation =
        artifact_multiset(source_request) == artifact_multiset(&materialized_request);
    let tool_conservation = source_request.context.tools == materialized_request.context.tools;
    let envelope_conservation = source_request.envelope == materialized_request.envelope;
    let order_change_only = only_authorized_order_changed(source_request, &materialized_request);
    let trust_conservation = segment_metadata_conserved(
        &source_analysis,
        &materialized_analysis,
        &candidate.ordered_segments,
    );
    let provenance_conservation = candidate.ordered_segments.iter().all(|reference| {
        reference.metadata_fingerprint.is_some()
            || metadata_unavailable(&source_analysis, reference)
    });
    if !content_conservation || !artifact_conservation {
        return Err(materialization(
            MaterializationErrorCode::ArtifactContentMismatch,
            "artifact content or occurrence multiset was not conserved",
        ));
    }
    if !tool_conservation {
        return Err(materialization(
            MaterializationErrorCode::UnexpectedToolChange,
            "tool surface changed during materialization",
        ));
    }
    if !envelope_conservation {
        return Err(materialization(
            MaterializationErrorCode::UnexpectedEnvelopeChange,
            "request envelope changed during materialization",
        ));
    }
    if !trust_conservation || !provenance_conservation {
        return Err(materialization(
            MaterializationErrorCode::TrustProvenanceMismatch,
            "trust or provenance metadata was not conserved",
        ));
    }
    if !order_change_only {
        return Err(materialization(
            MaterializationErrorCode::UnexpectedContentChange,
            "materialization changed more than the authorized artifact order",
        ));
    }

    let certificate = MaterializationSafetyCertificate {
        schema_id: SAFETY_CERTIFICATE_SCHEMA_ID.to_string(),
        schema_version: SAFETY_CERTIFICATE_SCHEMA_VERSION,
        source_identity_verified: true,
        candidate_identity_verified: true,
        authorized_transformation_verified: true,
        content_conservation,
        artifact_conservation,
        tool_conservation,
        envelope_conservation,
        trust_conservation,
        provenance_conservation,
        order_change_only,
        request_diff_reference: RequestDiffReference {
            source_request_fingerprint: source_fingerprint.clone(),
            materialized_request_fingerprint: materialized_request_fingerprint.clone(),
            planned_diff_fingerprint,
            actual_diff_fingerprint,
        },
        invariant_results: invariant_results(),
        certification_status: CertificationStatus::CertifiedForExperimentMaterialization,
        provenance: BTreeMap::from([
            ("source".to_string(), "p0-l4-neutral-request".to_string()),
            ("planner".to_string(), "p0-l11-layout-candidate".to_string()),
            (
                "evaluator".to_string(),
                "p0-l12-candidate-evaluation".to_string(),
            ),
            ("runtime_execution".to_string(), "not_performed".to_string()),
            ("performance_claim".to_string(), "not_allowed".to_string()),
        ]),
    };
    certificate.validate()?;
    let evaluation_fingerprint = canonical_hash(evaluation)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let materialized = MaterializedCandidate {
        schema_id: MATERIALIZATION_SCHEMA_ID.to_string(),
        schema_version: MATERIALIZATION_SCHEMA_VERSION,
        source_request_fingerprint: source_fingerprint,
        candidate_fingerprint: candidate.layout_fingerprint.clone(),
        evaluation_fingerprint,
        materialized_request,
        materialized_request_fingerprint,
        safety_certificate: certificate,
        provenance,
    };
    materialized.validate()?;
    Ok(materialized)
}

pub fn build_candidate_experiment_pair(
    materialized: &MaterializedCandidate,
    source_case_id: impl Into<String>,
    candidate_case_id: impl Into<String>,
    provenance: BTreeMap<String, String>,
) -> Result<CandidateExperimentPair, BenchmarkError> {
    materialized.validate()?;
    validate_provenance(&provenance)?;
    let source_case_id = source_case_id.into();
    let candidate_case_id = candidate_case_id.into();
    validate_case_id(&source_case_id)?;
    validate_case_id(&candidate_case_id)?;
    if source_case_id == candidate_case_id {
        return Err(BenchmarkError::validation(
            "control and candidate case IDs must differ",
        ));
    }
    let safety_certificate_fingerprint = canonical_hash(&materialized.safety_certificate)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let mut pair = CandidateExperimentPair {
        schema_id: EXPERIMENT_PAIR_SCHEMA_ID.to_string(),
        schema_version: EXPERIMENT_PAIR_SCHEMA_VERSION,
        source_request_fingerprint: materialized.source_request_fingerprint.clone(),
        candidate_request_fingerprint: materialized.materialized_request_fingerprint.clone(),
        candidate_fingerprint: materialized.candidate_fingerprint.clone(),
        safety_certificate_fingerprint,
        source_case_id,
        candidate_case_id,
        pair_fingerprint: String::new(),
        provenance,
    };
    pair.pair_fingerprint = pair_fingerprint(&pair);
    pair.validate()?;
    Ok(pair)
}

fn validate_evaluation_identity(
    evaluation: &CandidateEvaluation,
    candidate: &LayoutCandidate,
    source_fingerprint: &str,
) -> Result<(), BenchmarkError> {
    evaluation.validate().map_err(|error| {
        materialization(
            MaterializationErrorCode::EvaluationMismatch,
            format!("P0-L12 evaluation is invalid: {error}"),
        )
    })?;
    let planned_diff_fingerprint = canonical_hash(&candidate.request_diff)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let reference = &evaluation.candidate;
    if reference.candidate_id != candidate.candidate_id
        || reference.layout_fingerprint != candidate.layout_fingerprint
        || reference.source_request_fingerprint != source_fingerprint
        || reference.candidate_request_fingerprint
            != candidate.request_diff.right_request_fingerprint
        || reference.request_diff_fingerprint != planned_diff_fingerprint
        || evaluation.structural.safety
            != CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints
        || evaluation.claim_permissions.performance_claims != ClaimPermission::NotAllowed
        || evaluation.claim_permissions.application_claims != ClaimPermission::NotAllowed
    {
        return Err(materialization(
            MaterializationErrorCode::EvaluationMismatch,
            "P0-L12 evaluation does not identify the supplied safe candidate",
        ));
    }
    if evaluation
        .blockers
        .contains(&crate::candidate_evaluation::EvidenceBlocker::CandidateSafetyNotEstablished)
    {
        return Err(materialization(
            MaterializationErrorCode::CandidateSafetyRejected,
            "P0-L12 recorded a structural safety blocker",
        ));
    }
    Ok(())
}

fn validate_authorized_transformation(
    candidate: &LayoutCandidate,
    source_analysis: &crate::context_stability::ContextStabilityAnalysis,
    inputs: &ContextStabilityInputs,
) -> Result<(), BenchmarkError> {
    if candidate.transformations.len() != 1 {
        return Err(materialization(
            MaterializationErrorCode::UnsupportedTransformation,
            "P0-L13 supports exactly one P0-L11 artifact reorder transformation",
        ));
    }
    let transformation = &candidate.transformations[0];
    if !matches!(
        transformation.kind,
        LayoutTransformationKind::AdjacentSwap | LayoutTransformationKind::RegionLocalMove
    ) {
        return Err(materialization(
            MaterializationErrorCode::UnsupportedTransformation,
            "candidate transformation is outside the P0-L13 reorder vocabulary",
        ));
    }
    if candidate.ordered_segments.len() != source_analysis.segments.len() {
        return Err(materialization(
            MaterializationErrorCode::ArtifactMissing,
            "candidate does not reference every source segment",
        ));
    }
    let mut positions = BTreeSet::new();
    for reference in &candidate.ordered_segments {
        if !positions.insert(reference.source_position) {
            return Err(materialization(
                MaterializationErrorCode::ArtifactDuplicated,
                "candidate repeats a source segment occurrence",
            ));
        }
        let source = source_analysis
            .segments
            .get(reference.source_position)
            .ok_or_else(|| {
                materialization(
                    MaterializationErrorCode::ArtifactMissing,
                    "candidate references a missing source segment",
                )
            })?;
        if reference.structural_path != source.structural_path
            || reference.component_id != source.component_id
            || reference.role != source.role
        {
            return Err(materialization(
                MaterializationErrorCode::CandidateIdentityMismatch,
                "candidate segment identity does not match P0-L11 source analysis",
            ));
        }
        if reference.content_fingerprint != source.content_fingerprint {
            return Err(materialization(
                MaterializationErrorCode::ArtifactContentMismatch,
                "candidate segment content fingerprint changed",
            ));
        }
        let expected_metadata =
            crate::layout_planner::metadata_fingerprint_for_segment(source, inputs)?;
        if reference.metadata_fingerprint != expected_metadata {
            return Err(materialization(
                MaterializationErrorCode::TrustProvenanceMismatch,
                "candidate segment metadata identity changed",
            ));
        }
    }
    if positions.len() != source_analysis.segments.len() {
        return Err(materialization(
            MaterializationErrorCode::ArtifactMissing,
            "candidate omits a source segment occurrence",
        ));
    }
    let changed_positions = candidate
        .ordered_segments
        .iter()
        .enumerate()
        .filter(|(position, reference)| *position != reference.source_position)
        .count();
    let mut expected_moved_segments = Vec::new();
    let mut expected_from_positions = Vec::new();
    let mut expected_to_positions = Vec::new();
    let new_positions = candidate
        .ordered_segments
        .iter()
        .enumerate()
        .map(|(position, reference)| (reference.source_position, position))
        .collect::<BTreeMap<usize, usize>>();
    for source_position in 0..source_analysis.segments.len() {
        let to_position = new_positions[&source_position];
        if source_position != to_position {
            let source_segment = &source_analysis.segments[source_position];
            if source_segment.role != crate::context_stability::ContextRole::ContextArtifact {
                return Err(materialization(
                    MaterializationErrorCode::UnsupportedTransformation,
                    "candidate transformation moves a fixed non-artifact segment",
                ));
            }
            expected_moved_segments.push(source_segment.structural_path.clone());
            expected_from_positions.push(source_position);
            expected_to_positions.push(to_position);
        }
    }
    if changed_positions == 0
        || transformation.moved_segments.len() != transformation.from_positions.len()
        || transformation.from_positions.len() != transformation.to_positions.len()
        || transformation.moved_segments != expected_moved_segments
        || transformation.from_positions != expected_from_positions
        || transformation.to_positions != expected_to_positions
    {
        return Err(materialization(
            MaterializationErrorCode::UnsupportedTransformation,
            "candidate does not describe a concrete artifact reorder",
        ));
    }
    Ok(())
}

fn validate_reorder_diff(diff: &RequestDiff) -> Result<(), BenchmarkError> {
    if diff.envelope_diff.cache_impact != CacheImpactAssessment::Unknown
        || diff.prefix_diff.cache_impact != CacheImpactAssessment::Unknown
        || diff.cache_impact != CacheImpactAssessment::Unknown
    {
        return Err(materialization(
            MaterializationErrorCode::PlannedActualDiffMismatch,
            "RequestDiff cache impact was not preserved as unknown",
        ));
    }
    if diff.interpretation.context != DiffState::Changed
        || diff.interpretation.envelope != DiffState::Identical
        || !diff.envelope_diff.changes.is_empty()
        || diff.prefix_diff.changes.len() != 1
        || diff.prefix_diff.changes[0].category != ChangeCategory::ArtifactOrderChanged
        || !diff.prefix_diff.changes[0].order_changed
        || diff.prefix_diff.changes[0].content_changed
        || diff.prefix_diff.changes[0].presence_changed
    {
        return Err(materialization(
            MaterializationErrorCode::UnexpectedContentChange,
            "actual RequestDiff contains a non-order transformation",
        ));
    }
    Ok(())
}

fn content_conserved(left: &ConformanceRequest, right: &ConformanceRequest) -> bool {
    left.context.system_instruction == right.context.system_instruction
        && left.context.user_content == right.context.user_content
        && artifact_content_multiset(left) == artifact_content_multiset(right)
}

fn artifact_multiset(request: &ConformanceRequest) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for artifact in &request.context.artifacts {
        *counts.entry(artifact.artifact_id.clone()).or_insert(0) += 1;
    }
    counts
}

fn artifact_content_multiset(request: &ConformanceRequest) -> BTreeMap<(String, String), usize> {
    let mut counts = BTreeMap::new();
    for artifact in &request.context.artifacts {
        *counts
            .entry((
                artifact.artifact_id.clone(),
                crate::hashing::hash_text(&artifact.content),
            ))
            .or_insert(0) += 1;
    }
    counts
}

fn only_authorized_order_changed(
    source: &ConformanceRequest,
    materialized: &ConformanceRequest,
) -> bool {
    source.context.system_instruction == materialized.context.system_instruction
        && source.context.user_content == materialized.context.user_content
        && source.context.tools == materialized.context.tools
        && source.envelope == materialized.envelope
        && source.context.artifacts != materialized.context.artifacts
        && artifact_multiset(source) == artifact_multiset(materialized)
}

fn segment_metadata_conserved(
    source: &crate::context_stability::ContextStabilityAnalysis,
    materialized: &crate::context_stability::ContextStabilityAnalysis,
    ordered_segments: &[crate::layout_planner::LayoutSegmentReference],
) -> bool {
    ordered_segments
        .iter()
        .enumerate()
        .all(|(position, reference)| {
            let Some(source_segment) = source.segments.get(reference.source_position) else {
                return false;
            };
            let Some(materialized_segment) = materialized.segments.get(position) else {
                return false;
            };
            equal_segment_metadata(source_segment, materialized_segment)
        })
}

fn equal_segment_metadata(left: &ContextSegmentAnalysis, right: &ContextSegmentAnalysis) -> bool {
    left.structural_path == right.structural_path
        && left.component_id == right.component_id
        && left.role == right.role
        && left.stability == right.stability
        && left.lifecycle == right.lifecycle
        && left.classification_source == right.classification_source
        && left.trust == right.trust
        && left.artifact_id == right.artifact_id
        && left.content_fingerprint == right.content_fingerprint
        && left.sizes == right.sizes
        && left.size_source == right.size_source
        && left.token_size == right.token_size
}

fn metadata_unavailable(
    analysis: &crate::context_stability::ContextStabilityAnalysis,
    reference: &crate::layout_planner::LayoutSegmentReference,
) -> bool {
    analysis
        .segments
        .get(reference.source_position)
        .is_some_and(|segment| {
            matches!(
                segment.classification_source,
                crate::context_stability::ClassificationSource::StructuralRole
                    | crate::context_stability::ClassificationSource::Unknown
            )
        })
}

fn invariant_results() -> Vec<InvariantResult> {
    [
        MaterializationInvariant::SourceIdentity,
        MaterializationInvariant::CandidateIdentity,
        MaterializationInvariant::AuthorizedTransformation,
        MaterializationInvariant::ContentConservation,
        MaterializationInvariant::ArtifactConservation,
        MaterializationInvariant::ToolConservation,
        MaterializationInvariant::EnvelopeConservation,
        MaterializationInvariant::TrustConservation,
        MaterializationInvariant::ProvenanceConservation,
        MaterializationInvariant::OrderChangeOnly,
        MaterializationInvariant::RequestDiffAgreement,
        MaterializationInvariant::StructuralReanalysis,
    ]
    .into_iter()
    .map(|invariant| InvariantResult {
        invariant,
        passed: true,
    })
    .collect()
}

fn pair_fingerprint(pair: &CandidateExperimentPair) -> String {
    let identity = (
        &pair.schema_id,
        pair.schema_version,
        &pair.source_request_fingerprint,
        &pair.candidate_request_fingerprint,
        &pair.candidate_fingerprint,
        &pair.safety_certificate_fingerprint,
        &pair.source_case_id,
        &pair.candidate_case_id,
        &pair.provenance,
    );
    canonical_hash(&identity).expect("bounded pair identity serializes")
}

fn materialization(code: MaterializationErrorCode, message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::materialization(code, message)
}

fn validate_provenance(provenance: &BTreeMap<String, String>) -> Result<(), BenchmarkError> {
    if provenance.len() > MAX_MATERIALIZATION_PROVENANCE {
        return Err(BenchmarkError::validation(
            "materialization provenance exceeds its bound",
        ));
    }
    for (key, value) in provenance {
        validate_text(key, "materialization provenance key")?;
        validate_text(value, "materialization provenance value")?;
    }
    Ok(())
}

fn validate_case_id(value: &str) -> Result<(), BenchmarkError> {
    validate_text(value, "experiment case ID")?;
    if value.len() > MAX_EXPERIMENT_CASE_ID_BYTES {
        return Err(BenchmarkError::validation(
            "experiment case ID exceeds its bound",
        ));
    }
    Ok(())
}

fn validate_hash(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if !is_hash(value) {
        return Err(BenchmarkError::validation(format!(
            "{field} must be a SHA-256 fingerprint"
        )));
    }
    Ok(())
}

fn is_hash(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_text(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_EXPERIMENT_CASE_ID_BYTES {
        return Err(BenchmarkError::validation(format!(
            "{field} must be non-empty and bounded"
        )));
    }
    Ok(())
}
