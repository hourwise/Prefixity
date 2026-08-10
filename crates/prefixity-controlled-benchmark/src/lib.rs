//! Isolated Phase 1B.7 controlled benchmark implementation.
//!
//! This crate is offline-only. It owns the controlled envelope, deterministic
//! self-authored seed, scripted world, evaluation oracle, and a one-way
//! planner-visible projection. It does not alter `RequestTrace`, planner
//! eligibility, CodeTraceBench, or live-provider code.

mod error;
mod fixtures;
mod hashing;
mod loader;
mod model;
mod oracle;
mod phase1b9;
mod planner;
mod world;

pub use error::BenchmarkError;
pub use fixtures::build_seed;
pub use loader::{
    canonical_envelope_json, envelope_hash, load_envelope, load_envelope_from_path, manifest_hash,
    validate_case, validate_envelope,
};
pub use model::{
    ActionIdentity, ActorRole, AggregateCounts, BenchmarkReport, ControlledCase,
    ControlledEnvelope, EvaluationRecord, Event, EventType, EvidenceClass, InterventionClass,
    InterventionManifest, OracleResult, OrderMetadata, PlannerEvidence, PlannerInput, PlannerRun,
    PlannerVisibility, QualityRiskCategory, Relation, RelationType, ScenarioIdentity, SourceKind,
    SourceProvenance, TimestampOrigin, TraceEnvelope, VariantRole, BENCHMARK_ID,
    ENVIRONMENT_REVISION, ORACLE_VERSION, RELATION_SEMANTICS_VERSION, SCHEMA_ID, SCHEMA_VERSION,
    TASK_REVISION,
};
pub use oracle::{evaluate_case, evaluate_envelopes};
pub use phase1b9::{
    blinded_trace_json, canonical_phase1b9_report_json, preregistration_hash, run_phase1b9_study,
    BlindedEvent, BlindedRelation, BlindedTrace, FrozenPlannerBaseline, Phase1b9DecisionRecord,
    Phase1b9Report, ResearchInterventionClass, ResearchPolicyDecision, PHASE_1B9_POLICY_VERSION,
    PHASE_1B9_SCOPE,
};
pub use planner::{project_planner_evidence, run_frozen_planner};
pub use world::{ExecutionStatus, ScriptedWorld, WorldExecution};

/// Build, evaluate, and run the frozen planner over every self-authored pair.
///
/// The evaluation sidecar is consumed only by the oracle. The planner runs
/// are created from `PlannerEvidence` projections before any evaluation result
/// is produced or consulted.
pub fn run_benchmark() -> Result<BenchmarkReport, BenchmarkError> {
    let cases = build_seed()?;
    let mut evaluations = Vec::with_capacity(cases.len());
    let mut planner_runs = Vec::with_capacity(cases.len() * 2);
    let mut manifest_hashes = std::collections::BTreeMap::new();
    let mut baseline_count = 0;
    let mut variant_count = 0;
    let mut control_count = 0;

    for case in &cases {
        manifest_hashes.insert(case.scenario_id.clone(), case.manifest_hash.clone());
        baseline_count += 1;
        match case.intervention.trace.variant_role {
            VariantRole::Variant => variant_count += 1,
            VariantRole::Control => control_count += 1,
            VariantRole::Baseline => {
                return Err(BenchmarkError::pair(
                    &case.scenario_id,
                    "intervention unexpectedly has baseline role",
                ));
            }
        }

        let baseline_evidence = project_planner_evidence(&case.baseline)?;
        let intervention_evidence = project_planner_evidence(&case.intervention)?;
        planner_runs.push(run_frozen_planner(&baseline_evidence)?);
        planner_runs.push(run_frozen_planner(&intervention_evidence)?);

        evaluations.push(evaluate_case(case)?);
    }

    let aggregate_input = serde_json::json!({
        "artifact_id": BENCHMARK_ID,
        "schema_id": SCHEMA_ID,
        "schema_version": SCHEMA_VERSION,
        "oracle_version": ORACLE_VERSION,
        "manifest_hashes": manifest_hashes,
        "evaluations": evaluations,
        "planner_runs": planner_runs,
    });
    let aggregate_hash = hashing::canonical_hash(&aggregate_input)
        .map_err(|error| BenchmarkError::validation(error.to_string()))?;
    let mut aggregate_counts = AggregateCounts {
        pass: 0,
        fail: 0,
        invalid_baseline: 0,
        inconclusive: 0,
    };
    for evaluation in &evaluations {
        aggregate_counts.record(evaluation.result);
    }

    Ok(BenchmarkReport {
        artifact_id: BENCHMARK_ID.to_string(),
        schema_id: SCHEMA_ID.to_string(),
        schema_version: SCHEMA_VERSION,
        oracle_version: ORACLE_VERSION.to_string(),
        scenario_count: cases.len(),
        baseline_count,
        variant_count,
        control_count,
        manifest_hashes,
        aggregate_hash,
        evaluations,
        aggregate_counts,
        planner_runs,
    })
}

pub fn canonical_report_json(report: &BenchmarkReport) -> Result<Vec<u8>, BenchmarkError> {
    hashing::canonical_json(report).map_err(|error| BenchmarkError::validation(error.to_string()))
}
