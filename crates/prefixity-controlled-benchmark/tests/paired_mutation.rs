use prefixity_controlled_benchmark::{
    build_synthetic_paired_mutation_seed, preflight_paired_mutation_experiment,
    prepare_paired_mutation_experiment, BenchmarkError, LiveEvidenceState,
    LivePreparationErrorCode, LiveRawEvidenceSource, LlamaCppLiveConfig, LlamaCppRequest,
    LlamaCppResponse, LlamaCppTransport, LoopbackEndpoint, PairedMutationDefinition,
    PairedMutationRunRecord, RuntimeProfileReference,
};
use prefixity_core::observation::{Observed, RuntimeIdentity};
use std::collections::BTreeMap;

fn profile() -> RuntimeProfileReference {
    RuntimeProfileReference {
        profile_id: "fixture-paired-loopback".to_string(),
        identity: RuntimeIdentity {
            backend: "llama.cpp".to_string(),
            provider: Observed::Known("local".to_string()),
            model: Observed::Known("fixture-model".to_string()),
            protocol: Observed::Known("llama.cpp-openai-chat-v1".to_string()),
            runtime: Observed::Known("llama-server".to_string()),
            runtime_version: Observed::Known("caller-supplied".to_string()),
            ..RuntimeIdentity::default()
        },
    }
}

fn config(execute_live: bool, fresh_server_for_run: bool) -> LlamaCppLiveConfig {
    LlamaCppLiveConfig {
        schema_id: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::LIVE_CONFIG_SCHEMA_VERSION,
        endpoint: LoopbackEndpoint::parse("http://127.0.0.1:8080/v1/chat/completions").unwrap(),
        llama_build: "caller-supplied".to_string(),
        model_identity: "ggml-org/Qwen3.5-0.8B-GGUF".to_string(),
        quantization: Some("Q4_0".to_string()),
        context_size: 8192,
        threads: None,
        gpu_offload: None,
        kv_cache: None,
        batch_size: None,
        generation_limit: 1,
        parallel_slots: 1,
        metrics_enabled: true,
        temperature: Some(0.0),
        top_p: Some(1.0),
        seed: Some(1),
        connect_timeout_ms: 100,
        request_timeout_ms: 1000,
        max_response_bytes: 1024 * 1024,
        max_context_bytes: 1024 * 1024,
        evidence_location: "evidence/p0-l6b".to_string(),
        execute_live,
        fresh_server_for_run,
        runtime_profile: profile(),
        provenance: BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    }
}

fn prepared() -> PairedMutationDefinition {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    prepare_paired_mutation_experiment(
        seed,
        profile(),
        true,
        BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    )
    .unwrap()
}

#[test]
fn paired_experiment_has_exactly_five_named_steps() {
    let definition = prepared();
    assert_eq!(definition.sequence.len(), 5);
    assert_eq!(
        definition
            .sequence
            .iter()
            .map(|step| step.step_id.as_str())
            .collect::<Vec<_>>(),
        ["A0", "A1", "B1", "C0", "C1"]
    );
}

#[test]
fn control_mutation_changes_only_volatile_artifact() {
    let definition = prepared();
    let diff = &definition.control_mutation.request_diff;
    assert_eq!(diff.prefix_diff.changes.len(), 1);
    assert_eq!(
        diff.prefix_diff.changes[0].category,
        prefixity_controlled_benchmark::ChangeCategory::ArtifactContentChanged
    );
    assert_ne!(
        definition.control_initial.context.artifacts[1].content,
        definition.control_mutated.context.artifacts[1].content
    );
    assert!(diff.envelope_diff.changes.is_empty());
}

#[test]
fn c0_is_certified_from_a0() {
    let definition = prepared();
    assert_eq!(
        definition.treatment_initial.source_request_fingerprint,
        definition.control_initial.request_fingerprint().unwrap()
    );
    assert!(definition
        .treatment_initial
        .safety_certificate
        .validate()
        .is_ok());
}

#[test]
fn c1_is_independently_certified_from_a1() {
    let definition = prepared();
    assert_eq!(
        definition.treatment_mutated.source_request_fingerprint,
        definition.control_mutated.request_fingerprint().unwrap()
    );
    assert!(definition
        .treatment_mutated
        .safety_certificate
        .validate()
        .is_ok());
    assert_ne!(
        definition.treatment_initial.safety_certificate,
        definition.treatment_mutated.safety_certificate
    );
}

#[test]
fn information_is_conserved_for_both_layouts() {
    let definition = prepared();
    for (source, candidate) in [
        (
            &definition.control_initial,
            &definition.treatment_initial.materialized_request,
        ),
        (
            &definition.control_mutated,
            &definition.treatment_mutated.materialized_request,
        ),
    ] {
        assert_eq!(
            source.context.system_instruction,
            candidate.context.system_instruction
        );
        assert_eq!(source.context.user_content, candidate.context.user_content);
        assert_eq!(source.context.tools, candidate.context.tools);
        assert_eq!(
            source
                .context
                .artifacts
                .iter()
                .map(|artifact| artifact.artifact_id.as_str())
                .collect::<std::collections::BTreeSet<_>>(),
            candidate
                .context
                .artifacts
                .iter()
                .map(|artifact| artifact.artifact_id.as_str())
                .collect::<std::collections::BTreeSet<_>>()
        );
    }
}

#[test]
fn control_and_treatment_structures_are_explicit() {
    let definition = prepared();
    assert_eq!(
        definition
            .control_initial
            .context
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>(),
        ["stable-a", "volatile-v", "stable-b"]
    );
    assert_eq!(
        definition
            .treatment_initial
            .materialized_request
            .context
            .artifacts
            .iter()
            .map(|artifact| artifact.artifact_id.as_str())
            .collect::<Vec<_>>(),
        ["stable-a", "stable-b", "volatile-v"]
    );
}

#[test]
fn p0_l10_inversion_reduction_is_recorded_without_a_claim() {
    let definition = prepared();
    assert!(
        definition.treatment_initial_inversion_count <= definition.control_initial_inversion_count
    );
    assert!(
        definition.treatment_initial_leading_segments
            >= definition.control_initial_leading_segments
    );
    assert_eq!(
        definition.primary_comparison.cache_outcome,
        "not_observed; no required direction"
    );
}

#[test]
fn p0_l7_layout_diffs_are_reorder_only() {
    let definition = prepared();
    for comparison in [&definition.control_layout, &definition.treatment_layout] {
        assert_eq!(comparison.request_diff.prefix_diff.changes.len(), 1);
        assert_eq!(
            comparison.request_diff.prefix_diff.changes[0].category,
            prefixity_controlled_benchmark::ChangeCategory::ArtifactOrderChanged
        );
        assert!(comparison.request_diff.envelope_diff.changes.is_empty());
    }
}

#[test]
fn p0_l7_treatment_mutation_is_the_same_volatile_change() {
    let definition = prepared();
    let control_change = &definition.control_mutation.request_diff.prefix_diff.changes[0];
    let treatment_change = &definition
        .treatment_mutation
        .request_diff
        .prefix_diff
        .changes[0];
    assert_eq!(control_change.category, treatment_change.category);
    assert_ne!(
        definition
            .treatment_initial
            .materialized_request
            .context
            .artifacts[2]
            .content,
        definition
            .treatment_mutated
            .materialized_request
            .context
            .artifacts[2]
            .content
    );
    assert!(!treatment_change.order_changed);
}

#[test]
fn identity_and_sequence_are_deterministic() {
    let first = prepared();
    let second = prepared();
    assert_eq!(first.experiment_id, second.experiment_id);
    assert_eq!(first.sequence, second.sequence);
}

#[test]
fn preflight_is_zero_network_and_records_comparisons() {
    let record = preflight_paired_mutation_experiment(&prepared(), &config(false, true)).unwrap();
    assert_eq!(record.state, LiveEvidenceState::Prepared);
    assert_eq!(record.network_calls, 0);
    assert_eq!(record.comparisons, ["A0-to-A1", "C0-to-C1", "A1-vs-C1"]);
    assert!(record.primary_outcome.contains("no required direction"));
}

#[test]
fn fresh_server_is_a_caller_assertion() {
    let error =
        preflight_paired_mutation_experiment(&prepared(), &config(false, false)).unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::FreshServerAssertionRequired,
            ..
        }
    ));
}

#[derive(Default)]
struct CountingTransport {
    calls: usize,
}

impl LlamaCppTransport for CountingTransport {
    fn chat_completion(
        &mut self,
        _request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        self.calls += 1;
        Err(BenchmarkError::validation("transport must not be called"))
    }
}

impl LiveRawEvidenceSource for CountingTransport {
    fn raw_evidence(&self) -> Vec<prefixity_controlled_benchmark::RawLlamaCppEvidence> {
        Vec::new()
    }
}

#[test]
fn paired_execution_requires_explicit_opt_in_without_network() {
    let mut transport = CountingTransport::default();
    let error = prefixity_controlled_benchmark::execute_paired_mutation_experiment(
        &prepared(),
        &config(false, true),
        &mut transport,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::LiveOptInRequired,
            ..
        }
    ));
    assert_eq!(transport.calls, 0);
}

#[test]
fn workload_is_bounded_and_uses_small_generation_limit() {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    assert!(seed.workload.control_initial_bytes < 256 * 1024);
    assert_eq!(seed.workload.context_limit, 8192);
    assert_eq!(config(false, true).generation_limit, 1);
}

#[test]
fn outcome_is_falsifiable_and_not_positive_by_construction() {
    let definition = prepared();
    assert_eq!(
        definition.expected_outcome,
        prefixity_controlled_benchmark::PairedOutcomeExpectation::NoRequiredDirection
    );
    assert!(definition
        .primary_comparison
        .interpretation
        .contains("not causal"));
}

#[test]
fn runtime_unknowns_remain_unknown_or_caller_supplied() {
    let value = config(false, true);
    assert!(value.threads.is_none());
    assert!(value.batch_size.is_none());
    assert!(value.kv_cache.is_none());
    assert!(value.gpu_offload.is_none());
    assert_eq!(value.model_identity, "ggml-org/Qwen3.5-0.8B-GGUF");
    assert_eq!(value.quantization.as_deref(), Some("Q4_0"));
}

#[test]
fn serialized_definition_contains_no_result_or_performance_claim() {
    let serialized = serde_json::to_string(&prepared()).unwrap();
    assert!(!serialized.contains("speedup"));
    assert!(!serialized.contains("statistical_significance"));
    assert!(!serialized.contains("performance_improvement"));
    assert!(serialized.contains("not_observed"));
}

#[test]
fn normal_workspace_inputs_are_synthetic_only() {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    let encoded = serde_json::to_string(&seed).unwrap();
    assert!(encoded.contains("synthetic"));
    assert!(!encoded.contains("secret"));
    assert!(!encoded.contains("private source"));
}

#[test]
fn definition_round_trips_deterministically() {
    let definition = prepared();
    let first = definition.canonical_json().unwrap();
    let second = definition.canonical_json().unwrap();
    assert_eq!(first, second);
}

#[test]
fn incomplete_run_cannot_be_complete_or_admitted() {
    let record = PairedMutationRunRecord {
        schema_id: prefixity_controlled_benchmark::PAIRED_MUTATION_SCHEMA_ID.to_string(),
        schema_version: prefixity_controlled_benchmark::PAIRED_MUTATION_SCHEMA_VERSION,
        experiment_id: "fixture-paired".to_string(),
        state: LiveEvidenceState::Partial,
        expected_steps: 5,
        completed_steps: 2,
        raw_evidence: Vec::new(),
        normalized_result: None,
        failure: Some("incomplete_sequence".to_string()),
        provenance: BTreeMap::new(),
    };
    assert!(record.validate().is_ok());
    let mut invalid = record;
    invalid.state = LiveEvidenceState::Normalized;
    assert!(invalid.validate().is_err());
}
