//! Phase 0B experiment manifest (versioned).
//!
//! The manifest describes what a live run planned to do. It never contains
//! credentials.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// The Phase 0B experiment manifest format version.
///
/// v2: `max_input_tokens` renamed to `max_estimated_input_tokens` (the
/// value is a conservative local Prefixity estimate, not a provider
/// billing/tokenizer guarantee).
pub const EXPERIMENT_FORMAT_VERSION: u32 = 2;

/// Describes one Phase 0B experiment. Serialized to `manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExperimentManifest {
    /// Manifest format version.
    pub experiment_format_version: u32,
    /// Sanitized experiment identifier.
    pub experiment_id: String,
    /// Provider identifier (`openai`, `anthropic`, `deepseek`).
    pub provider: String,
    /// The explicit versioned API-surface usage schema (e.g.
    /// `openai-chat-completions-v1`). The provider name alone is not
    /// sufficient because one provider may expose different usage semantics
    /// across endpoints.
    pub api_surface: String,
    /// The allowlisted endpoint that was (or would be) called.
    pub endpoint: String,
    /// The exact provider model id used (never substituted).
    pub model: String,
    /// Scenario name.
    pub scenario: String,
    /// Seed for the deterministic synthetic prefix.
    pub stable_prefix_seed: u64,
    /// Approximate target prefix size in tokens.
    pub target_prefix_tokens: u64,
    /// Planned request count.
    pub request_count: usize,
    /// Creation time (ISO-8601 UTC).
    pub created_at: String,
    /// Prefixity commit SHA at run time, when resolvable locally.
    pub commit_sha: Option<String>,
    /// Optional experiment notes.
    pub notes: Option<String>,
    /// The `--max-requests` value in force.
    pub max_requests: usize,
    /// The `--max-estimated-input-tokens` safety ceiling in force
    /// (conservative local Prefixity estimate; not a provider
    /// billing/tokenizer guarantee).
    pub max_estimated_input_tokens: u64,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Input values needed to build a manifest. Kept as a struct to avoid a wide
/// argument list.
#[derive(Debug, Clone)]
pub struct ManifestInput {
    /// Sanitized experiment identifier.
    pub experiment_id: String,
    /// Provider identifier.
    pub provider: String,
    /// Versioned API-surface usage schema identifier.
    pub api_surface: String,
    /// Allowlisted endpoint URL.
    pub endpoint: String,
    /// Exact model id.
    pub model: String,
    /// Scenario name.
    pub scenario: String,
    /// Seed for the deterministic synthetic prefix.
    pub seed: u64,
    /// Approximate target prefix size in tokens.
    pub target_prefix_tokens: u64,
    /// Planned request count.
    pub request_count: usize,
    /// Optional notes.
    pub notes: Option<String>,
    /// `--max-requests` in force.
    pub max_requests: usize,
    /// `--max-estimated-input-tokens` safety ceiling (conservative local
    /// Prefixity estimate; not a provider billing/tokenizer guarantee).
    pub max_estimated_input_tokens: u64,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
}

/// Build a manifest from experiment configuration values.
pub fn build_manifest(input: &ManifestInput) -> ExperimentManifest {
    ExperimentManifest {
        experiment_format_version: EXPERIMENT_FORMAT_VERSION,
        experiment_id: input.experiment_id.clone(),
        provider: input.provider.clone(),
        api_surface: input.api_surface.clone(),
        endpoint: input.endpoint.clone(),
        model: input.model.clone(),
        scenario: input.scenario.clone(),
        stable_prefix_seed: input.seed,
        target_prefix_tokens: input.target_prefix_tokens,
        request_count: input.request_count,
        created_at: iso8601_utc_now(),
        commit_sha: detect_commit_sha(),
        notes: input.notes.clone(),
        max_requests: input.max_requests,
        max_estimated_input_tokens: input.max_estimated_input_tokens,
        timeout_ms: input.timeout_ms,
    }
}

/// Current time as an ISO-8601 UTC string.
pub fn iso8601_utc_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    iso8601_utc_from_unix(secs)
}

/// Format Unix seconds as ISO-8601 UTC (`YYYY-MM-DDTHH:MM:SSZ`).
pub fn iso8601_utc_from_unix(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (hour, minute, second) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Convert days since 1970-01-01 to (year, month, day) in the proleptic
/// Gregorian calendar (Howard Hinnant's `civil_from_days`).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Best-effort detection of the current commit SHA by reading `.git/HEAD`
/// and its referenced ref file directly (no shell invocation).
pub fn detect_commit_sha() -> Option<String> {
    let head = std::fs::read_to_string(".git/HEAD").ok()?;
    let head = head.trim();
    if let Some(path) = head.strip_prefix("ref: ") {
        std::fs::read_to_string(format!(".git/{path}"))
            .ok()
            .map(|s| s.trim().to_string())
    } else {
        Some(head.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso8601_known_values() {
        assert_eq!(iso8601_utc_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso8601_utc_from_unix(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(iso8601_utc_from_unix(1_752_000_000), "2025-07-08T18:40:00Z");
    }

    #[test]
    fn manifest_contains_no_credential_fields() {
        let manifest = build_manifest(&ManifestInput {
            experiment_id: "exp-1".to_string(),
            provider: "openai".to_string(),
            api_surface: "openai-chat-completions-v1".to_string(),
            endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
            model: "gpt-test".to_string(),
            scenario: "schema-smoke".to_string(),
            seed: 42,
            target_prefix_tokens: 8000,
            request_count: 1,
            notes: None,
            max_requests: 3,
            max_estimated_input_tokens: 50_000,
            timeout_ms: 60_000,
        });
        let json = serde_json::to_string(&manifest).unwrap();
        let lower = json.to_lowercase();
        assert!(!lower.contains("api_key"));
        assert!(!lower.contains("authorization"));
        assert!(!lower.contains("secret"));
        assert!(!lower.contains("credential"));
        assert_eq!(
            manifest.experiment_format_version,
            EXPERIMENT_FORMAT_VERSION
        );
        assert_eq!(manifest.api_surface, "openai-chat-completions-v1");
        assert!(manifest.endpoint.starts_with("https://"));
    }

    #[test]
    fn detect_commit_sha_is_optional() {
        // In a repo it resolves; in a bare environment it returns None.
        // Either way the call must not panic and must not contain a secret.
        let sha = detect_commit_sha();
        if let Some(value) = sha {
            assert!(!value.is_empty());
        }
    }
}
