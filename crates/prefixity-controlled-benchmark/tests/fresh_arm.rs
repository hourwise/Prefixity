use prefixity_controlled_benchmark::{
    aggregate_fresh_arm_results, build_synthetic_paired_mutation_seed, execute_fresh_arm,
    finalize_fresh_arm_record, persist_fresh_arm_record, plan_request_layout,
    prepare_fresh_arm_experiment, prepare_paired_mutation_experiment, BenchmarkError,
    CandidateEvaluation, EvidenceSourceClass, FreshArmKind, FreshArmRunRecord,
    LivePreparationErrorCode, LiveRawEvidenceSource, LlamaCppLiveConfig, LlamaCppRequest,
    LlamaCppResponse, LlamaCppTransport, LoopbackEndpoint, PairedMutationDefinition,
    RawLlamaCppEvidence, RuntimeProfileReference,
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
        connect_timeout_ms: 1000,
        request_timeout_ms: 600000,
        max_response_bytes: 1024 * 1024,
        max_context_bytes: 1024 * 1024,
        evidence_location: "evidence/p0-l6e".to_string(),
        execute_live,
        fresh_server_for_run,
        runtime_profile: profile(),
        provenance: BTreeMap::from([
            ("caller".to_string(), "offline-test".to_string()),
            (
                "observed_at".to_string(),
                "2026-08-14T00:00:00Z".to_string(),
            ),
        ]),
    }
}

fn bundle_with_epochs(
    control_epoch_id: &str,
    treatment_epoch_id: &str,
) -> (
    PairedMutationDefinition,
    prefixity_controlled_benchmark::FreshArmExperimentDefinition,
    LlamaCppLiveConfig,
) {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    let candidate = plan_request_layout(
        &seed.control_mutated,
        &seed.stability_inputs,
        &seed.constraints,
    )
    .unwrap()
    .candidates
    .into_iter()
    .next()
    .expect("synthetic fixture should have one safe candidate");
    let paired = prepare_paired_mutation_experiment(
        seed,
        profile(),
        true,
        BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    )
    .unwrap();
    let design = prepare_fresh_arm_experiment(
        &paired,
        candidate,
        &config(false, true),
        control_epoch_id,
        treatment_epoch_id,
        BTreeMap::from([("caller".to_string(), "offline-test".to_string())]),
    )
    .unwrap();
    (paired, design, config(false, true))
}

fn prepared() -> (
    PairedMutationDefinition,
    prefixity_controlled_benchmark::FreshArmExperimentDefinition,
    LlamaCppLiveConfig,
) {
    bundle_with_epochs("control-epoch-001", "treatment-epoch-001")
}

#[derive(Default)]
struct CompleteOfflineTransport {
    calls: usize,
    bodies: Vec<Vec<u8>>,
    raw: Vec<RawLlamaCppEvidence>,
}

impl LlamaCppTransport for CompleteOfflineTransport {
    fn chat_completion(
        &mut self,
        request: &LlamaCppRequest,
    ) -> Result<LlamaCppResponse, BenchmarkError> {
        self.calls += 1;
        self.bodies.push(
            serde_json::to_vec(request)
                .map_err(|error| BenchmarkError::validation(error.to_string()))?,
        );
        self.raw.push(RawLlamaCppEvidence {
            schema_id: prefixity_controlled_benchmark::RAW_EVIDENCE_SCHEMA_ID.to_string(),
            schema_version: prefixity_controlled_benchmark::RAW_EVIDENCE_SCHEMA_VERSION,
            request_identity_fingerprint: "a".repeat(64),
            response_status: 200,
            response_body_bytes: 2,
            response_body_fingerprint: "b".repeat(64),
            elapsed_ms: 1,
            raw_telemetry: BTreeMap::new(),
            provenance: BTreeMap::from([("transport".to_string(), "offline-fake".to_string())]),
        });
        Ok(LlamaCppResponse {
            timings: None,
            usage: None,
            raw_telemetry: BTreeMap::new(),
        })
    }
}

impl LiveRawEvidenceSource for CompleteOfflineTransport {
    fn raw_evidence(&self) -> Vec<RawLlamaCppEvidence> {
        self.raw.clone()
    }
}

fn run_both(
    design: &prefixity_controlled_benchmark::FreshArmExperimentDefinition,
) -> (FreshArmRunRecord, FreshArmRunRecord) {
    let live_config = config(true, true);
    let mut control_transport = CompleteOfflineTransport::default();
    let control = execute_fresh_arm(
        design,
        FreshArmKind::Control,
        &live_config,
        &mut control_transport,
    )
    .unwrap();
    let mut treatment_transport = CompleteOfflineTransport::default();
    let treatment = execute_fresh_arm(
        design,
        FreshArmKind::Treatment,
        &live_config,
        &mut treatment_transport,
    )
    .unwrap();
    assert_eq!(control_transport.calls, 2);
    assert_eq!(treatment_transport.calls, 2);
    (control, treatment)
}

#[test]
fn design_has_two_independent_two_case_arms_and_no_b1() {
    let (paired, design, _) = prepared();
    design.validate().unwrap();
    assert_eq!(
        design
            .control
            .steps
            .iter()
            .map(|step| step.case_id.as_str())
            .collect::<Vec<_>>(),
        ["A0", "A1"]
    );
    assert_eq!(
        design
            .treatment
            .steps
            .iter()
            .map(|step| step.case_id.as_str())
            .collect::<Vec<_>>(),
        ["C0", "C1"]
    );
    assert!(design.no_interference_case);
    assert!(design.control.fresh_server_for_arm);
    assert!(design.treatment.fresh_server_for_arm);
    assert_eq!(
        design.treatment.mutation_request_diff,
        paired.treatment_mutation.request_diff
    );
    assert_eq!(design.candidate_pair.source_case_id, "A1");
    assert_eq!(design.candidate_pair.candidate_case_id, "C1");
    assert!(!design.control.steps.iter().any(|step| step.case_id == "B1"));
    assert!(!design
        .treatment
        .steps
        .iter()
        .any(|step| step.case_id == "B1"));
}

#[test]
fn duplicate_epoch_ids_are_rejected_at_preparation_and_validation() {
    let seed = build_synthetic_paired_mutation_seed().unwrap();
    let candidate = plan_request_layout(
        &seed.control_mutated,
        &seed.stability_inputs,
        &seed.constraints,
    )
    .unwrap()
    .candidates
    .into_iter()
    .next()
    .unwrap();
    let paired =
        prepare_paired_mutation_experiment(seed, profile(), true, BTreeMap::new()).unwrap();
    let error = prepare_fresh_arm_experiment(
        &paired,
        candidate,
        &config(false, true),
        "same-epoch",
        "same-epoch",
        BTreeMap::new(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("epoch IDs must differ"));

    let (_, mut design, _) = prepared();
    design.treatment.epoch_id = design.control.epoch_id.clone();
    assert!(design.validate().is_err());
}

#[test]
fn preflight_is_zero_network_and_projects_max_tokens_one_for_all_four_cases() {
    let (_, design, _) = prepared();
    let readiness = prefixity_controlled_benchmark::preflight_fresh_arm_experiment(
        &design,
        &config(false, true),
    )
    .unwrap();
    readiness.validate(&design).unwrap();
    assert_eq!(readiness.network_calls, 0);
    assert_eq!(readiness.generation_limit, 1);
    assert_eq!(readiness.arms.len(), 2);
    assert_eq!(readiness.arms[0].step_ids, ["A0", "A1"]);
    assert_eq!(readiness.arms[1].step_ids, ["C0", "C1"]);
}

#[test]
fn exact_attempt006_runtime_contract_is_required() {
    let (_, design, _) = prepared();
    let mut invalid = config(false, true);
    invalid.request_timeout_ms = 30000;
    let error = prefixity_controlled_benchmark::preflight_fresh_arm_experiment(&design, &invalid)
        .unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::InvalidConfiguration,
            ..
        }
    ));

    let error = prefixity_controlled_benchmark::preflight_fresh_arm_experiment(
        &design,
        &config(false, false),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        BenchmarkError::LiveHarness {
            code: LivePreparationErrorCode::FreshServerAssertionRequired,
            ..
        }
    ));
}

#[test]
fn control_and_treatment_execution_are_independent_and_bounded() {
    let (_, design, _) = prepared();
    let (control, treatment) = run_both(&design);
    assert_eq!(
        control.state,
        prefixity_controlled_benchmark::LiveEvidenceState::Normalized
    );
    assert_eq!(
        treatment.state,
        prefixity_controlled_benchmark::LiveEvidenceState::Normalized
    );
    assert_eq!(control.completed_steps, 2);
    assert_eq!(treatment.completed_steps, 2);
    assert_eq!(control.transport_attempts, 2);
    assert_eq!(treatment.transport_attempts, 2);
    assert_eq!(
        control
            .normalized_result
            .as_ref()
            .unwrap()
            .cases
            .iter()
            .map(|case| case.case_id.as_str())
            .collect::<Vec<_>>(),
        ["A0", "A1"]
    );
    assert_eq!(
        treatment
            .normalized_result
            .as_ref()
            .unwrap()
            .cases
            .iter()
            .map(|case| case.case_id.as_str())
            .collect::<Vec<_>>(),
        ["C0", "C1"]
    );
}

#[test]
fn each_arm_can_be_finalized_and_persisted_before_the_other_arm() {
    let (_, design, _) = prepared();
    let (control, _) = run_both(&design);
    let bytes = finalize_fresh_arm_record(&control, &design.control).unwrap();
    assert!(!bytes.is_empty());
    let path = std::env::temp_dir().join(format!(
        "prefixity-p0-l6e-control-{}.json",
        std::process::id()
    ));
    persist_fresh_arm_record(&path, &control, &design.control).unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn aggregation_requires_both_fresh_arms_and_matching_identity() {
    let (_, design, _) = prepared();
    let (control, treatment) = run_both(&design);
    assert!(aggregate_fresh_arm_results(&design, Some(&control), None, None).is_err());

    let mut stale = treatment.clone();
    stale.runtime_config_fingerprint = "c".repeat(64);
    assert!(aggregate_fresh_arm_results(&design, Some(&control), Some(&stale), None).is_err());

    let mut duplicated = treatment.clone();
    duplicated.epoch_id = control.epoch_id.clone();
    assert!(aggregate_fresh_arm_results(&design, Some(&control), Some(&duplicated), None).is_err());
}

#[test]
fn aggregation_uses_experimental_source_and_preserves_noncausal_claim_bounds() {
    let (_, design, _) = prepared();
    let (control, treatment) = run_both(&design);
    let aggregate =
        aggregate_fresh_arm_results(&design, Some(&control), Some(&treatment), None).unwrap();
    aggregate.validate().unwrap();
    for diagnostic in [
        &aggregate.control_mutation,
        &aggregate.treatment_mutation,
        &aggregate.candidate_comparison,
    ] {
        assert_eq!(
            diagnostic.evidence.causality,
            prefixity_controlled_benchmark::CausalityStatus::NotEstablished
        );
        assert_eq!(
            diagnostic.observation_comparison.left.source,
            EvidenceSourceClass::ExperimentallyObservedRuntime
        );
        assert_eq!(
            diagnostic.observation_comparison.right.source,
            EvidenceSourceClass::ExperimentallyObservedRuntime
        );
    }
    assert_eq!(
        aggregate
            .candidate_evaluation
            .claim_permissions
            .performance_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
    assert_eq!(
        aggregate
            .candidate_evaluation
            .claim_permissions
            .application_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
    assert_eq!(
        aggregate
            .candidate_evaluation
            .claim_permissions
            .causal_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
    assert_ne!(
        aggregate.candidate_evaluation.next_action,
        prefixity_controlled_benchmark::NextAction::NoAction
    );
}

#[test]
fn semantic_identity_is_deterministic_distinct_from_parent_and_epoch_independent() {
    let (paired, first, _) = prepared();
    let (_, same, _) = prepared();
    let (_, different_epochs, _) = bundle_with_epochs("control-epoch-002", "treatment-epoch-002");
    assert_eq!(first.semantic_experiment_id, same.semantic_experiment_id);
    assert_eq!(
        first.semantic_experiment_id,
        different_epochs.semantic_experiment_id
    );
    assert_ne!(first.semantic_experiment_id, paired.experiment_id);
    assert_ne!(first.semantic_experiment_id, "0".repeat(64));
}

#[test]
fn p0_l13_materialization_identities_are_reused_without_perturbation() {
    let (paired, design, _) = prepared();
    assert_eq!(
        design.candidate_pair.source_request_fingerprint,
        paired.treatment_mutated.source_request_fingerprint
    );
    assert_eq!(
        design.candidate_pair.candidate_request_fingerprint,
        paired.treatment_mutated.materialized_request_fingerprint
    );
    assert_eq!(
        design.candidate_pair.candidate_fingerprint,
        paired.treatment_mutated.candidate_fingerprint
    );
    assert_eq!(
        design.candidate.layout_fingerprint,
        paired.treatment_mutated.candidate_fingerprint
    );
    assert!(paired
        .treatment_initial
        .safety_certificate
        .validate()
        .is_ok());
    assert!(paired
        .treatment_mutated
        .safety_certificate
        .validate()
        .is_ok());
}

#[test]
fn normalized_arm_record_requires_explicit_final_state() {
    let (_, design, _) = prepared();
    let (control, _) = run_both(&design);
    let mut invalid = control.clone();
    invalid.state = prefixity_controlled_benchmark::LiveEvidenceState::Partial;
    assert!(invalid.validate_against(&design.control).is_err());
}

#[test]
fn no_network_transport_is_needed_for_design_or_preflight() {
    let (_, design, _) = prepared();
    let readiness = prefixity_controlled_benchmark::preflight_fresh_arm_experiment(
        &design,
        &config(false, true),
    )
    .unwrap();
    assert_eq!(readiness.network_calls, 0);
}

#[test]
fn canonical_design_is_stable() {
    let (_, first, _) = prepared();
    let (_, second, _) = prepared();
    assert_eq!(
        first.canonical_json().unwrap(),
        second.canonical_json().unwrap()
    );
}

#[test]
fn capability_and_application_claims_remain_gated_when_profile_is_absent() {
    let (_, design, _) = prepared();
    let (control, treatment) = run_both(&design);
    let aggregate =
        aggregate_fresh_arm_results(&design, Some(&control), Some(&treatment), None).unwrap();
    let evaluation: &CandidateEvaluation = &aggregate.candidate_evaluation;
    assert_eq!(
        evaluation.claim_permissions.causal_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
    assert_eq!(
        evaluation.claim_permissions.performance_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
    assert_eq!(
        evaluation.claim_permissions.application_claims,
        prefixity_controlled_benchmark::ClaimPermission::NotAllowed
    );
}
