//! Deterministic, provider-neutral context stability and volatility analysis.
//!
//! P0-L10 consumes the existing P0-L2 ContextArtifact metadata and the
//! P0-L4 neutral request. It classifies the current structure, records
//! boundaries and stability inversions, and deliberately produces no rewrite,
//! cache prediction, score, or optimization action.

use crate::conformance::{ConformanceRequest, ToolDefinition};
use crate::error::BenchmarkError;
use crate::hashing::hash_text;
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ContextArtifact, Observed, TokenCount,
    TrustLevel,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const CONTEXT_STABILITY_SCHEMA_ID: &str = "prefixity.context-stability-analysis";
pub const CONTEXT_STABILITY_SCHEMA_VERSION: u32 = 1;
pub const MAX_STABILITY_SEGMENTS: usize = 256;
pub const MAX_STABILITY_BOUNDARIES: usize = 255;
pub const MAX_STABILITY_FINDINGS: usize = 256;
pub const MAX_STABILITY_PROVENANCE: usize = 16;
pub const MAX_STABILITY_TEXT_BYTES: usize = 512;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextRole {
    SystemInstruction,
    ContextArtifact,
    CurrentUserTask,
    ToolDefinition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClassificationSource {
    ExplicitMetadata,
    StructuralRole,
    DerivedRule,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralRoleDefault {
    pub stability: ArtifactStability,
    pub lifecycle: ArtifactLifecycle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructuralRoleDefaults {
    pub system_instruction: StructuralRoleDefault,
    pub current_user_task: StructuralRoleDefault,
    pub tool_definition: StructuralRoleDefault,
}

impl Default for StructuralRoleDefaults {
    fn default() -> Self {
        Self {
            system_instruction: StructuralRoleDefault {
                stability: ArtifactStability::Stable,
                lifecycle: ArtifactLifecycle::PersistentVersioned,
            },
            current_user_task: StructuralRoleDefault {
                stability: ArtifactStability::Volatile,
                lifecycle: ArtifactLifecycle::Transient,
            },
            tool_definition: StructuralRoleDefault {
                stability: ArtifactStability::Unknown,
                lifecycle: ArtifactLifecycle::Unknown,
            },
        }
    }
}

/// Optional P0-L2 metadata for request components. Request artifacts are
/// keyed by their P0-L4 artifact IDs; tool metadata is keyed by tool name.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ContextStabilityInputs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_instruction: Option<ContextArtifact>,
    #[serde(default)]
    pub artifacts: BTreeMap<String, ContextArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_user_task: Option<ContextArtifact>,
    #[serde(default)]
    pub tools: BTreeMap<String, ContextArtifact>,
    #[serde(default)]
    pub defaults: StructuralRoleDefaults,
    #[serde(default)]
    pub provenance: BTreeMap<String, String>,
}

impl ContextStabilityInputs {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.provenance.len() > MAX_STABILITY_PROVENANCE {
            return Err(BenchmarkError::validation(
                "context stability provenance exceeds its bound",
            ));
        }
        for (key, value) in &self.provenance {
            validate_text(key, "context stability provenance key")?;
            validate_text(value, "context stability provenance value")?;
        }
        let mut ids = BTreeMap::new();
        if let Some(artifact) = &self.system_instruction {
            validate_metadata_artifact(artifact, "system_instruction")?;
            ids.insert(artifact.artifact_id.clone(), "system_instruction");
        }
        for (key, artifact) in &self.artifacts {
            validate_metadata_artifact(artifact, &format!("artifacts.{key}"))?;
            if let Some(previous) = ids.insert(artifact.artifact_id.clone(), key.as_str()) {
                return Err(BenchmarkError::validation(format!(
                    "context stability metadata reuses artifact_id {} at {previous} and {key}",
                    artifact.artifact_id
                )));
            }
        }
        if let Some(artifact) = &self.current_user_task {
            validate_metadata_artifact(artifact, "current_user_task")?;
            if let Some(previous) = ids.insert(artifact.artifact_id.clone(), "current_user_task") {
                return Err(BenchmarkError::validation(format!(
                    "context stability metadata reuses artifact_id {} at {previous} and current_user_task",
                    artifact.artifact_id
                )));
            }
        }
        for (key, artifact) in &self.tools {
            validate_metadata_artifact(artifact, &format!("tools.{key}"))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextSegmentAnalysis {
    pub position: usize,
    pub structural_path: String,
    pub component_id: Option<String>,
    pub role: ContextRole,
    pub stability: ArtifactStability,
    pub lifecycle: ArtifactLifecycle,
    pub classification_source: ClassificationSource,
    pub trust: Observed<TrustLevel>,
    pub artifact_id: Option<String>,
    pub content_fingerprint: String,
    pub sizes: ArtifactSizes,
    pub size_source: SizeSource,
    pub token_size: Observed<TokenCount>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeSource {
    ExplicitMetadata,
    StructuralRequestBytes,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryDirection {
    TowardMoreStable,
    TowardMoreVolatile,
    NoKnownMovement,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryClassification {
    Classified,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilityBoundary {
    pub position: usize,
    pub left_segment: usize,
    pub right_segment: usize,
    pub left_stability: ArtifactStability,
    pub right_stability: ArtifactStability,
    pub left_lifecycle: ArtifactLifecycle,
    pub right_lifecycle: ArtifactLifecycle,
    pub direction: BoundaryDirection,
    pub classification: BoundaryClassification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StabilityFindingKind {
    StabilityInversion,
    UnknownStabilitySegment,
    VolatileBeforeStable,
    AppendOnlyRegion,
    TransientStableSegment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilityFinding {
    pub kind: StabilityFindingKind,
    pub segment: Option<usize>,
    pub boundary: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeadingRegionLimit {
    Complete,
    Empty,
    LimitedByUnknown,
    LimitedByStabilityInversion,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilityAlignedLeadingRegion {
    pub segment_count: usize,
    pub known_byte_size: Observed<u64>,
    pub limit: LeadingRegionLimit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StabilitySummary {
    pub immutable_segments: usize,
    pub stable_segments: usize,
    pub append_only_segments: usize,
    pub volatile_segments: usize,
    pub unknown_segments: usize,
    pub known_stable_bytes: u64,
    pub known_append_only_bytes: u64,
    pub known_volatile_bytes: u64,
    pub known_immutable_bytes: u64,
    pub known_bytes_total: u64,
    pub unknown_bytes: Observed<u64>,
    pub unknown_size_segments: usize,
    pub token_units: Observed<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextStabilityAnalysis {
    pub schema_id: String,
    pub schema_version: u32,
    pub request_fingerprint: String,
    pub context_fingerprint: String,
    pub segments: Vec<ContextSegmentAnalysis>,
    pub boundaries: Vec<StabilityBoundary>,
    pub leading_region: StabilityAlignedLeadingRegion,
    pub summary: StabilitySummary,
    pub findings: Vec<StabilityFinding>,
    pub provenance: BTreeMap<String, String>,
}

impl ContextStabilityAnalysis {
    pub fn validate(&self) -> Result<(), BenchmarkError> {
        if self.schema_id != CONTEXT_STABILITY_SCHEMA_ID
            || self.schema_version != CONTEXT_STABILITY_SCHEMA_VERSION
        {
            return Err(BenchmarkError::validation(
                "unsupported context stability analysis schema",
            ));
        }
        if self.segments.is_empty() || self.segments.len() > MAX_STABILITY_SEGMENTS {
            return Err(BenchmarkError::validation(
                "context stability analysis has an invalid segment count",
            ));
        }
        if self.boundaries.len() > MAX_STABILITY_BOUNDARIES
            || self.findings.len() > MAX_STABILITY_FINDINGS
        {
            return Err(BenchmarkError::validation(
                "context stability analysis exceeds a diagnostic bound",
            ));
        }
        if self.provenance.len() > MAX_STABILITY_PROVENANCE {
            return Err(BenchmarkError::validation(
                "context stability analysis provenance exceeds its bound",
            ));
        }
        for (key, value) in &self.provenance {
            validate_text(key, "analysis provenance key")?;
            validate_text(value, "analysis provenance value")?;
        }
        for (position, segment) in self.segments.iter().enumerate() {
            if segment.position != position {
                return Err(BenchmarkError::validation(
                    "context stability segment positions must be contiguous",
                ));
            }
            if segment.content_fingerprint.is_empty() {
                return Err(BenchmarkError::validation(
                    "context stability segment fingerprint must not be empty",
                ));
            }
        }
        for (position, boundary) in self.boundaries.iter().enumerate() {
            if boundary.position != position
                || boundary.left_segment != position
                || boundary.right_segment != position + 1
            {
                return Err(BenchmarkError::validation(
                    "context stability boundaries must be adjacent and ordered",
                ));
            }
        }
        Ok(())
    }
}

pub fn analyze_context_stability(
    request: &ConformanceRequest,
    inputs: &ContextStabilityInputs,
) -> Result<ContextStabilityAnalysis, BenchmarkError> {
    request.validate()?;
    inputs.validate()?;
    let request_fingerprint = request.request_fingerprint()?;
    let context_fingerprint = request.context_fingerprint()?;
    let mut segments = Vec::new();

    push_segment(
        &mut segments,
        "context.system_instruction",
        None,
        ContextRole::SystemInstruction,
        &request.context.system_instruction,
        inputs.system_instruction.as_ref(),
        &inputs.defaults.system_instruction,
    );
    for artifact in &request.context.artifacts {
        push_segment(
            &mut segments,
            &format!("context.artifacts[{}]", artifact.artifact_id),
            Some(artifact.artifact_id.clone()),
            ContextRole::ContextArtifact,
            &artifact.content,
            inputs.artifacts.get(&artifact.artifact_id),
            &StructuralRoleDefault {
                stability: ArtifactStability::Unknown,
                lifecycle: ArtifactLifecycle::Unknown,
            },
        );
    }
    push_segment(
        &mut segments,
        "context.current_user",
        None,
        ContextRole::CurrentUserTask,
        &request.context.user_content,
        inputs.current_user_task.as_ref(),
        &inputs.defaults.current_user_task,
    );
    for tool in &request.context.tools {
        let content = serde_json::to_vec(tool).expect("validated tool serializes");
        push_tool_segment(
            &mut segments,
            tool,
            &content,
            inputs.tools.get(&tool.name),
            &inputs.defaults.tool_definition,
        );
    }

    let boundaries = build_boundaries(&segments);
    let leading_region = leading_region(&segments, &boundaries);
    let summary = summarize(&segments);
    let findings = findings(&segments, &boundaries);
    let analysis = ContextStabilityAnalysis {
        schema_id: CONTEXT_STABILITY_SCHEMA_ID.to_string(),
        schema_version: CONTEXT_STABILITY_SCHEMA_VERSION,
        request_fingerprint,
        context_fingerprint,
        segments,
        boundaries,
        leading_region,
        summary,
        findings,
        provenance: if inputs.provenance.is_empty() {
            BTreeMap::from([(
                "source".to_string(),
                "neutral-request-and-context-artifact-metadata".to_string(),
            )])
        } else {
            inputs.provenance.clone()
        },
    };
    analysis.validate()?;
    Ok(analysis)
}

pub fn analyze_request_stability(
    request: &ConformanceRequest,
) -> Result<ContextStabilityAnalysis, BenchmarkError> {
    analyze_context_stability(request, &ContextStabilityInputs::default())
}

fn push_segment(
    segments: &mut Vec<ContextSegmentAnalysis>,
    path: &str,
    component_id: Option<String>,
    role: ContextRole,
    content: &str,
    metadata: Option<&ContextArtifact>,
    role_default: &StructuralRoleDefault,
) {
    let fingerprint = metadata
        .and_then(|value| known_string(&value.content_hash))
        .unwrap_or_else(|| hash_text(content));
    let (stability, lifecycle, source, trust, sizes, size_source, token_size, artifact_id) =
        component_values(metadata, role_default, content.len() as u64);
    segments.push(ContextSegmentAnalysis {
        position: segments.len(),
        structural_path: path.to_string(),
        component_id,
        role,
        stability,
        lifecycle,
        classification_source: source,
        trust,
        artifact_id,
        content_fingerprint: fingerprint,
        sizes,
        size_source,
        token_size,
    });
}

fn push_tool_segment(
    segments: &mut Vec<ContextSegmentAnalysis>,
    tool: &ToolDefinition,
    content: &[u8],
    metadata: Option<&ContextArtifact>,
    role_default: &StructuralRoleDefault,
) {
    let fingerprint = metadata
        .and_then(|value| known_string(&value.content_hash))
        .unwrap_or_else(|| hash_text(&String::from_utf8_lossy(content)));
    let (stability, lifecycle, source, trust, sizes, size_source, token_size, artifact_id) =
        component_values(metadata, role_default, content.len() as u64);
    segments.push(ContextSegmentAnalysis {
        position: segments.len(),
        structural_path: format!("context.tools[{}]", tool.name),
        component_id: Some(tool.name.clone()),
        role: ContextRole::ToolDefinition,
        stability,
        lifecycle,
        classification_source: source,
        trust,
        artifact_id,
        content_fingerprint: fingerprint,
        sizes,
        size_source,
        token_size,
    });
}

#[allow(clippy::type_complexity)]
fn component_values(
    metadata: Option<&ContextArtifact>,
    role_default: &StructuralRoleDefault,
    structural_bytes: u64,
) -> (
    ArtifactStability,
    ArtifactLifecycle,
    ClassificationSource,
    Observed<TrustLevel>,
    ArtifactSizes,
    SizeSource,
    Observed<TokenCount>,
    Option<String>,
) {
    if let Some(metadata) = metadata {
        return (
            metadata.stability.clone(),
            metadata.lifecycle.clone(),
            ClassificationSource::ExplicitMetadata,
            metadata.trust.clone(),
            metadata.sizes.clone(),
            SizeSource::ExplicitMetadata,
            metadata.sizes.token_size.clone(),
            Some(metadata.artifact_id.clone()),
        );
    }
    let source = if role_default.stability == ArtifactStability::Unknown
        && role_default.lifecycle == ArtifactLifecycle::Unknown
    {
        ClassificationSource::Unknown
    } else {
        ClassificationSource::StructuralRole
    };
    (
        role_default.stability.clone(),
        role_default.lifecycle.clone(),
        source,
        Observed::Unknown,
        ArtifactSizes {
            byte_size: Observed::Known(structural_bytes),
            ..ArtifactSizes::default()
        },
        SizeSource::StructuralRequestBytes,
        Observed::NotObserved,
        None,
    )
}

fn build_boundaries(segments: &[ContextSegmentAnalysis]) -> Vec<StabilityBoundary> {
    segments
        .windows(2)
        .enumerate()
        .map(|(position, pair)| {
            let left = &pair[0];
            let right = &pair[1];
            let direction = stability_direction(&left.stability, &right.stability);
            StabilityBoundary {
                position,
                left_segment: position,
                right_segment: position + 1,
                left_stability: left.stability.clone(),
                right_stability: right.stability.clone(),
                left_lifecycle: left.lifecycle.clone(),
                right_lifecycle: right.lifecycle.clone(),
                classification: if direction == BoundaryDirection::Unknown {
                    BoundaryClassification::Unknown
                } else {
                    BoundaryClassification::Classified
                },
                direction,
            }
        })
        .collect()
}

fn stability_direction(left: &ArtifactStability, right: &ArtifactStability) -> BoundaryDirection {
    let (Some(left_rank), Some(right_rank)) = (stability_rank(left), stability_rank(right)) else {
        return BoundaryDirection::Unknown;
    };
    match right_rank.cmp(&left_rank) {
        std::cmp::Ordering::Less => BoundaryDirection::TowardMoreStable,
        std::cmp::Ordering::Greater => BoundaryDirection::TowardMoreVolatile,
        std::cmp::Ordering::Equal => BoundaryDirection::NoKnownMovement,
    }
}

fn stability_rank(stability: &ArtifactStability) -> Option<u8> {
    match stability {
        ArtifactStability::Immutable => Some(0),
        ArtifactStability::Stable => Some(1),
        ArtifactStability::AppendOnly => Some(2),
        ArtifactStability::Volatile => Some(3),
        ArtifactStability::Unknown => None,
    }
}

fn leading_region(
    segments: &[ContextSegmentAnalysis],
    boundaries: &[StabilityBoundary],
) -> StabilityAlignedLeadingRegion {
    let mut count = 0;
    let mut known_bytes = 0u64;
    let mut unknown_size = false;
    let mut limit = LeadingRegionLimit::Complete;
    for (index, segment) in segments.iter().enumerate() {
        if segment.stability == ArtifactStability::Unknown {
            limit = if count == 0 {
                LeadingRegionLimit::Empty
            } else {
                LeadingRegionLimit::LimitedByUnknown
            };
            break;
        }
        count += 1;
        match segment.sizes.byte_size {
            Observed::Known(bytes) => known_bytes = known_bytes.saturating_add(bytes),
            Observed::Unknown | Observed::NotObserved => unknown_size = true,
        }
        if let Some(boundary) = boundaries.get(index) {
            if boundary.direction == BoundaryDirection::TowardMoreStable {
                limit = LeadingRegionLimit::LimitedByStabilityInversion;
                break;
            }
            if boundary.direction == BoundaryDirection::Unknown {
                limit = LeadingRegionLimit::LimitedByUnknown;
                break;
            }
        }
    }
    StabilityAlignedLeadingRegion {
        segment_count: count,
        known_byte_size: if unknown_size {
            Observed::Unknown
        } else {
            Observed::Known(known_bytes)
        },
        limit,
    }
}

fn summarize(segments: &[ContextSegmentAnalysis]) -> StabilitySummary {
    let mut summary = StabilitySummary {
        immutable_segments: 0,
        stable_segments: 0,
        append_only_segments: 0,
        volatile_segments: 0,
        unknown_segments: 0,
        known_stable_bytes: 0,
        known_append_only_bytes: 0,
        known_volatile_bytes: 0,
        known_immutable_bytes: 0,
        known_bytes_total: 0,
        unknown_bytes: Observed::Known(0),
        unknown_size_segments: 0,
        token_units: Observed::NotObserved,
    };
    for segment in segments {
        match segment.stability {
            ArtifactStability::Immutable => summary.immutable_segments += 1,
            ArtifactStability::Stable => summary.stable_segments += 1,
            ArtifactStability::AppendOnly => summary.append_only_segments += 1,
            ArtifactStability::Volatile => summary.volatile_segments += 1,
            ArtifactStability::Unknown => summary.unknown_segments += 1,
        }
        match segment.sizes.byte_size {
            Observed::Known(bytes) => {
                summary.known_bytes_total = summary.known_bytes_total.saturating_add(bytes);
                match segment.stability {
                    ArtifactStability::Immutable => {
                        summary.known_immutable_bytes =
                            summary.known_immutable_bytes.saturating_add(bytes)
                    }
                    ArtifactStability::Stable => {
                        summary.known_stable_bytes =
                            summary.known_stable_bytes.saturating_add(bytes)
                    }
                    ArtifactStability::AppendOnly => {
                        summary.known_append_only_bytes =
                            summary.known_append_only_bytes.saturating_add(bytes)
                    }
                    ArtifactStability::Volatile => {
                        summary.known_volatile_bytes =
                            summary.known_volatile_bytes.saturating_add(bytes)
                    }
                    ArtifactStability::Unknown => {}
                }
            }
            Observed::Unknown | Observed::NotObserved => {
                summary.unknown_size_segments += 1;
                summary.unknown_bytes = Observed::Unknown;
            }
        }
    }
    summary
}

fn findings(
    segments: &[ContextSegmentAnalysis],
    boundaries: &[StabilityBoundary],
) -> Vec<StabilityFinding> {
    let mut findings = Vec::new();
    for segment in segments {
        if segment.stability == ArtifactStability::Unknown {
            findings.push(StabilityFinding {
                kind: StabilityFindingKind::UnknownStabilitySegment,
                segment: Some(segment.position),
                boundary: None,
            });
        }
        if segment.stability == ArtifactStability::AppendOnly {
            findings.push(StabilityFinding {
                kind: StabilityFindingKind::AppendOnlyRegion,
                segment: Some(segment.position),
                boundary: None,
            });
        }
        if matches!(
            segment.stability,
            ArtifactStability::Immutable | ArtifactStability::Stable
        ) && segment.lifecycle == ArtifactLifecycle::Transient
        {
            findings.push(StabilityFinding {
                kind: StabilityFindingKind::TransientStableSegment,
                segment: Some(segment.position),
                boundary: None,
            });
        }
    }
    for boundary in boundaries {
        if boundary.direction == BoundaryDirection::TowardMoreStable {
            findings.push(StabilityFinding {
                kind: StabilityFindingKind::StabilityInversion,
                segment: None,
                boundary: Some(boundary.position),
            });
            if boundary.left_stability == ArtifactStability::Volatile
                && matches!(
                    boundary.right_stability,
                    ArtifactStability::Immutable | ArtifactStability::Stable
                )
            {
                findings.push(StabilityFinding {
                    kind: StabilityFindingKind::VolatileBeforeStable,
                    segment: None,
                    boundary: Some(boundary.position),
                });
            }
        }
    }
    findings.truncate(MAX_STABILITY_FINDINGS);
    findings
}

fn known_string(value: &Observed<String>) -> Option<String> {
    match value {
        Observed::Known(value) => Some(value.clone()),
        Observed::Unknown | Observed::NotObserved => None,
    }
}

fn validate_metadata_artifact(
    artifact: &ContextArtifact,
    field: &str,
) -> Result<(), BenchmarkError> {
    artifact
        .validate()
        .map_err(|error| BenchmarkError::validation(format!("{field}: {error}")))
}

fn validate_text(value: &str, field: &str) -> Result<(), BenchmarkError> {
    if value.trim().is_empty() || value.len() > MAX_STABILITY_TEXT_BYTES {
        return Err(BenchmarkError::validation(format!(
            "{field} must be non-empty and at most {MAX_STABILITY_TEXT_BYTES} bytes"
        )));
    }
    Ok(())
}
