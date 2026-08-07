//! Minimal provider adapters for Phase 0B: OpenAI, Anthropic, DeepSeek.
//!
//! Each adapter exposes a common research interface, hard-codes its
//! allowlisted base URL, builds the request body, and extracts the full safe
//! usage object (preserving unknown fields) as `RawUsage`. No analysis logic
//! lives here.

use crate::error::LiveError;
use crate::scenario::Scenario;
use prefixity_core::model::RawUsage;
use prefixity_core::usage::{
    SCHEMA_ANTHROPIC_MESSAGES_V1, SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1,
    SCHEMA_OPENAI_CHAT_COMPLETIONS_V1,
};
use serde_json::Value;
use std::collections::BTreeMap;

/// Conservative Phase 0B experimental settle delay (ms) applied **before
/// DeepSeek's final request** so best-effort, asynchronous cache persistence
/// has time to complete after the previous request established a new
/// common-prefix boundary. This is an experimental control, NOT an official
/// DeepSeek requirement or a scientifically validated optimum. The important
/// settle period is after the request that first exposes a divergence: for
/// `stable-prefix`/`early-divergence` that is C (after A/B establish the
/// common prefix); for `late-divergence` it is D (after C first diverges the
/// late suffix and lets DeepSeek discover the shorter common core).
pub const DEEPSEEK_SETTLE_DELAY_MS: u64 = 10_000;

/// The explicit per-provider, per-scenario request plan.
///
/// Kept explicit (rather than hidden special-case arithmetic) so each
/// provider/scenario combination is auditable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderTurnPlan {
    /// Number of requests the scenario runs for this provider.
    pub turns: usize,
    /// First turn (1-based, inclusive) whose early header is diverged, for
    /// `early-divergence`. `None` means the header is never diverged.
    pub header_diverges_from: Option<usize>,
    /// First turn (1-based, inclusive) whose **late mutable suffix** is
    /// changed, for `late-divergence`. `None` for scenarios without a late
    /// suffix. Keeps `late-divergence` structurally distinct from
    /// `stable-prefix`.
    pub late_suffix_mutates_from: Option<usize>,
    /// Experimental settle delay (ms) applied **before the final turn** (for
    /// example, before DeepSeek's C request after A/B established the common
    /// prefix, or before DeepSeek's D request after C first diverged the
    /// late suffix). Zero for OpenAI/Anthropic and schema-smoke.
    pub settle_delay_ms: u64,
}

impl ProviderTurnPlan {
    /// A plan with no early-header divergence and no settle delay.
    pub fn stable(turns: usize) -> ProviderTurnPlan {
        ProviderTurnPlan {
            turns,
            header_diverges_from: None,
            late_suffix_mutates_from: None,
            settle_delay_ms: 0,
        }
    }

    /// An early-divergence plan where the early header diverges at `turn`.
    pub fn diverging(turns: usize, turn: usize) -> ProviderTurnPlan {
        ProviderTurnPlan {
            turns,
            header_diverges_from: Some(turn),
            late_suffix_mutates_from: None,
            settle_delay_ms: 0,
        }
    }

    /// A late-divergence plan where the late mutable suffix changes from
    /// `turn` onward (header and core stay identical).
    pub fn late(turns: usize, turn: usize) -> ProviderTurnPlan {
        ProviderTurnPlan {
            turns,
            header_diverges_from: None,
            late_suffix_mutates_from: Some(turn),
            settle_delay_ms: 0,
        }
    }

    /// A DeepSeek plan: three requests with the experimental settle delay
    /// applied before the final (third) turn.
    pub fn deepseek(header_diverges_from: Option<usize>) -> ProviderTurnPlan {
        ProviderTurnPlan {
            turns: 3,
            header_diverges_from,
            late_suffix_mutates_from: None,
            settle_delay_ms: DEEPSEEK_SETTLE_DELAY_MS,
        }
    }

    /// A DeepSeek late-divergence plan: **four** requests (A, B, C, D). A and
    /// B carry the ORIGINAL late suffix and demonstrate long stable-prefix
    /// cache availability. C first diverges the late suffix (variant 1),
    /// exposing the shorter common stable core. D carries a SECOND distinct
    /// suffix variant (so it cannot simply hit C's request-boundary cache)
    /// and tests whether the common core persisted after C. The experimental
    /// settle delay is applied **before D**, after C has completed and
    /// allowed common-prefix persistence.
    pub fn deepseek_late() -> ProviderTurnPlan {
        ProviderTurnPlan {
            turns: 4,
            header_diverges_from: None,
            late_suffix_mutates_from: Some(3),
            settle_delay_ms: DEEPSEEK_SETTLE_DELAY_MS,
        }
    }

    /// The pre-request delay (ms) to apply before the given 1-based turn.
    pub fn pre_request_delay_ms(&self, turn: usize) -> u64 {
        if self.settle_delay_ms > 0 && turn == self.turns {
            self.settle_delay_ms
        } else {
            0
        }
    }

    /// Whether the late mutable suffix is changed on the given 1-based turn.
    pub fn late_suffix_mutates(&self, turn: usize) -> bool {
        self.late_suffix_mutates_from
            .is_some_and(|first| turn >= first)
    }

    /// The first turn (1-based) whose late suffix mutates, if any.
    pub fn late_mutation_turn(&self) -> Option<usize> {
        self.late_suffix_mutates_from
    }

    /// The 1-based turns, in order, whose late suffix mutates (e.g. `[3, 4]`
    /// for the four-turn DeepSeek late plan, `[2]` for OpenAI/Anthropic).
    /// Empty when the scenario has no late suffix.
    pub fn late_mutation_turns(&self) -> Vec<usize> {
        (1..=self.turns)
            .filter(|turn| self.late_suffix_mutates(*turn))
            .collect()
    }
}

/// Common research interface for a live provider.
pub trait LiveProvider: Send + Sync + std::fmt::Debug {
    /// Canonical provider id (`openai`, `anthropic`, `deepseek`).
    fn provider_id(&self) -> &'static str;
    /// The explicit versioned API-surface usage schema this adapter emits
    /// (e.g. `openai-chat-completions-v1`). Distinct from the provider id:
    /// one provider may expose different usage semantics across endpoints.
    fn usage_schema(&self) -> &'static str;
    /// The environment variable that holds this provider's credential.
    fn credential_env_var(&self) -> &'static str;
    /// Allowlisted base URL (never user-supplied).
    fn base_url(&self) -> &'static str;
    /// Endpoint path appended to the base URL.
    fn endpoint_path(&self) -> &'static str;
    /// Name of the credential header (`Authorization` or `x-api-key`).
    fn auth_header_name(&self) -> &'static str;
    /// Value of the credential header built from the raw key.
    fn auth_header_value(&self, key: &str) -> String;
    /// Provider-specific static headers (e.g. `anthropic-version`).
    fn extra_headers(&self) -> Vec<(&'static str, &'static str)>;
    /// Build the request JSON body for one request. `suffix` is the optional
    /// late mutable suffix (present only for `late-divergence`); when set it
    /// is emitted as a separate wire block between the prefix and the tail.
    fn build_request_body(
        &self,
        model: &str,
        header: &str,
        prefix: &str,
        suffix: Option<&str>,
        tail: &str,
    ) -> Result<Value, LiveError>;
    /// Extract the full safe usage object as `RawUsage`, or `None` if the
    /// response carries no usage object.
    fn extract_raw_usage(&self, body: &Value) -> Option<RawUsage>;
    /// A provider request identifier if present in the body (safe).
    fn request_id(&self, body: &Value) -> Option<String>;
    /// Structural path of the header block in the wire message.
    fn header_structural_path(&self) -> &'static str;
    /// Structural path of the prefix (core) block in the wire message.
    fn prefix_structural_path(&self) -> &'static str;
    /// Structural path of the late mutable suffix block in the wire message.
    fn suffix_structural_path(&self) -> &'static str;
    /// Structural path of the tail block in the wire message. `has_suffix`
    /// shifts the tail position on chat-completions-style message arrays.
    fn tail_structural_path(&self, has_suffix: bool) -> &'static str;
    /// The explicit per-provider, per-scenario turn plan: how many requests
    /// run and, for `early-divergence`, the first turn (1-based, inclusive)
    /// whose early header is diverged. OpenAI and Anthropic diverge at turn
    /// B; DeepSeek runs A/B/C and diverges only at turn C so that A and B
    /// first establish the common prefix.
    fn plan_turns(&self, scenario: Scenario) -> ProviderTurnPlan;
}

/// Shared helper: extract the `usage` object verbatim.
fn usage_from_body(body: &Value) -> Option<RawUsage> {
    let usage = body.get("usage")?.as_object()?;
    let mut raw = BTreeMap::new();
    for (key, value) in usage {
        raw.insert(key.clone(), value.clone());
    }
    Some(RawUsage {
        provider_schema: String::new(), // filled by the adapter
        raw,
    })
}

fn id_from_body(body: &Value) -> Option<String> {
    body.get("id").and_then(Value::as_str).map(str::to_string)
}

fn chat_completions_body(
    model: &str,
    header: &str,
    prefix: &str,
    suffix: Option<&str>,
    tail: &str,
) -> Value {
    let mut messages = vec![
        serde_json::json!({ "role": "system", "content": header }),
        serde_json::json!({ "role": "system", "content": prefix }),
    ];
    if let Some(suffix) = suffix {
        // Late-divergence only: a real separate wire block for the mutable
        // suffix, between the stable core and the tail.
        messages.push(serde_json::json!({ "role": "system", "content": suffix }));
    }
    messages.push(serde_json::json!({ "role": "user", "content": tail }));
    serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": 8,
        "temperature": 0,
        "stream": false
    })
}

fn deepseek_chat_completions_body(
    model: &str,
    header: &str,
    prefix: &str,
    suffix: Option<&str>,
    tail: &str,
) -> Value {
    // Phase 0B measures prompt/cache behaviour, not reasoning: thinking is
    // explicitly disabled so request cost is dominated by the prefix. The
    // model is never hard-coded; it always comes from the CLI.
    let mut body = chat_completions_body(model, header, prefix, suffix, tail);
    body["thinking"] = serde_json::json!({ "type": "disabled" });
    body
}

/// OpenAI adapter. Baseline uses the default/native caching behaviour; no
/// explicit cache controls are added in Phase 0B baseline.
#[derive(Debug, Default)]
pub struct OpenAiProvider;

impl LiveProvider for OpenAiProvider {
    fn provider_id(&self) -> &'static str {
        "openai"
    }
    fn usage_schema(&self) -> &'static str {
        SCHEMA_OPENAI_CHAT_COMPLETIONS_V1
    }
    fn credential_env_var(&self) -> &'static str {
        crate::credentials::OPENAI_API_KEY
    }
    fn base_url(&self) -> &'static str {
        "https://api.openai.com"
    }
    fn endpoint_path(&self) -> &'static str {
        "/v1/chat/completions"
    }
    fn auth_header_name(&self) -> &'static str {
        "Authorization"
    }
    fn auth_header_value(&self, key: &str) -> String {
        format!("Bearer {key}")
    }
    fn extra_headers(&self) -> Vec<(&'static str, &'static str)> {
        Vec::new()
    }
    fn build_request_body(
        &self,
        model: &str,
        header: &str,
        prefix: &str,
        suffix: Option<&str>,
        tail: &str,
    ) -> Result<Value, LiveError> {
        Ok(chat_completions_body(model, header, prefix, suffix, tail))
    }
    fn extract_raw_usage(&self, body: &Value) -> Option<RawUsage> {
        usage_from_body(body).map(|mut u| {
            u.provider_schema = self.usage_schema().to_string();
            u
        })
    }
    fn request_id(&self, body: &Value) -> Option<String> {
        id_from_body(body)
    }
    fn header_structural_path(&self) -> &'static str {
        "messages[0]"
    }
    fn prefix_structural_path(&self) -> &'static str {
        "messages[1]"
    }
    fn suffix_structural_path(&self) -> &'static str {
        "messages[2]"
    }
    fn tail_structural_path(&self, has_suffix: bool) -> &'static str {
        if has_suffix {
            "messages[3]"
        } else {
            "messages[2]"
        }
    }
    fn plan_turns(&self, scenario: Scenario) -> ProviderTurnPlan {
        match scenario {
            Scenario::SchemaSmoke => ProviderTurnPlan::stable(1),
            Scenario::EarlyDivergence => ProviderTurnPlan::diverging(2, 2),
            // Late-divergence mutates the late suffix on the measurement
            // turn (B); StablePrefix keeps a single prefix block.
            Scenario::LateDivergence => ProviderTurnPlan::late(2, 2),
            _ => ProviderTurnPlan::stable(2),
        }
    }
}

/// Anthropic adapter. Uses explicit `cache_control` on the large prefix
/// block (the controlled experiment requires a cacheable prefix longer than
/// the model's documented minimum; the ~8k-token default exceeds it).
#[derive(Debug, Default)]
pub struct AnthropicProvider;

impl LiveProvider for AnthropicProvider {
    fn provider_id(&self) -> &'static str {
        "anthropic"
    }
    fn usage_schema(&self) -> &'static str {
        SCHEMA_ANTHROPIC_MESSAGES_V1
    }
    fn credential_env_var(&self) -> &'static str {
        crate::credentials::ANTHROPIC_API_KEY
    }
    fn base_url(&self) -> &'static str {
        "https://api.anthropic.com"
    }
    fn endpoint_path(&self) -> &'static str {
        "/v1/messages"
    }
    fn auth_header_name(&self) -> &'static str {
        "x-api-key"
    }
    fn auth_header_value(&self, key: &str) -> String {
        key.to_string()
    }
    fn extra_headers(&self) -> Vec<(&'static str, &'static str)> {
        vec![("anthropic-version", "2023-06-01")]
    }
    fn build_request_body(
        &self,
        model: &str,
        header: &str,
        prefix: &str,
        suffix: Option<&str>,
        tail: &str,
    ) -> Result<Value, LiveError> {
        let mut system = vec![
            serde_json::json!({ "type": "text", "text": header }),
            serde_json::json!({
                "type": "text",
                "text": prefix,
                "cache_control": { "type": "ephemeral" }
            }),
        ];
        if let Some(suffix) = suffix {
            // Late-divergence only: a real separate system text block for the
            // mutable suffix, after the cached core.
            system.push(serde_json::json!({ "type": "text", "text": suffix }));
        }
        Ok(serde_json::json!({
            "model": model,
            "max_tokens": 8,
            "system": system,
            "messages": [
                { "role": "user", "content": tail }
            ]
        }))
    }
    fn extract_raw_usage(&self, body: &Value) -> Option<RawUsage> {
        usage_from_body(body).map(|mut u| {
            u.provider_schema = self.usage_schema().to_string();
            u
        })
    }
    fn request_id(&self, body: &Value) -> Option<String> {
        id_from_body(body)
    }
    fn header_structural_path(&self) -> &'static str {
        "system[0]"
    }
    fn prefix_structural_path(&self) -> &'static str {
        "system[1]"
    }
    fn suffix_structural_path(&self) -> &'static str {
        "system[2]"
    }
    fn tail_structural_path(&self, _has_suffix: bool) -> &'static str {
        // Anthropic's tail lives in the separate `messages` array, so its
        // position is independent of the system-block count.
        "messages[0]"
    }
    fn plan_turns(&self, scenario: Scenario) -> ProviderTurnPlan {
        match scenario {
            Scenario::SchemaSmoke => ProviderTurnPlan::stable(1),
            Scenario::EarlyDivergence => ProviderTurnPlan::diverging(2, 2),
            Scenario::LateDivergence => ProviderTurnPlan::late(2, 2),
            _ => ProviderTurnPlan::stable(2),
        }
    }
}

/// DeepSeek adapter.
///
/// DeepSeek documentation describes cache construction as potentially
/// requiring a prior completed request, so `stable-prefix` and
/// `early-divergence` plan **three** requests (A, B, C): A and B first
/// establish the common prefix, and the C request is the one whose reuse is
/// measured. `late-divergence` plans **four** requests (A, B, C, D): A/B
/// carry the original late suffix, C first diverges it (variant 1), and D
/// carries a second distinct suffix variant to test common-core persistence
/// after C. For `early-divergence` the early header is only diverged at C,
/// never at B. Plans are still bounded by `--max-requests`. Thinking is
/// explicitly disabled (see [`DeepSeekProvider::build_request_body`]).
#[derive(Debug, Default)]
pub struct DeepSeekProvider;

impl LiveProvider for DeepSeekProvider {
    fn provider_id(&self) -> &'static str {
        "deepseek"
    }
    fn usage_schema(&self) -> &'static str {
        SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1
    }
    fn credential_env_var(&self) -> &'static str {
        crate::credentials::DEEPSEEK_API_KEY
    }
    fn base_url(&self) -> &'static str {
        "https://api.deepseek.com"
    }
    fn endpoint_path(&self) -> &'static str {
        "/chat/completions"
    }
    fn auth_header_name(&self) -> &'static str {
        "Authorization"
    }
    fn auth_header_value(&self, key: &str) -> String {
        format!("Bearer {key}")
    }
    fn extra_headers(&self) -> Vec<(&'static str, &'static str)> {
        Vec::new()
    }
    fn build_request_body(
        &self,
        model: &str,
        header: &str,
        prefix: &str,
        suffix: Option<&str>,
        tail: &str,
    ) -> Result<Value, LiveError> {
        Ok(deepseek_chat_completions_body(
            model, header, prefix, suffix, tail,
        ))
    }
    fn extract_raw_usage(&self, body: &Value) -> Option<RawUsage> {
        usage_from_body(body).map(|mut u| {
            u.provider_schema = self.usage_schema().to_string();
            u
        })
    }
    fn request_id(&self, body: &Value) -> Option<String> {
        id_from_body(body)
    }
    fn header_structural_path(&self) -> &'static str {
        "messages[0]"
    }
    fn prefix_structural_path(&self) -> &'static str {
        "messages[1]"
    }
    fn suffix_structural_path(&self) -> &'static str {
        "messages[2]"
    }
    fn tail_structural_path(&self, has_suffix: bool) -> &'static str {
        if has_suffix {
            "messages[3]"
        } else {
            "messages[2]"
        }
    }
    fn plan_turns(&self, scenario: Scenario) -> ProviderTurnPlan {
        match scenario {
            Scenario::SchemaSmoke => ProviderTurnPlan::stable(1),
            // A and B first establish the common prefix; a conservative 10s
            // settle delay is applied before C so best-effort async cache
            // persistence can complete. The early header only diverges at C.
            Scenario::EarlyDivergence => ProviderTurnPlan::deepseek(Some(3)),
            // Late-divergence is four requests: A/B carry the original late
            // suffix, C first diverges it (variant 1), D carries a second
            // distinct suffix variant. The 10s settle is applied before D,
            // after C has exposed the common-prefix boundary.
            Scenario::LateDivergence => ProviderTurnPlan::deepseek_late(),
            _ => ProviderTurnPlan::deepseek(None),
        }
    }
}

/// Resolve a provider by id. Base URLs are allowlisted inside the adapters;
/// unknown ids are rejected.
pub fn provider_from_id(id: &str) -> Result<Box<dyn LiveProvider>, LiveError> {
    match id {
        "openai" => Ok(Box::new(OpenAiProvider)),
        "anthropic" => Ok(Box::new(AnthropicProvider)),
        "deepseek" => Ok(Box::new(DeepSeekProvider)),
        other => Err(LiveError::argument(format!(
            "unknown provider '{other}'. expected one of: openai, anthropic, deepseek"
        ))),
    }
}

/// The supported provider ids.
pub fn provider_ids() -> &'static [&'static str] {
    &["openai", "anthropic", "deepseek"]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body_str(value: &Value) -> String {
        serde_json::to_string(value).unwrap()
    }

    #[test]
    fn provider_allowlist_rejects_unknown() {
        assert!(provider_from_id("openai").is_ok());
        assert!(provider_from_id("anthropic").is_ok());
        assert!(provider_from_id("deepseek").is_ok());
        assert!(provider_from_id("mystery-vendor").is_err());
        assert!(provider_from_id("http://evil.example").is_err());
    }

    #[test]
    fn base_urls_are_allowlisted_constants() {
        let openai = OpenAiProvider;
        let anthropic = AnthropicProvider;
        let deepseek = DeepSeekProvider;
        assert_eq!(openai.base_url(), "https://api.openai.com");
        assert_eq!(anthropic.base_url(), "https://api.anthropic.com");
        assert_eq!(deepseek.base_url(), "https://api.deepseek.com");
    }

    #[test]
    fn openai_request_body_is_deterministic_and_has_no_key() {
        let body = OpenAiProvider
            .build_request_body("gpt-test", "h", "p", None, "t")
            .unwrap();
        let text = body_str(&body);
        assert!(text.contains("\"gpt-test\""));
        assert!(text.contains("\"messages\""));
        assert!(text.contains("\"h\"")); // header content
        assert!(!text.contains("Bearer"));
        assert!(!text.contains("api_key"));
    }

    #[test]
    fn anthropic_request_body_uses_cache_control_on_prefix_only() {
        let body = AnthropicProvider
            .build_request_body("claude-test", "h", "p", None, "t")
            .unwrap();
        let text = body_str(&body);
        // cache_control is on the prefix system block only; the credential
        // header is not part of the body.
        assert!(text.contains("cache_control"));
        assert!(text.contains("ephemeral"));
        assert!(!text.contains("x-api-key"));
    }

    #[test]
    fn deepseek_plans_three_requests_for_b_d_and_diverges_only_at_c() {
        let deepseek = DeepSeekProvider;
        assert_eq!(
            deepseek.plan_turns(Scenario::SchemaSmoke),
            ProviderTurnPlan::stable(1)
        );
        // B and C prime with A and B, then measure the third request with a
        // settle delay before it.
        assert_eq!(
            deepseek.plan_turns(Scenario::StablePrefix),
            ProviderTurnPlan::deepseek(None)
        );
        // Late-divergence is four requests (A/B/C/D): the late suffix first
        // mutates at C, and D carries a second distinct variant.
        assert_eq!(
            deepseek.plan_turns(Scenario::LateDivergence),
            ProviderTurnPlan::deepseek_late()
        );
        // Early divergence only at turn C so A/B establish the prefix.
        assert_eq!(
            deepseek.plan_turns(Scenario::EarlyDivergence),
            ProviderTurnPlan::deepseek(Some(3))
        );
    }

    #[test]
    fn deepseek_settle_delay_applies_before_the_final_turn() {
        // StablePrefix and EarlyDivergence are three turns: the settle
        // applies before C (after A/B establish the common prefix).
        for scenario in [Scenario::StablePrefix, Scenario::EarlyDivergence] {
            let plan = DeepSeekProvider.plan_turns(scenario);
            assert_eq!(plan.turns, 3);
            assert_eq!(plan.pre_request_delay_ms(1), 0);
            assert_eq!(plan.pre_request_delay_ms(2), 0);
            assert_eq!(plan.pre_request_delay_ms(3), DEEPSEEK_SETTLE_DELAY_MS);
        }
        // LateDivergence is four turns: the settle applies before D (after C
        // first diverges the late suffix and lets the common core persist).
        let late = DeepSeekProvider.plan_turns(Scenario::LateDivergence);
        assert_eq!(late.turns, 4);
        assert_eq!(late.pre_request_delay_ms(1), 0);
        assert_eq!(late.pre_request_delay_ms(2), 0);
        assert_eq!(late.pre_request_delay_ms(3), 0);
        assert_eq!(late.pre_request_delay_ms(4), DEEPSEEK_SETTLE_DELAY_MS);
        // schema-smoke has no settle delay.
        let smoke = DeepSeekProvider.plan_turns(Scenario::SchemaSmoke);
        assert_eq!(smoke.settle_delay_ms, 0);
        assert_eq!(smoke.pre_request_delay_ms(1), 0);
    }

    #[test]
    fn openai_and_anthropic_keep_two_request_plans() {
        let openai = OpenAiProvider;
        let anthropic = AnthropicProvider;
        for provider in [&openai as &dyn LiveProvider, &anthropic] {
            assert_eq!(
                provider.plan_turns(Scenario::SchemaSmoke),
                ProviderTurnPlan::stable(1)
            );
            assert_eq!(
                provider.plan_turns(Scenario::StablePrefix),
                ProviderTurnPlan::stable(2)
            );
            // Late-divergence mutates the late suffix on the measurement
            // turn (B), keeping it distinct from StablePrefix.
            assert_eq!(
                provider.plan_turns(Scenario::LateDivergence),
                ProviderTurnPlan::late(2, 2)
            );
            // Two-request early divergence changes the header at turn B.
            assert_eq!(
                provider.plan_turns(Scenario::EarlyDivergence),
                ProviderTurnPlan::diverging(2, 2)
            );
        }
    }

    #[test]
    fn openai_and_anthropic_plans_are_delay_free() {
        for provider in [&OpenAiProvider as &dyn LiveProvider, &AnthropicProvider] {
            for scenario in [
                Scenario::SchemaSmoke,
                Scenario::StablePrefix,
                Scenario::EarlyDivergence,
                Scenario::LateDivergence,
            ] {
                let plan = provider.plan_turns(scenario);
                assert_eq!(plan.settle_delay_ms, 0);
                for turn in 1..=plan.turns {
                    assert_eq!(plan.pre_request_delay_ms(turn), 0);
                }
            }
        }
    }

    #[test]
    fn deepseek_request_body_explicitly_disables_thinking() {
        // The model stays explicit (never hard-coded in the adapter);
        // thinking is disabled and temperature stays 0.
        let body = DeepSeekProvider
            .build_request_body("deepseek-v4-flash", "h", "p", None, "t")
            .unwrap();
        assert_eq!(body["model"], "deepseek-v4-flash");
        assert_eq!(body["temperature"], 0);
        assert_eq!(body["thinking"]["type"], "disabled");
    }

    #[test]
    fn openai_request_body_has_no_thinking_field() {
        let body = OpenAiProvider
            .build_request_body("gpt-test", "h", "p", None, "t")
            .unwrap();
        assert!(body.get("thinking").is_none());
        assert_eq!(body["temperature"], 0);
    }

    #[test]
    fn late_divergence_plan_mutates_suffix_at_the_measurement_turns() {
        // DeepSeek: A/B carry the original suffix; C mutates it (variant 1)
        // and D mutates it again (variant 2).
        let deepseek_plan = DeepSeekProvider.plan_turns(Scenario::LateDivergence);
        assert_eq!(deepseek_plan.late_mutation_turn(), Some(3));
        assert_eq!(deepseek_plan.late_mutation_turns(), vec![3, 4]);
        assert!(!deepseek_plan.late_suffix_mutates(1));
        assert!(!deepseek_plan.late_suffix_mutates(2));
        assert!(deepseek_plan.late_suffix_mutates(3));
        assert!(deepseek_plan.late_suffix_mutates(4));
        // OpenAI/Anthropic: B is the only measurement turn.
        for provider in [&OpenAiProvider as &dyn LiveProvider, &AnthropicProvider] {
            let plan = provider.plan_turns(Scenario::LateDivergence);
            assert_eq!(plan.late_mutation_turn(), Some(2));
            assert_eq!(plan.late_mutation_turns(), vec![2]);
            assert!(!plan.late_suffix_mutates(1));
            assert!(plan.late_suffix_mutates(2));
            // StablePrefix never mutates a suffix.
            assert_eq!(
                provider
                    .plan_turns(Scenario::StablePrefix)
                    .late_mutation_turn(),
                None
            );
        }
        assert_eq!(
            DeepSeekProvider
                .plan_turns(Scenario::StablePrefix)
                .late_mutation_turn(),
            None
        );
        assert!(DeepSeekProvider
            .plan_turns(Scenario::StablePrefix)
            .late_mutation_turns()
            .is_empty());
    }

    #[test]
    fn late_divergence_bodies_emit_separate_suffix_wire_block() {
        // OpenAI/DeepSeek: four messages; the suffix is a separate system
        // message between core and tail.
        let openai = OpenAiProvider
            .build_request_body("m", "h", "core", Some("suffix"), "tail")
            .unwrap();
        let messages = openai["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 4);
        assert_eq!(messages[0]["content"], "h");
        assert_eq!(messages[1]["content"], "core");
        assert_eq!(messages[2]["content"], "suffix");
        assert_eq!(messages[3]["content"], "tail");

        let deepseek = DeepSeekProvider
            .build_request_body("m", "h", "core", Some("suffix"), "tail")
            .unwrap();
        assert_eq!(deepseek["messages"].as_array().unwrap().len(), 4);
        assert_eq!(deepseek["messages"][2]["content"], "suffix");
        assert_eq!(deepseek["thinking"]["type"], "disabled");

        // Anthropic: a separate system text block for the suffix.
        let anthropic = AnthropicProvider
            .build_request_body("m", "h", "core", Some("suffix"), "tail")
            .unwrap();
        let system = anthropic["system"].as_array().unwrap();
        assert_eq!(system.len(), 3);
        assert_eq!(system[0]["text"], "h");
        assert_eq!(system[1]["text"], "core");
        assert_eq!(system[2]["text"], "suffix");
    }

    #[test]
    fn stable_prefix_bodies_have_no_suffix_block() {
        let openai = OpenAiProvider
            .build_request_body("m", "h", "p", None, "tail")
            .unwrap();
        assert_eq!(openai["messages"].as_array().unwrap().len(), 3);
        let anthropic = AnthropicProvider
            .build_request_body("m", "h", "p", None, "tail")
            .unwrap();
        assert_eq!(anthropic["system"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn openai_extracts_usage_preserving_unknown_fields() {
        let body: Value = serde_json::from_str(
            r#"{
                "id": "chatcmpl-abc",
                "model": "gpt-test",
                "usage": {
                    "prompt_tokens": 100,
                    "completion_tokens": 5,
                    "total_tokens": 105,
                    "prompt_tokens_details": { "cached_tokens": 80 },
                    "some_future_field": 123
                }
            }"#,
        )
        .unwrap();
        let usage = OpenAiProvider.extract_raw_usage(&body).unwrap();
        assert_eq!(usage.provider_schema, SCHEMA_OPENAI_CHAT_COMPLETIONS_V1);
        assert_eq!(usage.raw.get("prompt_tokens").unwrap().as_u64(), Some(100));
        // Unknown fields are preserved verbatim.
        assert_eq!(
            usage.raw.get("some_future_field").unwrap().as_u64(),
            Some(123)
        );
        assert_eq!(
            OpenAiProvider.request_id(&body).as_deref(),
            Some("chatcmpl-abc")
        );
    }

    #[test]
    fn anthropic_extracts_three_category_usage() {
        let body: Value = serde_json::from_str(
            r#"{
                "id": "msg_01abc",
                "usage": {
                    "input_tokens": 500,
                    "output_tokens": 8,
                    "cache_creation_input_tokens": 7500,
                    "cache_read_input_tokens": 100
                }
            }"#,
        )
        .unwrap();
        let usage = AnthropicProvider.extract_raw_usage(&body).unwrap();
        assert_eq!(usage.provider_schema, SCHEMA_ANTHROPIC_MESSAGES_V1);
        assert_eq!(
            usage
                .raw
                .get("cache_creation_input_tokens")
                .unwrap()
                .as_u64(),
            Some(7500)
        );
        assert_eq!(
            usage.raw.get("cache_read_input_tokens").unwrap().as_u64(),
            Some(100)
        );
    }

    #[test]
    fn deepseek_extracts_hit_and_miss() {
        let body: Value = serde_json::from_str(
            r#"{
                "id": "abc123",
                "usage": {
                    "prompt_cache_hit_tokens": 800,
                    "prompt_cache_miss_tokens": 200,
                    "completion_tokens": 8
                }
            }"#,
        )
        .unwrap();
        let usage = DeepSeekProvider.extract_raw_usage(&body).unwrap();
        assert_eq!(usage.provider_schema, SCHEMA_DEEPSEEK_CHAT_COMPLETIONS_V1);
        assert_eq!(
            usage.raw.get("prompt_cache_hit_tokens").unwrap().as_u64(),
            Some(800)
        );
    }

    #[test]
    fn missing_usage_is_none() {
        let body: Value = serde_json::from_str(r#"{"id":"x","model":"m","choices":[]}"#).unwrap();
        assert!(OpenAiProvider.extract_raw_usage(&body).is_none());
    }

    #[test]
    fn malformed_usage_is_none() {
        let body: Value = serde_json::from_str(r#"{"id":"x","usage":"not-an-object"}"#).unwrap();
        assert!(DeepSeekProvider.extract_raw_usage(&body).is_none());
    }

    #[test]
    fn auth_headers_are_correct_per_provider() {
        assert_eq!(OpenAiProvider.auth_header_name(), "Authorization");
        assert_eq!(OpenAiProvider.auth_header_value("k"), "Bearer k");
        assert_eq!(AnthropicProvider.auth_header_name(), "x-api-key");
        assert_eq!(AnthropicProvider.auth_header_value("k"), "k");
        assert_eq!(DeepSeekProvider.auth_header_name(), "Authorization");
        assert_eq!(DeepSeekProvider.auth_header_value("k"), "Bearer k");
    }
}
