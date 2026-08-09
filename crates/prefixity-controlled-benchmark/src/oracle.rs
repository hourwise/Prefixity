//! Deterministic quality oracle for the self-authored scripted world.

use crate::error::BenchmarkError;
use crate::loader::{validate_case, validate_envelope};
use crate::model::{
    ControlledCase, ControlledEnvelope, EvaluationRecord, InterventionManifest, OracleResult,
};
use crate::world::{ExecutionStatus, ScriptedWorld, WorldExecution};
use std::collections::{BTreeMap, BTreeSet};

pub fn evaluate_case(case: &ControlledCase) -> Result<EvaluationRecord, BenchmarkError> {
    validate_case(case)?;
    evaluate_envelopes(&case.baseline, &case.intervention, &case.manifest)
}

/// Evaluate structurally valid envelopes without rechecking pair equality.
/// This is intentionally public for the invalid-baseline test path: a valid
/// trace can still fail the baseline task predicate and must be reported as
/// `INVALID_BASELINE`, not silently treated as a negative intervention result.
pub fn evaluate_envelopes(
    baseline: &ControlledEnvelope,
    intervention: &ControlledEnvelope,
    manifest: &InterventionManifest,
) -> Result<EvaluationRecord, BenchmarkError> {
    validate_envelope(baseline)?;
    validate_envelope(intervention)?;
    let world = ScriptedWorld;
    let baseline_run = world.execute(baseline)?;
    let intervention_run = world.execute(intervention)?;

    if baseline_run.status == ExecutionStatus::Unresolved {
        return Ok(record(
            baseline,
            intervention,
            manifest,
            OracleResult::Inconclusive,
            &baseline_run,
            &intervention_run,
            Vec::new(),
            "baseline execution is unresolved",
        ));
    }
    if baseline_run.status != ExecutionStatus::Complete
        || !baseline_run.completed
        || !baseline_matches_expected(baseline, &baseline_run.final_state)
    {
        return Ok(record(
            baseline,
            intervention,
            manifest,
            OracleResult::InvalidBaseline,
            &baseline_run,
            &intervention_run,
            Vec::new(),
            "baseline did not satisfy its deterministic task predicate",
        ));
    }

    let collateral = differing_state_keys(&baseline_run.final_state, &intervention_run.final_state);
    let result = match intervention_run.status {
        ExecutionStatus::Unresolved => OracleResult::Inconclusive,
        ExecutionStatus::TaskFailure => OracleResult::Fail,
        ExecutionStatus::Complete
            if intervention_run.completed
                && baseline_matches_expected(intervention, &intervention_run.final_state)
                && collateral.is_empty()
                && intervention_run.final_state == baseline_run.final_state =>
        {
            OracleResult::Pass
        }
        ExecutionStatus::Complete => OracleResult::Fail,
    };
    let note = match result {
        OracleResult::Pass => "variant preserved task predicates and collateral invariants",
        OracleResult::Fail => "variant failed task predicates or collateral invariants",
        OracleResult::InvalidBaseline => "baseline was not a valid quality reference",
        OracleResult::Inconclusive => "execution did not produce a deterministic conclusion",
    };
    Ok(record(
        baseline,
        intervention,
        manifest,
        result,
        &baseline_run,
        &intervention_run,
        collateral,
        note,
    ))
}

#[allow(clippy::too_many_arguments)]
fn record(
    baseline: &ControlledEnvelope,
    intervention: &ControlledEnvelope,
    manifest: &InterventionManifest,
    result: OracleResult,
    baseline_run: &WorldExecution,
    intervention_run: &WorldExecution,
    collateral_state_keys: Vec<String>,
    note: &str,
) -> EvaluationRecord {
    EvaluationRecord {
        scenario_id: baseline.scenario.scenario_id.clone(),
        manifest_id: manifest.manifest_id.clone(),
        baseline_trace_id: baseline.trace.trace_id.clone(),
        intervention_trace_id: intervention.trace.trace_id.clone(),
        result,
        baseline_completed: baseline_run.completed,
        intervention_completed: intervention_run.completed,
        baseline_final_state_hash: baseline_run.final_state_hash.clone(),
        intervention_final_state_hash: intervention_run.final_state_hash.clone(),
        collateral_state_keys,
        note: note.to_string(),
    }
}

fn baseline_matches_expected(
    envelope: &ControlledEnvelope,
    state: &BTreeMap<String, String>,
) -> bool {
    state == &expected_final_state(envelope)
}

fn expected_final_state(envelope: &ControlledEnvelope) -> BTreeMap<String, String> {
    let mut state = BTreeMap::from([
        (
            "scenario_id".to_string(),
            envelope.scenario.scenario_id.clone(),
        ),
        (
            "initial_state_id".to_string(),
            envelope.scenario.initial_state_id.clone(),
        ),
        (
            "task_revision".to_string(),
            envelope.scenario.task_revision.clone(),
        ),
    ]);
    let values: &[(&str, &str)] = match envelope.scenario.scenario_id.as_str() {
        "S01_irrelevant_context_removal" => &[("profile_updated", "done")],
        "S02_load_bearing_removal_failure" => {
            &[("inventory_checked", "ready"), ("checkout", "complete")]
        }
        "S03_explicit_supersession_deferral" => &[("policy_version", "v2")],
        "S04_action_result_needed_later" => &[("record_created", "yes"), ("record_updated", "yes")],
        "S05_action_result_not_needed" => &[("profile_updated", "done")],
        "S06_dependency_chain_preservation" => &[
            ("record_created", "yes"),
            ("chain_created", "yes"),
            ("authorized", "yes"),
            ("committed", "yes"),
        ],
        "S07_safe_context_relocation" => &[("reference_execution", "complete")],
        "S08_protocol_breaking_relocation" => &[("handshake_execution", "complete")],
        "S09_repeated_context_removal" => &[("reference_execution", "complete")],
        "S10_repeated_but_load_bearing" => &[("load_bearing_execution", "complete")],
        "S11_already_efficient_noop" => &[("minimal_task", "complete")],
        "S12_ambiguous_evidence" => &[("ambiguous_task", "complete")],
        _ => &[],
    };
    for (key, value) in values {
        state.insert((*key).to_string(), (*value).to_string());
    }
    state
}

fn differing_state_keys(
    baseline: &BTreeMap<String, String>,
    intervention: &BTreeMap<String, String>,
) -> Vec<String> {
    let keys = baseline
        .keys()
        .chain(intervention.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    keys.into_iter()
        .filter(|key| baseline.get(key) != intervention.get(key))
        .collect()
}
