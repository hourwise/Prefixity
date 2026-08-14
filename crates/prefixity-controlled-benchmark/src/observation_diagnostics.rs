//! Evidence-backed comparison of two neutral cache observations.
//!
//! P0-L8 deliberately compares references to observations rather than
//! embedding the observations themselves.  The result is a bounded diagnostic
//! aid: it reports directional metric changes and association with a P0-L7
//! request diff, but it does not claim causality, statistical significance, or
//! a universal performance score.

use crate::conformance::ConformanceCaseResult;
use crate::diff::{ChangeCategory, RequestDiff};
use crate::hashing::canonical_hash;
use prefixity_core::observation::{CacheObservation, Observed, RuntimeIdentity, TokenCount};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const OBSERVATION_COMPARISON_SCHEMA_ID: &str = "prefixity.observation-comparison";
pub const OBSERVATION_COMPARISON_SCHEMA_VERSION: u32 = 1;
pub const CACHE_DIAGNOSTIC_SCHEMA_ID: &str = "prefixity.cache-diagnostic";
pub const CACHE_DIAGNOSTIC_SCHEMA_VERSION: u32 = 1;

const MAX_COMPARABILITY_REASONS: usize = 16;
const MAX_SIGNAL_DIRECTIONS: usize = 32;

/// Evidence source classes used by diagnostics.  A capability document is
/// documentation, not an observation; this enum keeps the distinction
/// visible when a diagnostic is assembled from controlled evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceClass {
    SyntheticProtocolTest,
    DocumentedCapability,
    ExperimentallyObservedRuntime,
    UnknownUnverified,
}

/// A compact identity projection used for comparison and audit.  It contains
/// identity dimensions and no raw telemetry or full observation payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIdentityReference {
    pub backend: String,
    #[serde(default)]
    pub provider: Observed<String>,
    #[serde(default)]
    pub model: Observed<String>,
    #[serde(default)]
    pub protocol: Observed<String>,
    #[serde(default)]
    pub runtime: Observed<String>,
    #[serde(default)]
    pub runtime_version: Observed<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    pub identity_fingerprint: String,
}

/// Reference to one observation.  Request/context/envelope fingerprints are
/// retained instead of copying the observation or its raw adapter telemetry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationReference {
    pub observation_schema_version: u32,
    pub observation_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub envelope_fingerprint: Option<String>,
    pub runtime: RuntimeIdentityReference,
    pub source: EvidenceSourceClass,
}

struct ReferenceMetadata {
    experiment_id: Option<String>,
    case_id: Option<String>,
    request_fingerprint: Option<String>,
    context_fingerprint: Option<String>,
    envelope_fingerprint: Option<String>,
    profile_id: Option<String>,
    source: EvidenceSourceClass,
}

impl ObservationReference {
    fn from_observation(observation: &CacheObservation, metadata: ReferenceMetadata) -> Self {
        let runtime = RuntimeIdentityReference::new(&observation.runtime, metadata.profile_id);
        Self {
            observation_schema_version: observation.schema_version,
            observation_id: observation.observation_id.clone(),
            experiment_id: metadata.experiment_id,
            case_id: metadata.case_id,
            request_fingerprint: metadata.request_fingerprint,
            context_fingerprint: metadata.context_fingerprint,
            envelope_fingerprint: metadata.envelope_fingerprint,
            runtime,
            source: metadata.source,
        }
    }

    /// Build a reference for an observation whose conformance fingerprints are
    /// not available.  The neutral observation identity fields are used when
    /// they were explicitly observed.
    pub fn from_observation_only(observation: &CacheObservation) -> Self {
        Self::from_observation(
            observation,
            ReferenceMetadata {
                experiment_id: None,
                case_id: None,
                request_fingerprint: known_string(&observation.context.serialized_request_identity),
                context_fingerprint: known_string(&observation.context.reusable_prefix_identity),
                envelope_fingerprint: None,
                profile_id: None,
                source: EvidenceSourceClass::UnknownUnverified,
            },
        )
    }

    /// Build a compact reference to a controlled conformance result.
    pub fn from_conformance_case(result: &ConformanceCaseResult, profile_id: Option<&str>) -> Self {
        Self::from_conformance_case_with_source(
            result,
            profile_id,
            EvidenceSourceClass::SyntheticProtocolTest,
        )
    }

    /// Build a compact reference to a controlled conformance result while
    /// preserving the evidence origin assigned by the caller.
    pub fn from_conformance_case_with_source(
        result: &ConformanceCaseResult,
        profile_id: Option<&str>,
        source: EvidenceSourceClass,
    ) -> Self {
        Self::from_observation(
            &result.observation,
            ReferenceMetadata {
                experiment_id: Some(result.experiment_id.clone()),
                case_id: Some(result.case_id.clone()),
                request_fingerprint: Some(result.request_fingerprint.clone()),
                context_fingerprint: Some(result.context_fingerprint.clone()),
                envelope_fingerprint: None,
                profile_id: profile_id.map(str::to_owned),
                source,
            },
        )
    }
}

impl RuntimeIdentityReference {
    fn new(identity: &RuntimeIdentity, profile_id: Option<String>) -> Self {
        let mut value = Self {
            backend: identity.backend.clone(),
            provider: identity.provider.clone(),
            model: identity.model.clone(),
            protocol: identity.protocol.clone(),
            runtime: identity.runtime.clone(),
            runtime_version: identity.runtime_version.clone(),
            profile_id,
            identity_fingerprint: String::new(),
        };
        value.identity_fingerprint = canonical_hash(&value)
            .unwrap_or_else(|_| "identity-fingerprint-unavailable".to_string());
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityMatch {
    Match,
    Mismatch,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentityComparison {
    pub backend: IdentityMatch,
    pub provider: IdentityMatch,
    pub model: IdentityMatch,
    pub protocol: IdentityMatch,
    pub runtime: IdentityMatch,
    pub profile: IdentityMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparabilityLevel {
    DirectlyComparable,
    PartiallyComparable,
    Incomparable,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparabilityReason {
    RuntimeIdentityMismatch,
    ModelIdentityMismatch,
    ProviderIdentityMismatch,
    ProtocolIdentityMismatch,
    ProfileIdentityMismatch,
    MissingRequestFingerprint,
    MissingContextFingerprint,
    MissingIdentityDimension,
    ObservationSchemaMismatch,
    TokenScopeMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparabilityReport {
    pub level: ComparabilityLevel,
    pub identity: IdentityComparison,
    pub reasons: Vec<ComparabilityReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricDirection {
    Increased,
    Decreased,
    Unchanged,
    Unavailable,
}

/// Directional change for a token count.  `left` and `right` preserve the
/// original Known/Unknown/NotObserved distinction; `delta` is absent for a
/// missing value or incompatible token scope.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenMetricDelta {
    pub left: Observed<TokenCount>,
    pub right: Observed<TokenCount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_change: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_change_denominator: Option<u64>,
    pub direction: MetricDirection,
}

/// Directional change for timing/resource measurements.  The relative value
/// is normalized only to the left-hand measurement; it is not a score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NumericMetricDelta {
    pub left: Observed<u64>,
    pub right: Observed<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_change: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_change_denominator: Option<u64>,
    pub direction: MetricDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenDeltas {
    pub transmitted_input_tokens: TokenMetricDelta,
    pub provider_cached_tokens: TokenMetricDelta,
    pub fresh_prefill_tokens: TokenMetricDelta,
    pub reconstructed_context_tokens: TokenMetricDelta,
    pub output_tokens: TokenMetricDelta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TimingDeltas {
    pub prefill_duration_ms: NumericMetricDelta,
    pub time_to_first_token_ms: NumericMetricDelta,
    pub generation_duration_ms: NumericMetricDelta,
    pub wall_duration_ms: NumericMetricDelta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceDeltas {
    pub ram_bytes: NumericMetricDelta,
    pub vram_bytes: NumericMetricDelta,
    pub kv_cache_bytes: NumericMetricDelta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenMetricName {
    TransmittedInputTokens,
    ProviderCachedTokens,
    FreshPrefillTokens,
    ReconstructedContextTokens,
    OutputTokens,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedRatio {
    pub derived: bool,
    pub numerator: TokenMetricName,
    pub denominator: TokenMetricName,
    pub left_numerator: Observed<TokenCount>,
    pub right_numerator: Observed<TokenCount>,
    pub left_denominator: Observed<TokenCount>,
    pub right_denominator: Observed<TokenCount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left_value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right_value: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    pub direction: MetricDirection,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DerivedMetrics {
    pub reuse_ratio: DerivedRatio,
    pub fresh_prefill_ratio: DerivedRatio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationComparison {
    pub schema_id: String,
    pub schema_version: u32,
    pub left: ObservationReference,
    pub right: ObservationReference,
    pub comparability: ComparabilityReport,
    pub token_deltas: TokenDeltas,
    pub derived_metrics: DerivedMetrics,
    pub timing_deltas: TimingDeltas,
    pub resource_deltas: ResourceDeltas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheRegressionAssessment {
    InsufficientEvidence,
    NoObservedCacheReuseChange,
    ObservedReuseIncrease,
    ObservedReuseDecrease,
    MixedObservations,
    Incomparable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CausalityStatus {
    NotEstablished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAssociation {
    NoStructuralDifference,
    StructuralDifferenceWithoutComparableObservation,
    StructuralDifferenceWithObservedMetricChange,
    StructuralDifferenceWithObservedReuseSignal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestObservationAlignment {
    Aligned,
    Mismatched,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticMetric {
    TransmittedInputTokens,
    ProviderCachedTokens,
    FreshPrefillTokens,
    ReconstructedContextTokens,
    OutputTokens,
    PrefillDurationMs,
    TimeToFirstTokenMs,
    GenerationDurationMs,
    WallDurationMs,
    RamBytes,
    VramBytes,
    KvCacheBytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceStatement {
    pub structural_change_categories: Vec<String>,
    pub observed_metric_directions: Vec<(DiagnosticMetric, MetricDirection)>,
    pub association: EvidenceAssociation,
    pub causality: CausalityStatus,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheDiagnostic {
    pub schema_id: String,
    pub schema_version: u32,
    pub request_diff: RequestDiff,
    pub observation_comparison: ObservationComparison,
    pub request_observation_alignment: RequestObservationAlignment,
    pub assessment: CacheRegressionAssessment,
    pub evidence: EvidenceStatement,
}

/// Compare two observations without embedding either raw observation in the
/// result.
pub fn compare_observations(
    left: &CacheObservation,
    right: &CacheObservation,
) -> ObservationComparison {
    compare_observations_with_references(
        left,
        right,
        ObservationReference::from_observation_only(left),
        ObservationReference::from_observation_only(right),
    )
}

/// Compare two controlled conformance observations using their stable case
/// and request/context references.
pub fn compare_conformance_cases(
    left: &ConformanceCaseResult,
    right: &ConformanceCaseResult,
    profile_id: Option<&str>,
) -> ObservationComparison {
    compare_observations_with_references(
        &left.observation,
        &right.observation,
        ObservationReference::from_conformance_case(left, profile_id),
        ObservationReference::from_conformance_case(right, profile_id),
    )
}

/// Combine a P0-L7 structural diff with an observation comparison.  The
/// result is association-only: causal attribution remains explicitly absent.
pub fn diagnose_cache(
    request_diff: &RequestDiff,
    left: &CacheObservation,
    right: &CacheObservation,
) -> CacheDiagnostic {
    let comparison = compare_observations(left, right);
    build_diagnostic(request_diff.clone(), comparison)
}

/// Variant of [`diagnose_cache`] for controlled conformance results.
pub fn diagnose_conformance_cache(
    request_diff: &RequestDiff,
    left: &ConformanceCaseResult,
    right: &ConformanceCaseResult,
    profile_id: Option<&str>,
) -> CacheDiagnostic {
    diagnose_conformance_cache_with_source(
        request_diff,
        left,
        right,
        profile_id,
        EvidenceSourceClass::SyntheticProtocolTest,
    )
}

/// Variant of [`diagnose_conformance_cache`] that preserves the evidence
/// origin supplied by the caller.  The default constructor remains synthetic
/// for backwards compatibility with protocol-only fixtures.
pub fn diagnose_conformance_cache_with_source(
    request_diff: &RequestDiff,
    left: &ConformanceCaseResult,
    right: &ConformanceCaseResult,
    profile_id: Option<&str>,
    source: EvidenceSourceClass,
) -> CacheDiagnostic {
    let comparison = compare_observations_with_references(
        &left.observation,
        &right.observation,
        ObservationReference::from_conformance_case_with_source(left, profile_id, source),
        ObservationReference::from_conformance_case_with_source(right, profile_id, source),
    );
    build_diagnostic(request_diff.clone(), comparison)
}

fn compare_observations_with_references(
    left: &CacheObservation,
    right: &CacheObservation,
    left_reference: ObservationReference,
    right_reference: ObservationReference,
) -> ObservationComparison {
    let comparability = compare_identity(&left_reference, &right_reference);
    let token_deltas = TokenDeltas {
        transmitted_input_tokens: token_delta(
            &left.accounting.transmitted_input_tokens,
            &right.accounting.transmitted_input_tokens,
        ),
        provider_cached_tokens: token_delta(
            &left.accounting.provider_cached_tokens,
            &right.accounting.provider_cached_tokens,
        ),
        fresh_prefill_tokens: token_delta(
            &left.accounting.fresh_prefill_tokens,
            &right.accounting.fresh_prefill_tokens,
        ),
        reconstructed_context_tokens: token_delta(
            &left.accounting.reconstructed_context_tokens,
            &right.accounting.reconstructed_context_tokens,
        ),
        output_tokens: token_delta(
            &left.accounting.output_tokens,
            &right.accounting.output_tokens,
        ),
    };
    let derived_metrics = DerivedMetrics {
        reuse_ratio: derived_ratio(
            TokenMetricName::ProviderCachedTokens,
            &left.accounting.provider_cached_tokens,
            &right.accounting.provider_cached_tokens,
            TokenMetricName::TransmittedInputTokens,
            &left.accounting.transmitted_input_tokens,
            &right.accounting.transmitted_input_tokens,
        ),
        fresh_prefill_ratio: derived_ratio(
            TokenMetricName::FreshPrefillTokens,
            &left.accounting.fresh_prefill_tokens,
            &right.accounting.fresh_prefill_tokens,
            TokenMetricName::TransmittedInputTokens,
            &left.accounting.transmitted_input_tokens,
            &right.accounting.transmitted_input_tokens,
        ),
    };
    ObservationComparison {
        schema_id: OBSERVATION_COMPARISON_SCHEMA_ID.to_string(),
        schema_version: OBSERVATION_COMPARISON_SCHEMA_VERSION,
        left: left_reference,
        right: right_reference,
        comparability,
        token_deltas,
        derived_metrics,
        timing_deltas: TimingDeltas {
            prefill_duration_ms: numeric_delta(
                &left.timing.prefill_duration_ms,
                &right.timing.prefill_duration_ms,
            ),
            time_to_first_token_ms: numeric_delta(
                &left.timing.time_to_first_token_ms,
                &right.timing.time_to_first_token_ms,
            ),
            generation_duration_ms: numeric_delta(
                &left.timing.generation_duration_ms,
                &right.timing.generation_duration_ms,
            ),
            wall_duration_ms: numeric_delta(
                &left.timing.wall_duration_ms,
                &right.timing.wall_duration_ms,
            ),
        },
        resource_deltas: ResourceDeltas {
            ram_bytes: numeric_delta(&left.resources.ram_bytes, &right.resources.ram_bytes),
            vram_bytes: numeric_delta(&left.resources.vram_bytes, &right.resources.vram_bytes),
            kv_cache_bytes: numeric_delta(
                &left.resources.kv_cache_bytes,
                &right.resources.kv_cache_bytes,
            ),
        },
    }
}

fn compare_identity(
    left: &ObservationReference,
    right: &ObservationReference,
) -> ComparabilityReport {
    let identity = IdentityComparison {
        backend: compare_required_string(&left.runtime.backend, &right.runtime.backend),
        provider: compare_observed_string(&left.runtime.provider, &right.runtime.provider),
        model: compare_observed_string(&left.runtime.model, &right.runtime.model),
        protocol: compare_observed_string(&left.runtime.protocol, &right.runtime.protocol),
        runtime: compare_observed_string(&left.runtime.runtime, &right.runtime.runtime),
        profile: compare_optional_string(
            left.runtime.profile_id.as_deref(),
            right.runtime.profile_id.as_deref(),
        ),
    };
    let mut reasons = Vec::new();
    if identity.backend == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::RuntimeIdentityMismatch);
    }
    if identity.provider == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::ProviderIdentityMismatch);
    }
    if identity.model == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::ModelIdentityMismatch);
    }
    if identity.protocol == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::ProtocolIdentityMismatch);
    }
    if identity.runtime == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::RuntimeIdentityMismatch);
    }
    if identity.profile == IdentityMatch::Mismatch {
        push_reason(&mut reasons, ComparabilityReason::ProfileIdentityMismatch);
    }
    if left.observation_schema_version != right.observation_schema_version {
        push_reason(&mut reasons, ComparabilityReason::ObservationSchemaMismatch);
    }
    if left.request_fingerprint.is_none() || right.request_fingerprint.is_none() {
        push_reason(&mut reasons, ComparabilityReason::MissingRequestFingerprint);
    }
    if left.context_fingerprint.is_none() || right.context_fingerprint.is_none() {
        push_reason(&mut reasons, ComparabilityReason::MissingContextFingerprint);
    }
    if identity.provider == IdentityMatch::Missing
        || identity.model == IdentityMatch::Missing
        || identity.protocol == IdentityMatch::Missing
        || identity.runtime == IdentityMatch::Missing
        || identity.profile == IdentityMatch::Missing
    {
        push_reason(&mut reasons, ComparabilityReason::MissingIdentityDimension);
    }
    let mismatch = reasons.iter().any(|reason| {
        matches!(
            reason,
            ComparabilityReason::RuntimeIdentityMismatch
                | ComparabilityReason::ModelIdentityMismatch
                | ComparabilityReason::ProviderIdentityMismatch
                | ComparabilityReason::ProtocolIdentityMismatch
                | ComparabilityReason::ProfileIdentityMismatch
                | ComparabilityReason::ObservationSchemaMismatch
        )
    });
    let missing_fingerprint = reasons.iter().any(|reason| {
        matches!(
            reason,
            ComparabilityReason::MissingRequestFingerprint
                | ComparabilityReason::MissingContextFingerprint
        )
    });
    let missing_identity = reasons.contains(&ComparabilityReason::MissingIdentityDimension);
    let level = if mismatch {
        ComparabilityLevel::Incomparable
    } else if missing_fingerprint {
        ComparabilityLevel::InsufficientEvidence
    } else if missing_identity {
        ComparabilityLevel::PartiallyComparable
    } else {
        ComparabilityLevel::DirectlyComparable
    };
    ComparabilityReport {
        level,
        identity,
        reasons,
    }
}

fn push_reason(reasons: &mut Vec<ComparabilityReason>, reason: ComparabilityReason) {
    if !reasons.contains(&reason) && reasons.len() < MAX_COMPARABILITY_REASONS {
        reasons.push(reason);
    }
}

fn compare_required_string(left: &str, right: &str) -> IdentityMatch {
    if left.is_empty() || right.is_empty() {
        IdentityMatch::Missing
    } else if left == right {
        IdentityMatch::Match
    } else {
        IdentityMatch::Mismatch
    }
}

fn compare_optional_string(left: Option<&str>, right: Option<&str>) -> IdentityMatch {
    match (left, right) {
        (Some(left), Some(right)) if left == right => IdentityMatch::Match,
        (Some(_), Some(_)) => IdentityMatch::Mismatch,
        _ => IdentityMatch::Missing,
    }
}

fn compare_observed_string(left: &Observed<String>, right: &Observed<String>) -> IdentityMatch {
    match (left, right) {
        (Observed::Known(left), Observed::Known(right)) if left == right => IdentityMatch::Match,
        (Observed::Known(_), Observed::Known(_)) => IdentityMatch::Mismatch,
        _ => IdentityMatch::Missing,
    }
}

fn known_string(value: &Observed<String>) -> Option<String> {
    match value {
        Observed::Known(value) => Some(value.clone()),
        Observed::Unknown | Observed::NotObserved => None,
    }
}

fn token_delta(left: &Observed<TokenCount>, right: &Observed<TokenCount>) -> TokenMetricDelta {
    let (delta, relative_change, denominator) = match (left, right) {
        (Observed::Known(left), Observed::Known(right)) if token_scopes_compatible(left, right) => {
            numeric_values(left.count, right.count)
        }
        _ => (None, None, None),
    };
    TokenMetricDelta {
        left: left.clone(),
        right: right.clone(),
        direction: direction(delta),
        delta,
        relative_change,
        relative_change_denominator: denominator,
    }
}

fn numeric_delta(left: &Observed<u64>, right: &Observed<u64>) -> NumericMetricDelta {
    let (delta, relative_change, denominator) = match (left, right) {
        (Observed::Known(left), Observed::Known(right)) => numeric_values(*left, *right),
        _ => (None, None, None),
    };
    NumericMetricDelta {
        left: left.clone(),
        right: right.clone(),
        direction: direction(delta),
        delta,
        relative_change,
        relative_change_denominator: denominator,
    }
}

fn numeric_values(left: u64, right: u64) -> (Option<i64>, Option<f64>, Option<u64>) {
    let delta = i128::from(right) - i128::from(left);
    let delta = i64::try_from(delta).ok();
    let denominator = (left > 0).then_some(left);
    let relative_change = match (delta, denominator) {
        (Some(delta), Some(denominator)) => Some(delta as f64 / denominator as f64),
        _ => None,
    };
    (delta, relative_change, denominator)
}

fn direction(delta: Option<i64>) -> MetricDirection {
    match delta {
        Some(delta) if delta > 0 => MetricDirection::Increased,
        Some(delta) if delta < 0 => MetricDirection::Decreased,
        Some(_) => MetricDirection::Unchanged,
        None => MetricDirection::Unavailable,
    }
}

fn token_scopes_compatible(left: &TokenCount, right: &TokenCount) -> bool {
    observed_scope_matches(&left.provider, &right.provider)
        && observed_scope_matches(&left.model, &right.model)
        && observed_scope_matches(&left.tokenizer, &right.tokenizer)
}

fn observed_scope_matches(left: &Observed<String>, right: &Observed<String>) -> bool {
    !matches!((left, right), (Observed::Known(left), Observed::Known(right)) if left != right)
}

fn derived_ratio(
    numerator: TokenMetricName,
    left_numerator: &Observed<TokenCount>,
    right_numerator: &Observed<TokenCount>,
    denominator: TokenMetricName,
    left_denominator: &Observed<TokenCount>,
    right_denominator: &Observed<TokenCount>,
) -> DerivedRatio {
    let left_value = ratio_value(left_numerator, left_denominator);
    let right_value = ratio_value(right_numerator, right_denominator);
    let delta = match (left_value, right_value) {
        (Some(left), Some(right)) => Some(right - left),
        _ => None,
    };
    DerivedRatio {
        derived: true,
        numerator,
        denominator,
        left_numerator: left_numerator.clone(),
        right_numerator: right_numerator.clone(),
        left_denominator: left_denominator.clone(),
        right_denominator: right_denominator.clone(),
        direction: derived_direction(delta),
        left_value,
        right_value,
        delta,
    }
}

fn ratio_value(
    numerator: &Observed<TokenCount>,
    denominator: &Observed<TokenCount>,
) -> Option<f64> {
    match (numerator, denominator) {
        (Observed::Known(numerator), Observed::Known(denominator))
            if denominator.count > 0 && token_scopes_compatible(numerator, denominator) =>
        {
            Some(numerator.count as f64 / denominator.count as f64)
        }
        _ => None,
    }
}

fn derived_direction(delta: Option<f64>) -> MetricDirection {
    match delta {
        Some(delta) if delta > f64::EPSILON => MetricDirection::Increased,
        Some(delta) if delta < -f64::EPSILON => MetricDirection::Decreased,
        Some(_) => MetricDirection::Unchanged,
        None => MetricDirection::Unavailable,
    }
}

fn build_diagnostic(
    request_diff: RequestDiff,
    comparison: ObservationComparison,
) -> CacheDiagnostic {
    let alignment = request_observation_alignment(&request_diff, &comparison);
    let assessment = assess_cache(&comparison, alignment);
    let evidence = evidence_statement(&request_diff, &comparison, assessment, alignment);
    CacheDiagnostic {
        schema_id: CACHE_DIAGNOSTIC_SCHEMA_ID.to_string(),
        schema_version: CACHE_DIAGNOSTIC_SCHEMA_VERSION,
        request_diff,
        observation_comparison: comparison,
        request_observation_alignment: alignment,
        assessment,
        evidence,
    }
}

fn assess_cache(
    comparison: &ObservationComparison,
    alignment: RequestObservationAlignment,
) -> CacheRegressionAssessment {
    if alignment == RequestObservationAlignment::Mismatched {
        return CacheRegressionAssessment::InsufficientEvidence;
    }
    match comparison.comparability.level {
        ComparabilityLevel::Incomparable => return CacheRegressionAssessment::Incomparable,
        ComparabilityLevel::InsufficientEvidence => {
            return CacheRegressionAssessment::InsufficientEvidence
        }
        ComparabilityLevel::DirectlyComparable | ComparabilityLevel::PartiallyComparable => {}
    }
    let reuse = comparison.token_deltas.provider_cached_tokens.direction;
    if reuse == MetricDirection::Unavailable {
        return CacheRegressionAssessment::InsufficientEvidence;
    }
    if reuse == MetricDirection::Unchanged {
        return CacheRegressionAssessment::NoObservedCacheReuseChange;
    }
    let timing_changed = [
        comparison.timing_deltas.prefill_duration_ms.direction,
        comparison.timing_deltas.time_to_first_token_ms.direction,
        comparison.timing_deltas.generation_duration_ms.direction,
        comparison.timing_deltas.wall_duration_ms.direction,
    ]
    .iter()
    .any(|direction| {
        matches!(
            direction,
            MetricDirection::Increased | MetricDirection::Decreased
        )
    });
    let prefill_changed = matches!(
        comparison.token_deltas.fresh_prefill_tokens.direction,
        MetricDirection::Increased | MetricDirection::Decreased
    );
    if timing_changed || prefill_changed {
        CacheRegressionAssessment::MixedObservations
    } else if reuse == MetricDirection::Increased {
        CacheRegressionAssessment::ObservedReuseIncrease
    } else {
        CacheRegressionAssessment::ObservedReuseDecrease
    }
}

fn evidence_statement(
    request_diff: &RequestDiff,
    comparison: &ObservationComparison,
    assessment: CacheRegressionAssessment,
    alignment: RequestObservationAlignment,
) -> EvidenceStatement {
    let structural_change_categories = structural_categories(request_diff);
    let observed_metric_directions = metric_directions(comparison);
    let has_structural_change = !structural_change_categories.is_empty();
    let has_observed_change = observed_metric_directions.iter().any(|(_, direction)| {
        matches!(
            direction,
            MetricDirection::Increased | MetricDirection::Decreased
        )
    });
    let has_reuse_signal = matches!(
        comparison.token_deltas.provider_cached_tokens.direction,
        MetricDirection::Increased | MetricDirection::Decreased
    );
    let association = if alignment != RequestObservationAlignment::Aligned && has_structural_change
    {
        EvidenceAssociation::StructuralDifferenceWithoutComparableObservation
    } else if !has_structural_change {
        EvidenceAssociation::NoStructuralDifference
    } else if !has_observed_change {
        EvidenceAssociation::StructuralDifferenceWithoutComparableObservation
    } else if has_reuse_signal {
        EvidenceAssociation::StructuralDifferenceWithObservedReuseSignal
    } else {
        EvidenceAssociation::StructuralDifferenceWithObservedMetricChange
    };
    let structural = if structural_change_categories.is_empty() {
        "none".to_string()
    } else {
        structural_change_categories.join(",")
    };
    let directions = if observed_metric_directions.is_empty() {
        "none".to_string()
    } else {
        observed_metric_directions
            .iter()
            .map(|(metric, direction)| {
                format!("{}={}", metric_name(*metric), direction_name(*direction))
            })
            .collect::<Vec<_>>()
            .join(",")
    };
    let statement = format!(
        "structural_changes={structural};comparability={};assessment={};observed_directions={directions};alignment={};association={};causality=not_established",
        comparability_name(comparison.comparability.level),
        assessment_name(assessment),
        alignment_name(alignment),
        association_name(association),
    );
    EvidenceStatement {
        structural_change_categories,
        observed_metric_directions,
        association,
        causality: CausalityStatus::NotEstablished,
        statement,
    }
}

fn request_observation_alignment(
    request_diff: &RequestDiff,
    comparison: &ObservationComparison,
) -> RequestObservationAlignment {
    match (
        comparison.left.request_fingerprint.as_deref(),
        comparison.right.request_fingerprint.as_deref(),
    ) {
        (Some(left), Some(right))
            if left == request_diff.left_request_fingerprint
                && right == request_diff.right_request_fingerprint =>
        {
            RequestObservationAlignment::Aligned
        }
        (Some(_), Some(_)) => RequestObservationAlignment::Mismatched,
        _ => RequestObservationAlignment::Unavailable,
    }
}

fn structural_categories(request_diff: &RequestDiff) -> Vec<String> {
    let mut categories = BTreeSet::new();
    for change in &request_diff.prefix_diff.changes {
        categories.insert(change_category_name(change.category));
    }
    for change in &request_diff.envelope_diff.changes {
        categories.insert(format!("envelope_{}", envelope_field_name(change.field)));
    }
    categories.into_iter().collect()
}

fn metric_directions(
    comparison: &ObservationComparison,
) -> Vec<(DiagnosticMetric, MetricDirection)> {
    let values = [
        (
            DiagnosticMetric::TransmittedInputTokens,
            comparison.token_deltas.transmitted_input_tokens.direction,
        ),
        (
            DiagnosticMetric::ProviderCachedTokens,
            comparison.token_deltas.provider_cached_tokens.direction,
        ),
        (
            DiagnosticMetric::FreshPrefillTokens,
            comparison.token_deltas.fresh_prefill_tokens.direction,
        ),
        (
            DiagnosticMetric::ReconstructedContextTokens,
            comparison
                .token_deltas
                .reconstructed_context_tokens
                .direction,
        ),
        (
            DiagnosticMetric::OutputTokens,
            comparison.token_deltas.output_tokens.direction,
        ),
        (
            DiagnosticMetric::PrefillDurationMs,
            comparison.timing_deltas.prefill_duration_ms.direction,
        ),
        (
            DiagnosticMetric::TimeToFirstTokenMs,
            comparison.timing_deltas.time_to_first_token_ms.direction,
        ),
        (
            DiagnosticMetric::GenerationDurationMs,
            comparison.timing_deltas.generation_duration_ms.direction,
        ),
        (
            DiagnosticMetric::WallDurationMs,
            comparison.timing_deltas.wall_duration_ms.direction,
        ),
        (
            DiagnosticMetric::RamBytes,
            comparison.resource_deltas.ram_bytes.direction,
        ),
        (
            DiagnosticMetric::VramBytes,
            comparison.resource_deltas.vram_bytes.direction,
        ),
        (
            DiagnosticMetric::KvCacheBytes,
            comparison.resource_deltas.kv_cache_bytes.direction,
        ),
    ];
    values
        .into_iter()
        .filter(|(_, direction)| *direction != MetricDirection::Unavailable)
        .take(MAX_SIGNAL_DIRECTIONS)
        .collect()
}

fn change_category_name(category: ChangeCategory) -> String {
    match category {
        ChangeCategory::TextContentChanged => "text_content_changed",
        ChangeCategory::ContentAdded => "content_added",
        ChangeCategory::ContentRemoved => "content_removed",
        ChangeCategory::ArtifactAdded => "artifact_added",
        ChangeCategory::ArtifactRemoved => "artifact_removed",
        ChangeCategory::ArtifactOrderChanged => "artifact_order_changed",
        ChangeCategory::ArtifactContentChanged => "artifact_content_changed",
        ChangeCategory::ToolAdded => "tool_added",
        ChangeCategory::ToolRemoved => "tool_removed",
        ChangeCategory::ToolOrderChanged => "tool_order_changed",
        ChangeCategory::ToolDefinitionChanged => "tool_definition_changed",
        ChangeCategory::OptionalSchemaFieldAdded => "optional_schema_field_added",
        ChangeCategory::OptionalSchemaFieldRemoved => "optional_schema_field_removed",
        ChangeCategory::OrderedSchemaFieldChanged => "ordered_schema_field_changed",
        ChangeCategory::JsonStructureChanged => "json_structure_changed",
        ChangeCategory::ValueChanged => "value_changed",
        ChangeCategory::PresenceChanged => "presence_changed",
    }
    .to_string()
}

fn envelope_field_name(field: crate::diff::EnvelopeField) -> &'static str {
    match field {
        crate::diff::EnvelopeField::Model => "model",
        crate::diff::EnvelopeField::Reasoning => "reasoning",
        crate::diff::EnvelopeField::ResponseFormat => "response_format",
    }
}

fn metric_name(metric: DiagnosticMetric) -> &'static str {
    match metric {
        DiagnosticMetric::TransmittedInputTokens => "transmitted_input_tokens",
        DiagnosticMetric::ProviderCachedTokens => "provider_cached_tokens",
        DiagnosticMetric::FreshPrefillTokens => "fresh_prefill_tokens",
        DiagnosticMetric::ReconstructedContextTokens => "reconstructed_context_tokens",
        DiagnosticMetric::OutputTokens => "output_tokens",
        DiagnosticMetric::PrefillDurationMs => "prefill_duration_ms",
        DiagnosticMetric::TimeToFirstTokenMs => "time_to_first_token_ms",
        DiagnosticMetric::GenerationDurationMs => "generation_duration_ms",
        DiagnosticMetric::WallDurationMs => "wall_duration_ms",
        DiagnosticMetric::RamBytes => "ram_bytes",
        DiagnosticMetric::VramBytes => "vram_bytes",
        DiagnosticMetric::KvCacheBytes => "kv_cache_bytes",
    }
}

fn direction_name(direction: MetricDirection) -> &'static str {
    match direction {
        MetricDirection::Increased => "increased",
        MetricDirection::Decreased => "decreased",
        MetricDirection::Unchanged => "unchanged",
        MetricDirection::Unavailable => "unavailable",
    }
}

fn comparability_name(level: ComparabilityLevel) -> &'static str {
    match level {
        ComparabilityLevel::DirectlyComparable => "directly_comparable",
        ComparabilityLevel::PartiallyComparable => "partially_comparable",
        ComparabilityLevel::Incomparable => "incomparable",
        ComparabilityLevel::InsufficientEvidence => "insufficient_evidence",
    }
}

fn assessment_name(assessment: CacheRegressionAssessment) -> &'static str {
    match assessment {
        CacheRegressionAssessment::InsufficientEvidence => "insufficient_evidence",
        CacheRegressionAssessment::NoObservedCacheReuseChange => "no_observed_cache_reuse_change",
        CacheRegressionAssessment::ObservedReuseIncrease => "observed_reuse_increase",
        CacheRegressionAssessment::ObservedReuseDecrease => "observed_reuse_decrease",
        CacheRegressionAssessment::MixedObservations => "mixed_observations",
        CacheRegressionAssessment::Incomparable => "incomparable",
    }
}

fn association_name(association: EvidenceAssociation) -> &'static str {
    match association {
        EvidenceAssociation::NoStructuralDifference => "no_structural_difference",
        EvidenceAssociation::StructuralDifferenceWithoutComparableObservation => {
            "structural_difference_without_comparable_observation"
        }
        EvidenceAssociation::StructuralDifferenceWithObservedMetricChange => {
            "structural_difference_with_observed_metric_change"
        }
        EvidenceAssociation::StructuralDifferenceWithObservedReuseSignal => {
            "structural_difference_with_observed_reuse_signal"
        }
    }
}

fn alignment_name(alignment: RequestObservationAlignment) -> &'static str {
    match alignment {
        RequestObservationAlignment::Aligned => "aligned",
        RequestObservationAlignment::Mismatched => "mismatched",
        RequestObservationAlignment::Unavailable => "unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        request_diff, CaseRelationship, ConformanceCaseResult, ConformanceRequest, MutationClass,
        RequestContext, RequestEnvelope,
    };
    use prefixity_core::observation::{
        CacheBehavior, ContextIdentity, ObservationOutcome, ResourceUsage, RuntimeIdentity,
        TimingObservation, TokenAccounting, CACHE_OBSERVATION_SCHEMA_VERSION,
    };

    fn scoped(count: u64) -> TokenCount {
        TokenCount {
            count,
            provider: Observed::Known("fixture-provider".to_string()),
            model: Observed::Known("fixture-model".to_string()),
            tokenizer: Observed::Known("fixture-tokenizer".to_string()),
        }
    }

    fn observation(id: &str) -> CacheObservation {
        CacheObservation {
            schema_version: CACHE_OBSERVATION_SCHEMA_VERSION,
            observation_id: id.to_string(),
            observed_at: "2026-01-01T00:00:00Z".to_string(),
            runtime: RuntimeIdentity {
                backend: "fixture-backend".to_string(),
                provider: Observed::Known("fixture-provider".to_string()),
                model: Observed::Known("fixture-model".to_string()),
                protocol: Observed::Known("fixture-protocol".to_string()),
                runtime: Observed::Known("fixture-runtime".to_string()),
                runtime_version: Observed::Known("1".to_string()),
                ..RuntimeIdentity::default()
            },
            context: ContextIdentity {
                serialized_request_identity: Observed::Known("request".to_string()),
                reusable_prefix_identity: Observed::Known("context".to_string()),
                ..ContextIdentity::default()
            },
            accounting: TokenAccounting::default(),
            timing: TimingObservation::default(),
            resources: ResourceUsage::default(),
            cache: CacheBehavior::default(),
            outcome: ObservationOutcome::default(),
            raw_telemetry: Default::default(),
        }
    }

    fn direct_pair() -> (CacheObservation, CacheObservation) {
        let mut left = observation("left");
        let mut right = observation("right");
        left.accounting.transmitted_input_tokens = Observed::Known(scoped(100));
        right.accounting.transmitted_input_tokens = Observed::Known(scoped(100));
        left.accounting.provider_cached_tokens = Observed::Known(scoped(20));
        right.accounting.provider_cached_tokens = Observed::Known(scoped(60));
        left.accounting.fresh_prefill_tokens = Observed::Known(scoped(80));
        right.accounting.fresh_prefill_tokens = Observed::Known(scoped(40));
        (left, right)
    }

    fn request_pair(envelope_change: bool) -> (ConformanceRequest, ConformanceRequest) {
        let left = ConformanceRequest {
            context: RequestContext {
                system_instruction: "system".to_string(),
                artifacts: Vec::new(),
                user_content: "user".to_string(),
                tools: Vec::new(),
            },
            envelope: RequestEnvelope {
                model: "model-a".to_string(),
                reasoning: None,
                response_format: None,
            },
        };
        let mut right = left.clone();
        if envelope_change {
            right.envelope.model = "model-b".to_string();
        } else {
            right.context.user_content.push_str(" changed");
        }
        (left, right)
    }

    #[test]
    fn exact_repeat_reuse_increase_is_directional() {
        let (left, right) = direct_pair();
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.comparability.level,
            ComparabilityLevel::PartiallyComparable
        );
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.delta,
            Some(40)
        );
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.direction,
            MetricDirection::Increased
        );
    }

    #[test]
    fn late_partial_reuse_is_not_flattened_into_one_metric() {
        let (mut left, mut right) = direct_pair();
        left.accounting.provider_cached_tokens = Observed::Known(scoped(0));
        right.accounting.provider_cached_tokens = Observed::Known(scoped(25));
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.delta,
            Some(25)
        );
        assert_eq!(
            comparison.token_deltas.fresh_prefill_tokens.delta,
            Some(-40)
        );
    }

    #[test]
    fn apparent_decrease_remains_a_directional_observation() {
        let (mut left, mut right) = direct_pair();
        left.accounting.provider_cached_tokens = Observed::Known(scoped(70));
        right.accounting.provider_cached_tokens = Observed::Known(scoped(30));
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.direction,
            MetricDirection::Decreased
        );
    }

    #[test]
    fn fresh_prefill_increase_is_independent_from_reuse() {
        let (mut left, mut right) = direct_pair();
        left.accounting.fresh_prefill_tokens = Observed::Known(scoped(20));
        right.accounting.fresh_prefill_tokens = Observed::Known(scoped(70));
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.token_deltas.fresh_prefill_tokens.direction,
            MetricDirection::Increased
        );
    }

    #[test]
    fn timing_only_has_no_cache_reuse_assessment() {
        let mut left = observation("left");
        let mut right = observation("right");
        left.timing.wall_duration_ms = Observed::Known(10);
        right.timing.wall_duration_ms = Observed::Known(20);
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            assess_cache(&comparison, RequestObservationAlignment::Aligned),
            CacheRegressionAssessment::InsufficientEvidence
        );
        assert_eq!(
            comparison.timing_deltas.wall_duration_ms.relative_change,
            Some(1.0)
        );
    }

    #[test]
    fn missing_telemetry_stays_unavailable() {
        let (left, right) = direct_pair();
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.timing_deltas.wall_duration_ms.direction,
            MetricDirection::Unavailable
        );
        assert_eq!(
            comparison.token_deltas.output_tokens.left,
            Observed::NotObserved
        );
    }

    #[test]
    fn explicit_zero_is_not_missing() {
        let (mut left, mut right) = direct_pair();
        left.accounting.provider_cached_tokens = Observed::Known(scoped(0));
        right.accounting.provider_cached_tokens = Observed::Known(scoped(0));
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.delta,
            Some(0)
        );
        assert_eq!(
            comparison.token_deltas.provider_cached_tokens.direction,
            MetricDirection::Unchanged
        );
    }

    #[test]
    fn incompatible_model_is_incomparable() {
        let (left, mut right) = direct_pair();
        right.runtime.model = Observed::Known("other-model".to_string());
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            comparison.comparability.level,
            ComparabilityLevel::Incomparable
        );
        assert_eq!(
            assess_cache(&comparison, RequestObservationAlignment::Aligned),
            CacheRegressionAssessment::Incomparable
        );
    }

    #[test]
    fn mixed_signal_is_not_called_better_or_worse() {
        let (mut left, mut right) = direct_pair();
        left.timing.wall_duration_ms = Observed::Known(10);
        right.timing.wall_duration_ms = Observed::Known(20);
        let comparison = compare_observations(&left, &right);
        assert_eq!(
            assess_cache(&comparison, RequestObservationAlignment::Aligned),
            CacheRegressionAssessment::MixedObservations
        );
    }

    #[test]
    fn envelope_only_diff_is_kept_separate_from_context_diff() {
        let (left_request, right_request) = request_pair(true);
        let diff = request_diff(&left_request, &right_request).expect("request diff");
        assert!(diff.prefix_diff.identical);
        assert!(!diff.envelope_diff.identical);
    }

    #[test]
    fn diagnostic_statement_is_deterministic_and_non_causal() {
        let (left_request, right_request) = request_pair(false);
        let diff = request_diff(&left_request, &right_request).expect("request diff");
        let (left, right) = direct_pair();
        let first = diagnose_cache(&diff, &left, &right);
        let second = diagnose_cache(&diff, &left, &right);
        assert_eq!(first, second);
        assert_eq!(first.evidence.causality, CausalityStatus::NotEstablished);
        assert!(first
            .evidence
            .statement
            .contains("causality=not_established"));
    }

    #[test]
    fn conformance_reference_preserves_explicit_evidence_source() {
        let result = ConformanceCaseResult {
            experiment_id: "experiment".to_string(),
            case_id: "case".to_string(),
            mutation: MutationClass::Baseline,
            relationship: CaseRelationship::Baseline,
            request_fingerprint: "request".to_string(),
            context_fingerprint: "context".to_string(),
            observation: observation("observation"),
        };
        assert_eq!(
            ObservationReference::from_conformance_case(&result, None).source,
            EvidenceSourceClass::SyntheticProtocolTest
        );
        assert_eq!(
            ObservationReference::from_conformance_case_with_source(
                &result,
                None,
                EvidenceSourceClass::ExperimentallyObservedRuntime,
            )
            .source,
            EvidenceSourceClass::ExperimentallyObservedRuntime
        );
    }

    #[test]
    fn derived_ratio_has_explicit_denominator_and_zero_is_safe() {
        let (mut left, mut right) = direct_pair();
        left.accounting.transmitted_input_tokens = Observed::Known(scoped(0));
        right.accounting.transmitted_input_tokens = Observed::Known(scoped(100));
        let comparison = compare_observations(&left, &right);
        assert!(comparison.derived_metrics.reuse_ratio.derived);
        assert_eq!(
            comparison.derived_metrics.reuse_ratio.denominator,
            TokenMetricName::TransmittedInputTokens
        );
        assert_eq!(comparison.derived_metrics.reuse_ratio.left_value, None);
    }
}
