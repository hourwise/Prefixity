//! Phase 0B experiment orchestration: planning, guardrails, execution and
//! reconciliation.
//!
//! Execution is sequential, bounded by explicit guardrails, and never
//! retries. A transport or provider error **stops** the run immediately
//! (partial artifacts remain reviewable).

use crate::artifacts;
use crate::content::{estimate_tokens, generate_prefix, header_for, tail_for};
use crate::credentials::Credentials;
use crate::error::LiveError;
use crate::manifest::{build_manifest, iso8601_utc_now, ManifestInput};
use crate::providers::{provider_from_id, LiveProvider};
use crate::result::{
    classify_pair, classify_schema_smoke, overall_conclusion, Conclusion, ExperimentResult,
    PairResult, RequestResult,
};
use crate::scenario::Scenario;
use crate::trace::{build_trace, RequestRecord};
use crate::transport::LiveHttpTransport;
use prefixity_core::compare::compare_traces;
use prefixity_core::usage::normalize_usage;
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for one live experiment (no credentials live here).
#[derive(Clone)]
pub struct ExperimentConfig {
    /// Provider id (`openai`, `anthropic`, `deepseek`).
    pub provider_id: String,
    /// Exact provider model id (never substituted).
    pub model: String,
    /// Scenario.
    pub scenario: Scenario,
    /// Seed for the deterministic synthetic prefix.
    pub seed: u64,
    /// Approximate target prefix size in tokens.
    pub target_prefix_tokens: u64,
    /// Request-count guard (default 3; hard ceiling 10 enforced by the CLI).
    pub max_requests: usize,
    /// Input-token safety ceiling.
    pub max_input_tokens: u64,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Runs directory (default `experiments/runs`).
    pub runs_dir: PathBuf,
    /// Sanitized experiment id.
    pub experiment_id: String,
    /// Optional notes.
    pub notes: Option<String>,
}

impl fmt::Debug for ExperimentConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No credential is stored here, but keep the output minimal anyway.
        f.debug_struct("ExperimentConfig")
            .field("provider_id", &self.provider_id)
            .field("model", &self.model)
            .field("scenario", &self.scenario.as_str())
            .field("seed", &self.seed)
            .field("max_requests", &self.max_requests)
            .field("max_input_tokens", &self.max_input_tokens)
            .field("experiment_id", &self.experiment_id)
            .finish()
    }
}

/// One planned request within an experiment.
#[derive(Debug, Clone)]
pub struct TurnSpec {
    /// 1-based turn number.
    pub turn: usize,
    /// Header block content.
    pub header: String,
    /// Large synthetic prefix content.
    pub prefix: String,
    /// Per-turn tail instruction.
    pub tail: String,
}

/// The resolved experiment plan (before guard checks).
#[derive(Debug)]
pub struct ExperimentPlan {
    /// The provider adapter.
    pub provider: Box<dyn LiveProvider>,
    /// The planned requests in order.
    pub turns: Vec<TurnSpec>,
    /// Total estimated bytes across all requests.
    pub estimated_bytes: u64,
    /// Total estimated tokens across all requests.
    pub estimated_tokens: u64,
    /// The artifact directory for this experiment.
    pub artifact_dir: PathBuf,
}

/// Build the plan for an experiment: resolves the provider, generates the
/// deterministic content, and computes estimates. Does not apply guards.
pub fn build_plan(config: &ExperimentConfig) -> Result<ExperimentPlan, LiveError> {
    let provider = provider_from_id(&config.provider_id)?;
    if config.model.trim().is_empty() {
        return Err(LiveError::argument("model must not be empty"));
    }
    let id = artifacts::sanitize_experiment_id(&config.experiment_id)?;

    let turn_plan = provider.plan_turns(config.scenario);
    let request_count = turn_plan.turns;
    let prefix = generate_prefix(config.seed, config.target_prefix_tokens);
    let header_base = header_for(&id, config.seed);

    let mut turns = Vec::with_capacity(request_count);
    let mut estimated_bytes = 0u64;
    let mut estimated_tokens = 0u64;
    for turn in 1..=request_count {
        // Early-divergence changes a block near the beginning (the header)
        // from the plan's configured divergence turn onward. For DeepSeek
        // this is turn C (A and B first establish the common prefix); for
        // OpenAI/Anthropic it is turn B.
        let header = if config.scenario == Scenario::EarlyDivergence
            && turn_plan
                .header_diverges_from
                .is_some_and(|first| turn >= first)
        {
            format!("{header_base} CHANGED")
        } else {
            header_base.clone()
        };
        let tail = tail_for(config.scenario, turn);
        let bytes = (header.len() + prefix.len() + tail.len()) as u64;
        let tokens = estimate_tokens(&header) + estimate_tokens(&prefix) + estimate_tokens(&tail);
        estimated_bytes = estimated_bytes.saturating_add(bytes);
        estimated_tokens = estimated_tokens.saturating_add(tokens);
        turns.push(TurnSpec {
            turn,
            header,
            prefix: prefix.clone(),
            tail,
        });
    }

    let artifact_dir = artifacts::artifact_dir(&config.runs_dir, &id)?;

    Ok(ExperimentPlan {
        provider,
        turns,
        estimated_bytes,
        estimated_tokens,
        artifact_dir,
    })
}

/// Apply the experiment guardrails. Refuses before any call is made when a
/// guard fails.
pub fn apply_guards(config: &ExperimentConfig, plan: &ExperimentPlan) -> Result<(), LiveError> {
    if plan.turns.len() > config.max_requests {
        return Err(LiveError::guard(format!(
            "scenario '{}' for provider '{}' needs {} request(s), but --max-requests is {}",
            config.scenario.as_str(),
            config.provider_id,
            plan.turns.len(),
            config.max_requests
        )));
    }
    for turn in &plan.turns {
        let tokens = estimate_tokens(&turn.header)
            + estimate_tokens(&turn.prefix)
            + estimate_tokens(&turn.tail);
        if tokens > config.max_input_tokens {
            return Err(LiveError::guard(format!(
                "request {} estimated input of {} tokens exceeds --max-input-tokens {}",
                turn.turn, tokens, config.max_input_tokens
            )));
        }
    }
    Ok(())
}

/// Information printed by a dry run. Contains no credential value.
#[derive(Debug, Clone)]
pub struct DryRunInfo {
    /// Provider id.
    pub provider: String,
    /// Model id.
    pub model: String,
    /// Scenario.
    pub scenario: String,
    /// Planned request count.
    pub request_count: usize,
    /// Estimated bytes across all requests.
    pub estimated_bytes: u64,
    /// Estimated tokens across all requests.
    pub estimated_tokens: u64,
    /// Artifact destination directory.
    pub artifact_dir: PathBuf,
    /// The environment variable that would be required.
    pub required_env_var: &'static str,
    /// `--max-requests` in force.
    pub max_requests: usize,
    /// `--max-input-tokens` in force.
    pub max_input_tokens: u64,
    /// Set when a guard would refuse the run.
    pub guard_reason: Option<String>,
}

/// Describe a dry run: builds the plan, applies the guards, and reports what
/// WOULD happen. Makes zero network requests.
pub fn describe_dry_run(config: &ExperimentConfig) -> Result<DryRunInfo, LiveError> {
    let plan = build_plan(config)?;
    let guard_reason = match apply_guards(config, &plan) {
        Ok(()) => None,
        Err(LiveError::Guard { message }) => Some(message),
        Err(other) => return Err(other),
    };
    Ok(DryRunInfo {
        provider: config.provider_id.clone(),
        model: config.model.clone(),
        scenario: config.scenario.as_str().to_string(),
        request_count: plan.turns.len(),
        estimated_bytes: plan.estimated_bytes,
        estimated_tokens: plan.estimated_tokens,
        artifact_dir: plan.artifact_dir,
        required_env_var: plan.provider.credential_env_var(),
        max_requests: config.max_requests,
        max_input_tokens: config.max_input_tokens,
        guard_reason,
    })
}

/// Execute a live experiment against a transport.
///
/// `credential` must be present (the CLI acquires it from the environment
/// only when `--execute-live` is set). All requests are sequential; there is
/// no retry. On a transport/provider error the run stops and a partial
/// `ExperimentResult` (with `error` set) is written so artifacts remain
/// reviewable.
pub fn execute_live_experiment(
    config: &ExperimentConfig,
    transport: &dyn LiveHttpTransport,
    credential: Option<&Credentials>,
) -> Result<ExperimentResult, LiveError> {
    let plan = build_plan(config)?;
    apply_guards(config, &plan)?;
    let credential = match credential {
        Some(c) => c,
        None => {
            return Err(LiveError::MissingCredential {
                env_var: plan.provider.credential_env_var(),
            })
        }
    };

    let provider = &plan.provider;
    let manifest = build_manifest(&ManifestInput {
        experiment_id: config.experiment_id.clone(),
        provider: config.provider_id.clone(),
        api_surface: provider.usage_schema().to_string(),
        endpoint: format!("{}{}", provider.base_url(), provider.endpoint_path()),
        model: config.model.clone(),
        scenario: config.scenario.as_str().to_string(),
        seed: config.seed,
        target_prefix_tokens: config.target_prefix_tokens,
        request_count: plan.turns.len(),
        notes: config.notes.clone(),
        max_requests: config.max_requests,
        max_input_tokens: config.max_input_tokens,
        timeout_ms: config.timeout_ms,
    });
    artifacts::write_json(&plan.artifact_dir.join("manifest.json"), &manifest)?;

    let mut requests: Vec<RequestResult> = Vec::new();
    let mut traces = Vec::new();
    let mut error: Option<String> = None;
    let mut schema_mismatch = false;

    for turn in &plan.turns {
        let body = match provider.build_request_body(
            &config.model,
            &turn.header,
            &turn.prefix,
            &turn.tail,
        ) {
            Ok(value) => value,
            Err(e) => {
                error = Some(e.to_string());
                break;
            }
        };
        let body_str = match serde_json::to_string(&body) {
            Ok(text) => text,
            Err(e) => {
                error = Some(format!("request serialization failed: {e}"));
                break;
            }
        };

        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert(
            provider.auth_header_name().to_string(),
            provider.auth_header_value(credential.raw()),
        );
        for (name, value) in provider.extra_headers() {
            headers.insert(name.to_string(), value.to_string());
        }

        let url = format!("{}{}", provider.base_url(), provider.endpoint_path());
        let started_at = iso8601_utc_now();
        let response = match transport.post_json(
            &url,
            &headers,
            &body_str,
            Duration::from_millis(config.timeout_ms),
        ) {
            Ok(response) => response,
            Err(e) => {
                // STOP: no retry, no further requests.
                error = Some(format!("request {} failed: {e}", turn.turn));
                break;
            }
        };

        let response_body: serde_json::Value = match serde_json::from_str(&response.body) {
            Ok(value) => value,
            Err(e) => {
                error = Some(format!(
                    "request {} returned an unparseable response body: {e}",
                    turn.turn
                ));
                break;
            }
        };

        let raw_usage = match provider.extract_raw_usage(&response_body) {
            Some(usage) => usage,
            None => {
                error = Some(format!(
                    "request {} response carries no usage object (schema mismatch)",
                    turn.turn
                ));
                schema_mismatch = true;
                break;
            }
        };
        let normalized = normalize_usage(&raw_usage);
        if normalized.normalization_source == "unknown-schema"
            || (normalized.total_input_tokens.is_none()
                && normalized.fresh_input_tokens.is_none()
                && normalized.cache_read_tokens.is_none()
                && normalized.cache_write_tokens.is_none()
                && normalized.output_tokens.is_none())
        {
            error = Some(format!(
                "request {} usage does not fit the '{}' normalizer: {}",
                turn.turn, raw_usage.provider_schema, normalized.explanation
            ));
            schema_mismatch = true;
            break;
        }
        // Schema-smoke additionally requires the endpoint schema's defining
        // fields to be derivable (e.g. DeepSeek needs the input/cache
        // categories; completion/output tokens alone are not evidence).
        if config.scenario == Scenario::SchemaSmoke
            && classify_schema_smoke(&normalized) == Conclusion::SchemaMismatch
        {
            error = Some(format!(
                "request {} schema-smoke failed: required fields for schema '{}' were not derivable: {}",
                turn.turn, raw_usage.provider_schema, normalized.explanation
            ));
            schema_mismatch = true;
            break;
        }

        let record = RequestRecord {
            turn: turn.turn,
            header: turn.header.clone(),
            prefix: turn.prefix.clone(),
            tail: turn.tail.clone(),
            raw_usage: raw_usage.clone(),
            provider_request_id: provider.request_id(&response_body),
            http_status: response.status,
            started_at,
            time_to_headers_ms: response.time_to_headers_ms,
            time_to_first_body_byte_ms: response.time_to_first_body_byte_ms,
            total_ms: response.total_ms,
        };
        let trace = build_trace(
            provider.as_ref(),
            &config.model,
            &config.experiment_id,
            &record,
        );

        let file_base = format!("{:02}", turn.turn);
        let trace_file = format!("request-{file_base}.trace.json");
        let raw_usage_file = format!("provider-raw-usage-{file_base}.json");
        artifacts::write_json(&plan.artifact_dir.join(&trace_file), &trace)?;
        artifacts::write_json(&plan.artifact_dir.join(&raw_usage_file), &raw_usage)?;

        traces.push(trace);
        requests.push(RequestResult {
            turn: turn.turn,
            http_status: response.status,
            provider_request_id: provider.request_id(&response_body),
            model_returned: response_body
                .get("model")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string),
            started_at: record.started_at,
            time_to_headers_ms: response.time_to_headers_ms,
            time_to_first_body_byte_ms: response.time_to_first_body_byte_ms,
            total_ms: response.total_ms,
            generated_bytes: (turn.header.len() + turn.prefix.len() + turn.tail.len()) as u64,
            prefixity_estimated_tokens: estimate_tokens(&turn.header)
                + estimate_tokens(&turn.prefix)
                + estimate_tokens(&turn.tail),
            normalized_usage: normalized.clone(),
            normalizer_warnings: vec![normalized.explanation.clone()],
            trace_file,
            raw_usage_file,
        });
    }

    // Reconciliation: compare consecutive traces and provider-reported usage.
    let mut pairs: Vec<PairResult> = Vec::new();
    for i in 0..traces.len().saturating_sub(1) {
        let comparison = compare_traces(&traces[i], &traces[i + 1], None).map_err(|e| {
            LiveError::InvalidResponse {
                message: format!("trace comparison failed: {e}"),
            }
        })?;
        let observed = comparison.observed_reusable_prefix_tokens;
        let reported = requests[i + 1].normalized_usage.cache_read_tokens;
        let difference = reported.map(|r| r as i64 - observed as i64);
        let conclusion = classify_pair(reported, observed);
        let note = match reported {
            Some(reported) => format!(
                "observed structural reuse {observed} (estimated tokens) vs provider-reported cache read {reported}"
            ),
            None => "provider reported no cache-read figure".to_string(),
        };
        pairs.push(PairResult {
            request_a: requests[i].turn,
            request_b: requests[i + 1].turn,
            observed_structural_reuse_tokens: observed,
            provider_reported_cache_read_tokens: reported,
            difference,
            conclusion,
            note,
        });
    }

    let schema_smoke = if config.scenario == Scenario::SchemaSmoke {
        requests
            .first()
            .map(|r| classify_schema_smoke(&r.normalized_usage))
    } else {
        None
    };
    let conclusion = if schema_mismatch {
        Conclusion::SchemaMismatch
    } else if error.is_some() {
        // An aborted run cannot claim a match.
        Conclusion::Inconclusive
    } else {
        overall_conclusion(schema_smoke, &pairs)
    };

    let summary = build_summary(config, &requests, &pairs, conclusion, error.as_deref());
    let result = ExperimentResult {
        provider: config.provider_id.clone(),
        model: config.model.clone(),
        scenario: config.scenario.as_str().to_string(),
        request_count: requests.len(),
        estimated_input_tokens: plan.estimated_tokens,
        requests,
        pairs,
        conclusion,
        summary,
        error,
    };
    artifacts::write_json(&plan.artifact_dir.join("result.json"), &result)?;
    Ok(result)
}

/// Find the 0-based index of a trace within `requests` by matching turn.
fn build_summary(
    config: &ExperimentConfig,
    requests: &[RequestResult],
    pairs: &[PairResult],
    conclusion: Conclusion,
    error: Option<&str>,
) -> String {
    let mut lines = Vec::new();
    lines.push(format!(
        "experiment {} ({}) provider={} model={} requests={}",
        config.experiment_id,
        config.scenario.as_str(),
        config.provider_id,
        config.model,
        requests.len()
    ));
    for request in requests {
        lines.push(format!(
            "  request {}: http={} total_ms={} estimated_tokens={} normalized_total_input={} cache_read={}",
            request.turn,
            request.http_status,
            request.total_ms,
            request.prefixity_estimated_tokens,
            request.normalized_usage.total_input_tokens.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()),
            request.normalized_usage.cache_read_tokens.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()),
        ));
    }
    for pair in pairs {
        lines.push(format!(
            "  pair {}->{}: observed_reuse={} reported_cache_read={} => {}",
            pair.request_a,
            pair.request_b,
            pair.observed_structural_reuse_tokens,
            pair.provider_reported_cache_read_tokens
                .map(|v| v.to_string())
                .unwrap_or_else(|| "?".to_string()),
            pair.conclusion.as_str(),
        ));
    }
    if let Some(err) = error {
        lines.push(format!("  aborted: {err}"));
    }
    lines.push(format!("conclusion: {}", conclusion.as_str()));
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(provider: &str, scenario: Scenario) -> ExperimentConfig {
        ExperimentConfig {
            provider_id: provider.to_string(),
            model: "deepseek-v4-flash".to_string(),
            scenario,
            seed: 42,
            target_prefix_tokens: 8000,
            max_requests: 10,
            max_input_tokens: 50_000,
            timeout_ms: 5_000,
            runs_dir: std::env::temp_dir().join("prefixity-live-plan-test"),
            experiment_id: "plan-test".to_string(),
            notes: None,
        }
    }

    #[test]
    fn deepseek_stable_prefix_plan_is_a_b_c_with_common_prefix() {
        let plan = build_plan(&test_config("deepseek", Scenario::StablePrefix)).unwrap();
        assert_eq!(plan.turns.len(), 3);
        // A, B, C share the same header and prefix; only the tails differ.
        assert_eq!(plan.turns[0].header, plan.turns[1].header);
        assert_eq!(plan.turns[1].header, plan.turns[2].header);
        assert_eq!(plan.turns[0].prefix, plan.turns[1].prefix);
        assert_eq!(plan.turns[1].prefix, plan.turns[2].prefix);
        assert_ne!(plan.turns[0].tail, plan.turns[1].tail);
        assert_ne!(plan.turns[1].tail, plan.turns[2].tail);
    }

    #[test]
    fn deepseek_late_divergence_plan_changes_tail_at_c_only() {
        let plan = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        assert_eq!(plan.turns.len(), 3);
        // A/B/C keep the same header and prefix; C uses a changed tail.
        assert_eq!(plan.turns[0].header, plan.turns[1].header);
        assert_eq!(plan.turns[1].header, plan.turns[2].header);
        assert_eq!(plan.turns[0].prefix, plan.turns[2].prefix);
        assert_ne!(plan.turns[1].tail, plan.turns[2].tail);
    }

    #[test]
    fn deepseek_early_divergence_keeps_header_stable_until_c() {
        let plan = build_plan(&test_config("deepseek", Scenario::EarlyDivergence)).unwrap();
        assert_eq!(plan.turns.len(), 3);
        // The early header must NOT change on turn B: A and B first
        // establish the common prefix. Only C diverges the header.
        assert_eq!(plan.turns[0].header, plan.turns[1].header);
        assert_ne!(plan.turns[1].header, plan.turns[2].header);
        assert!(plan.turns[2].header.ends_with("CHANGED"));
        assert!(!plan.turns[1].header.ends_with("CHANGED"));
        // The prefix is identical across all three requests.
        assert_eq!(plan.turns[0].prefix, plan.turns[2].prefix);
    }

    #[test]
    fn openai_early_divergence_still_changes_header_at_b() {
        let plan = build_plan(&test_config("openai", Scenario::EarlyDivergence)).unwrap();
        assert_eq!(plan.turns.len(), 2);
        assert_ne!(plan.turns[0].header, plan.turns[1].header);
        assert!(plan.turns[1].header.ends_with("CHANGED"));
    }

    #[test]
    fn openai_and_anthropic_non_smoke_scenarios_stay_two_requests() {
        for provider in ["openai", "anthropic"] {
            let plan = build_plan(&test_config(provider, Scenario::StablePrefix)).unwrap();
            assert_eq!(plan.turns.len(), 2);
            let plan = build_plan(&test_config(provider, Scenario::LateDivergence)).unwrap();
            assert_eq!(plan.turns.len(), 2);
        }
    }
}
