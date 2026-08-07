//! Phase 0B experiment orchestration: planning, guardrails, execution and
//! reconciliation.
//!
//! Execution is sequential, bounded by explicit guardrails, and never
//! retries. A transport or provider error **stops** the run immediately
//! (partial artifacts remain reviewable).

use crate::artifacts;
use crate::content::{
    estimate_tokens, generate_changed_late_suffix, generate_changed_late_suffix_variant2,
    generate_late_divergence_prefix, generate_prefix, header_for, tail_for, SuffixVariant,
    LATE_DIVERGENCE_CORE_PERCENT, LATE_DIVERGENCE_SUFFIX_PERCENT,
};
use crate::credentials::Credentials;
use crate::error::LiveError;
use crate::manifest::{build_manifest, iso8601_utc_now, ManifestInput};
use crate::providers::{provider_from_id, LiveProvider};
use crate::result::{
    classify_pair, classify_schema_smoke, overall_conclusion, reuse_ratio, Conclusion,
    ExperimentResult, PairResult, PairRole, RequestResult,
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
    /// Conservative **local Prefixity-estimate** input ceiling. This is NOT a
    /// provider billing/tokenizer guarantee.
    pub max_estimated_input_tokens: u64,
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
            .field(
                "max_estimated_input_tokens",
                &self.max_estimated_input_tokens,
            )
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
    /// Large synthetic prefix (stable core) content.
    pub prefix: String,
    /// Late mutable suffix content, if any (`late-divergence` only).
    pub suffix: Option<String>,
    /// Which deterministic late-suffix variant this turn carries
    /// (`SuffixVariant::None` for scenarios without a late suffix).
    pub suffix_variant: SuffixVariant,
    /// Per-turn tail instruction.
    pub tail: String,
    /// Pre-request delay (ms) applied before this request (experimental
    /// settle control; 0 for most requests).
    pub pre_request_delay_ms: u64,
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
    // Late-divergence splits the prefix into a stable core (90%) and a late
    // mutable suffix (10%); other scenarios keep a single prefix block.
    let (prefix, original_suffix) = if config.scenario == Scenario::LateDivergence {
        generate_late_divergence_prefix(config.seed, config.target_prefix_tokens)
    } else {
        (
            generate_prefix(config.seed, config.target_prefix_tokens),
            String::new(),
        )
    };
    // Two distinct changed suffix variants: variant 1 (first mutation turn,
    // e.g. DeepSeek C) and variant 2 (later mutation turns, e.g. DeepSeek D).
    // Variant 2 differs from BOTH the original and variant 1, so D cannot
    // hit C's complete request by re-sending identical suffix content.
    let changed_suffix = if config.scenario == Scenario::LateDivergence {
        generate_changed_late_suffix(config.seed, config.target_prefix_tokens)
    } else {
        String::new()
    };
    let changed_suffix_2 = if config.scenario == Scenario::LateDivergence {
        generate_changed_late_suffix_variant2(config.seed, config.target_prefix_tokens)
    } else {
        String::new()
    };
    let header_base = header_for(&id, config.seed);
    let first_mutation_turn = turn_plan.late_mutation_turn();

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
        // Late-divergence: the ORIGINAL suffix before the mutation turns,
        // changed variant 1 on the first mutation turn, and changed variant
        // 2 on subsequent mutation turns — all deterministic.
        let (suffix, suffix_variant) = if config.scenario == Scenario::LateDivergence {
            if turn_plan.late_suffix_mutates(turn) {
                if first_mutation_turn == Some(turn) {
                    (Some(changed_suffix.clone()), SuffixVariant::Variant1)
                } else {
                    (Some(changed_suffix_2.clone()), SuffixVariant::Variant2)
                }
            } else {
                (Some(original_suffix.clone()), SuffixVariant::Original)
            }
        } else {
            (None, SuffixVariant::None)
        };
        let suffix_text = suffix.as_deref().unwrap_or("");
        let bytes = (header.len() + prefix.len() + suffix_text.len() + tail.len()) as u64;
        let tokens = estimate_tokens(&header)
            + estimate_tokens(&prefix)
            + estimate_tokens(suffix_text)
            + estimate_tokens(&tail);
        estimated_bytes = estimated_bytes.saturating_add(bytes);
        estimated_tokens = estimated_tokens.saturating_add(tokens);
        turns.push(TurnSpec {
            turn,
            header,
            prefix: prefix.clone(),
            suffix,
            suffix_variant,
            tail,
            pre_request_delay_ms: turn_plan.pre_request_delay_ms(turn),
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
        if tokens > config.max_estimated_input_tokens {
            return Err(LiveError::guard(format!(
                "request {} estimated input of {} tokens exceeds --max-estimated-input-tokens {} (a conservative local Prefixity estimate, not a provider guarantee)",
                turn.turn, tokens, config.max_estimated_input_tokens
            )));
        }
    }
    Ok(())
}

/// A minimal delay executor so execution can apply experimental settle
/// delays without coupling unit/integration tests to real multi-second
/// sleeps.
pub trait Sleep: Send + Sync {
    /// Block for at least `millis` milliseconds on the current thread.
    fn sleep(&self, millis: u64);
}

/// The real implementation: blocks the current thread for the duration.
pub struct StdThreadSleeper;

impl Sleep for StdThreadSleeper {
    fn sleep(&self, millis: u64) {
        std::thread::sleep(Duration::from_millis(millis));
    }
}

/// Late-divergence experimental split, exposed so dry runs and manifests are
/// auditable.
#[derive(Debug, Clone)]
pub struct LateDivergenceInfo {
    /// Percentage of the prefix kept as the stable core.
    pub core_percent: u64,
    /// Percentage of the prefix in the late mutable suffix.
    pub suffix_percent: u64,
    /// The 1-based turns (in order) on which the late suffix mutates (e.g.
    /// `[3, 4]` for the four-turn DeepSeek late plan, `[2]` for
    /// OpenAI/Anthropic).
    pub mutation_turns: Vec<usize>,
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
    /// The planned request descriptors (turn, label, pre-delay).
    pub turns: Vec<TurnSpec>,
    /// Late-divergence split, when the scenario is `late-divergence`.
    pub late_divergence: Option<LateDivergenceInfo>,
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
    /// `--max-estimated-input-tokens` in force (conservative local Prefixity
    /// estimate; not a provider tokenizer/billing guarantee).
    pub max_estimated_input_tokens: u64,
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
    let late_divergence = if config.scenario == Scenario::LateDivergence {
        let turn_plan = plan.provider.plan_turns(config.scenario);
        Some(LateDivergenceInfo {
            core_percent: LATE_DIVERGENCE_CORE_PERCENT,
            suffix_percent: LATE_DIVERGENCE_SUFFIX_PERCENT,
            mutation_turns: turn_plan.late_mutation_turns(),
        })
    } else {
        None
    };
    Ok(DryRunInfo {
        provider: config.provider_id.clone(),
        model: config.model.clone(),
        scenario: config.scenario.as_str().to_string(),
        request_count: plan.turns.len(),
        turns: plan.turns.clone(),
        late_divergence,
        estimated_bytes: plan.estimated_bytes,
        estimated_tokens: plan.estimated_tokens,
        artifact_dir: plan.artifact_dir,
        required_env_var: plan.provider.credential_env_var(),
        max_requests: config.max_requests,
        max_estimated_input_tokens: config.max_estimated_input_tokens,
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
    sleeper: &dyn Sleep,
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
        request_pre_delays_ms: plan.turns.iter().map(|t| t.pre_request_delay_ms).collect(),
        late_divergence_core_percent: (config.scenario == Scenario::LateDivergence)
            .then_some(LATE_DIVERGENCE_CORE_PERCENT),
        late_divergence_suffix_percent: (config.scenario == Scenario::LateDivergence)
            .then_some(LATE_DIVERGENCE_SUFFIX_PERCENT),
        late_suffix_mutation_turns: (config.scenario == Scenario::LateDivergence)
            .then(|| provider.plan_turns(config.scenario).late_mutation_turns()),
        notes: config.notes.clone(),
        max_requests: config.max_requests,
        max_estimated_input_tokens: config.max_estimated_input_tokens,
        timeout_ms: config.timeout_ms,
    });
    artifacts::write_json(&plan.artifact_dir.join("manifest.json"), &manifest)?;

    let mut requests: Vec<RequestResult> = Vec::new();
    let mut traces = Vec::new();
    let mut error: Option<String> = None;
    let mut schema_mismatch = false;

    for turn in &plan.turns {
        // Experimental settle delay before this request (e.g. DeepSeek's 10s
        // period before C, after B has fully completed, so best-effort async
        // cache persistence can finish). No delay is applied after the final
        // request.
        if turn.pre_request_delay_ms > 0 {
            sleeper.sleep(turn.pre_request_delay_ms);
        }
        let body = match provider.build_request_body(
            &config.model,
            &turn.header,
            &turn.prefix,
            turn.suffix.as_deref(),
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
            suffix: turn.suffix.clone(),
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
            pre_request_delay_ms: turn.pre_request_delay_ms,
            generated_bytes: (turn.header.len()
                + turn.prefix.len()
                + turn.suffix.as_deref().unwrap_or("").len()
                + turn.tail.len()) as u64,
            prefixity_estimated_tokens: estimate_tokens(&turn.header)
                + estimate_tokens(&turn.prefix)
                + estimate_tokens(turn.suffix.as_deref().unwrap_or(""))
                + estimate_tokens(&turn.tail),
            normalized_usage: normalized.clone(),
            normalizer_warnings: vec![normalized.explanation.clone()],
            trace_file,
            raw_usage_file,
        });
    }

    // Reconciliation: compare consecutive traces and provider-reported usage
    // through PROPORTIONS. Absolute token counts from different tokenizers
    // (Prefixity chars/4 vs provider tokens) are never subtracted. Each pair
    // is labelled diagnostic or primary: the final pair is the PRIMARY
    // measurement pair and drives the overall conclusion; earlier pairs
    // (priming / cache-availability / first-divergence) are retained as
    // diagnostic evidence.
    let mut pairs: Vec<PairResult> = Vec::new();
    for i in 0..traces.len().saturating_sub(1) {
        let comparison = compare_traces(&traces[i], &traces[i + 1], None).map_err(|e| {
            LiveError::InvalidResponse {
                message: format!("trace comparison failed: {e}"),
            }
        })?;
        let observed = comparison.observed_reusable_prefix_tokens;
        let request_b = &requests[i + 1];
        let prefixity_input = request_b.prefixity_estimated_tokens;
        let provider_cache_read = request_b.normalized_usage.cache_read_tokens;
        let provider_total = request_b.normalized_usage.total_input_tokens;

        // Each proportion is relative to its own denominator.
        let structural_ratio = reuse_ratio(observed, prefixity_input);
        let provider_ratio = match (provider_cache_read, provider_total) {
            (Some(read), Some(total)) => reuse_ratio(read, total),
            _ => None,
        };
        let reuse_ratio_difference = match (structural_ratio, provider_ratio) {
            (Some(s), Some(p)) => Some((s - p).abs()),
            _ => None,
        };
        let conclusion = classify_pair(structural_ratio, provider_ratio);
        let note = match (provider_cache_read, provider_total) {
            (Some(read), Some(_)) => format!(
                "structural reuse POTENTIAL {} of Prefixity-estimated request input ({} estimated tokens) vs REALIZED provider cache reuse {} of provider-reported input ({} provider tokens); realization gap {} => {}",
                pct(structural_ratio),
                observed,
                pct(provider_ratio),
                read,
                ratio_diff_str(reuse_ratio_difference),
                conclusion.as_str(),
            ),
            _ => "provider reported no cache-read or total-input figure".to_string(),
        };
        pairs.push(PairResult {
            request_a: requests[i].turn,
            request_b: request_b.turn,
            observed_structural_reuse_estimated_tokens: observed,
            request_b_prefixity_estimated_input_tokens: prefixity_input,
            structural_reuse_ratio: structural_ratio,
            provider_reported_cache_read_tokens: provider_cache_read,
            provider_reported_total_input_tokens: provider_total,
            provider_cache_reuse_ratio: provider_ratio,
            reuse_ratio_difference,
            role: if i == traces.len().saturating_sub(2) {
                PairRole::Primary
            } else {
                PairRole::Diagnostic
            },
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
            "  request {}: http={} total_ms={} estimated_tokens={} pre_delay_ms={} normalized_total_input={} cache_read={}",
            request.turn,
            request.http_status,
            request.total_ms,
            request.prefixity_estimated_tokens,
            request.pre_request_delay_ms,
            request.normalized_usage.total_input_tokens.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()),
            request.normalized_usage.cache_read_tokens.map(|v| v.to_string()).unwrap_or_else(|| "?".to_string()),
        ));
    }
    for pair in pairs {
        lines.push(format!(
            "  pair {}->{} [{}]: structural_reuse_ratio={} ({} estimated tokens) provider_cache_reuse_ratio={} ({} provider tokens) => {}",
            pair.request_a,
            pair.request_b,
            pair.role.label(),
            pct(pair.structural_reuse_ratio),
            pair.observed_structural_reuse_estimated_tokens,
            pct(pair.provider_cache_reuse_ratio),
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

/// Format a reuse proportion as a percentage for human-readable output.
fn pct(ratio: Option<f64>) -> String {
    match ratio {
        Some(r) => format!("{:.1}%", r * 100.0),
        None => "unavailable".to_string(),
    }
}

/// Format the reuse-ratio difference for human-readable output.
fn ratio_diff_str(diff: Option<f64>) -> String {
    match diff {
        Some(d) => format!("{:.3}", d),
        None => "unavailable".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::DeepSeekProvider;
    use prefixity_core::compare::compare_traces;
    use prefixity_core::model::RawUsage;
    use std::collections::BTreeMap;

    fn test_config(provider: &str, scenario: Scenario) -> ExperimentConfig {
        ExperimentConfig {
            provider_id: provider.to_string(),
            model: "deepseek-v4-flash".to_string(),
            scenario,
            seed: 42,
            target_prefix_tokens: 8000,
            max_requests: 10,
            max_estimated_input_tokens: 50_000,
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
        // StablePrefix has no late mutable suffix.
        assert!(plan.turns.iter().all(|t| t.suffix.is_none()));
    }

    #[test]
    fn deepseek_late_divergence_plan_is_four_turns_a_b_c_d() {
        let plan = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        assert_eq!(plan.turns.len(), 4);
        // A and B: header, core and ORIGINAL suffix identical; tails distinct.
        assert_eq!(plan.turns[0].header, plan.turns[1].header);
        assert_eq!(plan.turns[0].prefix, plan.turns[1].prefix);
        assert_eq!(plan.turns[0].suffix, plan.turns[1].suffix);
        assert_ne!(plan.turns[0].tail, plan.turns[1].tail);
        // C: header and core unchanged, late suffix CHANGED (variant 1).
        assert_eq!(plan.turns[1].header, plan.turns[2].header);
        assert_eq!(plan.turns[1].prefix, plan.turns[2].prefix);
        assert_ne!(plan.turns[1].suffix, plan.turns[2].suffix);
        // D: header and core unchanged, late suffix CHANGED (variant 2),
        // different from BOTH the original (A/B) and C's variant 1.
        assert_eq!(plan.turns[1].header, plan.turns[3].header);
        assert_eq!(plan.turns[1].prefix, plan.turns[3].prefix);
        assert_ne!(plan.turns[0].suffix, plan.turns[3].suffix);
        assert_ne!(plan.turns[2].suffix, plan.turns[3].suffix);
        // Tails are all distinct.
        assert_ne!(plan.turns[2].tail, plan.turns[3].tail);
        // Every late-divergence turn carries a suffix.
        assert!(plan.turns.iter().all(|t| t.suffix.is_some()));
    }

    #[test]
    fn deepseek_late_divergence_suffix_variants_are_assigned_per_turn() {
        use crate::content::SuffixVariant;
        let plan = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        let variants: Vec<SuffixVariant> = plan.turns.iter().map(|t| t.suffix_variant).collect();
        assert_eq!(
            variants,
            vec![
                SuffixVariant::Original,
                SuffixVariant::Original,
                SuffixVariant::Variant1,
                SuffixVariant::Variant2,
            ]
        );
    }

    #[test]
    fn deepseek_late_divergence_delay_plan_is_zero_zero_zero_ten_seconds() {
        let plan = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        assert_eq!(plan.turns.len(), 4);
        let delays: Vec<u64> = plan.turns.iter().map(|t| t.pre_request_delay_ms).collect();
        // The important settle period is AFTER C (which first exposes the
        // late-divergence common-prefix boundary) and BEFORE D.
        assert_eq!(delays, vec![0, 0, 0, 10_000]);
    }

    #[test]
    fn late_divergence_is_not_equivalent_to_stable_prefix() {
        let stable = build_plan(&test_config("deepseek", Scenario::StablePrefix)).unwrap();
        let late = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        // StablePrefix has no late suffix; LateDivergence does.
        assert!(stable.turns.iter().all(|t| t.suffix.is_none()));
        assert!(late.turns.iter().all(|t| t.suffix.is_some()));
        // The prefix content differs too: stable generates the full target,
        // late generates the 90% core.
        assert_ne!(stable.turns[0].prefix, late.turns[0].prefix);
        // The plans are not equal.
        assert_ne!(
            DeepSeekProvider.plan_turns(Scenario::StablePrefix),
            DeepSeekProvider.plan_turns(Scenario::LateDivergence)
        );
    }

    #[test]
    fn openai_and_anthropic_late_divergence_mutate_suffix_at_b() {
        for provider in ["openai", "anthropic"] {
            let plan = build_plan(&test_config(provider, Scenario::LateDivergence)).unwrap();
            assert_eq!(plan.turns.len(), 2);
            // A: original suffix; B: changed suffix; header/core identical.
            assert_eq!(plan.turns[0].header, plan.turns[1].header);
            assert_eq!(plan.turns[0].prefix, plan.turns[1].prefix);
            assert!(plan.turns[0].suffix.is_some() && plan.turns[1].suffix.is_some());
            assert_ne!(plan.turns[0].suffix, plan.turns[1].suffix);
        }
    }

    #[test]
    fn late_divergence_structural_compare_stops_at_the_suffix() {
        let config = test_config("deepseek", Scenario::LateDivergence);
        let plan = build_plan(&config).unwrap();
        let provider = plan.provider.as_ref();
        let model = &config.model;
        let experiment_id = &config.experiment_id;
        let trace_b = build_trace(provider, model, experiment_id, &record_for(&plan.turns[1]));
        let trace_c = build_trace(provider, model, experiment_id, &record_for(&plan.turns[2]));
        let comparison = compare_traces(&trace_b, &trace_c, None).unwrap();
        // The first meaningful divergence is the late mutable suffix.
        assert_eq!(
            comparison.first_divergence.as_ref().map(|d| d.position),
            Some(2)
        );
        // Reuse retains header + stable core only.
        let expected_reuse =
            estimate_tokens(&plan.turns[1].header) + estimate_tokens(&plan.turns[1].prefix);
        assert_eq!(comparison.observed_reusable_prefix_tokens, expected_reuse);
        let total = estimate_tokens(&plan.turns[1].header)
            + estimate_tokens(&plan.turns[1].prefix)
            + estimate_tokens(plan.turns[1].suffix.as_deref().unwrap())
            + estimate_tokens(&plan.turns[1].tail);
        let ratio = expected_reuse as f64 / total as f64;
        // Materially below stable-prefix (~0.998) but still high, broadly
        // consistent with the experimental ~90/10 late split.
        assert!(
            ratio > 0.80 && ratio < 0.95,
            "late-divergence reuse ratio {ratio} should sit around the 90/10 split"
        );
    }

    #[test]
    fn late_divergence_structural_compare_shows_lower_reuse_than_stable_prefix() {
        let stable_config = test_config("deepseek", Scenario::StablePrefix);
        let late_config = test_config("deepseek", Scenario::LateDivergence);
        let stable = build_plan(&stable_config).unwrap();
        let late = build_plan(&late_config).unwrap();
        let stable_provider = stable.provider.as_ref();
        let late_provider = late.provider.as_ref();

        // StablePrefix: reuse through header + full prefix (B -> C).
        let sb = build_trace(
            stable_provider,
            &stable_config.model,
            &stable_config.experiment_id,
            &record_for(&stable.turns[1]),
        );
        let sc = build_trace(
            stable_provider,
            &stable_config.model,
            &stable_config.experiment_id,
            &record_for(&stable.turns[2]),
        );
        let stable_compare = compare_traces(&sb, &sc, None).unwrap();
        let stable_total = estimate_tokens(&stable.turns[1].header)
            + estimate_tokens(&stable.turns[1].prefix)
            + estimate_tokens(&stable.turns[1].tail);
        let stable_ratio =
            stable_compare.observed_reusable_prefix_tokens as f64 / stable_total as f64;

        // LateDivergence: reuse through header + core only.
        let lb = build_trace(
            late_provider,
            &late_config.model,
            &late_config.experiment_id,
            &record_for(&late.turns[1]),
        );
        let lc = build_trace(
            late_provider,
            &late_config.model,
            &late_config.experiment_id,
            &record_for(&late.turns[2]),
        );
        let late_compare = compare_traces(&lb, &lc, None).unwrap();
        let late_total = estimate_tokens(&late.turns[1].header)
            + estimate_tokens(&late.turns[1].prefix)
            + estimate_tokens(late.turns[1].suffix.as_deref().unwrap())
            + estimate_tokens(&late.turns[1].tail);
        let late_ratio = late_compare.observed_reusable_prefix_tokens as f64 / late_total as f64;

        assert!(stable_ratio > 0.99, "stable ratio {stable_ratio}");
        assert!(
            late_ratio < stable_ratio - 0.03,
            "late ratio {late_ratio} must be materially below stable {stable_ratio}"
        );
        assert!(late_ratio > 0.80, "late ratio {late_ratio}");
    }

    #[test]
    fn deepseek_late_c_to_d_first_structural_divergence_is_late_suffix() {
        let config = test_config("deepseek", Scenario::LateDivergence);
        let plan = build_plan(&config).unwrap();
        let provider = plan.provider.as_ref();
        let model = &config.model;
        let experiment_id = &config.experiment_id;
        // C and D share header and stable core; only the late suffix content
        // (variant 1 vs variant 2) and the tail differ.
        assert_eq!(plan.turns[2].header, plan.turns[3].header);
        assert_eq!(plan.turns[2].prefix, plan.turns[3].prefix);
        assert_ne!(plan.turns[2].suffix, plan.turns[3].suffix);
        assert_ne!(plan.turns[2].tail, plan.turns[3].tail);
        let trace_c = build_trace(provider, model, experiment_id, &record_for(&plan.turns[2]));
        let trace_d = build_trace(provider, model, experiment_id, &record_for(&plan.turns[3]));
        let comparison = compare_traces(&trace_c, &trace_d, None).unwrap();
        // The first (and only) structural divergence is the late-suffix block
        // (position 2); header and core are reused.
        let divergence = comparison.first_divergence.as_ref().expect("a divergence");
        assert_eq!(divergence.position, 2);
        assert_eq!(divergence.current_block_id, "late-suffix");
        let expected_reuse =
            estimate_tokens(&plan.turns[2].header) + estimate_tokens(&plan.turns[2].prefix);
        assert_eq!(comparison.observed_reusable_prefix_tokens, expected_reuse);
    }

    #[test]
    fn deepseek_late_b_to_c_and_c_to_d_reuse_are_high_but_below_stable() {
        // Both first-divergence pairs (B -> C and C -> D) observe reuse
        // through header + stable core only: high but materially below the
        // ~0.998 stable-prefix reuse. Ratios are not hard-coded to exact
        // values.
        let stable_config = test_config("deepseek", Scenario::StablePrefix);
        let late_config = test_config("deepseek", Scenario::LateDivergence);
        let stable = build_plan(&stable_config).unwrap();
        let late = build_plan(&late_config).unwrap();
        let stable_provider = stable.provider.as_ref();
        let late_provider = late.provider.as_ref();

        // StablePrefix B -> C ratio (baseline ~0.998).
        let sb = build_trace(
            stable_provider,
            &stable_config.model,
            &stable_config.experiment_id,
            &record_for(&stable.turns[1]),
        );
        let sc = build_trace(
            stable_provider,
            &stable_config.model,
            &stable_config.experiment_id,
            &record_for(&stable.turns[2]),
        );
        let stable_compare = compare_traces(&sb, &sc, None).unwrap();
        let stable_total = estimate_tokens(&stable.turns[1].header)
            + estimate_tokens(&stable.turns[1].prefix)
            + estimate_tokens(&stable.turns[1].tail);
        let stable_ratio =
            stable_compare.observed_reusable_prefix_tokens as f64 / stable_total as f64;

        // LateDivergence B -> C and C -> D ratios.
        let lb = build_trace(
            late_provider,
            &late_config.model,
            &late_config.experiment_id,
            &record_for(&late.turns[1]),
        );
        let lc = build_trace(
            late_provider,
            &late_config.model,
            &late_config.experiment_id,
            &record_for(&late.turns[2]),
        );
        let ld = build_trace(
            late_provider,
            &late_config.model,
            &late_config.experiment_id,
            &record_for(&late.turns[3]),
        );
        let bc = compare_traces(&lb, &lc, None).unwrap();
        let cd = compare_traces(&lc, &ld, None).unwrap();
        let late_total = estimate_tokens(&late.turns[1].header)
            + estimate_tokens(&late.turns[1].prefix)
            + estimate_tokens(late.turns[1].suffix.as_deref().unwrap())
            + estimate_tokens(&late.turns[1].tail);
        let bc_ratio = bc.observed_reusable_prefix_tokens as f64 / late_total as f64;
        let cd_ratio = cd.observed_reusable_prefix_tokens as f64 / late_total as f64;

        assert!(stable_ratio > 0.99, "stable ratio {stable_ratio}");
        for (name, ratio) in [("B->C", bc_ratio), ("C->D", cd_ratio)] {
            assert!(
                ratio > 0.80 && ratio < 0.95,
                "{name} late ratio {ratio} should sit around the 90/10 split"
            );
            assert!(
                ratio < stable_ratio - 0.03,
                "{name} late ratio {ratio} must be materially below stable {stable_ratio}"
            );
        }
        // B -> C and C -> D observe approximately the same structural reuse
        // (same header + stable core split; only the late suffix content
        // differs between the pairs).
        assert!(
            (bc_ratio - cd_ratio).abs() < 0.01,
            "B->C {bc_ratio} and C->D {cd_ratio} should agree closely"
        );
    }

    #[test]
    fn deepseek_late_live_b_to_c_structural_ratio_reproduces() {
        // Sanitized live evidence (deepseek-late-divergence-01, 2026-08-07):
        // B -> C observed structural reuse 7245 / 8063 estimated tokens =
        // 0.8985. The content is deterministic from seed 42 + scenario, so
        // the offline plan regenerates the same structural observation.
        let mut config = test_config("deepseek", Scenario::LateDivergence);
        config.experiment_id = "deepseek-late-divergence-01".to_string();
        let plan = build_plan(&config).unwrap();
        let provider = plan.provider.as_ref();
        let model = &config.model;
        let experiment_id = &config.experiment_id;
        let trace_b = build_trace(provider, model, experiment_id, &record_for(&plan.turns[1]));
        let trace_c = build_trace(provider, model, experiment_id, &record_for(&plan.turns[2]));
        let comparison = compare_traces(&trace_b, &trace_c, None).unwrap();
        let total = estimate_tokens(&plan.turns[1].header)
            + estimate_tokens(&plan.turns[1].prefix)
            + estimate_tokens(plan.turns[1].suffix.as_deref().unwrap())
            + estimate_tokens(&plan.turns[1].tail);
        let ratio = comparison.observed_reusable_prefix_tokens as f64 / total as f64;
        assert!(
            (ratio - 0.8985489271983133).abs() < 0.001,
            "expected live B->C structural ratio ~0.8985, got {ratio}"
        );
    }

    #[test]
    fn deepseek_early_live_b_to_c_structural_reuse_is_zero() {
        // Sanitized live evidence (deepseek-early-divergence-01, 2026-08-07):
        // B -> C changed the early header, destroying all structural reuse
        // (observed 0 / 8066 estimated tokens). Reproducible offline from
        // the deterministic plan.
        let mut config = test_config("deepseek", Scenario::EarlyDivergence);
        config.experiment_id = "deepseek-early-divergence-01".to_string();
        let plan = build_plan(&config).unwrap();
        let provider = plan.provider.as_ref();
        let model = &config.model;
        let experiment_id = &config.experiment_id;
        let trace_b = build_trace(provider, model, experiment_id, &record_for(&plan.turns[1]));
        let trace_c = build_trace(provider, model, experiment_id, &record_for(&plan.turns[2]));
        let comparison = compare_traces(&trace_b, &trace_c, None).unwrap();
        assert_eq!(
            comparison.observed_reusable_prefix_tokens, 0,
            "early header break must destroy structural reuse"
        );
        let divergence = comparison.first_divergence.as_ref().unwrap();
        assert_eq!(divergence.position, 0);
        assert_eq!(divergence.current_block_id, "prefix-header");
    }

    #[test]
    fn dry_run_exposes_late_divergence_split_and_never_sleeps() {
        let start = std::time::Instant::now();
        let info = describe_dry_run(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        let ld = info.late_divergence.as_ref().expect("late-divergence info");
        assert_eq!(ld.core_percent, 90);
        assert_eq!(ld.suffix_percent, 10);
        assert_eq!(ld.mutation_turns, vec![3, 4]);
        assert_eq!(info.request_count, 4);
        let delays: Vec<u64> = info.turns.iter().map(|t| t.pre_request_delay_ms).collect();
        assert_eq!(delays, vec![0, 0, 0, 10_000]);
        assert!(start.elapsed().as_millis() < 5_000);
    }

    /// Build a minimal RequestRecord from a turn spec for trace comparison.
    fn record_for(turn: &TurnSpec) -> RequestRecord {
        RequestRecord {
            turn: turn.turn,
            header: turn.header.clone(),
            prefix: turn.prefix.clone(),
            suffix: turn.suffix.clone(),
            tail: turn.tail.clone(),
            raw_usage: RawUsage {
                provider_schema: "synthetic".to_string(),
                raw: BTreeMap::new(),
            },
            provider_request_id: None,
            http_status: 200,
            started_at: "2026-08-07T00:00:00Z".to_string(),
            time_to_headers_ms: 1,
            time_to_first_body_byte_ms: Some(1),
            total_ms: 1,
        }
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

    #[test]
    fn deepseek_settle_delay_is_recorded_in_turn_specs() {
        // StablePrefix and EarlyDivergence: A=0, B=0, C=10000 (settle before
        // the measured third request, after A/B establish the common prefix).
        for scenario in [Scenario::StablePrefix, Scenario::EarlyDivergence] {
            let plan = build_plan(&test_config("deepseek", scenario)).unwrap();
            assert_eq!(plan.turns.len(), 3);
            assert_eq!(plan.turns[0].pre_request_delay_ms, 0);
            assert_eq!(plan.turns[1].pre_request_delay_ms, 0);
            assert_eq!(plan.turns[2].pre_request_delay_ms, 10_000);
        }
        // LateDivergence is four turns: A=0, B=0, C=0, D=10000 (the settle
        // is after C, which first exposes the common-prefix boundary).
        let late = build_plan(&test_config("deepseek", Scenario::LateDivergence)).unwrap();
        assert_eq!(late.turns.len(), 4);
        assert_eq!(late.turns[0].pre_request_delay_ms, 0);
        assert_eq!(late.turns[1].pre_request_delay_ms, 0);
        assert_eq!(late.turns[2].pre_request_delay_ms, 0);
        assert_eq!(late.turns[3].pre_request_delay_ms, 10_000);
        // schema-smoke has no delay.
        let smoke = build_plan(&test_config("deepseek", Scenario::SchemaSmoke)).unwrap();
        assert_eq!(smoke.turns[0].pre_request_delay_ms, 0);
        // OpenAI/Anthropic remain delay-free.
        for provider in ["openai", "anthropic"] {
            for scenario in [
                Scenario::SchemaSmoke,
                Scenario::StablePrefix,
                Scenario::EarlyDivergence,
                Scenario::LateDivergence,
            ] {
                let plan = build_plan(&test_config(provider, scenario)).unwrap();
                assert!(
                    plan.turns.iter().all(|t| t.pre_request_delay_ms == 0),
                    "{} {} must be delay-free",
                    provider,
                    scenario.as_str()
                );
            }
        }
    }

    #[test]
    fn dry_run_exposes_delays_and_never_sleeps() {
        // describe_dry_run takes no sleeper, so it structurally cannot wait;
        // additionally prove it returns quickly.
        let start = std::time::Instant::now();
        let info = describe_dry_run(&test_config("deepseek", Scenario::StablePrefix)).unwrap();
        let delays: Vec<u64> = info.turns.iter().map(|t| t.pre_request_delay_ms).collect();
        assert_eq!(delays, vec![0, 0, 10_000]);
        assert!(
            start.elapsed().as_millis() < 5_000,
            "dry-run must not actually sleep for the settle period"
        );
    }
}
