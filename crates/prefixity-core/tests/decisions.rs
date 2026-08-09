//! Phase 1B.0 intervention-plan contract and conservative-baseline tests.

mod common;

use prefixity_core::analysis::TraceRef;
use prefixity_core::decision::{
    plan_interventions, EvidenceSources, EvidenceStrength, InterventionClass, InterventionPlan,
    InterventionRecommendation, ProviderStateDependence, QualityRisk, ReasonCode,
};
use prefixity_core::hash::hash_content;
use prefixity_core::model::{ContextBlock, RequestTrace, TRACE_FORMAT_VERSION};
use std::collections::BTreeMap;

fn plan_for(name: &str) -> InterventionPlan {
    plan_interventions(&common::load_fixture(name)).unwrap()
}

fn classes(plan: &InterventionPlan) -> Vec<InterventionClass> {
    plan.recommendations
        .iter()
        .map(|recommendation| recommendation.class)
        .collect()
}

fn block(id: &str, source: &str, position: usize) -> ContextBlock {
    ContextBlock {
        id: id.to_string(),
        source: source.to_string(),
        position,
        content_hash: hash_content(id),
        token_count: Some(10),
        byte_count: id.len() as u64,
        content: None,
        semantic_zone: None,
        structural_path: None,
        role: None,
        sensitivity: None,
        dependencies: Vec::new(),
        lifetime: None,
        optional: false,
        required: false,
        stale: false,
        metadata: BTreeMap::new(),
    }
}

fn trace(blocks: Vec<ContextBlock>) -> RequestTrace {
    RequestTrace {
        format_version: TRACE_FORMAT_VERSION,
        request_id: "phase-1b-test".to_string(),
        session_id: None,
        timestamp: None,
        provider: "synthetic".to_string(),
        model: "synthetic-model".to_string(),
        blocks,
        usage: None,
        latency: None,
        metadata: BTreeMap::new(),
    }
}

fn contract_recommendation(class: InterventionClass) -> InterventionRecommendation {
    InterventionRecommendation {
        class,
        target_block_ids: if class == InterventionClass::DoNothing {
            Vec::new()
        } else {
            vec!["block-1".to_string()]
        },
        reason_codes: vec![ReasonCode::NoJustifiedIntervention],
        explanation: "contract serialization fixture".to_string(),
        evidence_strength: EvidenceStrength::Unknown,
        source_evidence: EvidenceSources {
            structural: vec!["fixture".to_string()],
            provider_cache: vec!["absent".to_string()],
            economic: vec!["absent".to_string()],
            quality: vec!["absent".to_string()],
            dependency: vec!["none".to_string()],
        },
        relevant_dependencies: Vec::new(),
        expected_structural_effect: "fixture".to_string(),
        expected_quality_risk: QualityRisk::Unknown,
        provider_state_dependence: ProviderStateDependence::Unknown,
        provider_evidence_present: false,
        economic_evidence_present: false,
        hypothetical_only: true,
    }
}

#[test]
fn identical_input_produces_identical_auditable_plan() {
    let trace = common::load_fixture("06-context-reduction-wins.json");
    let first = plan_interventions(&trace).unwrap();
    let second = plan_interventions(&trace).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_string(&first).unwrap(),
        serde_json::to_string(&second).unwrap()
    );
}

#[test]
fn all_six_decision_classes_serialize_through_one_contract() {
    let recommendations = InterventionClass::all()
        .into_iter()
        .map(contract_recommendation)
        .collect::<Vec<_>>();
    let plan = InterventionPlan {
        contract_version: 1,
        trace: TraceRef {
            request_id: "contract".to_string(),
            session_id: None,
            provider: "synthetic".to_string(),
            model: "synthetic-model".to_string(),
        },
        recommendations,
        retained_block_ids: vec!["block-1".to_string()],
        planner_notes: vec!["fixture".to_string()],
        hypothetical_only: true,
    };
    let value = serde_json::to_value(plan).unwrap();
    let serialized = value["recommendations"].as_array().unwrap();
    let actual = serialized
        .iter()
        .map(|recommendation| recommendation["class"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            "KEEP",
            "DEFER",
            "PRUNE",
            "RELOCATE_CANDIDATE",
            "COMPRESS_CANDIDATE",
            "DO_NOTHING"
        ]
    );
}

#[test]
fn required_block_is_never_pruned_or_deferred() {
    let plan = plan_for("08-unsafe-pruning-example.json");
    assert!(plan
        .retained_block_ids
        .iter()
        .any(|id| id == "critical-config"));
    assert!(plan.recommendations.iter().all(|recommendation| {
        !matches!(
            recommendation.class,
            InterventionClass::Prune | InterventionClass::Defer
        )
    }));
    assert!(plan.recommendations[0]
        .reason_codes
        .contains(&ReasonCode::RequiredBlock));
}

#[test]
fn unknown_safety_defaults_to_do_nothing() {
    let plan = plan_interventions(&trace(vec![block("mystery", "unclassified", 0)])).unwrap();
    assert_eq!(classes(&plan), vec![InterventionClass::DoNothing]);
    assert!(plan.recommendations[0]
        .reason_codes
        .contains(&ReasonCode::UnknownSafety));
}

#[test]
fn optional_stale_tool_result_gets_defensible_prune_candidate() {
    let plan = plan_for("06-context-reduction-wins.json");
    let recommendation = plan
        .recommendations
        .iter()
        .find(|recommendation| recommendation.target_block_ids == ["stale-tool-output"])
        .unwrap();
    assert_eq!(recommendation.class, InterventionClass::Prune);
    assert!(recommendation
        .reason_codes
        .contains(&ReasonCode::OptionalStaleToolResult));
    assert!(recommendation.hypothetical_only);
}

#[test]
fn retained_dependency_blocks_destructive_recommendation() {
    let mut source = common::load_fixture("06-context-reduction-wins.json");
    source.blocks[7]
        .dependencies
        .push("stale-tool-output".to_string());
    let plan = plan_interventions(&source).unwrap();
    assert!(plan.recommendations.iter().all(|recommendation| {
        !matches!(
            recommendation.class,
            InterventionClass::Prune | InterventionClass::Defer
        )
    }));
    assert!(plan.recommendations.iter().any(|recommendation| {
        recommendation.target_block_ids == ["stale-tool-output"]
            && recommendation
                .reason_codes
                .contains(&ReasonCode::DependencyClosureProtected)
    }));
}

#[test]
fn optional_volatile_tool_result_gets_defer_candidate() {
    let mut source = common::load_fixture("06-context-reduction-wins.json");
    source.blocks[6].stale = false;
    let plan = plan_interventions(&source).unwrap();
    let recommendation = plan
        .recommendations
        .iter()
        .find(|recommendation| recommendation.target_block_ids == ["stale-tool-output"])
        .unwrap();
    assert_eq!(recommendation.class, InterventionClass::Defer);
    assert!(recommendation
        .reason_codes
        .contains(&ReasonCode::OptionalVolatileToolResult));
}

#[test]
fn safe_structural_relocation_is_only_a_hypothetical_candidate() {
    let first = block("file", "file_content", 0);
    let second = block("map", "repository_map", 1);
    let source = trace(vec![first, second]);
    let original = source.clone();
    let plan = plan_interventions(&source).unwrap();
    assert!(plan
        .recommendations
        .iter()
        .any(|recommendation| recommendation.class == InterventionClass::RelocateCandidate));
    assert_eq!(source, original);
    assert!(plan
        .recommendations
        .iter()
        .all(|recommendation| { recommendation.class != InterventionClass::CompressCandidate }));
}

#[test]
fn unsafe_cross_zone_or_chronological_relocation_is_do_nothing() {
    let plan = plan_for("16-global-reorder-would-be-unsafe.json");
    assert_eq!(classes(&plan), vec![InterventionClass::DoNothing]);
    assert!(plan.recommendations[0]
        .reason_codes
        .contains(&ReasonCode::CrossZoneRelocationRejected));
}

#[test]
fn no_justified_intervention_remains_do_nothing() {
    let plan = plan_for("07-already-optimal.json");
    assert_eq!(classes(&plan), vec![InterventionClass::DoNothing]);
    assert!(plan.recommendations[0]
        .reason_codes
        .contains(&ReasonCode::NoJustifiedIntervention));
}

#[test]
fn evidence_dimensions_and_presence_flags_remain_distinct() {
    let plan = plan_for("06-context-reduction-wins.json");
    let prune = plan
        .recommendations
        .iter()
        .find(|recommendation| recommendation.class == InterventionClass::Prune)
        .unwrap();
    assert!(prune.provider_evidence_present);
    assert!(!prune.economic_evidence_present);
    assert!(!prune.source_evidence.structural.is_empty());
    assert!(!prune.source_evidence.provider_cache.is_empty());
    assert!(!prune.source_evidence.economic.is_empty());
    assert!(!prune.source_evidence.quality.is_empty());
    assert!(!prune.source_evidence.dependency.is_empty());
}

#[test]
fn planner_never_mutates_original_trace() {
    for fixture in [
        "06-context-reduction-wins.json",
        "07-already-optimal.json",
        "16-global-reorder-would-be-unsafe.json",
    ] {
        let source = common::load_fixture(fixture);
        let original = source.clone();
        let _ = plan_interventions(&source).unwrap();
        assert_eq!(source, original, "planner mutated fixture {fixture}");
    }
}
