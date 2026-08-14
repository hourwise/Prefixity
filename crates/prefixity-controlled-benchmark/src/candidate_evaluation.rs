//! Conservative candidate evaluation and evidence gating.
//!
//! P0-L12 consumes the existing P0-L8, P0-L9, P0-L10, and P0-L11 records. It
//! reports what the current evidence justifies; it does not execute or apply
//! a candidate and it never turns structural cleanliness into a performance
//! claim.

use crate::capability_registry::{CapabilityKey, CapabilityProfile, CapabilityState};
use crate::diff::EnvelopeDiff;
use crate::error::BenchmarkError;
use crate::hashing::canonical_hash;
use crate::layout_planner::{CandidateSafetyStatus, LayoutCandidate};
use crate::observation_diagnostics::{
    CacheDiagnostic, CacheRegressionAssessment, CausalityStatus, ComparabilityLevel,
    EvidenceAssociation, EvidenceSourceClass, RequestObservationAlignment,
};
use prefixity_core::observation::{Observed, RuntimeCacheCapabilities};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const CANDIDATE_EVALUATION_SCHEMA_ID: &str = "prefixity.candidate-evaluation";
pub const CANDIDATE_EVALUATION_SCHEMA_VERSION: u32 = 1;
pub const CANDIDATE_EVALUATOR_VERSION: &str = "p0-l12-v1";
pub const MAX_EVALUATION_OBSERVATIONS: usize = 16;
pub const MAX_EVALUATION_BLOCKERS: usize = 16;
pub const MAX_EVALUATION_PROVENANCE: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    StructuralOnly,
    CapabilityCompatible,
    ReadyForExperiment,
    ObservationallySupported,
    MixedEvidence,
    UnsupportedByCurrentEvidence,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimPermission {
    Allowed,
    AllowedIfEvidenced,
    AllowedIfRealObservation,
    NotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateHypothesis {
    ReducedStabilityInversionMayIncreaseReusableLeadingContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimPermissions {
    pub structural_claims: ClaimPermission,
    pub capability_claims: ClaimPermission,
    pub observation_claims: ClaimPermission,
    pub performance_claims: ClaimPermission,
    pub causal_claims: ClaimPermission,
    pub application_claims: ClaimPermission,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateReference {
    pub candidate_id: String,
    pub layout_fingerprint: String,
    pub source_request_fingerprint: String,
    pub candidate_request_fingerprint: String,
    pub request_diff_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeProfileReference {
    pub profile_id: String,
    pub identity_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralAssessment {
    pub safety: CandidateSafetyStatus,
    pub source_inversion_count: usize,
    pub candidate_inversion_count: usize,
    pub source_leading_segments: usize,
    pub candidate_leading_segments: usize,
    pub source_unknown_boundary_count: usize,
    pub candidate_unknown_boundary_count: usize,
    pub moved_segment_count: usize,
    pub changed_relative_relationships: usize,
    pub cache_impact: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityAssessment {
    NotProvided,
    SupportedDocumented,
    SupportedObserved,
    UnsupportedDocumented,
    UnsupportedObserved,
    UnknownDocumented,
    UnknownUnverified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityGateAssessment {
    pub capability: CapabilityKey,
    pub state: CapabilityAssessment,
    pub profile: Option<RuntimeProfileReference>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationRelevance {
    Relevant,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationRelevanceReason {
    RuntimeIdentityMismatch,
    ModelIdentityMismatch,
    ProtocolIdentityMismatch,
    RuntimeVersionIdentityMismatch,
    ProfileIdentityMismatch,
    RequestFingerprintMismatch,
    CandidateMutationMismatch,
    EnvelopeCompatibilityMismatch,
    IncomparableObservation,
    InsufficientObservationFields,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationEvidence {
    pub left_observation_id: String,
    pub right_observation_id: String,
    pub diagnostic_fingerprint: String,
    pub source: EvidenceSourceClass,
    pub relevance: ObservationRelevance,
    pub reasons: Vec<ObservationRelevanceReason>,
    pub assessment: CacheRegressionAssessment,
    pub association: EvidenceAssociation,
    pub causality: CausalityStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentReadiness {
    Available,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentState {
    pub readiness: EnvironmentReadiness,
    pub blockers: Vec<EvidenceBlocker>,
}

impl EnvironmentState {
    pub fn available() -> Self {
        Self {
            readiness: EnvironmentReadiness::Available,
            blockers: Vec::new(),
        }
    }

    pub fn blocked(blockers: Vec<EvidenceBlocker>) -> Self {
        Self {
            readiness: EnvironmentReadiness::Blocked,
            blockers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DesignReadiness {
    Ready,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionReadiness {
    Ready,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentReadiness {
    pub design: DesignReadiness,
    pub environment: EnvironmentReadiness,
    pub execution: ExecutionReadiness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    ControlledExperiment,
    GatherCapabilityEvidence,
    GatherRuntimeObservation,
    ResolveEnvironment,
    RejectCandidate,
    NoAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBlocker {
    CandidateSafetyNotEstablished,
    RuntimeCapabilityPending,
    RuntimeCapabilityUnsupported,
    RuntimeIdentityMismatch,
    ObservationIdentityMismatch,
    SyntheticOnlyEvidence,
    DocumentedOnlyEvidence,
    NoExperimentalObservation,
    MixedObservations,
    EnvironmentUnavailable,
    InsufficientObservationFields,
    NoObservedCacheBenefit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationProvenance {
    pub evaluator_version: String,
    pub source: String,
    pub environment: EnvironmentReadiness,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateEvaluation {
    pub schema_id: String,
    pub schema_version: u32,
    pub candidate: CandidateReference,
    pub hypothesis: CandidateHypothesis,
    pub runtime_profile: Option<RuntimeProfileReference>,
    pub structural: StructuralAssessment,
    pub capability: CapabilityGateAssessment,
    pub observations: Vec<ObservationEvidence>,
    pub evidence_state: EvidenceState,
    pub claim_permissions: ClaimPermissions,
    pub experiment_readiness: ExperimentReadiness,
    pub next_action: NextAction,
    pub blockers: Vec<EvidenceBlocker>,
    pub provenance: EvaluationProvenance,
}

impl CandidateEvaluation {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CANDIDATE_EVALUATION_SCHEMA_ID
            || self.schema_version != CANDIDATE_EVALUATION_SCHEMA_VERSION
        {
            return Err(validation("unsupported candidate evaluation schema"));
        }
        if self.observations.len() > MAX_EVALUATION_OBSERVATIONS
            || self.blockers.len() > MAX_EVALUATION_BLOCKERS
        {
            return Err(validation(
                "candidate evaluation exceeds a bounded record limit",
            ));
        }
        if self.blockers.windows(2).any(|pair| pair[0] > pair[1]) {
            return Err(validation(
                "candidate evaluation blockers are not deterministic",
            ));
        }
        if self
            .observations
            .windows(2)
            .any(|pair| observation_sort_key(&pair[0]) > observation_sort_key(&pair[1]))
        {
            return Err(validation(
                "candidate evaluation observations are not deterministic",
            ));
        }
        if self.structural.cache_impact != "unknown"
            || self.claim_permissions.performance_claims != ClaimPermission::NotAllowed
            || self.claim_permissions.causal_claims != ClaimPermission::NotAllowed
            || self.claim_permissions.application_claims != ClaimPermission::NotAllowed
        {
            return Err(validation(
                "candidate evaluation widened a forbidden claim scope",
            ));
        }
        if self
            .observations
            .iter()
            .any(|observation| observation.causality != CausalityStatus::NotEstablished)
        {
            return Err(validation("candidate evaluation established causality"));
        }
        Ok(())
    }

    pub fn canonical_json(&self) -> Result<Vec<u8>, BenchmarkError> {
        self.validate()?;
        serde_json::to_vec(self).map_err(|error| validation(error.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CandidateEvaluationInput<'a> {
    pub candidate: &'a LayoutCandidate,
    pub capability_profile: Option<&'a CapabilityProfile>,
    pub observations: &'a [CacheDiagnostic],
    pub environment: &'a EnvironmentState,
}

pub fn evaluate_candidate(
    input: CandidateEvaluationInput<'_>,
) -> Result<CandidateEvaluation, BenchmarkError> {
    input.candidate.resulting_analysis.validate()?;
    input.environment.validate()?;
    if input.observations.len() > MAX_EVALUATION_OBSERVATIONS {
        return Err(validation(
            "candidate evaluation observations exceed their bound",
        ));
    }
    if let Some(profile) = input.capability_profile {
        profile.validate()?;
    }

    let candidate = candidate_reference(input.candidate)?;
    let runtime_profile = input
        .capability_profile
        .map(runtime_profile_reference)
        .transpose()?;
    let structural = structural_assessment(input.candidate);
    let capability = capability_assessment(input.capability_profile, runtime_profile.clone());

    let mut observations = input
        .observations
        .iter()
        .map(|diagnostic| {
            classify_observation(input.candidate, input.capability_profile, diagnostic)
        })
        .collect::<Vec<_>>();
    observations.sort_by(|left, right| {
        (
            &left.left_observation_id,
            &left.right_observation_id,
            source_rank(left.source),
        )
            .cmp(&(
                &right.left_observation_id,
                &right.right_observation_id,
                source_rank(right.source),
            ))
    });

    let mut blockers = BTreeSet::new();
    if input.candidate.safety != CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints {
        blockers.insert(EvidenceBlocker::CandidateSafetyNotEstablished);
    }
    match capability.state {
        CapabilityAssessment::NotProvided
        | CapabilityAssessment::UnknownDocumented
        | CapabilityAssessment::UnknownUnverified => {
            blockers.insert(EvidenceBlocker::RuntimeCapabilityPending);
        }
        CapabilityAssessment::UnsupportedDocumented | CapabilityAssessment::UnsupportedObserved => {
            blockers.insert(EvidenceBlocker::RuntimeCapabilityUnsupported);
        }
        CapabilityAssessment::SupportedDocumented | CapabilityAssessment::SupportedObserved => {}
    }

    let relevant = observations
        .iter()
        .filter(|observation| observation.relevance == ObservationRelevance::Relevant)
        .collect::<Vec<_>>();
    for observation in &observations {
        if observation.relevance == ObservationRelevance::Rejected {
            blockers.insert(EvidenceBlocker::ObservationIdentityMismatch);
            if observation
                .reasons
                .contains(&ObservationRelevanceReason::InsufficientObservationFields)
            {
                blockers.insert(EvidenceBlocker::InsufficientObservationFields);
            }
        }
    }
    if !relevant.is_empty()
        && relevant.iter().all(|observation| {
            observation.source != EvidenceSourceClass::ExperimentallyObservedRuntime
        })
    {
        if relevant
            .iter()
            .any(|observation| observation.source == EvidenceSourceClass::SyntheticProtocolTest)
        {
            blockers.insert(EvidenceBlocker::SyntheticOnlyEvidence);
        }
        if relevant
            .iter()
            .any(|observation| observation.source == EvidenceSourceClass::DocumentedCapability)
        {
            blockers.insert(EvidenceBlocker::DocumentedOnlyEvidence);
        }
    }

    let experimental = relevant
        .iter()
        .filter(|observation| {
            observation.source == EvidenceSourceClass::ExperimentallyObservedRuntime
        })
        .collect::<Vec<_>>();
    if experimental.is_empty() {
        blockers.insert(EvidenceBlocker::NoExperimentalObservation);
    }
    let has_mixed = experimental
        .iter()
        .any(|observation| observation.assessment == CacheRegressionAssessment::MixedObservations);
    let has_positive = experimental.iter().any(|observation| {
        observation.assessment == CacheRegressionAssessment::ObservedReuseIncrease
    });
    let has_contrary = experimental.iter().any(|observation| {
        matches!(
            observation.assessment,
            CacheRegressionAssessment::ObservedReuseDecrease
                | CacheRegressionAssessment::NoObservedCacheReuseChange
        )
    });
    if has_mixed || (has_positive && has_contrary) {
        blockers.insert(EvidenceBlocker::MixedObservations);
    } else if has_contrary {
        blockers.insert(EvidenceBlocker::NoObservedCacheBenefit);
    }

    if input.environment.readiness == EnvironmentReadiness::Blocked {
        blockers.insert(EvidenceBlocker::EnvironmentUnavailable);
    }
    blockers.extend(input.environment.blockers.iter().copied());
    if blockers.len() > MAX_EVALUATION_BLOCKERS {
        return Err(validation(
            "candidate evaluation blockers exceed their bound",
        ));
    }
    let blockers = blockers.into_iter().collect::<Vec<_>>();

    let capability_known = matches!(
        capability.state,
        CapabilityAssessment::SupportedDocumented | CapabilityAssessment::SupportedObserved
    );
    let safe =
        input.candidate.safety == CandidateSafetyStatus::OrderingSafeUnderDeclaredConstraints;
    let design = if safe && capability_known {
        DesignReadiness::Ready
    } else {
        DesignReadiness::Blocked
    };
    let execution = if design == DesignReadiness::Ready
        && input.environment.readiness == EnvironmentReadiness::Available
    {
        ExecutionReadiness::Ready
    } else {
        ExecutionReadiness::Blocked
    };
    let evidence_state = if !safe {
        EvidenceState::Blocked
    } else if input.capability_profile.is_none() {
        EvidenceState::StructuralOnly
    } else if matches!(
        capability.state,
        CapabilityAssessment::NotProvided
            | CapabilityAssessment::UnknownDocumented
            | CapabilityAssessment::UnknownUnverified
            | CapabilityAssessment::UnsupportedDocumented
            | CapabilityAssessment::UnsupportedObserved
    ) {
        EvidenceState::Blocked
    } else if has_mixed || (has_positive && has_contrary) {
        EvidenceState::MixedEvidence
    } else if has_contrary {
        EvidenceState::UnsupportedByCurrentEvidence
    } else if has_positive {
        EvidenceState::ObservationallySupported
    } else if input.capability_profile.is_some() {
        EvidenceState::ReadyForExperiment
    } else {
        EvidenceState::StructuralOnly
    };

    let next_action = match evidence_state {
        EvidenceState::Blocked if !safe => NextAction::RejectCandidate,
        EvidenceState::Blocked => match capability.state {
            CapabilityAssessment::UnsupportedDocumented
            | CapabilityAssessment::UnsupportedObserved => NextAction::RejectCandidate,
            _ => NextAction::GatherCapabilityEvidence,
        },
        EvidenceState::UnsupportedByCurrentEvidence => NextAction::NoAction,
        EvidenceState::MixedEvidence => NextAction::GatherRuntimeObservation,
        EvidenceState::ObservationallySupported => NextAction::NoAction,
        EvidenceState::ReadyForExperiment => {
            if input.environment.readiness == EnvironmentReadiness::Blocked {
                NextAction::ResolveEnvironment
            } else {
                NextAction::ControlledExperiment
            }
        }
        EvidenceState::StructuralOnly | EvidenceState::CapabilityCompatible => {
            NextAction::GatherCapabilityEvidence
        }
    };

    let provenance = EvaluationProvenance {
        evaluator_version: CANDIDATE_EVALUATOR_VERSION.to_string(),
        source: "p0-l10-p0-l11-candidate-plus-p0-l8-p0-l9-evidence".to_string(),
        environment: input.environment.readiness,
    };

    let evaluation = CandidateEvaluation {
        schema_id: CANDIDATE_EVALUATION_SCHEMA_ID.to_string(),
        schema_version: CANDIDATE_EVALUATION_SCHEMA_VERSION,
        candidate,
        hypothesis: CandidateHypothesis::ReducedStabilityInversionMayIncreaseReusableLeadingContext,
        runtime_profile,
        structural,
        capability,
        observations,
        evidence_state,
        claim_permissions: ClaimPermissions {
            structural_claims: if safe {
                ClaimPermission::Allowed
            } else {
                ClaimPermission::NotAllowed
            },
            capability_claims: if input.capability_profile.is_some() {
                ClaimPermission::AllowedIfEvidenced
            } else {
                ClaimPermission::NotAllowed
            },
            observation_claims: ClaimPermission::AllowedIfRealObservation,
            performance_claims: ClaimPermission::NotAllowed,
            causal_claims: ClaimPermission::NotAllowed,
            application_claims: ClaimPermission::NotAllowed,
        },
        experiment_readiness: ExperimentReadiness {
            design,
            environment: input.environment.readiness,
            execution,
        },
        next_action,
        blockers,
        provenance,
    };
    evaluation.validate()?;
    Ok(evaluation)
}

fn candidate_reference(candidate: &LayoutCandidate) -> Result<CandidateReference, BenchmarkError> {
    Ok(CandidateReference {
        candidate_id: candidate.candidate_id.clone(),
        layout_fingerprint: candidate.layout_fingerprint.clone(),
        source_request_fingerprint: candidate.request_diff.left_request_fingerprint.clone(),
        candidate_request_fingerprint: candidate.request_diff.right_request_fingerprint.clone(),
        request_diff_fingerprint: canonical_hash(&candidate.request_diff)
            .map_err(|error| validation(error.to_string()))?,
    })
}

fn runtime_profile_reference(
    profile: &CapabilityProfile,
) -> Result<RuntimeProfileReference, BenchmarkError> {
    Ok(RuntimeProfileReference {
        profile_id: profile.profile_id.clone(),
        identity_fingerprint: canonical_hash(&profile.capabilities.identity)
            .map_err(|error| validation(error.to_string()))?,
    })
}

fn structural_assessment(candidate: &LayoutCandidate) -> StructuralAssessment {
    let effect = &candidate.structural_effect;
    StructuralAssessment {
        safety: candidate.safety,
        source_inversion_count: effect.source.inversion_count,
        candidate_inversion_count: effect.candidate.inversion_count,
        source_leading_segments: effect.source.stability_aligned_leading_segments,
        candidate_leading_segments: effect.candidate.stability_aligned_leading_segments,
        source_unknown_boundary_count: effect.source.unknown_boundary_count,
        candidate_unknown_boundary_count: effect.candidate.unknown_boundary_count,
        moved_segment_count: effect.moved_segment_count,
        changed_relative_relationships: effect.changed_relative_relationships,
        cache_impact: "unknown".to_string(),
    }
}

fn capability_assessment(
    profile: Option<&CapabilityProfile>,
    reference: Option<RuntimeProfileReference>,
) -> CapabilityGateAssessment {
    let Some(profile) = profile else {
        return CapabilityGateAssessment {
            capability: CapabilityKey::PrefixReuse,
            state: CapabilityAssessment::NotProvided,
            profile: None,
        };
    };
    let cell = profile.capability(CapabilityKey::PrefixReuse);
    CapabilityGateAssessment {
        capability: CapabilityKey::PrefixReuse,
        state: match cell.state {
            CapabilityState::SupportedDocumented => CapabilityAssessment::SupportedDocumented,
            CapabilityState::SupportedObserved => CapabilityAssessment::SupportedObserved,
            CapabilityState::UnsupportedDocumented => CapabilityAssessment::UnsupportedDocumented,
            CapabilityState::UnsupportedObserved => CapabilityAssessment::UnsupportedObserved,
            CapabilityState::UnknownDocumented => CapabilityAssessment::UnknownDocumented,
            CapabilityState::UnknownUnverified => CapabilityAssessment::UnknownUnverified,
        },
        profile: reference,
    }
}

fn classify_observation(
    candidate: &LayoutCandidate,
    profile: Option<&CapabilityProfile>,
    diagnostic: &CacheDiagnostic,
) -> ObservationEvidence {
    let comparison = &diagnostic.observation_comparison;
    let source = if comparison.left.source == comparison.right.source {
        comparison.left.source
    } else {
        EvidenceSourceClass::UnknownUnverified
    };
    let mut reasons = BTreeSet::new();
    if diagnostic.request_observation_alignment != RequestObservationAlignment::Aligned {
        reasons.insert(ObservationRelevanceReason::RequestFingerprintMismatch);
    }
    if diagnostic.request_diff != candidate.request_diff {
        reasons.insert(ObservationRelevanceReason::CandidateMutationMismatch);
    }
    if !envelope_compatible(
        &diagnostic.request_diff.envelope_diff,
        &candidate.request_diff.envelope_diff,
    ) {
        reasons.insert(ObservationRelevanceReason::EnvelopeCompatibilityMismatch);
    }
    if matches!(
        comparison.comparability.level,
        ComparabilityLevel::Incomparable
    ) {
        reasons.insert(ObservationRelevanceReason::IncomparableObservation);
    }
    if matches!(
        comparison.comparability.level,
        ComparabilityLevel::InsufficientEvidence
    ) {
        reasons.insert(ObservationRelevanceReason::InsufficientObservationFields);
    }
    if let Some(profile) = profile {
        for reference in [&comparison.left.runtime, &comparison.right.runtime] {
            add_identity_reasons(
                &mut reasons,
                &profile.profile_id,
                &profile.capabilities,
                reference,
            );
        }
    } else {
        reasons.insert(ObservationRelevanceReason::InsufficientObservationFields);
    }
    let relevance = if reasons.is_empty() {
        ObservationRelevance::Relevant
    } else {
        ObservationRelevance::Rejected
    };
    ObservationEvidence {
        left_observation_id: comparison.left.observation_id.clone(),
        right_observation_id: comparison.right.observation_id.clone(),
        diagnostic_fingerprint: canonical_hash(diagnostic)
            .unwrap_or_else(|_| "diagnostic-fingerprint-unavailable".to_string()),
        source,
        relevance,
        reasons: reasons.into_iter().collect(),
        assessment: diagnostic.assessment,
        association: diagnostic.evidence.association,
        causality: diagnostic.evidence.causality,
    }
}

/// Compare envelope semantics without treating the request fingerprints that
/// identify the surrounding request pair as envelope fields.  A context or
/// ordering mutation can legitimately change those fingerprints while the
/// provider envelope remains identical.
fn envelope_compatible(left: &EnvelopeDiff, right: &EnvelopeDiff) -> bool {
    left.schema_id == right.schema_id
        && left.schema_version == right.schema_version
        && left.identical == right.identical
        && left.changes == right.changes
        && left.cache_impact == right.cache_impact
}

fn add_identity_reasons(
    reasons: &mut BTreeSet<ObservationRelevanceReason>,
    profile_id: &str,
    capabilities: &RuntimeCacheCapabilities,
    reference: &crate::observation_diagnostics::RuntimeIdentityReference,
) {
    if capabilities.identity.backend != reference.backend {
        reasons.insert(ObservationRelevanceReason::RuntimeIdentityMismatch);
    }
    compare_identity_field(
        &capabilities.identity.provider,
        &reference.provider,
        ObservationRelevanceReason::RuntimeIdentityMismatch,
        reasons,
    );
    compare_identity_field(
        &capabilities.identity.model,
        &reference.model,
        ObservationRelevanceReason::ModelIdentityMismatch,
        reasons,
    );
    compare_identity_field(
        &capabilities.identity.protocol,
        &reference.protocol,
        ObservationRelevanceReason::ProtocolIdentityMismatch,
        reasons,
    );
    compare_identity_field(
        &capabilities.identity.runtime,
        &reference.runtime,
        ObservationRelevanceReason::RuntimeIdentityMismatch,
        reasons,
    );
    compare_identity_field(
        &capabilities.identity.runtime_version,
        &reference.runtime_version,
        ObservationRelevanceReason::RuntimeVersionIdentityMismatch,
        reasons,
    );
    if reference
        .profile_id
        .as_deref()
        .is_some_and(|value| value != profile_id)
    {
        reasons.insert(ObservationRelevanceReason::ProfileIdentityMismatch);
    }
}

fn compare_identity_field(
    expected: &Observed<String>,
    observed: &Observed<String>,
    mismatch: ObservationRelevanceReason,
    reasons: &mut BTreeSet<ObservationRelevanceReason>,
) {
    match (expected, observed) {
        (Observed::Known(expected), Observed::Known(observed)) if expected == observed => {}
        (Observed::Known(_), Observed::Known(_)) => {
            reasons.insert(mismatch);
        }
        (Observed::Known(_), Observed::Unknown | Observed::NotObserved) => {
            reasons.insert(ObservationRelevanceReason::InsufficientObservationFields);
        }
        _ => {}
    }
}

fn source_rank(source: EvidenceSourceClass) -> u8 {
    match source {
        EvidenceSourceClass::SyntheticProtocolTest => 0,
        EvidenceSourceClass::DocumentedCapability => 1,
        EvidenceSourceClass::ExperimentallyObservedRuntime => 2,
        EvidenceSourceClass::UnknownUnverified => 3,
    }
}

fn observation_sort_key(observation: &ObservationEvidence) -> (&str, &str, u8) {
    (
        &observation.left_observation_id,
        &observation.right_observation_id,
        source_rank(observation.source),
    )
}

impl EnvironmentState {
    fn validate(&self) -> Result<(), BenchmarkError> {
        if self.blockers.len() > MAX_EVALUATION_BLOCKERS {
            return Err(validation("environment blockers exceed their bound"));
        }
        Ok(())
    }
}

fn validation(message: impl Into<String>) -> BenchmarkError {
    BenchmarkError::validation(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_stability::ContextStabilityInputs;
    use crate::layout_planner::{
        plan_request_layout, LayoutPlanningConstraints, OrderingConstraint,
    };
    use crate::{
        diagnose_cache, load_approved_capability_registry, ConformanceRequest,
        ContextArtifactInput, RequestContext, RequestEnvelope,
    };
    use prefixity_core::observation::{
        ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, CacheBehavior,
        CacheObservation, CapabilityEvidence, CapabilitySupport, ContextArtifact, ContextIdentity,
        ObservationOutcome, Observed, ResourceUsage, RuntimeIdentity, TimingObservation,
        TokenAccounting, TokenCount, CACHE_OBSERVATION_SCHEMA_VERSION,
    };
    use std::collections::BTreeMap;
    use std::path::Path;

    fn request() -> ConformanceRequest {
        ConformanceRequest {
            context: RequestContext {
                system_instruction: "system".to_string(),
                artifacts: vec![
                    ContextArtifactInput {
                        artifact_id: "stable-a".to_string(),
                        content: "stable".to_string(),
                    },
                    ContextArtifactInput {
                        artifact_id: "volatile-b".to_string(),
                        content: "volatile".to_string(),
                    },
                    ContextArtifactInput {
                        artifact_id: "stable-c".to_string(),
                        content: "stable".to_string(),
                    },
                ],
                user_content: "user".to_string(),
                tools: Vec::new(),
            },
            envelope: RequestEnvelope {
                model: "fixture-model".to_string(),
                reasoning: None,
                response_format: None,
            },
        }
    }

    fn artifact(id: &str, stability: ArtifactStability) -> ContextArtifact {
        ContextArtifact {
            schema_version: 1,
            artifact_id: id.to_string(),
            origin_id: format!("origin-{id}"),
            content_source_id: Observed::Known(format!("source-{id}")),
            content_hash: Observed::Unknown,
            revision: Observed::Known("v1".to_string()),
            artifact_type: ArtifactType::Text,
            stability,
            lifecycle: ArtifactLifecycle::PersistentVersioned,
            sizes: ArtifactSizes {
                byte_size: Observed::Known(8),
                ..ArtifactSizes::default()
            },
            cache: Default::default(),
            trust: Observed::Known(prefixity_core::observation::TrustLevel::Trusted),
            provenance: BTreeMap::new(),
            metadata: BTreeMap::new(),
        }
    }

    fn candidate() -> LayoutCandidate {
        let request = request();
        let inputs = ContextStabilityInputs {
            artifacts: BTreeMap::from([
                (
                    "stable-a".to_string(),
                    artifact("stable-a", ArtifactStability::Stable),
                ),
                (
                    "volatile-b".to_string(),
                    artifact("volatile-b", ArtifactStability::Volatile),
                ),
                (
                    "stable-c".to_string(),
                    artifact("stable-c", ArtifactStability::Stable),
                ),
            ]),
            ..ContextStabilityInputs::default()
        };
        let constraints = LayoutPlanningConstraints {
            constraints: ["stable-a", "volatile-b", "stable-c"]
                .into_iter()
                .map(|id| OrderingConstraint::MovableWithinCompatibleRegion {
                    segment: format!("context.artifacts[{id}]"),
                    region: "artifact-sequence".to_string(),
                })
                .collect(),
            provenance: BTreeMap::new(),
        };
        plan_request_layout(&request, &inputs, &constraints)
            .unwrap()
            .candidates
            .into_iter()
            .next()
            .expect("fixture should produce a safe candidate")
    }

    fn profile() -> CapabilityProfile {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        load_approved_capability_registry(&root)
            .unwrap()
            .query(&crate::CapabilityQuery {
                protocol: Some("llama.cpp-openai-chat-v1".to_string()),
                ..Default::default()
            })
            .into_iter()
            .next()
            .unwrap()
            .clone()
    }

    fn observation(id: &str, request_fingerprint: String, cached: u64) -> CacheObservation {
        CacheObservation {
            schema_version: CACHE_OBSERVATION_SCHEMA_VERSION,
            observation_id: id.to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            runtime: RuntimeIdentity {
                backend: "llama.cpp".to_string(),
                provider: Observed::Known("local".to_string()),
                model: Observed::Known("fixture-model".to_string()),
                protocol: Observed::Known("llama.cpp-openai-chat-v1".to_string()),
                runtime: Observed::Known("llama.cpp".to_string()),
                ..RuntimeIdentity::default()
            },
            context: ContextIdentity {
                serialized_request_identity: Observed::Known(request_fingerprint),
                reusable_prefix_identity: Observed::Known("fixture-context".to_string()),
                ..ContextIdentity::default()
            },
            accounting: TokenAccounting {
                provider_cached_tokens: Observed::Known(TokenCount {
                    count: cached,
                    provider: Observed::Known("local".to_string()),
                    model: Observed::Known("fixture-model".to_string()),
                    tokenizer: Observed::Known("fixture-tokenizer".to_string()),
                }),
                ..TokenAccounting::default()
            },
            timing: TimingObservation::default(),
            resources: ResourceUsage::default(),
            cache: CacheBehavior::default(),
            outcome: ObservationOutcome::default(),
            raw_telemetry: BTreeMap::new(),
        }
    }

    fn diagnostic(
        candidate: &LayoutCandidate,
        source: EvidenceSourceClass,
        right_cached: u64,
    ) -> CacheDiagnostic {
        let left = observation(
            "left",
            candidate.request_diff.left_request_fingerprint.clone(),
            10,
        );
        let right = observation(
            "right",
            candidate.request_diff.right_request_fingerprint.clone(),
            right_cached,
        );
        let mut diagnostic = diagnose_cache(&candidate.request_diff, &left, &right);
        diagnostic.observation_comparison.left.source = source;
        diagnostic.observation_comparison.right.source = source;
        diagnostic
    }

    fn evaluate<'a>(
        candidate: &'a LayoutCandidate,
        profile: Option<&'a CapabilityProfile>,
        observations: &'a [CacheDiagnostic],
    ) -> CandidateEvaluation {
        evaluate_candidate(CandidateEvaluationInput {
            candidate,
            capability_profile: profile,
            observations,
            environment: &EnvironmentState::available(),
        })
        .unwrap()
    }

    #[test]
    fn candidate_alone_is_structural_only_and_not_executable() {
        let candidate = candidate();
        let evaluation = evaluate(&candidate, None, &[]);
        assert_eq!(evaluation.evidence_state, EvidenceState::StructuralOnly);
        assert_eq!(
            evaluation.experiment_readiness.design,
            DesignReadiness::Blocked
        );
        assert_eq!(
            evaluation.experiment_readiness.execution,
            ExecutionReadiness::Blocked
        );
        assert_eq!(evaluation.next_action, NextAction::GatherCapabilityEvidence);
        assert_eq!(
            evaluation.claim_permissions.performance_claims,
            ClaimPermission::NotAllowed
        );
    }

    #[test]
    fn documented_capability_is_ready_for_experiment_but_not_support() {
        let candidate = candidate();
        let evaluation = evaluate(&candidate, Some(&profile()), &[]);
        assert_eq!(evaluation.evidence_state, EvidenceState::ReadyForExperiment);
        assert_eq!(
            evaluation.experiment_readiness.design,
            DesignReadiness::Ready
        );
        assert_eq!(
            evaluation.experiment_readiness.execution,
            ExecutionReadiness::Ready
        );
        assert_eq!(evaluation.next_action, NextAction::ControlledExperiment);
        assert_eq!(
            evaluation.blockers,
            vec![EvidenceBlocker::NoExperimentalObservation]
        );
    }

    #[test]
    fn unknown_and_unsupported_capabilities_are_distinct_blockers() {
        let candidate = candidate();
        let mut unknown = profile();
        unknown.capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Unknown;
        unknown.capabilities.prefix_cache.prefix_reuse.evidence = CapabilityEvidence::Unverified;
        unknown.capabilities.prefix_cache.prefix_reuse.details = Observed::NotObserved;
        let unknown = CapabilityProfile::from_capabilities(
            unknown.capabilities,
            crate::RegistryEvidenceOrigin::SyntheticFixture,
            Default::default(),
        )
        .unwrap();
        let evaluation = evaluate(&candidate, Some(&unknown), &[]);
        assert_eq!(evaluation.evidence_state, EvidenceState::Blocked);
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::RuntimeCapabilityPending));

        let mut unsupported = profile();
        unsupported.capabilities.prefix_cache.prefix_reuse.support = CapabilitySupport::Unsupported;
        unsupported.capabilities.prefix_cache.prefix_reuse.evidence =
            CapabilityEvidence::Documented;
        unsupported.capabilities.prefix_cache.prefix_reuse.details = Observed::Known(false);
        let unsupported = CapabilityProfile::from_capabilities(
            unsupported.capabilities,
            crate::RegistryEvidenceOrigin::ProjectDocumentation,
            Default::default(),
        )
        .unwrap();
        let evaluation = evaluate(&candidate, Some(&unsupported), &[]);
        assert_eq!(evaluation.evidence_state, EvidenceState::Blocked);
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::RuntimeCapabilityUnsupported));
        assert_eq!(evaluation.next_action, NextAction::RejectCandidate);
    }

    #[test]
    fn synthetic_observation_cannot_promote_real_support() {
        let candidate = candidate();
        let diagnostics = vec![diagnostic(
            &candidate,
            EvidenceSourceClass::SyntheticProtocolTest,
            50,
        )];
        let evaluation = evaluate(&candidate, Some(&profile()), &diagnostics);
        assert_ne!(
            evaluation.evidence_state,
            EvidenceState::ObservationallySupported,
            "{evaluation:?}"
        );
        assert!(
            evaluation
                .blockers
                .contains(&EvidenceBlocker::SyntheticOnlyEvidence),
            "{evaluation:?}"
        );
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::NoExperimentalObservation));
    }

    #[test]
    fn experimental_positive_evidence_is_observational_only() {
        let candidate = candidate();
        let diagnostics = vec![diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        )];
        let evaluation = evaluate(&candidate, Some(&profile()), &diagnostics);
        assert_eq!(
            evaluation.evidence_state,
            EvidenceState::ObservationallySupported,
            "{evaluation:?}"
        );
        assert!(evaluation
            .observations
            .iter()
            .all(|observation| observation.causality == CausalityStatus::NotEstablished));
        assert_eq!(
            evaluation.claim_permissions.causal_claims,
            ClaimPermission::NotAllowed
        );
    }

    #[test]
    fn mixed_and_contrary_evidence_remain_distinct() {
        let candidate = candidate();
        let mut mixed = diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        );
        mixed.assessment = CacheRegressionAssessment::MixedObservations;
        let evaluation = evaluate(&candidate, Some(&profile()), &[mixed]);
        assert_eq!(evaluation.evidence_state, EvidenceState::MixedEvidence);

        let contrary = diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            5,
        );
        let evaluation = evaluate(&candidate, Some(&profile()), &[contrary]);
        assert_eq!(
            evaluation.evidence_state,
            EvidenceState::UnsupportedByCurrentEvidence
        );
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::NoObservedCacheBenefit));
    }

    #[test]
    fn identity_and_candidate_mismatches_reject_observations() {
        let candidate = candidate();
        let mut diagnostic = diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        );
        diagnostic.request_observation_alignment = RequestObservationAlignment::Mismatched;
        let evaluation = evaluate(&candidate, Some(&profile()), &[diagnostic]);
        assert_eq!(evaluation.evidence_state, EvidenceState::ReadyForExperiment);
        assert_eq!(
            evaluation.observations[0].relevance,
            ObservationRelevance::Rejected
        );
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::ObservationIdentityMismatch));
    }

    #[test]
    fn unrelated_request_fingerprint_does_not_imply_envelope_mismatch() {
        let candidate = candidate();
        let mut diagnostic = diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        );
        diagnostic.request_diff.left_request_fingerprint = "unrelated-left".to_string();
        diagnostic.request_diff.right_request_fingerprint = "unrelated-right".to_string();
        let evaluation = evaluate(&candidate, Some(&profile()), &[diagnostic]);
        assert_eq!(
            evaluation.observations[0].relevance,
            ObservationRelevance::Rejected
        );
        assert!(evaluation.observations[0]
            .reasons
            .contains(&ObservationRelevanceReason::CandidateMutationMismatch));
        assert!(!evaluation.observations[0]
            .reasons
            .contains(&ObservationRelevanceReason::EnvelopeCompatibilityMismatch));
    }

    #[test]
    fn genuine_envelope_difference_remains_rejected_as_envelope_mismatch() {
        let candidate = candidate();
        let mut diagnostic = diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        );
        diagnostic.request_diff.envelope_diff.identical = false;
        let evaluation = evaluate(&candidate, Some(&profile()), &[diagnostic]);
        assert_eq!(
            evaluation.observations[0].relevance,
            ObservationRelevance::Rejected
        );
        assert!(evaluation.observations[0]
            .reasons
            .contains(&ObservationRelevanceReason::EnvelopeCompatibilityMismatch));
    }

    #[test]
    fn runtime_model_identity_mismatch_cannot_influence_evidence() {
        let candidate = candidate();
        let mut mismatch = profile();
        mismatch.capabilities.identity.model = Observed::Known("other-model".to_string());
        let mismatch = CapabilityProfile::from_capabilities(
            mismatch.capabilities,
            crate::RegistryEvidenceOrigin::ProjectDocumentation,
            Default::default(),
        )
        .unwrap();
        let diagnostics = vec![diagnostic(
            &candidate,
            EvidenceSourceClass::ExperimentallyObservedRuntime,
            50,
        )];
        let evaluation = evaluate(&candidate, Some(&mismatch), &diagnostics);
        assert_eq!(evaluation.evidence_state, EvidenceState::ReadyForExperiment);
        assert_eq!(
            evaluation.observations[0].relevance,
            ObservationRelevance::Rejected
        );
        assert!(evaluation.observations[0]
            .reasons
            .contains(&ObservationRelevanceReason::ModelIdentityMismatch));
    }

    #[test]
    fn unsafe_candidate_cannot_become_experiment_ready() {
        let mut candidate = candidate();
        candidate.safety = CandidateSafetyStatus::Rejected;
        let evaluation = evaluate(&candidate, Some(&profile()), &[]);
        assert_eq!(evaluation.evidence_state, EvidenceState::Blocked);
        assert_eq!(evaluation.next_action, NextAction::RejectCandidate);
        assert!(evaluation
            .blockers
            .contains(&EvidenceBlocker::CandidateSafetyNotEstablished));
    }

    #[test]
    fn environment_blocked_is_separate_from_design_readiness() {
        let candidate = candidate();
        let environment = EnvironmentState::blocked(vec![EvidenceBlocker::EnvironmentUnavailable]);
        let evaluation = evaluate_candidate(CandidateEvaluationInput {
            candidate: &candidate,
            capability_profile: Some(&profile()),
            observations: &[],
            environment: &environment,
        })
        .unwrap();
        assert_eq!(evaluation.evidence_state, EvidenceState::ReadyForExperiment);
        assert_eq!(
            evaluation.experiment_readiness.design,
            DesignReadiness::Ready
        );
        assert_eq!(
            evaluation.experiment_readiness.environment,
            EnvironmentReadiness::Blocked
        );
        assert_eq!(
            evaluation.experiment_readiness.execution,
            ExecutionReadiness::Blocked
        );
        assert_eq!(evaluation.next_action, NextAction::ResolveEnvironment);
    }

    #[test]
    fn repeated_evaluation_is_deterministic_and_input_is_unchanged() {
        let candidate = candidate();
        let before = candidate.clone();
        let first = evaluate(&candidate, Some(&profile()), &[])
            .canonical_json()
            .unwrap();
        let second = evaluate(&candidate, Some(&profile()), &[])
            .canonical_json()
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(candidate, before);
    }
}
