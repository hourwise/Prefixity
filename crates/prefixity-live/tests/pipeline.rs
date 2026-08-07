//! Full-pipeline integration tests for the Phase 0B live harness.
//!
//! These tests are **fully offline**: every run uses the [`MockTransport`]
//! and a fake credential from an environment variable. They prove the whole
//! pipeline (planning, guardrails, request building, usage normalization,
//! trace writing, reconciliation, classification, artifact safety) works
//! without internet or real API credentials.

mod common;

use prefixity_live::credentials::Credentials;
use prefixity_live::error::LiveError;
use prefixity_live::experiment::{
    describe_dry_run, execute_live_experiment, ExperimentConfig, Sleep,
};
use prefixity_live::result::Conclusion;
use prefixity_live::scenario::Scenario;
use prefixity_live::transport::{err_response, ok_response, MockTransport};
use std::path::PathBuf;

const TEST_KEY_VALUE: &str = "sk-test-secret-value";

/// A no-op sleeper so integration tests never wait real seconds.
struct NoopSleeper;
impl Sleep for NoopSleeper {
    fn sleep(&self, _millis: u64) {}
}

/// A sleeper that records every requested delay (ms) for assertions.
#[derive(Default)]
struct RecordingSleeper {
    calls: std::sync::Mutex<Vec<u64>>,
}
impl RecordingSleeper {
    fn calls(&self) -> Vec<u64> {
        self.calls.lock().unwrap().clone()
    }
}
impl Sleep for RecordingSleeper {
    fn sleep(&self, millis: u64) {
        self.calls.lock().unwrap().push(millis);
    }
}

/// A fake credential read from a constant test env var.
fn test_key() -> Credentials {
    std::env::set_var("PREFIXITY_LIVE_TEST_KEY", TEST_KEY_VALUE);
    Credentials::from_env("PREFIXITY_LIVE_TEST_KEY").unwrap()
}

fn config(
    provider: &str,
    scenario: Scenario,
    runs_dir: PathBuf,
    experiment_id: &str,
) -> ExperimentConfig {
    ExperimentConfig {
        provider_id: provider.to_string(),
        model: "test-model".to_string(),
        scenario,
        seed: 42,
        target_prefix_tokens: 8000,
        max_requests: 3,
        max_estimated_input_tokens: 50_000,
        timeout_ms: 5_000,
        runs_dir,
        experiment_id: experiment_id.to_string(),
        notes: Some("integration test".to_string()),
    }
}

/// Assert no credential value appears in any artifact under `dir`.
fn assert_no_credential_in_dir(dir: &std::path::Path) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            let data = std::fs::read_to_string(&path).unwrap();
            assert!(
                !data.contains(TEST_KEY_VALUE),
                "credential leaked into artifact {:?}",
                path
            );
        }
    }
}

#[test]
fn dry_run_makes_zero_network_and_needs_no_credential() {
    // Remove the real provider env var to prove dry-run never looks it up.
    std::env::remove_var("OPENAI_API_KEY");
    let cfg = config(
        "openai",
        Scenario::StablePrefix,
        common::temp_dir("dry"),
        "dry-run-1",
    );
    let info = describe_dry_run(&cfg).unwrap();
    assert_eq!(info.provider, "openai");
    assert_eq!(info.scenario, "stable-prefix");
    assert_eq!(info.request_count, 2);
    assert!(info.estimated_tokens > 8_000);
    assert!(info.estimated_bytes > 0);
    assert_eq!(info.required_env_var, "OPENAI_API_KEY");
    assert!(info.guard_reason.is_none());
    // The dry-run report must not contain a credential value.
    let report = format!("{info:?}");
    assert!(!report.contains("secret"));
    assert!(!report.contains("Bearer"));
}

#[test]
fn openai_stable_prefix_full_pipeline_with_mock() {
    let key = test_key();
    let runs = common::temp_dir("pipeline");
    let cfg = config("openai", Scenario::StablePrefix, runs.clone(), "openai-b-1");
    let mock = MockTransport::new(vec![
        ok_response(200, &common::openai_ok(8100, 8, None)),
        ok_response(200, &common::openai_ok(8100, 8, Some(8000))),
    ]);

    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(mock.call_count(), 2);
    assert!(result.error.is_none());
    assert_eq!(result.request_count, 2);
    assert_eq!(result.conclusion, Conclusion::Match);
    assert_eq!(result.pairs.len(), 1);
    assert_eq!(
        result.pairs[0].provider_reported_cache_read_tokens,
        Some(8000)
    );
    // Observed structural reuse should be large (the ~8k prefix matched).
    assert!(result.pairs[0].observed_structural_reuse_estimated_tokens > 7_000);
    // The ratios are the comparison basis: both near 1.0 with a tiny
    // absolute difference, despite very different token bases.
    let structural = result.pairs[0].structural_reuse_ratio.unwrap();
    let provider = result.pairs[0].provider_cache_reuse_ratio.unwrap();
    assert!(structural > 0.95);
    assert!(provider > 0.95);
    assert!(result.pairs[0].reuse_ratio_difference.unwrap() < 0.10);
    assert_eq!(result.pairs[0].conclusion, Conclusion::Match);

    // All expected artifacts exist.
    let dir = runs.join("openai-b-1");
    for name in [
        "manifest.json",
        "request-01.trace.json",
        "request-02.trace.json",
        "provider-raw-usage-01.json",
        "provider-raw-usage-02.json",
        "result.json",
    ] {
        assert!(dir.join(name).exists(), "missing artifact {name}");
    }

    // No credential in any artifact.
    assert_no_credential_in_dir(&dir);

    // The trace is format v2 with 3 blocks and raw usage.
    let trace: prefixity_core::model::RequestTrace =
        serde_json::from_str(&std::fs::read_to_string(dir.join("request-01.trace.json")).unwrap())
            .unwrap();
    assert_eq!(trace.format_version, 2);
    assert_eq!(trace.blocks.len(), 3);
    assert!(trace.usage.is_some());

    // The manifest records the exact model and scenario, no credentials.
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["model"], "test-model");
    assert_eq!(manifest["scenario"], "stable-prefix");
    assert_eq!(manifest["experiment_format_version"], 4);
    assert_eq!(manifest["max_estimated_input_tokens"], 50_000);

    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn anthropic_schema_smoke_normalizes_three_categories() {
    let key = test_key();
    let runs = common::temp_dir("anthro");
    let cfg = config(
        "anthropic",
        Scenario::SchemaSmoke,
        runs.clone(),
        "anthropic-a-1",
    );
    let mock = MockTransport::single(200, &common::anthropic_ok(500, 8, 100, 7500));

    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(mock.call_count(), 1);
    assert!(result.error.is_none());
    assert_eq!(result.conclusion, Conclusion::Match);
    let request = &result.requests[0];
    assert_eq!(request.normalized_usage.total_input_tokens, Some(8100));
    assert_eq!(request.normalized_usage.fresh_input_tokens, Some(500));
    assert_eq!(request.normalized_usage.cache_read_tokens, Some(100));
    assert_eq!(request.normalized_usage.cache_write_tokens, Some(7500));
    assert_eq!(request.model_returned.as_deref(), Some("claude-test-model"));
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn deepseek_stable_prefix_plans_three_requests_and_reports_pairs() {
    let key = test_key();
    let runs = common::temp_dir("ds");
    let cfg = config(
        "deepseek",
        Scenario::StablePrefix,
        runs.clone(),
        "deepseek-b-1",
    );
    // Cache construction may take a prior completed request: A and B miss,
    // C hits.
    let mock = MockTransport::new(vec![
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(8000, 100, 8)),
    ]);
    let sleeper = RecordingSleeper::default();

    let result = execute_live_experiment(&cfg, &mock, Some(&key), &sleeper).unwrap();
    assert_eq!(mock.call_count(), 3);
    // The experimental settle delay is applied exactly once, before C.
    assert_eq!(sleeper.calls(), vec![10_000]);
    assert_eq!(result.request_count, 3);
    assert_eq!(result.pairs.len(), 2);
    // Pair (1,2): provider reports no reuse despite an identical prefix.
    assert_eq!(result.pairs[0].provider_reported_cache_read_tokens, Some(0));
    assert_eq!(result.pairs[0].conclusion, Conclusion::NoMatch);
    assert_eq!(
        result.pairs[0].provider_reported_total_input_tokens,
        Some(8100)
    );
    // Structural reuse is a large proportion of the estimated input while
    // the provider ratio is zero -> the ratio difference is large.
    assert!(result.pairs[0].structural_reuse_ratio.unwrap() > 0.95);
    assert_eq!(result.pairs[0].provider_cache_reuse_ratio.unwrap(), 0.0);
    assert!(result.pairs[0].reuse_ratio_difference.unwrap() > 0.9);
    // Pair (2,3): provider reports reuse ~= observed -> Match.
    assert_eq!(
        result.pairs[1].provider_reported_cache_read_tokens,
        Some(8000)
    );
    assert_eq!(result.pairs[1].conclusion, Conclusion::Match);
    // Overall conclusion is the last pair.
    assert_eq!(result.conclusion, Conclusion::Match);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn late_divergence_observes_large_structural_reuse() {
    let key = test_key();
    let runs = common::temp_dir("late");
    let cfg = config("openai", Scenario::LateDivergence, runs.clone(), "late-1");
    let mock = MockTransport::new(vec![
        ok_response(200, &common::openai_ok(8100, 8, None)),
        ok_response(200, &common::openai_ok(8100, 8, Some(8000))),
    ]);
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    // Late-divergence retains header + stable core (~90% split): still high,
    // but materially below the ~99.8% stable-prefix reuse.
    let reuse = result.pairs[0].observed_structural_reuse_estimated_tokens;
    assert!(
        reuse > 6_500 && reuse < 8_000,
        "late-divergence reuse {reuse} should sit around the 90/10 split"
    );
    assert_eq!(result.pairs[0].conclusion, Conclusion::Match);
    assert_eq!(result.conclusion, Conclusion::Match);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn early_divergence_observes_no_structural_reuse() {
    let key = test_key();
    let runs = common::temp_dir("early");
    let cfg = config("openai", Scenario::EarlyDivergence, runs.clone(), "early-1");
    let mock = MockTransport::new(vec![
        ok_response(200, &common::openai_ok(8100, 8, None)),
        // Provider reports zero cached tokens, matching the structural
        // prediction that an early change destroys the prefix.
        ok_response(200, &common::openai_ok(8100, 8, Some(0))),
    ]);
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(
        result.pairs[0].observed_structural_reuse_estimated_tokens,
        0
    );
    assert_eq!(result.conclusion, Conclusion::Match);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn max_request_guard_refuses_before_any_call() {
    let key = test_key();
    let runs = common::temp_dir("guard");
    let mut cfg = config("deepseek", Scenario::StablePrefix, runs.clone(), "guard-1");
    cfg.max_requests = 2; // deepseek stable-prefix plans 3
    let mock = MockTransport::new(Vec::new());
    let err = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap_err();
    assert!(matches!(err, LiveError::Guard { .. }));
    assert_eq!(mock.call_count(), 0);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn token_ceiling_refuses_before_any_call() {
    let key = test_key();
    let runs = common::temp_dir("tokens");
    let mut cfg = config("openai", Scenario::StablePrefix, runs.clone(), "tokens-1");
    cfg.max_estimated_input_tokens = 100; // the ~8k prefix far exceeds this
    let mock = MockTransport::new(Vec::new());
    let err = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap_err();
    assert!(matches!(err, LiveError::Guard { .. }));
    assert!(err.to_string().contains("max-estimated-input-tokens"));
    assert_eq!(mock.call_count(), 0);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn no_retry_on_timeout() {
    let key = test_key();
    let runs = common::temp_dir("retry");
    let cfg = config("openai", Scenario::StablePrefix, runs.clone(), "retry-1");
    let mock = MockTransport::new(vec![err_response(LiveError::Timeout)]);
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    // Exactly one call: no automatic retry.
    assert_eq!(mock.call_count(), 1);
    assert!(result.error.is_some());
    assert_eq!(result.conclusion, Conclusion::Inconclusive);
    // Partial artifacts remain reviewable.
    assert!(runs.join("retry-1").join("result.json").exists());
    assert_no_credential_in_dir(&runs.join("retry-1"));
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn http_error_stops_and_errors_never_expose_credentials() {
    let key = test_key();
    let runs = common::temp_dir("http");
    let cfg = config("openai", Scenario::StablePrefix, runs.clone(), "http-1");
    let mock = MockTransport::new(vec![err_response(LiveError::HttpStatus { status: 401 })]);
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(mock.call_count(), 1);
    let error = result.error.unwrap();
    assert!(error.contains("401"));
    assert!(!error.contains(TEST_KEY_VALUE));
    assert!(!error.contains("Bearer"));
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn redirect_status_stops_without_following_or_retrying() {
    // A 302 must stop the run: no redirect is followed, no retry occurs, and
    // the 3xx body is never parsed as provider JSON.
    let key = test_key();
    let runs = common::temp_dir("redirect");
    let cfg = config("openai", Scenario::StablePrefix, runs.clone(), "redirect-1");
    let mock = MockTransport::new(vec![
        ok_response(302, "<html>redirect</html>"),
        // A success response that must never be consumed.
        ok_response(200, &common::openai_ok(8100, 8, Some(8000))),
    ]);
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(mock.call_count(), 1);
    assert!(result.error.is_some());
    assert!(result.error.unwrap().contains("302"));
    assert_eq!(result.conclusion, Conclusion::Inconclusive);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn schema_mismatch_is_classified_as_schema_mismatch() {
    let key = test_key();
    let runs = common::temp_dir("schema");
    let cfg = config("openai", Scenario::StablePrefix, runs.clone(), "schema-1");
    let mock = MockTransport::single(200, &common::usage_only_unknown_fields());
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(result.conclusion, Conclusion::SchemaMismatch);
    assert!(result.error.unwrap().contains("does not fit"));
    assert_no_credential_in_dir(&runs.join("schema-1"));
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn missing_usage_stops_with_schema_mismatch() {
    let key = test_key();
    let runs = common::temp_dir("nousage");
    let cfg = config("openai", Scenario::SchemaSmoke, runs.clone(), "nousage-1");
    let mock = MockTransport::single(200, &common::no_usage());
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(result.conclusion, Conclusion::SchemaMismatch);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn deepseek_completion_only_is_schema_mismatch() {
    // DeepSeek raw usage that reports completion_tokens but lacks
    // prompt_cache_hit_tokens / prompt_cache_miss_tokens must be
    // SCHEMA_MISMATCH, not MATCH: the input/cache categories that define the
    // schema were never derivable.
    let key = test_key();
    let runs = common::temp_dir("dscomp");
    let cfg = config(
        "deepseek",
        Scenario::SchemaSmoke,
        runs.clone(),
        "deepseek-completion-only",
    );
    let mock = MockTransport::single(200, &common::deepseek_completion_only());
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    assert_eq!(mock.call_count(), 1);
    assert_eq!(result.conclusion, Conclusion::SchemaMismatch);
    assert!(result.error.is_some());
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn missing_credential_is_rejected_before_any_call() {
    let runs = common::temp_dir("nocred");
    let cfg = config("openai", Scenario::SchemaSmoke, runs.clone(), "nocred-1");
    let mock = MockTransport::new(Vec::new());
    let err = execute_live_experiment(&cfg, &mock, None, &NoopSleeper).unwrap_err();
    assert!(matches!(err, LiveError::MissingCredential { .. }));
    assert_eq!(mock.call_count(), 0);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn hostile_experiment_id_is_rejected_before_any_call() {
    let key = test_key();
    let runs = common::temp_dir("hostile");
    let cfg = config("openai", Scenario::SchemaSmoke, runs.clone(), "../escape");
    let mock = MockTransport::new(Vec::new());
    let err = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap_err();
    assert!(matches!(err, LiveError::InvalidArgument { .. }));
    assert_eq!(mock.call_count(), 0);
    // Nothing was created outside the runs directory.
    assert!(!runs.join("escape").exists());
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn experiment_config_debug_contains_no_credential() {
    let cfg = config(
        "openai",
        Scenario::SchemaSmoke,
        common::temp_dir("cfg"),
        "cfg-1",
    );
    let debug = format!("{cfg:?}");
    assert!(!debug.contains(TEST_KEY_VALUE));
    assert!(!debug.contains("secret"));
}

#[test]
fn requests_are_sequential_in_plan_order() {
    let key = test_key();
    let runs = common::temp_dir("seq");
    let cfg = config("deepseek", Scenario::StablePrefix, runs.clone(), "seq-1");
    let mock = MockTransport::new(vec![
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(8000, 100, 8)),
    ]);
    let _ = execute_live_experiment(&cfg, &mock, Some(&key), &NoopSleeper).unwrap();
    let calls = mock.calls();
    assert_eq!(calls.len(), 3);
    // The same allowlisted URL is used each time; bodies are distinct tails.
    assert!(calls
        .iter()
        .all(|c| c.url.starts_with("https://api.deepseek.com")));
    assert_ne!(calls[0].body, calls[1].body);
    assert_ne!(calls[1].body, calls[2].body);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn deepseek_execution_sleeps_once_for_settle_before_c() {
    let key = test_key();
    let runs = common::temp_dir("ds-settle");
    let cfg = config(
        "deepseek",
        Scenario::StablePrefix,
        runs.clone(),
        "ds-settle-1",
    );
    let mock = MockTransport::new(vec![
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(8000, 100, 8)),
    ]);
    let sleeper = RecordingSleeper::default();
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &sleeper).unwrap();
    // Exactly one sleep (before C), with the configured settle period; no
    // sleep before A/B and none after the final request.
    assert_eq!(sleeper.calls(), vec![10_000]);
    assert_eq!(result.request_count, 3);
    // Each request records its planned pre-delay.
    let delays: Vec<u64> = result
        .requests
        .iter()
        .map(|r| r.pre_request_delay_ms)
        .collect();
    assert_eq!(delays, vec![0, 0, 10_000]);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn deepseek_late_divergence_pipeline_mutates_suffix_and_reconciles() {
    let key = test_key();
    let runs = common::temp_dir("ds-late");
    let cfg = config(
        "deepseek",
        Scenario::LateDivergence,
        runs.clone(),
        "ds-late-1",
    );
    // A and B share the original late suffix; C changes it and the provider
    // reports cache reuse matching the stable core (~7200 of 8100).
    let mock = MockTransport::new(vec![
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(0, 8100, 8)),
        ok_response(200, &common::deepseek_ok(7200, 900, 8)),
    ]);
    let sleeper = RecordingSleeper::default();
    let result = execute_live_experiment(&cfg, &mock, Some(&key), &sleeper).unwrap();
    assert_eq!(mock.call_count(), 3);
    assert_eq!(sleeper.calls(), vec![10_000]);

    // Every late-divergence trace has 4 real structural blocks.
    let dir = runs.join("ds-late-1");
    let trace_c: prefixity_core::model::RequestTrace =
        serde_json::from_str(&std::fs::read_to_string(dir.join("request-03.trace.json")).unwrap())
            .unwrap();
    assert_eq!(trace_c.blocks.len(), 4);
    let ids: Vec<&str> = trace_c.blocks.iter().map(|b| b.id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["prefix-header", "synthetic-prefix", "late-suffix", "tail"]
    );
    // B -> C is the measured pair: reuse is high but below stable-prefix,
    // and reconciles to MATCH against the provider's reported cache ratio.
    let last = &result.pairs[1];
    assert_eq!(last.conclusion, Conclusion::Match);
    let structural = last.structural_reuse_ratio.unwrap();
    assert!(structural > 0.80 && structural < 0.95, "got {structural}");
    assert!(last.reuse_ratio_difference.unwrap() < 0.10);
    assert_eq!(result.conclusion, Conclusion::Match);
    std::fs::remove_dir_all(&runs).ok();
}

#[test]
fn deepseek_dry_run_exposes_settle_delay_without_sleeping() {
    let runs = common::temp_dir("ds-dry");
    let cfg = config("deepseek", Scenario::StablePrefix, runs.clone(), "ds-dry-1");
    let start = std::time::Instant::now();
    let info = describe_dry_run(&cfg).unwrap();
    // Dry-run never sleeps: it returns quickly even though C plans 10s.
    assert!(start.elapsed().as_millis() < 5_000);
    let delays: Vec<u64> = info.turns.iter().map(|t| t.pre_request_delay_ms).collect();
    assert_eq!(delays, vec![0, 0, 10_000]);
    std::fs::remove_dir_all(&runs).ok();
}
