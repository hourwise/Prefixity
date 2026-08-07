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
    /// Build the request JSON body for one request.
    fn build_request_body(
        &self,
        model: &str,
        header: &str,
        prefix: &str,
        tail: &str,
    ) -> Result<Value, LiveError>;
    /// Extract the full safe usage object as `RawUsage`, or `None` if the
    /// response carries no usage object.
    fn extract_raw_usage(&self, body: &Value) -> Option<RawUsage>;
    /// A provider request identifier if present in the body (safe).
    fn request_id(&self, body: &Value) -> Option<String>;
    /// Structural path of the header block in the wire message.
    fn header_structural_path(&self) -> &'static str;
    /// Structural path of the prefix block in the wire message.
    fn prefix_structural_path(&self) -> &'static str;
    /// Structural path of the tail block in the wire message.
    fn tail_structural_path(&self) -> &'static str;
    /// Number of requests the given scenario plans for this provider.
    fn request_count(&self, scenario: Scenario) -> usize;
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

fn chat_completions_body(model: &str, header: &str, prefix: &str, tail: &str) -> Value {
    serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": header },
            { "role": "system", "content": prefix },
            { "role": "user", "content": tail }
        ],
        "max_tokens": 8,
        "temperature": 0,
        "stream": false
    })
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
        tail: &str,
    ) -> Result<Value, LiveError> {
        Ok(chat_completions_body(model, header, prefix, tail))
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
    fn tail_structural_path(&self) -> &'static str {
        "messages[2]"
    }
    fn request_count(&self, scenario: Scenario) -> usize {
        match scenario {
            Scenario::SchemaSmoke => 1,
            _ => 2,
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
        tail: &str,
    ) -> Result<Value, LiveError> {
        Ok(serde_json::json!({
            "model": model,
            "max_tokens": 8,
            "system": [
                { "type": "text", "text": header },
                { "type": "text", "text": prefix, "cache_control": { "type": "ephemeral" } }
            ],
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
    fn tail_structural_path(&self) -> &'static str {
        "messages[0]"
    }
    fn request_count(&self, scenario: Scenario) -> usize {
        match scenario {
            Scenario::SchemaSmoke => 1,
            _ => 2,
        }
    }
}

/// DeepSeek adapter.
///
/// DeepSeek documentation describes cache construction as potentially
/// requiring a prior completed request, so the stable-prefix scenario plans
/// **three** requests for this provider (A, B, C) and the observed behaviour
/// is preserved rather than assumed. The plan is still bounded by
/// `--max-requests`.
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
        tail: &str,
    ) -> Result<Value, LiveError> {
        Ok(chat_completions_body(model, header, prefix, tail))
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
    fn tail_structural_path(&self) -> &'static str {
        "messages[2]"
    }
    fn request_count(&self, scenario: Scenario) -> usize {
        match scenario {
            Scenario::SchemaSmoke => 1,
            Scenario::StablePrefix => 3, // documented: cache construction may need a prior request
            _ => 2,
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
            .build_request_body("gpt-test", "h", "p", "t")
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
            .build_request_body("claude-test", "h", "p", "t")
            .unwrap();
        let text = body_str(&body);
        // cache_control is on the prefix system block only; the credential
        // header is not part of the body.
        assert!(text.contains("cache_control"));
        assert!(text.contains("ephemeral"));
        assert!(!text.contains("x-api-key"));
    }

    #[test]
    fn deepseek_stable_prefix_plans_three_requests() {
        let deepseek = DeepSeekProvider;
        assert_eq!(deepseek.request_count(Scenario::StablePrefix), 3);
        assert_eq!(deepseek.request_count(Scenario::SchemaSmoke), 1);
        assert_eq!(deepseek.request_count(Scenario::EarlyDivergence), 2);
        assert_eq!(deepseek.request_count(Scenario::LateDivergence), 2);
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
