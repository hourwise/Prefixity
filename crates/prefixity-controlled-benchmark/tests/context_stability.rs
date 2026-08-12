use prefixity_controlled_benchmark::{
    analyze_context_stability, analyze_request_stability, BoundaryClassification,
    BoundaryDirection, ClassificationSource, ConformanceRequest, ContextRole,
    ContextStabilityInputs, LeadingRegionLimit, OrderedJsonObject, RequestContext, RequestEnvelope,
    SizeSource, StabilityFindingKind, StructuralRoleDefault, StructuralRoleDefaults,
    ToolDefinition,
};
use prefixity_core::observation::{
    ArtifactLifecycle, ArtifactSizes, ArtifactStability, ArtifactType, ContextArtifact, Observed,
    TrustLevel, CONTEXT_ARTIFACT_SCHEMA_VERSION,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::Path;

fn request() -> ConformanceRequest {
    ConformanceRequest {
        context: RequestContext {
            system_instruction: "system".to_string(),
            artifacts: vec![
                prefixity_controlled_benchmark::ContextArtifactInput {
                    artifact_id: "source".to_string(),
                    content: "source bytes".to_string(),
                },
                prefixity_controlled_benchmark::ContextArtifactInput {
                    artifact_id: "history".to_string(),
                    content: "history bytes".to_string(),
                },
            ],
            user_content: "current task".to_string(),
            tools: vec![
                ToolDefinition {
                    name: "read_file".to_string(),
                    description: "read".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
                ToolDefinition {
                    name: "dynamic".to_string(),
                    description: "dynamic".to_string(),
                    parameters: OrderedJsonObject::new(vec![]),
                },
            ],
        },
        envelope: RequestEnvelope {
            model: "fixture-model".to_string(),
            reasoning: None,
            response_format: None,
        },
    }
}

fn metadata(
    id: &str,
    stability: ArtifactStability,
    lifecycle: ArtifactLifecycle,
    trust: Observed<TrustLevel>,
    bytes: Option<u64>,
) -> ContextArtifact {
    ContextArtifact {
        schema_version: CONTEXT_ARTIFACT_SCHEMA_VERSION,
        artifact_id: id.to_string(),
        origin_id: format!("origin-{id}"),
        content_source_id: Observed::Known(format!("source-{id}")),
        content_hash: Observed::Unknown,
        revision: Observed::Known("v1".to_string()),
        artifact_type: ArtifactType::Text,
        stability,
        lifecycle,
        sizes: ArtifactSizes {
            byte_size: bytes.map_or(Observed::Unknown, Observed::Known),
            logical_size: Observed::Unknown,
            token_size: Observed::NotObserved,
        },
        cache: Default::default(),
        trust,
        provenance: BTreeMap::new(),
        metadata: BTreeMap::new(),
    }
}

fn inputs_with_sequence(
    source_stability: ArtifactStability,
    history_stability: ArtifactStability,
    source_lifecycle: ArtifactLifecycle,
) -> ContextStabilityInputs {
    ContextStabilityInputs {
        artifacts: BTreeMap::from([
            (
                "source".to_string(),
                metadata(
                    "source",
                    source_stability,
                    source_lifecycle,
                    Observed::Known(TrustLevel::Trusted),
                    Some(100),
                ),
            ),
            (
                "history".to_string(),
                metadata(
                    "history",
                    history_stability,
                    ArtifactLifecycle::PersistentVersioned,
                    Observed::Known(TrustLevel::Untrusted),
                    Some(40),
                ),
            ),
        ]),
        tools: BTreeMap::from([(
            "read_file".to_string(),
            metadata(
                "tool-read",
                ArtifactStability::Stable,
                ArtifactLifecycle::PersistentVersioned,
                Observed::Known(TrustLevel::Trusted),
                Some(20),
            ),
        )]),
        ..ContextStabilityInputs::default()
    }
}

#[test]
fn explicit_metadata_and_lifecycle_are_preserved_independently() {
    let analysis = analyze_context_stability(
        &request(),
        &inputs_with_sequence(
            ArtifactStability::Stable,
            ArtifactStability::AppendOnly,
            ArtifactLifecycle::Transient,
        ),
    )
    .unwrap();
    let source = &analysis.segments[1];
    assert_eq!(source.role, ContextRole::ContextArtifact);
    assert_eq!(source.stability, ArtifactStability::Stable);
    assert_eq!(source.lifecycle, ArtifactLifecycle::Transient);
    assert_eq!(
        source.classification_source,
        ClassificationSource::ExplicitMetadata
    );
    assert_eq!(source.trust, Observed::Known(TrustLevel::Trusted));
    assert_eq!(source.size_source, SizeSource::ExplicitMetadata);
    assert_eq!(
        analysis.segments[2].stability,
        ArtifactStability::AppendOnly
    );
    assert_eq!(
        analysis.segments[2].classification_source,
        ClassificationSource::ExplicitMetadata
    );
}

#[test]
fn structural_defaults_are_explicit_and_overridable() {
    let defaulted = analyze_request_stability(&request()).unwrap();
    assert_eq!(defaulted.segments[0].stability, ArtifactStability::Stable);
    assert_eq!(
        defaulted.segments[0].classification_source,
        ClassificationSource::StructuralRole
    );
    assert_eq!(
        defaulted.segments[3].stability,
        ArtifactStability::Volatile,
        "the current-user default is a documented structural rule"
    );
    assert_eq!(
        defaulted.segments[4].classification_source,
        ClassificationSource::Unknown
    );

    let inputs = ContextStabilityInputs {
        defaults: StructuralRoleDefaults {
            system_instruction: StructuralRoleDefault {
                stability: ArtifactStability::Immutable,
                lifecycle: ArtifactLifecycle::PersistentVersioned,
            },
            current_user_task: StructuralRoleDefault {
                stability: ArtifactStability::Stable,
                lifecycle: ArtifactLifecycle::Transient,
            },
            tool_definition: StructuralRoleDefault {
                stability: ArtifactStability::AppendOnly,
                lifecycle: ArtifactLifecycle::PersistentVersioned,
            },
        },
        ..ContextStabilityInputs::default()
    };
    let overridden = analyze_context_stability(&request(), &inputs).unwrap();
    assert_eq!(
        overridden.segments[0].stability,
        ArtifactStability::Immutable
    );
    assert_eq!(overridden.segments[3].stability, ArtifactStability::Stable);
    assert_eq!(
        overridden.segments[4].stability,
        ArtifactStability::AppendOnly
    );
}

#[test]
fn monotonic_stability_has_no_inversion_and_inversion_is_bounded() {
    let mut monotonic_inputs = inputs_with_sequence(
        ArtifactStability::Stable,
        ArtifactStability::AppendOnly,
        ArtifactLifecycle::PersistentVersioned,
    );
    monotonic_inputs.tools.clear();
    let monotonic = analyze_context_stability(&request(), &monotonic_inputs).unwrap();
    assert!(!monotonic
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::StabilityInversion));

    let inverted = analyze_context_stability(
        &request(),
        &inputs_with_sequence(
            ArtifactStability::Volatile,
            ArtifactStability::Stable,
            ArtifactLifecycle::PersistentVersioned,
        ),
    )
    .unwrap();
    assert!(inverted
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::StabilityInversion));
    assert!(inverted
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::VolatileBeforeStable));
    assert_eq!(
        inverted.leading_region.limit,
        LeadingRegionLimit::LimitedByStabilityInversion
    );
    assert_eq!(
        inverted.boundaries[1].direction,
        BoundaryDirection::TowardMoreStable
    );
}

#[test]
fn unknown_middle_stays_unknown_and_limits_leading_region() {
    let mut inputs = inputs_with_sequence(
        ArtifactStability::Stable,
        ArtifactStability::Unknown,
        ArtifactLifecycle::PersistentVersioned,
    );
    inputs.tools.clear();
    let analysis = analyze_context_stability(&request(), &inputs).unwrap();
    assert_eq!(analysis.segments[2].stability, ArtifactStability::Unknown);
    assert!(analysis
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::UnknownStabilitySegment));
    assert_eq!(
        analysis.boundaries[1].classification,
        BoundaryClassification::Unknown
    );
    assert_eq!(
        analysis.leading_region.limit,
        LeadingRegionLimit::LimitedByUnknown
    );
    assert_eq!(analysis.leading_region.segment_count, 2);
}

#[test]
fn append_only_and_transient_stable_are_distinct_findings() {
    let mut inputs = inputs_with_sequence(
        ArtifactStability::AppendOnly,
        ArtifactStability::Stable,
        ArtifactLifecycle::Transient,
    );
    inputs.artifacts.get_mut("history").unwrap().stability = ArtifactStability::Stable;
    inputs.system_instruction = Some(metadata(
        "system",
        ArtifactStability::Stable,
        ArtifactLifecycle::Transient,
        Observed::Known(TrustLevel::Trusted),
        Some(10),
    ));
    let analysis = analyze_context_stability(&request(), &inputs).unwrap();
    assert!(analysis
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::AppendOnlyRegion));
    assert!(analysis
        .findings
        .iter()
        .any(|finding| finding.kind == StabilityFindingKind::TransientStableSegment));
    assert_ne!(
        analysis.segments[1].stability,
        ArtifactStability::Volatile,
        "transient does not imply volatile"
    );
}

#[test]
fn tool_order_is_preserved_and_unknown_dynamic_tool_material_is_visible() {
    let analysis = analyze_context_stability(
        &request(),
        &inputs_with_sequence(
            ArtifactStability::Stable,
            ArtifactStability::AppendOnly,
            ArtifactLifecycle::PersistentVersioned,
        ),
    )
    .unwrap();
    assert_eq!(
        analysis
            .segments
            .iter()
            .filter(|segment| segment.role == ContextRole::ToolDefinition)
            .map(|segment| segment.component_id.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("read_file"), Some("dynamic")]
    );
    assert_eq!(
        analysis.segments[4].stability,
        ArtifactStability::Stable,
        "explicit tool metadata is retained"
    );
    assert_eq!(analysis.segments[5].stability, ArtifactStability::Unknown);
    assert_eq!(
        analysis.segments[5].classification_source,
        ClassificationSource::Unknown
    );
}

#[test]
fn trust_is_separate_from_stability_and_unknown_sizes_are_not_zero() {
    let mut inputs = inputs_with_sequence(
        ArtifactStability::Stable,
        ArtifactStability::Stable,
        ArtifactLifecycle::PersistentVersioned,
    );
    inputs.artifacts.get_mut("history").unwrap().sizes.byte_size = Observed::Unknown;
    let analysis = analyze_context_stability(&request(), &inputs).unwrap();
    assert_eq!(
        analysis.segments[2].trust,
        Observed::Known(TrustLevel::Untrusted)
    );
    assert_eq!(analysis.segments[2].stability, ArtifactStability::Stable);
    assert_eq!(analysis.summary.unknown_bytes, Observed::Unknown);
    assert_eq!(analysis.summary.unknown_size_segments, 1);
    assert_eq!(analysis.leading_region.known_byte_size, Observed::Unknown);
    assert!(analysis.summary.known_stable_bytes > 0);
    assert_eq!(analysis.summary.token_units, Observed::NotObserved);
}

#[test]
fn deterministic_serialization_does_not_mutate_request_or_copy_content() {
    let original = request();
    let analysis =
        analyze_context_stability(&original, &ContextStabilityInputs::default()).unwrap();
    let repeated =
        analyze_context_stability(&original, &ContextStabilityInputs::default()).unwrap();
    assert_eq!(analysis, repeated);
    assert_eq!(
        serde_json::to_vec(&analysis).unwrap(),
        serde_json::to_vec(&repeated).unwrap()
    );
    assert_eq!(original, request());
    assert!(serde_json::to_string(&analysis)
        .unwrap()
        .contains("content_fingerprint"));
    assert!(!serde_json::to_string(&analysis)
        .unwrap()
        .contains("current task"));
    assert!(!serde_json::to_string(&analysis)
        .unwrap()
        .contains("\"action\""));
}

#[test]
fn existing_p0_l4_fixture_is_analyzable_offline() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("fixtures/conformance/coding-agent-cache-conformance-v1.json");
    let bytes = std::fs::read(path).unwrap();
    let experiment: prefixity_controlled_benchmark::ConformanceExperiment =
        serde_json::from_slice(&bytes).unwrap();
    let analysis = analyze_request_stability(&experiment.baseline_request).unwrap();
    analysis.validate().unwrap();
    assert_eq!(analysis.schema_version, 1);
}

#[test]
fn malformed_metadata_is_rejected_without_relaxing_p0_l2_validation() {
    let mut inputs = ContextStabilityInputs::default();
    let mut artifact = metadata(
        "source",
        ArtifactStability::Stable,
        ArtifactLifecycle::PersistentVersioned,
        Observed::Unknown,
        Some(10),
    );
    artifact.metadata.insert("large".to_string(), json!(1));
    inputs.artifacts.insert("source".to_string(), artifact);
    let mut bad = inputs.artifacts.get_mut("source").unwrap().clone();
    bad.schema_version = 99;
    inputs.artifacts.insert("source".to_string(), bad);
    assert!(analyze_context_stability(&request(), &inputs).is_err());
}
