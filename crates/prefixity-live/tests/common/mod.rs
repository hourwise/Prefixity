//! Shared helpers for `prefixity-live` integration tests.
//!
//! These tests are fully offline: they use the [`MockTransport`] and never
//! make provider or network calls, and they need no credentials.

#![allow(dead_code)]

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Create a unique temporary directory for one test.
pub fn temp_dir(tag: &str) -> PathBuf {
    let unique = format!(
        "prefixity-live-test-{tag}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A valid OpenAI-shaped chat completions response body.
pub fn openai_ok(prompt: u64, completion: u64, cached: Option<u64>) -> String {
    let mut usage = serde_json::json!({
        "prompt_tokens": prompt,
        "completion_tokens": completion,
        "total_tokens": prompt + completion,
    });
    if let Some(cached) = cached {
        usage["prompt_tokens_details"] = serde_json::json!({ "cached_tokens": cached });
    }
    serde_json::to_string(&serde_json::json!({
        "id": "chatcmpl-test",
        "model": "gpt-test-model",
        "choices": [
            { "index": 0, "message": { "role": "assistant", "content": "OK" }, "finish_reason": "stop" }
        ],
        "usage": usage,
    }))
    .unwrap()
}

/// A valid Anthropic-shaped messages response body.
pub fn anthropic_ok(input: u64, output: u64, cache_read: u64, cache_creation: u64) -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "msg_01test",
        "model": "claude-test-model",
        "role": "assistant",
        "content": [ { "type": "text", "text": "OK" } ],
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": input,
            "output_tokens": output,
            "cache_creation_input_tokens": cache_creation,
            "cache_read_input_tokens": cache_read
        }
    }))
    .unwrap()
}

/// A valid DeepSeek-shaped chat completions response body.
pub fn deepseek_ok(hit: u64, miss: u64, completion: u64) -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "deepseek-test",
        "model": "deepseek-test-model",
        "choices": [
            { "index": 0, "message": { "role": "assistant", "content": "OK" }, "finish_reason": "stop" }
        ],
        "usage": {
            "prompt_cache_hit_tokens": hit,
            "prompt_cache_miss_tokens": miss,
            "completion_tokens": completion
        }
    }))
    .unwrap()
}

/// A DeepSeek-shaped response whose usage reports completion tokens but
/// lacks the defining `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens`
/// input categories.
pub fn deepseek_completion_only() -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "deepseek-test",
        "model": "deepseek-test-model",
        "choices": [
            { "index": 0, "message": { "role": "assistant", "content": "OK" }, "finish_reason": "stop" }
        ],
        "usage": { "completion_tokens": 8 }
    }))
    .unwrap()
}

/// A response whose usage object contains only unknown fields.
pub fn usage_only_unknown_fields() -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "x",
        "model": "m",
        "usage": { "some_new_field": 1, "another_unknown": "yes" }
    }))
    .unwrap()
}

/// A response with no usage object at all.
pub fn no_usage() -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "x",
        "model": "m",
        "choices": []
    }))
    .unwrap()
}

/// A response whose usage object is not an object.
pub fn malformed_usage() -> String {
    serde_json::to_string(&serde_json::json!({
        "id": "x",
        "model": "m",
        "usage": "not-an-object"
    }))
    .unwrap()
}
